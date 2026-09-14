#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests inspect compiled VM results"
)]

use iris_runtime::{CoreClass, KernelError, Value};
use iris_vm::{MachineError, compile, run};

fn evaluate(source: &str) -> Result<Value, MachineError> {
    let given = compile(source).expect("source compiles");
    run(&given)
}

#[test]
fn canonical_object_when_ordinary_instruction_fails() {
    for (operation, class) in [
        ("Target.new(1)", CoreClass::ArgumentError),
        ("target.run()", CoreClass::ArgumentError),
        ("target.run.call()", CoreClass::ArgumentError),
        (
            "Reflection::Class.method(Target, :run).bind(target).call()",
            CoreClass::ArgumentError,
        ),
        (
            "Reflection::Class.invoke(Reflection::Class.method(Target, :run), target, %[])",
            CoreClass::ArgumentError,
        ),
        ("Target.open(nil)", CoreClass::TypeError),
        ("1 as Named", CoreClass::TypeError),
        ("raise :x from :plain", CoreClass::TypeError),
        ("ArgumentChanges.new(rest: nil)", CoreClass::TypeError),
    ] {
        let given = format!(
            "contract Named {{ fun name() -> String }}; class Target {{ public fun initialize() {{ nil }}; public fun run(value) {{ value }} }}; let target = Target.new(); try {{ {operation} }} catch error, context {{ %[error, context, error.class] }}"
        );
        let when = evaluate(&given).expect(operation);
        let Value::Array(values) = when else {
            panic!("expected catch result: {operation}")
        };
        let values = values.elements();
        assert!(
            matches!(values[0], Value::Object(_)),
            "{operation}: {values:?}"
        );
        assert_eq!(values[2], Value::Class(class.id()), "{operation}");
        let Value::ExceptionContext(_, value, cause, suppressed, sites, _) = &values[1] else {
            panic!("expected context: {operation}")
        };
        assert_eq!(value.as_ref(), &values[0], "{operation}");
        assert_eq!(cause.as_ref(), &Value::Nil, "{operation}");
        assert!(suppressed.is_empty() && sites.is_empty(), "{operation}");
    }
}

#[test]
fn symbols_keep_identity_when_named_like_core_errors() {
    for name in ["ArgumentError", "TypeError"] {
        let given = format!(
            "let symbol = :{name}; try {{ try {{ raise symbol }} catch error: {name} {{ false }} }} catch error: Symbol, context {{ error == symbol && context.value == symbol && !(error is? {name}) }}"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{name}");
    }
}

#[test]
fn typed_catch_when_cause_is_invalid() {
    let given = "try { raise :x from :plain } catch error: TypeError, context { error is? Kernel::TypeError && context.value same? error && !(error is? Symbol) }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn original_cause_when_explicit_context_is_legal() {
    let given = "let original = try { raise :old } catch error, context { context }; try { raise :new from original } catch error, context { error == :new && context.value == :new && context.cause same? original }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn absent_cause_when_explicit_nil_suppresses_chaining() {
    let given = "try { try { raise :old } catch error { raise :new from nil } } catch error, context { error == :new && context.value == :new && context.cause == nil }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn primary_context_survives_when_rethrown_across_a_call() {
    let given = "class State { public class property context: Object = nil }; class Target { public fun run(value) { value }; public fun fail() { try { self.run() } catch error, context { State.context = context; raise } } }; try { Target.new().fail() } catch error, context { context same? State.context && context.value same? error && context.re_raise_sites.length == 1 && error is? ArgumentError }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn fresh_context_when_an_earlier_failure_was_caught() {
    let given = "class Target { public fun run(value) { value } }; let target = Target.new(); let old = try { target.run() } catch error, context { context }; try { target.run() } catch error, context { !(context same? old) && !(error same? old.value) && context.value same? error && context.cause == nil }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn later_raise_survives_when_a_converted_error_was_handled() {
    let given = "class Target { public fun run(value) { value } }; try { Target.new().run() } catch error { nil }; try { raise :later } finally { nil }";
    let when = evaluate(given);
    assert!(
        matches!(when, Err(MachineError::Raised(payload)) if payload.0 == Value::Symbol("later".into()))
    );
}

#[test]
fn original_machine_error_when_nothing_handles_failure() {
    for (given, expected) in [
        (
            "class Target { public fun initialize() { nil } }; Target.new(1)",
            MachineError::ArgumentError,
        ),
        (
            "raise :x from :plain",
            MachineError::Kernel(KernelError::Type),
        ),
    ] {
        let when = evaluate(given);
        assert_eq!(when, Err(expected));
    }
}

#[test]
fn primary_context_survives_when_cleanup_propagates_across_a_call() {
    let given = "class State { public class property context: Object = nil }; class Target { public fun run(value) { value }; public fun fail() { try { self.run() } catch error: TypeError { false } finally { nil } } }; try { Target.new().fail() } catch error, context { error.class == ArgumentError && context.value same? error && context.cause == nil }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn outer_failure_survives_when_cleanup_handles_another_failure() {
    let given = "class Target { public fun run(value) { value } }; let target = Target.new(); try { target.run() } finally { try { raise :x from :plain } catch error { nil } }";
    let when = evaluate(given);
    assert_eq!(when, Err(MachineError::ArgumentError));
}

#[test]
fn core_error_observation_when_object_is_boxed() {
    let given = "class Target { public fun run(value) { value } }; try { Target.new().run() } catch error { error.class == Kernel::ArgumentError && error.class_name == :ArgumentError && error.to_string() == \"ArgumentError\" }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}
