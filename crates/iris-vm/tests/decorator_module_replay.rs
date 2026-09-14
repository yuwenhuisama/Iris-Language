#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  if context.reason != arguments[1] { raise :reason }
  let digit = arguments[0] as Integer
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   if invocation.slot[0] != Provider { raise :owner }
   (next.call() as Integer) * 10 + digit + calls
  })
 }
}
"#;

#[test]
fn fresh_shared_chain_keeps_old_bindings_when_module_is_reopened_twice() {
    let given = format!(
        r#"{WRAP}
module Provider {{ @Wrap(1, :origin) @Wrap(2, :origin) public fun value() -> Integer {{ 7 }} }}
class Target mixin Provider {{}}
let target = Target.new()
let old = target.value
let reflected = Reflection::Class.method(Target, :value)
let first = Provider.value()
open module Provider {{ @Wrap(1, :open) @Wrap(2, :open) public override fun value() -> Integer {{ 8 }} }}
let middle = target.value
let second = Provider.value()
open module Provider {{ @Wrap(1, :open) @Wrap(2, :open) public override fun value() -> Integer {{ 8 }} }}
let third = target.value()
first == 732 && second == 832 && third == 832 && Provider.value() == 843 && old.call() == 743 && Reflection::Class.invoke(reflected, target, %[]) == 754 && Reflection::Module.invoke(reflected, Provider, %[]) == 765 && middle.call() == 843
"#
    );
    let program = compile(&given).expect("compile Module replays");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn plan_precedes_author_effects_when_module_reopen_plan_is_impure() {
    let given = r#"
class Wrong {}
impl Wrong for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { print(:forbidden); Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }
}
module Provider { public fun value() -> Integer { 7 } }
raise :runtime_started
open module Provider { @Wrong() public override fun value() -> Integer { 8 } }
"#;
    let program = compile(given).expect("compile Module plan");
    let when = run(&program);
    assert_eq!(
        when,
        Err(iris_vm::MachineError::LexicalDiagnostic(
            "IRIS-DECORATOR-NONDETERMINISTIC"
        ))
    );
}

#[test]
fn undecorated_replacement_clears_chain_when_decorated_module_is_reopened() {
    let given = format!(
        r#"{WRAP}
module Provider {{ @Wrap(1, :origin) public fun value() -> Integer {{ 7 }} }}
class Target mixin Provider {{}}
let old = Target.new().value
open module Provider {{ public override fun value() -> Integer {{ 8 }} }}
Provider.value() == 8 && old.call() == 72
"#
    );
    let program = compile(&given).expect("compile unwrapped replacement");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn frozen_header_is_rejected_when_decorated_module_open_changes_composition_or_policy() {
    for header in ["mixin Other", "meta deny method_body"] {
        let given = format!(
            r#"{WRAP}
module Other {{}}
module Provider {{ @Wrap(1, :origin) public fun value() -> Integer {{ 7 }} }}
open module Provider {header} {{ @Wrap(1, :open) public override fun value() -> Integer {{ 8 }} }}
0
"#
        );
        let program = compile(&given).expect("compile frozen header");
        let when = run(&program);
        assert_eq!(when, Err(iris_vm::MachineError::MetaTransactionError));
    }
}

#[test]
fn full_signature_is_validated_before_transform_when_module_replacement_is_incompatible() {
    let given = format!(
        r#"{WRAP}
module Provider {{ @Wrap(1, :origin) public fun value() -> Integer {{ 7 }} }}
open module Provider {{ @Wrap(1, :wrong) public override fun value() -> String {{ "bad" }} }}
0
"#
    );
    let program = compile(&given).expect("compile signature mismatch");
    let when = run(&program);
    assert_eq!(when, Err(iris_vm::MachineError::TypeContractError));
}

#[test]
fn module_decorator_receives_open_reason_when_replaying_declaration() {
    let given = r#"
class Inspect {}
impl Inspect for ModuleDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  if context.reason != arguments[0] { raise :wrong_reason }
  Transformation.empty
 }
}
@Inspect(:origin) module Provider { public fun value() -> Integer { 7 } }
@Inspect(:open) open module Provider { public override fun value() -> Integer { 8 } }
Provider.value()
"#;
    let program = compile(given).expect("compile Module decorator replay");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Integer(8_u64.into())));
}
