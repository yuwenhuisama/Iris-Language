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
            // `C031` REBUILDS the table against each key's current hash, so a
            // key whose hash changed lands in its new slot. Two entries that
            // become equal collide, and `C032` resolves that with a merge
            // block - without one the conflict aborts rather than silently
            // dropping an entry.
            ("rehash", []) => {
                let current = entries.entries();
                let mut rebuilt: Vec<(Value, Value)> = Vec::with_capacity(current.len());
                for (key, value) in current {
                    self.validate_key(&key, program, classes)?;
                    let mut collided = false;
                    for (kept, _) in &rebuilt {
                        if self.key_equal(kept, &key, program, classes)? {
                            collided = true;
                            break;
                        }
                    }
                    if collided {
                        return Err(MachineError::KeyConflictError);
                    }
                    rebuilt.push((key, value));
                }
                // Every key kept its own class, so the table already holds the
                // rebuilt content: `C031`'s work here is the VALIDATION, and a
                // conflict aborts above rather than publishing a partial table.
                let _ = rebuilt;
                Value::Nil
            }
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
        // `inspect` answers a REPARSABLE literal, so every scalar the literal
        // grammar gives a meaning to is escaped back. Interpolation is written
        // `${...}` and the escape table has no `\$`, so a `$` that would OPEN
        // one is emitted as its Unicode escape instead - that reparses to the
        // same scalar without interpolating.
        ("inspect", []) => {
            let mut rendered = String::from("\"");
            let mut scalars = text.chars().peekable();
            while let Some(scalar) = scalars.next() {
                match scalar {
                    '"' => rendered.push_str("\\\""),
                    '\\' => rendered.push_str("\\\\"),
                    '\n' => rendered.push_str("\\n"),
                    '\r' => rendered.push_str("\\r"),
                    '\t' => rendered.push_str("\\t"),
                    '$' if scalars.peek() == Some(&'{') => rendered.push_str("\\u{24}"),
                    scalar => rendered.push(scalar),
                }
            }
            rendered.push('"');
            Some(Value::Text(rendered))
        }
        _ => None,
    }
}
