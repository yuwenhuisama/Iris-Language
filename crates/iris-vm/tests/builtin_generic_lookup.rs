use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

fn evaluate(source: &str) -> Result<Value, MachineError> {
    run(&compile(source).expect("builtin generic lookup source compiles"))
}

#[test]
fn closed_collection_name_uses_the_canonical_builtin_class() {
    for (closed, plain) in [
        ("Array<Integer>", "Array"),
        ("Hash<Symbol, Integer>", "Hash"),
    ] {
        let given = format!(
            "let closed = {closed}; let canonical = {plain}; %[closed == canonical, closed.type == canonical.type, closed.type.arguments.length]"
        );
        assert_eq!(
            evaluate(&given),
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Bool(true),
                Value::Integer(0_u64.into()),
            ]))),
            "{closed}"
        );
    }
}

#[test]
fn missing_generic_name_remains_a_name_error() {
    assert_eq!(
        evaluate("MissingArray<Integer>"),
        Err(MachineError::NameError)
    );
}
