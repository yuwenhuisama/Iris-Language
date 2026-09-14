use iris_runtime::{
    Capability, ClassError, ClassRegistry, CompositionEdge, DecoratorTransform, DecoratorViolation,
    MethodBody, MroEntry, RevisionId, Selector, StaticSpine, Visibility,
};

#[test]
fn origin_is_hidden_when_reserved_for_a_declaration() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();

    // When
    let class = registry.stage_class_origin(StaticSpine::new(7), None, &[])?;

    // Then
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        registry.active(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert!(registry.dispatch(class, Selector::new(1)).is_err());
    assert!(registry.is_staging(class));
    assert_eq!(
        registry.staged_origin(class)?.static_spine(),
        StaticSpine::new(7)
    );
    assert_eq!(
        registry.revision(RevisionId::new(0)),
        Err(ClassError::UnknownRevisionId(RevisionId::new(0)))
    );
    Ok(())
}

#[test]
fn origin_and_open_share_one_commit_when_the_complete_group_is_valid() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let existing = registry.define_class(StaticSpine::new(1), None)?;
    registry.begin_transaction(existing)?;
    registry.stage_decorators(
        existing,
        [DecoratorTransform::metadata("open", ["changed"])],
    )?;
    let class = registry.stage_class_origin(StaticSpine::new(2), Some(existing), &[])?;
    let selector = Selector::new(8);
    let method =
        registry.publish_origin_method(class, selector, MethodBody::new(9), Visibility::Public)?;
    registry.stage_origin_property(class, Selector::new(10), MethodBody::new(11))?;
    registry.stage_decorators(class, [DecoratorTransform::metadata("origin", ["checked"])])?;

    // When
    let published = registry.commit_declaration_group()?;

    // Then
    assert_eq!(published.len(), 2);
    let origin = registry.active(class)?;
    let reopened = registry.active(existing)?;
    assert_eq!((origin.number(), reopened.number()), (1, 2));
    assert_eq!(origin.commit_id(), reopened.commit_id());
    assert_eq!(origin.commit_id(), 2);
    assert_eq!(origin.id(), RevisionId::new(2));
    assert_eq!(origin.methods().get(&selector), Some(&method.id()));
    assert_eq!(origin.properties()[0].selector(), Selector::new(10));
    assert_eq!(origin.decorators()[0].identity(), "origin");
    assert!(!registry.is_staging(class));
    Ok(())
}

#[test]
fn entire_group_rolls_back_when_origin_decorator_is_invalid() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let existing = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(existing)?.clone();
    registry.begin_transaction(existing)?;
    registry.stage_decorators(
        existing,
        [DecoratorTransform::metadata("unpublished", ["value"])],
    )?;
    let class = registry.stage_class_origin(StaticSpine::new(2), None, &[])?;
    registry.stage_decorators(class, [DecoratorTransform::ChangeNominalIdentity])?;

    // When
    let result = registry.commit_declaration_group();

    // Then
    assert_eq!(
        result,
        Err(ClassError::DecoratorViolation {
            class,
            violation: DecoratorViolation::NominalIdentity
        })
    );
    assert_eq!(registry.active(existing)?, &before);
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert!(!registry.is_staging(existing));
    assert!(!registry.is_staging(class));
    let next = registry.define_class(StaticSpine::new(3), None)?;
    assert_ne!(next, class);
    assert_eq!(
        registry.active(next)?.id(),
        RevisionId::new(before.id().raw() + 1)
    );
    assert_eq!(registry.active(next)?.commit_id(), before.commit_id() + 1);
    Ok(())
}

#[test]
fn origin_rolls_back_when_an_existing_target_conflicts() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let existing = registry.define_class(StaticSpine::new(1), None)?;
    registry.begin_transaction(existing)?;
    let class = registry.stage_class_origin(StaticSpine::new(2), None, &[])?;
    let intervening = registry.publish(registry.open(existing)?)?;

    // When
    let result = registry.commit_declaration_group();

    // Then
    assert_eq!(
        result,
        Err(ClassError::MetaTransactionConflict { class: existing })
    );
    assert_eq!(registry.active(existing)?, &intervening);
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    let later = registry.publish(registry.open(existing)?)?;
    assert_eq!(later.id().raw(), intervening.id().raw() + 1);
    assert_eq!(later.commit_id(), intervening.commit_id() + 1);
    Ok(())
}

