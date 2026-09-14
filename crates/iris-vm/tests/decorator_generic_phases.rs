#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

const DECORATOR: &str = r#"
class Effects {
 public class property count: Integer = 0
 public class property reason: Symbol = :unset
}
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan {
  if declaration.kind != :method { raise :kind }
  if declaration.selector != :echo { raise :selector }
  Plan.empty
 }
 public fun transform(declaration, arguments, context) -> Transformation {
  if declaration.name != :echo { raise :name }
  if declaration.visibility != :public { raise :visibility }
  if declaration.owner == nil { raise :owner }
  Effects.count = Effects.count + 1
  Effects.reason = context.reason
  print(context.reason)
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   (next.call() as Integer) + calls
  })
 }
}
"#;

fn check_phases(declaration: &str, receiver: &str, reason: &str) {
    let given = format!(
        r#"{DECORATOR}
{declaration}
let initial_count = Effects.count
let initial_reason = Effects.reason
let target = {receiver}
let first = target.echo<Integer>(10)
let first_count = Effects.count
let first_reason = Effects.reason
let second = target.echo<String>(10)
let second_count = Effects.count
let second_reason = Effects.reason
let third = target.echo<Integer>(10)
%[initial_count, first_count, second_count, Effects.count,
 initial_reason, first_reason, second_reason, Effects.reason,
 first, second, third]
"#
    );

    let when = run(&compile(&given).expect("compile"));

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into()),
            Value::Integer(3_u64.into()),
            Value::Integer(3_u64.into()),
            Value::Symbol(reason.into()),
            Value::Symbol("closed_materialization".into()),
            Value::Symbol("closed_materialization".into()),
            Value::Symbol("closed_materialization".into()),
            Value::Integer(11_u64.into()),
            Value::Integer(11_u64.into()),
            Value::Integer(12_u64.into()),
        ])))
    );
}

#[test]
fn origin_and_closed_transforms_run_when_class_method_is_generic() {
    check_phases(
        "class Target { @Wrap() public class fun echo<Element>(value: Integer) -> Integer { value } }",
        "Target",
        "origin",
    );
}

#[test]
fn origin_and_closed_transforms_run_when_instance_method_is_generic() {
    check_phases(
        "class Target { @Wrap() public fun echo<Element>(value: Integer) -> Integer { value } }",
        "Target.new()",
        "origin",
    );
}

#[test]
fn open_and_closed_transforms_run_when_class_method_is_generic() {
    check_phases(
        "class Target { public class fun echo<Element>(value: Integer) -> Integer { value } } open class Target { @Wrap() public override class fun echo<Element>(value: Integer) -> Integer { value } }",
        "Target",
        "open",
    );
}

#[test]
fn open_and_closed_transforms_run_when_instance_method_is_generic() {
    check_phases(
        "class Target { public fun echo<Element>(value: Integer) -> Integer { value } } open class Target { @Wrap() public override fun echo<Element>(value: Integer) -> Integer { value } }",
        "Target.new()",
        "open",
    );
}
