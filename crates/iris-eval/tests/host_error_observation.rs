use iris_eval::backend::{Backend, Interpreter, Observation};
use iris_eval::{EvaluationError, Session, evaluate_host as evaluate};
use iris_runtime::Value;

#[test]
fn uncaught_call_error_has_nominal_code_when_runtime_is_dropped() {
    let given =
        "let callback = { |value| value }; try { callback.call() } catch error: Symbol { error }";

    let when = Interpreter.execute(given);

    assert_eq!(
        when,
        iris_eval::backend::Support::Ran(Observation::Error("ArgumentError".into()))
    );
}

#[test]
fn uncaught_await_error_has_nominal_code_when_runtime_is_dropped() {
    let given = "module M { public async fun run() -> Object { await 7 } } Host.run(M.run())";

    let when = Interpreter.execute(given);

    assert_eq!(
        when,
        iris_eval::backend::Support::Ran(Observation::Error("Runtime(Type)".into()))
    );
}

#[test]
fn symbol_remains_a_raised_value_when_named_argument_error() {
    let given = "raise :ArgumentError";

    let when = evaluate(given);

    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol(
            "ArgumentError".into()
        )))
    );
}

#[test]
fn session_remains_usable_when_nominal_failure_escapes() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("let callback = { |value| value }; nil")
        .unwrap();

    let when = Observation::of(given.evaluate("callback.call()"));

    assert_eq!(when, Observation::Error("ArgumentError".into()));
    assert_eq!(
        given.evaluate("callback.call(7)"),
        Ok(Value::Integer(7_u8.into()))
    );
}

#[test]
fn error_context_is_stable_when_failed_task_is_observed_repeatedly() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("let run = { async || -> Object; await 7 }; let task = run.call(); nil")
        .unwrap();

    let when = [
        given.evaluate_host("Host.run(task)"),
        given.evaluate_host("Host.run(task)"),
    ];

    let [
        Err(EvaluationError::HostException(first)),
        Err(EvaluationError::HostException(second)),
    ] = when
    else {
        panic!("expected owned host exception metadata")
    };
    assert_eq!(first, second);
    assert_eq!(first.kind.code(), "TypeError");
    let Some(Value::ExceptionContext(_, value, ..)) = first.context else {
        panic!("expected retained task context")
    };
    assert_eq!(*value, first.value);
    assert_eq!(
        given.evaluate("try { Host.run(task) } catch error { error is? TypeError }"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn core_error_name_cannot_be_redeclared_as_a_user_class() {
    let given = "class ArgumentError { } raise ArgumentError.new()";

    let when = evaluate(given);

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn uncaught_type_error_is_self_contained_when_ordinary_builtin_fails() {
    let given = "try { \"value\".split(1) } catch error: Symbol { error }";

    let when = evaluate(given);

    let Err(EvaluationError::HostException(exception)) = when else {
        panic!("expected owned host exception metadata")
    };
    assert_eq!(exception.kind.code(), "TypeError");
    let Some(Value::ExceptionContext(_, value, ..)) = exception.context else {
        panic!("expected retained context")
    };
    assert_eq!(*value, exception.value);
}
