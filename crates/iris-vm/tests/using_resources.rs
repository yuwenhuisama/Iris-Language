#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

#[test]
fn using_passes_exact_resource_when_block_declares_parameter() {
    let source = "class Resource { public fun close() { nil } } let resource = Resource.new(); let returned = using(resource) { |held| held }; returned == resource";
    let program = compile(source).expect("source compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Bool(true)));
}

#[test]
fn using_closes_once_after_resumption_when_block_accepts_resource() {
    let source = r#"
mut log = %[]
class Resource { public fun close() { log.append(:closed) } }
class Worker {
 public async fun run(gate) {
  let resource = Resource.new()
  using(resource) { |held|
   if held != resource { raise :wrong_resource }
   log.append(:entered)
   await gate
   log.append(:resumed)
  }
  log.append(:after_using)
  :done
 }
}
module Scenario {
 public fun run() {
  let gate = Gate.new()
  let task = Worker.new().run(gate)
  let before = log.length()
  Gate.complete(gate, 1)
  let result = Host.run(task)
   %[before, result, log]
 }
}
Scenario.run()
"#;
    let program = compile(source).expect("source compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Symbol("done".into()),
            Value::Array(ArrayRef::new(vec![
                Value::Symbol("entered".into()),
                Value::Symbol("resumed".into()),
                Value::Symbol("closed".into()),
                Value::Symbol("after_using".into()),
            ])),
        ])))
    );
}

#[test]
fn using_preserves_zero_parameter_block_with_captures() {
    let source = "class Resource { public fun close() { nil } } let answer = 7; using(Resource.new()) { answer }";
    let program = compile(source).expect("source compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn using_rejects_extra_required_parameter_without_weakening_closure_arity() {
    let source = "class Resource { public fun close() { nil } } try { using(Resource.new()) { |first, second| :body } } catch error { error.class == ArgumentError && error is? ArgumentError && !(error is? Symbol) }";
    let program = compile(source).expect("source compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Bool(true)));
}
