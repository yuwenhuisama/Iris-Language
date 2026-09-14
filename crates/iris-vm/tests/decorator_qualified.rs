#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   (next.call() as Integer) + 10
  })
 }
}
contract Named { fun value(input: Integer) -> Integer }
"#;

#[test]
fn qualified_fixture_runs_when_impl_populates_ordinary_surface() {
    let given = format!(
        r#"{WRAP}
        class Target {{}}
        impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }}
        let target = Target.new()
        %[target.value(1), (target as Named)..value(2), target.value(3)]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(11_u64.into()),
            Value::Integer(12_u64.into()),
            Value::Integer(13_u64.into()),
        ])))
    );
}

#[test]
fn wrapper_metadata_is_ordinary_when_impl_precedes_owner() {
    let given = format!(
        r#"{}
        impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }}
        class Target {{}}
        (Target.new() as Named)..value(7)
        "#,
        WRAP.replace(
            "(next.call() as Integer) + 10",
            "if !(invocation.slot[0] is? Type) || invocation.slot[0] != Target.type || invocation.slot[0] == Target || invocation.slot[1] != :value || invocation.slot[2] != nil || invocation.slot[3] != :method { raise :slot }; (next.call() as Integer) + 10"
        )
    );
    let when = run(&compile(&given).expect("compile impl before owner"));
    assert_eq!(when, Ok(Value::Integer(17_u64.into())));
}

#[test]
fn qualified_body_is_captured_when_wrapper_calls_next() {
    let given = format!(
        "{WRAP} class Target {{}} impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }} (Target.new() as Named)..value(7)"
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(17_u64.into())));
}

#[test]
fn qualified_input_is_checked_when_caller_passes_wrong_type() {
    let given = format!(
        "{WRAP} class Target {{}} impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }} try {{ (Target.new() as Named)..value(:wrong) }} catch error {{ error is? TypeError }}"
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn qualified_fixture_runs_when_stack_is_one_mib() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(qualified_fixture_runs_when_impl_populates_ordinary_surface)
        .expect("thread")
        .join()
        .expect("bounded stack");
}

#[test]
fn qualified_result_is_checked_when_wrapper_changes_return_type() {
    let given = format!(
        "{} class Target {{}} impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }} try {{ (Target.new() as Named)..value(7) }} catch error {{ error is? TypeError }}",
        WRAP.replace("(next.call() as Integer) + 10", ":wrong")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn qualified_slots_share_body_when_two_contracts_have_compatible_selector() {
    let given = format!(
        "{WRAP} contract Other {{ fun value(input: Integer) -> Integer }} class Target {{}} impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }} impl Target for Other {{}} let target = Target.new(); %[target.value(1), (target as Named)..value(7), (target as Other)..value(3)]"
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(11_u64.into()),
            Value::Integer(17_u64.into()),
            Value::Integer(13_u64.into()),
        ])))
    );
}

#[test]
fn impl_is_rejected_when_shared_selector_has_incompatible_contract() {
    let given = format!(
        "{WRAP} contract Other {{ fun value(input: String) -> String }} class Target {{}} impl Target for Named {{ @Wrap() public fun value(input: Integer) -> Integer {{ input }} }} impl Target for Other {{}}"
    );
    let when = run(&compile(&given).expect("compile incompatible candidate"));
    assert_eq!(when, Err(MachineError::TypeContractError));
}
