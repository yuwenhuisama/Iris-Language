use iris_runtime::{ClassError, ClassRegistry, ModuleId, RevisionId, StaticSpine};

fn origin(registry: &mut ClassRegistry) -> Result<iris_runtime::ClassId, ClassError> {
    registry.define_class(StaticSpine::new(1), None)
}

#[test]
fn reopening_class_publishes_new_active_revision_without_changing_class_identity()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;
    let before = registry.active_revision(class)?;

    // When
    let mut candidate = registry.open(class)?;
    candidate.add_module(ModuleId::new(8));
    registry.publish(candidate)?;

    // Then
    assert_eq!(registry.class(class)?.id(), class);
    assert_ne!(registry.active_revision(class)?, before);
    Ok(())
}

#[test]
fn origin_and_successive_publications_use_consecutive_per_class_revision_numbers()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;

    // When
    let first_number = registry.active(class)?.number();
    let second = registry.publish(registry.open(class)?)?;
    let third = registry.publish(registry.open(class)?)?;

    // Then
    assert_eq!(first_number, 1);
    assert_eq!(second.number(), 2);
    assert_eq!(third.number(), 3);
    Ok(())
}

#[test]
fn failed_candidate_does_not_consume_revision_number_or_alter_active_revision()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;
    let active = registry.active_revision(class)?;
    let mut candidate = registry.open(class)?;
    candidate.replace_static_spine(StaticSpine::new(2));

    // When
    let failure = registry.publish(candidate);
    let active_after_failure = registry.active_revision(class)?;
    let later = registry.publish(registry.open(class)?)?;

    // Then
    assert_eq!(failure, Err(ClassError::StaticSpineDowngrade { class }));
    assert_eq!(active_after_failure, active);
    assert_eq!(registry.active_revision(class)?, later.id());
    assert_ne!(registry.active_revision(class)?, active);
    assert_eq!(later.number(), 2);
    Ok(())
}

#[test]
fn transaction_group_shares_one_commit_id_and_groups_increase_strictly() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let first = origin(&mut registry)?;
    let second = origin(&mut registry)?;

    // When
    let first_group = registry.publish_group([registry.open(first)?, registry.open(second)?])?;
    let second_group = registry.publish_group([registry.open(first)?])?;

    // Then
    assert_eq!(first_group[0].commit_id(), first_group[1].commit_id());
    assert!(second_group[0].commit_id() > first_group[0].commit_id());
    Ok(())
}

#[test]
fn superseded_revision_remains_reachable_with_its_metadata() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;
    let old = registry.active_revision(class)?;
    let mut candidate = registry.open(class)?;
    candidate.add_module(ModuleId::new(13));

    // When
    registry.publish(candidate)?;

    // Then
    assert_eq!(registry.revision(old)?.modules(), &[]);
    assert_eq!(registry.active(class)?.modules(), &[ModuleId::new(13)]);
    Ok(())
}

#[test]
fn rollback_publishes_a_new_revision_instead_of_reactivating_history() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;
    let original = registry.active_revision(class)?;
    let updated = registry.publish(registry.open(class)?)?;

    // When
    let rollback = registry.rollback(class, original)?;

    // Then
    assert_ne!(rollback.id(), original);
    assert_ne!(rollback.id(), updated.id());
    assert_eq!(rollback.number(), 3);
    assert_eq!(registry.active_revision(class)?, rollback.id());
    Ok(())
}

#[test]
fn validation_failure_returns_a_typed_error_without_panicking() -> Result<(), ClassError> {
    // Given
    let registry = ClassRegistry::new();

    // When
    let result = registry.open(iris_runtime::ClassId::new(99));

    // Then
    assert!(matches!(
        result,
        Err(ClassError::UnknownClassId(id)) if id == iris_runtime::ClassId::new(99)
    ));
    Ok(())
}

#[test]
fn rollback_of_unknown_revision_returns_a_typed_error() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = origin(&mut registry)?;

    // When
    let result = registry.rollback(class, RevisionId::new(99));

    // Then
    assert_eq!(
        result,
        Err(ClassError::RevisionArtifactUnavailable(RevisionId::new(99)))
    );
    Ok(())
}
