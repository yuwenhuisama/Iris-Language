use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

#[test]
fn construction_rejects_when_generic_arguments_are_missing_or_partial() {
    for construction in [
        "Box.new()",
        "Box<Integer>.new()",
        "Box<Integer, String, Nil>.new()",
        "let raw = Box; raw.new()",
        "let partial = Box<Integer>; partial.new()",
    ] {
        let given = format!("class Box<T, U> {{}}; {construction}");
        let when = evaluate(&given);
        assert_eq!(
            when,
            Err(EvaluationError::TypeContractError),
            "{construction}"
        );
    }
}

#[test]
fn definition_remains_usable_when_read_as_metadata() {
    let given = "class Box<T, U> {}; let definition = Box; definition same? Box";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn construction_accepts_when_all_arguments_are_explicit_including_nil() {
    let given = "class Box<T, U> {}; Box<Integer, Nil>.new() is? Box<Integer, Nil>";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn nominal_bound_accepts_when_argument_is_declared_child() {
    let given = "class Base {}; class Child extends Base {}; class Box<T> where T: Base {}; Box<Child>.new() is? Box<Child>";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn nominal_bound_rejects_when_argument_is_unrelated() {
    let given = "class Base {}; class Other {}; class Box<T> where T: Base {}; Box<Other>.new()";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn f_bound_accepts_when_contract_explicitly_names_the_argument() {
    let given = "contract Comparable<T> {}; class Key {}; impl Key for Comparable<Key> {}; class Box<T> where T: Comparable<T> {}; Box<Key>.new() is? Box<Key>";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn f_bound_rejects_when_contract_argument_differs() {
    let given = "contract Comparable<T> {}; class Key {}; impl Key for Comparable<Integer> {}; class Box<T> where T: Comparable<T> {}; Box<Key>.new()";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn nested_closed_bindings_survive_when_constructor_builds_another_generic() {
    let given = r#"
        class Inner<T> {}
        class Outer<T> {
            fun initialize() { @value = Inner<T>.new() }
            public fun value() { @value }
        }
        Outer<Integer>.new().value() is? Inner<Integer>
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn nested_f_bound_accepts_when_closed_owner_arguments_are_substituted() {
    let given = "contract Comparable<T> {}; class Key<T> {}; impl Key<T> for Comparable<Key<T>> {}; class Box<T> where T: Comparable<T> {}; Box<Key<Integer>>.new() is? Box<Key<Integer>>";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn nested_raw_definition_rejects_when_used_as_a_type_argument() {
    let given = "class Inner<T> {}; class Outer<T> {}; Outer<Inner>.new()";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn open_rejects_when_new_bound_excludes_existing_closed_class() {
    let given = "class Base {}; class Other {}; class Box<T> {}; let value = Box<Other>.new(); open class Box<T> where T: Base {}";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn object_bound_accepts_when_argument_is_builtin_or_nil() {
    for argument in ["String", "Integer", "Nil"] {
        let given =
            format!("class Box<T> where T: Object {{}}; Box<{argument}>.new() is? Box<{argument}>");
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{argument}");
    }
}

#[test]
fn composed_bounds_check_when_peer_parameter_is_substituted() {
    for (argument, accepted) in [("Child", true), ("Other", false), ("Nil", false)] {
        let given = format!(
            "class Base {{}}; class Child extends Base {{}}; class Other {{}}; class Box<T, U> where U: T & NonNil {{}}; Box<Base, {argument}>.new() is? Box<Base, {argument}>"
        );
        let when = evaluate(&given);
        let expected = if accepted {
            Ok(Value::Bool(true))
        } else {
            Err(EvaluationError::TypeContractError)
        };
        assert_eq!(when, expected, "{argument}");
    }
}
