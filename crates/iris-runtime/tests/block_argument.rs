use iris_runtime::{
    ArrayRef, ClassId, ClassRegistry, HeapPayload, Kernel, KernelError, RuntimeError, RuntimeHeap,
    Value, reachable_from,
};

#[test]
fn block_argument_clone_preserves_nested_identity() {
    let given = ArrayRef::new(vec![Value::Bool(true)]);
    let argument = Value::BlockArgument(Box::new(Value::Array(given.clone())));

    let when = argument.clone();

    let Value::BlockArgument(inner) = when else {
        unreachable!("block argument transport must survive cloning")
    };
    let Value::Array(held) = *inner else {
        unreachable!("nested array must survive cloning")
    };
    assert!(held.same(&given));
}

#[test]
fn block_argument_trace_reaches_nested_objects() -> Result<(), RuntimeError> {
    let mut heap = RuntimeHeap::new();
    let nested = heap.alloc(ClassId::new(1), HeapPayload::InstanceFields(vec![]))?;
    let holder = heap.alloc(
        ClassId::new(1),
        HeapPayload::InstanceFields(vec![Value::Object(nested)]),
    )?;
    let root = Value::BlockArgument(Box::new(Value::Array(ArrayRef::new(vec![Value::Object(
        holder,
    )]))));

    let when = reachable_from(&heap, &Default::default(), [&root]);

    assert_eq!(when.ids(), vec![nested, holder]);
    Ok(())
}

#[test]
fn block_argument_has_no_user_class() -> Result<(), KernelError> {
    let mut registry = ClassRegistry::new();
    let kernel = Kernel::new(&mut registry)?;
    let given = Value::BlockArgument(Box::new(Value::Nil));

    let when = kernel.class_of(&given);

    assert_eq!(when, Err(KernelError::Type));
    Ok(())
}
