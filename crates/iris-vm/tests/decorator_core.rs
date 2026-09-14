#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert adapter results"
)]

use iris_runtime::{ArrayRef, DecoratorValue, Value};
use iris_vm::{MachineError, compile, run};

fn evaluate(source: &str) -> Result<Value, MachineError> {
    let program = compile(source).expect("core source compiles without fallback");
    run(&program)
}

#[test]
fn changes_are_available_without_a_phase() {
    let source =
        "%[ArgumentChanges.empty is? ArgumentChanges, ArgumentChanges.new() is? ArgumentChanges]";
    let outcome = evaluate(source);
    assert_eq!(
        outcome,
        Ok(Value::Array(ArrayRef::new(vec![Value::Bool(true); 2])))
    );
}

#[test]
fn changes_constructor_preserves_presence_and_snapshots() {
    let source = "mut values = %{:value: 7}; let changes = ArgumentChanges.new(positional: values, block: nil); let assigned = values[:value] = 9; changes";
    let outcome = evaluate(source).expect("constructor succeeds");
    let Value::Decorator(record) = outcome else {
        panic!("typed record")
    };
    let DecoratorValue::ArgumentChanges(changes) = *record else {
        panic!("changes")
    };
    assert_eq!(
        changes.positional(),
        Some(&[("value".into(), Value::Integer(7_u64.into()))][..])
    );
    assert_eq!(changes.block(), Some(&Value::Nil));
    assert_eq!(changes.rest(), None);
}

#[test]
fn constructor_rejects_invalid_channels_and_shapes() {
    for (arguments, expected) in [
        ("1", "ArgumentError"),
        ("unknown: 1", "ArgumentError"),
        ("block: nil, block: nil", "ArgumentError"),
        ("positional: nil", "TypeError"),
        ("positional: %{\"value\": 1}", "TypeError"),
        ("rest: nil", "TypeError"),
    ] {
        let source = format!(
            "try {{ ArgumentChanges.new({arguments}) }} catch error {{ error is? {expected} }}"
        );
        let outcome = evaluate(&source);
        assert_eq!(outcome, Ok(Value::Bool(true)), "{arguments}");
    }
}

#[test]
fn factories_raise_typed_outside_phase_errors() {
    for expression in [
        "Plan.empty",
        "Transformation.empty",
        "Transformation.wrap_method(nil)",
        "Transformation.wrap_getter(nil)",
        "Transformation.wrap_setter(nil)",
        "Transformation.add_method(:value, nil)",
    ] {
        let source = format!(
            "try {{ {expression} }} catch error {{ %[error is? DecoratorProtocolError, error.category, error.owner] }}"
        );
        let outcome = evaluate(&source);
        assert_eq!(
            outcome,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Symbol("outside_phase".into()),
                Value::Nil
            ]))),
            "{expression}"
        );
    }
}

#[test]
fn core_records_cannot_be_constructed_or_mutated() {
    for name in [
        "Invocation",
        "InvocationSignature",
        "InvocationParameter",
        "DecoratorContext",
        "DecoratorProtocolError",
        "Plan",
        "Transformation",
    ] {
        let outcome = evaluate(&format!("{name}.new()"));
        assert!(
            matches!(outcome, Err(MachineError::MessageNotFound { .. })),
            "{name}: {outcome:?}"
        );
    }
    let outcome = evaluate(
        "let changes = ArgumentChanges.empty; try { changes.block = nil } catch error { error }",
    );
    assert_eq!(outcome, Ok(Value::Symbol("ReadonlyMutationError".into())));
}

#[test]
fn core_type_annotations_and_casts_are_exact() {
    let outcome = evaluate(
        "let changes: ArgumentChanges = ArgumentChanges.new(); %[(changes as ArgumentChanges) is? ArgumentChanges, changes is? Invocation]",
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(false)
        ])))
    );
    assert!(evaluate("let wrong: Invocation = ArgumentChanges.empty; wrong").is_err());
}

#[test]
fn all_constructor_fields_accept_typed_source_containers() {
    let source = "let positional: Hash<Symbol, Object> = %{:value: 1}; let rest: Array<Object> = %[2]; ArgumentChanges.new(positional: positional, keywords: %{:label: :ok}, rest: rest, keyword_rest: %{:extra: true}, block: nil) is? ArgumentChanges";
    assert_eq!(evaluate(source), Ok(Value::Bool(true)));
}

#[test]
fn kernel_qualified_names_share_core_identity() {
    assert_eq!(
        evaluate("Kernel::ArgumentChanges.new() is? ArgumentChanges"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn constructor_rejects_a_trailing_block() {
    assert_eq!(
        evaluate(
            "try { ArgumentChanges.new() { || -> Object; nil } } catch error { error is? ArgumentError }"
        ),
        Ok(Value::Bool(true))
    );
}

#[test]
fn protocol_error_survives_a_typed_catch() {
    assert_eq!(
        evaluate("try { Plan.empty } catch error: DecoratorProtocolError { error.category }"),
        Ok(Value::Symbol("outside_phase".into()))
    );
}

#[test]
fn ordinary_symbol_catch_does_not_require_core_registration() {
    assert_eq!(
        evaluate("try { raise :original } catch error: Symbol { error }"),
        Ok(Value::Symbol("original".into()))
    );
}

#[test]
fn decorated_methods_execute_wrapped_bytecode() {
    let source = "class Wrap {} impl Wrap for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; (next.call() as Integer) + 10 }) } } class Target { @Wrap() public fun value() -> Integer { 7 } } Target.new().value()";
    assert_eq!(evaluate(source), Ok(Value::Integer(17_u64.into())));
}

#[test]
fn real_decorator_contract_requires_both_members() {
    for name in [
        "ClassDecorator",
        "ModuleDecorator",
        "ContractDecorator",
        "MethodDecorator",
        "PropertyDecorator",
    ] {
        let complete = format!(
            "class Complete {{}} impl Complete for {name} {{ public fun plan(declaration, arguments) -> Plan {{ Plan.empty }} public fun transform(declaration, arguments, context) -> Transformation {{ Transformation.empty }} }} 7"
        );
        assert_eq!(
            evaluate(&complete),
            Ok(Value::Integer(7_u64.into())),
            "{name}"
        );
        let missing = format!(
            "class Missing {{}} impl Missing for {name} {{ public fun plan(declaration, arguments) -> Plan {{ Plan.empty }} }} 7"
        );
        assert!(
            compile(&missing).is_err() || evaluate(&missing).is_err(),
            "{name}"
        );
    }
}
