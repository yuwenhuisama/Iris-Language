use iris_runtime::Value;

use super::super::{IteratorSource, Machine, MachineError};

/// Materializes the values a Range would yield, in order.
///
/// `IRIS-V1-COLLECTIONS-C039` gives openness and step their meaning here once,
/// so iterating a Range and asking it for an array cannot disagree about which
/// values it holds.
pub(crate) fn range_values(range: &iris_runtime::RangeValue) -> Result<Vec<Value>, MachineError> {
    let Some(start) = range.start.to_i128() else {
        return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
    };
    let Some(end) = range.end.to_i128() else {
        return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
    };
    let Some(step) = range.step.to_i128() else {
        return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
    };
    let mut values = Vec::new();
    let mut current = start;
    while (step > 0 && (current < end || (range.inclusive_end && current == end)))
        || (step < 0 && (current > end || (range.inclusive_end && current == end)))
    {
        let value = current
            .to_string()
            .parse()
            .map_err(|_| MachineError::Kernel(iris_runtime::KernelError::Type))?;
        values.push(Value::Integer(value));
        current += step;
    }
    Ok(values)
}

impl Machine {
    pub(crate) fn open_builtin_iterator(
        &mut self,
        receiver: &Value,
    ) -> Result<Option<Value>, MachineError> {
        let source = match receiver {
            Value::Array(source) => IteratorSource::Array {
                expected_version: source.version(),
                source: source.clone(),
            },
            Value::Hash(source) => IteratorSource::Hash {
                keys: source.entries().into_iter().map(|(key, _)| key).collect(),
                expected_version: source.version(),
                source: source.clone(),
            },
            Value::Range(range) => IteratorSource::Values(range_values(range)?),
            // `C061` gives a MutableString cursor a LIVE view over its own
            // content: it walks SCALARS and any change to the receiver
            // invalidates it, rather than walking a snapshot the program can
            // no longer see.
            Value::MutableString(source) => IteratorSource::Text {
                values: source
                    .text()
                    .chars()
                    .map(|scalar| Value::Text(scalar.to_string()))
                    .collect(),
                expected_version: source.version(),
                source: source.clone(),
            },
            // `C075` yields Integer byte values in source order, and Bytes is
            // IMMUTABLE so its cursor can never fail fast.
            Value::Bytes(bytes) => IteratorSource::Values(
                bytes
                    .iter()
                    .map(|byte| Value::Integer(u64::from(*byte).into()))
                    .collect(),
            ),
            _ => return Ok(None),
        };
        let identity = iris_runtime::ObjectId::new(self.next_iterator);
        self.next_iterator = self.next_iterator.saturating_add(1);
        let value = match source {
            IteratorSource::Hash { .. } => Value::HashIterator(identity),
            IteratorSource::Array { .. }
            | IteratorSource::Values(_)
            | IteratorSource::Text { .. } => Value::ArrayIterator(identity),
        };
        self.iterators.insert(
            identity,
            super::super::IteratorRecord {
                source: Some(source),
                position: 0,
                removed_current: false,
            },
        );
        Ok(Some(value))
    }

    pub(crate) fn iteration_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, MachineError> {
        if !arguments.is_empty() {
            return Ok(None);
        }
        match (receiver, selector) {
            (Value::IterationYield(_), "yield?") => Ok(Some(Value::Bool(true))),
            (Value::IterationYield(_), "done?") => Ok(Some(Value::Bool(false))),
            (Value::IterationYield(value), "value") => Ok(Some((**value).clone())),
            (Value::IterationDone, "yield?") => Ok(Some(Value::Bool(false))),
            (Value::IterationDone, "done?") => Ok(Some(Value::Bool(true))),
            (Value::IterationDone, "value") => Err(MachineError::IteratorState),
            (Value::ArrayIterator(identity) | Value::HashIterator(identity), "close") => {
                if let Some(iterator) = self.iterators.get_mut(identity) {
                    iterator.source = None;
                }
                Ok(Some(Value::Nil))
            }
            (Value::ArrayIterator(identity) | Value::HashIterator(identity), "next") => {
                self.advance_iterator(*identity).map(Some)
            }
            // `C026` lets a Hash iterator remove the entry it just YIELDED,
            // once. Before the first `next` there is no current entry, and a
            // second removal names none either - both are iterator-state
            // failures rather than silent no-ops. The removal advances the
            // expected version, so the iterator keeps walking what remains
            // instead of reporting concurrent modification against itself.
            (Value::HashIterator(identity), "remove_current") => {
                let Some(iterator) = self.iterators.get_mut(identity) else {
                    return Err(MachineError::IteratorState);
                };
                if iterator.removed_current || iterator.position == 0 {
                    return Err(MachineError::IteratorState);
                }
                let Some(IteratorSource::Hash {
                    source,
                    keys,
                    expected_version,
                }) = iterator.source.as_mut()
                else {
                    return Err(MachineError::IteratorState);
                };
                if source.version() != *expected_version {
                    return Err(MachineError::ConcurrentModification);
                }
                let Some(key) = keys.get(iterator.position - 1).cloned() else {
                    return Err(MachineError::IteratorState);
                };
                source.remove(&key);
                *expected_version = source.version();
                iterator.removed_current = true;
                Ok(Some(Value::Nil))
            }
            _ => Ok(None),
        }
    }

    fn advance_iterator(
        &mut self,
        identity: iris_runtime::ObjectId,
    ) -> Result<Value, MachineError> {
        let Some(iterator) = self.iterators.get_mut(&identity) else {
            return Err(MachineError::IteratorState);
        };
        let Some(source) = iterator.source.as_ref() else {
            return Ok(Value::IterationDone);
        };
        let value = match source {
            IteratorSource::Array {
                source,
                expected_version,
            } => {
                if source.version() != *expected_version {
                    iterator.source = None;
                    return Err(MachineError::ConcurrentModification);
                }
                source.elements().get(iterator.position).cloned()
            }
            IteratorSource::Hash {
                source,
                keys,
                expected_version,
            } => {
                if source.version() != *expected_version {
                    iterator.source = None;
                    return Err(MachineError::ConcurrentModification);
                }
                keys.get(iterator.position)
                    .and_then(|key| source.get(key))
                    .map(|value| Value::Tuple(vec![keys[iterator.position].clone(), value]))
            }
            IteratorSource::Values(values) => values.get(iterator.position).cloned(),
            IteratorSource::Text {
                source,
                values,
                expected_version,
            } => {
                if source.version() != *expected_version {
                    iterator.source = None;
                    return Err(MachineError::ConcurrentModification);
                }
                values.get(iterator.position).cloned()
            }
        };
        let Some(value) = value else {
            iterator.source = None;
            return Ok(Value::IterationDone);
        };
        iterator.position += 1;
        // A fresh yield makes a NEW entry current, so the previous removal no
        // longer bars one.
        iterator.removed_current = false;
        Ok(Value::IterationYield(Box::new(value)))
    }
}
