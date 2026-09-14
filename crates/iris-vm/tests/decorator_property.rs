#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

const BOTH: &str = r#"
class Wrap {}
impl Wrap for PropertyDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  let digit = arguments[0] as Integer
  mut reads = 0
  mut writes = 0
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   if invocation.slot[1] != :value { raise :selector }
   if invocation.slot[3] != :getter { raise :kind }
   reads = reads + 1
   (next.call() as Integer) * 10 + digit + reads - 1
  }).wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   if invocation.slot[1] != :value { raise :selector }
   if invocation.slot[3] != :setter { raise :kind }
   writes = writes + 1
   (next.call() as Integer) * 10 + digit + writes - 1
  })
 }
}
"#;

fn evaluate(given: &str) -> Result<Value, MachineError> {
    let program = compile(given).expect("property source compiles");
    run(&program)
}

#[test]
fn accessor_chains_are_independent_when_both_operations_target_getter_declaration() {
    let given = format!(
        "{BOTH} class Target {{ @Wrap(1) @Wrap(2) public property fun value() -> Integer {{ 7 }} public property fun value=(value: Integer) -> Integer {{ value }} }} let target = Target.new(); %[target.value, target.value = 8, target.value, target.value = 9]"
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [721_u64, 821, 732, 932]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn property_body_is_independent_when_method_or_property_set_capabilities_are_denied() {
    for denied in [
        "method_body",
        "method_set",
        "property_set",
        "method_body, method_set, property_set",
    ] {
        let given = format!(
            "{BOTH} class Target meta deny {denied} {{ @Wrap(1) public property fun value() -> Integer {{ 7 }} public property fun value=(value: Integer) -> Integer {{ value }} }} let target = Target.new(); %[target.value, target.value = 8]"
        );
        let when = evaluate(&given);
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(71_u64.into()),
                Value::Integer(81_u64.into())
            ]))),
            "{denied}"
        );
    }
}

#[test]
fn property_wrapping_is_denied_when_property_body_is_denied() {
    let given = format!(
        "{BOTH} class Target meta deny property_body {{ @Wrap(1) public property fun value() -> Integer {{ 7 }} public property fun value=(value: Integer) -> Integer {{ value }} }}"
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
fn missing_setter_aborts_candidate_when_other_changes_are_staged() {
    let given = format!(
        r#"{BOTH}
class Target {{ }}
let revision = Target.active_revision
let commit = Reflection::Class.revision(Target).fetch(:commit_id)
let failed = try {{ Target.open() {{ |candidate|;
 candidate.define_method(:staged, {{ || -> Integer; 99 }})
 @Wrap(1) public property fun value() -> Integer {{ 7 }}
 }}; false }} catch error {{ error is? ArgumentError }}
%[failed, Target.active_revision == revision, Reflection::Class.revision(Target).fetch(:commit_id) == commit, Target.method(:staged) == nil, Target.method(:value) == nil]
"#
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![Value::Bool(true); 5])))
    );
}

#[test]
fn accessor_result_is_checked_when_wrapper_short_circuits() {
    for (operation, declaration, call) in [
        ("wrap_getter", "value() -> Integer { 7 }", "target.value"),
        (
            "wrap_setter",
            "value=(value: Integer) -> Symbol { :written }",
            "target.value = 8",
        ),
    ] {
        let given = format!(
            r#"
class Wrong {{}}
impl Wrong for PropertyDecorator {{
 public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
 public fun transform(declaration, arguments, context) -> Transformation {{
  Transformation.{operation}({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; false }})
 }}
}}
class Target {{ @Wrong() public property fun {declaration} }}
let target = Target.new()
try {{ {call}; false }} catch error {{ error is? TypeError }}
"#
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{operation}");
    }
}

#[test]
fn callback_getter_is_invoked_when_published_with_property_metadata() {
    let given = format!(
        "{BOTH} class Target {{ public property fun value=(value: Integer) -> Integer {{ value }} }} let opened = Target.open() {{ |candidate|; @Wrap(1) public property fun value() -> Integer {{ 7 }} }}; Target.new().value"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(71_u64.into())));
}

#[test]
fn callback_property_replacement_uses_property_body_when_other_capabilities_are_denied() {
    let given = format!(
        "{BOTH} class Target meta deny method_body, method_set, property_set {{ public property fun value() -> Integer {{ 5 }} public property fun value=(value: Integer) -> Integer {{ value }} }} let opened = Target.open() {{ |candidate|; @Wrap(1) public property fun value() -> Integer {{ 7 }} }}; Target.new().value"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(71_u64.into())));
}

#[test]
fn ordinary_method_is_not_an_accessor_when_getter_is_requested() {
    let given = format!(
        "{BOTH} class Target {{ public fun value() -> Integer {{ 5 }} @Wrap(1) public property fun value=(value: Integer) -> Integer {{ value }} }}"
    );
    let when = evaluate(&given);
    assert_eq!(when, Err(MachineError::ArgumentError));
}

#[test]
fn property_wrappers_fit_when_stack_is_one_mib() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(accessor_chains_are_independent_when_both_operations_target_getter_declaration)
        .expect("thread starts")
        .join()
        .expect("property wrappers fit bounded stack");
}
