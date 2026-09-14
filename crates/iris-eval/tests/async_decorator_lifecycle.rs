use iris_eval::{Session, evaluate};
use iris_runtime::{ArrayRef, Value};

fn scenario(wrapper: &str, body: &str) -> String {
    format!(
        r#"
class State {{
    public class property gate: Object = nil
    public class property bridge: Object = nil
    public class property next: Object = nil
    public class property context: Object = nil
    public class property finished: Integer = 0
}}
class Wrap {{}}
impl Wrap for MethodDecorator {{
    public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
    public fun transform(declaration, arguments, context) -> Transformation {{
        Transformation.wrap_method({{ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object; {wrapper} }})
    }}
}}
class Target {{ @Wrap() public async fun value() -> Integer {{ {body} }} }}
State.gate = Gate.new()
let outer = Target.new().value()
nil
"#
    )
}

#[test]
fn abandoned_failure_remains_reportable_when_only_failed_owner_is_observed() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&scenario(
            "State.next = next; State.bridge = next.call(); raise :owner_failed",
            "await State.gate; State.finished = State.finished + 1; raise :inner_failed",
        ))
        .unwrap();
    given.evaluate("let primary = try { Host.run(outer) } catch error, context { State.context = context; error }; nil").unwrap();
    let diagnostic = given.decorator_lifetime_diagnostics().to_vec();
    assert_eq!(diagnostic.len(), 1);
    given
        .evaluate("let posted = Gate.complete(State.gate, 1); nil")
        .unwrap();
    given
        .evaluate("let ready = { async || -> Object; :ready }; Host.run(ready.call())")
        .unwrap();

    let when = given.evaluate("let reports = Diagnostics.unobserved_failures(); let report = reports[0]; %[primary, State.finished, reports.length(), report[1] same? State.bridge, report[2]]");

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("owner_failed".into()),
            Value::Integer(1u64.into()),
            Value::Integer(1u64.into()),
            Value::Bool(true),
            Value::Symbol("inner_failed".into())
        ])))
    );
    assert_eq!(given.decorator_lifetime_diagnostics(), diagnostic);
    assert_eq!(
        given.evaluate("Diagnostics.unobserved_failures().length()"),
        Ok(Value::Integer(1u64.into()))
    );
}

#[test]
fn pending_failure_keeps_root_when_finally_handles_another_failure() {
    let mut given = Session::new().unwrap();
    given.evaluate(&scenario(
        "try { await next.call() } catch error, context { State.context = context; raise } finally { try { raise :cleanup_noise } catch noise { nil } }",
        "await State.gate; raise :inner_failed",
    )).unwrap();
    given
        .evaluate("let posted = Gate.complete(State.gate, 1); nil")
        .unwrap();

    let when = given.evaluate("let first = try { Host.run(outer) } catch error, context { context }; let second = try { Host.run(outer) } catch error, context { context }; %[first same? State.context, first same? second, first.value]");

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Symbol("inner_failed".into())
        ])))
    );
}

#[test]
fn eager_guard_creates_own_root_when_callback_previously_caught_failure() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&scenario(
            "try { raise :caught } catch error, context { State.context = context }; :wrong",
            "7",
        ))
        .unwrap();

    let when = given.evaluate("let first = try { Host.run(outer) } catch error, context { context }; let second = try { Host.run(outer) } catch error, context { context }; %[first same? second, first same? State.context, first.value is? TypeError]");

    assert_eq!(when, evaluate("%[true, false, true]"));
}

#[test]
fn lifetime_diagnostic_retains_all_fields_when_owner_raises_with_pending_bridge() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&scenario(
            "State.next = next; State.bridge = next.call(); raise :owner_failed",
            "await State.gate; 7",
        ))
        .unwrap();
    let next = given.evaluate("State.next").unwrap();
    let bridge = given.evaluate("State.bridge").unwrap();
    let outer = given.evaluate("outer").unwrap();
    let primary = given
        .evaluate("try { Host.run(outer) } catch error, context { context }")
        .unwrap();

    let when = given.decorator_lifetime_diagnostics();

    assert_eq!(when.len(), 1);
    let Value::Tuple(fields) = &when[0] else {
        panic!("expected diagnostic tuple")
    };
    assert_eq!(fields.len(), 8);
    assert_eq!(
        &fields[..4],
        &[
            Value::Symbol("IRIS-DECORATOR-PROTOCOL".into()),
            Value::Symbol("error".into()),
            Value::Symbol("runtime".into()),
            Value::Symbol("unfinished_inner".into()),
        ]
    );
    assert!(matches!(fields[4], Value::Task(_)));
    assert_ne!(fields[4], outer);
    assert_ne!(fields[4], bridge);
    assert_eq!(fields[5], next);
    assert_eq!(fields[6], bridge);
    assert_eq!(fields[7], primary);
    let Value::ExceptionContext(_, value, cause, suppressed, sites, _) = primary else {
        panic!("expected primary context")
    };
    assert_eq!(*value, Value::Symbol("owner_failed".into()));
    assert_eq!(*cause, Value::Nil);
    assert!(suppressed.is_empty());
    assert!(sites.is_empty());
}

#[test]
fn ready_failure_keeps_root_when_forwarded_through_two_layers() {
    let mut given = Session::new().unwrap();
    let source = scenario(
        "State.bridge = next.call(); try { await State.bridge } catch error, context { State.context = context; raise }",
        "raise :inner_failed",
    ).replace("@Wrap() public", "@Wrap() @Wrap() public");
    given.evaluate(&source).unwrap();

    let when = given.evaluate("let first = try { Host.run(outer) } catch error, context { context }; let second = try { Host.run(outer) } catch error, context { context }; %[first same? State.context, first same? second, Diagnostics.unobserved_failures().length()]");

    assert_eq!(when, evaluate("%[true, true, 0]"));
    assert!(given.decorator_lifetime_diagnostics().is_empty());
}
