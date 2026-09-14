use iris_eval::evaluate;
use iris_runtime::{ClassId, ContractId, Value};

#[test]
fn class_identity_matches_when_vm_registers_core_after_user_declarations() {
    for source in [
        "class User {} User",
        "class User {} ArgumentChanges.empty; User",
        "%[Symbol, Type, Tuple, TypeError, ArgumentError, Invocation, Array, Hash]",
    ] {
        let program = iris_vm::compile(source).unwrap();
        assert_eq!(evaluate(source).unwrap(), iris_vm::run(&program).unwrap());
    }
    assert_eq!(
        evaluate("class User {} User").unwrap(),
        Value::Class(ClassId::new(7))
    );
}

#[test]
fn contract_identity_matches_when_core_and_source_contracts_are_interleaved() {
    for source in [
        "contract User {} User",
        "%[ClassDecorator, MethodDecorator, PropertyDecorator]",
        "class Hook {} impl Hook for ClassDecorator { public fun plan(d, c) -> Plan { Plan.empty } public fun transform(d, p, c) -> Transformation { Transformation.empty } } contract User {} let types = %[ClassDecorator, User]; types",
        "contract ClassDecorator {} %[ClassDecorator, ModuleDecorator]",
    ] {
        let program = iris_vm::compile(source).unwrap();
        assert_eq!(
            evaluate(source).unwrap(),
            iris_vm::run(&program).unwrap(),
            "{source}"
        );
    }
    assert_eq!(
        evaluate("contract User {} User").unwrap(),
        Value::Contract(ContractId::new(3), Vec::new())
    );
}
