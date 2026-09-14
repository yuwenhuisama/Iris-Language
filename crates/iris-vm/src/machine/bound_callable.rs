use super::{Machine, MachineError};
use crate::compile::Program;
use iris_runtime::{BoundMethod, BoundReceiver, ClassId, Value};

impl Machine {
    pub(super) fn direct_owner_bindings(
        &self,
        method: iris_runtime::Method,
        receiver: Option<&Value>,
        (program, classes): (&Program, &[ClassId]),
    ) -> Result<Vec<(String, Value)>, MachineError> {
        let iris_runtime::MethodOwner::Class(owner) = method.owner() else {
            return Ok(Vec::new());
        };
        let Some(index) = classes.iter().position(|class| *class == owner) else {
            return Ok(Vec::new());
        };
        if program.classes[index].type_parameters.is_empty() {
            return Ok(Vec::new());
        }
        let Some(Value::Object(object)) = receiver else {
            return Err(MachineError::UnsupportedConstruct);
        };
        if self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?
            != owner
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        let arguments = self
            .runtime
            .type_arguments_of(*object)
            .map_err(MachineError::Construction)?;
        if arguments.len() != program.classes[index].type_parameters.len() {
            return Err(MachineError::UnsupportedConstruct);
        }
        let signature = self
            .method_signatures
            .get(&method.id())
            .ok_or(MachineError::UnsupportedConstruct)?;
        if !matches!(
            signature.kind,
            iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Property
        ) || signature.impl_contract == Some(None)
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        self.receiver_type_bindings(&Value::Object(*object), (program, classes))
    }

    pub(super) fn receiver_type_bindings(
        &self,
        receiver: &Value,
        (program, classes): (&Program, &[ClassId]),
    ) -> Result<Vec<(String, Value)>, MachineError> {
        let (class, arguments) = match receiver {
            Value::Object(object) => (
                self.runtime
                    .class_of(*object)
                    .map_err(MachineError::Construction)?,
                self.runtime
                    .type_arguments_of(*object)
                    .map_err(MachineError::Construction)?
                    .to_vec(),
            ),
            Value::ClosedClass(class, arguments) => (*class, arguments.clone()),
            _ => return Ok(Vec::new()),
        };
        let Some(index) = classes.iter().position(|held| *held == class) else {
            return Ok(Vec::new());
        };
        Ok(program.classes[index]
            .type_parameters
            .iter()
            .cloned()
            .zip(
                arguments
                    .into_iter()
                    .map(|argument| Value::Type(argument.class(), argument.arguments().to_vec())),
            )
            .collect())
    }

    pub(super) fn invoke_bound_callable(
        &mut self,
        bound: &BoundMethod,
        arguments: &[Value],
        (program, classes): (&Program, &[ClassId]),
    ) -> Result<Value, MachineError> {
        let method = bound.method();
        let receiver = match bound.receiver() {
            BoundReceiver::Object(object) => {
                let class = self
                    .runtime
                    .class_of(object)
                    .map_err(MachineError::Construction)?;
                self.runtime
                    .registry()
                    .validate_method_binding(class, method)
                    .map_err(iris_runtime::ConstructionError::from)
                    .map_err(MachineError::Construction)?;
                Some(Value::Object(object))
            }
            BoundReceiver::Class(class) => {
                self.runtime
                    .registry()
                    .validate_method_binding(class, method)
                    .map_err(iris_runtime::ConstructionError::from)
                    .map_err(MachineError::Construction)?;
                Some(Value::Class(class))
            }
            BoundReceiver::Module(module) => {
                if method.owner() != iris_runtime::MethodOwner::Module(module) {
                    return Err(MachineError::UnsupportedConstruct);
                }
                self.module_main_receiver(method, program)
            }
        };
        let mut passed = Vec::with_capacity(arguments.len() + usize::from(receiver.is_some()));
        passed.extend(receiver);
        passed.extend_from_slice(arguments);
        if self.wrapper_chains.contains_key(&method.id()) {
            return self.invoke_wrapped(method, passed, program, classes);
        }
        let owner = self
            .resolve_method_body(method.body(), program)
            .map_err(|error| error.with_overflow(MachineError::UnsupportedConstruct))?;
        if self.closed_method_bindings(method).is_none()
            && !self
                .direct_owner_bindings(method, passed.first(), (&owner.program, &owner.classes))?
                .is_empty()
        {
            return self.invoke_selected_method(method, passed, &owner.program, &owner.classes);
        }
        let bindings = self.closed_method_bindings(method).unwrap_or_default();
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = self
            .resolve_method_body(method.body(), program)
            .map_err(|error| error.with_overflow(MachineError::UnsupportedConstruct))
            .and_then(|owner| {
                self.invoke_function(owner.function, passed, &owner.program, &owner.classes)
            });
        self.method_types = previous;
        result
    }
}
