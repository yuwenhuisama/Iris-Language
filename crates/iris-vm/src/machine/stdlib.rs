mod array;
mod hash_text;
mod iteration;
mod json;
mod support;

use iris_runtime::{ClassId, MethodOwner, Value, Visibility};

use super::{Machine, MachineError, VerifyError, selector_id, value_class_name};
use crate::compile::Program;

impl Machine {
    pub(super) fn binary_send(
        &mut self,
        selector: &str,
        receiver: Value,
        argument: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        // A contract VIEW is the value it wraps seen THROUGH a contract, so an
        // operator applies to that value: `(1 as N) == (1 as N)` is the Integer
        // comparison. The view exists to select which methods are visible, not
        // to change what the value is.
        let receiver = match receiver {
            Value::ContractView(inner, _) => *inner,
            other => other,
        };
        let argument = match argument {
            Value::ContractView(inner, _) => *inner,
            other => other,
        };
        // A REOPENED built-in class may redefine an operator, and `<` and `>`
        // are derived from `<=>` rather than being separate methods - so a
        // redefined `<=>` has to reach them, or `1 < 2` would keep answering
        // from the native comparison the reopen replaced.
        if !program.builtin_reopens.is_empty()
            && matches!(selector, "<" | "<=" | ">" | ">=" | "<=>" | "==" | "!=")
            && let Some(value) = self.reopened_builtin_send(
                &receiver,
                selector,
                std::slice::from_ref(&argument),
                program,
                classes,
            )?
        {
            return Ok(value);
        }
        if !program.builtin_reopens.is_empty()
            && matches!(selector, "<" | "<=" | ">" | ">=")
            && let Some(ordering) = self.reopened_builtin_send(
                &receiver,
                "<=>",
                std::slice::from_ref(&argument),
                program,
                classes,
            )?
        {
            // A reopened built-in orders by the same rule: no order at all is
            // a false comparison rather than a refusal.
            if ordering == Value::Nil {
                return Ok(Value::Bool(false));
            }
            let Value::Integer(ordering) = ordering else {
                return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
            };
            let ordering: i64 = ordering.decimal_text().parse().unwrap_or_default();
            return Ok(Value::Bool(match selector {
                "<" => ordering < 0,
                "<=" => ordering <= 0,
                ">" => ordering > 0,
                _ => ordering >= 0,
            }));
        }
        // A class may define an OPERATOR for its own instances, and an operator
        // arrives here rather than through `Send` - so authored dispatch has to
        // run, or `V.new() + 1` answers MessageNotFound for a method the class
        // plainly declares.
        if let Some(value) =
            self.authored_operator(&receiver, selector, &argument, program, classes)?
        {
            return Ok(value);
        }
        // `<` and `>` are DERIVED from `<=>` rather than being methods of their
        // own, so a class defining `<=>` gets them without writing them. The
        // derivation is here rather than at the class, because the ordering
        // has to be turned into a Bool after the authored body answers.
        if matches!(selector, "<" | "<=" | ">" | ">=")
            && let Some(ordering) =
                self.authored_operator(&receiver, "<=>", &argument, program, classes)?
        {
            // `C092` lets two values have NO ORDER, which `<=>` reports as
            // nil - and a comparison against no order is FALSE rather than a
            // type failure. Only a body answering some OTHER non-Integer is
            // the type failure the clause names.
            if ordering == Value::Nil {
                return Ok(Value::Bool(false));
            }
            // `D-094` separates two failures: another INTEGER satisfies the
            // broad `Integer?` return type but violates the protocol, which is
            // a ComparisonContractError, while a non-Integer, non-nil answer
            // violates the return type itself and is a type failure.
            let Value::Integer(ordering) = ordering else {
                return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
            };
            let ordering: i64 = match ordering.decimal_text().parse() {
                Ok(ordering @ -1..=1) => ordering,
                _ => return Err(MachineError::ComparisonContractError),
            };
            return Ok(Value::Bool(match selector {
                "<" => ordering < 0,
                "<=" => ordering <= 0,
                ">" => ordering > 0,
                _ => ordering >= 0,
            }));
        }
        // `C091` derives EQUALITY from `<=>` when a class defines one and no
        // `==` of its own: two objects are equal when they compare EQUAL, so
        // the authored body runs. Two references to ONE object are already
        // equal by identity, so nothing is consulted there - which is why the
        // body is called for a distinct pair and not for a self-comparison.
        if matches!(selector, "==" | "!=")
            && let (Value::Object(left), Value::Object(right)) = (&receiver, &argument)
            && left != right
            && let Some(ordering) =
                self.authored_operator(&receiver, "<=>", &argument, program, classes)?
        {
            let equal = matches!(&ordering, Value::Integer(ordering)
                if ordering.decimal_text() == "0");
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // `C092` gives every value a `<=>`: two values with no order answer
        // nil rather than refusing, and an OBJECT with no `<=>` of its own is
        // exactly that case. An iteration signal orders against its own kind.
        if selector == "<=>"
            && let Some(ordering) = self.default_ordering(&receiver, &argument, program, classes)?
        {
            return Ok(ordering);
        }
        // A MUTABLE string appends IN PLACE with `<<`, so every reference to it
        // sees the write - while `+` answers a NEW text, leaving the receiver
        // untouched. Both reached the kernel, which installs neither on this
        // family, so a program that plainly appends answered MessageNotFound.
        if let Value::MutableString(text) = &receiver
            && matches!(selector, "<<" | "+")
        {
            let addition = match &argument {
                Value::Text(addition) => addition.clone(),
                Value::MutableString(addition) => addition.text(),
                _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
            };
            if selector == "<<" {
                text.set(format!("{}{addition}", text.text()));
                return Ok(receiver.clone());
            }
            return Ok(Value::Text(format!("{}{addition}", text.text())));
        }
        // An IDENTITY-bearing value compares by WHICH value it is: two
        // iterators over one array are distinct, and each equals itself.
        if matches!(selector, "==" | "!=")
            && let Some(same) = identity_equality(&receiver, &argument)
        {
            return Ok(Value::Bool(if selector == "==" { same } else { !same }));
        }
        // A CLASS may define an operator on its singleton side, and an
        // operator arrives here rather than through `Send` - so authored
        // dispatch has to be consulted, or `P + P` would miss a `class fun +`
        // that `P.+(P)` finds. Only a class receiver takes this path: every
        // other authored receiver is already reached below.
        if let Value::Class(class) = &receiver
            && classes.iter().any(|known| known == class)
            && let Some(index) = classes.iter().position(|known| known == class)
            && program.classes[index]
                .class_methods
                .iter()
                .chain(
                    program.classes[index]
                        .reopens
                        .iter()
                        .flat_map(|reopen| &reopen.class_methods),
                )
                .any(|(name, _)| name == selector)
        {
            return self.class_method_value(
                *class,
                selector,
                std::slice::from_ref(&argument),
                program,
                classes,
            );
        }
        if let Value::Bytes(mut bytes) = receiver {
            if selector != "+" {
                return self.send(selector, Value::Bytes(bytes), &[argument]);
            }
            let addition = match argument {
                Value::Bytes(bytes) => bytes,
                Value::ByteArray(bytes) => bytes.bytes(),
                _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
            };
            bytes.extend_from_slice(&addition);
            return Ok(Value::Bytes(bytes));
        }
        if let Value::ByteArray(bytes) = receiver {
            if selector != "+" {
                return self.send(selector, Value::ByteArray(bytes), &[argument]);
            }
            let addition = match argument {
                Value::Bytes(bytes) => bytes,
                Value::ByteArray(bytes) => bytes.bytes(),
                _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
            };
            let mut joined = bytes.bytes();
            joined.extend_from_slice(&addition);
            return Ok(Value::ByteArray(iris_runtime::ByteArrayRef::new(joined)));
        }
        let Value::Text(text) = receiver else {
            return self.send(selector, receiver, &[argument]);
        };
        if selector != "+" {
            return self.send(selector, Value::Text(text), &[argument]);
        }
        let addition = self.text_operand(argument, program, classes)?;
        Ok(Value::Text(format!("{text}{addition}")))
    }

    pub(super) fn text_operand(
        &mut self,
        value: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<String, MachineError> {
        if let Some(text) = render_text(&value) {
            return Ok(text);
        }
        let Value::Object(object) = value else {
            return Err(MachineError::MessageNotFound {
                receiver_class: value_class_name(&value).to_owned(),
                selector: "to_string".to_owned(),
            });
        };
        let selector = selector_id(program, "to_string")
            .ok_or_else(|| MachineError::UnknownSelector("to_string".to_owned()))?;
        let method = self
            .runtime
            .dispatch_instance(object, selector)
            .map_err(MachineError::Construction)?;
        let function = usize::try_from(method.body().raw()).map_err(|_| {
            MachineError::Invalid(VerifyError::UnknownFunction {
                function: usize::MAX,
            })
        })?;
        let callee = program
            .functions
            .get(function)
            .cloned()
            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                function,
            }))?;
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            vec![Value::Object(object)],
            program,
            classes,
        )?;
        match returned.into_iter().next().unwrap_or(Value::Nil) {
            Value::Text(text) => Ok(text),
            _ => Err(MachineError::TypeContractError),
        }
    }

    /// Decides truth through `to_bool`, per `IRIS-V1-CONTROL-C022`.
    ///
    /// A class may DEFINE `to_bool`, and then its result is the answer, so
    /// truth cannot be read off the value's shape: `if p` must take the else
    /// branch for a `p` whose `to_bool` answers false. Only when no authored
    /// method answers does the default apply, where exactly `false` and `nil`
    /// are falsey. A non-Bool result is a `TypeContractError` rather than
    /// being coerced, which is what stops `to_bool` answering `0` from
    /// quietly meaning false.
    pub(super) fn test_truth(
        &mut self,
        value: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<bool, MachineError> {
        let method = match self.authored_to_bool(value, program, classes)? {
            Some(result) => iris_runtime::TruthinessMethod::Returns(result),
            None => iris_runtime::TruthinessMethod::Default,
        };
        iris_runtime::Truthiness::test(value, method).map_err(|error| match error {
            iris_runtime::TruthinessError::TypeContract => MachineError::TypeContractError,
            iris_runtime::TruthinessError::Raised(value) => {
                MachineError::Raised(Box::new((value, Value::Nil)))
            }
        })
    }

    /// Runs an object's OWN `to_bool`, or answers None when it has none.
    ///
    /// Only an Object can carry an authored method, and a class that does not
    /// define one must fall through to the default rather than failing: most
    /// values have no `to_bool` at all.
    fn authored_to_bool(
        &mut self,
        value: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let Value::Object(object) = value else {
            return Ok(None);
        };
        let Some(selector) = selector_id(program, "to_bool") else {
            return Ok(None);
        };
        let Ok(method) = self.runtime.dispatch_instance(*object, selector) else {
            // `C096` gives a REMOVED `to_bool` one last route: truth testing
            // invokes `method_missing(:to_bool, [], nil)` once and uses its
            // result. Only a selector the class actually BLOCKED takes it -
            // a value that never had `to_bool` at all falls through to the
            // `C094` default rather than reaching a handler.
            let class = self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction)?;
            let removed = self
                .runtime
                .registry()
                .active(class)
                .map(|revision| revision.tombstones().contains(&selector))
                .unwrap_or(false);
            if !removed {
                return Ok(None);
            }
            return self.invoke_method_missing(*object, "to_bool", &[], program, classes);
        };
        let Ok(function) = usize::try_from(method.body().raw()) else {
            return Ok(None);
        };
        let Some(callee) = program.functions.get(function).cloned() else {
            return Ok(None);
        };
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            vec![Value::Object(*object)],
            program,
            classes,
        )?;
        Ok(Some(returned.into_iter().next().unwrap_or(Value::Nil)))
    }

    pub(super) fn authored_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let result = match receiver {
            Value::Task(_) if selector == "class_name" && arguments.is_empty() => {
                Some(Value::Text("Task".to_owned()))
            }
            Value::Array(_)
            | Value::Hash(_)
            | Value::Range(_)
            | Value::MutableString(_)
            | Value::Bytes(_)
                if selector == "iterator" && arguments.is_empty() =>
            {
                self.open_builtin_iterator(receiver)?
            }
            Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::IterationYield(_)
            | Value::IterationDone => self.iteration_send(receiver, selector, arguments)?,
            Value::Array(values) => {
                self.array_send(values, receiver, selector, arguments, program, classes)?
            }
            Value::Hash(entries) => {
                self.hash_send(entries, selector, arguments, program, classes)?
            }
            // `IRIS-V1-COLLECTIONS-C082` makes `=~` and `!~` ordinary sends on
            // the SUBJECT, so a Regex operand arrives as an argument here.
            Value::Text(_) | Value::MutableString(_)
                if matches!(selector, "=~" | "!~") && matches!(arguments, [Value::Regex(_)]) =>
            {
                let subject = match receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
                };
                let [Value::Regex(regex)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Some(regex_match(&subject, regex, selector)?)
            }
            // `IRIS-V1-COLLECTIONS-C083` exposes the full match, both range
            // pairs, the Regex used, and the captures - with a group that did
            // not participate staying nil rather than an empty string.
            // A stored PROPERTY is written through its setter selector, `a.n = 5`
            // being a send of `n=` to the object. It is instance state rather
            // than a method, so the write lands in the slot the declaration
            // registered.
            Value::Object(object)
                if selector.ends_with('=')
                    && arguments.len() == 1
                    && classes
                        .iter()
                        .position(|known| {
                            self.runtime
                                .class_of(*object)
                                .is_ok_and(|held| held == *known)
                        })
                        .and_then(|index| program.classes.get(index))
                        .is_some_and(|declaration| {
                            declaration
                                .stored_properties
                                .iter()
                                .any(|property| property.name == selector.trim_end_matches('='))
                        }) =>
            {
                let name = selector.trim_end_matches('=');
                let Some(slot) = selector_id(program, name) else {
                    return Err(MachineError::UnknownSelector(name.to_owned()));
                };
                let [value] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                self.runtime
                    .assign_raw_ivar(*object, slot, value.clone())
                    .map_err(MachineError::Construction)?;
                Some(value.clone())
            }
            // A CLASS-level property is written through its setter selector,
            // `Cache.n = 9` being a send of `n=` to the Class. It is class
            // state rather than a method, so the write lands in the class
            // variable the declaration registered.
            Value::Class(class)
                if selector.ends_with('=')
                    && arguments.len() == 1
                    && selector_id(program, selector.trim_end_matches('=')).is_some_and(
                        |slot| {
                            self.runtime
                                .class_var(*class, slot)
                                .is_ok_and(|held| held.is_some())
                        },
                    ) =>
            {
                let name = selector.trim_end_matches('=');
                let Some(slot) = selector_id(program, name) else {
                    return Err(MachineError::UnknownSelector(name.to_owned()));
                };
                let [value] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                self.runtime
                    .assign_class_var(*class, slot, value.clone())
                    .map_err(MachineError::Construction)?;
                Some(value.clone())
            }
            // `C030` makes a second close a NO-OP rather than a second
            // release, and the counter is how a caller proves that - so it is
            // readable rather than internal.
            Value::NativeResource(_) => match (selector, arguments) {
                ("close", []) => {
                    let status = iris_abi::iris_payload_close();
                    if !matches!(
                        status,
                        iris_abi::IrisStatus::Success | iris_abi::IrisStatus::InvalidHandle
                    ) {
                        return Err(MachineError::UnsupportedConstruct);
                    }
                    Some(Value::Nil)
                }
                ("releases", []) => Some(Value::Integer(
                    u64::from(iris_abi::iris_payload_release_count()).into(),
                )),
                _ => None,
            },
            Value::Library(library) => Self::library_send(library, selector, arguments)?,
            Value::Match(matched) => match (selector, arguments) {
                ("text" | "to_string", []) => Some(Value::Text(matched.text.clone())),
                ("regex", []) => Some(Value::Regex(Box::new(matched.regex.clone()))),
                ("byte_start", []) => Some(Value::Integer(
                    u64::try_from(matched.byte_start).unwrap_or_default().into(),
                )),
                ("byte_end", []) => Some(Value::Integer(
                    u64::try_from(matched.byte_end).unwrap_or_default().into(),
                )),
                ("start", []) => Some(Value::Integer(
                    u64::try_from(matched.scalar_start)
                        .unwrap_or_default()
                        .into(),
                )),
                ("end", []) => Some(Value::Integer(
                    u64::try_from(matched.scalar_end).unwrap_or_default().into(),
                )),
                ("capture", [index]) => {
                    let found = match index {
                        Value::Integer(index) => index
                            .to_usize()
                            .and_then(|index| index.checked_sub(1))
                            .and_then(|index| matched.captures.get(index).cloned())
                            .flatten(),
                        Value::Symbol(name) | Value::Text(name) => matched
                            .named
                            .iter()
                            .find(|(known, _)| known == name)
                            .and_then(|(_, value)| value.clone()),
                        _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
                    };
                    Some(found.map_or(Value::Nil, Value::Text))
                }
                _ => None,
            },
            // `append` converts its operand through `to_string` and mutates
            // IN PLACE, so every reference to the string sees the write. A
            // conversion that RAISES leaves the receiver untouched, which is
            // what makes a failed append observable as no change at all.
            Value::MutableString(text) if selector == "append" && arguments.len() == 1 => {
                let addition = self.text_operand(arguments[0].clone(), program, classes)?;
                text.set(format!("{}{addition}", text.text()));
                Some(receiver.clone())
            }
            // `C042` pins case mapping to a fixed Unicode data version rather
            // than a host locale. The PLAIN spelling answers a NEW mutable
            // string and leaves the receiver alone, while the `!` spelling
            // commits ATOMICALLY in place - `C058` builds the whole
            // replacement before the receiver is touched - and answers the
            // receiver itself, which is what `same?` observes.
            Value::MutableString(text)
                if matches!(selector, "upcase" | "downcase" | "upcase!" | "downcase!")
                    && arguments.is_empty() =>
            {
                let current = text.text();
                let changed = if selector.starts_with("upcase") {
                    current.to_uppercase()
                } else {
                    current.to_lowercase()
                };
                if selector.ends_with('!') {
                    text.set(changed);
                    Some(receiver.clone())
                } else {
                    Some(Value::MutableString(iris_runtime::MutableStringRef::new(
                        changed,
                    )))
                }
            }
            // `clear` empties the content and answers the RECEIVER, which is
            // what `same?` observes - a fresh empty string would compare
            // false against the one the program still holds.
            Value::MutableString(text) if selector == "clear" && arguments.is_empty() => {
                text.set(String::new());
                Some(receiver.clone())
            }
            // `C061` makes the byte view a LIVE Iterable over the receiver
            // rather than a detached Array, so a later content change
            // invalidates an active cursor.
            Value::MutableString(_) if selector == "bytes" && arguments.is_empty() => {
                Some(receiver.clone())
            }
            // `replace` sets the WHOLE content and answers the receiver, so
            // every reference to the string sees the new text.
            Value::MutableString(text) if selector == "replace" && arguments.len() == 1 => {
                let replacement = self.text_operand(arguments[0].clone(), program, classes)?;
                text.set(replacement);
                Some(receiver.clone())
            }
            // `IRIS-V1-COLLECTIONS-C042` pins normalization and case folding to
            // a fixed Unicode data version rather than a host locale, and
            // `C044` exposes GRAPHEME CLUSTERS explicitly because `length`
            // counts scalars and one cluster may span several of them.
            Value::Text(_) | Value::MutableString(_)
                if matches!(selector, "casefold" | "nfc" | "nfd" | "graphemes")
                    && arguments.is_empty() =>
            {
                let text = match receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
                };
                Some(match selector {
                    "casefold" => Value::Text(
                        icu_casemap::CaseMapper::new()
                            .fold_string(&text)
                            .into_owned(),
                    ),
                    "nfc" => {
                        use unicode_normalization::UnicodeNormalization;
                        Value::Text(text.nfc().collect())
                    }
                    "nfd" => {
                        use unicode_normalization::UnicodeNormalization;
                        Value::Text(text.nfd().collect())
                    }
                    _ => Value::Array(iris_runtime::ArrayRef::new(
                        unicode_segmentation::UnicodeSegmentation::graphemes(text.as_str(), true)
                            .map(|cluster| Value::Text(cluster.to_owned()))
                            .collect(),
                    )),
                })
            }
            // `C051` makes `to_array` the ordered element sequence, which a
            // Range answers through the SAME materialization iterating it
            // uses - the two cannot disagree about which values it holds.
            Value::Range(range) if selector == "to_array" && arguments.is_empty() => Some(
                Value::Array(iris_runtime::ArrayRef::new(iteration::range_values(range)?)),
            ),
            // `C051` makes `to_array` the ordered element sequence for every
            // sequence-shaped receiver, so a Tuple and a byte string
            // materialize the same way a Range does rather than answering a
            // missing message.
            Value::Tuple(values) if selector == "to_array" && arguments.is_empty() => {
                Some(Value::Array(iris_runtime::ArrayRef::new(values.clone())))
            }
            Value::Bytes(bytes) if selector == "to_array" && arguments.is_empty() => {
                Some(Value::Array(iris_runtime::ArrayRef::new(
                    bytes
                        .iter()
                        .map(|byte| Value::Integer(u64::from(*byte).into()))
                        .collect(),
                )))
            }
            Value::ByteArray(bytes) if selector == "to_array" && arguments.is_empty() => {
                Some(Value::Array(iris_runtime::ArrayRef::new(
                    bytes
                        .bytes()
                        .iter()
                        .map(|byte| Value::Integer(u64::from(*byte).into()))
                        .collect(),
                )))
            }
            // `C038` spells the operand `range.by(step: Integer)`, so the
            // stride arrives as a KEYWORD argument rather than a bare
            // positional one.
            Value::Range(range) if selector == "by" && arguments.len() == 1 => {
                let step = match &arguments[0] {
                    Value::KeywordArgument(name, value) if name == "step" => value.as_ref(),
                    value => value,
                };
                let Value::Integer(step) = step else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                let Some(magnitude) = step.to_i128() else {
                    return Err(MachineError::RangeError);
                };
                // A zero step never advances, and a step whose SIGN walks away
                // from the end never arrives: both are refused up front rather
                // than looping forever.
                if magnitude == 0 {
                    return Err(MachineError::RangeError);
                }
                let descending = iris_runtime::Numeric::compare(
                    &iris_runtime::NumericValue::Integer(range.end.clone()),
                    &iris_runtime::NumericValue::Integer(range.start.clone()),
                ) == Some(std::cmp::Ordering::Less);
                if (descending && magnitude > 0) || (!descending && magnitude < 0) {
                    return Err(MachineError::RangeError);
                }
                Some(Value::Range(Box::new(iris_runtime::RangeValue {
                    start: range.start.clone(),
                    end: range.end.clone(),
                    inclusive_end: range.inclusive_end,
                    step: step.clone(),
                })))
            }
            Value::Text(text) => hash_text::text_send(text, selector, arguments),
            Value::Integer(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(value.decimal_text()))
            }
            Value::Bool(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(value.to_string()))
            }
            Value::Nil if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text("nil".to_owned()))
            }
            // An OBJECT hashes by IDENTITY, which survives a relocation: that
            // is what makes a retained object's hash stable across a
            // collection while two distinct objects differ.
            Value::Object(object) if selector == "hash" && arguments.is_empty() => {
                let hash = self
                    .runtime
                    .identity_hash(*object)
                    .map_err(|_| MachineError::SerializationError)?;
                Some(Value::Integer(hash.into()))
            }
            // `D-142` lets user code ITERATE and COPY a runtime-owned
            // collection but never insert, delete, replace or reorder it, so
            // a mutating selector is refused rather than reaching the
            // ordinary Array path that would happily perform it.
            Value::ReadonlyArray(_)
                if matches!(
                    selector,
                    "append"
                        | "push"
                        | "delete"
                        | "clear"
                        | "insert"
                        | "[]="
                        | "reverse!"
                        | "sort!"
                ) =>
            {
                return Err(MachineError::ReadonlyMutation);
            }
            // `D-159` makes an exception CONTEXT readable but never
            // assignable, so a write names the READONLY property rather than
            // a setter the language never had.
            Value::ExceptionContext(..) if selector.ends_with('=') => {
                return Err(MachineError::ReadonlyProperty);
            }
            // A context member is read as a bare MEMBER and as a CALL alike,
            // so the called form answers the same value rather than reporting
            // a selector the language plainly defines.
            Value::ExceptionContext(_, value, cause, suppressed, sites, location)
                if arguments.is_empty() =>
            {
                match selector {
                    "value" => Some((**value).clone()),
                    "cause" => Some((**cause).clone()),
                    "suppressed" => Some(Value::ReadonlyArray(suppressed.clone())),
                    "re_raise_sites" => Some(Value::ReadonlyArray(sites.clone())),
                    "original_stack" => Some(Value::ReadonlyArray(Vec::new())),
                    "raise_location" => Some((**location).clone()),
                    _ => None,
                }
            }
            // A CONTRACT is interned once per definition, so its hash is fixed
            // by identity: `C.hash() == C.hash()` holds because both name the
            // same contract.
            Value::Contract(contract) if selector == "hash" && arguments.is_empty() => {
                Some(Value::Integer(contract.raw().into()))
            }
            // A MutableString answers its CURRENT content, so a read after a
            // write sees the replacement rather than the text the value was
            // built with.
            // `C068` renders a byte string as TEXT when its bytes are valid
            // UTF-8, which is what `to_string` answers - the kernel installs
            // it on String alone, so a byte string reported the selector
            // absent for a rendering the language defines.
            Value::Bytes(bytes) if selector == "to_string" && arguments.is_empty() => {
                match core::str::from_utf8(bytes) {
                    Ok(text) => Some(Value::Text(text.to_owned())),
                    // Bytes that are not valid UTF-8 name no text, which is an
                    // ENCODING failure rather than a type mismatch: the
                    // receiver is the right kind, its content simply does not
                    // decode.
                    Err(_) => return Err(MachineError::EncodingError),
                }
            }
            Value::ByteArray(bytes) if selector == "to_string" && arguments.is_empty() => {
                match String::from_utf8(bytes.bytes()) {
                    Ok(text) => Some(Value::Text(text)),
                    Err(_) => return Err(MachineError::EncodingError),
                }
            }
            Value::MutableString(text) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(text.text()))
            }
            // `C067` makes Bytes the IMMUTABLE form, so a ByteArray answers a
            // snapshot of its current content rather than a view onto it - a
            // later mutation must not be visible through the answer.
            Value::ByteArray(bytes) if selector == "to_bytes" && arguments.is_empty() => {
                Some(Value::Bytes(bytes.bytes()))
            }
            Value::Bytes(bytes) if selector == "to_bytes" && arguments.is_empty() => {
                Some(Value::Bytes(bytes.clone()))
            }
            // These families answer `length` in the reference, and a value the
            // backend can PRODUCE but not measure is the defect that has
            // recurred here: a suppressed list handed to a catch, or a byte
            // literal, is useless if a program cannot ask how long it is.
            Value::Bytes(bytes) if selector == "length" && arguments.is_empty() => Some(
                Value::Integer(iris_runtime::IntegerValue::from(bytes.len() as u64)),
            ),
            Value::ByteArray(bytes) if selector == "length" && arguments.is_empty() => Some(
                Value::Integer(iris_runtime::IntegerValue::from(bytes.bytes().len() as u64)),
            ),
            Value::ReadonlyArray(values) if selector == "length" && arguments.is_empty() => Some(
                Value::Integer(iris_runtime::IntegerValue::from(values.len() as u64)),
            ),
            Value::Symbol(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(value.clone()))
            }
            Value::Float64(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(float_text(*value)))
            }
            Value::Float32(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(float_text(f64::from(*value))))
            }
            // C017 numbers the origin revision 1 and gives the next per-Class
            // integer to each successful structural publication. The Class
            // answered `contracts` but not this, so a program that verified
            // died in the machine on a property the reference plainly has.
            Value::Class(class) if selector == "active_revision" && arguments.is_empty() => {
                let revision = self
                    .runtime
                    .registry()
                    .active(*class)
                    .map_err(MachineError::Class)?;
                Some(Value::Integer(iris_runtime::IntegerValue::from(
                    revision.number(),
                )))
            }
            // A class RECOMPOSES its module edges: `add_module` includes one
            // and `remove_module` drops it, both against the transaction
            // candidate rather than the published revision.
            Value::Class(class)
                if matches!(selector, "add_module" | "remove_module")
                    && matches!(arguments, [Value::Symbol(_)]) =>
            {
                let [Value::Symbol(name)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                let Some((_, module)) = self.modules.iter().find(|(known, _)| known == name) else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                let module = *module;
                self.runtime
                    .registry_mut()
                    .recompose_candidate(*class, module, selector == "add_module")
                    .map_err(MachineError::Class)?;
                Some(Value::Nil)
            }
            // `C119` makes the direct and the reflective entry points ONE
            // implementation, so both arrive here. A contract the class
            // DECLARED is part of its static spine and cannot be dropped;
            // removing one it never declared changes no such fact, so that is
            // a no-op rather than a refusal.
            Value::Class(class) if selector == "remove_contract" => {
                let [Value::Contract(contract)] = arguments else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                let declared =
                    classes
                        .iter()
                        .position(|known| known == class)
                        .is_some_and(|index| {
                            program.classes[index]
                                .contracts
                                .iter()
                                .any(|known| *known as u64 + 1 == contract.raw())
                        });
                if declared {
                    return Err(MachineError::TypeContractError);
                }
                Some(Value::Nil)
            }
            // `C023` lets a class RESHAPE its own method set: an alias binds a
            // second selector to one method, `remove_method` drops the class's
            // own, and `undef_method` blocks the selector outright so an
            // inherited one no longer answers either. Each names the class it
            // acts on, so they reach the registry directly.
            Value::Class(class)
                if matches!(selector, "alias_method")
                    && matches!(arguments, [Value::Symbol(_), Value::Symbol(_)]) =>
            {
                let [Value::Symbol(alias), Value::Symbol(original)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                let (Some(alias), Some(original)) =
                    (selector_id(program, alias), selector_id(program, original))
                else {
                    return Err(MachineError::UnknownSelector(alias.clone()));
                };
                self.runtime
                    .registry_mut()
                    .alias_method(*class, alias, original)
                    .map_err(MachineError::Class)?;
                Some(Value::Nil)
            }
            Value::Class(class)
                if matches!(selector, "remove_method" | "undef_method")
                    && matches!(arguments, [Value::Symbol(_)]) =>
            {
                let [Value::Symbol(name)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                let Some(slot) = selector_id(program, name) else {
                    return Err(MachineError::UnknownSelector(name.clone()));
                };
                if selector == "remove_method" {
                    self.runtime
                        .registry_mut()
                        .remove_method(*class, slot)
                        .map_err(MachineError::Class)?;
                } else {
                    self.runtime
                        .registry_mut()
                        .undef_method(*class, slot)
                        .map_err(MachineError::Class)?;
                }
                Some(Value::Nil)
            }
            // `C119` makes `A.method(:f)` and the reflective call ONE surface,
            // so the direct form answers the same Method value rather than
            // reporting the selector absent.
            Value::Class(class)
                if selector == "method" && matches!(arguments, [Value::Symbol(_)]) =>
            {
                let [Value::Symbol(name)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                };
                let Some(slot) = selector_id(program, name) else {
                    return Err(MachineError::UnknownSelector(name.clone()));
                };
                match self.runtime.registry().dispatch(*class, slot) {
                    Ok(iris_runtime::DispatchOutcome::Invoke(method)) => {
                        Some(Value::Method(method))
                    }
                    _ => Some(Value::Nil),
                }
            }
            Value::Class(class) if selector == "contracts" && arguments.is_empty() => {
                let declared = classes
                    .iter()
                    .position(|known| known == class)
                    .map(|index| {
                        program.classes[index]
                            .contracts
                            .iter()
                            .map(|contract| {
                                Value::Contract(iris_runtime::ContractId::new(*contract as u64 + 1))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Some(Value::Array(iris_runtime::ArrayRef::new(declared)))
            }
            Value::Method(method) => match selector {
                "selector" if arguments.is_empty() => Some(
                    self.selector_name(program, method.selector())
                        .map(Value::Symbol)
                        .unwrap_or(Value::Nil),
                ),
                "owner" if arguments.is_empty() => Some(match method.owner() {
                    MethodOwner::Class(class) => Value::Class(class),
                    MethodOwner::Module(_) => Value::Nil,
                }),
                "visibility" if arguments.is_empty() => Some(Value::Symbol(
                    match method.visibility() {
                        Visibility::Public => "public",
                        Visibility::Protected => "protected",
                        Visibility::Private => "private",
                    }
                    .to_owned(),
                )),
                "parameters" if arguments.is_empty() => {
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let metadata = program
                        .functions
                        .get(function)
                        .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                            function,
                        }))?;
                    Some(Value::Array(iris_runtime::ArrayRef::new(
                        metadata
                            .parameter_types
                            .iter()
                            .cloned()
                            .map(Value::Symbol)
                            .collect(),
                    )))
                }
                "return_type" if arguments.is_empty() => {
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let metadata = program
                        .functions
                        .get(function)
                        .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                            function,
                        }))?;
                    Some(Value::Symbol(metadata.return_type.clone()))
                }
                "source" if arguments.is_empty() => {
                    let MethodOwner::Class(owner) = method.owner() else {
                        return Ok(Some(Value::Symbol("dynamic-only".to_owned())));
                    };
                    let revision = self
                        .runtime
                        .registry()
                        .active(owner)
                        .map_err(MachineError::Class)?;
                    Some(Value::Array(iris_runtime::ArrayRef::new(vec![
                        Value::Symbol("runtime-local".to_owned()),
                        Value::Integer(revision.number().into()),
                        Value::Integer(revision.commit_id().into()),
                        Value::Symbol("static".to_owned()),
                    ])))
                }
                "bind" => {
                    let [target] = arguments else {
                        return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
                    };
                    let bound = match target {
                        Value::Object(object) => self
                            .runtime
                            .registry_mut()
                            .bind_retained_instance(*object, *method),
                        Value::Class(class) => self
                            .runtime
                            .registry_mut()
                            .bind_retained_class(*class, *method),
                        _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
                    }
                    .map_err(iris_runtime::ConstructionError::from)
                    .map_err(MachineError::Construction)?;
                    Some(Value::BoundMethod(bound))
                }
                _ => None,
            },
            // `IDENTITY-C030` gives a script RUNTIME-LOCAL package identity, so
            // a Type names that package rather than a publishable one.
            Value::Type(..) if selector == "package" && arguments.is_empty() => {
                Some(Value::Symbol("runtime-local".to_owned()))
            }
            // `C078` derives a nominal Type's PUBLISHABLE identity hash from
            // the package, the qualified name and the major API version, so
            // two Types with identical declarations stay distinct and moving
            // one between packages changes its identity. An identity hash
            // would answer a fresh number per read instead.
            Value::Type(class, _) if selector == "hash" && arguments.is_empty() => {
                let name = classes
                    .iter()
                    .position(|known| known == class)
                    .and_then(|index| program.classes.get(index))
                    .map(|declaration| declaration.name.clone())
                    .unwrap_or_default();
                Some(Value::Integer(iris_runtime::contract_type_hash(
                    "runtime-local",
                    &name,
                    1,
                )))
            }
            // A NOMINAL type answers the type ARGUMENTS it was closed over,
            // which is empty for a plain class - answering nothing at all made
            // a selector the language plainly defines look absent.
            Value::Type(_, type_arguments) if selector == "arguments" && arguments.is_empty() => {
                Some(Value::Array(iris_runtime::ArrayRef::new(
                    type_arguments
                        .iter()
                        .map(|argument| Value::Type(*argument, Vec::new()))
                        .collect(),
                )))
            }
            Value::Type(class, _) if selector == "kind" && arguments.is_empty() => {
                Some(Value::Symbol("nominal".to_owned()))
            }
            value @ (Value::Type(..) | Value::ComposedType(_))
                if selector == "type" && arguments.is_empty() =>
            {
                Some(value.clone())
            }
            // A COMPOSED type answers the atoms it was built from, so
            // `(A & (B | C)).type.members[1]` names the inner union itself
            // rather than flattening it away.
            // A NOMINAL type answers the SELECTORS its class defines, which is
            // the called form of the same member read - answering only for a
            // composed type left `A.type.members()` reporting a selector the
            // language plainly defines as present.
            Value::Type(class, _) if selector == "members" && arguments.is_empty() => {
                let selectors: Vec<_> = self
                    .runtime
                    .registry()
                    .active(*class)
                    .map_err(MachineError::Class)?
                    .methods()
                    .keys()
                    .copied()
                    .collect();
                Some(Value::Array(iris_runtime::ArrayRef::new(
                    selectors
                        .into_iter()
                        .map(|selector| {
                            self.selector_name(program, selector)
                                .map_or(Value::Nil, Value::Symbol)
                        })
                        .collect(),
                )))
            }
            Value::ComposedType(form) if selector == "members" && arguments.is_empty() => {
                let members = match form {
                    iris_runtime::ComposedType::Never => Vec::new(),
                    iris_runtime::ComposedType::Union(members)
                    | iris_runtime::ComposedType::Intersection(members) => {
                        members.iter().map(reflect_atom).collect()
                    }
                };
                Some(Value::Array(iris_runtime::ArrayRef::new(members)))
            }
            Value::ComposedType(form) if selector == "kind" && arguments.is_empty() => {
                Some(Value::Symbol(
                    match form {
                        iris_runtime::ComposedType::Never => "never",
                        iris_runtime::ComposedType::Union(_) => "union",
                        iris_runtime::ComposedType::Intersection(_) => "intersection",
                    }
                    .to_owned(),
                ))
            }
            Value::Type(left, _) if selector == "subtype?" => {
                let [Value::Type(right, _)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Some(Value::Bool(self.is_subtype(*left, *right)?))
            }
            Value::Type(target, _) if selector == "assignable?" => {
                let [Value::Type(source, _)] = arguments else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Some(Value::Bool(self.is_subtype(*source, *target)?))
            }
            _ => None,
        };
        // A USER-DEFINED receiver dispatches through its own class, so an
        // authored name it happens to share is not a refusal - it simply is
        // not the authored surface's business. Refusing here made every
        // authored spelling unusable as a method name: `class C { fun first()
        // }` reported MessageNotFound for a method the class plainly declares.
        //
        // The refusal still applies to the BUILT-IN families, where an
        // authored name that answered nothing means the selector genuinely is
        // absent - that is what pins Array `size` as MessageNotFound rather
        // than letting it fall through to a generic dispatch error.
        // A REOPENED built-in class answers a method the native surface never
        // had, so the class's own dispatch is consulted before the selector is
        // called absent: `open class String { fun shout() }` must reach every
        // String value, not a new class.
        if result.is_none()
            && !program.builtin_reopens.is_empty()
            && !matches!(receiver, Value::Object(_) | Value::Class(_))
            && let Some(value) =
                self.reopened_builtin_send(receiver, selector, arguments, program, classes)?
        {
            return Ok(Some(value));
        }
        if result.is_none()
            && authored_selector(selector)
            && !matches!(receiver, Value::Object(_) | Value::Class(_))
        {
            return Err(MachineError::MessageNotFound {
                receiver_class: value_class_name(receiver).to_owned(),
                selector: selector.to_owned(),
            });
        }
        Ok(result)
    }
}

