#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

fn source(wrapper: &str, body: &str, entry: &str) -> String {
    format!(
        r#"
class State {{
 public class property gate: Object = nil
 public class property count: Integer = 0
 public class property defaults: Integer = 0
 public class property context: Object = nil
 public class fun default_value() -> Integer {{ State.defaults = State.defaults + 1; 7 }}
}}
class Other {{ public class fun echo<Element>(value: Element) -> Element {{ value }} }}
class Wrap {{}}
impl Wrap for MethodDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{
  Transformation.wrap_method({{ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
   if invocation.method_type_arguments.length() != 1 {{ raise :arity }}
   if invocation.signature.parameters[0].type != invocation.method_type_arguments[0] {{ raise :parameter }}
   if invocation.signature.result != invocation.method_type_arguments[0] {{ raise :result }}
   {wrapper}
  }})
 }}
}}
class Target {{ @Wrap() public async class fun echo<Element>(value: Element = State.default_value()) -> Element {{ {body} }} }}
module Scenario {{ public fun run() {{ {entry} }} }}
Scenario.run()
"#
    )
}

#[test]
fn closed_task_is_exact_when_ready_or_pending() {
    for body in [
        "value",
        "let before: Element = value; await State.gate; let nested = Other.echo<String>(\"nested\"); let after: Element = before; after",
    ] {
        let given = source(
            "await next.call()",
            body,
            r#"
State.gate = Gate.new()
let task: Task<Integer> = Target.echo<Integer>(7)
let before = task is? Task<Integer> && !(task is? Task<Object>)
Gate.complete(State.gate, 1)
let result = Host.run(task)
%[before, result, task is? Task<Integer>, task is? Task<Object>]
"#,
        );
        let when = run(&compile(&given).expect("compile"));
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Integer(7_u64.into()),
                Value::Bool(true),
                Value::Bool(false)
            ])))
        );
    }
}

#[test]
fn closed_bindings_are_isolated_when_tasks_interleave() {
    let given = source(
        "await next.call()",
        "let before: Element = value; await State.gate; let after: Element = before; after",
        r#"
let first = Gate.new(); let second = Gate.new()
State.gate = first; let integer: Task<Integer> = Target.echo<Integer>(7)
State.gate = second; let text: Task<String> = Target.echo<String>("hello")
Gate.complete(second, 1); let text_result = Host.run(text)
Gate.complete(first, 1); let integer_result = Host.run(integer)
%[integer_result, text_result, integer is? Task<Integer>, text is? Task<String>]
"#,
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Text("hello".into()),
            Value::Bool(true),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn bad_patch_is_rejected_before_body_when_wrapper_resumes() {
    let given = source(
        "await State.gate; await next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))",
        "State.count = State.count + 1; value",
        r#"
State.gate = Gate.new(); let task = Target.echo<Integer>(7)
Gate.complete(State.gate, 1)
let rejected = try { Host.run(task); false } catch error { error is? TypeError }
%[rejected, State.count]
"#,
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn failed_task_keeps_context_when_return_contract_fails() {
    for (wrapper, body) in [
        ("await next.call()", "await State.gate; \"wrong\""),
        ("await State.gate; \"wrong\"", "value"),
    ] {
        let given = source(
            wrapper,
            body,
            r#"
State.gate = Gate.new(); let task: Task<Integer> = Target.echo<Integer>(7)
Gate.complete(State.gate, 1)
let first = try { Host.run(task); false } catch error, context { State.context = context; error is? TypeError }
let second = try { Host.run(task); false } catch error, context { (error is? TypeError) && (context same? State.context) }
%[first, second, task is? Task<Integer>]
"#,
        );
        let when = run(&compile(&given).expect("compile"));
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![Value::Bool(true); 3])))
        );
    }
}

#[test]
fn defaults_execute_once_when_next_retries_after_suspension() {
    let given = source(
        "await next.call(); await next.call()",
        "await State.gate; State.count = State.count + 1; let typed: Element = value; typed",
        r#"
State.gate = Gate.new(); let task = Target.echo<Integer>()
Gate.complete(State.gate, 1); let result = Host.run(task)
%[result, State.defaults, State.count]
"#,
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into())
        ])))
    );
}

#[test]
fn wrapper_uses_lexical_bindings_when_target_uses_same_type_name() {
    let given = r#"
class State { public class property gate: Object = nil }
class Factory {
 public class fun wrapper<Element>(value: Element) {
  { async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
   let before: Element = value
   await State.gate
   let after: Element = before
   if after != "lexical" { raise :lexical }
   await next.call()
  }
 }
}
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation { Transformation.wrap_method(Factory.wrapper<String>("lexical")) }
}
class Target { @Wrap() public async class fun echo<Element>(value: Element) -> Element { let result: Element = value; result } }
module Scenario { public fun run() {
 State.gate = Gate.new(); let task = Target.echo<Integer>(7)
 Gate.complete(State.gate, 1); Host.run(task)
} }
Scenario.run()
"#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn nested_closure_keeps_bindings_when_using_resumes() {
    let given = source(
        "await next.call()",
        "using(Resource.new()) { |resource| let before: Element = value; await State.gate; let after: Element = before }; let result: Element = value; result",
        "State.gate = Gate.new(); let task = Target.echo<Integer>(7); Gate.complete(State.gate, 1); Host.run(task)",
    );
    let given = format!("class Resource {{ public fun close() {{ nil }} }} {given}");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn inner_layer_rejects_return_before_outer_observes_success() {
    let given = source(
        "try { await next.call() } catch error { if error is? TypeError { 9 } else { 0 } }",
        "await State.gate; \"wrong\"",
        "State.gate = Gate.new(); let task = Target.echo<Integer>(7); Gate.complete(State.gate, 1); Host.run(task)",
    );
    let given = given.replace("@Wrap() public async", "@Wrap() @Wrap() public async");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn captures_are_per_closed_method_when_materialization_repeats() {
    let given = source("calls = calls + 1; await State.gate; if invocation.method_type_arguments[0] == Integer.type { calls } else { await next.call() }", "value", r#"
State.gate = Gate.new()
let first = Target.echo<Integer>(0); let text = Target.echo<String>("hello")
Gate.complete(State.gate, 1)
let first_result = Host.run(first); let text_result = Host.run(text)
%[first_result, text_result, Host.run(Target.echo<Integer>(0)), State.count]
"#).replace("Transformation.wrap_method({", "State.count = State.count + 1; mut calls = 0; Transformation.wrap_method({");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Text("hello".into()),
            Value::Integer(2_u64.into()),
            Value::Integer(3_u64.into())
        ])))
    );
}

#[test]
fn returned_async_closure_checks_closed_result_when_resumed() {
    let given = r#"
class Factory { public class fun make<Element>(value: Element, gate) {
 { async || -> Element; await gate; let after: Element = value; after }
} }
module Scenario { public fun run() {
 let gate = Gate.new(); let closure = Factory.make<Integer>(7, gate)
 let task: Task<Integer> = closure.call(); Gate.complete(gate, 1); Host.run(task)
} }
Scenario.run()
"#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}
