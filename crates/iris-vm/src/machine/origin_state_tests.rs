use super::{DECORATORS, Machine, compile, run};
use iris_runtime::{ArrayRef, ClassId, Value};

const EFFECTS: &str = r#"
class Payload { }
class Effects {
 public class property count: Integer = 0
 public class property kept: Object = nil
 public class fun make() -> Object {
  Effects.count = Effects.count + 1
  Effects.kept = Payload.new()
  Payload.new()
 }
 public class fun bump() -> Integer {
  Effects.count = Effects.count + 1
  9
 }
 public class fun fail() -> Integer {
  Effects.count = Effects.count + 1
  raise :initializer_failed
 }
}
"#;

#[test]
fn shared_cells_retain_mutability_when_origin_initialization_commits() {
    let given = format!(
        r#"{DECORATORS}
@Add() class Target {{
 shared let @@fixed = 20
 shared mut @@count = 1
 public class fun read() {{ @@fixed + @@count }}
 public class fun bump() {{ @@count = @@count + 1 }}
}}
let bumped = Target.bump()
%[Target.read(), Target.active_revision]
"#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(22_u64.into()),
            Value::Integer(1_u64.into()),
        ])))
    );
}

#[test]
fn shared_immutable_cell_rejects_assignment_when_origin_is_published() {
    let given = format!(
        "{DECORATORS} @Add() class Target {{ shared let @@fixed = 20; public class fun overwrite() {{ @@fixed = 21 }} }} Target.overwrite()"
    );
    let when = run(&compile(&given).expect("compile"));
    assert!(matches!(
        when,
        Err(crate::MachineError::Construction(
            iris_runtime::ConstructionError::ImmutableClassVariable { .. }
        ))
    ));
}

#[test]
fn payload_is_rooted_in_single_commit_when_origin_is_published() {
    let given = compile(&format!("{EFFECTS} {DECORATORS} @Add() class Target {{ shared let @@fixed = 20; public class property held: Object = Effects.make() }} 0")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine.register_callable_types(&given).expect("callables");
    let mut classes = machine.register_classes(&given).expect("classes");
    let before = machine
        .runtime
        .registry()
        .active(classes[3])
        .expect("decorator")
        .clone();

    let when = machine.publish_origin(&given, &mut classes);

    assert_eq!(when, Ok(()));
    let target = classes[4];
    let after = machine.runtime.registry().active(target).expect("target");
    assert_eq!(after.number(), 1);
    assert_eq!(after.id().raw(), before.id().raw() + 1);
    assert_eq!(after.commit_id(), before.commit_id() + 1);
    let slot = super::super::selector_id(&given, "held").expect("held");
    let held = match machine.runtime.class_var(target, slot).expect("payload") {
        Some(Value::Object(object)) => Some(object),
        _ => None,
    }
    .expect("initializer payload must be published");
    machine.runtime.collect_garbage([]);
    assert_eq!(
        machine.runtime.class_of(held).expect("payload survives"),
        classes[0]
    );
    assert!(
        machine
            .pending_class_initializers
            .keys()
            .all(|(class, _)| *class != target)
    );
}

#[test]
fn initializer_runs_once_before_transform_when_property_is_unread() {
    let given = format!(
        r#"{EFFECTS}
class Check {{}}
impl Check for ClassDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{
  if Effects.count != 1 {{ raise :not_initialized }}
  Transformation.empty
 }}
}}
@Check() class Target {{ public class property count: Integer = Effects.bump() }}
%[Effects.count, Target.count, Target.count, Effects.count, Target.active_revision]
"#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(1_u64.into()),
        ])))
    );
}

#[test]
fn failed_initializer_discards_origin_when_earlier_payload_was_staged() {
    assert_failed_origin("Effects.fail()", "Transformation.empty", 2);
}

#[test]
fn failed_transform_discards_origin_when_initializers_succeeded() {
    assert_failed_origin("Effects.bump()", "raise :transform_failed", 2);
}

#[test]
fn public_construction_is_refused_when_initializer_uses_provisional_self() {
    assert_failed_origin("self.new()", "Transformation.empty", 1);
}

#[test]
fn public_name_is_hidden_when_initializer_loads_provisional_class() {
    assert_failed_origin("Target.new()", "Transformation.empty", 1);
}

fn assert_failed_origin(initializer: &str, transform: &str, effects: u64) {
    let given = compile(&format!(
        r#"{EFFECTS}
class Change {{}}
impl Change for ClassDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{ {transform} }}
}}
@Change() class Target {{
 shared let @@fixed = 20
 public class property first: Object = Effects.make()
 public class property second: Object = {initializer}
}}
0
"#
    ))
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine.register_callable_types(&given).expect("callables");
    let mut classes = machine.register_classes(&given).expect("classes");
    let before = machine
        .runtime
        .registry()
        .active(classes[2])
        .expect("decorator")
        .clone();
    let signatures = machine.method_signatures.len();
    let metadata = machine.decorator_metadata.len();
    let failed = ClassId::new(classes[2].raw() + 1);

    let when = machine.publish_origin(&given, &mut classes);

    assert!(when.is_err(), "{when:?}");
    assert_eq!(classes.len(), 3);
    assert!(machine.runtime.registry().class(failed).is_err());
    assert!(machine.runtime.registry().staged_origin(failed).is_err());
    assert_eq!(machine.method_signatures.len(), signatures);
    assert_eq!(machine.decorator_metadata.len(), metadata);
    assert!(
        machine
            .pending_class_initializers
            .keys()
            .all(|(class, _)| *class != failed)
    );
    let count = super::super::selector_id(&given, "count").expect("count");
    assert_eq!(
        machine
            .runtime
            .class_var(classes[1], count)
            .expect("effects"),
        Some(Value::Integer(effects.into()))
    );
    let kept = super::super::selector_id(&given, "kept").expect("kept");
    let kept = match machine.runtime.class_var(classes[1], kept).expect("kept") {
        Some(Value::Object(object)) => Some(object),
        _ => None,
    }
    .expect("initializer must retain its external effect");
    assert!(machine.runtime.collect_garbage([]).0 >= 1);
    assert_eq!(
        machine
            .runtime
            .class_of(kept)
            .expect("external object survives"),
        classes[0]
    );
    let valid = compile("class Valid { shared let @@fixed = 7 } 0").expect("valid");
    let mut next_classes = Vec::new();
    machine
        .publish_origin(&valid, &mut next_classes)
        .expect("next origin");
    let next = machine
        .runtime
        .registry()
        .active(next_classes[0])
        .expect("revision");
    assert_eq!(next.number(), 1);
    assert_eq!(next.id().raw(), before.id().raw() + 1);
    assert_eq!(next.commit_id(), before.commit_id() + 1);
}
