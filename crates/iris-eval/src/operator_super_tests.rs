use iris_runtime::Value as RuntimeValue;

use super::{EvaluationError, evaluate};

#[test]
fn logical_not_uses_to_bool_dispatch_for_builtins_and_overrides() {
    // Given
    let source = "class A { public fun to_bool() -> Bool { false } }; [!true, !nil, !A.new()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ]))
    );
}

#[test]
fn logical_and_preserves_falsy_operand_without_evaluating_rhs() {
    // Given
    let source = "mut side = 0; class Falsy { public fun to_bool() -> Bool { false } }; class Rhs { public fun evaluate() -> Integer { side = 1 } }; let value = Falsy.new() && Rhs.new().evaluate(); [value, side]";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Ok(RuntimeValue::Array(values))
            if matches!(values.as_slice(), [RuntimeValue::Object(_), RuntimeValue::Integer(value)] if value == &0_u8.into())
    ));
}

#[test]
fn logical_and_does_not_coerce_nil_to_false() {
    // Given
    let source = "nil && 1";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Nil));
}

#[test]
fn logical_or_preserves_truthy_operand_and_evaluates_rhs_when_falsy() {
    // Given
    let preserve = "mut side = 0; class Truthy { public fun to_bool() -> Bool { true } }; class Rhs { public fun evaluate() -> Integer { side = 1 } }; let value = Truthy.new() || Rhs.new().evaluate(); [value, side]";
    let evaluate_rhs = "nil || 1";

    // When
    let preserved = evaluate(preserve);
    let fallback = evaluate(evaluate_rhs);

    // Then
    assert!(matches!(
        preserved,
        Ok(RuntimeValue::Array(values))
            if matches!(values.as_slice(), [RuntimeValue::Object(_), RuntimeValue::Integer(value)] if value == &0_u8.into())
    ));
    assert_eq!(fallback, Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn logical_operators_propagate_raised_to_bool_and_reject_non_bool_results() {
    // Given
    let raises = [
        "class A { public fun to_bool() -> Bool { raise :boom } }; !A.new()",
        "class A { public fun to_bool() -> Bool { raise :boom } }; A.new() && 1",
        "class A { public fun to_bool() -> Bool { raise :boom } }; A.new() || 1",
    ];
    let non_bools = [
        "class A { public fun to_bool() -> Bool { 1 } }; !A.new()",
        "class A { public fun to_bool() -> Bool { 1 } }; A.new() && 1",
        "class A { public fun to_bool() -> Bool { 1 } }; A.new() || 1",
    ];

    // When
    let raised_results = raises.map(evaluate);
    let non_bool_results = non_bools.map(evaluate);

    // Then
    assert!(raised_results.into_iter().all(|result| {
        result == Err(EvaluationError::Raised(RuntimeValue::Symbol("boom".into())))
    }));
    assert!(
        non_bool_results
            .into_iter()
            .all(|result| result == Err(EvaluationError::TypeContractError))
    );
}

#[test]
fn qualified_super_uses_the_superclass_method_and_bare_super_still_works() {
    // Given
    let qualified = "class B { public fun m() -> Symbol { :base } }; class C extends B { override public fun m() -> Symbol { super.m() } }; C.new().m()";
    let bare = "class B { public fun m() -> Symbol { :base } }; class C extends B { override public fun m() -> Symbol { super() } }; C.new().m()";

    // When
    let qualified_result = evaluate(qualified);
    let bare_result = evaluate(bare);

    // Then
    assert_eq!(qualified_result, Ok(RuntimeValue::Symbol("base".into())));
    assert_eq!(bare_result, Ok(RuntimeValue::Symbol("base".into())));
}

#[test]
fn logical_operators_cannot_be_declared_as_class_methods() {
    // Given
    let sources = [
        "class A { public fun !() -> Bool { true } }",
        "class A { public fun &&(other: Object) -> Bool { true } }",
        "class A { public fun ||(other: Object) -> Bool { true } }",
    ];

    // When
    let results = sources.map(evaluate);

    // Then
    assert!(
        results
            .into_iter()
            .all(|result| result == Err(EvaluationError::ParseDiagnostic))
    );
}
