use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

#[test]
fn constructs_when_explicit_closed_contract_matches_the_f_bound() {
    let given = r#"
        contract Comparable<T> { fun compare(other: T) -> Integer }
        class Key {}
        impl Key for Comparable<Key> { public fun compare(other: Key) -> Integer { 7 } }
        class Box<T> where T: Comparable<T> {
            public fun initialize(value: T) -> Nil { nil }
        }
        Box<Key>.new(Key.new()) is? Box<Key>
    "#;

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn rejects_when_explicit_contract_has_the_wrong_self_argument() {
    let given = r#"
        contract Comparable<T> { fun compare(other: T) -> Integer }
        class Key {}
        impl Key for Comparable<Object> { public fun compare(other: Object) -> Integer { 7 } }
        class Box<T> where T: Comparable<T> {}
        Box<Key>.new()
    "#;

    let when = evaluate(given);

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn constructs_when_ordinary_constructor_receives_its_owner_type_argument() {
    let given = r#"
        class Box<T> {
            public fun initialize(value: T) -> Nil { nil }
        }
        Box<String>.new("iris") is? Box<String>
    "#;

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn rejects_when_ordinary_constructor_argument_violates_the_closed_owner_type() {
    let given = r#"
        class Box<T> {
            public fun initialize(value: T) -> Nil { nil }
        }
        Box<String>.new(7)
    "#;

    let when = evaluate(given);

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn rejects_when_generic_contract_arguments_are_omitted() {
    let given = r#"
        contract Comparable<T> {}
        class Key {}
        impl Key for Comparable {}
        class Box<T> where T: Comparable<T> {}
        Box<Key>.new()
    "#;

    let when = evaluate(given);

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn rejects_when_matching_members_have_no_nominal_conformance() {
    let given = r#"
        contract Comparable<T> { fun compare(other: T) -> Integer }
        class Key {
            public fun compare(other: Key) -> Integer { 7 }
        }
        class Box<T> where T: Comparable<T> {}
        Box<Key>.new()
    "#;

    let when = evaluate(given);

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}
