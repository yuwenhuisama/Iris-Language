use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ObjectId, Value};

pub(super) fn run(evaluator: &mut SourceEvaluator, source: &str) -> Result<Value, EvaluationError> {
    let parsed = iris_parser::parse(source);
    assert!(
        parsed.program_accepted,
        "{source}: {:?}",
        parsed.diagnostics
    );
    evaluator.set_source(source);
    evaluator.program(&parsed.program)
}

pub(super) fn object(value: Value) -> ObjectId {
    let Value::Object(identity) = value else {
        panic!("expected an ordinary object, got {value:?}")
    };
    identity
}

pub(super) fn fixture() -> SourceEvaluator {
    let mut evaluator = SourceEvaluator::new_in_package("gc.tests").unwrap();
    run(
        &mut evaluator,
        "class Probe { public fun value() -> Integer { 41 } }; nil",
    )
    .unwrap();
    evaluator
}

#[test]
fn capture_is_reclaimed_when_only_unreachable_closure_record_retains_it() {
    let mut given = fixture();
    run(&mut given, "module Factory { public fun make() { let held = Probe.new(); { held } } }; let saved = Factory.make(); nil").unwrap();
    let held = object(run(&mut given, "saved.call()").unwrap());
    given.names.remove("saved");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn completed_task_payload_is_reclaimed_when_program_drops_task() {
    let mut given = fixture();
    run(&mut given, "module Factory { public async fun make() -> Object { Probe.new() } }; let saved = Factory.make(); nil").unwrap();
    let held = object(run(&mut given, "Host.run(saved)").unwrap());
    given.names.remove("saved");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn escaped_capture_survives_when_reached_through_runtime_instance_storage() {
    let mut given = fixture();
    run(&mut given, "class Holder { public fun keep(value) { @held = value }; public fun read() { @held } }; module Factory { public fun make() { let held = Probe.new(); { held } } }; let saved = Factory.make(); let holder = Holder.new(); holder.keep(saved); nil").unwrap();
    let held = object(run(&mut given, "saved.call()").unwrap());
    let hash = given.runtime.identity_hash(held).unwrap();
    given.names.remove("saved");

    given.collect_garbage().unwrap();

    assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    assert_eq!(
        run(&mut given, "holder.read().call().value()"),
        Ok(Value::Integer(41u64.into()))
    );
}

#[test]
fn runtime_storage_capture_is_reclaimed_when_owning_instance_dies() {
    let mut given = fixture();
    run(&mut given, "class Holder { public fun keep(value) { @held = value } }; module Factory { public fun make() { let held = Probe.new(); { held } } }; let saved = Factory.make(); let holder = Holder.new(); holder.keep(saved); nil").unwrap();
    let held = object(run(&mut given, "saved.call()").unwrap());
    given.names.remove("saved");
    given.names.remove("holder");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn escaped_capture_survives_when_only_class_storage_retains_closure() {
    let mut given = fixture();
    run(&mut given, "class Holder { public class fun keep(value) { @held = value }; public class fun read() { @held } }; module Factory { public fun make() { let held = Probe.new(); { held } } }; let saved = Factory.make(); Holder.keep(saved); nil").unwrap();
    let held = object(run(&mut given, "saved.call()").unwrap());
    let hash = given.runtime.identity_hash(held).unwrap();
    given.names.remove("saved");

    given.collect_garbage().unwrap();

    assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    assert_eq!(
        run(&mut given, "Holder.read().call().value()"),
        Ok(Value::Integer(41u64.into()))
    );
}

#[test]
fn caller_mutable_cell_survives_when_async_body_shadows_same_name() {
    let mut given = fixture();

    let when = run(
        &mut given,
        "module Worker { public async fun inner() -> Object { mut held = 0; NativeFixture.compact_gc(); nil }; public fun outer() -> Integer { mut held = Probe.new(); inner(); held.value() } }; Worker.outer()",
    );

    assert_eq!(when, Ok(Value::Integer(41u64.into())));
}

#[test]
fn caller_mutable_cell_survives_when_sync_body_shadows_same_name() {
    let mut given = fixture();

    let when = run(
        &mut given,
        "module Worker { public fun inner() -> Nil { mut held = 0; NativeFixture.compact_gc(); nil }; public fun outer() -> Integer { mut held = Probe.new(); inner(); held.value() } }; Worker.outer()",
    );

    assert_eq!(when, Ok(Value::Integer(41u64.into())));
}

#[test]
fn earlier_argument_survives_when_later_argument_collects() {
    let mut given = fixture();

    let when = run(
        &mut given,
        "module Worker { public fun collect() -> Nil { NativeFixture.compact_gc(); nil }; public fun choose(first, second) -> Integer { first.value() } }; Worker.choose(Probe.new(), Worker.collect())",
    );

    assert_eq!(when, Ok(Value::Integer(41u64.into())));
}

#[test]
fn failed_task_value_survives_when_only_unobserved_diagnostics_retains_it() {
    let mut given = fixture();
    run(&mut given, "module Worker { public async fun fail() -> Object { raise Probe.new() } }; let failed = Worker.fail(); nil").unwrap();
    let held = object(given.unobserved_failures[0].1.clone());
    let hash = given.runtime.identity_hash(held).unwrap();
    given.names.remove("failed");

    given.collect_garbage().unwrap();

    assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    assert_eq!(
        run(&mut given, "Diagnostics.unobserved_failures().length()"),
        Ok(Value::Integer(1u64.into()))
    );
}

#[test]
fn pending_task_capture_survives_when_program_drops_task_handle() {
    let mut given = fixture();
    run(&mut given, "module Worker { public async fun wait(gate, held) -> Object { await gate; held.value() } }; let gate = Gate.new(); let held = Probe.new(); let pending = Worker.wait(gate, held); nil").unwrap();
    let held = object(given.names["held"].value());
    given.names.remove("held");
    given.names.remove("pending");

    given.collect_garbage().unwrap();

    assert!(given.runtime.identity_hash(held).is_ok());
    run(&mut given, "Gate.complete(gate, nil)").unwrap();
    given.drive_ready_continuations().unwrap();
    assert!(given.tasks.values().any(|outcome| {
        outcome
            .as_ref()
            .is_ok_and(|value| *value == Value::Integer(41u64.into()))
    }));
}
