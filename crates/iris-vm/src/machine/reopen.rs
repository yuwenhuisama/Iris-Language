use super::{Machine, MachineError, VerifyError, selector_id};
use crate::compile::{ClassReopen, Program};
use iris_runtime::{ClassId, Method, Selector, Visibility};

#[cfg(test)]
mod tests;

impl Machine {
    pub(super) fn remember_signature(&mut self, method: Method, program: &Program) {
        if let Ok(index) = self.resolve_method_body(method.body(), program)
            && let Some(signature) = index
                .program
                .functions
                .get(index.function)
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
        let owner = self.code.register(program)?;
        let program = owner.as_ref();
        self.code.retain_classes(program, classes)?;
        program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .modules
            .replace(self.modules.clone());
        let class = *classes
            .get(class_index)
            .ok_or(MachineError::SerializationError)?;
        let reopen = program
            .classes
            .get(class_index)
            .and_then(|class| class.reopens.get(reopen_index))
            .ok_or(MachineError::SerializationError)?;
        let signatures = self.method_signatures.clone();
        let chains = self.wrapper_chains.clone();
        let qualified = self.qualified_methods.clone();
        let metadata = self.decorator_metadata.len();
        let selectors = self.dynamic_selectors.clone();
        let next_selector = self.next_dynamic_selector;
        self.runtime
            .registry_mut()
            .begin_transaction(class)
            .map_err(MachineError::Class)?;
        self.open_depth += 1;
        let outcome = (|| {
            for (name, _) in &reopen.mixins {
                let module = self
                    .modules
                    .iter()
                    .find_map(|(known, module)| (known == name).then_some(*module))
                    .ok_or(MachineError::NameError)?;
                self.runtime
                    .registry_mut()
                    .recompose_candidate(class, module, true)
                    .map_err(MachineError::Class)?;
            }
            self.stage_members((class, reopen), program)?;
            let target = crate::compile::decorators::Target::Reopen {
                class: class_index,
                artifact: reopen_index,
            };
            let applications = program
                .decorator_applications
                .iter()
                .filter(|application| application.target == target)
                .collect::<Vec<_>>();
            self.install_decorator_applications(class, &applications, program, classes)?;
            self.validate_replacements(program)?;
            self.validate_contract_obligations(class, class_index, reopen, program)?;
            self.runtime
                .registry_mut()
                .commit_transaction(class)
                .map_err(MachineError::Class)
        })();
        self.open_depth -= 1;
        self.pending_replacements.clear();
        if outcome.is_err() {
            self.runtime.registry_mut().roll_back_transaction(class);
            self.method_signatures = signatures;
            self.wrapper_chains = chains;
            self.qualified_methods.slots = qualified.slots;
            self.qualified_methods.records = qualified.records;
            self.qualified_methods.definitions = qualified.definitions;
            self.decorator_metadata.truncate(metadata);
            self.dynamic_selectors = selectors;
            self.next_dynamic_selector = next_selector;
        }
        outcome
    }

    fn validate_contract_obligations(
        &self,
        class: ClassId,
        class_index: usize,
        reopen: &ClassReopen,
        program: &Program,
    ) -> Result<(), MachineError> {
        for contract in &program.classes[class_index].contracts {
            for requirement in &program.contracts[*contract].requirements {
                let selector = selector_id(program, &requirement.selector)
                    .ok_or_else(|| MachineError::UnknownSelector(requirement.selector.clone()))?;
                let method = self
                    .effective_candidate_method(class, selector)?
                    .or_else(|| {
                        reopen.mixins.iter().rev().find_map(|(name, _)| {
                            self.modules
                                .iter()
                                .find_map(|(known, module)| (known == name).then_some(*module))
                                .and_then(|module| {
                                    self.runtime.registry().module_method(module, selector)
                                })
                        })
                    })
                    .ok_or(MachineError::TypeContractError)?;
                let implementation = self
                    .method_signatures
                    .get(&method.id())
                    .ok_or(MachineError::TypeContractError)?;
                let promise = program.contracts[*contract]
                    .signatures
                    .iter()
                    .rev()
                    .find(|signature| signature.selector == requirement.selector)
                    .ok_or(MachineError::TypeContractError)?;
                let arguments = program.classes[class_index]
                    .contract_arguments
                    .iter()
                    .find(|(index, _)| index == contract)
                    .map(|(_, arguments)| arguments.as_slice())
                    .unwrap_or_default();
                let promise = crate::compile::effective_contracts::closed_requirement(
                    &program.contracts[*contract],
                    arguments,
                    promise,
                );
                if !iris_syntax::method_signature_compatible(
                    implementation,
                    &promise,
                    |source, target| {
                        super::nominal_relation::nominal_subtype(program, source, target)
                    },
                ) {
                    return Err(MachineError::TypeContractError);
                }
            }
        }
        Ok(())
    }

    pub(super) fn apply_members(
        &mut self,
        target: (ClassId, &ClassReopen),
        program: &Program,
    ) -> Result<(), MachineError> {
        let owner = self.code.register(program)?;
        let program = owner.as_ref();
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

    pub(super) fn stage_members(
        &mut self,
        target: (ClassId, &ClassReopen),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (class, reopen) = target;
        for (contract, name, function) in &reopen.qualified_impls {
            let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
            let slot = (class, program.contract_identity(*contract), selector);
            let old = self
                .qualified_method(slot)
                .ok_or(MachineError::UnsupportedConstruct)?;
            let previous = self
                .method_signatures
                .get(&old.id())
                .ok_or(MachineError::UnsupportedConstruct)?;
            let selected = program.functions[*function]
                .signature
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?;
            if !iris_syntax::method_signature_compatible(selected, previous, |source, target| {
                super::nominal_relation::nominal_subtype(program, source, target)
            }) {
                return Err(MachineError::TypeContractError);
            }
            self.publish_qualified_method(slot, *function, program)?;
        }
        for (singleton, table) in [(false, &reopen.methods), (true, &reopen.class_methods)] {
            for (name, function) in table {
                let selector = selector_id(program, name)
                    .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                let body = self.code.body(*function, program)?;
                let visibility = match program.functions[*function]
                    .signature
                    .as_ref()
                    .map(|signature| signature.visibility)
                {
                    Some(iris_syntax::Visibility::Private) => Visibility::Private,
                    Some(iris_syntax::Visibility::Protected) => Visibility::Protected,
                    Some(iris_syntax::Visibility::Public) | None => Visibility::Public,
                };
                let method =
                    if singleton {
                        self.runtime
                            .registry_mut()
                            .publish_singleton_method(class, selector, body, visibility)
                    } else if program.functions[*function].signature.as_ref().is_some_and(
                        |signature| signature.kind == iris_syntax::MethodKind::Property,
                    ) {
                        let capability =
                            if self.effective_candidate_method(class, selector)?.is_some() {
                                iris_runtime::Capability::PropertyBody
                            } else {
                                iris_runtime::Capability::PropertySet
                            };
                        self.runtime
                            .registry()
                            .require_candidate_meta_capability(class, capability)
                            .map_err(MachineError::Class)?;
                        self.runtime
                            .registry_mut()
                            .publish_origin_method(class, selector, body, visibility)
                    } else {
                        self.runtime
                            .registry_mut()
                            .publish_method(class, selector, body, visibility)
                    }
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
                self.pending_replacements.insert(
                    (class, selector, singleton),
                    super::method_removal::PendingMethod::Function(body),
                );
            }
        }
        Ok(())
    }
}
