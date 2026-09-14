#![expect(clippy::expect_used, reason = "tests require compiled source")]

use iris_runtime::Value;
use iris_vm::{compile, run};

fn evaluate(decorator: &str, body: &str) -> Result<Value, iris_vm::MachineError> {
    let source = format!(
        r#"
class Effects {{
 public class property wrappers: Integer = 0
 public class property bodies: Integer = 0
}}
class Wrap {{}}
impl Wrap for MethodDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{
  Transformation.wrap_method({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   Effects.wrappers = Effects.wrappers + 1
   next.call()
  }})
 }}
}}
class Target {{
 {decorator} protected fun hidden() -> Integer {{
  Effects.bodies = Effects.bodies + 1
  7
 }}
 public fun value() -> Integer {{ self.hidden() }}
}}
{body}
"#
    );
    run(&compile(&source).expect("visibility source compiles"))
}

#[test]
fn protected_method_runs_when_called_inside_its_class() {
    for decorator in ["", "@Wrap()"] {
        let result = evaluate(decorator, "Target.new().value()");
        assert_eq!(result, Ok(Value::Integer(7_u64.into())));
    }
}

#[test]
fn protected_method_rejects_external_send_before_wrapper_or_body() {
    for decorator in ["", "@Wrap()"] {
        let result = evaluate(
            decorator,
            "let target = Target.new(); let error = try { target.hidden() } catch error { error }; %[error, Effects.wrappers, Effects.bodies]",
        );
        assert_eq!(
            result,
            Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
                Value::Symbol("MethodVisibilityError".into()),
                Value::Integer(0_u64.into()),
                Value::Integer(0_u64.into()),
            ])))
        );
    }
}

#[test]
fn protected_method_rejects_external_retention() {
    for decorator in ["", "@Wrap()"] {
        let result = evaluate(decorator, "Target.new().hidden");
        assert!(
            matches!(
                result,
                Err(iris_vm::MachineError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::VisibilityDenied { .. }
                    )
                ))
            ),
            "{result:?}"
        );
    }
}

#[test]
fn protected_override_calls_super_when_lexically_authorized() {
    let source = "class Parent { protected fun hidden() -> Integer { 7 } } class Child extends Parent { protected override fun hidden() -> Integer { super() + 1 } public fun value() -> Integer { self.hidden() } } Child.new().value()";
    let result = run(&compile(source).expect("protected super compiles"));
    assert_eq!(result, Ok(Value::Integer(8_u64.into())));
}

#[test]
fn wrapped_protected_super_runs_the_retained_chain() {
    let result = evaluate(
        "@Wrap()",
        "class Child extends Target { protected override fun hidden() -> Integer { super() + 1 } public override fun value() -> Integer { self.hidden() } } let result = Child.new().value(); %[result, Effects.wrappers, Effects.bodies]",
    );
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(8_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(1_u64.into()),
        ])))
    );
}
