use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{ArrayRef, Value};

fn decorated(body: &str) -> String {
    format!(
        r#"
class Payload {{}}
class State {{
    public class property gate: Object = nil
    public class property first: Object = nil
    public class property second: Object = nil
    public class property result: Object = nil
    public class property context: Object = nil
}}
class Capture {{}}
impl Capture for MethodDecorator {{
    public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
    public fun transform(declaration, arguments, context) -> Transformation {{
        Transformation.wrap_method({{ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
            let bridge = next.call()
            if State.first same? nil {{ State.first = bridge }} else {{ State.second = bridge }}
            try {{ await bridge }}
            catch error, context {{ State.context = context; raise }}
        }})
    }}
}}
class Target {{
    @Capture() @Capture() public async fun value() -> Payload {{ await State.gate; {body} }}
}}
State.gate = Gate.new()
State.result = Payload.new()
let outer = Target.new().value()
nil
"#
    )
}

#[test]
fn task_typed_guard_is_invariant_when_async_method_returns_integer() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Target { public async fun value() -> Integer { 7 } }; let task = Target.new().value(); nil").unwrap();

    let when = ["Integer", "Object", "Symbol"].map(|result| {
        given.evaluate(&format!(
            "let checked: Task<{result}> = task; checked same? task"
        ))
    });

    assert_eq!(
        when,
        [
            Ok(Value::Bool(true)),
            Err(EvaluationError::TypeContractError),
            Err(EvaluationError::TypeContractError),
        ]
    );
}

#[test]
fn async_closure_signature_and_cast_are_exact_when_result_is_integer() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("let callback = { async || -> Integer; 7 }; nil")
        .unwrap();

    let when = given.evaluate(
        r#"let exact = callback as Closure<() -> Task<Integer>>
        %[callback.type == (Closure<() -> Task<Integer>>).type,
         callback is? Closure<() -> Task<Integer>>,
         callback is? Closure<() -> Task<Object>>,
         callback is? Closure<() -> Task<Symbol>>,
         callback is? Closure<() -> Integer>,
         (callback as? Closure<() -> Task<Object>>) same? nil,
         (callback as? Closure<() -> Task<Symbol>>) same? nil,
         exact same? callback, Host.run(exact.call())]"#,
    );

    assert_eq!(
        when,
        evaluate("%[true, true, false, false, false, true, true, true, 7]")
    );
}

#[test]
fn outer_and_bridge_types_stay_invariant_when_pending_tasks_complete() {
    let mut given = Session::new().unwrap();
    given.evaluate(&decorated("State.result")).unwrap();
    let probe = "%[outer is? Task<Payload>, outer is? Task<Object>, outer is? Task<Symbol>, State.first is? Task<Object>, State.first is? Task<Payload>, State.first is? Task<Symbol>, State.second is? Task<Object>, State.second is? Task<Payload>, State.second is? Task<Symbol>]";

    let before = given.evaluate(probe);
    let completed = given
        .evaluate("let posted = Gate.complete(State.gate, 1); Host.run(outer) same? State.result");
    let after = given.evaluate(probe);

    assert_eq!(completed, Ok(Value::Bool(true)));
    let expected = evaluate("%[true, false, false, true, false, false, true, false, false]");
    assert_eq!([before, after], [expected.clone(), expected]);
}

#[test]
fn decorated_task_typed_guards_accept_only_their_own_result_type() {
    let mut given = Session::new().unwrap();
    given.evaluate(&decorated("State.result")).unwrap();
    let probes = [
        "let checked: Task<Payload> = outer; checked same? outer",
        "let checked: Task<Object> = State.first; checked same? State.first",
        "let checked: Task<Object> = State.second; checked same? State.second",
        "let checked: Task<Object> = outer; checked same? outer",
        "let checked: Task<Payload> = State.first; checked same? State.first",
        "let checked: Task<Payload> = State.second; checked same? State.second",
    ];

    let before = probes.map(|probe| given.evaluate(probe));
    let completed = given
        .evaluate("let posted = Gate.complete(State.gate, 1); Host.run(outer) same? State.result");
    let after = probes.map(|probe| given.evaluate(probe));

    assert_eq!(completed, Ok(Value::Bool(true)));
    let expected = [
        Ok(Value::Bool(true)),
        Ok(Value::Bool(true)),
        Ok(Value::Bool(true)),
        Err(EvaluationError::TypeContractError),
        Err(EvaluationError::TypeContractError),
        Err(EvaluationError::TypeContractError),
    ];
    assert_eq!([before, after], [expected.clone(), expected]);
}

#[test]
fn repeated_await_preserves_context_when_decorated_task_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate(&decorated("raise :failure")).unwrap();
    given
        .evaluate("let posted = Gate.complete(State.gate, 1); nil")
        .unwrap();

    let when = given.evaluate(
        r#"let first = try { Host.run(outer) } catch error, context { context }
        let second = try { Host.run(outer) } catch error, context { context }
        let bridge = try { Host.run(State.first) } catch error, context { context }
        %[first same? second, first same? bridge, first same? State.context,
         first.value, first.cause, first.suppressed.length()]"#,
    );

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
            Value::Symbol("failure".into()),
            Value::Nil,
            Value::Integer(0u64.into()),
        ])))
    );
}