#[test]
fn reserved_origin_is_discarded_when_the_body_aborts() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;

    // When
    registry.roll_back_group();

    // Then
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert!(registry.staged_origin(class).is_err());
    let next = registry.define_class(StaticSpine::new(2), None)?;
    assert_eq!(registry.active(next)?.id(), RevisionId::new(0));
    assert_eq!(registry.active(next)?.commit_id(), 1);
    Ok(())
}

#[test]
fn origin_inherits_final_parent_mro_when_parent_is_in_the_group() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class(StaticSpine::new(1), None)?;
    let module = registry.define_module(&[])?;
    registry.begin_transaction(parent)?;
    registry.recompose_candidate(parent, module, true)?;
    let class = registry.stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;

    // When
    registry.commit_declaration_group()?;

    // Then
    assert_eq!(
        registry.active(class)?.mro(),
        &[
            MroEntry::Class(class),
            MroEntry::Class(parent),
            MroEntry::Module(module)
        ]
    );
    Ok(())
}

#[test]
fn group_rolls_back_when_final_parent_policy_denies_subclassing() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(parent)?.clone();
    let policy = iris_runtime::MetaCapabilities::denying(&[Capability::Subclass]);
    let module = registry.define_module_with_capabilities(&[], policy)?;
    registry.begin_transaction(parent)?;
    registry.recompose_candidate(parent, module, true)?;
    let class = registry.stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;

    // When
    let result = registry.commit_declaration_group();

    // Then
    assert!(matches!(
        result,
        Err(ClassError::MetaCapabilityDenied {
            operation: Capability::Subclass,
            ..
        })
    ));
    assert_eq!(registry.active(parent)?, &before);
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    Ok(())
}

#[test]
fn origin_publishes_once_when_policy_denies_reflective_member_changes() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let policy =
        iris_runtime::MetaCapabilities::denying(&[Capability::MethodSet, Capability::PropertySet]);
    let module = registry.define_module(&[])?;
    let class = registry.stage_class_origin(
        StaticSpine::new(2).with_meta_capabilities(policy),
        None,
        &[CompositionEdge::new(module, true)],
    )?;
    registry.publish_origin_method(
        class,
        Selector::new(1),
        MethodBody::new(2),
        Visibility::Public,
    )?;
    registry.stage_origin_property(class, Selector::new(3), MethodBody::new(4))?;

    // When
    registry.commit_declaration_group()?;

    // Then
    let origin = registry.active(class)?;
    assert_eq!(origin.number(), 1);
    assert_eq!(origin.id(), RevisionId::new(0));
    assert_eq!(origin.commit_id(), 1);
    assert_eq!(origin.meta_capabilities(), policy);
    assert!(origin.composition_edges()[0].private_access());
    Ok(())
}

#[test]
fn origin_rolls_back_when_an_open_candidate_has_an_invalid_decorator() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let existing = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(existing)?.clone();
    registry.begin_transaction(existing)?;
    registry.stage_decorators(existing, [DecoratorTransform::ChangePackageIdentity])?;
    let class = registry.stage_class_origin(StaticSpine::new(2), None, &[])?;

    // When
    let result = registry.commit_declaration_group();

    // Then
    assert_eq!(
        result,
        Err(ClassError::DecoratorViolation {
            class: existing,
            violation: DecoratorViolation::PackageIdentity
        })
    );
    assert_eq!(
        registry.class(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(registry.active(existing)?, &before);
    let later = registry.publish(registry.open(existing)?)?;
    assert_eq!(later.id().raw(), before.id().raw() + 1);
    assert_eq!(later.commit_id(), before.commit_id() + 1);
    Ok(())
}

#[test]
fn empty_declaration_commit_consumes_nothing_when_group_was_aborted() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;
    registry.roll_back_transaction(class);

    // When
    let result = registry.commit_declaration_group()?;

    // Then
    assert!(result.is_empty());
    assert!(registry.staged_origin(class).is_err());
    let next = registry.define_class(StaticSpine::new(2), None)?;
    assert_eq!(registry.active(next)?.id(), RevisionId::new(0));
    assert_eq!(registry.active(next)?.commit_id(), 1);
    Ok(())
}
