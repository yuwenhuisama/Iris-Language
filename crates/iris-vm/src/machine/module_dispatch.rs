use super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::{ClassId, DispatchError, Method, MethodOwner, ModuleId, Value, Visibility};

impl Machine {
    pub(super) fn module_main_receiver(&self, method: Method, program: &Program) -> Option<Value> {
        let MethodOwner::Module(module) = method.owner() else {
            return None;
        };
        let function = self.resolve_method_body(method.body(), program).ok()?;
        let body = function.program.functions.get(function.function)?;
        let receiver = body
            .signature
            .as_ref()
            .is_some_and(|signature| body.parameters == signature.parameters.len() + 1);
        if receiver || self.wrapper_chains.contains_key(&method.id()) {
            return function
                .program
                .link
                .as_ref()?
                .modules
                .borrow()
                .iter()
                .find_map(|(name, identity)| {
                    (*identity == module).then(|| Value::Symbol(name.clone()))
                });
        }
        None
    }
    pub(super) fn active_module_id(&self, name: &str) -> Result<ModuleId, MachineError> {
        let module = self
            .modules
            .iter()
            .find_map(|(known, module)| (known == name).then_some(*module))
            .ok_or(MachineError::NameError)?;
        self.runtime
            .registry()
            .active_module(module)
            .map_err(|_| MachineError::NameError)?;
        Ok(module)
    }

    pub(super) fn invoke_resolved_function(
        &mut self,
        function: usize,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let body = &program.functions[function];
        if let Some((owner, selector)) = body.name.split_once('.')
            && program.modules.iter().any(|module| module.name == owner)
        {
            return self.invoke_module_member(
                (owner, selector, Some(owner)),
                arguments,
                program,
                classes,
            );
        }
        let explicit = std::mem::take(&mut self.explicit_method_types);
        let bindings = match &body.signature {
            Some(signature) => {
                let receiver = usize::from(body.parameters == signature.parameters.len() + 1);
                self.select_method_types((signature, &arguments[receiver..]), explicit)?
            }
            None if explicit.is_empty() => Vec::new(),
            None => return Err(MachineError::ArgumentError),
        };
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = self.invoke_function(function, arguments, program, classes);
        self.method_types = previous;
        result
    }

    pub(super) fn invoke_module_member(
        &mut self,
        address: (&str, &str, Option<&str>),
        arguments: Vec<Value>,
        program: &Program,
        _classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let (owner, name, caller) = address;
        let module = self.active_module_id(owner)?;
        if name == "rollback" {
            let flattened = super::prepared_call::native_block_arguments(
                &Value::Symbol(owner.into()),
                name,
                &arguments,
            )?;
            return self.rollback_module(module, flattened.as_deref().unwrap_or(&arguments));
        }
        let selector = selector_id(program, name)
            .or_else(|| self.dynamic_selectors.get(name).copied())
            .ok_or(MachineError::NameError)?;
        let method = self
            .runtime
            .registry()
            .module_method(module, selector)
            .ok_or(MachineError::NameError)?;
        if method.visibility() != Visibility::Public && caller != Some(owner) {
            return Err(MachineError::Construction(
                DispatchError::VisibilityDenied { selector }.into(),
            ));
        }
        let function = self
            .resolve_method_body(method.body(), program)
            .map_err(|_| MachineError::UnsupportedConstruct)?;
        let code_owner = function;
        let (program, classes, function) = (
            code_owner.program.as_ref(),
            code_owner.classes.as_slice(),
            code_owner.function,
        );
        let body = &program.functions[function];
        let receiver = body
            .signature
            .as_ref()
            .is_some_and(|signature| body.parameters == signature.parameters.len() + 1);
        let mut passed = Vec::with_capacity(arguments.len() + usize::from(receiver));
        if receiver || self.wrapper_chains.contains_key(&method.id()) {
            passed.push(Value::Symbol(owner.into()));
        }
        passed.extend(arguments);
        if self.wrapper_chains.contains_key(&method.id()) {
            if !self.explicit_method_types.is_empty() {
                self.explicit_method_types.clear();
                return Err(MachineError::ArgumentError);
            }
            return self.invoke_wrapped(method, passed, program, classes);
        }
        let explicit = std::mem::take(&mut self.explicit_method_types);
        let bindings = match &body.signature {
            Some(signature) => {
                self.select_method_types((signature, &passed[usize::from(receiver)..]), explicit)?
            }
            None if explicit.is_empty() => Vec::new(),
            None => return Err(MachineError::ArgumentError),
        };
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = self.invoke_function(function, passed, program, classes);
        self.method_types = previous;
        result
    }
}
