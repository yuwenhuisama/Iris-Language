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

#[test]
fn safe_navigation_temporaries_do_not_accumulate_across_async_calls() {
    // Given
    let mut evaluator = SourceEvaluator::new_in_package("test").unwrap();
    let parsed = iris_parser::parse(
        "class Node { public fun pass(value) -> Object { value } }; module Worker { public async fun run(gate, node) -> Object { node?.pass(1); try { node?.missing() } catch error { node?.pass(await gate) } } }; let gate = Gate.new(); let task = Worker.run(gate, Node.new()); task",
    );
    assert!(parsed.is_clean(), "{parsed:#?}");

    // When
    let Value::Task(task) = evaluator.program(&parsed.program).unwrap() else {
        panic!("Task expected")
    };

    // Then
    let continuation = &evaluator.suspended[&task].continuation;
    let temporary_count = continuation.scopes.last().map_or(0, |scope| {
        scope.keys().filter(|name| name.starts_with('\0')).count()
    });
    assert!(
        temporary_count == 1,
        "only the active safe-call receiver may remain across suspension"
    );

    evaluator
        .program(&iris_parser::parse("Gate.complete(gate, 9)").program)
        .unwrap();
    evaluator.drive_ready_continuations().unwrap();
    assert_eq!(evaluator.tasks[&task], Ok(Value::Integer(9_u8.into())));
    assert!(!evaluator.suspended.contains_key(&task));
}
