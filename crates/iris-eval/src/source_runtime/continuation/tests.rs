use super::*;

#[test]
fn task_identity_precedes_eager_body_and_restores_caller_when_suspended_or_failed() {
    let mut evaluator = SourceEvaluator::new_in_package("test").unwrap();
    let caller = ObjectId::new(42);
    evaluator.current_task = Some(caller);
    let parsed = iris_parser::parse(
        "module Work { public async fun run() -> Object { Gate.new() }; public async fun fail() -> Object { raise :failure }; public async fun wait(gate) -> Object { await gate } }; let task = Work.run(); task",
    );
    let Value::Task(identity) = evaluator.program(&parsed.program).unwrap() else {
        panic!("Task expected")
    };
    let Value::Gate(gate) = evaluator.tasks[&identity].as_ref().unwrap() else {
        panic!("Gate expected")
    };
    assert!(identity < *gate);
    assert_eq!(evaluator.current_task, Some(caller));
    let parsed = iris_parser::parse("let failed = Work.fail(); failed");
    assert!(matches!(
        evaluator.program(&parsed.program),
        Ok(Value::Task(_))
    ));
    assert_eq!(evaluator.current_task, Some(caller));
    let parsed =
        iris_parser::parse("let gate = Gate.new(); let waiting = Work.wait(gate); waiting");
    let Value::Task(waiting) = evaluator.program(&parsed.program).unwrap() else {
        panic!("Task expected")
    };
    assert!(evaluator.suspended.contains_key(&waiting));
    assert_eq!(evaluator.current_task, Some(caller));
    let parsed = iris_parser::parse("let posted = Gate.complete(gate, 9); nil");
    evaluator.program(&parsed.program).unwrap();
    evaluator.drive_ready_continuations().unwrap();
    assert_eq!(evaluator.tasks[&waiting], Ok(Value::Integer(9u64.into())));
    assert_eq!(evaluator.current_task, Some(caller));
}
