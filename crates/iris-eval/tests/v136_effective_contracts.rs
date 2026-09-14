use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

#[test]
fn inherited_requirement_agrees_across_sends_views_and_guards() {
    let given = r#"
        contract Parent { fun value() -> Integer }
        contract Child extends Parent {}
        class A {}
        impl A for Child { public fun value() -> Integer { 7 } }
        fun guarded(value: Parent) -> Integer { (value as Parent)..value() }
        let value = A.new()
        value.value() + (value as Child)..value() + (value as Parent)..value() + guarded(value)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(28u64.into())));
}

#[test]
fn diamond_requirements_merge_compatible_signatures() {
    let given = r#"
        contract Root { fun value(input: Integer) -> Integer }
        contract Left extends Root {}
        contract Right extends Root { fun value(other: Integer) -> Integer }
        contract Child extends Left, Right {}
        class A {}
        impl A for Child { public fun value(input: Object) -> Integer { 7 } }
        (A.new() as Root)..value(1)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn incompatible_parent_signatures_reject_the_contract() {
    let given = r#"
        contract Left { fun value(input: Integer) -> Integer }
        contract Right { fun value(input: String) -> String }
        contract Child extends Left, Right {}
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn generic_parent_substitution_reaches_transitive_views_and_guards() {
    let given = r#"
        contract Root<T> { fun value(input: T) -> T }
        contract Middle<U> extends Root<U> {}
        contract Child extends Middle<Integer> {}
        class A {}
        impl A for Child { public fun value(input: Integer) -> Integer { input } }
        fun guarded(value: Root<Integer>) -> Integer { (value as Root<Integer>)..value(3) }
        let value = A.new()
        value.value(1) + (value as Child)..value(2) + guarded(value)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(6u64.into())));
}

#[test]
fn generic_parent_rejects_incompatible_implementation() {
    let given = r#"
        contract Parent<T> { fun value(input: T) -> T }
        contract Child extends Parent<Integer> {}
        class A { public fun value(input: String) -> Integer { 1 } }
        impl A for Child {}
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn subclass_inherits_generic_conformance_without_duplicate_impl() {
    let given = r#"
        contract Parent<T> { fun value() -> T }
        contract Child extends Parent<Integer> {}
        class Base {}
        impl Base for Child { public fun value() -> Integer { 7 } }
        class Derived extends Base { public override fun value() -> Integer { super() + 1 } }
        let value = Derived.new()
        value.value() + (value as Child)..value() + (value as Parent<Integer>)..value()
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(24u64.into())));
}

#[test]
fn generic_parent_guard_rejects_wrong_closed_arguments() {
    let given = r#"
        contract Parent<T> { fun value() -> T }
        contract Child extends Parent<Integer> {}
        class A {}
        impl A for Child { public fun value() -> Integer { 7 } }
        fun guarded(value: Parent<String>) -> Nil { nil }
        guarded(A.new())
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn inherited_requirement_rejects_incompatible_candidate_override() {
    let given = r#"
        contract Parent { fun value() -> Integer }
        class Base {}
        impl Base for Parent { public fun value() -> Integer { 7 } }
        class Derived extends Base { public override fun value() -> String { "wrong" } }
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn inherited_denial_reaches_implementing_class() {
    let given = r#"
        contract Parent meta deny method_set {}
        contract Child extends Parent {}
        class A {}
        impl A for Child {}
        open class A { public fun added() -> Nil { nil } }
        1
    "#;
    let when = evaluate(given);
    assert!(matches!(
        when,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
}

#[test]
fn full_inherited_signature_rejects_changed_async_shape() {
    let given = r#"
        contract Parent { fun value() -> Integer }
        contract Child extends Parent {}
        class A { public async fun value() -> Integer { 1 } }
        impl A for Child {}
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn generic_subclass_inherits_closed_parent_conformance() {
    let given = r#"
        contract Parent<T> { fun value(input: T) -> T }
        class Base {}
        impl Base for Parent<Integer> { public fun value(input: Integer) -> Integer { input } }
        class Derived<T> extends Base {}
        (Derived<String>.new() as Parent<Integer>)..value(7)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn same_generic_parent_cannot_be_inherited_with_conflicting_arguments() {
    let given = r#"
        contract Parent<T> {}
        contract Left extends Parent<Integer> {}
        contract Right extends Parent<String> {}
        contract Child extends Left, Right {}
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn builtin_parent_requirement_is_not_dropped() {
    let given = r#"
        contract Values extends Iterable<Integer> {}
        class A {}
        impl A for Values {}
        1
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn closed_generic_child_substitutes_parent_parameter_names() {
    let given = r#"
        contract Parent<T> { fun value(input: T) -> T }
        contract Child<U> extends Parent<U> {}
        class A {}
        impl A for Child<Integer> { public fun value(input: Integer) -> Integer { input } }
        let value = A.new()
        (value as Child<Integer>)..value(3) + (value as Parent<Integer>)..value(4)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn existing_child_view_can_be_cast_to_its_generic_parent() {
    let given = r#"
        contract Parent<T> { fun value() -> T }
        contract Child extends Parent<Integer> {}
        class A {}
        impl A for Child { public fun value() -> Integer { 7 } }
        let child = A.new() as Child
        (child as Parent<Integer>)..value()
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn inherited_generic_cast_rejects_wrong_arguments() {
    let given = r#"
        contract Parent<T> { fun value() -> T }
        contract Child extends Parent<Integer> {}
        class A {}
        impl A for Child { public fun value() -> Integer { 7 } }
        A.new() as Parent<String>
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(EvaluationError::Runtime(iris_runtime::KernelError::Type))
    );
}
