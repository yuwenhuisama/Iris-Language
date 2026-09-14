use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

#[test]
fn duplicate_class_and_module_origins_are_rejected_not_merged() {
    // Given / When
    let classes = evaluate("class Same {} class Same {} Same");
    let modules = evaluate("module Same {} module Same {} Same");

    // Then
    assert_eq!(classes, Err(EvaluationError::ParseDiagnostic));
    assert_eq!(modules, Err(EvaluationError::ParseDiagnostic));
}

#[test]
fn an_open_body_extends_one_existing_origin() {
    // Given
    let source = "class Item { public fun first() -> Integer { 1 } } open class Item { public fun second() -> Integer { 2 } } let item = Item.new(); item.first() + item.second()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(Value::Integer(3_u8.into())));
}