fn authored_selector(selector: &str) -> bool {
    matches!(
        selector,
        "length"
            | "size"
            | "map"
            | "each"
            | "each_with_index"
            | "select"
            | "reject"
            | "reduce"
            | "find"
            | "count"
            | "sum"
            | "min"
            | "max"
            | "sort"
            | "push"
            | "append"
            | "clear"
            | "insert"
            | "pop"
            | "join"
            | "first"
            | "last"
            | "reverse"
            | "at"
            | "to_string"
            | "include?"
            | "index_of"
            | "concat"
            | "slice"
            | "take"
            | "drop"
            | "uniq"
            | "flatten"
            | "all?"
            | "any?"
            | "keys"
            | "values"
            | "merge"
            | "has_key?"
            | "delete"
            | "to_array"
            | "split"
            | "trim"
            | "replace"
            | "starts_with?"
            | "ends_with?"
            | "contains?"
            | "downcase"
            | "chars"
            | "to_symbol"
            | "contracts"
            | "kind"
            | "subtype?"
            | "assignable?"
            | "selector"
            | "owner"
            | "visibility"
            | "parameters"
            | "return_type"
            | "source"
            | "bind"
    )
}

pub(super) fn render_text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        Value::Integer(value) => Some(value.decimal_text()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Nil => Some("nil".to_owned()),
        Value::Symbol(value) => Some(value.clone()),
        // Both widths answer text so a float can be printed and interpolated.
        // The reference renders an integral value with a trailing `.0`, which
        // keeps `1.0` distinguishable from the Integer `1` in the text.
        Value::Float64(value) => Some(float_text(*value)),
        Value::Float32(value) => Some(float_text(f64::from(*value))),
        _ => None,
    }
}

