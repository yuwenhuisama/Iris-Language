use iris_runtime::{ClassId, HeapPayload, IntegerValue, RuntimeError, RuntimeHeap, Value};

#[test]
fn allocating_two_objects_yields_distinct_ids_and_identity_hashes() -> Result<(), RuntimeError> {
    // Given
    let mut heap = RuntimeHeap::new();
    let class = ClassId::new(7);

    // When
    let first = heap.alloc(class, HeapPayload::InstanceFields(Vec::new()))?;
    let second = heap.alloc(class, HeapPayload::InstanceFields(Vec::new()))?;

    // Then
    assert_ne!(first, second);
    assert_ne!(
        heap.lookup(first)?.identity_hash(),
        heap.lookup(second)?.identity_hash()
    );
    Ok(())
}

#[test]
fn object_identity_hash_is_stable_across_repeated_lookups() -> Result<(), RuntimeError> {
    // Given
    let mut heap = RuntimeHeap::new();
    let object = heap.alloc(ClassId::new(7), HeapPayload::InstanceFields(Vec::new()))?;

    // When
    let first_hash = heap.lookup(object)?.identity_hash();
    let second_hash = heap.lookup(object)?.identity_hash();

    // Then
    assert_eq!(first_hash, second_hash);
    Ok(())
}

#[test]
fn identity_hash_is_not_derived_from_sequential_storage_indices() -> Result<(), RuntimeError> {
    // Given
    let mut heap = RuntimeHeap::new();
    let class = ClassId::new(7);

    // When
    let first = heap.alloc(class, HeapPayload::InstanceFields(Vec::new()))?;
    let second = heap.alloc(class, HeapPayload::InstanceFields(Vec::new()))?;
    let third = heap.alloc(class, HeapPayload::InstanceFields(Vec::new()))?;
    let hashes = [
        heap.lookup(first)?.identity_hash(),
        heap.lookup(second)?.identity_hash(),
        heap.lookup(third)?.identity_hash(),
    ];

    // Then
    assert_ne!(hashes[0], 0);
    assert_ne!(hashes[1], 1);
    assert_ne!(hashes[2], 2);
    Ok(())
}

#[test]
fn looking_up_a_nonexistent_object_id_returns_a_typed_error() {
    // Given
    let heap = RuntimeHeap::new();

    // When
    let result = heap.lookup(iris_runtime::ObjectId::new(99));

    // Then
    assert_eq!(
        result,
        Err(RuntimeError::UnknownObjectId(iris_runtime::ObjectId::new(
            99
        )))
    );
}

#[test]
fn value_round_trips_a_large_integer_exactly() -> Result<(), num_bigint::ParseBigIntError> {
    // Given
    let integer = "1844674407370955161618446744073709551616".parse::<IntegerValue>()?;
    let value = Value::Integer(integer.clone());

    // When
    let round_tripped = value.clone();

    // Then
    assert_eq!(round_tripped, value);
    assert_eq!(round_tripped, Value::Integer(integer));
    Ok(())
}
