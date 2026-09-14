use iris_runtime::{
    Capability, ClassError, ClassRegistry, MetaCapabilities, MethodBody, PolicyOrigin, Selector,
    StaticSpine, Visibility,
};

#[test]
fn staged_module_denial_applies_when_active_policy_allows() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let module = registry.define_module_with_capabilities(
        &[],
        MetaCapabilities::denying(&[Capability::MethodSet]),
    )?;
    registry.begin_transaction(class)?;
    registry.recompose_candidate(class, module, true)?;
    registry.require_meta_capability(class, Capability::MethodSet)?;

    // When
    let result = registry.publish_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Private,
    );

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodSet, policy_origin: PolicyOrigin::Module(origin), ..
    }) if target == class && origin == module));
    Ok(())
}

#[test]
fn active_denial_survives_when_candidate_removes_denying_module() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module_with_capabilities(
        &[],
        MetaCapabilities::denying(&[Capability::MethodSet]),
    )?;
    let class = registry.define_class_with_capabilities_and_modules(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
        &[module],
    )?;
    registry.begin_transaction(class)?;
    registry.recompose_candidate(class, module, false)?;

    // When
    let result = registry.require_candidate_meta_capability(class, Capability::MethodSet);

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodSet, policy_origin: PolicyOrigin::Module(origin), ..
    }) if target == class && origin == module));
    Ok(())
}

#[test]
fn decorated_replacement_uses_candidate_slot_when_active_slot_is_absent() -> Result<(), ClassError>
{
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class_with_capabilities(
        StaticSpine::new(1),
        None,
        MetaCapabilities::denying(&[Capability::MethodSet]),
    )?;
    registry.begin_origin_transaction(class)?;
    let selector = Selector::new(1);
    registry.publish_origin_method(class, selector, MethodBody::new(1), Visibility::Public)?;

    // When
    let method = registry.publish_decorated_method(
        class,
        selector,
        MethodBody::new(2),
        Visibility::Public,
        [],
    )?;

    // Then
    assert_eq!(registry.staged_method(class, selector), Some(method.id()));
    assert!(registry.active(class)?.methods().is_empty());
    Ok(())
}

#[test]
fn newer_active_denial_wins_when_staged_policy_is_older() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.begin_transaction(class)?;
    let mut intervening = registry.open(class)?;
    intervening.deny_meta(&[Capability::MethodSet]);
    registry.publish(intervening)?;

    // When
    let result = registry.require_candidate_meta_capability(class, Capability::MethodSet);

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodSet, policy_origin: PolicyOrigin::Class(origin), ..
    }) if target == class && origin == class));
    Ok(())
}
