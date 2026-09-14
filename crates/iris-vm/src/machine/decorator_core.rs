use iris_runtime::decorator_protocol::{
    ArgumentChanges, ConstructionError, Plan, SnapshotTypes, Transformation,
};
use iris_runtime::{ClassId, CoreClass, DecoratorValue, KernelError, NominalType, Value};

use super::{Machine, MachineError};

impl Machine {
    pub(super) fn register_core_records(&mut self) -> Result<(), MachineError> {
        self.kernel
            .register_decorator_classes(self.runtime.registry_mut())
            .map_err(MachineError::Kernel)?;
        for &name in crate::core_names::SUPPORT_TYPES {
            if !self.core_support.contains_key(name) {
                if name == "MetaCapabilityError" {
                    let object = self.builtin_class("Object")?;
                    let class = self
                        .runtime
                        .registry_mut()
                        .define_class(iris_runtime::StaticSpine::new(1), Some(object))
                        .map_err(MachineError::Class)?;
                    self.core_support.insert(name, class);
                    continue;
                }
                let core =
                    CoreClass::from_name(name).ok_or(MachineError::Kernel(KernelError::Type))?;
                let class = self
                    .runtime
                    .registry_mut()
                    .register_core_class(
                        core,
                        self.kernel
                            .class(iris_runtime::BuiltinClass::Object)
                            .map_err(MachineError::Kernel)?,
                    )
                    .map_err(MachineError::Class)?;
                self.core_support.insert(name, class);
            }
        }
        Ok(())
    }

    pub(super) fn snapshot_types(&self) -> Result<SnapshotTypes, MachineError> {
        let object = self.builtin_class("Object")?;
        let symbol = self.builtin_class("Symbol")?;
        Ok(SnapshotTypes {
            object: Value::Type(object, Vec::new()),
            symbol: Value::Type(symbol, Vec::new()),
            type_type: Value::Type(self.builtin_class("Type")?, Vec::new()),
            invocation_parameter: Value::Type(
                self.builtin_class("InvocationParameter")?,
                Vec::new(),
            ),
            symbol_object_tuple: Value::Type(
                self.builtin_class("Tuple")?,
                vec![
                    NominalType::new(symbol, Vec::new()),
                    NominalType::new(object, Vec::new()),
                ],
            ),
        })
    }

