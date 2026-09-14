#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

const SOURCE: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   if invocation.receiver != invocation.slot[0] { raise :receiver }
   if invocation.method_type_arguments.length() != 1 { raise :arity }
   if invocation.signature.parameters[0].type != invocation.method_type_arguments[0] { raise :parameter }
   if invocation.signature.result != invocation.method_type_arguments[0] { raise :result }
   next.call()
  })
 }
}
class Target {
 @Wrap() public class fun echo<Element>(value: Element) -> Element { value }
}
"#;

#[test]
fn generic_integer_is_closed_when_explicitly_selected() {
    let given = format!("{SOURCE} Target.echo<Integer>(7)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn generic_string_is_closed_when_dynamic_receiver_is_selected() {
    let given = format!("{SOURCE} let target = Target; target.echo<String>(\"hello\")");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Text("hello".into())));
}

#[test]
fn generic_input_is_rejected_when_closed_type_differs() {
    let given = format!(
        "{SOURCE} try {{ Target.echo<Integer>(\"wrong\") }} catch error {{ error is? TypeError }}"
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_patch_is_rejected_when_closed_type_differs() {
    let given = format!(
        "{} try {{ Target.echo<String>(\"hello\") }} catch error {{ error is? TypeError }}",
        SOURCE.replace(
            "next.call()",
            "next.call(ArgumentChanges.new(positional: %{:value: 7}))"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_return_is_rejected_when_zero_attempt_wrapper_violates_result() {
    let given = format!(
        "{} try {{ Target.echo<Integer>(7) }} catch error {{ error is? TypeError }}",
        SOURCE.replace("next.call()", "\"wrong\"")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_cli_fixture_runs_when_types_are_closed() {
    let given = include_str!("../../iris-cli/tests/decorator_targets/class_object_generic.iris");
    let when = run(&compile(given).expect("compile"));
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn generic_body_return_is_checked_when_original_body_violates_result() {
    let given = format!(
        "{} try {{ Target.echo<Integer>(7) }} catch error {{ error is? TypeError }}",
        SOURCE.replace("{ value }", "{ \"wrong\" }")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_type_arity_is_checked_when_arguments_are_supplied() {
    let given = format!(
        "{SOURCE} try {{ Target.echo<Integer, String>(7) }} catch error {{ error is? ArgumentError }}"
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_type_is_resolved_when_argument_name_is_unknown() {
    let given = format!("{SOURCE} Target.echo<Missing>(7)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Err(iris_vm::MachineError::NameError));
}

#[test]
fn generic_inference_is_closed_when_direct_fixed_input_is_known() {
    let given = format!("{SOURCE} Target.echo(7)");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn generic_capture_state_is_independent_when_closed_types_differ() {
    let given = format!(
        "{} %[Target.echo<Integer>(0), Target.echo<String>(\"first\"), Target.echo<Integer>(0)]",
        SOURCE.replace(
            "next.call()",
            "if invocation.method_type_arguments[0] == Integer.type { calls } else { next.call() }"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Text("first".into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn generic_bindings_are_restored_when_nested_call_uses_same_parameter_name() {
    let given = format!(
        "{} class Other {{ public class fun echo<Element>(value: Element) -> Element {{ value }} }} Target.echo<Integer>(7)",
        SOURCE.replace(
            "{ value }",
            "{ let ignored = Other.echo<String>(\"nested\"); value }"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn generic_fixture_runs_when_stack_is_one_mib() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(generic_cli_fixture_runs_when_types_are_closed)
        .expect("thread")
        .join()
        .expect("bounded stack");
}

#[test]
fn generic_string_metadata_is_exact_when_string_is_selected() {
    let given = format!("{} Target.echo<String>(\"hello\")", SOURCE.replace("next.call()", "if invocation.method_type_arguments[0] != String.type { raise :wrong_type }; next.call()"));
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Text("hello".into())));
}

#[test]
fn generic_defaults_run_once_when_next_retries() {
    let given = format!(
        "{} class Effects {{ public class property count: Integer = 0; public class fun bump() -> Integer {{ Effects.count = Effects.count + 1; 7 }} }} let result = Target.echo<Integer>(); Effects.count",
        SOURCE
            .replace("value: Element)", "value: Element = Effects.bump())")
            .replace("next.call()", "next.call(); next.call()")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn generic_transform_runs_once_when_closed_materialization_is_reused() {
    let given = format!(
        "class Effects {{ public class property count: Integer = 0 }} {} let first = Target.echo<Integer>(7); let second = Target.echo<String>(\"hello\"); let third = Target.echo<Integer>(8); Effects.count",
        SOURCE.replace(
            "mut calls = 0",
            "Effects.count = Effects.count + 1; mut calls = 0"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(3_u64.into())));
}

#[test]
fn generic_module_call_uses_explicit_types_when_lowered_to_call_ir() {
    let given = "module Helpers { public fun echo<Element>(value: Element) -> Element { value } } Helpers.echo<Integer>(\"wrong\")";
    let when = run(&compile(given).expect("compile"));
    assert_eq!(when, Err(iris_vm::MachineError::TypeContractError));
}

#[test]
fn generic_type_arity_is_checked_when_target_is_nongeneric() {
    let given = "class Target { public class fun echo(value: Integer) -> Integer { value } } try { Target.echo<Integer>(7) } catch error { error is? ArgumentError }";
    let when = run(&compile(given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_input_prevents_wrapper_effects_when_string_contract_is_violated() {
    let given = format!(
        "{} class Effects {{ public class property count: Integer = 0 }} let result = try {{ Target.echo<String>(7) }} catch error {{ error is? TypeError }}; %[result, Effects.count]",
        SOURCE.replace(
            "calls = calls + 1",
            "calls = calls + 1; Effects.count = Effects.count + 1"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn generic_patch_prevents_body_effects_when_integer_contract_is_violated() {
    let given = format!(
        "{} class Effects {{ public class property count: Integer = 0 }} let result = try {{ Target.echo<Integer>(7) }} catch error {{ error is? TypeError }}; %[result, Effects.count]",
        SOURCE
            .replace(
                "next.call()",
                "next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))"
            )
            .replace("{ value }", "{ Effects.count = Effects.count + 1; value }")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn generic_string_return_rejects_integer_when_wrapper_short_circuits() {
    let given = format!(
        "{} try {{ Target.echo<String>(\"hello\") }} catch error {{ error is? TypeError }}",
        SOURCE.replace("next.call()", "7")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}
