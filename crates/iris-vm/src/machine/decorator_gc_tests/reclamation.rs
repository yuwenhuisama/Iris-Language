#![expect(clippy::panic, reason = "tests assert exact typed task outcomes")]

use super::*;

#[test]
fn retired_chain_is_reclaimed_when_last_bound_method_is_released() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class State { public class property captured: Object = nil }
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let payload = Payload.new()
  State.captured = payload
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   payload.value() + (next.call() as Integer)
  })
 }
}
class Target { @Wrap() public fun value() -> Integer { 7 } }
let state_class = State
let retained = Target.new().value
open class Target { public override fun value() -> Integer { 9 } }
nil
"#).expect("retired capture fixture compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    let state = class(&machine, "state_class");
    let slot = selector_id(&given, "captured").expect("captured slot");
    let payload = object(
        machine
            .runtime
            .class_var(state, slot)
            .expect("class storage")
            .expect("payload"),
    );
    machine
        .runtime
        .assign_class_var(state, slot, Value::Nil)
        .expect("release observation");
    machine.bindings.remove("retained");

    compact(&mut machine);

    assert_eq!(
        machine.runtime.identity_hash(payload),
        Err(RuntimeError::UnknownObjectId(payload))
    );
}

#[test]
fn completed_task_payload_is_reclaimed_when_last_task_binding_is_released() {
    let given = compile(
        r#"
class Payload {}
module Worker { public async fun run() { Payload.new() } }
let task = Worker.run()
let payload = Host.run(task)
nil
"#,
    )
    .expect("task payload fixture compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    let payload = object(machine.bindings.remove("payload").expect("payload binding"));
    compact(&mut machine);
    assert!(machine.runtime.identity_hash(payload).is_ok());
    machine.bindings.remove("task");

    compact(&mut machine);

    assert_eq!(
        machine.runtime.identity_hash(payload),
        Err(RuntimeError::UnknownObjectId(payload))
    );
}

#[test]
fn unobserved_failure_payload_survives_when_task_binding_is_released() {
    let given = compile(
        r#"
class Payload {}
module Worker { public async fun run() { raise Payload.new() } }
let task = Worker.run()
nil
"#,
    )
    .expect("failed task fixture compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    let Value::Task(task) = machine.bindings.remove("task").expect("task binding") else {
        panic!("expected Task");
    };
    let Err(error) = &machine.tasks[&task] else {
        panic!("failed Task");
    };
    let MachineError::Raised(raised) = error.as_ref() else {
        panic!("raised payload");
    };
    let payload = object(raised.0.clone());

    compact(&mut machine);

    assert!(machine.runtime.identity_hash(payload).is_ok());
    assert!(machine.unobserved_failures.contains(&task));
}
