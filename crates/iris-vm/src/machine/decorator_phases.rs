use super::{Machine, MachineError, selector_id};
use crate::compile::decorators::Application;
use crate::compile::{Instruction, Program};
use iris_runtime::decorator_protocol::{DecoratorPhase, DecoratorReason};
use iris_runtime::{ClassId, DecoratorValue, Value};

impl Machine {
    pub(super) fn check_decorator_effect(
        &self,
        instruction: &Instruction,
    ) -> Result<(), MachineError> {
        if self.decorator_planning
            && matches!(
                instruction,
                Instruction::Print { .. }
                    | Instruction::LoadGlobal { .. }
                    | Instruction::StoreGlobal { .. }
                    | Instruction::LoadBinding { .. }
                    | Instruction::StoreBinding { .. }
                    | Instruction::PublishBinding { .. }
                    | Instruction::GetClassVar { .. }
                    | Instruction::SetClassVar { .. }
                    | Instruction::NativeCall { .. }
                    | Instruction::NativeFixture { .. }
                    | Instruction::FfiOpen { .. }
                    | Instruction::Await { .. }
                    | Instruction::HostRun { .. }
                    | Instruction::GateNew { .. }
                    | Instruction::GateComplete { .. }
                    | Instruction::OpenClass { .. }
                    | Instruction::DefineMethod { .. }
                    | Instruction::DeclareMethod { .. }
                    | Instruction::Reflection { .. }
                    | Instruction::Revision { .. }
            )
        {
            return Err(MachineError::LexicalDiagnostic(
                "IRIS-DECORATOR-NONDETERMINISTIC",
            ));
        }
        if !self.decorator_phases.is_empty() && matches!(instruction, Instruction::Await { .. }) {
            return Err(MachineError::MetaTransactionError);
        }
        if !self.decorator_phases.is_empty() && matches!(instruction, Instruction::OpenClass { .. })
        {
            return Err(MachineError::MetaTransactionError);
        }
        Ok(())
    }

    pub(super) fn execute_decorator_phase(
        &mut self,
        application: &Application,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        self.execute_decorator_phase_with_reason((application, None), program, classes)
    }

