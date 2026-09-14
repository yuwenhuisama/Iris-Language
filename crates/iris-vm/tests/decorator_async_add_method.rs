#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn class_decorator_generated_async_add_method_is_awaited_by_public_method() {
    let source = r#"
class Add {}
impl Add for ClassDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.add_method(:added, { async || -> Integer; 42 })
 }
}
@Add()
class Target { public async fun value() -> Integer { await self.added() } }
Host.run(Target.new().value())
"#;
    let result = run(&compile(source).expect("class decorator fixture compiles"));
    assert_eq!(result, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn module_decorator_generated_async_add_method_is_awaited_by_public_method() {
    let source = r#"
class Add {}
impl Add for ModuleDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.add_method(:added, { async || -> Integer; 42 })
 }
}
@Add()
module Provider { public async fun value() -> Integer { await self.added() } }
Host.run(Provider.value())
"#;
    let result = run(&compile(source).expect("module decorator fixture compiles"));
    assert_eq!(result, Ok(Value::Integer(42_u64.into())));
}
