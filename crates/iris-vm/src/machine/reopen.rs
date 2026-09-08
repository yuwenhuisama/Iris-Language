use super::{Machine, MachineError, VerifyError, selector_id};
use crate::compile::{ClassReopen, Program};
use iris_runtime::{ClassId, Method, MethodBody, Selector, Visibility};

impl Machine {
    pub(super) fn remember_signature(&mut self, method: Method, program: &Program) {
        if let Ok(index) = usize::try_from(method.body().raw())
            && let Some(signature) = program
                .functions
                .get(index)
                .and_then(|body| body.signature.as_ref())
        {
            self.method_signatures
                .insert(method.id(), signature.clone());
        }
    }

    pub(super) fn validate_replacements(&self, program: &Program) -> Result<(), MachineError> {
        for &slot in self.pending_replacements.keys() {
            self.validate_candidate_slot(slot, program)?;
        }
        Ok(())
    }

    pub(super) fn validate_replacement(
        &self,
        replacement: ((ClassId, Selector, bool), usize),
        program: &Program,
    ) -> Result<(), MachineError> {
        let ((class, selector, singleton), function) = replacement;
        let mut promise = None;
        let active = self
            .runtime
            .registry()
            .active(class)
            .map_err(MachineError::Class)?;
        for entry in active.mro() {
            let method = match entry {
                iris_runtime::MroEntry::Class(owner) => {
                    let revision = self
                        .runtime
                        .registry()
                        .active(*owner)
                        .map_err(MachineError::Class)?;
                    let table = if singleton {
                        revision.singleton_methods()
                    } else {
                        revision.methods()
                    };
                    table.get(&selector).copied()
                }
                iris_runtime::MroEntry::Module(module) => {
                    if singleton {
                        None
                    } else {
                        self.runtime
                            .registry()
                            .module_method(*module, selector)
                            .map(|method| method.id())
                    }
                }
            };
            if let Some(method) = method {
                promise = self.method_signatures.get(&method);
                break;
            }
        }
        let Some(promise) = promise else {
            return Ok(());
        };
        let body = program
            .functions
            .get(function)
            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                function,
            }))?;
        let Some(signature) = body.signature.as_ref() else {
            return Err(MachineError::TypeContractError);
        };
        if iris_syntax::method_signature_compatible(signature, promise, |source, target| {
            super::nominal_relation::nominal_subtype(program, source, target)
        }) {
            Ok(())
        } else {
            Err(MachineError::TypeContractError)
        }
    }

    pub(super) fn apply_reopen(
        &mut self,
        program: &Program,
        classes: &[ClassId],
        class_index: usize,
        reopen_index: usize,
    ) -> Result<(), MachineError> {
        let class = *classes
            .get(class_index)
            .ok_or(MachineError::SerializationError)?;
        let reopen = program
            .classes
            .get(class_index)
            .and_then(|class| class.reopens.get(reopen_index))
            .ok_or(MachineError::SerializationError)?;
        self.apply_members((class, reopen), program)
    }

    pub(super) fn apply_members(
        &mut self,
        target: (ClassId, &ClassReopen),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, _) = target;
        self.runtime
            .registry_mut()
            .begin_transaction(class)
            .map_err(MachineError::Class)?;
        let outcome = self
            .stage_members(target, program)
            .and_then(|()| self.validate_replacements(program))
            .and_then(|()| {
                self.runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(MachineError::Class)
            });
        self.pending_replacements.clear();
        if outcome.is_err() {
            self.runtime.registry_mut().roll_back_transaction(class);
        }
        outcome
    }

    fn stage_members(
        &mut self,
        target: (ClassId, &ClassReopen),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, reopen) = target;
        for (singleton, table) in [(false, &reopen.methods), (true, &reopen.class_methods)] {
            for (name, function) in table {
                let selector = selector_id(program, name)
                    .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                let body = MethodBody::new(u64::try_from(*function).map_err(|_| {
                    MachineError::Invalid(VerifyError::UnknownFunction {
                        function: *function,
                    })
                })?);
                let method = if singleton {
                    self.runtime.registry_mut().publish_singleton_method(
                        class,
                        selector,
                        body,
                        Visibility::Public,
                    )
                } else {
                    self.runtime.registry_mut().publish_method(
                        class,
                        selector,
                        body,
                        Visibility::Public,
                    )
                }
                .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
                self.pending_replacements.insert(
                    (class, selector, singleton),
                    super::method_removal::PendingMethod::Function(*function),
                );
            }
        }
        Ok(())
    }
}
