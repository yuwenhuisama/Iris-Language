use super::decorator_errors::{argument_error, phase_error};
use super::{Binding, EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{ArgumentChanges, DecoratorValue, Plan, SnapshotTypes};
use iris_runtime::{BuiltinClass, CoreClass, NominalType, Value};

mod core_errors;

pub(super) const RECORDS: [&str; 8] = [
    "Invocation",
    "InvocationSignature",
    "InvocationParameter",
    "ArgumentChanges",
    "DecoratorContext",
    "Plan",
    "Transformation",
    "DecoratorProtocolError",
];

impl SourceEvaluator {
    pub(super) fn install_decorator_core(&mut self) -> Result<(), EvaluationError> {
        let object = self
            .kernel
            .class(BuiltinClass::Object)
            .map_err(EvaluationError::Runtime)?;
        for name in ["Symbol", "Type", "Tuple", "TypeError", "ArgumentError"] {
            let core = CoreClass::from_name(name).ok_or(EvaluationError::NameError)?;
            let class = self
                .runtime
                .registry_mut()
                .register_core_class(core, object)
                .map_err(EvaluationError::Class)?;
            self.names
                .insert(name.into(), Binding::immutable(Value::Class(class)));
            self.names.insert(
                format!("Kernel::{name}"),
                Binding::immutable(Value::Class(class)),
            );
        }
        let symbol = self
            .class_name("Symbol")?
            .ok_or(EvaluationError::NameError)?;
        let type_class = self.class_name("Type")?.ok_or(EvaluationError::NameError)?;
        let tuple = self
            .class_name("Tuple")?
            .ok_or(EvaluationError::NameError)?;
        let parameter = self
            .kernel
            .core_class("InvocationParameter")
            .ok_or(EvaluationError::NameError)?;
        self.snapshot_types = Some(SnapshotTypes {
            object: Value::Type(object, Vec::new()),
            symbol: Value::Type(symbol, Vec::new()),
            type_type: Value::Type(type_class, Vec::new()),
            invocation_parameter: Value::Type(parameter, Vec::new()),
            symbol_object_tuple: Value::Type(
                tuple,
                vec![
                    NominalType::new(symbol, Vec::new()),
                    NominalType::new(object, Vec::new()),
                ],
            ),
        });
        for name in [
            "ClassDecorator",
            "ModuleDecorator",
            "ContractDecorator",
            "MethodDecorator",
            "PropertyDecorator",
        ] {
            let contract =
                iris_runtime::core_contract_id(name).ok_or(EvaluationError::NameError)?;
            self.decorator_contracts.push(contract);
            self.contract_names.insert(name.into(), contract);
            self.contract_names
                .insert(format!("Kernel::{name}"), contract);
            self.names.insert(
                name.into(),
                Binding::immutable(Value::Contract(contract, Vec::new())),
            );
            self.names.insert(
                format!("Kernel::{name}"),
                Binding::immutable(Value::Contract(contract, Vec::new())),
            );
            self.contract_parents.insert(contract, Vec::new());
            self.contract_requirements
                .insert(contract, vec!["plan".into(), "transform".into()]);
            let mut requirements = Vec::new();
            for (selector, arity, result) in
                [("plan", 2, "Plan"), ("transform", 3, "Transformation")]
            {
                self.contract_requirement_arities
                    .insert((contract, selector.into()), arity);
                self.contract_requirement_parameters.insert(
                    (contract, selector.into()),
                    vec![Some(iris_syntax::TypeExpression::Name("Object".into())); arity],
                );
                self.contract_requirement_returns.insert(
                    (contract, selector.into()),
                    iris_syntax::TypeExpression::Name(result.into()),
                );
                requirements.push(iris_syntax::MethodDeclaration {
                    decorators: Vec::new(),
                    is_async: false,
                    is_override: false,
                    impl_contract: None,
                    kind: iris_syntax::MethodKind::Instance,
                    selector: selector.into(),
                    type_parameters: Vec::new(),
                    parameters: (0..arity)
                        .map(|index| iris_syntax::Parameter {
                            name: format!("argument{index}"),
                            category: iris_syntax::ParameterCategory::Positional,
                            annotation: Some(iris_syntax::TypeExpression::Name("Object".into())),
                            default: None,
                        })
                        .collect(),
                    return_type: Some(iris_syntax::TypeExpression::Name(result.into())),
                    visibility: iris_syntax::Visibility::Public,
                    body: None,
                });
            }
            self.qualified_contracts
                .requirements
                .insert(contract, requirements);
        }
        Ok(())
    }

    pub(super) fn decorator_core_send(
        &self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        match receiver {
            Value::Class(class) => {
                let Some(name) = RECORDS
                    .iter()
                    .find(|name| self.kernel.core_class(name) == Some(*class))
                else {
                    return Ok(None);
                };
                match (*name, selector) {
                    ("ArgumentChanges", "empty") => {
                        if !arguments.is_empty() {
                            return Err(EvaluationError::ArgumentError);
                        }
                        Ok(Some(
                            DecoratorValue::ArgumentChanges(ArgumentChanges::empty()).into(),
                        ))
                    }
                    ("ArgumentChanges", "new") => {
                        let keywords = arguments
                            .iter()
                            .map(|argument| match argument {
                                Value::KeywordArgument(name, value) => {
                                    Ok((name.clone(), value.as_ref().clone()))
                                }
                                _ => Err(EvaluationError::ArgumentError),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let changes = ArgumentChanges::parse(&keywords).map_err(argument_error)?;
                        Ok(Some(DecoratorValue::ArgumentChanges(changes).into()))
                    }
                    ("Plan", "empty") => {
                        if !arguments.is_empty() {
                            return Err(EvaluationError::ArgumentError);
                        }
                        Plan::empty(self.decorator_phase_stack.last())
                            .map(|record| Some(DecoratorValue::Plan(record).into()))
                            .map_err(phase_error)
                    }
                    (
                        "Transformation",
                        "empty" | "add_method" | "wrap_method" | "wrap_getter" | "wrap_setter",
                    ) => {
                        validate_operation_arguments(selector, arguments)?;
                        self.transformation_operation(None, selector, arguments)
                    }
                    (_, "new") => Err(EvaluationError::MessageNotFound {
                        receiver_class: (*name).into(),
                        selector: selector.into(),
                    }),
                    _ => Ok(None),
                }
            }
            Value::Decorator(record) => {
                if selector.ends_with('=') && selector != "==" {
                    return Err(EvaluationError::ReadonlyMutation);
                }
                if matches!(record.as_ref(), DecoratorValue::Transformation(_))
                    && matches!(
                        selector,
                        "add_method" | "wrap_method" | "wrap_getter" | "wrap_setter"
                    )
                {
                    validate_operation_arguments(selector, arguments)?;
                    let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                        return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                    };
                    return self.transformation_operation(
                        Some(transformation),
                        selector,
                        arguments,
                    );
                }
                let types = self
                    .snapshot_types
                    .as_ref()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                if let Some(value) = record.read_field(selector, types) {
                    if !arguments.is_empty() {
                        return Err(EvaluationError::ArgumentError);
                    }
                    return Ok(Some(value));
                }
                match selector {
                    "class_name" if arguments.is_empty() => {
                        Ok(Some(Value::Symbol(record.core_name().into())))
                    }
                    "class" if arguments.is_empty() => {
                        Ok(self.kernel.core_class(record.core_name()).map(Value::Class))
                    }
                    "type" if arguments.is_empty() => Ok(self
                        .kernel
                        .core_class(record.core_name())
                        .map(|class| Value::Type(class, Vec::new()))),
                    "to_string" | "inspect" if arguments.is_empty() => {
                        Ok(Some(Value::Text(record.core_name().into())))
                    }
                    "class_name" | "class" | "type" | "to_string" | "inspect" => {
                        Err(EvaluationError::ArgumentError)
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}

fn validate_operation_arguments(
    selector: &str,
    arguments: &[Value],
) -> Result<(), EvaluationError> {
    if arguments
        .iter()
        .any(|value| matches!(value, Value::KeywordArgument(..) | Value::BlockArgument(_)))
    {
        return Err(EvaluationError::ArgumentError);
    }
    match (selector, arguments) {
        ("empty", [])
        | ("wrap_method" | "wrap_getter" | "wrap_setter", [_])
        | ("add_method", [Value::Symbol(_), _]) => Ok(()),
        ("add_method", [_, _]) => Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
        _ => Err(EvaluationError::ArgumentError),
    }
}
