#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   if invocation.receiver != invocation.slot[0] { raise :receiver }
   if invocation.signature.parameters[0].type != Integer.type { raise :parameter }
   if invocation.signature.result != Integer.type { raise :result }
   if invocation.method_type_arguments.length() != 0 { raise :arguments }
   (next.call() as Integer) + calls
  })
 }
}
class Target {
 @Wrap() public class fun value(input: Integer) -> Integer { input }
 public fun value(input: Integer) -> Integer { 100 }
}
"#;

#[test]
fn class_object_chain_persists_when_calls_repeat() {
    let given = format!("{WRAP} let first = Target.value(7); Target.value(first)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(10_u64.into())));
}

#[test]
fn class_object_input_is_checked_when_wrong_type_is_passed() {
    let given =
        format!("{WRAP} try {{ Target.value(:wrong) }} catch error {{ error is? TypeError }}");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn class_object_result_is_checked_when_wrapper_returns_wrong_type() {
    let given = format!(
        "{} try {{ Target.value(7) }} catch error {{ error is? TypeError }}",
        WRAP.replace("(next.call() as Integer) + calls", ":wrong")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn instance_namespace_is_unchanged_when_singleton_is_wrapped() {
    let given = format!("{WRAP} Target.new().value(7)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(100_u64.into())));
}

#[test]
fn class_object_chain_is_admitted_when_receiver_is_a_value() {
    let given = format!("{WRAP} let target = Target; target.value(7)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(8_u64.into())));
}

#[test]
fn class_object_capture_is_refused_when_member_extraction_is_unsupported() {
    let given =
        format!("{WRAP} let held = Target.value; let first = held.call(7); held.call(first)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Err(iris_vm::MachineError::MessageNotFound {
            receiver_class: "Class".into(),
            selector: "value".into(),
        })
    );
}

#[test]
fn class_object_patch_is_checked_when_replacement_has_wrong_type() {
    let given = format!(
        "{} try {{ Target.value(7) }} catch error {{ error is? TypeError }}",
        WRAP.replace(
            "(next.call() as Integer) + calls",
            "next.call(ArgumentChanges.new(positional: %{:input: :wrong}))"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn class_object_defaults_run_once_when_wrapper_retries() {
    let given = format!(
        "{} class Effects {{ public class property count: Integer = 0; public class fun bump() -> Integer {{ Effects.count = Effects.count + 1; 7 }} }} let result = Target.value(); Effects.count",
        WRAP.replace(
            "(next.call() as Integer) + calls",
            "next.call(); next.call()"
        )
        .replace(
            "public class fun value(input: Integer)",
            "public class fun value(input: Integer = Effects.bump())"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn class_object_chain_runs_when_stack_is_one_mib() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(class_object_chain_persists_when_calls_repeat)
        .expect("thread")
        .join()
        .expect("bounded stack");
}
