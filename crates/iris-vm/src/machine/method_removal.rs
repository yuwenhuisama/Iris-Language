use super::{Machine, MachineError, OpenGroupState, VerifyError};
use crate::compile::Program;
use iris_runtime::{ClassId, Method, MroEntry, Selector};

#[derive(Clone, Copy)]
pub(super) enum PendingMethod {
    Function(usize),
    Removed,
    Undefined,
}

impl Machine {
    pub(super) fn effective_candidate_method(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<Option<Method>, MachineError> {
        let registry = self.runtime.registry();
        for entry in registry.active(class).map_err(MachineError::Class)?.mro() {
            let method = match entry {
                MroEntry::Class(owner) => {
                    match self.pending_replacements.get(&(*owner, selector, false)) {
                        Some(PendingMethod::Undefined) => return Ok(None),
                        Some(PendingMethod::Removed) => continue,
                        Some(PendingMethod::Function(_)) => registry
                            .staged_method(*owner, selector)
                            .and_then(|method| registry.method_by_id(method)),
                        None => {
                            let active = registry.active(*owner).map_err(MachineError::Class)?;
                            if active.tombstones().contains(&selector) {
                                return Ok(None);
                            }
                            let method = if registry.is_staging(*owner) {
                                registry.staged_method(*owner, selector)
                            } else {
                                active.methods().get(&selector).copied()
                            };
                            method.and_then(|method| registry.method_by_id(method))
                        }
                    }
                }
                MroEntry::Module(module) => registry.module_method(*module, selector),
            };
            if method.is_some() {
                return Ok(method);
            }
        }
        Ok(None)
    }

    pub(super) fn validate_candidate_slot(
        &self,
        slot: (ClassId, Selector, bool),
        program: &Program,
    ) -> Result<(), MachineError> {
        let function = if slot.2 {
            match self.pending_replacements.get(&slot) {
                Some(PendingMethod::Function(function)) => *function,
                Some(PendingMethod::Removed | PendingMethod::Undefined) | None => return Ok(()),
            }
        } else {
            match self.effective_candidate_method(slot.0, slot.1)? {
                Some(method) => usize::try_from(method.body().raw()).map_err(|_| {
                    MachineError::Invalid(VerifyError::UnknownFunction {
                        function: usize::MAX,
                    })
                })?,
                None => return Ok(()),
            }
        };
        self.validate_replacement((slot, function), program)
    }

    pub(super) fn remove_checked(
        &mut self,
        target: (ClassId, Selector, bool),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, selector, undefine) = target;
        let slot = (class, selector, false);
        let immediate = self.open_depth == 0;
        let outcome = (|| {
            self.runtime
                .registry_mut()
                .begin_transaction(class)
                .map_err(MachineError::Class)?;
            let change = if undefine {
                self.runtime.registry_mut().undef_method(class, selector)
            } else {
                self.runtime.registry_mut().remove_method(class, selector)
            };
            change.map_err(MachineError::Class)?;
            self.pending_replacements.insert(
                slot,
                if undefine {
                    PendingMethod::Undefined
                } else {
                    PendingMethod::Removed
                },
            );
            if immediate {
                self.validate_candidate_slot(slot, program)?;
                self.runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(MachineError::Class)?;
            }
            Ok(())
        })();
        if immediate {
            self.pending_replacements.remove(&slot);
            if outcome.is_err() {
                self.runtime.registry_mut().roll_back_transaction(class);
            }
        } else if outcome.is_err() {
            self.open_group_state = Some(OpenGroupState::Aborted);
        }
        outcome
    }
}
