use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{ArrayRef, Value};

const METHODS: &str = "public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + 1 })
    }";
const TARGET: &str =
    "class Target { @Wrap() public fun value() -> Integer { 7 } }; Target.new().value()";

#[test]
fn decorator_applies_when_impl_precedes_origin() {
    let given = format!("impl Wrap for MethodDecorator {{ {METHODS} }} class Wrap {{}} {TARGET}");
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn decorator_applies_when_impl_follows_target() {
    let given = format!("class Wrap {{}} {TARGET}; impl Wrap for MethodDecorator {{ {METHODS} }}");
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn decorator_applies_when_definition_is_exported() {
    let given =
        format!("export class Wrap {{}} impl Wrap for MethodDecorator {{ {METHODS} }} {TARGET}");
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn decorator_applies_when_impl_was_defined_in_previous_evaluation() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&format!(
            "class Wrap {{}} impl Wrap for MethodDecorator {{ {METHODS} }}; 0"
        ))
        .unwrap();
    let when = given.evaluate(TARGET);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn decorator_applies_when_empty_impl_uses_existing_methods() {
    let given = format!("class Wrap {{ {METHODS} }} impl Wrap for MethodDecorator {{}} {TARGET}");
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn origin_consumes_one_revision_and_commit_when_static_impl_is_present() {
    let given = "class Before { public fun mark() { 0 } } contract Show { fun show() -> String } class Item {} impl Item for Show { public fun show() -> String { \"item\" } } %[Item.active_revision, Item.method(:show).source[2] - Before.method(:mark).source[2]]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1u64.into()),
            Value::Integer(1u64.into())
        ])))
    );
}

#[test]
fn contract_deny_contributes_when_impl_is_static() {
    let given = "class Item {} impl Item for Show { public fun show() -> String { \"item\" } } contract Show meta deny method_set { fun show() -> String } Item.denied_capabilities";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![Value::Symbol(
            "method_set".into()
        )])))
    );
}

#[test]
fn decorated_impl_method_is_wrapped_when_origin_is_published() {
    let given = format!(
        "class Wrap {{}} impl Wrap for MethodDecorator {{ {METHODS} }} contract Read {{ fun value() -> Integer }} class Target {{}} impl Target for Read {{ @Wrap() public fun value() -> Integer {{ 7 }} }} (Target.new() as Read)..value()"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn origin_is_unpublished_when_impl_requirement_is_missing() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate(
        "contract Read { fun value() -> Integer } class Target {} impl Target for Read {}; 0",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn generic_contract_view_works_when_impl_is_empty() {
    let given = "contract Read<T> { fun value(value: T) -> T } class Target<T> { public fun value(value: T) -> T { value } } impl Target<T> for Read<T> {} (Target<Integer>.new() as Read<Integer>)..value(7)";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn contract_deny_is_enforced_when_origin_transform_adds_a_method() {
    let given = "class Add {} impl Add for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.add_method(:extra, { 1 }) } } contract Fixed meta deny method_set {} @Add() class Target {} impl Target for Fixed {}; 0";
    let when = iris_eval::evaluate_with_class_publication(given, "Target");
    assert!(matches!(
        when.0,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied {
                operation: iris_runtime::Capability::MethodSet,
                ..
            }
        ))
    ));
    assert!(!when.1);
}

#[test]
fn multiple_contract_views_work_when_impls_share_an_existing_method() {
    let given = "impl Target for First {} class Target { public fun value() -> Integer { 7 } } impl Target for Second {} contract Second { fun value() -> Integer } contract First { fun value() -> Integer } (Target.new() as First)..value() + (Target.new() as Second)..value()";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(14u64.into())));
}
