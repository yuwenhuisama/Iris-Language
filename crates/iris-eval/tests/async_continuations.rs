use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

#[test]
fn effects_execute_once_when_two_gates_suspend_nested_operands() {
    let source = r#"
        mut effects = %[];
        module Work {
            public fun mark(value) -> Integer { effects.append(value); value }
            public async fun run(first, second) -> Array {
                mut count = 0;
                count += 1;
                let left = Work.mark(10) + (await first);
                count += 1;
                let right = Work.mark(20) + (await second);
                %[left, right, count, effects]
            }
        }
        let first = Gate.new(); let second = Gate.new();
        let task = Work.run(first, second);
        let posted_first = Gate.complete(first, 1);
        let posted_second = Gate.complete(second, 2);
        Host.run(task)
    "#;
    let outcome = evaluate(source);
    assert_eq!(outcome, evaluate("%[11, 22, 2, %[10, 20]]"));
}

#[test]
fn async_closure_returns_task_when_body_completes_eagerly() {
    let source = "let work = { async || -> Integer; 7 }; Host.run(work.call())";
    let outcome = evaluate(source);
    assert_eq!(outcome, Ok(Value::Integer(7u64.into())));
}

#[test]
fn match_guard_retains_subject_when_await_resumes_false() {
    let source = r#"module Work { public async fun run(gate) -> Integer {
        match %[7] { [item] if (await gate) => item, [other] => other + 1 }
    } }; let gate = Gate.new(); let task = Work.run(gate);
    let posted = Gate.complete(gate, false); Host.run(task)"#;
    let outcome = evaluate(source);
    assert_eq!(outcome, Ok(Value::Integer(8u64.into())));
}

#[test]
fn cells_and_operands_survive_when_host_drives_two_separate_posts() {
    let mut session = Session::new().unwrap();
    session
        .evaluate(
            r#"mut effects = %[]; module Work {
        public fun mark(n) -> Integer { effects.append(n); n }
        public async fun run(a, b) -> Array {
            mut count = 0; count += 1;
            let first = Work.mark(10) + (await a);
            count += 1;
            let second = Work.mark(20) + (await b);
            %[first, second, count, effects]
        }
    }; let a = Gate.new(); let b = Gate.new(); let task = Work.run(a, b); nil"#,
        )
        .unwrap();
    session
        .evaluate("let posted = Gate.complete(a, 1); nil")
        .unwrap();
    assert_eq!(
        session.evaluate("Host.run(task)"),
        Err(EvaluationError::UnsupportedConstruct)
    );
    assert_eq!(session.evaluate("effects"), evaluate("%[10, 20]"));
    session
        .evaluate("let posted_again = Gate.complete(b, 2); nil")
        .unwrap();
    let outcome = session.evaluate("Host.run(task)");
    assert_eq!(outcome, evaluate("%[11, 22, 2, %[10, 20]]"));
}

#[test]
fn cleanup_and_loops_resume_when_catch_and_finally_await() {
    let source = r#"mut log = %[];
        class Resource { public fun close() -> Nil { log.append(:close) } }
        module Work { public async fun run(a, b) -> Array {
            using(Resource.new()) { |resource|
                mut count = 0;
                while count < 2 { count += 1; log.append(count); await a };
                try { raise :failure } catch error {
                    log.append(error); await b
                } finally { await a; log.append(:finally) }
            };
            log
        } }
        let a = Gate.new(); let b = Gate.new(); let task = Work.run(a, b);
        let first = Gate.complete(a, 1); let second = Gate.complete(b, 2);
        Host.run(task)"#;
    let outcome = evaluate(source);
    assert_eq!(
        outcome,
        evaluate("let expected = %[1, 2, :failure, :finally, :close]; expected")
    );
}

#[test]
fn awaiting_pending_task_resumes_when_child_completes() {
    let source = r#"mut log = %[];
        module Work {
            public async fun child(gate) -> Integer { log.append(:child); await gate }
            public async fun parent(child) -> Integer { log.append(:parent); 10 + (await child) }
        }
        let gate = Gate.new(); let child = Work.child(gate); let parent = Work.parent(child);
        let posted = Gate.complete(gate, 7); let value = Host.run(parent);
        %[value, log]"#;
    let outcome = evaluate(source);
    assert_eq!(
        outcome,
        evaluate("let expected = %[17, %[:child, :parent]]; expected")
    );
}

#[test]
fn for_cursor_resumes_when_its_body_awaits() {
    let source = r#"module Work { public async fun run(gate) -> Array {
        mut output = %[];
        for item in %[1, 2] { let value = item + (await gate); output.append(value) };
        output
    } }; let gate = Gate.new(); let task = Work.run(gate);
    let posted = Gate.complete(gate, 10); Host.run(task)"#;
    let outcome = evaluate(source);
    assert_eq!(outcome, evaluate("%[11, 12]"));
}

#[test]
fn async_closure_keeps_captured_cell_when_it_suspends() {
    let source = r#"mut count = 0; let gate = Gate.new();
        let work = { async || -> Integer; count += 1; let value = await gate; count += value; count };
        let task = work.call(); let posted = Gate.complete(gate, 7); Host.run(task)"#;
    let outcome = evaluate(source);
    assert_eq!(outcome, Ok(Value::Integer(8u64.into())));
}

#[test]
fn assignment_preserves_read_and_skips_logical_rhs_when_suspended() {
    let mut session = Session::new().unwrap();
    session
        .evaluate(
            r#"mut count = 1; module Work { public async fun run(gate) -> Integer {
        mut skipped = false; skipped &&= (await gate); count += (await gate); count
    } }; let gate = Gate.new(); let task = Work.run(gate); nil"#,
        )
        .unwrap();
    session.evaluate("count = 100").unwrap();
    session
        .evaluate("let posted = Gate.complete(gate, 7); nil")
        .unwrap();
    let outcome = session.evaluate("Host.run(task)");
    assert_eq!(outcome, Ok(Value::Integer(8u64.into())));
}
