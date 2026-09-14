#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

fn source(wrapper: &str, body: &str, entry: &str) -> String {
    format!(
        r#"
class State {{ public class property gate: Object = nil; public class property bodies: Integer = 0 }}
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
class Target {{ @Wrap() public async fun echo<Element>(value: Element) -> Element {{ {body} }} }}
module Scenario {{ public fun run() {{ let target = Target.new(); {entry} }} }}
Scenario.run()
"#
    )
}

#[test]
fn closed_tasks_keep_bindings_when_two_types_suspend_and_interleave() {
    let given = source(
        "await next.call()",
        "let before: Element = value; await State.gate; Other.echo<String>(\"nested\"); let after: Element = before; after",
        r#"
let first = Gate.new(); let second = Gate.new()
State.gate = first; let integer: Task<Integer> = target.echo<Integer>(7)
State.gate = second; let text: Task<String> = target.echo("hello")
Gate.complete(second, 1); let text_result = Host.run(text)
Gate.complete(first, 1); let integer_result = Host.run(integer)
%[integer_result, text_result, integer is? Task<Integer>, text is? Task<String>, integer is? Task<Object>, text is? Task<Object>]
"#,
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Text("hello".into()),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(false)
        ])))
    );
}

#[test]
fn patch_is_rejected_before_inner_body_when_wrapper_resumes() {
    let given = source(
        "await State.gate; await next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))",
        "State.bodies = State.bodies + 1; value",
        r#"
State.gate = Gate.new(); let task: Task<Integer> = target.echo<Integer>(7)
Gate.complete(State.gate, 1)
let rejected = try { Host.run(task); false } catch error { error is? TypeError }
%[rejected, State.bodies, task is? Task<Integer>]
"#,
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn closed_return_is_rejected_when_body_or_wrapper_resumes_with_wrong_type() {
    for (wrapper, body) in [
        ("await next.call()", "await State.gate; \"wrong\""),
        ("await State.gate; \"wrong\"", "value"),
    ] {
        let given = source(
            wrapper,
            body,
            r#"
State.gate = Gate.new(); let task: Task<Integer> = target.echo<Integer>(7)
Gate.complete(State.gate, 1)
try { Host.run(task); false } catch error { error is? TypeError }
"#,
        );
        let when = run(&compile(&given).expect("compile"));
        assert_eq!(when, Ok(Value::Bool(true)));
    }
}

#[test]
fn lexical_wrapper_bindings_are_preserved_when_target_closes_same_parameter_name() {
    let given = source(
        "await next.call()",
        "let result: Element = value; result",
        "State.gate = Gate.new(); let task = target.echo<Integer>(7); Gate.complete(State.gate, 1); Host.run(task)",
    );
    let start = given
        .find("  Transformation.wrap_method(")
        .expect("wrapper start");
    let end = start + given[start..].find("\n }\n}").expect("transform end");
    let given = format!(
        r#"
class Factory {{ public class fun wrapper<Element>(value: Element) {{
 {{ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
  let before: Element = value; await State.gate; let after: Element = before
  if after != "lexical" {{ raise :lexical }}
  await next.call()
 }}
}} }}
{} Transformation.wrap_method(Factory.wrapper<String>("lexical")){}
"#,
        &given[..start],
        &given[end..]
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}
