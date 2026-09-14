#![expect(clippy::expect_used, reason = "tests require real VM fixtures")]

use super::{Machine, MachineError, selector_id};
use crate::compile;
use iris_runtime::{ClassId, DecoratorValue, ObjectId, RuntimeError, Value};

mod execution;
mod reclamation;
mod survival;

fn object(value: Value) -> ObjectId {
    let Value::Object(identity) = value else {
        panic!("expected heap object, got {value:?}");
    };
    identity
}

fn class(machine: &Machine, name: &str) -> ClassId {
    let Value::Class(identity) = machine.bindings[name] else {
        panic!("expected Class binding {name}");
    };
    identity
}

fn compact(machine: &mut Machine) {
    machine.native_fixture("compact_gc", &[]).expect("VM GC");
}

#[test]
fn ordinary_capture_is_reclaimed_when_its_last_program_binding_is_released() {
    let given = compile(
        r#"
class Payload { public fun value() -> Integer { 42 } }
module Factory {
 public fun make() {
  let payload = Payload.new()
  { || -> Object; payload }
 }
}
let kept = Factory.make()
let payload = kept.call()
nil
"#,
    )
    .expect("ordinary closure compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    let payload = object(machine.bindings.remove("payload").expect("payload binding"));
    let hash = machine
        .runtime
        .identity_hash(payload)
        .expect("allocated payload");
    compact(&mut machine);
    assert_eq!(machine.runtime.identity_hash(payload), Ok(hash));

    machine.bindings.remove("kept").expect("last closure root");
    compact(&mut machine);

    assert_eq!(
        machine.runtime.identity_hash(payload),
        Err(RuntimeError::UnknownObjectId(payload))
    );
}

#[test]
fn failed_transform_captures_follow_escaped_program_roots_not_candidate_bookkeeping() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class State {
 public class property saved: Object = nil
 public class property transient: Object = nil
}
class Escape {}
impl Escape for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let escaped = Payload.new()
  let candidate = Payload.new()
  State.saved = { || -> Object; escaped }
  State.transient = candidate
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   candidate.value() + (next.call() as Integer)
  })
 }
}
class Fail {}
impl Fail for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation { raise :failed }
}
class Target { public fun value() -> Integer { 7 } }
let state_class = State
open class Target {
 @Escape() public override fun value() -> Integer { 700 }
 @Fail() public fun late() -> Integer { 900 }
}
"#).expect("failed candidate compiles");
    let mut machine = Machine::new().expect("machine");
    let Err(MachineError::Raised(failure)) = machine.execute(&given) else {
        panic!("late transform must raise");
    };
    assert_eq!(failure.0, Value::Symbol("failed".into()));
    let state = class(&machine, "state_class");
    let saved_slot = selector_id(&given, "saved").expect("saved selector");
    let transient_slot = selector_id(&given, "transient").expect("transient selector");
    let Some(Value::Closure(saved)) = machine.runtime.class_var(state, saved_slot).expect("saved")
    else {
        panic!("closure must escape failed transform");
    };
    let escaped = object(
        machine
            .invoke_closure_value(saved, &[], &given, &[])
            .expect("escaped call"),
    );
    let candidate = object(
        machine
            .runtime
            .class_var(state, transient_slot)
            .expect("transient")
            .expect("candidate payload"),
    );
    let hash = machine
        .runtime
        .identity_hash(escaped)
        .expect("escaped payload");
    machine
        .runtime
        .assign_class_var(state, transient_slot, Value::Nil)
        .expect("release observation root");

    compact(&mut machine);
    let candidate_after_gc = machine.runtime.identity_hash(candidate);
    assert_eq!(machine.runtime.identity_hash(escaped), Ok(hash));
    assert_eq!(
        machine.invoke_closure_value(saved, &[], &given, &[]),
        Ok(Value::Object(escaped))
    );
    machine
        .runtime
        .assign_class_var(state, saved_slot, Value::Nil)
        .expect("release escaped closure root");
    compact(&mut machine);

    assert_eq!(
        (candidate_after_gc, machine.runtime.identity_hash(escaped)),
        (
            Err(RuntimeError::UnknownObjectId(candidate)),
            Err(RuntimeError::UnknownObjectId(escaped))
        )
    );
}

#[test]
fn finished_next_releases_invocation_objects_when_user_roots_are_released() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class State {
 public class property captured: Object = nil
 public class property saved: Object = nil
}
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let captured = Payload.new()
  State.captured = captured
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   State.saved = next
   captured.value() + (next.call() as Integer)
  })
 }
}
class Target { @Wrap() public fun value(payload) -> Integer { payload.value() } }
let state_class = State
let target = Target.new()
let payload = Payload.new()
let retained = target.value
retained.call(payload)
"#).expect("wrapper invocation compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Integer(84_u64.into())));
    let state = class(&machine, "state_class");
    let captured_slot = selector_id(&given, "captured").expect("captured selector");
    let saved_slot = selector_id(&given, "saved").expect("saved selector");
    let captured = object(
        machine
            .runtime
            .class_var(state, captured_slot)
            .expect("captured")
            .expect("wrapper payload"),
    );
    let hash = machine
        .runtime
        .identity_hash(captured)
        .expect("wrapper capture");
    let receiver = object(machine.bindings.remove("target").expect("receiver binding"));
    let payload = object(
        machine
            .bindings
            .remove("payload")
            .expect("argument binding"),
    );
    machine
        .runtime
        .assign_class_var(state, captured_slot, Value::Nil)
        .expect("release observation root");
    compact(&mut machine);
    assert!(machine.runtime.identity_hash(receiver).is_ok());
    assert_eq!(machine.runtime.identity_hash(captured), Ok(hash));
    let Some(Value::Closure(saved)) = machine
        .runtime
        .class_var(state, saved_slot)
        .expect("saved next")
    else {
        panic!("next must escape invocation");
    };
    let Err(MachineError::Raised(failure)) = machine.invoke_closure_value(saved, &[], &given, &[])
    else {
        panic!("collection must not revive expired next permission");
    };
    let Value::Decorator(record) = &failure.0 else {
        panic!("protocol error");
    };
    let DecoratorValue::ProtocolError(error) = record.as_ref() else {
        panic!("protocol error");
    };
    assert_eq!(
        error.category(),
        iris_runtime::decorator_protocol::ProtocolCategory::Expired
    );
    assert_eq!(error.owner(), Some(saved));

    machine
        .bindings
        .remove("retained")
        .expect("retained Method root");
    machine
        .runtime
        .assign_class_var(state, saved_slot, Value::Nil)
        .expect("release next root");
    compact(&mut machine);

    assert_eq!(machine.runtime.identity_hash(captured), Ok(hash));
    assert_eq!(
        (
            machine.runtime.identity_hash(receiver),
            machine.runtime.identity_hash(payload)
        ),
        (
            Err(RuntimeError::UnknownObjectId(receiver)),
            Err(RuntimeError::UnknownObjectId(payload))
        )
    );
}
