use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, Selector, Value};

impl SourceEvaluator {
    pub(super) fn read_class_slot(
        &mut self,
        class: ClassId,
        slot: Selector,
    ) -> Result<Value, EvaluationError> {
        if self
            .module_candidate
            .is_some_and(|module| self.module_classes.get(&module) == Some(&class))
            && self.runtime.registry().is_staging(class)
        {
            return self
                .runtime
                .candidate_class_raw_ivar(class, slot)
                .map_err(EvaluationError::Construction);
        }
        if let Some((initializer, context)) =
            self.lazy_class_properties.get(&(class, slot)).cloned()
        {
            let previous = self.rooted(self.wrapper_context());
            self.restore_wrapper_context(context);
            self.lexical_class = Some(class);
            let value = self.expression(
                &initializer,
                &std::collections::HashMap::new(),
                Some(Value::Class(class)),
            );
            self.restore_wrapper_context(std::rc::Rc::unwrap_or_clone(previous));
            self.write_class_slot(class, slot, value?)?;
        }
        match self.upgrade_state.slots.get(&(class, slot)) {
            Some(value) => Ok(value.clone()),
            None => self
                .runtime
                .class_raw_ivar(class, slot)
                .map_err(EvaluationError::Construction),
        }
    }

    pub(super) fn write_class_slot(
        &mut self,
        class: ClassId,
        slot: Selector,
        value: Value,
    ) -> Result<Value, EvaluationError> {
        let value = if self
            .module_candidate
            .is_some_and(|module| self.module_classes.get(&module) == Some(&class))
        {
            self.runtime
                .registry_mut()
                .begin_transaction(class)
                .map_err(EvaluationError::Class)?;
            self.runtime
                .assign_candidate_class_raw_ivar(class, slot, value)
                .map_err(EvaluationError::Construction)
        } else if self.upgrade_state.active {
            self.runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?;
            self.upgrade_state
                .slots
                .insert((class, slot), value.clone());
            Ok(value)
        } else {
            self.runtime
                .assign_class_raw_ivar(class, slot, value)
                .map_err(EvaluationError::Construction)
        }?;
        self.lazy_class_properties.remove(&(class, slot));
        Ok(value)
    }
}
