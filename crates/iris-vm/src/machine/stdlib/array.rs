use iris_runtime::{ArrayRef, ClassId, KernelError, Value};

use super::super::{Machine, MachineError, resolve_index, truthy};
use crate::compile::Program;

impl Machine {
    pub(super) fn array_send(
        &mut self,
        values: &ArrayRef,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let result = match (selector, arguments) {
            ("length" | "count", []) => Value::Integer((values.len() as u64).into()),
            // `C017` and `C018` require an Iterator to RELEASE its source, an
            // ownership fact rather than a timing one. This answers how many
            // live references share the Array body, so a vector observes the
            // release directly instead of needing a collector to run. It is a
            // conformance probe on REPRESENTATION, not part of the Array
            // surface the specification defines for programs.
            ("share_count", []) => Value::Integer(
                u64::try_from(values.share_count())
                    .unwrap_or_default()
                    .into(),
            ),
            ("map", [block @ Value::Closure(_)]) => Value::Array(ArrayRef::new(
                values
                    .elements()
                    .into_iter()
                    .map(|value| self.invoke_closure(block, &[value], program, classes))
                    .collect::<Result<_, _>>()?,
            )),
            ("each" | "each_with_index", [block @ Value::Closure(_)]) => {
                for (index, value) in values.elements().into_iter().enumerate() {
                    let arguments = if selector == "each" {
                        vec![value]
                    } else {
                        vec![value, Value::Integer((index as u64).into())]
                    };
                    self.invoke_closure(block, &arguments, program, classes)?;
                }
                receiver.clone()
            }
            ("select" | "reject", [block @ Value::Closure(_)]) => {
                let mut selected = Vec::new();
                for value in values.elements() {
                    let decision =
                        self.invoke_closure(block, std::slice::from_ref(&value), program, classes)?;
                    if truthy(&decision) == (selector == "select") {
                        selected.push(value);
                    }
                }
                Value::Array(ArrayRef::new(selected))
            }
            ("find", [block @ Value::Closure(_)]) => {
                let mut found = Value::Nil;
                for value in values.elements() {
                    if truthy(&self.invoke_closure(
                        block,
                        std::slice::from_ref(&value),
                        program,
                        classes,
                    )?) {
                        found = value;
                        break;
                    }
                }
                found
            }
            ("count", [block @ Value::Closure(_)]) => {
                let mut count = 0_u64;
                for value in values.elements() {
                    if truthy(&self.invoke_closure(block, &[value], program, classes)?) {
                        count = count.saturating_add(1);
                    }
                }
                Value::Integer(count.into())
            }
            ("reduce", [initial, block @ Value::Closure(_)]) => {
                self.reduce(values, initial.clone(), block, program, classes)?
            }
            ("reduce", [block @ Value::Closure(_)]) => {
                let mut elements = values.elements().into_iter();
                let Some(initial) = elements.next() else {
                    return Ok(Some(Value::Nil));
                };
                self.reduce(
                    &ArrayRef::new(elements.collect()),
                    initial,
                    block,
                    program,
                    classes,
                )?
            }
            ("sum", []) => {
                let mut sum = Value::Integer(0_u8.into());
                for value in values.elements() {
                    sum = self.send("+", sum, &[value])?;
                }
                sum
            }
            ("min" | "max", []) => self.extreme(values, selector)?,
            ("sort", []) => Value::Array(ArrayRef::new(self.sorted(values)?)),
            ("push", [value]) => {
                values.mutate(|elements| elements.push(value.clone()));
                receiver.clone()
            }
            // C024 makes append, insert, delete and clear the explicit growth
            // and removal operations, and they answer NIL rather than the
            // receiver - unlike `push`, which answers the Array. The
            // difference is observable, so the two spellings are not aliases.
            ("append", [value]) => {
                values.mutate(|elements| elements.push(value.clone()));
                Value::Nil
            }
            ("clear", []) => {
                values.mutate(Vec::clear);
                Value::Nil
            }
            // `delete` removes the FIRST equal element, using the in-order
            // element comparison C026 fixes as Array equality.
            ("delete", [target]) => {
                values.mutate(|elements| {
                    if let Some(position) = elements.iter().position(|held| held == target) {
                        elements.remove(position);
                    }
                });
                Value::Nil
            }
            ("insert", [Value::Integer(index), value]) => {
                // The END position is a valid insertion point, so the length
                // itself resolves even though it is out of range for a read.
                // The END position is a valid insertion point, which the
                // ordinary read resolver rejects because it is out of range
                // for a READ. Resolving against `length + 1` admits it while
                // still refusing anything past it.
                let length = values.len();
                let Some(position) = resolve_index(index, length.saturating_add(1)) else {
                    return Err(MachineError::IndexError);
                };
                if position > length {
                    return Err(MachineError::IndexError);
                }
                values.mutate(|elements| elements.insert(position, value.clone()));
                Value::Nil
            }
            ("pop", []) => values.mutate(Vec::pop).unwrap_or(Value::Nil),
            ("join", [Value::Text(separator)]) => {
                Value::Text(self.rendered(values)?.join(separator))
            }
            ("first", []) => values.get(0).unwrap_or(Value::Nil),
            ("last", []) => values.elements().into_iter().last().unwrap_or(Value::Nil),
            ("reverse", []) => {
                let mut result = values.elements();
                result.reverse();
                Value::Array(ArrayRef::new(result))
            }
            ("include?", [needle]) => Value::Bool(self.position(values, needle)?.is_some()),
            ("index_of", [needle]) => self
                .position(values, needle)?
                .map(|index| Value::Integer((index as u64).into()))
                .unwrap_or(Value::Nil),
            ("concat", [Value::Array(other)]) => {
                let mut result = values.elements();
                result.extend(other.elements());
                Value::Array(ArrayRef::new(result))
            }
            ("slice", [Value::Integer(start), Value::Integer(length)]) => {
                let elements = values.elements();
                let Some(start) = super::super::resolve_index(start, elements.len()) else {
                    return Ok(Some(Value::Array(ArrayRef::new(Vec::new()))));
                };
                let Some(length) = length.to_usize() else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                let end = start.saturating_add(length).min(elements.len());
                Value::Array(ArrayRef::new(elements[start..end].to_vec()))
            }
            ("take" | "drop", [Value::Integer(count)]) => {
                let Some(count) = count.to_usize() else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                let elements = values.elements();
                let split = count.min(elements.len());
                Value::Array(ArrayRef::new(if selector == "take" {
                    elements[..split].to_vec()
                } else {
                    elements[split..].to_vec()
                }))
            }
            ("uniq", []) => Value::Array(ArrayRef::new(self.unique(values)?)),
            ("flatten", []) => {
                Value::Array(ArrayRef::new(super::support::flatten(values.elements())))
            }
            ("all?" | "any?", [block @ Value::Closure(_)]) => {
                let seeking_all = selector == "all?";
                let mut answer = seeking_all;
                for value in values.elements() {
                    if truthy(&self.invoke_closure(block, &[value], program, classes)?)
                        != seeking_all
                    {
                        answer = !seeking_all;
                        break;
                    }
                }
                Value::Bool(answer)
            }
            ("at", [Value::Integer(index)]) => super::super::resolve_index(index, values.len())
                .and_then(|index| values.get(index))
                .unwrap_or(Value::Nil),
            ("to_string", []) => Value::Text(format!("[{}]", self.rendered(values)?.join(", "))),
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}
