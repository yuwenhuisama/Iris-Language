use iris_eval::evaluate;
use iris_runtime::{ArrayRef, Value};

#[test]
fn nonnull_evaluates_its_operand_once_and_preserves_identity() {
    // Given
    let source = "global mut $calls = 0; class Item {}; fun make() { $calls = $calls + 1; Item.new() }; let item = make()!; %[item same? item, $calls]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn nonnull_nil_raises_the_canonical_type_error_with_context() {
    // Given
    let source = "try { nil! } catch error: TypeError, context { %[error is? TypeError, context.value same? error] }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
        ])))
    );
}
