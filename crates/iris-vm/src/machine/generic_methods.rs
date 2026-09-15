use super::decorator_wrappers::WrapperChain;
use super::{Machine, MachineError};
use crate::compile::Program;
use iris_runtime::decorator_protocol::{DecoratorReason, Operation};
use iris_runtime::{ClassId, DecoratorValue, Method, MethodId, Value};
use iris_syntax::{ParameterCategory, TypeExpression};

mod metadata;
#[cfg(test)]
mod tests;

pub(super) struct ClosedMethod {
    pub(super) canonical: MethodId,
    pub(super) types: Vec<Value>,
    pub(super) owner_bindings: Vec<(String, Value)>,
    pub(super) method: Method,
}

impl Machine {
    pub(super) fn with_method_types<T>(
        &mut self,
        bindings: Vec<(String, Value)>,
        run: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = run(self);
        self.method_types = previous;
        result
    }

    pub(super) fn closed_method_bindings(&self, method: Method) -> Option<Vec<(String, Value)>> {
        let closed = self
            .closed_methods
            .iter()
            .find(|closed| closed.method.id() == method.id())?;
        let signature = self.method_signatures.get(&method.id())?;
        Some(
            signature
                .type_parameters
                .iter()
                .cloned()
                .zip(closed.types.iter().cloned())
                .chain(closed.owner_bindings.iter().cloned())
                .collect(),
        )
    }

    pub(super) fn invoke_selected_method(
        &mut self,
        method: Method,
        arguments: Vec<Value>,
        program: &Program,
        _classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let qualifier = self.selected_qualifier.take();
        let function = self
            .resolve_method_body(method.body(), program)
            .map_err(|_| MachineError::UnsupportedConstruct)?;
        let owner = function;
        let (program, classes, function) = (
            owner.program.as_ref(),
            owner.classes.as_slice(),
            owner.function,
        );
        let explicit = std::mem::take(&mut self.explicit_method_types);
        let signature = program.functions[function]
            .signature
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        if !signature.type_parameters.is_empty()
            && (!matches!(
                signature.kind,
                iris_syntax::MethodKind::Class | iris_syntax::MethodKind::Instance
            ) || signature.impl_contract == Some(None))
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        let receiver_class = match arguments.first() {
            Some(Value::Class(class)) => Some(*class),
            Some(Value::Object(object)) => Some(
                self.runtime
                    .class_of(*object)
                    .map_err(MachineError::Construction)?,
            ),
            _ => None,
        };
        let owner_bindings =
            self.direct_owner_bindings(method, arguments.first(), (program, classes))?;
        if let Some(class) = receiver_class {
            self.runtime
                .registry()
                .validate_method_binding(class, method)
                .map_err(iris_runtime::ConstructionError::from)
                .map_err(MachineError::Construction)?;
        }
        let mut bindings = self.select_method_types((signature, &arguments[1..]), explicit)?;
        let types = bindings
            .iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        bindings.extend(owner_bindings.iter().cloned());
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = (|| {
            let selected = if types.is_empty() && owner_bindings.is_empty() {
                method
            } else {
                self.materialize_method(method, (types, owner_bindings), program)?
            };
            if self.wrapper_chains.contains_key(&selected.id()) {
                self.invoke_wrapped(selected, arguments, qualifier, program, classes)
            } else {
                self.invoke_function(function, arguments, program, classes)
            }
        })();
        self.method_types = previous;
        result
    }

