use super::{
    ArrayCursor, EvaluationError, SourceEvaluator, check_slice_range, resolve_index, slice_bounds,
};
use iris_runtime::{ArrayRef, ImmutableArray, Value};

impl SourceEvaluator {
    pub(super) fn immutable_collection_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        let name = match receiver {
            Value::ImmutableArray(_) => "Array",
            Value::ImmutableHash(_) => "Hash",
            _ => return Ok(None),
        };
        if selector.ends_with('=') && selector != "=="
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
                    | "remove"
                    | "clear"
                    | "replace"
                    | "update"
                    | "merge!"
                    | "reverse!"
                    | "sort!"
                    | "map!"
                    | "filter!"
                    | "rehash"
                    | "remove_current"
            )
        {
            return Err(EvaluationError::ReadonlyMutation);
        }
        match (selector, arguments) {
            ("class_name", []) => return Ok(Some(Value::Symbol(name.into()))),
            ("class", []) => return Ok(self.kernel.core_class(name).map(Value::Class)),
            ("type", []) => {
                let types = match receiver {
                    Value::ImmutableArray(array) => vec![array.element_type()],
                    Value::ImmutableHash(hash) => vec![hash.key_type(), hash.value_type()],
                    _ => return Ok(None),
                };
                let arguments = types
                    .into_iter()
                    .map(|value| match value {
                        Value::Type(class, arguments) => {
                            Ok(iris_runtime::NominalType::new(*class, arguments.clone()))
                        }
                        _ => Err(EvaluationError::UnsupportedConstruct),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(self
                    .kernel
                    .core_class(name)
                    .map(|class| Value::Type(class, arguments)));
            }
            ("same?", [other]) => {
                return Ok(Some(Value::Bool(match (receiver, other) {
                    (Value::ImmutableArray(left), Value::ImmutableArray(right)) => left.same(right),
                    (Value::ImmutableHash(left), Value::ImmutableHash(right)) => left.same(right),
                    _ => false,
                })));
            }
            ("iterator", []) => {
                let values = match receiver {
                    Value::ImmutableArray(array) => array.elements().to_vec(),
                    Value::ImmutableHash(hash) => hash
                        .entries()
                        .iter()
                        .map(|(key, value)| Value::Tuple(vec![key.clone(), value.clone()]))
                        .collect(),
                    _ => return Ok(None),
                };
                let identity = self.next_context_identity();
                self.array_iterators.insert(
                    identity,
                    ArrayCursor {
                        values: Some(ArrayRef::new(values)),
                        position: 0,
                        expected_version: 0,
                        fail_fast: false,
                    },
                );
                return Ok(Some(Value::ArrayIterator(identity)));
            }
            _ => {}
        }
        let result = match (receiver, selector, arguments) {
            (Value::ImmutableArray(array), "[]", [Value::Range(range)]) => {
                check_slice_range(range)?;
                let span = slice_bounds(&range.start, &range.end, range.inclusive_end, array.len());
                Value::Array(ArrayRef::new(
                    array.elements().get(span).unwrap_or_default().to_vec(),
                ))
            }
            (Value::ImmutableArray(array), "length", []) => Value::Integer(
                u64::try_from(array.len())
                    .map_err(|_| EvaluationError::IndexError)?
                    .into(),
            ),
            (Value::ImmutableHash(hash), "length", []) => Value::Integer(
                u64::try_from(hash.len())
                    .map_err(|_| EvaluationError::IndexError)?
                    .into(),
            ),
            (Value::ImmutableArray(array), "empty?", []) => Value::Bool(array.is_empty()),
            (Value::ImmutableHash(hash), "empty?", []) => Value::Bool(hash.is_empty()),
            (Value::ImmutableArray(array), "[]" | "fetch", [Value::Integer(index)]) => {
                let value = resolve_index(index, array.len())
                    .and_then(|index| array.get(index))
                    .cloned();
                if selector == "fetch" {
                    value.ok_or(EvaluationError::IndexError)?
                } else {
                    value.unwrap_or(Value::Nil)
                }
            }
            (Value::ImmutableArray(_), "[]" | "fetch", [_]) => {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            }
            (Value::ImmutableHash(hash), "[]" | "fetch", [key]) => {
                let mut found = None;
                self.key_hash(key)?;
                for (held, value) in hash.entries() {
                    if self.key_equal(held, key)? {
                        found = Some(value.clone());
                        break;
                    }
                }
                if selector == "fetch" {
                    found.ok_or(EvaluationError::KeyError)?
                } else {
                    found.unwrap_or(Value::Nil)
                }
            }
            (Value::ImmutableHash(hash), "keys", []) => Value::ImmutableArray(ImmutableArray::new(
                hash.entries().iter().map(|(key, _)| key.clone()).collect(),
                hash.key_type().clone(),
            )),
            (Value::ImmutableHash(hash), "values", []) => {
                Value::ImmutableArray(ImmutableArray::new(
                    hash.entries()
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect(),
                    hash.value_type().clone(),
                ))
            }
            (_, "length" | "empty?" | "iterator" | "fetch" | "[]" | "keys" | "values", _) => {
                return Err(EvaluationError::ArgumentError);
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    pub(super) fn immutable_annotation_admits(
        &mut self,
        value: &Value,
        annotation: &iris_syntax::TypeExpression,
    ) -> Result<Option<bool>, EvaluationError> {
        let iris_syntax::TypeExpression::Generic { name, arguments } = annotation else {
            return Ok(None);
        };
        let matches = match (value, name.as_str(), arguments.as_slice()) {
            (Value::ImmutableArray(array), "Array", [element]) => {
                *array.element_type() == self.reify_type(element)?
            }
            (Value::ImmutableHash(hash), "Hash", [key, value]) => {
                *hash.key_type() == self.reify_type(key)?
                    && *hash.value_type() == self.reify_type(value)?
            }
            (Value::Array(array), "Array", [element]) => {
                for held in array.elements() {
                    if !self.annotation_admits(&held, element)? {
                        return Ok(Some(false));
                    }
                }
                true
            }
            (Value::Hash(hash), "Hash", [key_type, value_type]) => {
                for (key, value) in hash.entries() {
                    if !self.annotation_admits(&key, key_type)?
                        || !self.annotation_admits(&value, value_type)?
                    {
                        return Ok(Some(false));
                    }
                }
                true
            }
            _ => return Ok(None),
        };
        Ok(Some(matches))
    }
}
