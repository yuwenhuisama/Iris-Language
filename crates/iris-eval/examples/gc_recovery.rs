use iris_eval::{EvaluationError, Session};
use iris_runtime::Value;

fn main() -> Result<(), EvaluationError> {
    let mut session = Session::new()?;
    let result = session.evaluate(r#"
        class Probe { public fun value() -> Integer { 41 } }
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                NativeFixture.compact_gc()
                let held = Probe.new()
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    held.value()
                })
            }
        }
        class Target {
            @Wrap() @Wrap() public class fun value<T>(value: T) -> Object { value }
        }
        Target.value(1)
    "#)?;
    assert_eq!(result, Value::Integer(41u64.into()));
    session.evaluate("NativeFixture.compact_gc()")?;
    assert_eq!(session.evaluate("Target.value(1)")?, result);
    assert!(session.evaluate("Target.value()").is_err());
    assert_eq!(session.evaluate("Target.value(2)")?, result);
    println!("PASS: construction roots, cached captures, invalid call, session recovery");
    Ok(())
}
