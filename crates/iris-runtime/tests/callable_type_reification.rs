use iris_runtime::{
    CallableKind, CallableSignature, ClassId, ClassRegistry, ComposedType, NominalType, Runtime,
    SignatureType, StaticSpine, TypeAtom, Value,
};

#[test]
fn composed_signature_reuses_identity_when_normal_forms_match()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let union = ComposedType::Union(vec![
        TypeAtom::Nominal(ClassId::new(1), vec![]),
        TypeAtom::Nominal(ClassId::new(2), vec![]),
    ]);
    let signature = CallableSignature::new(vec![union.clone().into()], ComposedType::Never.into());
    let first = registry.intern_callable_type(CallableKind::Closure, signature)?;

    // When
    let second = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![union.into()], ComposedType::Never.into()),
    )?;

    // Then
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn interning_creates_no_constructible_class_when_runtime_has_no_user_classes()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut runtime = Runtime::new();
    let value = runtime.registry_mut().intern_callable_type(
        CallableKind::BoundMethod,
        CallableSignature::new(vec![], ComposedType::Never.into()),
    )?;
    let Value::Type(identity, _) = value else {
        unreachable!()
    };

    // When
    let allocation = runtime.allocate(identity);
    let origin = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime.registry_mut().commit_declaration_group()?;

    // Then
    assert!(allocation.is_err());
    assert_eq!(origin.raw(), 0);
    assert_eq!(runtime.registry().active(origin)?.id().raw(), 0);
    assert_eq!(runtime.registry().active(origin)?.commit_id(), 1);
    assert!(runtime.registry().callable_type(identity).is_some());
    Ok(())
}

#[test]
fn nominal_result_is_invariant_when_classes_have_an_inheritance_relationship()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class(StaticSpine::new(1), None)?;
    let child = registry.define_class(StaticSpine::new(2), Some(parent))?;

    // When
    let parent_type = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], NominalType::new(parent, vec![]).into()),
    )?;
    let child_type = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], NominalType::new(child, vec![]).into()),
    )?;

    // Then
    assert_ne!(parent_type, child_type);
    Ok(())
}

#[test]
fn nested_callable_result_reuses_identity_when_inner_type_is_reified_again()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut registry = ClassRegistry::new();
    let inner_signature = CallableSignature::new(vec![], ComposedType::Never.into());
    let inner =
        registry.intern_callable_type(CallableKind::BoundMethod, inner_signature.clone())?;
    let Value::Type(inner_id, _) = inner else {
        unreachable!()
    };
    let result = SignatureType::Nominal(NominalType::new(inner_id, vec![]));
    let first = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], result),
    )?;

    // When
    let Value::Type(repeated_id, _) =
        registry.intern_callable_type(CallableKind::BoundMethod, inner_signature)?
    else {
        unreachable!()
    };
    let second = registry.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], NominalType::new(repeated_id, vec![]).into()),
    )?;

    // Then
    assert_eq!(first, second);
    assert_eq!(
        registry
            .callable_type(repeated_id)
            .ok_or("missing inner Type")?
            .kind(),
        CallableKind::BoundMethod
    );
    Ok(())
}
