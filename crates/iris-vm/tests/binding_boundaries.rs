#![expect(
    clippy::unwrap_used,
    reason = "tests assert compilation and diagnostic category"
)]

use iris_runtime::Value;

#[test]
fn compile_rejects_when_inferred_binding_changes_type() {
    let source = "mut port = 8080; port = \"9090\"; port";
    let outcome = iris_vm::compile(source);
    assert_eq!(
        outcome.unwrap_err().kind,
        iris_vm::CompileErrorKind::StaticDiagnostic {
            code: "BINDING_FIXED_LOCAL_TYPE"
        }
    );
}

#[test]
fn compile_rejects_when_annotated_binding_changes_type() {
    let source = "mut port: Integer = 8080; port = \"9090\"; port";
    let outcome = iris_vm::compile(source);
    assert_eq!(
        outcome.unwrap_err().kind,
        iris_vm::CompileErrorKind::StaticDiagnostic {
            code: "BINDING_FIXED_LOCAL_TYPE"
        }
    );
}

#[test]
fn compile_rejects_when_assignment_leaves_union() {
    let source = "mut port: Integer | String = 8080; port = true; port";
    let outcome = iris_vm::compile(source);
    assert_eq!(
        outcome.unwrap_err().kind,
        iris_vm::CompileErrorKind::StaticDiagnostic {
            code: "BINDING_FIXED_LOCAL_TYPE"
        }
    );
}

#[test]
fn cell_retains_value_when_dynamic_assignment_fails() {
    let source = r#"
        module Input { public module fun read(value) { value } }
        mut port = 8080
        let rejected = try { port = Input.read("9090") } catch error { nil }
        port
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Integer(8080_u64.into())));
}

#[test]
fn captured_cell_retains_value_when_dynamic_assignment_fails() {
    let source = r#"
        module Main {
            public module fun run(value) {
                mut port: Integer = 8080
                let write = { ||; port = value }
                try { write.call() } catch error { nil }
                port
            }
        }
        Main.run("9090")
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Integer(8080_u64.into())));
}

#[test]
fn program_cell_retains_value_when_method_assignment_fails() {
    let source = r#"
        mut port: Integer | String = 8080
        module Main { public module fun write(value) { port = value } }
        let rejected = try { Main.write(true) } catch error { nil }
        port
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Integer(8080_u64.into())));
}

#[test]
fn unknown_initializer_stays_dynamic_when_runtime_value_is_integer() {
    let source = r#"
        module Input { public module fun read(value) { value } }
        mut port = Input.read(8080)
        let written = port = "9090"
        port
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Text("9090".to_owned())));
}

#[test]
fn union_assignment_succeeds_when_value_is_a_member() {
    let source = "mut port: Integer | String = 8080; let written = port = \"9090\"; port";
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Text("9090".to_owned())));
}

#[test]
fn rejection_is_catchable_when_method_local_write_crosses_boundary() {
    let source = r#"
        module Main {
            public module fun run(value) {
                mut port = 8080
                let failure = try { port = value } catch error { error }
                [failure, port]
            }
        }
        Main.run("9090")
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("TypeContractError".to_owned()),
            Value::Integer(8080_u64.into()),
        ])))
    );
}

#[test]
fn nested_capture_keeps_contract_when_write_crosses_boundary() {
    let source = r#"
        module Main {
            public module fun run(value) {
                mut port = 8080
                let outer = { ||; { ||; port = value } }
                let write = outer.call()
                try { write.call() } catch error { nil }
                port
            }
        }
        Main.run("9090")
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Integer(8080_u64.into())));
}

#[test]
fn logical_assignment_retains_cell_when_selected_value_is_incompatible() {
    let source = r#"
        module Input { public module fun read(value) { value } }
        mut enabled = false
        let failure = try { enabled ||= Input.read("yes") } catch error { nil }
        enabled
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Bool(false)));
}

#[test]
fn compound_assignment_checks_result_when_operator_changes_type() {
    let source = r#"
        class Counter { public fun +(value) { "bad" } }
        mut counter: Counter = Counter.new()
        let original = counter
        let failure = try { counter += 1 } catch error { nil }
        counter.same?(original)
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Bool(true)));
}

#[test]
fn rhs_effects_survive_when_guard_rejects_storage() {
    let source = r#"
        module Input { public module fun read(log) { log.append(:rhs); "bad" } }
        let log = []
        mut port = 8080
        let failure = try { port = Input.read(log) } catch error { log.append(error) }
        [port, log]
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(8080_u64.into()),
            Value::Array(iris_runtime::ArrayRef::new(vec![
                Value::Symbol("rhs".to_owned()),
                Value::Symbol("TypeContractError".to_owned())
            ])),
        ])))
    );
}

#[test]
fn logical_assignment_skips_guard_when_write_branch_is_skipped() {
    let source = r#"
        module Input { public module fun read() { raise :unexpected } }
        mut enabled = true
        let written = enabled ||= Input.read()
        enabled
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(Value::Bool(true)));
}
