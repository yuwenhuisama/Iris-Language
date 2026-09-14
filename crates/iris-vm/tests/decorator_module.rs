#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

const ADD: &str = r#"
class Add {}
impl Add for ModuleDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  let held = a[0] as Integer
  mut calls = 0
  Transformation.add_method(:added, { |delta: Integer| -> Integer;
   calls = calls + 1
   held + delta + calls
  })
 }
}
"#;

#[test]
fn captures_are_private_and_fresh_when_modules_are_decorated() {
    let source = format!(
        r#"{ADD}
@Add(10) module First {{ public fun value() -> Integer {{ self.added(1) }} }}
@Add(20) module Second {{ public fun value() -> Integer {{ self.added(1) }} }}
class A mixin First {{}}
class B mixin Second {{}}
let first = A.new()
%[first.value(), first.value(), B.new().value(), try {{ first.added(1) }} catch error {{ error }}]
"#
    );
    let program = compile(&source).expect("Module fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(12_u64.into()),
            Value::Integer(13_u64.into()),
            Value::Integer(22_u64.into()),
            Value::Symbol("MethodVisibilityError".into()),
        ])))
    );
}

#[test]
fn addition_is_rejected_when_module_denies_method_set() {
    let source = format!("{ADD} @Add(1) module Provider meta deny method_set {{}} 0");
    let program = compile(&source).expect("capability fixture compiles");
    let result = run(&program);
    assert!(
        matches!(result, Err(iris_vm::MachineError::Raised(_))),
        "{result:?}"
    );
}

#[test]
fn plan_is_checked_when_module_decorator_is_impure() {
    let source = "class Bad {} impl Bad for ModuleDecorator { public fun plan(d, a) -> Plan { print(:effect); Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Bad() module Provider {} 0";
    let program = compile(source).expect("planning fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Err(iris_vm::MachineError::LexicalDiagnostic(
            "IRIS-DECORATOR-NONDETERMINISTIC"
        ))
    );
}

#[test]
fn module_name_is_hidden_when_transform_reads_unpublished_target() {
    let source = "class Inspect {} impl Inspect for ModuleDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { let leaked = Provider; Transformation.empty } } @Inspect() module Provider {} 0";
    let program = compile(source).expect("unpublished fixture compiles");
    let result = run(&program);
    assert_eq!(result, Err(iris_vm::MachineError::NameError));
}

#[test]
fn module_private_access_is_denied_when_foreign_module_has_class_grant() {
    let source = format!(
        r#"{ADD}
@Add(1) module Provider {{ public fun value() -> Integer {{ self.added(1) }} }}
module Foreign {{ public fun steal() -> Integer {{ self.added(1) }} }}
class Target mixin Provider, Foreign private {{}}
try {{ Target.new().steal() }} catch error {{ error }}
"#
    );
    let program = compile(&source).expect("foreign private fixture compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Symbol("MethodVisibilityError".into())));
}

#[test]
fn decorator_runs_once_when_main_and_composed_surfaces_share_member() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |inv: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   if inv.slot[0] != Provider { raise :owner }
   (next.call() as Integer) + calls
  })
 }
}

module Provider { @Wrap() public fun value(value: Integer) -> Integer { value } }
class Target mixin Provider {}
%[Provider.value(10), Target.new().value(10), Provider.value(10)]
"#;
    let program = compile(source).expect("dual surface fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(11_u64.into()),
            Value::Integer(12_u64.into()),
            Value::Integer(13_u64.into()),
        ])))
    );
}

#[test]
fn full_signature_is_retained_when_module_main_binds_defaults_and_keywords() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |inv: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   if inv.defaulted.length() != 1 { raise :defaults }
   if inv.positional[:value] != 7 { raise :positional }
   if inv.keywords[:extra] != 3 { raise :keywords }
   next.call()
  })
 }
}
module Provider { @Wrap() public fun value(value: Integer = 7, key extra: Integer = 2) -> Integer { value + extra } }
Provider.value(extra: 3)
"#;
    let program = compile(source).expect("signature fixture compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(10_u64.into())));
}

#[test]
fn inherited_capability_is_enforced_when_component_denies_additions() {
    let source = format!(
        "{ADD} module Locked meta deny method_set {{}} @Add(1) module Provider mixin Locked {{}} 0"
    );
    let program = compile(&source).expect("inherited capability fixture compiles");
    let result = run(&program);
    assert!(
        matches!(result, Err(iris_vm::MachineError::Raised(ref error)) if error.0 == Value::Symbol("MetaCapabilityError".into()))
    );
}

#[test]
fn addition_collisions_abort_when_selector_is_already_declared() {
    let source = format!(
        "{ADD} @Add(1) module Provider {{ private fun added(delta: Integer) -> Integer {{ delta }} }} 0"
    );
    let program = compile(&source).expect("collision fixture compiles");
    let result = run(&program);
    assert_eq!(result, Err(iris_vm::MachineError::MetaTransactionError));
}
