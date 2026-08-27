use iris_runtime::{ArrayRef, ClassId, HashRef, Value};

use super::super::{Machine, MachineError, truthy};
use crate::compile::Program;

impl Machine {
    pub(super) fn hash_send(
        &mut self,
        entries: &HashRef,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let result = match (selector, arguments) {
            ("length", []) => Value::Integer((entries.len() as u64).into()),
            ("keys", []) => {
                Value::Array(entries.entries().into_iter().map(|(key, _)| key).collect())
            }
            ("values", []) => Value::Array(
                entries
                    .entries()
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect(),
            ),
            ("include?" | "has_key?", [key]) => Value::Bool(entries.contains_key(key)),
            // `fetch` differs from indexing exactly here: indexing answers nil
            // for an absent key and `fetch` REFUSES, which is what makes it an
            // assertion that the key is present.
            ("fetch", [key]) => entries
                .entries()
                .iter()
                .position(|(held, _)| held == key)
                .and_then(|position| entries.value_at(position))
                .ok_or(MachineError::KeyError)?,
            ("delete", [key]) => entries
                .entries()
                .iter()
                .position(|(held, _)| held == key)
                .and_then(|position| entries.remove_at(position))
                .unwrap_or(Value::Nil),
            ("map", [block @ Value::Closure(_)]) => {
                let mut mapped = Vec::new();
                for (key, value) in entries.entries() {
                    mapped.push(self.invoke_closure(block, &[key, value], program, classes)?);
                }
                Value::Array(ArrayRef::new(mapped))
            }
            ("select", [block @ Value::Closure(_)]) => {
                let mut selected = Vec::new();
                for (key, value) in entries.entries() {
                    if truthy(&self.invoke_closure(
                        block,
                        &[key.clone(), value.clone()],
                        program,
                        classes,
                    )?) {
                        selected.push((key, value));
                    }
                }
                Value::Hash(HashRef::new(selected))
            }
            ("merge", [Value::Hash(other)]) => {
                let merged = HashRef::new(entries.entries());
                for (key, value) in other.entries() {
                    if let Some(position) =
                        merged.entries().iter().position(|(held, _)| held == &key)
                    {
                        merged.remove_at(position);
                    }
                    merged.insert_at(None, key, value);
                }
                Value::Hash(merged)
            }
            ("to_array", []) => Value::Array(ArrayRef::new(
                entries
                    .entries()
                    .into_iter()
                    .map(|(key, value)| Value::Tuple(vec![key, value]))
                    .collect(),
            )),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}

pub(super) fn text_send(text: &str, selector: &str, arguments: &[Value]) -> Option<Value> {
    match (selector, arguments) {
        ("length", []) => Some(Value::Integer((text.chars().count() as u64).into())),
        ("to_string", []) => Some(Value::Text(text.to_owned())),
        ("split", [Value::Text(separator)]) => Some(Value::Array(
            text.split(separator)
                .map(|piece| Value::Text(piece.to_owned()))
                .collect(),
        )),
        ("trim", []) => Some(Value::Text(text.trim().to_owned())),
        ("replace", [Value::Text(from), Value::Text(to)]) => {
            Some(Value::Text(text.replace(from, to)))
        }
        ("starts_with?", [Value::Text(prefix)]) => Some(Value::Bool(text.starts_with(prefix))),
        ("ends_with?", [Value::Text(suffix)]) => Some(Value::Bool(text.ends_with(suffix))),
        ("contains?", [Value::Text(needle)]) => Some(Value::Bool(text.contains(needle))),
        ("downcase", []) => Some(Value::Text(text.to_lowercase())),
        ("chars", []) => Some(Value::Array(
            text.chars()
                .map(|scalar| Value::Text(scalar.to_string()))
                .collect(),
        )),
        ("to_symbol", []) => Some(Value::Symbol(text.to_owned())),
        _ => None,
    }
}
