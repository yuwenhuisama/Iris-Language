#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

fn wrapped(body: &str, entry: &str) -> String {
    format!(
        r#"
class State {{
 public class property calls: Integer = 0
 public class property saved: Object = nil
 public class property task: Object = nil
 public class property gate: Object = nil
}}
module Helpers {{
 public fun sync(next) {{ next.call() }}
 public async fun foreign(next) {{ next.call() }}
 public async fun park(gate) {{ await gate; :ready }}
 public async fun owner() {{ Target.new().value() }}
}}
class Wrap {{}}
impl Wrap for MethodDecorator {{
 public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
 public fun transform(declaration, arguments, context) -> Transformation {{
  Transformation.wrap_method({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   {body}
  }})
 }}
}}
class Target {{
 @Wrap() public fun value() -> Object {{ State.calls = State.calls + 1; :inner }}
}}
module Scenario {{ public fun run() {{ {entry} }} }}
Scenario.run()
"#
    )
}

fn evaluate(source: &str) -> Result<Value, iris_vm::MachineError> {
    let program = compile(source).expect("source compiles to bytecode");
    run(&program)
}

#[test]
fn next_runs_when_helper_is_synchronous() {
    let source = wrapped("Helpers.sync(next)", "Target.new().value(); State.calls");
    let result = evaluate(&source);
    assert_eq!(result, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn next_refuses_foreign_task_when_async_function_runs_eagerly() {
    let source = wrapped(
        "State.task = Helpers.foreign(next); :outer",
        "Target.new().value(); try { Host.run(State.task) } catch error { %[error.category, State.calls] }",
    );
    let result = evaluate(&source);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("foreign_task".into()),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn next_refuses_foreign_task_when_async_closure_runs_eagerly() {
    let source = wrapped(
        "let helper = { async || -> Object; next.call() }; State.task = helper.call(); :outer",
        "Target.new().value(); try { Host.run(State.task) } catch error { %[error.category, State.calls] }",
    );
    let result = evaluate(&source);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("foreign_task".into()),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn owner_next_remains_valid_when_foreign_helper_fails() {
    let source = wrapped(
        "State.task = Helpers.foreign(next); next.call()",
        "let owner = Helpers.owner(); Host.run(owner); let reports = Diagnostics.unobserved_failures(); let report = reports[0]; let error = report[2]; %[State.calls, error.category]",
    );
    let result = evaluate(&source);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Symbol("foreign_task".into())
        ])))
    );
}

#[test]
fn expiry_precedes_foreign_identity_when_saved_next_is_called_later() {
    let source = wrapped(
        "State.saved = next; :outer",
        "Target.new().value(); let task = Helpers.foreign(State.saved); try { Host.run(task) } catch error { error.category }",
    );
    let result = evaluate(&source);
    assert_eq!(result, Ok(Value::Symbol("expired".into())));
}

#[test]
fn owner_next_remains_valid_when_child_suspends() {
    let source = wrapped(
        "State.task = Helpers.park(State.gate); next.call()",
        "State.gate = Gate.new(); let owner = Helpers.owner(); Host.run(owner); Gate.complete(State.gate, 1); Host.run(State.task); State.calls",
    );
    let result = evaluate(&source);
    assert_eq!(result, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn closure_answers_task_when_body_completes_eagerly() {
    let source = "let callback = { async || -> Integer; 7 }; Host.run(callback.call())";
    let result = evaluate(source);
    assert_eq!(result, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn closure_resumes_when_captured_gate_completes() {
    let source = "let gate = Gate.new(); let callback = { async || -> Integer; let posted = await gate; (posted as Integer) + 1 }; let task = callback.call(); let posted = Gate.complete(gate, 7); Host.run(task)";
    let result = evaluate(source);
    assert_eq!(result, Ok(Value::Integer(8_u64.into())));
}

#[test]
fn unobserved_failure_is_removed_when_foreign_task_is_observed() {
    let source = wrapped(
        "State.task = Helpers.foreign(next); next.call()",
        "Target.new().value(); try { Host.run(State.task) } catch error { error.category }; Diagnostics.unobserved_failures().length()",
    );
    let result = evaluate(&source);
    assert_eq!(result, Ok(Value::Integer(0_u64.into())));
}

#[test]
fn resumed_owner_can_call_next_after_foreign_child_fails() {
    let source = wrapped(
        "State.task = Helpers.foreign(next); next.call()",
        "State.gate = Gate.new(); let owner = { async || -> Object; await State.gate; Target.new().value() }; let task = owner.call(); Gate.complete(State.gate, 1); Host.run(task); let reports = Diagnostics.unobserved_failures(); let report = reports[0]; let error = report[2]; %[State.calls, error.category]",
    );
    let result = evaluate(&source);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Symbol("foreign_task".into())
        ])))
    );
}

#[test]
fn closure_return_type_failure_is_recorded_when_task_resumes() {
    let source = "let gate = Gate.new(); let callback = { async || -> Integer; await gate; :wrong }; let task = callback.call(); let posted = Gate.complete(gate, 7); try { Host.run(task) } catch error { error is? TypeError }";
    let result = evaluate(source);
    assert_eq!(result, Ok(Value::Bool(true)));
}
