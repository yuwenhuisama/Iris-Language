use iris_eval::{EvaluationError, Session};
use iris_runtime::Value;

type TestResult = Result<(), EvaluationError>;

#[test]
fn public_member_and_old_state_survive_when_open_adds_a_member() -> TestResult {
    let mut given = Session::new()?;
    given.evaluate("module M { public fun old(){1}; public class property x: Integer = 7 } M")?;

    given.evaluate("open module M { public fun added(){2} } M")?;

    assert_eq!(
        given.evaluate("M.old() + M.added() + M.x"),
        Ok(Value::Integer(10_u8.into()))
    );
    Ok(())
}

#[test]
fn shared_state_is_preserved_when_open_initializer_mutates_then_raises() -> TestResult {
    let mut given = Session::new()?;
    given.evaluate("module M { shared mut @@count = 7; public module fun read(){@@count}; public class fun fail(){@@count = 9; raise :stop} } M")?;

    let when = given.evaluate("open module M { public class property x: Integer = M.fail() }");

    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert_eq!(given.evaluate("M.read()"), Ok(Value::Integer(7_u8.into())));
    Ok(())
}

#[test]
fn shared_initializer_reads_candidate_when_previous_cell_is_initialized() -> TestResult {
    let mut given = Session::new()?;

    let when = given.evaluate("module M { shared let @@count = 7; public class fun read(){@@count}; public class property x: Integer = M.read() } M.x");

    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
    Ok(())
}

#[test]
fn class_method_wrapper_executes_when_origin_initializer_calls_it() -> TestResult {
    let mut given = Session::new()?;
    given.evaluate("class Wrap {}; impl Wrap for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.wrap_method({ |inv: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + 1 }) } } 0")?;

    let when = given.evaluate("module M { @Wrap() public class fun value(){7}; public class property x: Integer = M.value() } M.x");

    assert_eq!(when, Ok(Value::Integer(8_u8.into())));
    Ok(())
}
