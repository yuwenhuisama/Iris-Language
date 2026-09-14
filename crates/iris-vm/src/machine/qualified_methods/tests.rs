#![expect(clippy::expect_used, reason = "tests require compiled VM fixtures")]

use super::*;
use iris_runtime::Value;

#[test]
fn qualified_cache_survives_when_open_transformation_fails() {
    let source = include_str!("../../../tests/closed_qualified.iris");
    let given = crate::compile(&format!(
        r#"{source}
        class Fail {{}}
        impl Fail for MethodDecorator {{
            public fun plan(d, a) -> Plan {{ Plan.empty }}
            public fun transform(d, a, c) -> Transformation {{ raise :failed }}
        }}
        let box = Box<Integer>.new()
        let first = (box as Named<Integer>)..value(7)
        open class Box {{ @Fail() public override fun value(value: T) -> T {{ value + 10 }} }}
    "#
    ))
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    let when = machine.execute(&given);
    assert!(matches!(when, Err(MachineError::Raised(_))));
    let closed = &machine.closed_methods[0];
    assert!(machine.qualified_methods.slots.keys().any(|slot| {
        machine
            .qualified_method(*slot)
            .is_some_and(|method| method.id() == closed.canonical)
    }));
    assert!(machine.wrapper_chains.contains_key(&closed.method.id()));
}

#[test]
fn closed_identity_is_reused_when_views_repeat() {
    let given = crate::compile("contract Named<Element> {} class Box<T> {} impl Box<T> for Named<T> {} let box = Box<Integer>.new(); %[box as Named<Integer>, box as Named<Integer>]").expect("compile");
    let mut machine = Machine::new().expect("machine");
    let when = machine.execute(&given).expect("views");
    let Value::Array(values) = when else {
        unreachable!()
    };
    let values = values.elements();
    assert_eq!(values[0], values[1]);
    assert_eq!(machine.qualified_methods.views.len(), 1);
}

#[test]
fn closed_chain_keeps_owner_when_program_is_dropped_after_republication() {
    let source = include_str!("../../../tests/closed_qualified.iris");
    let original = crate::compile(&format!("{source} let box = Box<Integer>.new(); let first = (box as Named<Integer>)..value(7); open class Box {{ @Wrap() public override fun value(value: T) -> T {{ value + 10 }} }} box")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let receiver = machine.execute(&original).expect("execute");
    let old = machine.closed_methods[0].method;
    let current = *machine
        .qualified_methods
        .slots
        .values()
        .find(|method| method.owner() == old.owner())
        .expect("current");
    machine
        .active_values
        .extend([Value::Method(old), receiver.clone()]);
    let unrelated = crate::compile("class Other { public fun value() { 99 } } Other.new().value()")
        .expect("unrelated");
    machine.execute(&unrelated).expect("execute unrelated");
    drop(original);
    machine.collect_engine();
    let when = machine.invoke_wrapped(
        old,
        vec![receiver, Value::Integer(9_u64.into())],
        &unrelated,
        &[],
    );
    assert_ne!(old.id(), current.id());
    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn pending_qualified_frames_keep_owner_when_caller_changes() {
    let source = include_str!("../../../tests/closed_qualified_async.iris");
    let (declarations, _) = source.split_once("let first_gate =").expect("entry");
    let original = crate::compile(&format!("{declarations} let gate = Gate.new(); %[gate, (Box<String>.new() as Named<String>)..value(\"retained\", gate)]")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let Value::Array(values) = machine.execute(&original).expect("pending") else {
        unreachable!()
    };
    let values = values.elements();
    let Value::Gate(gate) = values[0] else {
        unreachable!()
    };
    machine.active_values.extend(values.iter().cloned());
    let current = crate::compile("class Other { public fun value() { 99 } } Other.new().value()")
        .expect("current");
    machine.execute(&current).expect("execute current");
    drop(original);
    machine.collect_engine();
    machine.gates.insert(gate, Some(Value::Nil));
    let when = machine.observe_task(values[1].clone(), &current, &[]);
    assert_eq!(when, Ok(Value::Text("retained".into())));
}

const SOURCE: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; (next.call() as Integer) + 10 })
 }
}
contract Named { fun value() -> Integer }
class Target {}
impl Target for Named {
 @Wrap() public fun value() -> Integer { 7 }
}
Target.new()
"#;

#[test]
fn old_chain_is_stable_when_qualified_identity_is_republished() {
    let program = crate::compile(SOURCE).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let receiver = machine.execute(&program).expect("execute");
    let selector = selector_id(&program, "value").expect("selector");
    let slot = *machine
        .qualified_methods
        .slots
        .keys()
        .find(|(_, _, known)| *known == selector)
        .expect("Target slot");
    let old = machine.qualified_method(slot).expect("selected method");
    assert_eq!(
        machine
            .runtime
            .registry()
            .resolve_local_or_ancestor_method(slot.0, selector)
            .expect("ordinary method"),
        old
    );
    assert!(machine.wrapper_chains.contains_key(&old.id()));
    let function = machine
        .resolve_method_body(old.body(), &program)
        .expect("body")
        .function;
    let reopen = crate::compile::ClassReopen {
        mixins: Vec::new(),
        qualified_impls: Vec::new(),
        methods: vec![("value".into(), function)],
        class_methods: Vec::new(),
    };
    machine
        .apply_members((slot.0, &reopen), &program)
        .expect("publish");
    let current = machine.qualified_method(slot).expect("republished method");
    assert_eq!(
        machine.invoke_selected_method(current, vec![receiver.clone()], &program, &[]),
        Ok(Value::Integer(7_u64.into()))
    );
    let when = machine.invoke_wrapped(old, vec![receiver], &program, &[]);
    assert_ne!(old.id(), current.id());
    assert_eq!(when, Ok(Value::Integer(17_u64.into())));
    assert!(machine.method_signatures.contains_key(&old.id()));
}

#[test]
fn whole_origin_rolls_back_when_qualified_body_capability_is_denied() {
    let given = SOURCE.replace("class Target {}", "class Target meta deny method_body {}");
    let program = crate::compile(&given).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let mut classes = machine.register_classes(&program).expect("classes");
    let before = machine
        .runtime
        .registry()
        .active(*classes.last().expect("decorator"))
        .expect("revision")
        .clone();
    let signatures = machine.method_signatures.len();
    let qualified = machine.qualified_methods.clone();
    let static_impls = machine.static_impl_slots.clone();
    let when = machine.publish_origin(&program, &mut classes);
    assert!(matches!(
        when,
        Err(MachineError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
    assert_eq!(machine.qualified_methods.slots, qualified.slots);
    assert_eq!(machine.qualified_methods.records, qualified.records);
    assert_eq!(machine.qualified_methods.definitions, qualified.definitions);
    assert_eq!(machine.static_impl_slots, static_impls);
    assert!(machine.wrapper_chains.is_empty());
    assert_eq!(machine.method_signatures.len(), signatures);
    let valid = crate::compile("class Valid { public fun value() -> Integer { 9 } }")
        .expect("compile valid");
    let mut valid_classes = Vec::new();
    machine
        .publish_origin(&valid, &mut valid_classes)
        .expect("valid origin");
    let after = machine
        .runtime
        .registry()
        .active(valid_classes[0])
        .expect("revision");
    assert_eq!(after.id().raw(), before.id().raw() + 1);
    assert_eq!(after.commit_id(), before.commit_id() + 1);
}