/// Renders a float exactly as the reference `to_string` does.
fn float_text(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "Infinity".to_owned()
        } else {
            "-Infinity".to_owned()
        };
    }
    let rendered = format!("{value}");
    if rendered.contains(['.', 'e', 'E']) {
        rendered
    } else {
        format!("{rendered}.0")
    }
}

/// Runs `=~` or `!~`, per `IRIS-V1-COLLECTIONS-C082` and `C083`.
///
/// `!~` is true exactly when `=~` would answer nil, and a successful `=~`
/// answers a Match carrying the full text, BOTH its byte and scalar ranges,
/// and its captures - where a group that did not participate stays absent
/// rather than becoming an empty string.
fn regex_match(
    subject: &str,
    regex: &iris_runtime::RegexValue,
    selector: &str,
) -> Result<Value, MachineError> {
    let compiled = regex::RegexBuilder::new(&regex.pattern)
        .case_insensitive(regex.flags.contains('i'))
        .multi_line(regex.flags.contains('m'))
        .dot_matches_new_line(regex.flags.contains('s'))
        .ignore_whitespace(regex.flags.contains('x'))
        .unicode(true)
        .build()
        .map_err(|_| MachineError::Kernel(iris_runtime::KernelError::Type))?;
    let found = compiled.captures(subject);
    if selector == "!~" {
        return Ok(Value::Bool(found.is_none()));
    }
    let Some(found) = found else {
        return Ok(Value::Nil);
    };
    let Some(whole) = found.get(0) else {
        return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
    };
    // C083 exposes SCALAR ranges alongside byte ranges, so they are counted
    // rather than assumed equal - they differ for any non-ASCII subject.
    let scalar_start = subject[..whole.start()].chars().count();
    let scalar_end = scalar_start + whole.as_str().chars().count();
    let captures = (1..compiled.captures_len())
        .map(|index| found.get(index).map(|group| group.as_str().to_owned()))
        .collect();
    let named = compiled
        .capture_names()
        .flatten()
        .map(|name| {
            (
                name.to_owned(),
                found.name(name).map(|group| group.as_str().to_owned()),
            )
        })
        .collect();
    Ok(Value::Match(Box::new(iris_runtime::MatchValue {
        text: whole.as_str().to_owned(),
        byte_start: whole.start(),
        byte_end: whole.end(),
        scalar_start,
        scalar_end,
        captures,
        named,
        regex: regex.clone(),
    })))
}

