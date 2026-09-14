use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ArrayRef, IntegerValue, Value};

#[test]
fn instance_fields_initialize_once_for_each_instance() {
    // Given
    let source = "global mut $next = 0; class Item { mut @id: Integer = ($next = $next + 1); public fun id() -> Integer { @id } } let first = Item.new(); let second = Item.new(); %[first.id(), first.id(), second.id()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(IntegerValue::from(1_u8)),
            Value::Integer(IntegerValue::from(1_u8)),
            Value::Integer(IntegerValue::from(2_u8)),
        ])))
    );
}

#[test]
fn instance_field_mutability_and_annotation_are_enforced() {
    // Given / When
    let immutable = evaluate(
        "class Item { let @id = 1; public fun replace() { @id = 2 } } Item.new().replace()",
    );
    let typed = evaluate(
        "class Item { mut @id: Integer = 1; public fun replace() { @id = \"bad\" } } Item.new().replace()",
    );

    // Then
    assert_eq!(immutable, Err(EvaluationError::ImmutableBinding));
    assert_eq!(typed, Err(EvaluationError::TypeContractError));
}

#[test]
fn open_class_cannot_add_an_instance_field() {
    // Given / When
    let result = evaluate("class Item {}; open class Item { mut @late = 1 }; Item.new()");

    // Then
    assert_eq!(result, Err(EvaluationError::ParseDiagnostic));
}

#[test]
fn instance_field_survives_static_impl_and_later_open_mixin() {
    // Given
    let source = "contract Named { fun name()->String } module Extra where Self: Named { public fun extra()->Integer{7} } class Item { let @name:String=\"iris\" public fun name()->String{@name} } impl Item for Named {} open class Item mixin Extra {} let item=Item.new(); item.extra()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(Value::Integer(IntegerValue::from(7_u8))));
}
