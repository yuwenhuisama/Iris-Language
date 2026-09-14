use super::{Machine, MachineError, Program, Value, selector_id};

impl Machine {
    pub(super) fn dynamic_property_send(
        &mut self,
        send: (&Value, &str, &[Value]),
        program: &Program,
    ) -> Result<Option<Value>, MachineError> {
        let (Value::Object(object), selector, arguments) = send else {
            return Ok(None);
        };
        let name = selector.strip_suffix('=').unwrap_or(selector);
        let Some(slot) = self.dynamic_selectors.get(name).copied() else {
            return Ok(None);
        };
        let class = self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?;
        if let Some(method_selector) = selector_id(program, selector)
            && !matches!(
                self.runtime.registry().dispatch(class, method_selector),
                Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. })
            )
        {
            return Ok(None);
        }
        let revision = self
            .runtime
            .registry()
            .active(class)
            .map_err(MachineError::Class)?;
        if !revision
            .properties()
            .iter()
            .any(|property| property.selector() == slot)
            || revision
                .properties()
                .iter()
                .any(|property| Some(property.selector()) == selector_id(program, name))
        {
            return Ok(None);
        }
        let value = match (selector.ends_with('='), arguments) {
            (false, []) => self.runtime.raw_ivar(*object, slot),
            (true, [value]) => self.runtime.assign_raw_ivar(*object, slot, value.clone()),
            _ => return Err(MachineError::ArgumentError),
        }
        .map_err(MachineError::Construction)?;
        Ok(Some(value))
    }
}
