use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ArrayRef, Value};

#[test]
fn contract_view_equality_hash_and_identityless_same_remain_distinct() {
    // Given
    let source = "contract Named { fun name() -> Symbol } class Item { public fun name() -> Symbol { :item } } impl Item for Named {} let item = Item.new(); let left = item as Named; let right = item as Named; %[left == right, left.hash() == right.hash(), item same? item]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
        ])))
    );
}

#[test]
fn contract_view_same_remains_an_identity_error() {
    // Given / When
    let result = evaluate(
        "contract C {} class Item {} impl Item for C {} let view = Item.new() as C; view same? view",
    );

    // Then
    assert_eq!(result, Err(EvaluationError::IdentityError));
}