impl Machine {
    /// Encodes a value as `IrisValue` data, per `IRIS-V1-LIBRARY-C018`.
    ///
    /// An OBJECT is asked for its own representation through `serialize`, and
    /// only a class declaring `Serializable` may answer one. `C003` keeps a
    /// live resource out of the stream rather than emitting its identity, so
    /// an unsupported family is refused instead of being rendered some other
    /// way.
    pub(super) fn irisvalue_encode(
        &mut self,
        value: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let representation = match &value {
            Value::Object(_) => {
                if !self.declares_serializable(&value, program, classes) {
                    return Err(MachineError::SerializationError);
                }
                self.instance_method_value(&value, "serialize", &[], program, classes)?
            }
            _ => value,
        };
        Self::check_encodable(&representation)?;
        Ok(representation)
    }

    /// Decodes an `IrisValue` stream, validating the header FIRST.
    ///
    /// `C016` checks magic and format version before decoding any payload that
    /// depends on them, and `C017` forbids allocating from a DECLARED length
    /// before that length is validated. `C020` routes a NOMINAL value through
    /// the class's own factory, and a stream naming no nominal class stays
    /// ordinary decoded data - `C010` refuses to instantiate classes from type
    /// names by default.
    pub(super) fn irisvalue_decode(
        &mut self,
        stream: Value,
        options: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Hash(stream) = stream else {
            return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
        };
        let field = |name: &str| stream.get(&Value::Text(name.to_owned()));
        if field("magic") != Some(Value::Text("IRISVALUE".to_owned()))
            || field("format_version") != Some(Value::Integer(1_u8.into()))
        {
            return Err(MachineError::LexicalDiagnostic(
                "IRISVALUE_INCOMPATIBLE_HEADER",
            ));
        }
        let limit = options
            .iter()
            .find_map(|option| match option {
                Value::KeywordArgument(name, limit) if name == "element_limit" => match &**limit {
                    Value::Integer(limit) => limit.to_usize(),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or(1024);
        if let Some(Value::Integer(declared)) = field("element_count")
            && declared.to_usize().is_none_or(|declared| declared > limit)
        {
            return Err(MachineError::LexicalDiagnostic(
                "IRISVALUE_LIMIT_OR_STRUCTURE",
            ));
        }
        let payload = field("payload").unwrap_or(Value::Nil);
        let Some(Value::Symbol(nominal)) = field("nominal") else {
            return Ok(payload);
        };
        self.decode_nominal(&nominal, field("schema_version"), payload, program, classes)
    }

    /// Rebuilds a NOMINAL value through the class's declared factory.
    fn decode_nominal(
        &mut self,
        nominal: &str,
        schema: Option<Value>,
        payload: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Some(index) = program
            .classes
            .iter()
            .position(|declaration| declaration.name == nominal)
        else {
            return Err(MachineError::SerializationError);
        };
        let Some(class) = classes.get(index).copied() else {
            return Err(MachineError::SerializationError);
        };
        // `C006` validates conformance BEFORE publishing, so a class that does
        // not declare `Serializable` is refused even though it has a factory:
        // otherwise any class with a `deserialize` could be built from a stream.
        if !program
            .contracts
            .iter()
            .position(|contract| contract.name == "Serializable")
            .is_some_and(|wanted| program.classes[index].contracts.contains(&wanted))
        {
            return Err(MachineError::SerializationError);
        }
        // Only schema version 1 is defined, so a stream declaring another is
        // refused rather than guessed at.
        let schema_ok = match schema {
            None => true,
            Some(Value::Integer(ref version)) => version.to_usize() == Some(1),
            Some(_) => false,
        };
        if !schema_ok {
            return Err(MachineError::LexicalDiagnostic(
                "IRISVALUE_INCOMPATIBLE_HEADER",
            ));
        }
        self.class_method_value(class, "deserialize", &[payload], program, classes)
    }

    /// Calls an authored INSTANCE method and answers its value.
    ///
    /// `authored_send` answers the BUILT-IN surface only, so a class's own
    /// `serialize` is not reachable through it: `C005` makes the
    /// representation ordinary Iris data the CLASS chooses, which means the
    /// object has to be asked rather than inspected.
    fn instance_method_value(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Object(object) = receiver else {
            return Err(MachineError::SerializationError);
        };
        let Some(slot) = selector_id(program, selector) else {
            return Err(MachineError::SerializationError);
        };
        let Ok(method) = self.runtime.dispatch_instance(*object, slot) else {
            return Err(MachineError::SerializationError);
        };
        let Ok(function) = usize::try_from(method.body().raw()) else {
            return Err(MachineError::SerializationError);
        };
        let Some(callee) = program.functions.get(function).cloned() else {
            return Err(MachineError::SerializationError);
        };
        let mut passed = vec![Value::Object(*object)];
        passed.extend_from_slice(arguments);
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            passed,
            program,
            classes,
        )?;
        Ok(returned.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Calls an authored CLASS method and answers its value.
    pub(super) fn class_method_value(
        &mut self,
        class: ClassId,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return Err(MachineError::SerializationError);
        };
        // Which BODY runs is the registry's answer, not this table's: a reopen
        // takes effect at the position it was written, so a static "last
        // definition wins" would run the replacement even for a call made
        // before it. The declaration is consulted only to know the selector is
        // a class method at all.
        if !program.classes[index]
            .class_methods
            .iter()
            .chain(
                program.classes[index]
                    .reopens
                    .iter()
                    .flat_map(|reopen| &reopen.class_methods),
            )
            .any(|(name, _)| name == selector)
        {
            return Err(MachineError::SerializationError);
        }
        let selector_id = selector_id(program, selector)
            .ok_or_else(|| MachineError::UnknownSelector(selector.to_owned()))?;
        let iris_runtime::DispatchOutcome::Invoke(method) = self
            .runtime
            .registry()
            .dispatch_class_object(class, selector_id)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(MachineError::Construction)?
        else {
            return Err(MachineError::SerializationError);
        };
        let function = usize::try_from(method.body().raw()).map_err(|_| {
            MachineError::Invalid(VerifyError::UnknownFunction {
                function: usize::MAX,
            })
        })?;
        let function = &function;
        let Some(callee) = program.functions.get(*function).cloned() else {
            return Err(MachineError::SerializationError);
        };
        let mut passed = vec![Value::Class(class)];
        passed.extend_from_slice(arguments);
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            passed,
            program,
            classes,
        )?;
        Ok(returned.into_iter().next().unwrap_or(Value::Nil))
    }

    /// Calls a method a REOPENED built-in class added, if it has one.
    ///
    /// The kernel's built-in classes carry no user methods until a reopen
    /// publishes one, so this is consulted only when a reopen exists and the
    /// native surface answered nothing.
    fn reopened_builtin_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        // Only a class the reopens actually NAMED can answer, so a value of
        // any other family is left to the ordinary refusal.
        let name = value_class_name(receiver);
        if !program
            .builtin_reopens
            .iter()
            .any(|reopen| reopen.target == name)
        {
            return Ok(None);
        }
        let Ok(class) = self.builtin_class(name) else {
            return Ok(None);
        };
        let Some(slot) = selector_id(program, selector) else {
            return Ok(None);
        };
        let Ok(iris_runtime::DispatchOutcome::Invoke(method)) =
            self.runtime.registry().dispatch(class, slot)
        else {
            return Ok(None);
        };
        let Ok(function) = usize::try_from(method.body().raw()) else {
            return Ok(None);
        };
        let Some(callee) = program.functions.get(function).cloned() else {
            return Ok(None);
        };
        let mut passed = vec![receiver.clone()];
        passed.extend_from_slice(arguments);
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            passed,
            program,
            classes,
        )?;
        Ok(Some(returned.into_iter().next().unwrap_or(Value::Nil)))
    }

