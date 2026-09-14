use iris_eval::{Session, evaluate};
use iris_runtime::Value;

#[test]
fn argument_error_is_nominal_when_ordinary_call_channels_are_invalid() {
    for invocation in [
        "target.run()",
        "target.run(1, 2)",
        "target.run(1, unknown: 2)",
        "target.run(value: 1, value: 2)",
        "target.run(1, &nil)",
        "target.with_block(&nil, &nil)",
        "callback.call()",
        "callback.call(1, unknown: 2)",
        "callback.call(value: 1, value: 2)",
        "callback.call(1, &nil)",
        "blocked.call(&nil, &nil)",
        "%[1].length(&nil, &nil)",
        "ArgumentChanges.new(unknown: nil)",
    ] {
        let given = format!(
            "class Target {{ public fun run(value) {{ value }}; public fun with_block(&block) {{ block }} }}; \
             let target = Target.new(); let callback = {{ |value| value }}; let blocked = {{ |&block| block }}; \
             try {{ {invocation} }} catch error, context {{ \
             error is? ArgumentError && error is? Kernel::ArgumentError && \
             error.class == ArgumentError && context.value same? error }}"
        );

        let when = evaluate(&given);

        assert_eq!(when, Ok(Value::Bool(true)), "{invocation}");
    }
}

#[test]
fn symbols_remain_symbols_when_named_like_core_errors() {
    let given = "let symbol = :ArgumentError; try { raise symbol } catch error, context { !(error is? ArgumentError) && error is? Symbol && error == symbol && context.value == symbol }";

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn type_error_is_nominal_when_an_ordinary_builtin_rejects_its_operand() {
    let given = "try { \"value\".split(1) } catch error, context { error is? TypeError && context.value same? error }";

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn original_context_survives_when_ordinary_error_is_rethrown() {
    let given = r#"
        class State { public class property context: Object = nil }
        let callback = { |value| value }
        try {
            try { callback.call() } catch error, context {
                State.context = context
                raise
            }
        } catch error, context {
            error is? ArgumentError && context.value same? error &&
            context same? State.context &&
            context.original_stack.length == State.context.original_stack.length &&
            context.re_raise_sites.length == 1
        }
    "#;

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn failure_gets_its_own_context_when_an_earlier_error_was_caught() {
    let given = "let old = try { raise :old } catch error, context { context }; let callback = { |value| value }; try { callback.call() } catch error, context { error is? ArgumentError && context.value same? error && !(context same? old) && context.cause == nil }";

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn failed_task_keeps_error_and_root_when_awaited_repeatedly() {
    for body in [
        "callback.call()",
        "try { callback.call() } catch error, context { State.context = context; raise }",
        "await gate; try { callback.call() } catch error, context { State.context = context; raise }",
    ] {
        let mut given = Session::new().unwrap();
        given
            .evaluate(&format!(
                r#"
            class State {{ public class property context: Object = nil }}
            let gate = Gate.new()
            let callback = {{ |value| value }}
            let run = {{ async || -> Object; {body} }}
            let task = run.call()
            Gate.complete(gate, nil)
            nil
        "#
            ))
            .unwrap();

        let when = given.evaluate(
            r#"
            let first = try { Host.run(task) } catch error, context { context }
            let second = try { Host.run(task) } catch error, context { context }
            let valid = first.value is? ArgumentError && first.value same? second.value &&
            first same? second && first.original_stack.length == second.original_stack.length &&
            (State.context == nil || first same? State.context)
            %[valid, first, second]
        "#,
        );

        let Value::Array(values) = when.unwrap() else {
            panic!("expected failure contexts: {body}")
        };
        let values = values.elements();
        assert_eq!(values[0], Value::Bool(true), "{body}");
        assert_eq!(values[1], values[2], "{body}");
    }
}

#[test]
fn class_name_and_text_keep_error_code_when_error_is_boxed() {
    let given = "let callback = { |value| value }; try { callback.call() } catch error { error.class_name == :ArgumentError && error.to_string() == \"ArgumentError\" }";

    let when = evaluate(given);

    assert_eq!(when, Ok(Value::Bool(true)));
}
