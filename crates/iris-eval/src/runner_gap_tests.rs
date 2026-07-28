use iris_runtime::{
    BuiltinClass, Kernel, KernelError, MethodBody, NativeSelector, Value as RuntimeValue,
    Visibility,
};

use super::{EvaluationError, evaluate};

#[test]
fn rejects_identity_less_operands_with_identity_error() {
    // Given
    let source = "Integer(1) same? Integer(1)";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Err(EvaluationError::Runtime(KernelError::Identity)));
}

#[test]
fn sends_not_equal_for_nan_and_ordinary_operands() {
    // Given
    let source = "[Float64.nan == Float64.nan, Float64.nan != Float64.nan, 1 != 2]";

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
fn identity_primitive_bypasses_replaced_equal_and_compare_slots()
-> Result<(), iris_runtime::KernelError> {
    // Given
    let mut kernel = Kernel::new()?;
    let bool_class = kernel.class(BuiltinClass::Bool)?;
    kernel.registry_mut().publish_method(
        bool_class,
        NativeSelector::Equal.id(),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    kernel.registry_mut().publish_method(
        bool_class,
        NativeSelector::Compare.id(),
        MethodBody::new(1),
        Visibility::Public,
    )?;

    // When
    let result = Kernel::same_identity(&RuntimeValue::Bool(true), &RuntimeValue::Bool(true));

    // Then
    assert_eq!(result, Ok(true));
    Ok(())
}

#[test]
fn constructs_negative_float64_infinity_for_fused_arithmetic() {
    // Given
    let construction = "Float64(-Infinity)";
    let fused_arithmetic = "Float64.infinity.mul_add(1, Float64(-Infinity))";

    // When
    let constructed = evaluate(construction);
    let fused = evaluate(fused_arithmetic);

    // Then
    assert!(matches!(
        constructed,
        Ok(RuntimeValue::Float64(value)) if value.is_infinite() && value.is_sign_negative()
    ));
    assert!(matches!(
        fused,
        Ok(RuntimeValue::Float64(value)) if value.is_nan()
    ));
}

#[test]
fn rejects_bare_infinity_outside_float64_construction() {
    // Given
    let source = "Infinity";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Err(EvaluationError::UnsupportedConstruct));
}
