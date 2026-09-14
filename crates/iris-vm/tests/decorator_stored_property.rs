#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for PropertyDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  if declaration.name != :value { raise :name }
  if declaration.type != Integer.type { raise :type }
  let digit = arguments[0] as Integer
  mut reads = 0
  mut writes = 0
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   reads = reads + 1
   (next.call() as Integer) * 10 + digit + reads - 1
  }).wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   writes = writes + 1
   (next.call() as Integer) * 10 + digit + writes - 1
  })
 }
}
"#;

fn evaluate(given: &str) -> Result<Value, MachineError> {
    run(&compile(given).expect("stored property compiles"))
}

#[test]
fn chains_capture_independently_when_generated_accessors_share_storage() {
    let given = format!(
        "{WRAP} class Target {{ @Wrap(1) @Wrap(2) public property value: Integer = 7 {{ public get; public set; }} public fun raw() {{ @value }} }} let target = Target.new(); %[target.value, target.value = 8, target.value, target.value = 9, target.raw(), Target.active_revision]"
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [721_u64, 821, 832, 932, 9, 1]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn external_read_is_denied_when_accessor_defaults_private() {
    let given = "class Target { public property value: Integer = 7 { get; public set; } public fun read() { self.value } } let target = Target.new(); %[target.read(), try { target.value; false } catch error { error == :MethodVisibilityError }]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn absent_setter_is_rejected_when_property_decorator_requests_it() {
    let given =
        format!("{WRAP} class Target {{ @Wrap(1) property value: Integer = 7 {{ public get; }} }}");
    let when = evaluate(&given);
    assert_eq!(when, Err(MachineError::ArgumentError));
}

#[test]
fn setter_is_absent_when_only_getter_is_written() {
    let given = "class Target { property value: Integer = 7 { public get; } } let target = Target.new(); %[try { target.value = 9; false } catch error { error == :MessageNotFound }, target.value]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(7_u64.into())
        ])))
    );
}

#[test]
fn property_body_controls_wrapping_when_other_capabilities_are_denied() {
    let given = format!(
        "{WRAP} class Target meta deny method_body, method_set, property_set {{ @Wrap(1) property value: Integer = 7 {{ public get; public set; }} }} Target.new().value"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(71_u64.into())));
}

#[test]
fn wrapping_is_denied_when_property_body_is_denied() {
    let given = format!(
        "{WRAP} class Target meta deny property_body {{ @Wrap(1) property value: Integer = 7 {{ public get; public set; }} }}"
    );
    let when = evaluate(&given);
    assert!(
        matches!(
            when,
            Err(MachineError::Class(
                iris_runtime::ClassError::MetaCapabilityDenied {
                    operation: iris_runtime::Capability::PropertyBody,
                    ..
                }
            ))
        ),
        "{when:?}"
    );
}

#[test]
fn default_initialization_is_preserved_when_shorthand_omits_accessors() {
    let given = "class Target { public property value: Object } let target = Target.new(); %[target.value, target.value = 9, target.value]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Nil,
            Value::Integer(9_u64.into()),
            Value::Integer(9_u64.into())
        ])))
    );
}

#[test]
fn replacement_uses_same_slot_when_property_body_changes() {
    let given = "class Target { property value: Integer = 7 { public get; public set; } } let target = Target.new(); open class Target { public override property fun value() -> Integer { @value + 1 } public override property fun value=(value: Integer) -> Integer { @value = value + 2 } } %[target.value, target.value = 9, target.value]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [8_u64, 11, 12]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn initializers_run_once_in_order_when_each_instance_is_constructed() {
    let given = r#"
class Effects { public class property count: Integer = 0 }
class Base {
 property first: Integer = seed() { public get; }
 public fun seed() -> Integer { Effects.count = Effects.count + 1 }
}
class Target extends Base {
 property second: Integer = self.first + seed() { public get; }
 public fun initialize() { Effects.count = Effects.count + 1 }
}
let first = Target.new()
let second = Target.new()
%[first.first, first.second, second.first, second.second, Effects.count]
"#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [1_u64, 3, 4, 9, 6]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn setter_keeps_storage_when_input_contract_is_rejected() {
    let given = format!(
        "{WRAP} class Target {{ @Wrap(1) property value: Integer = 7 {{ public get; public set; }} public fun raw() {{ @value }} }} let target = Target.new(); %[try {{ target.value = false; false }} catch error {{ error is? TypeError }}, target.raw()]"
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(7_u64.into())
        ])))
    );
}

#[test]
fn accessor_result_is_checked_when_wrapper_short_circuits_generated_body() {
    for (operation, call) in [
        ("wrap_getter", "target.value"),
        ("wrap_setter", "target.value = 8"),
    ] {
        let given = format!(
            r#"
class Wrong {{}}
impl Wrong for PropertyDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{
  Transformation.{operation}({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; false }})
 }}
}}
class Target {{ @Wrong() property value: Integer = 7 {{ public get; public set; }} }}
let target = Target.new()
try {{ {call}; false }} catch error {{ error is? TypeError }}
"#
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{operation}");
    }
}

#[test]
fn empty_block_has_no_accessors_when_decorator_requests_getter() {
    let given = format!("{WRAP} class Target {{ @Wrap(1) property value: Integer = 7 {{ }} }}");
    let when = evaluate(&given);
    assert_eq!(when, Err(MachineError::ArgumentError));
}

#[test]
fn readonly_decorator_runs_once_when_only_getter_is_present() {
    let given = r#"
class Effects { public class property count: Integer = 0 }
class Read {}
impl Read for PropertyDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Effects.count = Effects.count + 1
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target { @Read() property value: Integer = 7 { public get; } }
let target = Target.new()
%[target.value, target.value, Effects.count, try { target.value = 8; false } catch error { error == :MessageNotFound }]
"#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Bool(true)
        ])))
    );
}
