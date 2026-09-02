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
            ("rehash", []) => self.rehash(entries, None, program, classes)?,
            // `C032` supplies the merge as a trailing block, which reaches the
            // send as an ordinary closure argument.
            ("rehash", [block @ Value::Closure(_)]) => {
                self.rehash(entries, Some(block.clone()), program, classes)?
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
            // `C028` dispatches the key's CURRENT `==` to find the slot, so a
            // lookup groups by EQUALITY rather than by representation - two
            // objects that compare equal name one entry, which a raw
            // comparison could not see.
            ("include?" | "has_key?", [key]) => {
                Value::Bool(self.hash_slot(entries, key, program, classes)?.is_some())
            }
            // `fetch` differs from indexing exactly here: indexing answers nil
            // for an absent key and `fetch` REFUSES, which is what makes it an
            // assertion that the key is present.
            ("fetch", [key]) => self
                .hash_slot(entries, key, program, classes)?
                .and_then(|position| entries.value_at(position))
                .ok_or(MachineError::KeyError)?,
            ("delete", [key]) => self
                .hash_slot(entries, key, program, classes)?
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
        // `C072` keeps text and binary conversion to EXPLICIT APIs, and the
        // encoding is UTF-8 - so the bytes are the text's own encoding rather
        // than a host-chosen one.
        ("to_bytes", []) => Some(Value::Bytes(text.as_bytes().to_vec())),
        // `C051` makes `to_array` the ordered element sequence, and `C009`
        // counts a SCALAR once - so an astral character is one element rather
        // than the several bytes that carry it.
        ("to_array", []) => Some(Value::Array(ArrayRef::new(
            text.chars()
                .map(|scalar| Value::Text(scalar.to_string()))
                .collect(),
        ))),
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

impl Machine {
    /// Rebuilds a Hash against its keys' CURRENT hashes.
    ///
    /// `IRIS-V1-COLLECTIONS-C031` rebuilds from current public hashes, so a
    /// key whose hash changed lands in its new slot rather than keeping a
    /// stale one. Two entries that become EQUAL collide into one equality
    /// class, and `C032` resolves that with a merge block - without one the
    /// conflict aborts rather than silently dropping an entry.
    fn rehash(
        &mut self,
        entries: &HashRef,
        merge: Option<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let current = entries.entries();
        let mut rebuilt: Vec<(Value, Value)> = Vec::with_capacity(current.len());
        for (key, value) in current {
            // Validation happens against the CURRENT hash, so an unhashable
            // key aborts the rehash rather than silently keeping its old slot.
            self.validate_key(&key, program, classes)?;
            let mut collision = None;
            for (index, (kept, _)) in rebuilt.iter().enumerate() {
                if self.key_equal(kept, &key, program, classes)? {
                    collision = Some(index);
                    break;
                }
            }
            let Some(index) = collision else {
                rebuilt.push((key, value));
                continue;
            };
            let Some(merge) = merge.clone() else {
                return Err(MachineError::KeyConflictError);
            };
            let (kept_key, kept_value) = rebuilt[index].clone();
            let merged = self.invoke_closure(
                &merge,
                &[kept_key, kept_value, key, value],
                program,
                classes,
            )?;
            // `C032` takes the block result as a two-element `(key, value)`
            // replacement. A different SHAPE is a Type failure, distinct from
            // the `C031` conflict a MISSING block reports.
            let Value::Tuple(replacement) = &merged else {
                return Err(MachineError::TypeContractError);
            };
            let [new_key, new_value] = replacement.as_slice() else {
                return Err(MachineError::TypeContractError);
            };
            // The returned key must REMAIN equal to the class it replaces, so
            // a key that leaves its own class is a new inconsistency.
            if !self.key_equal(new_key, &rebuilt[index].0, program, classes)? {
                return Err(MachineError::KeyConflictError);
            }
            self.validate_key(new_key, program, classes)?;
            rebuilt[index] = (new_key.clone(), new_value.clone());
        }
        // Every check passed, so the replacement is published ATOMICALLY - a
        // failure above leaves the original table untouched.
        entries.replace_entries(rebuilt);
        Ok(Value::Nil)
    }
}

impl Machine {
    /// The slot a KEY names, under the current equality.
    ///
    /// `C028` dispatches each key's own `==` to decide, so two objects that
    /// compare EQUAL name one entry - a raw comparison over representations
    /// cannot see that and leaves a lookup missing an entry that is present.
    pub(crate) fn hash_slot(
        &mut self,
        entries: &HashRef,
        key: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<usize>, MachineError> {
        // `C028` uses the key's current `hash` AND `==`, so a key whose hash
        // has MOVED since insertion no longer finds its entry: the stored slot
        // was placed under the old hash. `C030` makes `rehash()` the remedy
        // and leaves the inconsistency until then to the program.
        let wanted = self.key_hash(key, program, classes)?;
        for (position, (held, _)) in entries.entries().iter().enumerate() {
            // The bucket recorded at INSERTION is what the entry sits under, so
            // a key whose hash has since moved misses it.
            let placed = entries.bucket_at(position).unwrap_or(Value::Nil);
            if placed != Value::Nil && placed != wanted {
                continue;
            }
            if self.key_equal(held, key, program, classes)? {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }
}
