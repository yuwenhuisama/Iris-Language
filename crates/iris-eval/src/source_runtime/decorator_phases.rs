use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{
    DecoratorKind, DecoratorPhase, DecoratorReason, DecoratorValue,
};
use iris_runtime::{ClassId, DispatchOutcome, Value};
use iris_syntax::{Decorator, MethodDeclaration, MethodKind};
use std::collections::HashMap;

pub(super) struct PhaseTarget {
    pub kind: DecoratorKind,
    pub reason: DecoratorReason,
    pub metadata: Value,
    pub candidate: Option<ClassId>,
}

pub(super) fn kind_error(planning: bool) -> EvaluationError {
    EvaluationError::DecoratorDiagnostic {
        code: "IRIS-DECORATOR-KIND",
        phase: if planning {
            "static"
        } else {
            "candidate validation"
        },
    }
}

impl SourceEvaluator {
    pub(super) fn transformation_operation(
        &self,
        previous: Option<&iris_runtime::decorator_protocol::Transformation>,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        use super::decorator_errors::phase_error;
        use iris_runtime::decorator_protocol::{Operation, Transformation};
        let phase = self.decorator_phase_stack.last();
        let empty = Transformation::empty(phase).map_err(phase_error)?;
        let transformation = previous.unwrap_or(&empty);
        let operation = match (selector, arguments) {
            ("empty", []) => return Ok(Some(DecoratorValue::Transformation(empty).into())),
            ("add_method", [Value::Symbol(selector), body]) => Operation::AddMethod {
                selector: selector.clone(),
                body: body.clone(),
            },
            ("wrap_method", [wrapper]) => Operation::WrapMethod(wrapper.clone()),
            ("wrap_getter", [wrapper]) => Operation::WrapGetter(wrapper.clone()),
            ("wrap_setter", [wrapper]) => Operation::WrapSetter(wrapper.clone()),
            _ => return Err(EvaluationError::ArgumentError),
        };
        transformation
            .append(phase, operation)
            .map(|record| Some(DecoratorValue::Transformation(record).into()))
            .map_err(|error| match error {
                iris_runtime::decorator_protocol::ConstructionError::Kind(_) => {
                    kind_error(self.decorator_planning)
                }
                error => phase_error(error),
            })
    }

    pub(super) fn execute_decorator_phase(
        &mut self,
        decorator: &Decorator,
        target: PhaseTarget,
    ) -> Result<Value, EvaluationError> {
        let class = self
            .class_name(&decorator.name)?
            .ok_or(EvaluationError::NameError)?;
        let contract_name = match target.kind {
            DecoratorKind::Class => "ClassDecorator",
            DecoratorKind::Module => "ModuleDecorator",
            DecoratorKind::Contract => "ContractDecorator",
            DecoratorKind::Method => "MethodDecorator",
            DecoratorKind::Property => "PropertyDecorator",
        };
        let contract =
            iris_runtime::core_contract_id(contract_name).ok_or(EvaluationError::NameError)?;
        if !self
            .class_contracts
            .get(&class)
            .is_some_and(|contracts| contracts.contains(&contract))
        {
            return Err(kind_error(self.decorator_planning));
        }
        self.validate_candidate_contracts(class)?;
        let selector = self.selector(if self.decorator_planning {
            "plan"
        } else {
            "transform"
        });
        let DispatchOutcome::Invoke(method) = self
            .runtime
            .registry()
            .dispatch(class, selector)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)?
        else {
            return Err(EvaluationError::TypeContractError);
        };
        self.decorator_phase_stack
            .push(DecoratorPhase::new(target.kind, target.reason));
        let previous_open = self.open_target;
        let previous_decorating = self.decorating_target;
        self.open_target = target.candidate;
        self.decorating_target = target.candidate;
        let result = (|| {
            let arguments = decorator
                .arguments
                .iter()
                .map(|argument| self.expression(argument, &HashMap::new(), None))
                .collect::<Result<Vec<_>, _>>()?;
            let receiver = Value::Object(self.construct(class, &[])?);
            let mut operands = vec![target.metadata, Value::ReadonlyArray(arguments)];
            if !self.decorator_planning {
                let phase = self
                    .decorator_phase_stack
                    .last()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                operands.push(DecoratorValue::Context(phase.context()).into());
            }
            let result = self.invoke_method(method, receiver, &operands)?;
            match &result {
                Value::Decorator(record) => match record.as_ref() {
                    DecoratorValue::Plan(plan) if self.decorator_planning => {
                        plan.validate_kind(target.kind)
                            .map_err(|_| kind_error(true))?;
                    }
                    DecoratorValue::Transformation(transformation) if !self.decorator_planning => {
                        transformation
                            .validate_kind(target.kind)
                            .map_err(|_| kind_error(false))?;
                    }
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                },
                _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
            }
            Ok(result)
        })();
        if let Some(mut phase) = self.decorator_phase_stack.pop() {
            phase.finish();
        }
        self.open_target = previous_open;
        self.decorating_target = previous_decorating;
        result
    }

    pub(super) fn transform_method_decorators(
        &mut self,
        class: ClassId,
        method: &MethodDeclaration,
        reason: DecoratorReason,
    ) -> Result<(), EvaluationError> {
        let mut chains: Vec<(MethodDeclaration, Vec<iris_runtime::ObjectId>)> = Vec::new();
        if (!method.type_parameters.is_empty() || self.generic_definitions.contains(&class))
            && !method.decorators.is_empty()
        {
            chains.push((method.clone(), Vec::new()));
        }
        for decorator in &method.decorators {
            let metadata = self.method_decorator_metadata(class, method)?;
            let result = self.execute_decorator_phase(
                decorator,
                PhaseTarget {
                    kind: method_kind(method),
                    reason,
                    metadata,
                    candidate: Some(class),
                },
            )?;
            let Value::Decorator(record) = result else {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            };
            let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            };
            for operation in transformation.operations() {
                use iris_runtime::decorator_protocol::Operation;
                let (target, wrapper) = match operation {
                    Operation::WrapMethod(wrapper) => (method.clone(), wrapper),
                    Operation::WrapGetter(wrapper) | Operation::WrapSetter(wrapper) => {
                        let name = method.selector.trim_end_matches('=');
                        let name = match operation {
                            Operation::WrapSetter(_) => format!("{name}="),
                            _ => name.to_owned(),
                        };
                        let selector = self.selector(&name);
                        let target = self
                            .candidate_method_declaration(class, selector)
                            .filter(|target| target.kind == MethodKind::Property)
                            .cloned()
                            .ok_or_else(|| {
                                self.core_boundary_error(EvaluationError::ArgumentError)
                            })?;
                        (target, wrapper)
                    }
                    Operation::AddMethod { .. } => return Err(kind_error(false)),
                };
                let wrapper = self.admit_wrapper(wrapper, target.is_async)?;
                if let Some((_, wrappers)) = chains
                    .iter_mut()
                    .find(|(declaration, _)| declaration.selector == target.selector)
                {
                    wrappers.push(wrapper);
                } else {
                    chains.push((target, vec![wrapper]));
                }
            }
        }
        for (target, wrappers) in chains {
            self.install_wrapper_chain(class, &target, wrappers)?;
        }
        Ok(())
    }
}

pub(super) const fn method_kind(method: &MethodDeclaration) -> DecoratorKind {
    match method.kind {
        MethodKind::Property => DecoratorKind::Property,
        MethodKind::Instance | MethodKind::Class | MethodKind::Module => DecoratorKind::Method,
    }
}
