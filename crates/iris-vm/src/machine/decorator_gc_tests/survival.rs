use super::*;

#[test]
fn active_wrapper_captures_survive_collection_in_nested_native_fixture() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
module Collector { public fun collect() { NativeFixture.compact_gc() } }
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let captured = Payload.new()
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   Collector.collect()
   captured.value() + (next.call() as Integer)
  })
 }
}
class Target { @Wrap() public fun value(payload) -> Integer { payload.value() } }
Target.new().value(Payload.new())
"#).expect("active wrapper compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Integer(84_u64.into())));
}

#[test]
fn suspended_wrapper_and_next_survive_collection_before_owner_resumes() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class State { public class property gate: Object = nil }
class Park {}
impl Park for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let captured = Payload.new()
  Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
   await State.gate
   let inner = await next.call()
   captured.value() + (inner as Integer)
  })
 }
}
class Target { @Park() public async fun value(payload) -> Integer { payload.value() } }
let assigned = State.gate = Gate.new()
let task = Target.new().value(Payload.new())
let collected = NativeFixture.compact_gc()
let posted = Gate.complete(State.gate, 1)
Host.run(task)
"#).expect("suspended wrapper compiles");
    let mut machine = Machine::new().expect("machine");

    let when = machine.execute(&given);

    assert_eq!(when, Ok(Value::Integer(84_u64.into())));
    assert!(machine.suspended.is_empty());
}

#[test]
fn retained_method_keeps_retired_wrapper_captures_after_replacement_and_collection() {
    let given = compile(r#"
class Payload { public fun value() -> Integer { 42 } }
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let captured = Payload.new()
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   captured.value() + (next.call() as Integer)
  })
 }
}
class Target { @Wrap() public fun value() -> Integer { 7 } }
let target = Target.new()
let retained = target.value
open class Target { public override fun value() -> Integer { 900 } }
nil
"#).expect("retired wrapper compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    machine
        .bindings
        .remove("target")
        .expect("release direct receiver root");
    let Value::BoundMethod(retained) = machine.bindings["retained"].clone() else {
        panic!("retained Method binding");
    };

    compact(&mut machine);
    let when = machine.invoke_bound_callable(&retained, &[], (&given, &[]));

    assert_eq!(when, Ok(Value::Integer(49_u64.into())));
}

#[test]
fn class_root_reaches_closure_captures_through_object_fields_and_container_cycles() {
    let given = compile(
        r#"
class Payload { public fun value() -> Integer { 42 } }
class Holder { public property item: Object = nil }
class State { public class property saved: Object = nil }
module Factory {
 public fun make() {
  let payload = Payload.new()
  { || -> Object; payload }
 }
}
let state_class = State
let holder = Holder.new()
let kept = Factory.make()
let payload = kept.call()
nil
"#,
    )
    .expect("heap rooted closure compiles");
    let mut machine = Machine::new().expect("machine");
    assert_eq!(machine.execute(&given), Ok(Value::Nil));
    let payload = object(machine.bindings.remove("payload").expect("payload"));
    let hash = machine
        .runtime
        .identity_hash(payload)
        .expect("allocated payload");
    let closure = machine.bindings.remove("kept").expect("closure");
    let holder = object(machine.bindings.remove("holder").expect("holder"));
    let entries = iris_runtime::HashRef::new(vec![(Value::Symbol("callback".into()), closure)]);
    let cycle = iris_runtime::ArrayRef::new(vec![Value::Hash(entries)]);
    cycle.mutate(|elements| elements.push(Value::Array(cycle.clone())));
    machine
        .runtime
        .assign_raw_ivar(
            holder,
            selector_id(&given, "item").expect("item"),
            Value::Array(cycle),
        )
        .expect("heap edge");
    let state = class(&machine, "state_class");
    machine
        .runtime
        .assign_class_var(
            state,
            selector_id(&given, "saved").expect("saved"),
            Value::Object(holder),
        )
        .expect("class root");

    compact(&mut machine);

    assert_eq!(machine.runtime.identity_hash(payload), Ok(hash));
}
