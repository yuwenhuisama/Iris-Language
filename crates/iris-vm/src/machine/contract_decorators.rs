use super::{Machine, MachineError};
use crate::compile::Program;
use crate::compile::decorators::Target;
use iris_runtime::{ClassId, DecoratorValue, ImmutableHash, Value};

impl Machine {
    pub(super) fn plan_declarations(
        &mut self,
        program: &Program,
        classes: &mut Vec<ClassId>,
    ) -> Result<(), MachineError> {
        for instruction in &program.instructions {
            match instruction {
                crate::Instruction::ApplyModuleDecorators { module } => {
                    self.publish_module(*module, program, classes)?
                }
                crate::Instruction::ApplyDecorators { class } if *class == classes.len() => {
                    self.publish_origin(program, classes)?
                }
                crate::Instruction::ApplyContractDecorators { contract } => {
                    self.publish_contract(*contract, program, classes)?
                }
                crate::Instruction::ApplyReopen { class, reopen } => {
                    for application in program.decorator_applications.iter().filter(|application| {
                        application.target
                            == (Target::Reopen {
                                class: *class,
                                artifact: *reopen,
                            })
                    }) {
                        self.execute_decorator_phase(application, program, classes)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn contract_metadata(
        &self,
        index: usize,
        program: &Program,
    ) -> Result<ImmutableHash, MachineError> {
        let declaration = &program.contracts[index];
        let requirements = declaration
            .requirements
            .iter()
            .map(|requirement| Value::Symbol(requirement.selector.clone()))
            .collect();
        let decorators = program
            .decorator_applications
            .iter()
            .filter(|application| application.target == Target::Contract(index))
            .map(|application| Value::Symbol(program.classes[application.decorator].name.clone()))
            .collect();
        let parents = declaration
            .parents
            .iter()
            .map(|name| {
                let parent = program
                    .contracts
                    .iter()
                    .position(|known| &known.name == name)
                    .ok_or(MachineError::NameError)?;
                Ok(Value::Contract(
                    program.contract_identity(parent),
                    Vec::new(),
                ))
            })
            .collect::<Result<Vec<_>, MachineError>>()?;
        let capabilities = super::runtime::meta_capabilities(&declaration.meta_deny)?;
        let fields = vec![
            ("name", Value::Symbol(declaration.name.clone())),
            ("kind", Value::Symbol("contract".into())),
            ("package", Value::Text(program.package_id().into())),
            (
                "type",
                Value::Contract(program.contract_identity(index), Vec::new()),
            ),
            ("parents", Value::ReadonlyArray(parents)),
            ("requirements", Value::ReadonlyArray(requirements)),
            ("decorators", Value::ReadonlyArray(decorators)),
            (
                "meta_capabilities",
                Value::ReadonlyArray(
                    capabilities
                        .denied()
                        .into_iter()
                        .map(|capability| Value::Symbol(super::capability_name(capability).into()))
                        .collect(),
                ),
            ),
        ];
        Ok(ImmutableHash::new(
            fields
                .into_iter()
                .map(|(name, value)| (Value::Symbol(name.into()), value))
                .collect(),
            Value::Type(self.builtin_class("Symbol")?, Vec::new()),
            Value::Type(self.builtin_class("Object")?, Vec::new()),
        ))
    }

    pub(super) fn publish_contract(
        &mut self,
        index: usize,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        self.register_core_records()?;
        let checkpoint = self.decorator_metadata.len();
        let result = (|| {
            for application in program
                .decorator_applications
                .iter()
                .filter(|application| application.target == Target::Contract(index))
            {
                let value = self.execute_decorator_phase(application, program, classes)?;
                if self.decorator_planning {
                    continue;
                }
                let Value::Decorator(record) = value else {
                    return Err(self.decorator_type_error());
                };
                let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                    return Err(self.decorator_type_error());
                };
                if !transformation.operations().is_empty() {
                    return Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"));
                }
            }
            self.contract_metadata(index, program)
        })();
        match result {
            Ok(metadata) => {
                self.decorator_metadata.push(metadata.clone());
                self.contracts
                    .insert(program.contract_identity(index), metadata);
                Ok(())
            }
            Err(error) => {
                self.decorator_metadata.truncate(checkpoint);
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_contract_stays_unpublished_when_operation_is_invalid() {
        let source = "class Add {} impl Add for ContractDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.add_method(:added, { || -> Integer; 1 }) } } @Add() contract Named { fun name() -> Symbol } class Implementor {} impl Implementor for Named { public fun name() -> Symbol { :name } } 0";
        let program = crate::compile(source).expect("fixture compiles");
        let mut machine = Machine::new().expect("machine");
        let result = machine.execute(&program);
        assert_eq!(
            result,
            Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"))
        );
        assert!(machine.contracts.is_empty());
        assert!(machine.decorator_metadata.is_empty());
    }

    #[test]
    fn retained_wrong_kind_transformation_is_rejected_before_contract_publication() {
        use iris_runtime::decorator_protocol::{
            DecoratorKind, DecoratorPhase, DecoratorReason, Transformation,
        };
        let source = "class Wrong {} impl Wrong for ContractDecorator { public fun plan(d, a) -> Plan { nil } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Wrong() contract Named {} 0";
        let mut program = crate::compile(source).expect("fixture compiles");
        let function = program
            .functions
            .iter_mut()
            .find(|function| function.name == "Wrong.transform")
            .expect("transform function");
        function.instructions = vec![
            crate::Instruction::LoadBinding {
                destination: 0,
                name: "retained".into(),
                shared: false,
            },
            crate::Instruction::Return { value: 0 },
        ];
        let phase = DecoratorPhase::new(DecoratorKind::Class, DecoratorReason::Origin);
        let transformation = Transformation::empty(Some(&phase)).expect("class transformation");
        let mut machine = Machine::new().expect("machine");
        machine.register_core_records().expect("core");
        let classes = machine.register_classes(&program).expect("classes");
        machine.bindings.insert(
            "retained".into(),
            DecoratorValue::Transformation(transformation).into(),
        );
        let Target::Contract(index) = program.decorator_applications[0].target else {
            panic!("contract target");
        };
        let result = machine.publish_contract(index, &program, &classes);
        assert_eq!(
            result,
            Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"))
        );
        assert!(machine.contracts.is_empty());
        assert!(machine.decorator_metadata.is_empty());
    }
}
