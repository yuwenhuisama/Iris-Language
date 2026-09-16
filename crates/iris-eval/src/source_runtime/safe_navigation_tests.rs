use crate::evaluate;
use iris_runtime::{ArrayRef, Value};

#[test]
fn safe_navigation_short_circuits_nil_before_all_downstream_postfix_work() {
    // Given
    let source = "nil?.missing({ raise :positional }.call(), named: { raise :keyword }.call(), &{ raise :block_argument }.call()) { raise :trailing_block }[{ raise :index }.call()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(Value::Nil));
}

#[test]
fn safe_navigation_evaluates_the_receiver_once_and_guards_each_nil_result() {
    // Given
    let source = "mut receivers = 0; mut lookups = 0; class Node { public fun first() { nil } public fun method_missing(name, args, block) { lookups = lookups + 1; raise :lookup } }; let receiver = { receivers = receivers + 1; Node.new() }; let result = receiver.call()?.first().missing(); %[result, receivers, lookups]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Nil,
            Value::Integer(1_u8.into()),
            Value::Integer(0_u8.into()),
        ])))
    );
}

#[test]
fn safe_navigation_dispatches_non_nil_and_false_receivers_normally() {
    // Given
    let source = "class Node { public fun ready?() { true } public fun save!() { 7 } }; let node = Node.new(); let ready = node?.ready?; %[ready.call(), node?.save!(), false?.to_string]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(7_u8.into()),
            Value::Text("false".into()),
        ])))
    );
}

#[test]
fn safe_navigation_runs_calls_indexes_and_trailing_blocks_while_non_nil() {
    // Given
    let source = "class Node { public fun values() { %[7] } public fun run(&block) { block.call() } }; let node = Node.new(); %[node?.values()[0], node?.run() { 9 }]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u8.into()),
            Value::Integer(9_u8.into()),
        ])))
    );
}