    fn declares_serializable(
        &mut self,
        value: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> bool {
        let Value::Object(object) = value else {
            return false;
        };
        let Ok(class) = self.runtime.class_of(*object) else {
            return false;
        };
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return false;
        };
        program
            .contracts
            .iter()
            .position(|contract| contract.name == "Serializable")
            .is_some_and(|wanted| program.classes[index].contracts.contains(&wanted))
    }

    /// Refuses a family `C018` does not list.
    ///
    /// A live resource - an FFI handle, an open File, a native payload - is
    /// rejected rather than having its identity emitted, which is what `C003`
    /// requires and what makes the refusal a property of the VALUE rather than
    /// of how it happens to render.
    fn check_encodable(value: &Value) -> Result<(), MachineError> {
        match value {
            Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Bytes(_) => Ok(()),
            Value::Tuple(elements) => elements.iter().try_for_each(Self::check_encodable),
            Value::Array(values) => values.elements().iter().try_for_each(Self::check_encodable),
            Value::Hash(entries) => entries.entries().iter().try_for_each(|(key, held)| {
                Self::check_encodable(key).and_then(|()| Self::check_encodable(held))
            }),
            _ => Err(MachineError::SerializationError),
        }
    }
}

impl Machine {
    /// Opens a native library, per `IRIS-V1-FFI-C043`.
    ///
    /// Each open takes a FRESH identity rather than being cached by path, so
    /// two opens of one path are two objects and `a == b` is false. `C045`
    /// accepts sidecar declarations at open time, each validated exactly as a
    /// programmatic bind would be, and `C043` spells that argument
    /// `declarations:` - matching a bare Hash would silently ignore the
    /// spelling and open a Library with nothing bound.
    pub(super) fn ffi_open(
        &mut self,
        path: &Value,
        options: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let path = self.text_operand(path.clone(), program, classes)?;
        let mut bound = Vec::new();
        // `C025` makes binding PRESERVE the recorded signature, so a validated
        // signature is retained beside its symbol rather than discarded once
        // it passed validation.
        let mut signatures = Vec::new();
        if let Some(Value::KeywordArgument(name, declarations)) = options.first()
            && name == "declarations"
            && let Value::Hash(declarations) = declarations.as_ref()
        {
            for (symbol, signature) in declarations.entries() {
                let (Value::Symbol(symbol) | Value::Text(symbol)) = &symbol else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Self::validate_ffi_signature(&signature)?;
                bound.push(symbol.clone());
                signatures.push((symbol.clone(), signature.clone()));
            }
        }
        let identity = self.next_context;
        self.next_context = self.next_context.saturating_add(1);
        Ok(Value::Library(Box::new(iris_runtime::LibraryValue {
            identity,
            path,
            bound,
            signatures,
        })))
    }

