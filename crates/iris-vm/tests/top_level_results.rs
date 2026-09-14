use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

fn execute(source: &str) -> Value {
    let program = compile(source).expect("focused VM source compiles");
    run(&program).expect("focused VM source runs")
}

#[test]
fn global_declaration_does_not_contribute_to_program_result() {
    let given = "global let $g = 3; module M { public fun read() -> Integer { $g } } M.read()";

    let when = execute(given);

    assert_eq!(when, Value::Integer(3_u64.into()));
}

#[test]
fn global_collection_mutated_inside_method_does_not_leak_into_program_result() {
    let given = "global mut $g = %[1]; module M { public fun mutate() -> Object { $g[0] = 8 } public fun run() -> Object { M.mutate(); $g[0] } } M.run()";

    let when = execute(given);

    assert_eq!(when, Value::Integer(8_u64.into()));
}

#[test]
fn revision_subscriber_setup_does_not_prepend_the_initial_global_value() {
    let given = "global mut $received = :none; class B { } module M { public fun run() -> Object { Revision.subscribe({ |event| $received = event[0] }); B.open() { |transaction| 1 }; Revision.flush(); $received } } M.run()";

    let when = execute(given);

    assert_eq!(when, Value::Symbol("RevisionEvent".into()));
}

#[test]
fn expression_statements_still_aggregate_in_source_order() {
    let given = "1; 2; 3";

    let when = execute(given);

    assert_eq!(
        when,
        Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into()),
            Value::Integer(3_u64.into()),
        ]))
    );
}

#[test]
fn assignment_expression_statements_still_contribute_results() {
    let given = "mut n = 0; n = 2; n";

    let when = execute(given);

    assert_eq!(
        when,
        Value::Array(ArrayRef::new(vec![
            Value::Integer(2_u64.into()),
            Value::Integer(2_u64.into()),
        ]))
    );
}
