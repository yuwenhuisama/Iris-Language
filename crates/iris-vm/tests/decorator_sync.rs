#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn first_written_wrapper_is_outermost() {
    let source = r#"
class A {}
impl A for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   (next.call() as Integer) * 10 + 1
  })
 }
}
class B {}
impl B for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   (next.call() as Integer) * 10 + 2
  })
 }
}
class Target {
 @A()
 @B()
 public fun value() -> Integer { 7 }
}
Target.new().value()
"#;
    let program = compile(source).expect("decorators compile to bytecode");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(721_u64.into())));
}

fn evaluate(source: &str) -> Result<Value, iris_vm::MachineError> {
    run(&compile(source).expect("source compiles to bytecode"))
}

#[test]
fn defaults_run_once_when_resolved_call_omits_parameter() {
    let source = "class Effects { public class property count: Integer = 0; public class fun bump() -> Integer { Effects.count = Effects.count + 1; 7 } } module M { public fun value(value: Integer = Effects.bump()) -> Integer { value } } let result = M.value(); Effects.count";
    assert_eq!(evaluate(source), Ok(Value::Integer(1_u64.into())));
}

#[test]
fn explicit_nil_does_not_evaluate_default() {
    let source = "module M { public fun default_value() { raise :default_ran } public fun value(value: Object = M.default_value()) -> Object { value } } M.value(nil)";
    assert_eq!(evaluate(source), Ok(Value::Nil));
}

#[test]
fn class_phase_installs_bytecode_closure_with_captures() {
    let source = r#"
class Add {}
impl Add for ClassDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  let held = arguments[0]
  Transformation.add_method(:added, { || -> Integer; held as Integer })
 }
}
@Add(41 + 1)
class Target { public fun value() -> Integer { self.added() } }
Target.new().value()
"#;
    assert_eq!(evaluate(source), Ok(Value::Integer(42_u64.into())));
}

#[test]
fn next_expires_when_wrapper_returns() {
    let source = r#"
class Saved { public class property next: Object = nil }
class Keep {}
impl Keep for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   Saved.next = next
   7
  })
 }
}
class Target { @Keep() public fun value() -> Integer { 1 } }
let result = Target.new().value()
try { Saved.next.call() } catch error { error.category }
"#;
    assert_eq!(evaluate(source), Ok(Value::Symbol("expired".into())));
}

#[test]
fn wrapper_order_runs_on_one_mib_stack() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(first_written_wrapper_is_outermost)
        .expect("thread starts")
        .join()
        .expect("wrapper completes on bounded stack");
}

#[test]
fn patches_keep_positional_and_keyword_channels_separate() {
    let source = r#"
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(positional: %{:left: 10}, keywords: %{:right: 20}))
  })
 }
}
class Target { @Patch() public fun value(left: Integer, key right: Integer) -> Integer { left + right } }
Target.new().value(1, right: 2)
"#;
    assert_eq!(evaluate(source), Ok(Value::Integer(30_u64.into())));
}

#[test]
fn declaration_metadata_is_readonly_and_descriptive() {
    let source = r#"
class Inspect {}
impl Inspect for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan {
  if declaration.selector != :value { raise :wrong_selector }
  Plan.empty
 }
 public fun transform(declaration, arguments, context) -> Transformation {
  if declaration.visibility != :public { raise :wrong_visibility }
  Transformation.empty
 }
}
class Target { @Inspect() public fun value() -> Integer { 7 } }
Target.new().value()
"#;
    assert_eq!(evaluate(source), Ok(Value::Integer(7_u64.into())));
}

#[test]
fn inner_result_is_checked_before_outer_wrapper_observes_it() {
    let source = r#"
class Recover {}
impl Recover for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
    try { next.call(); 0 } catch error { if error is? TypeError { 9 } else { 0 } }
  })
 }
}
class Wrong {}
impl Wrong for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; :wrong })
 }
}
class Target { @Recover() @Wrong() public fun value() -> Integer { 7 } }
Target.new().value()
"#;
    assert_eq!(evaluate(source), Ok(Value::Integer(9_u64.into())));
}

#[test]
fn retained_bound_method_keeps_published_chain() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; (next.call() as Integer) + 10 })
 }
}
class Target { @Wrap() public fun value() -> Integer { 7 } }
let held = Target.new().value
held.call()
"#;
    assert_eq!(evaluate(source), Ok(Value::Integer(17_u64.into())));
}
