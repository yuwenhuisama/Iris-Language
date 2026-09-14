use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

#[test]
fn impl_is_order_independent_and_joins_the_ordinary_surface() {
    // Given
    let source = "impl Box for Show { public fun show() -> String { \"box\" } } contract Show { fun show() -> String } class Box {} Box.new().show()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(Value::Text("box".into())));
}

#[test]
fn empty_impl_binds_an_existing_compatible_method() {
    // Given
    let source = "contract Show { fun show() -> String } class Box { public fun show() -> String { \"box\" } } impl Box for Show {} (Box.new() as Show)..show()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(Value::Text("box".into())));
}

#[test]
fn impl_rejects_duplicates_missing_requirements_and_extra_methods() {
    // Given / When
    let duplicate = evaluate(
        "contract C { fun m() -> Nil } class A {} impl A for C { fun m() -> Nil { nil } } impl A for C { fun m() -> Nil { nil } } A",
    );
    let missing = evaluate("contract C { fun m() -> Nil } class A {} impl A for C {} A");
    let extra = evaluate(
        "contract C { fun m() -> Nil } class A {} impl A for C { fun m() -> Nil { nil } fun extra() -> Nil { nil } } A",
    );

    // Then
    assert_eq!(duplicate, Err(EvaluationError::TypeContractError));
    assert_eq!(missing, Err(EvaluationError::TypeContractError));
    assert_eq!(extra, Err(EvaluationError::TypeContractError));
}
