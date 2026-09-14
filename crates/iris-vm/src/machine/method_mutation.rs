use super::{Machine, MachineError, OpenGroupState};
use crate::compile::Program;
use iris_runtime::{ClassId, Selector, Visibility};

impl Machine {
    pub(super) fn alias_checked(
        &mut self,
        target: (ClassId, Selector, Selector),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, alias, original) = target;
        let immediate = self.open_depth == 0;
        let outcome = (|| {
            self.runtime
                .registry_mut()
                .begin_transaction(class)
                .map_err(MachineError::Class)?;
            self.runtime
                .registry_mut()
                .alias_method(class, alias, original)
                .map_err(MachineError::Class)?;
            let method = self
                .runtime
                .registry()
                .staged_method(class, alias)
                .and_then(|method| self.runtime.registry().method_by_id(method))
                .ok_or(MachineError::TypeContractError)?;
            let function = self
                .resolve_method_body(method.body(), program)
                .map_err(|error| error.with_overflow(MachineError::TypeContractError))?;
            if immediate {
                self.validate_replacement(
                    ((class, alias, false), function.function),
                    &function.program,
                )?;
                self.runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(MachineError::Class)?;
            } else {
                self.pending_replacements.insert(
                    (class, alias, false),
                    super::method_removal::PendingMethod::Function(method.body()),
                );
            }
            Ok(())
        })();
        if immediate && outcome.is_err() {
            self.runtime.registry_mut().roll_back_transaction(class);
        }
        if !immediate && outcome.is_err() {
            self.open_group_state = Some(OpenGroupState::Aborted);
        }
        outcome
    }

    pub(super) fn define_checked(
        &mut self,
        target: (ClassId, Selector, usize),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, selector, function) = target;
        let outcome = (|| {
            if self.open_depth > 0 {
                self.runtime
                    .registry_mut()
                    .begin_transaction(class)
                    .map_err(MachineError::Class)?;
            } else {
                self.validate_replacement(((class, selector, false), function), program)?;
            }
            let body = self.code.body(function, program)?;
            let method = self
                .runtime
                .registry_mut()
                .publish_method(class, selector, body, Visibility::Public)
                .map_err(MachineError::Class)?;
            self.dynamic_methods.insert(method.id());
            self.remember_signature(method, program);
            if self.open_depth > 0 {
                self.pending_replacements.insert(
                    (class, selector, false),
                    super::method_removal::PendingMethod::Function(body),
                );
            }
            Ok(())
        })();
        if self.open_depth > 0 && outcome.is_err() {
            self.open_group_state = Some(OpenGroupState::Aborted);
        }
        outcome
    }
}
