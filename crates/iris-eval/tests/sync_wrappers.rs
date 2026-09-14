use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

fn decorator(name: &str, body: &str) -> String {
    format!(
        "class {name} {{}} impl {name} for MethodDecorator {{ public fun plan(d, a) -> Plan {{ Plan.empty }}; public fun transform(d, a, c) -> Transformation {{ Transformation.wrap_method({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; {body} }}) }} }}"
    )
}

#[test]
fn first_wrapper_is_outermost() {
    let source = format!(
        "{}; {}; class Target {{ @A() @B() public fun value() -> Integer {{ 1 }} }}; Target.new().value()",
        decorator("A", "next.call() * 2"),
        decorator("B", "next.call() + 3")
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(8u64.into())));
}

#[test]
fn next_patches_original_named_parameter() {
    let source = format!(
        "{}; class Target {{ @Patch() public fun value(value: Integer) -> Integer {{ value }} }}; Target.new().value(1)",
        decorator(
            "Patch",
            "next.call(ArgumentChanges.new(positional: %{:value: 9}))"
        )
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(9u64.into())));
}

#[test]
fn zero_attempt_result_raises_actual_type_error() {
    let source = format!(
        "{}; class Target {{ @Cache() public fun value() -> Integer {{ raise :body }} }}; try {{ Target.new().value() }} catch error {{ error is? TypeError }}",
        decorator("Cache", "\"wrong\"")
    );
    assert_eq!(evaluate(&source), Ok(Value::Bool(true)));
}

#[test]
fn invalid_initial_input_never_enters_wrapper() {
    let source = format!(
        "{}; class Target {{ @Cache() public fun value(value: Integer) -> Integer {{ value }} }}; let input: Object = \"wrong\"; try {{ Target.new().value(input) }} catch error {{ error is? TypeError }}",
        decorator("Cache", "raise :wrapper")
    );
    assert_eq!(evaluate(&source), Ok(Value::Bool(true)));
}

#[test]
fn wrong_wrapper_signature_is_rejected() {
    let source = "class Wrong {} impl Wrong for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.wrap_method({ |invocation: Object, next: Closure<(ArgumentChanges) -> Object>| -> Object; 7 }) } }; class Target { @Wrong() public fun value() -> Integer { 1 } }; 0";
    assert!(matches!(
        evaluate(source),
        Err(EvaluationError::Raised(Value::Object(_)))
    ));
}

#[test]
fn retained_next_expires_when_its_wrapper_returns() {
    let source = format!(
        "mut saved: Object = nil; {}; class Target {{ @Save() public fun value() -> Integer {{ 7 }} }}; let result = Target.new().value(); try {{ saved.call() }} catch error {{ %[error is? DecoratorProtocolError, error.category, error.owner == saved] }}",
        decorator("Save", "saved = next; next.call()")
    );
    assert_eq!(
        evaluate(&source),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Symbol("expired".into()),
            Value::Bool(true),
        ])))
    );
}

#[test]
fn reentrant_next_is_rejected_before_another_body_entry() {
    let source = format!(
        "mut saved: Object = nil; {}; class Target {{ @Save() public fun value() -> Integer {{ try {{ saved.call() }} catch error {{ if error.category == :overlap {{ return 7 }} }}; 0 }} }}; Target.new().value()",
        decorator("Save", "saved = next; next.call()")
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(7u64.into())));
}

#[test]
fn outer_wrapper_can_recover_only_after_inner_result_guard() {
    let source = format!(
        "{}; {}; class Target {{ @Recover() @Bad() public fun value() -> Integer {{ 1 }} }}; Target.new().value()",
        decorator(
            "Recover",
            "try { next.call(); 0 } catch error { if error is? TypeError { return 9 }; 0 }"
        ),
        decorator("Bad", "\"wrong\"")
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(9u64.into())));
}

#[test]
fn wrapper_and_original_keep_their_own_lexical_receivers() {
    let source = "class Add {}
    impl Add for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            @offset = 40
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + @offset })
        }
    }
    class Target {
        public fun initialize() { @offset = 2 }
        @Add() public fun value() -> Integer { @offset }
    }
    Target.new().value()";
    assert_eq!(evaluate(source), Ok(Value::Integer(42u64.into())));
}

#[test]
fn each_retry_starts_from_incoming_snapshot_not_previous_patch() {
    let source = format!(
        "{}; class Target {{ @Retry() public fun value(value: Integer = 7) -> Integer {{ value }} }}; Target.new().value()",
        decorator(
            "Retry",
            "next.call(ArgumentChanges.new(positional: %{:value: 99})); next.call()"
        )
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(7u64.into())));
}
