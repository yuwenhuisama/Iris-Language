use super::super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::ModuleId;

impl Machine {
    pub(super) fn stage_module_replay(
        &mut self,
        target: (ModuleId, usize),
        program: &Program,
    ) -> Result<(), MachineError> {
        let (module, index) = target;
        let declaration = &program.modules[index];
        self.runtime
            .registry_mut()
            .begin_module_transaction(module)
            .map_err(|error| self.module_candidate_error(error))?;
        if !declaration.mixins.is_empty() {
            self.runtime
                .registry_mut()
                .stage_module_composition(module, &[])
                .map_err(|error| self.module_candidate_error(error))?;
        }
        if !declaration.meta_deny.is_empty() {
            let policy = super::super::runtime::meta_capabilities(&declaration.meta_deny)?;
            self.runtime
                .registry_mut()
                .stage_module_policy(module, policy)
                .map_err(|error| self.module_candidate_error(error))?;
        }
        let artifact = declaration
            .replay
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        for &function in &artifact.functions {
            let signature = program.functions[function]
                .signature
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?;
            let selector =
                selector_id(program, &signature.selector).ok_or(MachineError::NameError)?;
            let original = self
                .runtime
                .registry()
                .staged_module(module)
                .map_err(|_| MachineError::MetaTransactionError)?
                .method(selector)
                .ok_or(MachineError::UnsupportedConstruct)?;
            let promise = self
                .method_signatures
                .get(&original.id())
                .ok_or(MachineError::TypeContractError)?;
            if !signature.is_override
                || signature.visibility != promise.visibility
                || !iris_syntax::method_signature_compatible(
                    signature,
                    promise,
                    |source, target| {
                        super::super::nominal_relation::nominal_subtype(program, source, target)
                    },
                )
            {
                return Err(MachineError::TypeContractError);
            }
            let body = self.code.body(function, program)?;
            let method = self
                .runtime
                .registry_mut()
                .stage_module_method_body(module, selector, body)
                .map_err(|error| self.module_candidate_error(error))?;
            self.remember_signature(method, program);
        }
        Ok(())
    }
}
