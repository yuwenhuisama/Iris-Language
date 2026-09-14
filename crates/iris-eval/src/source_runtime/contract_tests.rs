use super::{EvaluationError, SourceEvaluator};
use iris_runtime::Value;
use iris_syntax::{Declaration, ExportDeclaration, ProgramEntry};

#[test]
fn exported_contract_is_planned_when_ast_wraps_decorated_declaration() {
    let mut given = SourceEvaluator::new_in_package("contract-tests").unwrap();
    let parsed = iris_parser::parse(
        "class Bad { fun helper() { print(1) } }; impl Bad for ContractDecorator { public fun plan(d,a) -> Plan { self.helper(); Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } }; @Bad() contract Named {}; 0",
    );
    assert!(parsed.program_accepted);
    let mut program = parsed.program;
    for entry in &mut program.entries {
        if let ProgramEntry::Declaration(Declaration::Contract(contract)) = entry {
            *entry = ProgramEntry::Declaration(Declaration::Export(Box::new(
                ExportDeclaration::Declaration(Box::new(Declaration::Contract(contract.clone()))),
            )));
        }
    }
    let when = given.program(&program);
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-NONDETERMINISTIC",
            phase: "static",
        })
    );
}

#[test]
fn owned_state_is_discarded_when_later_contract_transform_fails() {
    let mut given = SourceEvaluator::new_in_package("contract-tests").unwrap();
    let parsed = iris_parser::parse(
        "contract Existing { fun old() -> Integer }; class Stamp {}; impl Stamp for ContractDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } }; class Stop {}; impl Stop for ContractDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { raise :abort } }; 0",
    );
    given.program(&parsed.program).unwrap();
    let original = given.contract_names["Existing"];
    let metadata_count = given.decorator_metadata.len();
    let contract_count = given.contract_metadata.len();
    let identity_count = given.package_contexts.contracts.len();
    let requirements = given.contract_requirements.clone();
    let parsed =
        iris_parser::parse("@Stamp() @Stop() contract Existing { fun replacement() -> Symbol }; 0");
    let when = given.program(&parsed.program);
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.contract_names["Existing"], original);
    assert_eq!(given.decorator_metadata.len(), metadata_count);
    assert_eq!(given.contract_metadata.len(), contract_count);
    assert_eq!(given.package_contexts.contracts.len(), identity_count);
    assert_eq!(given.contract_requirements, requirements);
    assert!(given.decorator_phase_stack.is_empty());
}