    pub(super) fn execute_decorator_phase_with_reason(
        &mut self,
        (application, reason): (&Application, Option<DecoratorReason>),
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        classes
            .get(application.decorator)
            .ok_or(MachineError::NameError)?;
        self.decorator_phases.push(DecoratorPhase::new(
            application.kind,
            reason.unwrap_or_else(|| if program.link.as_ref().is_some_and(|link| !link.history_methods.borrow().is_empty()) {
                DecoratorReason::Rollback
            } else if self.open_depth > 0
                || matches!(application.target, crate::compile::decorators::Target::Reopen { .. })
                || matches!(application.target, crate::compile::decorators::Target::Module(index)
                    if program.modules[index].replay.as_ref().is_some_and(|artifact| artifact.reopen)) {
                DecoratorReason::Open
            } else {
                DecoratorReason::Origin
            }),
        ));
        let result = (|| {
            let arguments =
                self.invoke_function(application.arguments, Vec::new(), program, classes)?;
            let Value::Array(arguments) = arguments else {
                return Err(MachineError::UnsupportedConstruct);
            };
            let construction = [Instruction::New {
                destination: 0,
                class: application.decorator,
                type_arguments: Vec::new(),
                first: 0,
                count: 0,
            }];
            let receiver =
                self.run_body(&construction, 1, Vec::new(), program, classes)?[0].clone();
            let Value::Object(object) = receiver else {
                return Err(MachineError::UnsupportedConstruct);
            };
            let phase_name = if self.decorator_planning {
                "plan"
            } else {
                "transform"
            };
            let selector = selector_id(program, phase_name).ok_or(MachineError::NameError)?;
            let method =
                match self.historical_method(classes[application.decorator], selector, program) {
                    Some(method) => method,
                    None => self
                        .runtime
                        .dispatch_instance(object, selector)
                        .map_err(MachineError::Construction)?,
                };
            let function = self
                .resolve_method_body(method.body(), program)
                .map_err(|_| MachineError::UnsupportedConstruct)?;
            let owner = function;
            if owner.program.functions[owner.function].is_async {
                return Err(MachineError::LexicalDiagnostic(
                    "IRIS-DECORATOR-NONDETERMINISTIC",
                ));
            }
            let (target_name, class) = match application.target {
                crate::compile::decorators::Target::Class(index)
                | crate::compile::decorators::Target::Reopen { class: index, .. } => {
                    (program.classes[index].name.as_str(), Some(classes[index]))
                }
                crate::compile::decorators::Target::Contract(index) => {
                    (program.contracts[index].name.as_str(), None)
                }
                crate::compile::decorators::Target::Module(index) => {
                    (program.modules[index].name.as_str(), None)
                }
            };
            let name = application
                .property
                .as_ref()
                .map(|property| property.name.as_str())
                .unwrap_or_else(|| {
                    application
                        .method
                        .and_then(|index| program.functions[index].signature.as_ref())
                        .map_or(target_name, |signature| signature.selector.as_str())
                });
            let mut fields = vec![
                (Value::Symbol("name".into()), Value::Symbol(name.into())),
                (
                    Value::Symbol("kind".into()),
                    Value::Symbol(application.kind.symbol().into()),
                ),
            ];
            if let Some(property) = &application.property {
                fields.push((
                    Value::Symbol("type".into()),
                    self.reify_type(&property.annotation, program, classes)?,
                ));
                fields.push((
                    Value::Symbol("visibility".into()),
                    Value::Symbol(
                        match property.visibility {
                            iris_syntax::Visibility::Public => "public",
                            iris_syntax::Visibility::Private => "private",
                            iris_syntax::Visibility::Protected => "protected",
                        }
                        .into(),
                    ),
                ));
            } else if let Some(function) = application.method {
                let signature = program.functions[function]
                    .signature
                    .as_ref()
                    .ok_or(MachineError::UnsupportedConstruct)?;
                fields.push((
                    Value::Symbol("signature".into()),
                    self.decorator_signature_metadata(application, (program, classes))?,
                ));
                fields.push((
                    Value::Symbol("selector".into()),
                    Value::Symbol(signature.selector.clone()),
                ));
                fields.push((
                    Value::Symbol("owner".into()),
                    match class {
                        Some(class) => Value::Type(class, Vec::new()),
                        None => Value::Symbol(target_name.into()),
                    },
                ));
                fields.push((
                    Value::Symbol("visibility".into()),
                    Value::Symbol(
                        match signature.visibility {
                            iris_syntax::Visibility::Public => "public",
                            iris_syntax::Visibility::Private => "private",
                            iris_syntax::Visibility::Protected => "protected",
                        }
                        .into(),
                    ),
                ));
            } else if let Some(class) = class {
                fields.push((Value::Symbol("type".into()), Value::Type(class, Vec::new())));
            }
            let metadata = match application.target {
                crate::compile::decorators::Target::Contract(index) => {
                    self.contract_metadata(index, program)?
                }
                crate::compile::decorators::Target::Class(_)
                | crate::compile::decorators::Target::Reopen { .. }
                | crate::compile::decorators::Target::Module(_) => {
                    iris_runtime::ImmutableHash::new(
                        fields,
                        Value::Type(self.builtin_class("Symbol")?, Vec::new()),
                        Value::Type(self.builtin_class("Object")?, Vec::new()),
                    )
                }
            };
            self.decorator_metadata.push(metadata.clone());
            let mut passed = vec![
                receiver,
                Value::ImmutableHash(metadata),
                Value::ReadonlyArray(arguments.elements()),
            ];
            if !self.decorator_planning {
                let phase = self
                    .decorator_phases
                    .last()
                    .ok_or(MachineError::UnsupportedConstruct)?;
                passed.push(DecoratorValue::Context(phase.context()).into());
            }
            let value =
                self.invoke_function(owner.function, passed, &owner.program, &owner.classes)?;
            let Value::Decorator(record) = &value else {
                return Err(self.decorator_type_error());
            };
            let valid = match record.as_ref() {
                DecoratorValue::Plan(plan) if self.decorator_planning => {
                    plan.validate_kind(application.kind).is_ok()
                }
                DecoratorValue::Transformation(transformation) if !self.decorator_planning => {
                    transformation.validate_kind(application.kind).is_ok()
                }
                _ => return Err(self.decorator_type_error()),
            };
            if !valid {
                return Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"));
            }
            Ok(value)
        })();
        if let Some(mut phase) = self.decorator_phases.pop() {
            phase.finish();
        }
        result
    }
}
