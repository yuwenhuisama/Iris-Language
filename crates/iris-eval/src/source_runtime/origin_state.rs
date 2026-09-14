use super::{EvaluationError, SourceEvaluator, compound_selector};
use iris_runtime::{ClassId, Value};
use iris_syntax::Expression;
use std::collections::HashMap;

#[cfg(test)]
#[path = "module_origin_tests.rs"]
mod tests;

impl SourceEvaluator {
    pub(super) fn class_level_property(
        &mut self,
        class: ClassId,
        name: &str,
        initializer: Expression,
    ) -> Result<(), EvaluationError> {
        let slot = self.selector(name);
        if !self.decorator_planning {
            let value =
                self.expression(&initializer, &HashMap::new(), Some(Value::Class(class)))?;
            if self.origin_names.contains_key(&class) {
                self.runtime
                    .assign_staged_class_raw_ivar(class, slot, value)
            } else {
                self.runtime.assign_class_raw_ivar(class, slot, value)
            }
            .map_err(EvaluationError::Construction)?;
        }
        self.class_level_properties
            .entry(class)
            .or_default()
            .push(slot);
        Ok(())
    }

    pub(super) fn origin_state_expression(
        &mut self,
        expression: &Expression,
        context: (&HashMap<String, Value>, Option<&Value>),
    ) -> Result<Option<Value>, EvaluationError> {
        let (locals, receiver) = context;
        let Some(Value::Class(class)) = receiver else {
            return Ok(None);
        };
        let module_candidate = self
            .module_candidate
            .is_some_and(|module| self.module_classes.get(&module) == Some(class));
        if !self.origin_names.contains_key(class) && !module_candidate {
            return Ok(None);
        }
        let name = match expression {
            Expression::RawIvar(name) => name,
            Expression::Member { receiver, selector }
                if matches!(receiver.as_ref(), Expression::Name(name) if name == "self")
                    && self.is_class_level_property(*class, selector) =>
            {
                selector
            }
            Expression::Assignment {
                left,
                operator,
                right,
            } => {
                let name = match left.as_ref() {
                    Expression::RawIvar(name) => name,
                    Expression::Member { receiver, selector }
                        if matches!(receiver.as_ref(), Expression::Name(name) if name == "self")
                            && self.is_class_level_property(*class, selector) =>
                    {
                        selector
                    }
                    _ => return Ok(None),
                };
                let slot = self.selector(name);
                let value = self.expression(right, locals, receiver.cloned())?;
                let value = match compound_selector(operator) {
                    Some(operator) => {
                        let current = if module_candidate {
                            self.read_class_slot(*class, slot)?
                        } else {
                            self.runtime
                                .candidate_class_raw_ivar(*class, slot)
                                .map_err(EvaluationError::Construction)?
                        };
                        self.send(current, operator, &[value])?
                    }
                    None => value,
                };
                if module_candidate {
                    return self.write_class_slot(*class, slot, value).map(Some);
                }
                return self
                    .runtime
                    .assign_candidate_class_raw_ivar(*class, slot, value)
                    .map(Some)
                    .map_err(EvaluationError::Construction);
            }
            _ => return Ok(None),
        };
        let slot = self.selector(name);
        if module_candidate {
            return self.read_class_slot(*class, slot).map(Some);
        }
        self.runtime
            .candidate_class_raw_ivar(*class, slot)
            .map(Some)
            .map_err(EvaluationError::Construction)
    }
}