    pub(super) fn core_record_send(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, MachineError> {
        if let Value::Object(object) = receiver {
            let class = self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction)?;
            let name = match class {
                class if class == CoreClass::ArgumentError.id() => Some("ArgumentError"),
                class if class == CoreClass::TypeError.id() => Some("TypeError"),
                _ => None,
            };
            if let Some(name) = name {
                let answer = match selector {
                    "class" => Some(Value::Class(class)),
                    "class_name" => Some(Value::Symbol(name.into())),
                    "to_string" | "inspect" => Some(Value::Text(name.into())),
                    _ => None,
                };
                if answer.is_some() && !arguments.is_empty() {
                    return Err(MachineError::ArgumentError);
                }
                return Ok(answer);
            }
        }
        if matches!(receiver, Value::Contract(..)) && selector == "type" && arguments.is_empty() {
            return Ok(Some(receiver.clone()));
        }
        if let Value::Contract(identity, _) = receiver
            && selector == "reflect"
        {
            if !arguments.is_empty() {
                return Err(MachineError::ArgumentError);
            }
            return self
                .contracts
                .get(identity)
                .cloned()
                .map(|metadata| Some(Value::ImmutableHash(metadata)))
                .ok_or(MachineError::NameError);
        }
        if let Value::ImmutableHash(record) = receiver
            && self
                .decorator_metadata
                .iter()
                .any(|known| known.same(record))
        {
            if selector.ends_with('=') && selector != "==" {
                return Err(MachineError::ReadonlyMutation);
            }
            if let Some(value) = record.get(&Value::Symbol(selector.into())) {
                if !arguments.is_empty() {
                    return Err(MachineError::ArgumentError);
                }
                return Ok(Some(value.clone()));
            }
        }
        if let Value::Decorator(record) = receiver {
            if selector.ends_with('=') && !matches!(selector, "==" | "!=") {
                return Err(MachineError::ReadonlyMutation);
            }
            if matches!(record.as_ref(), DecoratorValue::Transformation(_))
                && matches!(
                    selector,
                    "add_method" | "wrap_method" | "wrap_getter" | "wrap_setter"
                )
            {
                let valid = match selector {
                    "add_method" => matches!(arguments, [_, _]),
                    _ => matches!(arguments, [_]),
                };
                if !valid
                    || arguments
                        .iter()
                        .any(|value| matches!(value, Value::KeywordArgument(..)))
                {
                    return Err(MachineError::ArgumentError);
                }
                if selector == "add_method" && !matches!(arguments[0], Value::Symbol(_)) {
                    return Err(self.raise_core_value(Value::Symbol("TypeError".into())));
                }
                let DecoratorValue::Transformation(previous) = record.as_ref() else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                return self
                    .transformation_operation(Some(previous), selector, arguments)
                    .map(Some);
            }
            if arguments.is_empty() {
                if selector == "class_name" {
                    return Ok(Some(Value::Symbol(record.core_name().into())));
                }
                if let Some(value) = record.read_field(selector, &self.snapshot_types()?) {
                    return Ok(Some(value));
                }
            }
            return Ok(None);
        }
        let Value::Class(class) = receiver else {
            return Ok(None);
        };
        let Some(name) = crate::core_names::RECORDS
            .iter()
            .copied()
            .find(|name| self.kernel.core_class(name) == Some(*class))
        else {
            return Ok(None);
        };
        let value = match (name, selector) {
            ("ArgumentChanges", "empty") => {
                if !arguments.is_empty() {
                    return Err(MachineError::ArgumentError);
                }
                DecoratorValue::ArgumentChanges(ArgumentChanges::empty()).into()
            }
            ("ArgumentChanges", "new") => {
                let keywords = arguments
                    .iter()
                    .map(|value| match value {
                        Value::KeywordArgument(name, value) => {
                            Ok((name.clone(), (**value).clone()))
                        }
                        _ => Err(MachineError::ArgumentError),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let changes = ArgumentChanges::parse(&keywords).map_err(|error| {
                    if error.is_type_error() {
                        self.raise_core_value(Value::Symbol("TypeError".into()))
                    } else {
                        MachineError::ArgumentError
                    }
                })?;
                DecoratorValue::ArgumentChanges(changes).into()
            }
            ("Plan", "empty") => {
                if !arguments.is_empty() {
                    return Err(MachineError::ArgumentError);
                }
                DecoratorValue::Plan(
                    Plan::empty(self.decorator_phases.last())
                        .map_err(|error| self.phase_error(error))?,
                )
                .into()
            }
            (
                "Transformation",
                "empty" | "add_method" | "wrap_method" | "wrap_getter" | "wrap_setter",
            ) => {
                let valid = match selector {
                    "empty" => arguments.is_empty(),
                    "add_method" => matches!(arguments, [_, _]),
                    _ => matches!(arguments, [_]),
                } && !arguments
                    .iter()
                    .any(|value| matches!(value, Value::KeywordArgument(..)));
                if !valid {
                    return Err(MachineError::ArgumentError);
                }
                if selector == "add_method" && !matches!(arguments[0], Value::Symbol(_)) {
                    return Err(self.raise_core_value(Value::Symbol("TypeError".into())));
                }
                self.transformation_operation(None, selector, arguments)?
            }
            (_, "new" | "call") => {
                return Err(MachineError::MessageNotFound {
                    receiver_class: name.into(),
                    selector: selector.into(),
                });
            }
            (_, "type") if arguments.is_empty() => Value::Type(*class, Vec::new()),
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    fn phase_error(&mut self, error: ConstructionError) -> MachineError {
        match error {
            ConstructionError::Protocol(error) => {
                self.raise_core_value(DecoratorValue::ProtocolError(error).into())
            }
            ConstructionError::Kind(_) => MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"),
        }
    }

    pub(super) fn raise_core_value(&mut self, value: Value) -> MachineError {
        let value = match self.core_boundary_value(value) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let context = self.named_failure_context(value.clone());
        MachineError::Raised(Box::new((value, context)))
    }

    pub(super) fn core_boundary_value(&mut self, value: Value) -> Result<Value, MachineError> {
        let class = match &value {
            Value::Symbol(name) if name == "ArgumentError" => CoreClass::ArgumentError,
            Value::Symbol(name) if name == "TypeError" => CoreClass::TypeError,
            _ => return Ok(value),
        };
        self.register_core_records()?;
        self.runtime
            .allocate(class.id())
            .map(Value::Object)
            .map_err(MachineError::Construction)
    }

    fn transformation_operation(
        &mut self,
        previous: Option<&Transformation>,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        use iris_runtime::decorator_protocol::Operation;
        let phase = self.decorator_phases.last();
        let empty = Transformation::empty(phase).map_err(|error| self.phase_error(error))?;
        let operation = match (selector, arguments) {
            ("empty", []) => return Ok(DecoratorValue::Transformation(empty).into()),
            ("add_method", [Value::Symbol(selector), body]) => Operation::AddMethod {
                selector: selector.clone(),
                body: body.clone(),
            },
            ("wrap_method", [wrapper]) => Operation::WrapMethod(wrapper.clone()),
            ("wrap_getter", [wrapper]) => Operation::WrapGetter(wrapper.clone()),
            ("wrap_setter", [wrapper]) => Operation::WrapSetter(wrapper.clone()),
            _ => return Err(MachineError::ArgumentError),
        };
        let record = previous
            .unwrap_or(&empty)
            .append(self.decorator_phases.last(), operation)
            .map_err(|error| self.phase_error(error))?;
        Ok(DecoratorValue::Transformation(record).into())
    }

    pub(super) fn core_type_test(
        &self,
        value: &Value,
        target: ClassId,
        arguments: &[NominalType],
    ) -> Result<Option<bool>, MachineError> {
        if let Some(name) = crate::core_names::RECORDS
            .iter()
            .copied()
            .find(|name| self.kernel.core_class(name) == Some(target))
        {
            return Ok(Some(
                arguments.is_empty()
                    && matches!(value, Value::Decorator(record) if record.core_name() == name),
            ));
        }
        if let Some(name) = self
            .core_support
            .iter()
            .find_map(|(name, class)| (*class == target).then_some(*name))
        {
            let admitted = match name {
                "Symbol" => matches!(value, Value::Symbol(_)),
                "Type" => matches!(value, Value::Type(..) | Value::ComposedType(_)),
                "Tuple" => matches!(value, Value::Tuple(_)),
                "ArgumentError" | "TypeError" => {
                    matches!(value, Value::Object(object) if self.runtime.class_of(*object).map_err(MachineError::Construction)? == target)
                }
                "MetaCapabilityError" => {
                    matches!(value, Value::Symbol(held) if held == name)
                }
                _ => return Err(MachineError::Kernel(KernelError::Type)),
            };
            return Ok(Some(admitted && arguments.is_empty()));
        }
        self.immutable_type_test(value, target, arguments)
    }
}
