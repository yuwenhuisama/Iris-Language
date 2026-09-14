use iris_runtime::{
    Capability, ClassError, ClassRegistry, CompositionEdge, DecoratorTransform, DispatchContext,
    DispatchError, DispatchOutcome, MetaCapabilities, MethodBody, PolicyOrigin, RevisionId,
    Selector, StaticSpine, Visibility,
};

const SLOT: Selector = Selector::new(1);
const ADDED: Selector = Selector::new(2);

#[test]
fn replacement_is_allowed_when_origin_denies_only_method_set() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(
        StaticSpine::new(1)
            .with_meta_capabilities(MetaCapabilities::denying(&[Capability::MethodSet])),
        None,
        &[],
    )?;
    let original =
        registry.publish_origin_method(class, SLOT, MethodBody::new(10), Visibility::Public)?;

    // When
    let wrapped = registry.publish_candidate_decorated_method(
        class,
        SLOT,
        MethodBody::new(20),
        Visibility::Public,
        [DecoratorTransform::metadata("wrap", ["checked"])],
    )?;

    // Then
    assert_ne!(wrapped.id(), original.id());
    assert_eq!(registry.method_by_id(wrapped.id()), Some(wrapped));
    assert_eq!(registry.staged_method(class, SLOT), Some(wrapped.id()));
    assert_eq!(
        registry.require_meta_capability(class, Capability::MethodBody),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        registry.active(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        registry.dispatch(class, SLOT),
        Err(DispatchError::Class(ClassError::UnknownClassId(class)))
    );
    registry.commit_declaration_group()?;
    let revision = registry.active(class)?;
    assert_eq!(
        (revision.id(), revision.number(), revision.commit_id()),
        (RevisionId::new(0), 1, 1)
    );
    assert_eq!(revision.decorators()[0].identity(), "wrap");
    assert_eq!(
        registry.dispatch(class, SLOT),
        Ok(DispatchOutcome::Invoke(wrapped))
    );
    Ok(())
}

#[test]
fn addition_is_denied_when_origin_denies_method_set() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(
        StaticSpine::new(1)
            .with_meta_capabilities(MetaCapabilities::denying(&[Capability::MethodSet])),
        None,
        &[],
    )?;

    // When
    let result = registry.publish_method(class, ADDED, MethodBody::new(20), Visibility::Private);

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodSet, policy_origin: PolicyOrigin::Class(origin), ..
    }) if target == class && origin == class));
    assert_eq!(registry.staged_method(class, ADDED), None);
    Ok(())
}

#[test]
fn wrap_is_denied_when_origin_denies_method_body() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(
        StaticSpine::new(1)
            .with_meta_capabilities(MetaCapabilities::denying(&[Capability::MethodBody])),
        None,
        &[],
    )?;
    let original =
        registry.publish_origin_method(class, SLOT, MethodBody::new(10), Visibility::Public)?;

    // When
    let result = registry.publish_candidate_decorated_method(
        class,
        SLOT,
        MethodBody::new(20),
        Visibility::Public,
        [],
    );

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodBody, policy_origin: PolicyOrigin::Class(origin), ..
    }) if target == class && origin == class));
    assert_eq!(registry.staged_method(class, SLOT), Some(original.id()));
    Ok(())
}

#[test]
fn private_addition_is_callable_after_commit_when_candidate_allows_method_set()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;
    registry.require_candidate_meta_capability(class, Capability::MethodSet)?;

    // When
    let method = registry.publish_method(class, ADDED, MethodBody::new(30), Visibility::Private)?;
    registry.commit_declaration_group()?;

    // Then
    assert_eq!(registry.method_by_id(method.id()), Some(method));
    assert_eq!(
        registry.dispatch(class, ADDED),
        Err(DispatchError::VisibilityDenied { selector: ADDED })
    );
    assert_eq!(
        registry.dispatch_with_context(class, ADDED, DispatchContext::implementation(class, true)),
        Ok(DispatchOutcome::Invoke(method))
    );
    assert_eq!(registry.active(class)?.number(), 1);
    Ok(())
}

#[test]
fn inherited_module_denial_reports_exact_origin_when_wrapping() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module_with_capabilities(
        &[],
        MetaCapabilities::denying(&[Capability::MethodBody]),
    )?;
    let parent = registry.define_class_with_capabilities_and_composition_edges(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
        &[CompositionEdge::new(module, false)],
    )?;
    let class = registry.stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;
    registry.publish_origin_method(class, SLOT, MethodBody::new(10), Visibility::Public)?;

    // When
    let result = registry.publish_method(class, SLOT, MethodBody::new(20), Visibility::Public);

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodBody, policy_origin: PolicyOrigin::Module(origin), ..
    }) if target == class && origin == module));
    Ok(())
}

#[test]
fn inherited_class_denial_reports_exact_origin_when_adding() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class_with_capabilities(
        StaticSpine::new(1),
        None,
        MetaCapabilities::denying(&[Capability::MethodSet]),
    )?;
    let class = registry.stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;

    // When
    let result = registry.require_candidate_meta_capability(class, Capability::MethodSet);

    // Then
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodSet, policy_origin: PolicyOrigin::Class(origin), ..
    }) if target == class && origin == parent));
    Ok(())
}

#[test]
fn rollback_consumes_no_publication_identity_when_checked_mutations_were_staged()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;
    registry.publish_method(class, ADDED, MethodBody::new(10), Visibility::Private)?;
    registry.publish_candidate_decorated_method(
        class,
        ADDED,
        MethodBody::new(20),
        Visibility::Private,
        [],
    )?;

    // When
    registry.roll_back_group();

    // Then
    assert_eq!(
        registry.require_candidate_meta_capability(class, Capability::MethodBody),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        registry.active(class),
        Err(ClassError::UnknownClassId(class))
    );
    let next = registry.define_class(StaticSpine::new(2), None)?;
    assert_eq!(
        (
            registry.active(next)?.id(),
            registry.active(next)?.commit_id()
        ),
        (RevisionId::new(0), 1)
    );
    Ok(())
}
