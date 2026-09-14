#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

const DECORATOR: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  let digit = arguments[0] as Integer
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   (next.call() as Integer) * 10 + digit + calls
  })
 }
}
"#;

#[test]
fn fresh_ordered_chain_keeps_old_binding_when_identical_reopen_runs_twice() {
    let given = format!(
        r#"{DECORATOR}
class Target {{ @Wrap(1) @Wrap(2) public fun value() -> Integer {{ 7 }} }}
let target = Target.new()
let old = target.value
let first = old.call()
open class Target {{ @Wrap(1) @Wrap(2) public override fun value() -> Integer {{ 8 }} }}
let middle = target.value
let second = middle.call()
open class Target {{ @Wrap(1) @Wrap(2) public override fun value() -> Integer {{ 8 }} }}
let third = target.value()
first == 732 && second == 832 && third == 832 && old.call() == 743 && middle.call() == 843
"#
    );
    let program = compile(&given).expect("compile decorated reopens");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn closed_chain_is_fresh_when_generic_canonical_method_is_reopened() {
    let given = format!(
        r#"{DECORATOR}
class Target {{ @Wrap(1) public class fun echo<Element>(value: Element) -> Element {{ value }} }}
let first = Target.echo<Integer>(7)
let second = Target.echo<Integer>(7)
open class Target {{ @Wrap(2) public override class fun echo<Element>(value: Element) -> Element {{ value }} }}
let third = Target.echo<Integer>(7)
first == 72 && second == 73 && third == 73 && Target.echo<Integer>(7) == 74
"#
    );
    let program = compile(&given).expect("compile generic reopen");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn reopen_plan_runs_before_effects_when_plan_is_invalid() {
    let given = r#"
class Wrong {}
impl Wrong for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { print(:forbidden); Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }
}
class Target { public fun value() -> Integer { 7 } }
raise :runtime_started
open class Target { @Wrong() public override fun value() -> Integer { 8 } }
"#;
    let program = compile(given).expect("compile reopen plan");
    let when = run(&program);
    assert_eq!(
        when,
        Err(iris_vm::MachineError::LexicalDiagnostic(
            "IRIS-DECORATOR-NONDETERMINISTIC"
        ))
    );
}

#[test]
fn exported_reopen_uses_its_own_chain_when_source_exports_declarations() {
    let given = format!(
        r#"{DECORATOR}
export class Target {{ @Wrap(1) public fun value() -> Integer {{ 7 }} }}
let target = Target.new()
let old = target.value
export open class Target {{ @Wrap(2) public override fun value() -> Integer {{ 8 }} }}
target.value() == 83 && old.call() == 72
"#
    );
    let program = compile(&given).expect("compile exported reopen");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}
