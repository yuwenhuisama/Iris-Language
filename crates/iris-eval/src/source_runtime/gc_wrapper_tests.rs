use super::gc_tests::{fixture, object, run};
use iris_runtime::Value;

const WRAPPED: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        let held = Probe.new()
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            held
        })
    }
}
class Target { @Wrap() public fun value() -> Object { nil } }
let target = Target.new()
nil
"#;

#[test]
fn closed_wrapper_capture_survives_when_later_application_collects() {
    let mut given = fixture();
    run(&mut given, r#"
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
        nil
    "#).unwrap();

    let when = run(&mut given, "Target.value(1)");

    assert_eq!(when, Ok(Value::Integer(41u64.into())));
}

#[test]
fn published_chain_capture_survives_when_no_program_closure_binding_exists() {
    let mut given = fixture();
    run(&mut given, WRAPPED).unwrap();
    let held = object(run(&mut given, "target.value()").unwrap());
    let hash = given.runtime.identity_hash(held).unwrap();

    given.collect_garbage().unwrap();

    assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    assert_eq!(
        run(&mut given, "target.value().value()"),
        Ok(Value::Integer(41u64.into()))
    );
}

#[test]
fn retired_chain_capture_is_reclaimed_when_replacement_is_published() {
    let mut given = fixture();
    run(&mut given, WRAPPED).unwrap();
    let held = object(run(&mut given, "target.value()").unwrap());
    run(
        &mut given,
        "open class Target { public override fun value() -> Object { nil } }; nil",
    )
    .unwrap();

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn retired_chain_capture_survives_when_bound_method_retains_it() {
    let mut given = fixture();
    run(&mut given, WRAPPED).unwrap();
    run(&mut given, "let retained = target.value; nil").unwrap();
    let held = object(run(&mut given, "retained.call()").unwrap());
    let hash = given.runtime.identity_hash(held).unwrap();
    run(
        &mut given,
        "open class Target { public override fun value() -> Object { nil } }; nil",
    )
    .unwrap();

    given.collect_garbage().unwrap();

    assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    assert_eq!(
        run(&mut given, "retained.call().value()"),
        Ok(Value::Integer(41u64.into()))
    );
}

#[test]
fn retired_chain_capture_is_reclaimed_when_last_bound_method_is_dropped() {
    let mut given = fixture();
    run(&mut given, WRAPPED).unwrap();
    run(&mut given, "let retained = target.value; nil").unwrap();
    let held = object(run(&mut given, "retained.call()").unwrap());
    run(
        &mut given,
        "open class Target { public override fun value() -> Object { nil } }; nil",
    )
    .unwrap();
    given.names.remove("retained");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn finished_next_releases_payload_without_reviving_permission_after_collection() {
    let mut given = fixture();
    run(&mut given, r#"
        mut saved = %[]
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    saved.append(next)
                    let unused = { invocation }
                    nil
                })
            }
        }
        class Target { @Wrap() public fun value(value) -> Object { nil } }
        let held = Probe.new()
        Target.new().value(held)
        nil
    "#).unwrap();
    let held = object(given.names["held"].value());
    given.names.remove("held");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
    assert_eq!(
        run(
            &mut given,
            "try { saved[0].call(); false } catch error { true }"
        ),
        Ok(Value::Bool(true))
    );
}
