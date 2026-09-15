use super::super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::{DecoratorValue, ModuleId, Value};
use std::collections::BTreeMap;

impl Machine {
    pub(in crate::machine) fn install_module_upgrade_decorators(
        &mut self,
        module: ModuleId,
        index: usize,
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<(), MachineError> {
        let mut wrappers = BTreeMap::<usize, Vec<iris_runtime::ObjectId>>::new();
        for application in program.decorator_applications.iter().filter(|application| {
            application.target == crate::compile::decorators::Target::Module(index)
        }) {
            let value = self.execute_decorator_phase_with_reason(
                (
                    application,
                    Some(iris_runtime::decorator_protocol::DecoratorReason::Upgrade),
                ),
                program,
                classes,
            )?;
            let Value::Decorator(record) = value else {
                return Err(self.decorator_type_error());
            };
            let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                return Err(self.decorator_type_error());
            };
            for operation in transformation.operations() {
                let iris_runtime::decorator_protocol::Operation::WrapMethod(wrapper) = operation
                else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                let function = application
                    .method
                    .ok_or(MachineError::UnsupportedConstruct)?;
                let wrapper =
                    self.admit_wrapper(wrapper, program, program.functions[function].is_async)?;
                wrappers.entry(function).or_default().push(wrapper);
            }
        }
        for (function, wrappers) in wrappers {
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
                .ok_or(MachineError::NameError)?;
            let method = self
                .runtime
                .registry_mut()
                .stage_module_method_body(module, selector, original.body())
                .map_err(|error| self.module_candidate_error(error))?;
            self.remember_signature(method, program);
            self.wrapper_chains.insert(
                method.id(),
                super::WrapperChain {
                    program: self.code.owner(program)?,
                    function,
                    wrappers,
                    body_closure: None,
                },
            );
        }
        Ok(())
    }
    pub(in crate::machine) fn stage_module_replay(
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
