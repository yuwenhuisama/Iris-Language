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

    fn text_operand(
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
            return Ok(None);
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
            Value::Array(_) | Value::Hash(_) | Value::Range(_)
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
            Value::Type(class, _) if selector == "kind" && arguments.is_empty() => {
                Some(Value::Symbol("nominal".to_owned()))
            }
            value @ (Value::Type(..) | Value::ComposedType(_))
                if selector == "type" && arguments.is_empty() =>
            {
                Some(value.clone())
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
