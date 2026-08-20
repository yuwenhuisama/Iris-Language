use iris_runtime::{ArrayRef, ClassId, KernelError, Value};

use super::super::{Machine, MachineError, truthy};
use super::render_text;
use crate::compile::{Function, Program};

impl Machine {
    pub(super) fn reduce(
        &mut self,
        values: &ArrayRef,
        mut result: Value,
        block: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        for value in values.elements() {
            result = self.invoke_closure(block, &[result, value], program, classes)?;
        }
        Ok(result)
    }

    pub(super) fn extreme(
        &mut self,
        values: &ArrayRef,
        selector: &str,
    ) -> Result<Value, MachineError> {
        let mut elements = values.elements().into_iter();
        let Some(mut extreme) = elements.next() else {
            return Ok(Value::Nil);
        };
        for candidate in elements {
            let Value::Integer(order) =
                self.send("<=>", extreme.clone(), std::slice::from_ref(&candidate))?
            else {
                return Err(MachineError::Kernel(KernelError::Type));
            };
            let Some(order) = order.to_i128() else {
                return Err(MachineError::Kernel(KernelError::Type));
            };
            if (selector == "min" && order > 0) || (selector == "max" && order < 0) {
                extreme = candidate;
            }
        }
        Ok(extreme)
    }

    pub(super) fn sorted(&mut self, values: &ArrayRef) -> Result<Vec<Value>, MachineError> {
        let mut sorted = Vec::new();
        for candidate in values.elements() {
            let mut position = sorted.len();
            for (index, held) in sorted.iter().enumerate() {
                let Value::Integer(order) =
                    self.send("<=>", candidate.clone(), std::slice::from_ref(held))?
                else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                if order.to_i128().is_some_and(|order| order < 0) {
                    position = index;
                    break;
                }
            }
            sorted.insert(position, candidate);
        }
        Ok(sorted)
    }

    pub(super) fn position(
        &mut self,
        values: &ArrayRef,
        needle: &Value,
    ) -> Result<Option<usize>, MachineError> {
        for (index, value) in values.elements().into_iter().enumerate() {
            if truthy(&self.send("==", value, std::slice::from_ref(needle))?) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    pub(super) fn unique(&mut self, values: &ArrayRef) -> Result<Vec<Value>, MachineError> {
        let mut unique = Vec::new();
        for value in values.elements() {
            if self
                .position(&ArrayRef::new(unique.clone()), &value)?
                .is_none()
            {
                unique.push(value);
            }
        }
        Ok(unique)
    }

    pub(super) fn rendered(&self, values: &ArrayRef) -> Result<Vec<String>, MachineError> {
        values
            .elements()
            .iter()
            .map(|value| render_text(value).ok_or(MachineError::Kernel(KernelError::Type)))
            .collect()
    }

    pub(super) fn invoke_closure(
        &mut self,
        closure: &Value,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Closure(identity) = closure else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        let Some(record) = self.closures.get(identity).cloned() else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        let Some(Function {
            instructions,
            registers,
            parameters,
            ..
        }) = program.functions.get(record.function).cloned()
        else {
            return Err(MachineError::Invalid(
                super::super::VerifyError::UnknownFunction {
                    function: record.function,
                },
            ));
        };
        let mut passed = record.captures;
        passed.extend_from_slice(arguments);
        if passed.len() != parameters {
            return Err(MachineError::Kernel(KernelError::Arity));
        }
        self.run_body(&instructions, registers, passed, program, classes)
            .map(|values| values.into_iter().next().unwrap_or(Value::Nil))
    }
}

pub(super) fn flatten(values: Vec<Value>) -> Vec<Value> {
    let mut flattened = Vec::new();
    for value in values {
        match value {
            Value::Array(values) => flattened.extend(flatten(values.elements())),
            value => flattened.push(value),
        }
    }
    flattened
}