    /// Checks a signature declares what `IRIS-V1-FFI-C047` requires.
    ///
    /// `C047` lists the data a signature MUST carry and requires missing data
    /// to reject the binding BEFORE any call occurs, so this checks presence
    /// rather than deferring to call time. `C046` denies any signature-less
    /// path, which is why an absent signature is a rejection and not a default.
    fn validate_ffi_signature(signature: &Value) -> Result<(), MachineError> {
        let Value::Hash(fields) = signature else {
            return Err(MachineError::IncompleteNativeSignature);
        };
        for required in ["convention", "parameters", "result", "errors"] {
            if !fields.contains_key(&Value::Symbol(required.to_owned())) {
                return Err(MachineError::IncompleteNativeSignature);
            }
        }
        // `C049` supports the stable C ABI ONLY, so a signature naming another
        // implementation language's convention is rejected here rather than
        // bound and left to fail at the call.
        if let Some(Value::Symbol(convention)) = fields.get(&Value::Symbol("convention".to_owned()))
            && convention.as_str() != "c"
        {
            return Err(MachineError::IncompleteNativeSignature);
        }
        if let Some(Value::Array(parameters)) = fields.get(&Value::Symbol("parameters".to_owned()))
        {
            for parameter in parameters.elements() {
                Self::validate_ffi_parameter(&parameter)?;
            }
        }
        Ok(())
    }

    /// Validates one declared FFI parameter.
    ///
    /// The rest of `C047`'s list is CONDITIONAL: pointer nullability,
    /// ownership, text encoding and buffer length are required "where needed".
    /// A plain scalar needs none of them, so they are demanded only once a
    /// parameter declares itself a pointer - and then before any call occurs.
    fn validate_ffi_parameter(parameter: &Value) -> Result<(), MachineError> {
        let Value::Hash(fields) = parameter else {
            return Ok(());
        };
        let Some(Value::Symbol(kind)) = fields.get(&Value::Symbol("type".to_owned())) else {
            return Err(MachineError::IncompleteNativeSignature);
        };
        if kind != "pointer" {
            return Ok(());
        }
        for required in ["nullable", "ownership"] {
            if !fields.contains_key(&Value::Symbol(required.to_owned())) {
                return Err(MachineError::IncompleteNativeSignature);
            }
        }
        // A TEXT pointer additionally needs its encoding and a BUFFER pointer
        // its length, since neither can be inferred from the pointer alone.
        if fields.get(&Value::Symbol("text".to_owned())) == Some(Value::Bool(true))
            && !fields.contains_key(&Value::Symbol("encoding".to_owned()))
        {
            return Err(MachineError::IncompleteNativeSignature);
        }
        if fields.get(&Value::Symbol("buffer".to_owned())) == Some(Value::Bool(true))
            && !fields.contains_key(&Value::Symbol("length".to_owned()))
        {
            return Err(MachineError::IncompleteNativeSignature);
        }
        Ok(())
    }

