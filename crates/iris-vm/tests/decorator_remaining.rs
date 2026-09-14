#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]
use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn block_patch_executes_with_exact_signature() {
    let source = include_str!("../../iris-cli/tests/decorator_protocol/parameter_patch.iris");
    let result = run(&compile(source).expect("fixture compiles"));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn callback_declaration_publishes_real_method() {
    let source = "class Target {} let result = Target.open() { |candidate|; public fun value() -> Integer { 42 } }; Target.new().value()";
    assert_eq!(
        run(&compile(source).expect("callback compiles")),
        Ok(Value::Integer(42_u64.into()))
    );
}

#[test]
fn capability_failure_rolls_back_callback_siblings() {
    let source = include_str!("../../iris-cli/tests/decorator_protocol/capability_rollback.iris");
    let result = run(&compile(source).expect("fixture compiles"));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn positional_closure_is_not_swallowed_by_optional_block() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target {
 @Wrap()
 public fun value(callback: Closure<() -> Integer>, &block: Block<() -> Integer> = nil) -> Integer {
  if block == nil { callback.call() } else { 0 }
 }
}
Target.new().value({ || -> Integer; 42 })
"#;
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Integer(42_u64.into()))
    );
}

#[test]
fn wrong_block_result_annotation_is_rejected_before_wrapper() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; 99 })
 }
}

class Target { @Wrap() public fun value(&block: Block<() -> Integer>) -> Integer { block.call() } }
try { Target.new().value() { || -> Symbol; :wrong } } catch error { error is? TypeError }
"#;
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Bool(true))
    );
}

#[test]
fn identical_closures_keep_written_call_channels() {
    let source = "class Target { public fun value(callback: Object = nil, &block: Block<() -> Integer> = nil) -> Integer { if block == nil { 1 } else { 2 } } } let first = Target.new().value({ || -> Integer; 7 }); let second = Target.new().value() { || -> Integer; 7 }; first * 10 + second";
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Integer(12_u64.into()))
    );
}

#[test]
fn callback_wrapping_publishes_once_and_uses_open_context() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  if c.reason != :open { raise :wrong_reason }
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; (next.call() as Integer) + 10 })
 }
}
class Target {}
let previous = Target.active_revision
let opened = Target.open() { |candidate|; @Wrap() public fun value() -> Integer { 7 } }
%[Target.new().value(), Target.active_revision == previous + 1]
"#;
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(17_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn caught_inner_decorator_failure_aborts_outer_candidate() {
    let source = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target meta deny method_body { public fun value() -> Integer { 7 } }
let revision = Target.active_revision
let commit = Reflection::Class.revision(Target).fetch(:commit_id)
let opened = Target.open() { |candidate|;
 candidate.define_method(:staged, { || -> Integer; 99 })
 try { @Wrap() public fun wrapped() -> Integer { 9 } } catch error { error is? MetaCapabilityError }
}
%[Target.active_revision == revision, Reflection::Class.revision(Target).fetch(:commit_id) == commit, Target.method(:staged) == nil, Target.method(:wrapped) == nil, Target.new().value()]
"#;
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
            Value::Integer(7_u64.into())
        ])))
    );
}

#[test]
fn patched_block_checks_full_callable_before_inner_entry() {
    let source = r#"
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(block: { || -> Symbol; :wrong }))
  })
 }
}
class Target { @Patch() public fun value(&block: Block<() -> Integer> = nil) -> Integer { raise :entered } }
try { Target.new().value() } catch error { error is? TypeError }
"#;
    assert_eq!(
        run(&compile(source).expect("source compiles")),
        Ok(Value::Bool(true))
    );
}

#[test]
fn all_remaining_paths_run_on_one_mib_stack() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            block_patch_executes_with_exact_signature();
            capability_failure_rolls_back_callback_siblings();
            callback_wrapping_publishes_once_and_uses_open_context();
        })
        .expect("thread starts")
        .join()
        .expect("bounded stack completes");
}

#[test]
fn bound_method_patch_preserves_receiver_when_signature_matches() {
    let source = r#"
class Effects { public class property body: Integer = 0; public class property helper: Integer = 0 }
class Helper {
 property value: Integer = 42
 public fun answer() -> Integer { Effects.helper = Effects.helper + 1; @value }
}
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(block: Helper.new().answer))
  })
 }
}
class Target {
 @Patch() public fun value(&block: Block<() -> Integer> = nil) -> Integer {
  Effects.body = Effects.body + 1
  block.call()
 }
}
let result = Target.new().value()
%[result, Effects.body, Effects.helper]
"#;
    let result = run(&compile(source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(42_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(1_u64.into())
        ])))
    );
}

#[test]
fn bound_method_patch_rejects_incompatible_return_before_body_entry() {
    let source = r#"
class Effects { public class property body: Integer = 0; public class property helper: Integer = 0 }
class Helper { public fun answer() -> Object { Effects.helper = Effects.helper + 1; 42 } }
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(block: Helper.new().answer))
  })
 }
}
class Target {
 @Patch() public fun value(&block: Block<() -> Integer> = nil) -> Integer {
  Effects.body = Effects.body + 1
  block.call()
 }
}
let result = try { Target.new().value() } catch error { error is? TypeError }
%[result, Effects.body, Effects.helper]
"#;
    let result = run(&compile(source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into()),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn callback_planner_runs_before_runtime_statements() {
    let source = r#"
class Fail {}
impl Fail for MethodDecorator {
 public fun plan(d, a) -> Plan { raise :planned }
 public fun transform(d, a, c) -> Transformation { Transformation.empty }
}
class Target {}
raise :runtime_started
Target.open() { |candidate|; @Fail() public fun value() -> Integer { 7 } }
"#;
    let result = run(&compile(source).expect("source compiles"));
    assert!(
        matches!(result, Err(iris_vm::MachineError::Raised(propagation)) if propagation.0 == Value::Symbol("planned".into()))
    );
}
