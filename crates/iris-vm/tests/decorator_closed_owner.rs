#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

const SOURCE: &str = include_str!("closed_owner.iris");

#[test]
fn bound_call_materializes_when_origin_transformation_is_empty() {
    let given = format!(
        r#"{}
let box = Box<Integer>.new()
let bound = box.value
%[bound.call(7), bound.call(8), Effects.closed]
"#,
        SOURCE.replace(
            "mut calls = 0",
            "if c.reason == :origin { return Transformation.empty }; mut calls = 0"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(8_u64.into()),
            Value::Integer(10_u64.into()),
            Value::Integer(1_u64.into())
        ])))
    );
}

#[test]
fn closed_materialization_retries_when_transform_fails_before_publication() {
    let given = format!(r#"{}
let box = Box<Integer>.new()
let rejected = try {{ box.value(1); false }} catch error {{ error == :retry }}
%[rejected, box.value(2), box.value(3), Effects.closed]
"#, SOURCE.replace("mut calls = 0", "if c.reason == :closed_materialization && Effects.closed == 1 { raise :retry }; mut calls = 0"));
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(3_u64.into()),
            Value::Integer(5_u64.into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn wrapper_result_is_rejected_when_it_violates_closed_owner_type() {
    let given = format!(
        r#"{}
try {{ Box<Integer>.new().value(1); false }} catch error {{ error is? TypeError }}
"#,
        SOURCE.replace("(value as Integer) + calls", "\"wrong\"")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn bindings_survive_when_async_owners_interleave() {
    let given = compile(include_str!("closed_owner_async.iris")).expect("compile");
    let when = run(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(2_u64.into()),
            Value::Text("s".into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn owner_and_method_arguments_are_separate_when_names_shadow() {
    let given = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   if invocation.owner_type_arguments[0] != Integer.type { raise :owner }
   if invocation.method_type_arguments[0] != invocation.signature.result { raise :method }
   if invocation.signature.result == Integer.type { calls } else { next.call() }
  })
 }
}
class Box<T> {
 @Wrap() public fun value<U>(owner: T, value: U) -> U { let result: U = value; result }
 @Wrap() public fun shadow<T>(value: T) -> T { let result: T = value; result }
}
let box = Box<Integer>.new()
%[box.value<String>(1, "a"), box.shadow<String>("b"), box.shadow<Integer>(7), box.shadow<Integer>(8)]
"#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Text("a".into()),
            Value::Text("b".into()),
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn retained_chain_keeps_captures_when_compatible_open_installs_fresh_chain() {
    let given = format!(
        r#"{SOURCE}
let box = Box<Integer>.new()
let first = box.value(7)
let retained = box.value
open class Box {{ @Wrap() public override fun value(value: T) -> T {{ (value as Integer) + 10 }} }}
%[first, box.value(7), retained.call(7), Effects.closed]
"#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(8_u64.into()),
            Value::Integer(18_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn captures_are_isolated_when_closed_owners_alternate() {
    let given = format!(
        r#"{SOURCE}
        %[Box<Integer>.new().value(7), Box<String>.new().value("s"), Box<Integer>.new().value(9), Effects.origins, Effects.closed]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(8_u64.into()),
            Value::Text("s".into()),
            Value::Integer(11_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn input_has_zero_effect_when_closed_owner_type_rejects_it() {
    let given = format!(
        r#"{SOURCE}
let rejected = try {{ Box<Integer>.new().value("wrong"); false }} catch error {{ error is? TypeError }}
        %[rejected, Effects.bodies, Effects.wrappers]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into()),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn patch_has_zero_body_effect_when_closed_owner_type_rejects_it() {
    let given = format!(
        r#"{}
let rejected = try {{ Box<Integer>.new().value(1); false }} catch error {{ error is? TypeError }}
        %[rejected, Effects.bodies]
    "#,
        SOURCE.replace(
            "next.call()",
            "next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))"
        )
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}