    /// Runs the `IRIS-V1-FFI` Library surface: bind, signature, bound?, call.
    pub(super) fn library_send(
        library: &iris_runtime::LibraryValue,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, MachineError> {
        let symbol_of = |value: &Value| match value {
            Value::Symbol(name) | Value::Text(name) => Some(name.clone()),
            _ => None,
        };
        Ok(match (selector, arguments) {
            // `C045` gives a programmatic bind the SAME validation a sidecar
            // declaration gets, and `C025` preserves the recorded signature.
            ("bind", [symbol, signature, ..]) => {
                let Some(symbol) = symbol_of(symbol) else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Self::validate_ffi_signature(signature)?;
                let mut bound = library.bound.clone();
                bound.push(symbol.clone());
                let mut signatures = library.signatures.clone();
                signatures.push((symbol, signature.clone()));
                Some(Value::Library(Box::new(iris_runtime::LibraryValue {
                    identity: library.identity,
                    path: library.path.clone(),
                    bound,
                    signatures,
                })))
            }
            // `C025` makes the recorded signature OBSERVABLE, and a symbol
            // that was never bound records none.
            ("signature", [symbol]) => {
                let Some(symbol) = symbol_of(symbol) else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Some(
                    library
                        .signatures
                        .iter()
                        .find_map(|(bound, signature)| {
                            (*bound == symbol).then(|| signature.clone())
                        })
                        .unwrap_or(Value::Nil),
                )
            }
            ("bound?", [symbol]) => {
                let Some(symbol) = symbol_of(symbol) else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                Some(Value::Bool(library.bound.contains(&symbol)))
            }
            // `C045` forbids invoking an UNBOUND symbol and `C046` denies any
            // signature-less escape hatch, so the refusal happens here and no
            // native boundary is crossed.
            ("call", [symbol, ..]) => {
                let Some(symbol) = symbol_of(symbol) else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                if !library.bound.contains(&symbol) {
                    return Err(MachineError::UnboundNativeSymbol);
                }
                return Err(MachineError::UnsupportedConstruct);
            }
            // `C043` makes each open an IDENTITY-BEARING Library, so two opens
            // of one path are two objects: equality compares identity rather
            // than the path they happen to share.
            ("==", [Value::Library(other)]) => {
                Some(Value::Bool(library.identity == other.identity))
            }
            ("==", [_]) => Some(Value::Bool(false)),
            ("class_name", []) => Some(Value::Text("FFI::Library".to_owned())),
            _ => None,
        })
    }
}

impl Machine {
    /// Decodes bytes in a NAMED Encoding, per `IRIS-V1-LIBRARY-C022`.
    ///
    /// Strict handling is the DEFAULT: an invalid sequence fails, and a lossy
    /// result appears only because the caller asked for it by name. Latin-1
    /// maps every byte to the scalar of that value, so it cannot fail at all
    /// and needs no error option.
    pub(super) fn encoding_decode(
        encoding: &str,
        value: &Value,
        options: &[Value],
    ) -> Result<Value, MachineError> {
        let bytes = match value {
            Value::Bytes(bytes) => bytes.clone(),
            Value::ByteArray(bytes) => bytes.bytes(),
            _ => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
        };
        let replace = options.iter().any(|option| {
            matches!(option, Value::KeywordArgument(name, mode)
                if name == "errors" && **mode == Value::Symbol("replace".to_owned()))
        });
        let decoded = match encoding {
            "Encoding::Latin_1" => Ok(bytes.iter().map(|byte| char::from(*byte)).collect()),
            "Encoding::UTF_16LE" | "Encoding::UTF_16BE" => {
                let big = encoding.ends_with("BE");
                let units: Vec<u16> = bytes
                    .chunks_exact(2)
                    .map(|pair| {
                        if big {
                            u16::from_be_bytes([pair[0], pair[1]])
                        } else {
                            u16::from_le_bytes([pair[0], pair[1]])
                        }
                    })
                    .collect();
                if bytes.len() % 2 == 0 {
                    String::from_utf16(&units).map_err(|_| ())
                } else {
                    Err(())
                }
            }
            _ => String::from_utf8(bytes.clone()).map_err(|_| ()),
        };
        match decoded {
            Ok(text) => Ok(Value::Text(text)),
            Err(()) if replace => Ok(Value::Text(String::from_utf8_lossy(&bytes).into_owned())),
            Err(()) => Err(MachineError::EncodingError),
        }
    }
}

impl Machine {
    /// Binds arguments to the categories `IRIS-V1-CONTROL-C023` gives.
    ///
    /// Positionals fill in order, `*rest` takes the remaining positionals as a
    /// fresh Array, a `key` parameter binds by NAME rather than position, and
    /// `**kwargs` collects the keywords no declared parameter matched. `C025`
    /// raises ArgumentError when a required parameter is left unbound, and
    /// `D-357` makes a DUPLICATE keyword an error rather than last-one-wins.
    ///
    /// The answer is a Tuple because the caller writes it straight into the
    /// frame's leading registers, one per declared parameter.
    pub(super) fn bind_parameters(
        kinds: &[(crate::compile::ParameterKind, String)],
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        use crate::compile::ParameterKind;
        let mut positional = Vec::new();
        let mut keyword: Vec<(String, Value)> = Vec::new();
        let mut block = Value::Nil;
        for argument in arguments {
            match argument {
                Value::KeywordArgument(name, value) => {
                    if keyword.iter().any(|(seen, _)| seen == name) {
                        return Err(MachineError::ArgumentError);
                    }
                    keyword.push((name.clone(), value.as_ref().clone()));
                }
                // A trailing Closure is the BLOCK argument, which `C023` binds
                // to `&name` rather than to a positional slot.
                Value::Closure(_) if matches!(kinds.last(), Some((ParameterKind::Block, _))) => {
                    block = argument.clone();
                }
                value => positional.push(value.clone()),
            }
        }
        let mut bound = Vec::with_capacity(kinds.len());
        let mut next = 0usize;
        for (kind, name) in kinds {
            let value = match kind {
                ParameterKind::Positional => {
                    let value = positional.get(next).cloned();
                    next += usize::from(value.is_some());
                    value
                }
                ParameterKind::Rest => {
                    let rest = positional.split_off(next.min(positional.len()));
                    Some(Value::Array(iris_runtime::ArrayRef::new(rest)))
                }
                ParameterKind::Keyword => keyword
                    .iter()
                    .position(|(seen, _)| seen == name)
                    .map(|index| keyword.remove(index).1),
                ParameterKind::KeywordRest => {
                    let rest = std::mem::take(&mut keyword)
                        .into_iter()
                        .map(|(name, value)| (Value::Symbol(name), value))
                        .collect();
                    Some(Value::Hash(iris_runtime::HashRef::new(rest)))
                }
                ParameterKind::Block => Some(block.clone()),
            };
            // An unfilled parameter is left nil here; a DEFAULT is written by
            // the `DefaultParameter` that follows, and `C025`'s ArgumentError
            // for a genuinely required one is raised by the arity check.
            bound.push(value.unwrap_or(Value::Nil));
        }
        Ok(Value::Tuple(bound))
    }
}

impl Machine {
    /// Builds and VALIDATES a Regex whose pattern was computed at run time.
    ///
    /// An interpolating literal cannot be checked when it compiles, so the
    /// same refusals apply here: an unsupported construct names itself and the
    /// engine reports which one.
    pub(super) fn make_regex(pattern: &str, flags: &str) -> Result<Value, MachineError> {
        if let Err(error) = regex::RegexBuilder::new(pattern)
            .case_insensitive(flags.contains('i'))
            .multi_line(flags.contains('m'))
            .dot_matches_new_line(flags.contains('s'))
            .ignore_whitespace(flags.contains('x'))
            .unicode(true)
            .build()
        {
            let reported = error.to_string();
            return Err(MachineError::LexicalDiagnostic(
                if reported.contains("backreference") {
                    "REGEX_UNSUPPORTED_BACKREFERENCE"
                } else if reported.contains("look-around")
                    || reported.contains("look-behind")
                    || reported.contains("look-ahead")
                {
                    "REGEX_UNSUPPORTED_LOOKBEHIND"
                } else {
                    "REGEX_SYNTAX"
                },
            ));
        }
        Ok(Value::Regex(Box::new(iris_runtime::RegexValue {
            pattern: pattern.to_owned(),
            flags: flags.to_owned(),
        })))
    }
}

impl Machine {
    /// Crosses the native ABI, per `IRIS-V1-FFI-C017`, `C018` and `C027`.
    ///
    /// `C018` lets native code raise ONLY through an ABI operation that
    /// creates an ExceptionContext, so this calls the real C ABI and converts
    /// its status plus handle into the ordinary Iris exception a `catch`
    /// observes. `C017` makes a status alone insufficient: the raised value is
    /// read back THROUGH the handle rather than recomputed, so a boundary that
    /// returned no usable context cannot still produce a correct-looking
    /// exception.
    pub(super) fn native_fixture(
        &mut self,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        match (selector, arguments) {
            // `C027` validates a payload descriptor BEFORE the runtime owns any
            // storage, so the resource reaches script only once registration
            // succeeded. Its release counter stays behind the ABI, which is
            // what makes `C030`'s idempotence observable rather than asserted.
            ("resource", []) => {
                iris_abi::iris_runtime_reset();
                let mut diagnostic = 0_u32;
                // SAFETY: `diagnostic` is a live local, so the pointer is valid.
                let status =
                    unsafe { iris_abi::iris_payload_register(8, 8, 0, 1, &raw mut diagnostic) };
                if status != iris_abi::IrisStatus::Success {
                    return Err(MachineError::UnsupportedConstruct);
                }
                let identity = self.next_context;
                self.next_context = self.next_context.saturating_add(1);
                Ok(Value::NativeResource(identity))
            }
            // `C060` makes a CORRECTLY SYNCHRONIZED observer see the write:
            // the writer thread is JOINED before the read below, so the
            // replacement is visible rather than racing. Doing the write on
            // this thread would prove nothing about synchronisation.
            ("concurrently_replace", [Value::MutableString(text), Value::Text(replacement)]) => {
                let writer = text.clone();
                let replacement = replacement.clone();
                let joined = std::thread::spawn(move || writer.set(replacement)).join();
                if joined.is_err() {
                    return Err(MachineError::UnsupportedConstruct);
                }
                Ok(Value::Nil)
            }
            ("concurrently_replace", _) => {
                Err(MachineError::Kernel(iris_runtime::KernelError::Type))
            }
            // `C031` collects unreachable objects and answers how many were
            // freed. The live FRAMES are the root set, together with the
            // bindings and other state the machine holds, so a value reachable
            // from any of them survives - which is what makes the identity of
            // a retained object stable across a collection.
            ("compact_gc", []) => {
                let mut roots: Vec<Value> = Vec::new();
                for frame in &self.frame_roots {
                    roots.extend(frame.iter().cloned());
                }
                roots.extend(self.globals.values().cloned());
                roots.extend(self.bindings.values().cloned());
                roots.extend(self.discarded_contexts.iter().cloned());
                roots.extend(self.revision_event_errors.iter().cloned());
                roots.extend(self.gates.values().flatten().cloned());
                for record in self.closures.values() {
                    roots.extend(record.captures.iter().cloned());
                }
                for task in self.suspended.iter() {
                    roots.extend(task.frame.registers.iter().cloned());
                }
                let (freed, _) = self.runtime.collect_garbage(roots.iter());
                Ok(Value::Integer(
                    u64::try_from(freed).unwrap_or(u64::MAX).into(),
                ))
            }
            ("raise", [Value::Integer(marker)]) => {
                let Some(marker) = marker.decimal_text().parse::<i64>().ok() else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                iris_abi::iris_runtime_reset();
                let mut context = iris_abi::IrisHandle::NULL;
                // SAFETY: `context` is a live local, so the out pointer is valid.
                let status = unsafe { iris_abi::iris_raise_marker(marker, &raw mut context) };
                if status != iris_abi::IrisStatus::Raised || context.is_null() {
                    return Err(MachineError::UnsupportedConstruct);
                }
                let mut carried = 0_i64;
                // SAFETY: `carried` is a live local, so the out pointer is valid.
                let read = unsafe { iris_abi::iris_handle_get_int(context, &raw mut carried) };
                if read != iris_abi::IrisStatus::Success {
                    return Err(MachineError::UnsupportedConstruct);
                }
                let Ok(recovered) = u64::try_from(carried) else {
                    return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
                };
                // `C020` makes this a CONVERSION rather than a long jump across
                // the native frame, so the raise carries a real
                // ExceptionContext that a `catch` binds like any other.
                let raised = Value::Integer(recovered.into());
                let identity = iris_runtime::ObjectId::new(self.next_context);
                self.next_context = self.next_context.saturating_add(1);
                let context = Value::ExceptionContext(
                    identity,
                    Box::new(raised.clone()),
                    Box::new(Value::Nil),
                    Vec::new(),
                    Vec::new(),
                    Box::new(Value::Nil),
                );
                Err(MachineError::Raised(Box::new((raised, context))))
            }
            _ => Err(MachineError::UnsupportedConstruct),
        }
    }
}

impl Machine {
    /// Runs an AUTHORED operator the receiver's class defines.
    ///
    /// An operator reaches `binary_send` rather than `Send`, and that path
    /// consulted only the kernel - so `V.new() + 1` answered MessageNotFound
    /// for a method the class plainly declares. A class that defines none is
    /// left to the kernel, which is what keeps `1 + 2` native.
    fn authored_operator(
        &mut self,
        receiver: &Value,
        selector: &str,
        argument: &Value,
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let Value::Object(object) = receiver else {
            return Ok(None);
        };
        let Some(slot) = selector_id(program, selector) else {
            return Ok(None);
        };
        if self.runtime.dispatch_instance(*object, slot).is_err() {
            return Ok(None);
        }
        self.instance_method_value(
            receiver,
            selector,
            std::slice::from_ref(argument),
            program,
            classes,
        )
        .map(Some)
    }

