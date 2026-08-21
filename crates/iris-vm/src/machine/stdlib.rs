mod array;
mod hash_text;
mod support;

use iris_runtime::{ClassId, Value};

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
            Value::Symbol(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(value.clone()))
            }
            Value::Float64(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(float_text(*value)))
            }
            Value::Float32(value) if selector == "to_string" && arguments.is_empty() => {
                Some(Value::Text(float_text(f64::from(*value))))
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
