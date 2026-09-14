use iris_runtime::{
    CallableKind, CallableSignature, ClassError, ClassId, ClassRegistry, CoreClass, NominalType,
    SignatureType, StaticSpine, Value,
};

const _: fn(CallableKind) = |kind| match kind {
    CallableKind::Closure | CallableKind::BoundMethod => (),
};

fn nominal(identity: u64) -> SignatureType {
    SignatureType::Nominal(NominalType::new(ClassId::new(identity), Vec::new()))
}

#[test]
fn equal_signatures_reuse_identity_when_constructed_independently()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let first = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![nominal(1)], nominal(2)),
    )?;

    // When
    let second = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![nominal(1)], nominal(2)),
    )?;

    // Then
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn signatures_are_distinct_when_kind_order_arity_or_result_changes()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let signatures = [
        (
            CallableKind::Closure,
            vec![nominal(1), nominal(2)],
            nominal(3),
        ),
        (
            CallableKind::BoundMethod,
            vec![nominal(1), nominal(2)],
            nominal(3),
        ),
        (
            CallableKind::Closure,
            vec![nominal(2), nominal(1)],
            nominal(3),
        ),
        (CallableKind::Closure, vec![nominal(1)], nominal(3)),
        (
            CallableKind::Closure,
            vec![nominal(1), nominal(2)],
            nominal(4),
        ),
    ];

    // When
    let values = signatures
        .into_iter()
        .map(|(kind, parameters, result)| {
            registry.intern_callable_type(kind, CallableSignature::new(parameters, result))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Then
    for (index, value) in values.iter().enumerate() {
        assert!(!values[..index].contains(value));
    }
    Ok(())
}

#[test]
fn nested_closed_task_result_preserves_structure_when_reified()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let task = ClassId::new(10);
    let array = ClassId::new(11);
    let result = SignatureType::Nominal(NominalType::new(
        task,
        vec![NominalType::new(
            array,
            vec![NominalType::new(ClassId::new(12), vec![])],
        )],
    ));
    let signature = CallableSignature::new(vec![], result.clone());

    // When
    let value = registry.intern_callable_type(CallableKind::BoundMethod, signature.clone())?;

    // Then
    let Value::Type(identity, arguments) = value else {
        unreachable!()
    };
    assert!(arguments.is_empty());
    let descriptor = registry
        .callable_type(identity)
        .ok_or("missing callable metadata")?;
    assert_eq!(descriptor.kind(), CallableKind::BoundMethod);
    assert_eq!(descriptor.signature(), &signature);
    assert_eq!(descriptor.signature().result(), &result);
    assert_eq!(
        registry.class(identity),
        Err(ClassError::UnknownClassId(identity))
    );
    assert!(registry.active(identity).is_err());
    Ok(())
}

#[test]
fn user_sequences_are_unchanged_when_callable_and_core_types_are_interned()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut control = ClassRegistry::new();
    let mut registry = ClassRegistry::new();
    let object = registry.define_class(StaticSpine::new(1), None)?;
    assert_eq!(object, control.define_class(StaticSpine::new(1), None)?);
    registry.register_core_class(CoreClass::Type, object)?;

    // When
    let value = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], nominal(object.raw())),
    )?;
    let next = registry.define_class(StaticSpine::new(2), None)?;
    let expected = control.define_class(StaticSpine::new(2), None)?;

    // Then
    assert_eq!(next, expected);
    assert_eq!(registry.active(next)?, control.active(expected)?);
    let Value::Type(identity, _) = value else {
        unreachable!()
    };
    assert_ne!(identity, CoreClass::Type.id());
    assert_ne!(identity, object);
    assert_ne!(identity, next);
    assert!(registry.callable_type(CoreClass::Type.id()).is_none());
    Ok(())
}

#[test]
fn generic_arguments_are_invariant_when_result_arguments_differ()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let task = ClassId::new(10);
    let first = CallableSignature::new(
        vec![],
        SignatureType::Nominal(NominalType::new(
            task,
            vec![NominalType::new(ClassId::new(1), vec![])],
        )),
    );
    let second = CallableSignature::new(
        vec![],
        SignatureType::Nominal(NominalType::new(
            task,
            vec![NominalType::new(ClassId::new(2), vec![])],
        )),
    );

    // When
    let first = registry.intern_callable_type(CallableKind::Closure, first)?;
    let second = registry.intern_callable_type(CallableKind::Closure, second)?;

    // Then
    assert_ne!(first, second);
    Ok(())
}
