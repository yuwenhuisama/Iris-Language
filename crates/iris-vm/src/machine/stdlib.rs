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
    fn class_method_value(
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
        let Some((_, function)) = program.classes[index]
            .class_methods
            .iter()
            .find(|(name, _)| name == selector)
        else {
            return Err(MachineError::SerializationError);
        };
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