    /// The ordering two values have when neither DEFINES one.
    ///
    /// `IRIS-V1-RUNTIME-C092` gives every value a `<=>`, so a pair with no
    /// order answers nil rather than refusing the message. An object that
    /// defines its own `<=>` is not decided here - authored dispatch runs
    /// first and this only covers what it left.
    fn default_ordering(
        &mut self,
        receiver: &Value,
        argument: &Value,
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        // An ITERATION signal orders against its own kind: two `done` signals
        // are equal, and a `done` against a `yield` has no order at all.
        if matches!(receiver, Value::IterationDone | Value::IterationYield(_)) {
            return Ok(Some(match (receiver, argument) {
                (Value::IterationDone, Value::IterationDone) => Value::Integer(0_u8.into()),
                (Value::IterationYield(left), Value::IterationYield(right)) => {
                    // Two YIELDS order by the values they carry, so the inner
                    // `<=>` decides. `C015` still holds the comparison to the
                    // ordering contract: an inner answer that is not -1, 0 or
                    // 1 breaks it, and answering nil let a carried value with
                    // NO order pass as though the signals were unordered.
                    let ordering = self.binary_send(
                        "<=>",
                        (**left).clone(),
                        (**right).clone(),
                        program,
                        classes,
                    )?;
                    let Value::Integer(value) = &ordering else {
                        return Err(MachineError::ComparisonContractError);
                    };
                    if !matches!(value.to_i128(), Some(-1..=1)) {
                        return Err(MachineError::ComparisonContractError);
                    }
                    return Ok(Some(ordering));
                }
                _ => Value::Nil,
            }));
        }
        // An OBJECT with no `<=>` of its own has no order, which is nil rather
        // than a refusal. Authored dispatch already ran, so reaching here
        // means the class defines none.
        if matches!(receiver, Value::Object(_)) {
            return Ok(Some(Value::Nil));
        }
        Ok(None)
    }
}

impl Machine {
    /// Validates a value used as a HASH KEY, per `IRIS-V1-COLLECTIONS-C087`.
    ///
    /// A key must have a hash, and an OBJECT supplies its own: a class
    /// defining `hash` is a legitimate key even though no specification-stable
    /// hash covers its family. Calling `public_hash` directly refused those,
    /// and it also spells a refusal `StableHash(..)` where the language says
    /// `InvalidKeyError` - one failure with two names.
    /// The BUCKET a key currently hashes to.
    ///
    /// `C028` dispatches the key's CURRENT `hash`, and an OBJECT supplies its
    /// own - so authored dispatch runs first. A key with no hash at all is not
    /// a legal key, which is the key failure rather than a missing message.
    pub(crate) fn key_hash(
        &mut self,
        key: &Value,
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        // A class DEFINING `hash` decides its own bucket, so its declared
        // method runs before the identity hash every object otherwise has -
        // reaching the identity first gave two keys that hash ALIKE two
        // different buckets and left them as separate entries.
        if let Value::Object(object) = key
            && let Some(slot) = selector_id(program, "hash")
            && let Ok(method) = self.runtime.dispatch_instance(*object, slot)
            && let Ok(function) = usize::try_from(method.body().raw())
            && let Some(callee) = program.functions.get(function).cloned()
        {
            let returned = self.run_body(
                &callee.instructions,
                callee.registers,
                vec![Value::Object(*object)],
                program,
                classes,
            )?;
            return Ok(returned.into_iter().next().unwrap_or(Value::Nil));
        }
        if matches!(key, Value::Object(_)) {
            return self
                .authored_send(key, "hash", &[], program, classes)?
                .ok_or(MachineError::InvalidKeyError);
        }
        self.send("hash", key.clone(), &[])
            .map_err(|_| MachineError::InvalidKeyError)
    }

    pub(super) fn validate_key(
        &mut self,
        key: &Value,
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        if matches!(key, Value::Object(_)) {
            // A class defining `hash` decides its own key; one that does not
            // has no hash to offer, which is the key failure.
            return match self.authored_send(key, "hash", &[], program, classes)? {
                Some(_) => Ok(()),
                None => Err(MachineError::InvalidKeyError),
            };
        }
        // An IDENTITY-bearing value hashes by its identity rather than by
        // content, which no specification-stable family hash covers - so a
        // context or a task is a legitimate key even though `public_hash`
        // alone refuses it.
        if self.send("hash", key.clone(), &[]).is_ok() {
            return Ok(());
        }
        iris_runtime::public_hash(key)
            .map(|_| ())
            .map_err(|_| MachineError::InvalidKeyError)
    }
}

impl Machine {
    /// Reports whether two keys are EQUAL under the current equality.
    ///
    /// A rehash groups keys into equality classes, and an object decides its
    /// own class through the `==` its class defines - so the comparison goes
    /// through that rather than over representations.
    pub(super) fn key_equal(
        &mut self,
        left: &Value,
        right: &Value,
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<bool, MachineError> {
        let equal = self.binary_send("==", left.clone(), right.clone(), program, classes)?;
        Ok(matches!(equal, Value::Bool(true)))
    }
}

impl Machine {
    /// Offers a missing selector to the class's `method_missing`.
    ///
    /// `IRIS-V1-RUNTIME-C099` gives a class a last say once ordinary dispatch
    /// found nothing, so a declared method still wins. The trailing block is
    /// passed as the separate `block` parameter rather than inside the
    /// positional snapshot, which is what lets a handler tell one from the
    /// other. A class declaring no handler answers `None`, leaving the
    /// original MessageNotFound to stand.
    pub(super) fn invoke_method_missing(
        &mut self,
        object: iris_runtime::ObjectId,
        missing: &str,
        arguments: &[Value],
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        // A missing `method_missing` must not recurse into itself.
        if missing == "method_missing" {
            return Ok(None);
        }
        // An object whose class declares NO `to_string` still answers one:
        // `D-111` renders it as its package and source name, which is what
        // lets an ordinary object be printed at all. A declared method wins,
        // since ordinary dispatch already ran before reaching here.
        if matches!(missing, "to_string" | "inspect") && arguments.is_empty() {
            let class = self
                .runtime
                .class_of(object)
                .map_err(MachineError::Construction)?;
            let name = classes
                .iter()
                .position(|known| *known == class)
                .and_then(|index| program.classes.get(index))
                .map(|declaration| declaration.name.clone())
                .unwrap_or_default();
            return Ok(Some(Value::Text(format!("<runtime-local::{name}>"))));
        }
        let Some(slot) = selector_id(program, "method_missing") else {
            return Ok(None);
        };
        if self.runtime.dispatch_instance(object, slot).is_err() {
            return Ok(None);
        }
        // `C099` splits a trailing Closure out of the positional snapshot.
        let (positional, block) = match arguments {
            [head @ .., Value::Closure(closure)] => (head.to_vec(), Value::Closure(*closure)),
            _ => (arguments.to_vec(), Value::Nil),
        };
        self.instance_method_value(
            &Value::Object(object),
            "method_missing",
            &[
                Value::Symbol(missing.to_owned()),
                Value::Array(iris_runtime::ArrayRef::new(positional)),
                block,
            ],
            program,
            classes,
        )
        .map(Some)
    }
}

/// Reports whether two IDENTITY-bearing values name one value.
///
/// An iterator, a closure, a task and the other identity-bearing families
/// compare by which value they are rather than by content: two iterators over
/// one array are distinct even though they would yield the same elements.
fn identity_equality(left: &Value, right: &Value) -> Option<bool> {
    let identity = |value: &Value| match value {
        Value::ArrayIterator(identity)
        | Value::HashIterator(identity)
        | Value::ByteIterator(identity)
        | Value::Generator(identity)
        | Value::Task(identity)
        | Value::Gate(identity)
        | Value::Closure(identity) => Some(identity.raw()),
        _ => None,
    };
    Some(identity(left)? == identity(right)?)
}

/// Reflects one type ATOM as the value a program observes.
///
/// A nominal atom answers its Type, a contract answers the Contract itself,
/// and a nested union answers a composed type - so a member read names what
/// the source wrote rather than a flattened list.
fn reflect_atom(atom: &iris_runtime::TypeAtom) -> Value {
    match atom {
        iris_runtime::TypeAtom::Nominal(class, arguments) => Value::Type(*class, arguments.clone()),
        iris_runtime::TypeAtom::NonNil => Value::Symbol("NonNil".to_owned()),
        iris_runtime::TypeAtom::Contract(contract) => Value::Contract(*contract),
        iris_runtime::TypeAtom::Union(nested) => {
            Value::ComposedType(iris_runtime::ComposedType::Union(nested.clone()))
        }
    }
}
