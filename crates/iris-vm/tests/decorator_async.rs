#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{Machine, compile, run};

fn source(wrapper: &str, body: &str, entry: &str) -> String {
    format!(
        r#"
class State {{ public class property gate: Object = nil; public class property bridge: Object = nil; public class property next: Object = nil; public class property context: Object = nil }}
class Wrap {{}}
impl Wrap for MethodDecorator {{
 public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
 public fun transform(declaration, arguments, context) -> Transformation {{
  Transformation.wrap_method({{ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object; {wrapper} }})
 }}
}}
class Target {{ @Wrap() public async fun value() -> Integer {{ {body} }} }}
module Scenario {{ public fun run() {{ {entry} }} }}
Scenario.run()
"#
    )
}

#[test]
fn pending_failure_reaches_catch_with_original_context() {
    let source = source(
        "try { await next.call() } catch error, context { State.context = context; raise }",
        "await State.gate; raise :inner_failed",
        "State.gate = Gate.new(); let task = Target.new().value(); Gate.complete(State.gate, 1); try { Host.run(task) } catch error, context { %[error, context same? State.context] }",
    );
    let result = run(&compile(&source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("inner_failed".into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn abandoned_bridge_failure_remains_reportable_after_owner_fails() {
    let source = source(
        "State.next = next; State.bridge = next.call(); raise :owner_failed",
        "await State.gate; raise :inner_failed",
        "State.gate = Gate.new(); let task = Target.new().value(); try { Host.run(task) } catch error { nil }; Gate.complete(State.gate, 1); let ready = { async || -> Object; :ready }; Host.run(ready.call()); let reports = Diagnostics.unobserved_failures(); let report = reports[0]; %[reports.length(), report[1] same? State.bridge, report[2]]",
    );
    let mut machine = Machine::new().expect("machine initializes");
    let result = machine.execute(&compile(&source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Bool(true),
            Value::Symbol("inner_failed".into())
        ])))
    );
    assert_eq!(machine.decorator_lifetime_diagnostics().len(), 1);
    let Value::Tuple(diagnostic) = &machine.decorator_lifetime_diagnostics()[0] else {
        unreachable!("structured lifetime diagnostic")
    };
    assert_eq!(
        &diagnostic[..4],
        &[
            Value::Symbol("IRIS-DECORATOR-PROTOCOL".into()),
            Value::Symbol("error".into()),
            Value::Symbol("runtime".into()),
            Value::Symbol("unfinished_inner".into()),
        ]
    );
    assert!(
        matches!(&diagnostic[7], Value::ExceptionContext(_, value, _, _, _, _) if value.as_ref() == &Value::Symbol("owner_failed".into()))
    );
}

#[test]
fn inner_result_is_checked_before_outer_wrapper_can_observe_success() {
    let source = source(
        "try { await next.call() } catch error { if error is? TypeError { 9 } else { 0 } }",
        ":wrong",
        "Host.run(Target.new().value())",
    );
    let result = run(&compile(&source).expect("source compiles"));
    assert_eq!(result, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn task_annotations_reject_covariant_bridge_and_outer_casts() {
    let source = source(
        "State.bridge = next.call(); await State.bridge",
        "7",
        "let outer = Target.new().value(); let bridge = State.bridge; let first = try { let wrong: Task<Object> = outer; false } catch error { true }; let second = try { let wrong: Task<Integer> = bridge; false } catch error { true }; %[first, second]",
    );
    let result = run(&compile(&source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn task_type_tests_and_casts_preserve_exact_adapter_results() {
    let source = source(
        "State.bridge = next.call(); await State.bridge",
        "7",
        "let outer = Target.new().value(); let bridge = State.bridge; %[outer is? Task<Integer>, outer is? Task<Object>, bridge is? Task<Object>, bridge is? Task<Integer>, (outer as Task<Integer>) same? outer]",
    );
    let result = run(&compile(&source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn ordinary_async_task_has_exact_result_metadata() {
    let source = "module M { public async fun value() -> Integer { 7 } } let task: Task<Integer> = M.value(); Host.run(task)";
    let result = run(&compile(source).expect("source compiles"));
    assert_eq!(result, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn async_wrappers_complete_on_one_mib_stack() {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let source = source("await next.call()", "7", "Host.run(Target.new().value())");
            assert_eq!(
                run(&compile(&source).expect("source compiles")),
                Ok(Value::Integer(7_u64.into()))
            );
        })
        .expect("thread starts")
        .join()
        .expect("bounded stack completes");
}

#[test]
fn all_published_async_fixtures_execute_in_vm() {
    for source in [
        include_str!("../../iris-cli/tests/decorator_async/eager_ready.iris"),
        include_str!("../../iris-cli/tests/decorator_async/pending_owner.iris"),
        include_str!("../../iris-cli/tests/decorator_async/bridge_identity.iris"),
        include_str!("../../iris-cli/tests/decorator_async/invalid_initial.iris"),
        include_str!("../../iris-cli/tests/decorator_async/bad_patch.iris"),
        include_str!("../../iris-cli/tests/decorator_async/expired.iris"),
        include_str!("../../iris-cli/tests/decorator_async/foreign_eager.iris"),
        include_str!("../../iris-cli/tests/decorator_async/overlap.iris"),
        include_str!("../../iris-cli/tests/decorator_async/unfinished_normal.iris"),
        include_str!("../../iris-cli/tests/decorator_async/zero_attempt_result.iris"),
    ] {
        let program = compile(source).expect("fixture compiles");
        assert!(run(&program).is_ok());
    }
}

#[test]
fn pending_using_survives_a_second_task_suspension() {
    let source = "mut log = %[]; class Resource { public fun close() { log.append(:closed) } } module M { public async fun child(gate) { await gate; 7 } public async fun outer(first, second) { using(Resource.new()) { |resource| await first; await M.child(second) }; :done } public fun run() { let first = Gate.new(); let second = Gate.new(); let task = M.outer(first, second); Gate.complete(first, 1); let ready = { async || -> Object; :ready }; Host.run(ready.call()); Gate.complete(second, 2); %[Host.run(task), log.length()] } } M.run()";
    let result = run(&compile(source).expect("source compiles"));
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("done".into()),
            Value::Integer(1_u64.into())
        ])))
    );
}
