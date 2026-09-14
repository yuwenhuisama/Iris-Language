use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

fn execute(source: &str) -> Result<Value, MachineError> {
    match compile(source) {
        Ok(program) => run(&program),
        Err(_) => Err(MachineError::UnsupportedConstruct),
    }
}

#[test]
fn named_contract_test_answers_true_for_static_conformer() {
    let given = "contract Named {} class Item {} impl Item for Named {} Item.new() is? Named";

    let when = execute(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn named_contract_test_answers_false_for_nonconformer() {
    let given = "contract Named {} class Item {} Item.new() is? Named";

    let when = execute(given);

    assert_eq!(when, Ok(Value::Bool(false)));
}

#[test]
fn named_contract_test_includes_inherited_conformance() {
    let given = "contract Named {} class Base {} impl Base for Named {} class Child extends Base {} Child.new() is? Named";

    let when = execute(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn closed_generic_contract_test_compares_arguments_invariantly() {
    let given = "contract Named<T> {} class Item {} impl Item for Named<Integer> {} let item = Item.new(); %[item is? Named<Integer>, item is? Named<String>]";

    let when = execute(given);

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(false),
        ])))
    );
}

#[test]
fn contract_test_evaluates_operand_once() {
    let given = "global mut $calls = 0; contract Named {} class Item {} impl Item for Named {} module Factory { public fun make() -> Item { $calls = $calls + 1; Item.new() } } %[Factory.make() is? Named, $calls]";

    let when = execute(given);

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn contract_test_preserves_unknown_type_error() {
    let given = "contract Named<T> {} class Item {} impl Item for Named<Integer> {} Item.new() is? Named<Missing>";

    let when = execute(given);

    assert_eq!(when, Err(MachineError::NameError));
}

#[test]
fn contract_test_with_wrong_generic_arity_answers_false() {
    let given = "contract Named<T> {} class Item {} impl Item for Named<Integer> {} Item.new() is? Named<Integer, String>";

    let when = execute(given);

    assert_eq!(when, Ok(Value::Bool(false)));
}
