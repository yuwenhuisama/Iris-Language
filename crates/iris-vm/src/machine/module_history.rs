use super::super::{Machine, MachineError, selector_id};
use crate::compile::{Program, decorators::Target};
use iris_runtime::{Capability, ModuleId, Value};

#[cfg(test)]
#[path = "module_history_tests.rs"]
mod tests;

impl Machine {
    pub(in crate::machine) fn rollback_module(
        &mut self,
        module: ModuleId,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        if self.open_depth > 0
            || self.decorator_planning
            || !self.decorator_phases.is_empty()
            || self.runtime.registry().staged_module(module).is_ok()
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        let (current, index) = self
            .code
            .module_owner(module)
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        let name = &current.modules[index].name;
        let (digest, source) = self.history_source((&current, name), arguments)?;
        let mut historical = crate::compile::history::compile_module(source, name)?;
        crate::compile::history::validate_module_source(&current.source, name)?;
        let target = historical
            .modules
            .iter()
            .position(|declaration| declaration.name == *name)
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        self.validate_module_history((module, target), (&current, &historical))?;
        historical.modules[target].meta_deny.clear();
        let historical = self.link_history(historical, &current)?;
        let classes = historical
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .classes
            .borrow()
            .clone();
        let metadata = self.decorator_metadata.len();
        self.decorator_planning = true;
        let planned = historical
            .decorator_applications
            .iter()
            .filter(|application| application.target == Target::Module(target))
            .try_for_each(|application| {
                self.execute_decorator_phase(application, &historical, &classes)
                    .map(|_| ())
            });
        self.decorator_planning = false;
        self.decorator_metadata.truncate(metadata);
        planned?;
        let modules = historical
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .modules
            .borrow()
            .clone();
        let previous = std::mem::replace(&mut self.modules, modules);
        let outcome = self.publish_module(target, &historical, &classes);
        self.modules = previous;
        outcome?;
        Ok(Value::Symbol(digest))
    }

    pub(super) fn validate_module_history(
        &mut self,
        (module, target): (ModuleId, usize),
        (current, historical): (&Program, &Program),
    ) -> Result<(), MachineError> {
        let active = self
            .runtime
            .registry()
            .active_module(module)
            .map_err(|_| MachineError::RevisionArtifactUnavailable)?;
        if !active.composition_edges().is_empty() {
            return Err(MachineError::UnsupportedConstruct);
        }
        let declaration = &historical.modules[target];
        if active.meta_capabilities()
            != super::super::runtime::meta_capabilities(&declaration.meta_deny)?
        {
            return Err(MachineError::MetaTransactionError);
        }
        let functions = &declaration
            .replay
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .functions;
        let selectors = self
            .code
            .selectors()
            .chain(self.dynamic_selectors.values().copied())
            .collect::<std::collections::HashSet<_>>();
        let methods = selectors
            .into_iter()
            .filter_map(|selector| active.method(selector))
            .collect::<Vec<_>>();
        if methods.len() != functions.len() {
            return Err(MachineError::TypeContractError);
        }
        for method in methods {
            let owner = self
                .resolve_method_body(method.body(), current)
                .map_err(|_| MachineError::UnsupportedConstruct)?;
            let body = &owner.program.functions[owner.function];
            if body.instructions.iter().any(|instruction| {
                matches!(
                    instruction,
                    crate::Instruction::NativeCall { .. }
                        | crate::Instruction::NativeFixture { .. }
                        | crate::Instruction::FfiOpen { .. }
                )
            }) {
                return Err(MachineError::UnsupportedConstruct);
            }
            let promise = body
                .signature
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?;
            if !promise.type_parameters.is_empty() {
                return Err(MachineError::UnsupportedConstruct);
            }
            let signature = functions
                .iter()
                .filter_map(|function| historical.functions[*function].signature.as_ref())
                .find(|signature| {
                    selector_id(current, &signature.selector) == Some(method.selector())
                })
                .ok_or(MachineError::TypeContractError)?;
            if promise.kind != signature.kind
                || promise.visibility != signature.visibility
                || !iris_syntax::method_signature_compatible(
                    signature,
                    promise,
                    |source, target| {
                        super::super::nominal_relation::nominal_subtype(current, source, target)
                    },
                )
            {
                return Err(MachineError::TypeContractError);
            }
        }
        if !active.meta_capabilities().allows(Capability::MethodBody) {
            return Err(self.raise_core_value(Value::Symbol("MetaCapabilityError".into())));
        }
        Ok(())
    }
}