    pub(super) fn select_method_types(
        &mut self,
        (signature, arguments): (&iris_syntax::MethodDeclaration, &[Value]),
        explicit: Vec<Value>,
    ) -> Result<Vec<(String, Value)>, MachineError> {
        let mut types = explicit;
        if types.is_empty() && !signature.type_parameters.is_empty() {
            for name in &signature.type_parameters {
                let position = signature.parameters.iter().position(|parameter| {
                    parameter.category == ParameterCategory::Positional
                        && parameter.annotation == Some(TypeExpression::Name(name.clone()))
                });
                let value = position
                    .and_then(|position| {
                        arguments
                            .iter()
                            .filter(|value| {
                                !matches!(
                                    value,
                                    Value::BlockArgument(_) | Value::KeywordArgument(..)
                                )
                            })
                            .nth(position)
                    })
                    .ok_or_else(|| self.decorator_type_error())?;
                let class = match value {
                    Value::Integer(_) => self.builtin_class("Integer")?,
                    Value::Text(_) => self.builtin_class("String")?,
                    Value::Bool(_) => self.builtin_class("Bool")?,
                    Value::Nil => self.builtin_class("Nil")?,
                    Value::Float32(_) => self.builtin_class("Float32")?,
                    Value::Float64(_) => self.builtin_class("Float64")?,
                    Value::Symbol(_) => self.builtin_class("Symbol")?,
                    _ => return Err(MachineError::UnsupportedConstruct),
                };
                types.push(Value::Type(class, Vec::new()));
            }
        }
        if types.len() != signature.type_parameters.len() {
            return Err(MachineError::ArgumentError);
        }
        if !types.is_empty()
            && signature.is_async
            && (!matches!(
                signature.kind,
                iris_syntax::MethodKind::Class | iris_syntax::MethodKind::Instance
            ) || signature.impl_contract == Some(None))
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        Ok(signature
            .type_parameters
            .iter()
            .cloned()
            .zip(types)
            .collect())
    }

    fn materialize_method(
        &mut self,
        canonical: Method,
        (types, owner_bindings): (Vec<Value>, Vec<(String, Value)>),
        program: &Program,
    ) -> Result<Method, MachineError> {
        if let Some(closed) = self.closed_methods.iter().find(|closed| {
            closed.canonical == canonical.id()
                && closed.types == types
                && closed.owner_bindings == owner_bindings
        }) {
            return Ok(closed.method);
        }
        let function = self
            .resolve_method_body(canonical.body(), program)
            .map_err(|_| MachineError::UnsupportedConstruct)?;
        let owner = function;
        let (program, classes, function) = (
            owner.program.as_ref(),
            owner.classes.as_slice(),
            owner.function,
        );
        let root_base = self.active_values.len();
        let result = (|| {
            let mut wrappers = Vec::new();
            for application in program
                .decorator_applications
                .iter()
                .filter(|application| application.method == Some(function))
            {
                let iris_runtime::MethodOwner::Class(class) = canonical.owner() else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                self.runtime
                    .registry()
                    .require_candidate_meta_capability(class, iris_runtime::Capability::MethodBody)
                    .map_err(MachineError::Class)?;
                let previous = std::mem::take(&mut self.method_types);
                let result = self.execute_decorator_phase_with_reason(
                    (application, Some(DecoratorReason::ClosedMaterialization)),
                    program,
                    classes,
                );
                self.method_types = previous;
                let Value::Decorator(record) = result? else {
                    return Err(self.decorator_type_error());
                };
                let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                    return Err(self.decorator_type_error());
                };
                for operation in transformation.operations() {
                    match operation {
                        Operation::WrapMethod(wrapper) => {
                            let callback = self.admit_wrapper(
                                wrapper,
                                program,
                                program.functions[function].is_async,
                            )?;
                            self.active_values.push(Value::Closure(callback));
                            wrappers.push(callback);
                        }
                        _ => return Err(MachineError::UnsupportedConstruct),
                    }
                }
            }
            let identity = self.allocate_managed_method_identity()?;
            let method = Method::new(
                identity,
                canonical.owner(),
                canonical.selector(),
                canonical.body(),
                canonical.visibility(),
            );
            self.remember_signature(method, program);
            if !wrappers.is_empty() {
                self.wrapper_chains.insert(
                    identity,
                    WrapperChain {
                        program: self.code.owner(program)?,
                        function,
                        wrappers,
                        body_closure: None,
                    },
                );
            }
            self.closed_methods.push(ClosedMethod {
                canonical: canonical.id(),
                types,
                owner_bindings,
                method,
            });
            Ok(method)
        })();
        self.active_values.truncate(root_base);
        result
    }
}
