use super::{Machine, MachineError};
use iris_runtime::{ArrayRef, ClassId, KernelError, NominalType, Value};

impl Machine {
    pub(super) fn immutable_callback_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &crate::compile::Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        match receiver {
            Value::ImmutableArray(array) => self.array_send(
                &ArrayRef::new(array.elements().to_vec()),
                receiver,
                selector,
                arguments,
                program,
                classes,
            ),
            Value::ImmutableHash(hash) if matches!(selector, "each" | "each_with_iterator") => {
                let [callback @ (Value::Closure(_) | Value::BoundMethod(_))] = arguments else {
                    return Err(MachineError::ArgumentError);
                };
                let cursor = self
                    .open_builtin_iterator(receiver)?
                    .ok_or(MachineError::IteratorState)?;
                let outcome = (|| {
                    for (key, value) in hash.entries() {
                        let mut passed = vec![key.clone(), value.clone()];
                        self.iteration_send(&cursor, "next", &[])?;
                        if selector == "each_with_iterator" {
                            passed.push(cursor.clone());
                        }
                        self.invoke_closure(callback, &passed, program, classes)?;
                    }
                    Ok(Some(receiver.clone()))
                })();
                self.iteration_send(&cursor, "close", &[])?;
                outcome
            }
            Value::ImmutableHash(hash) => self.hash_send(
                &iris_runtime::HashRef::new(hash.entries().to_vec()),
                selector,
                arguments,
                program,
                classes,
            ),
            _ => Ok(None),
        }
    }

    pub(super) fn immutable_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, MachineError> {
        if !matches!(receiver, Value::ImmutableArray(_) | Value::ImmutableHash(_)) {
            return Ok(None);
        }
        if selector.ends_with('!')
            || (selector.ends_with('=') && !matches!(selector, "==" | "!="))
            || matches!(
                selector,
                "append"
                    | "push"
                    | "pop"
                    | "shift"
                    | "unshift"
                    | "insert"
                    | "delete"
                    | "delete_at"
                    | "clear"
                    | "remove"
                    | "remove_at"
                    | "replace"
                    | "store"
                    | "update"
                    | "rehash"
                    | "fill"
            )
        {
            return Err(MachineError::ReadonlyMutation);
        }
        if selector == "iterator" && arguments.is_empty() {
            return self.open_builtin_iterator(receiver);
        }
        let length = match receiver {
            Value::ImmutableArray(array) => array.len(),
            Value::ImmutableHash(hash) => hash.len(),
            _ => return Ok(None),
        };
        let value = match (selector, arguments) {
            ("length" | "count", []) => Value::Integer(
                u64::try_from(length)
                    .map_err(|_| MachineError::RangeError)?
                    .into(),
            ),
            ("empty?", []) => Value::Bool(length == 0),
            ("class_name", []) => Value::Symbol(super::value_class_name(receiver).into()),
            ("[]" | "fetch", [key]) => {
                let value = match receiver {
                    Value::ImmutableArray(array) => {
                        let Value::Integer(index) = key else {
                            return Err(MachineError::Kernel(KernelError::Type));
                        };
                        super::resolve_index(index, array.len())
                            .and_then(|index| array.get(index))
                            .cloned()
                    }
                    Value::ImmutableHash(hash) => hash.get(key).cloned(),
                    _ => return Ok(None),
                };
                match value {
                    Some(value) => value,
                    None if selector == "fetch" => {
                        return Err(match receiver {
                            Value::ImmutableHash(_) => MachineError::KeyError,
                            _ => MachineError::IndexError,
                        });
                    }
                    None => Value::Nil,
                }
            }
            ("to_array", []) => match receiver {
                Value::ImmutableArray(array) => {
                    Value::Array(ArrayRef::new(array.elements().to_vec()))
                }
                Value::ImmutableHash(hash) => Value::Array(ArrayRef::new(
                    hash.entries()
                        .iter()
                        .map(|(key, value)| Value::Tuple(vec![key.clone(), value.clone()]))
                        .collect(),
                )),
                _ => return Ok(None),
            },
            ("keys" | "values", []) => match receiver {
                Value::ImmutableHash(hash) => Value::Array(ArrayRef::new(
                    hash.entries()
                        .iter()
                        .map(|(key, value)| {
                            if selector == "keys" {
                                key.clone()
                            } else {
                                value.clone()
                            }
                        })
                        .collect(),
                )),
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(super) fn immutable_type_test(
        &self,
        value: &Value,
        target: ClassId,
        arguments: &[NominalType],
    ) -> Result<Option<bool>, MachineError> {
        let types = match value {
            Value::ImmutableArray(array) if self.kernel.core_class("Array") == Some(target) => {
                vec![array.element_type()]
            }
            Value::ImmutableHash(hash) if self.kernel.core_class("Hash") == Some(target) => {
                vec![hash.key_type(), hash.value_type()]
            }
            Value::Array(_) if self.kernel.core_class("Array") == Some(target) => {
                return Ok(Some(arguments.is_empty()));
            }
            Value::Hash(_) if self.kernel.core_class("Hash") == Some(target) => {
                return Ok(Some(arguments.is_empty()));
            }
            _ => return Ok(None),
        };
        Ok(Some(arguments.is_empty() || (types.len() == arguments.len() && types.iter().zip(arguments).all(|(held, requested)| {
            matches!(held, Value::Type(class, parameters) if *class == requested.class() && parameters == requested.arguments())
        }))))
    }
}
