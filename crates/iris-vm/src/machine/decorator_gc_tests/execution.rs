use super::*;

#[test]
fn caller_registers_survive_when_nested_callee_collects() {
    let given = compile(
        r#"
class Payload { public fun value() -> Integer { 42 } }
module Collector { public fun collect() { NativeFixture.compact_gc() } }
module Caller {
 public fun run() {
  let payload = Payload.new()
  Collector.collect()
  payload.value()
 }
}
Caller.run()
"#,
    )
    .expect("caller fixture compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn ready_queue_frames_survive_when_first_resumed_task_collects() {
    let given = compile(
        r#"
class Payload { public fun value() -> Integer { 42 } }
module Worker {
 public async fun first(gate) {
  await gate
  NativeFixture.compact_gc()
 }
 public async fun second(gate) {
  let payload = Payload.new()
  await gate
  payload.value()
 }
}
let gate = Gate.new()
let first = Worker.first(gate)
let second = Worker.second(gate)
let posted = Gate.complete(gate, 1)
Host.run(second)
"#,
    )
    .expect("ready queue fixture compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn closed_method_capture_survives_when_canonical_method_remains_published() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let payload = Payload.new()
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   payload.value()
   next.call()
  })
 }
}
class Target { @Wrap() public class fun value<T>(item: T) -> T { item } }
let first = Target.value<Integer>(7)
let collected = NativeFixture.compact_gc()
Target.value<Integer>(9)
"#).expect("closed method fixture compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn register_publication_preserves_share_count_when_iterator_releases_source() {
    let given = compile(
        r#"
module Probe {
 public fun run() {
  let array = %[1]
  let before = array.share_count()
  let iterator = array.iterator()
  let held = array.share_count()
  let first = iterator.next()
  let done = iterator.next()
  let after = array.share_count()
  held > before && after == before
 }
}
Probe.run()
"#,
    )
    .expect("sharing fixture compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Bool(true)));
}
