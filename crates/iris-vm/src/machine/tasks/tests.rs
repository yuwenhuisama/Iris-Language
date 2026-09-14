#![expect(clippy::expect_used, reason = "tests require initialized machines")]

use super::*;

#[test]
fn parent_identity_is_restored_when_nested_task_returns_fails_or_suspends() {
    for outcome in [
        Ok(Value::Nil),
        Err(MachineError::ArgumentError),
        Err(MachineError::Suspended(ObjectId::new(3))),
    ] {
        let mut machine = Machine::new().expect("machine initializes");
        let parent = ObjectId::new(1);
        let child = ObjectId::new(2);
        let expected = outcome.clone();
        let result = machine.with_task(parent, |machine| {
            let result = machine.with_task(child, |machine| {
                assert_eq!(machine.current_task, Some(TaskId::new(child)));
                outcome
            });
            assert_eq!(machine.current_task, Some(TaskId::new(parent)));
            result
        });
        assert_eq!(result, expected);
        assert_eq!(machine.current_task, None);
        assert_eq!(machine.async_depth, 0);
    }
}

#[test]
fn identity_is_allocated_before_nested_eager_task_executes() {
    let program = crate::compile("module M { public async fun child() { 7 } public async fun parent() { M.child() } } M.parent()").expect("source compiles");
    let mut machine = Machine::new().expect("machine initializes");
    let result = machine.execute(&program).expect("parent starts");
    let parent = ObjectId::new(900_000);
    assert_eq!(result, Value::Task(parent));
    let child = machine.tasks.get(&parent).expect("parent settled");
    assert_eq!(child, &Ok(Value::Task(ObjectId::new(900_001))));
    assert_eq!(machine.current_task, None);
}

#[test]
fn resuming_task_restores_observer_when_body_fails() {
    let program = crate::compile(
        r#"
let gate = Gate.new()
let callback = { async || -> Object; await gate; raise :failed }
let task = callback.call()
task
"#,
    )
    .expect("source compiles");
    let mut machine = Machine::new().expect("machine initializes");
    let result = machine.execute(&program).expect("task suspends");
    let identity = ObjectId::new(900_001);
    assert_eq!(result, Value::Task(identity));
    assert_eq!(machine.current_task, None);
    let gate = machine.suspended[0].frame.gate;
    machine.gates.insert(gate, Some(Value::Nil));
    let observer = ObjectId::new(42);
    machine.current_task = Some(TaskId::new(observer));
    let outcome = machine.observe_task(result, &program, &[]);
    assert!(matches!(outcome, Err(MachineError::Raised(_))));
    assert_eq!(machine.current_task, Some(TaskId::new(observer)));
    assert_eq!(machine.async_depth, 0);
}

#[test]
fn wrapper_scope_records_resumed_task_as_owner() {
    let program = crate::compile(r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target { @Wrap() public fun value() -> Integer { 7 } }
let gate = Gate.new()
let callback = { async || -> Object; await gate; Target.new().value() }
let task = callback.call()
let posted = Gate.complete(gate, 1)
let observed = Host.run(task)
task
"#).expect("source compiles");
    let mut machine = Machine::new().expect("machine initializes");
    let task = machine.execute(&program).expect("task resumes");
    let mut owners = Vec::new();
    for next in machine.next_continuations.values() {
        next.visit_values(&mut |value| {
            if matches!(value, Value::Task(_)) {
                owners.push(value.clone());
            }
        });
    }
    assert_eq!(owners, vec![task]);
    assert_eq!(machine.current_task, None);
}

#[test]
fn async_adapters_allocate_distinct_typed_tasks_and_callback_owner() {
    let program = crate::compile(r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object; await next.call() })
 }
}
class Target { @Wrap() public async fun value() -> Integer { 7 } }
Target.new().value()
"#).expect("source compiles");
    let mut machine = Machine::new().expect("machine initializes");
    let outer = machine.execute(&program).expect("call succeeds");
    assert_eq!(machine.tasks.len(), 4);
    assert!(
        machine
            .tasks
            .values()
            .all(|outcome| outcome == &Ok(Value::Integer(7_u64.into())))
    );
    let integer = machine.builtin_class("Integer").expect("integer type");
    let object = machine.builtin_class("Object").expect("object type");
    assert_eq!(
        machine
            .task_types
            .values()
            .filter(|value| **value == Value::Type(integer, vec![]))
            .count(),
        2
    );
    assert_eq!(
        machine
            .task_types
            .values()
            .filter(|value| **value == Value::Type(object, vec![]))
            .count(),
        2
    );
    let next = machine
        .next_continuations
        .values()
        .next()
        .expect("wrapper owns next");
    let owner = next.scope.owner().task().expect("async owner").object_id();
    assert_ne!(outer, Value::Task(owner));
    assert_eq!(
        machine.task_types.get(&owner),
        Some(&Value::Type(object, vec![]))
    );
    assert_eq!(machine.current_task, None);
}
