use iris_runtime::{IntegerValue, Kernel, KernelError, NativeSelector, Value};

#[test]
fn rejects_identity_less_operands_with_a_typed_identity_error() {
    // Given
    let value = Value::Integer(IntegerValue::from(1_u8));

    // When
    let result = Kernel::same_identity(&value, &value);

    // Then
    assert_eq!(result, Err(KernelError::Identity));
}

#[test]
fn dispatches_not_equal_as_a_native_method_send() -> Result<(), KernelError> {
    // Given
    let mut registry = iris_runtime::ClassRegistry::new();
    let kernel = Kernel::new(&mut registry)?;

    // When
    let result = kernel.send(
        &registry,
        Value::Integer(IntegerValue::from(1_u8)),
        NativeSelector::NotEqual,
        &[Value::Integer(IntegerValue::from(2_u8))],
    );

    // Then
    assert_eq!(result, Ok(Value::Bool(true)));
    Ok(())
}
