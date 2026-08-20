use iris_runtime::{ArrayRef, ClassId, HashRef, Value};

use super::{Machine, MachineError, truthy, value_class_name};
use crate::compile::{Function, Program};

impl Machine {
    pub(super) fn authored_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let result = match receiver {
            Value::Array(values) => {
                self.array_send(values, receiver, selector, arguments, program, classes)?
            }
            Value::Hash(entries) => {
                self.hash_send(entries, selector, arguments, program, classes)?
            }
            Value::Text(text) => text_send(text, selector, arguments),
            _ => None,
        };
        if result.is_none() && authored_selector(selector) {
            return Err(MachineError::MessageNotFound {
                receiver_class: value_class_name(receiver).to_owned(),
                selector: selector.to_owned(),
            });
        }
        Ok(result)
    }

    fn array_send(
        &mut self,
        values: &ArrayRef,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Option<Value>, MachineError> {
        let result = match (selector, arguments) {
            ("length", []) => Value::Integer((values.len() as u64).into()),
            ("map", [block @ Value::Closure(_)]) => Value::Array(ArrayRef::new(
                values
                    .elements()
                    .into_iter()
                    .map(|value| self.invoke_closure(block, &[value], program, classes))
                    .collect::<Result<_, _>>()?,
            )),
            ("each", [block @ Value::Closure(_)]) => {
                for value in values.elements() {
                    self.invoke_closure(block, &[value], program, classes)?;
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
            ("reduce", [initial, block @ Value::Closure(_)]) => {
                let mut result = initial.clone();
                for value in values.elements() {
                    result = self.invoke_closure(block, &[result, value], program, classes)?;
                }
                result
            }
            ("reduce", [block @ Value::Closure(_)]) => {
                let mut elements = values.elements().into_iter();
                let Some(mut result) = elements.next() else {
                    return Ok(Some(Value::Nil));
                };
                for value in elements {
                    result = self.invoke_closure(block, &[result, value], program, classes)?;
                }
                result
            }
            ("push", [value]) => {
                values.mutate(|elements| elements.push(value.clone()));
                receiver.clone()
            }
            ("pop", []) => values.mutate(Vec::pop).unwrap_or(Value::Nil),
            ("join", [Value::Text(separator)]) => {
                let rendered = values
                    .elements()
                    .iter()
                    .map(render_text)
                    .collect::<Option<Vec<_>>>();
                let Some(rendered) = rendered else {
                    return Ok(None);
                };
                Value::Text(rendered.join(separator))
            }
            ("first", []) => values.get(0).unwrap_or(Value::Nil),
            ("last", []) => values.elements().into_iter().last().unwrap_or(Value::Nil),
            ("reverse", []) => {
                let mut elements = values.elements();
                elements.reverse();
                Value::Array(ArrayRef::new(elements))
            }
            ("at", [Value::Integer(index)]) => super::resolve_index(index, values.len())
                .and_then(|index| values.get(index))
                .unwrap_or(Value::Nil),
            ("to_string", []) => {
                let rendered = values
                    .elements()
                    .iter()
                    .map(render_text)
                    .collect::<Option<Vec<_>>>();
                let Some(rendered) = rendered else {
                    return Ok(None);
                };
                Value::Text(format!("[{}]", rendered.join(", ")))
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn hash_send(
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
            ("delete", [key]) => {
                let position = entries.entries().iter().position(|(held, _)| held == key);
                position
                    .and_then(|position| entries.remove_at(position))
                    .unwrap_or(Value::Nil)
            }
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

    fn invoke_closure(
        &mut self,
        closure: &Value,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Closure(identity) = closure else {
            return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
        };
        let Some(record) = self.closures.get(identity).cloned() else {
            return Err(MachineError::Kernel(iris_runtime::KernelError::Type));
        };
        let Some(Function {
            instructions,
            registers,
            parameters,
            ..
        }) = program.functions.get(record.function).cloned()
        else {
            return Err(MachineError::Invalid(super::VerifyError::UnknownFunction {
                function: record.function,
            }));
        };
        let mut passed = record.captures;
        passed.extend_from_slice(arguments);
        if passed.len() != parameters {
            return Err(MachineError::Kernel(iris_runtime::KernelError::Arity));
        }
        self.run_body(&instructions, registers, passed, program, classes)
            .map(|values| values.into_iter().next().unwrap_or(Value::Nil))
    }
}

fn text_send(text: &str, selector: &str, arguments: &[Value]) -> Option<Value> {
    match (selector, arguments) {
        ("length", []) => Some(Value::Integer((text.chars().count() as u64).into())),
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

fn render_text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        Value::Integer(value) => Some(value.decimal_text()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Nil => Some("nil".to_owned()),
        _ => None,
    }
}

fn authored_selector(selector: &str) -> bool {
    matches!(
        selector,
        "length"
            | "size"
            | "map"
            | "each"
            | "select"
            | "reject"
            | "reduce"
            | "push"
            | "pop"
            | "join"
            | "first"
            | "last"
            | "reverse"
            | "at"
            | "to_string"
            | "keys"
            | "values"
            | "include?"
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
    )
}
