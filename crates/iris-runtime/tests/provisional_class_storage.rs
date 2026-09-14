use iris_runtime::{
    ClassError, ClassId, ConstructionError, DecoratorTransform, DecoratorViolation, RevisionId,
    Runtime, Selector, StaticSpine, Value,
};

const SLOT: Selector = Selector::new(1);

#[test]
fn backend_reads_own_write_when_origin_is_unpublished() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;

    // When
    let stored = runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Bool(true))?;

    // Then
    assert_eq!(stored, Value::Bool(true));
    assert_eq!(
        runtime.staged_class_raw_ivar(class, SLOT)?,
        Some(Value::Bool(true))
    );
    Ok(())
}

#[test]
fn public_storage_rejects_target_when_origin_has_staged_slots() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Bool(true))?;
    let error = ConstructionError::Class(ClassError::UnknownClassId(class));

    // When / Then
    assert_eq!(runtime.class_raw_ivar(class, SLOT), Err(error.clone()));
    assert_eq!(
        runtime.assign_class_raw_ivar(class, SLOT, Value::Nil),
        Err(error.clone())
    );
    assert_eq!(runtime.class_raw_ivar_names(class), Err(error.clone()));
    assert_eq!(
        runtime.remove_class_raw_ivar(class, SLOT),
        Err(error.clone())
    );
    assert_eq!(runtime.allocate(class), Err(error));
    Ok(())
}

#[test]
fn nil_is_materialized_when_origin_storage_commits() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    let absent = Selector::new(2);
    assert_eq!(runtime.staged_class_raw_ivar(class, SLOT)?, None);
    runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Nil)?;
    assert_eq!(
        runtime.staged_class_raw_ivar(class, SLOT)?,
        Some(Value::Nil)
    );
    assert_eq!(runtime.staged_class_raw_ivar(class, absent)?, None);

    // When
    let revisions = runtime.commit_declaration_group()?;

    // Then
    assert_eq!(revisions.len(), 1);
    assert_eq!(
        (
            revisions[0].id(),
            revisions[0].number(),
            revisions[0].commit_id()
        ),
        (RevisionId::new(0), 1, 1)
    );
    assert_eq!(runtime.class_raw_ivar(class, SLOT)?, Value::Nil);
    assert_eq!(runtime.class_raw_ivar_names(class)?, vec![SLOT]);
    assert_eq!(
        runtime.staged_class_raw_ivar(class, SLOT),
        Err(ConstructionError::Class(ClassError::UnknownClassId(class)))
    );
    Ok(())
}

#[test]
fn storage_is_discarded_when_metadata_validation_fails() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Bool(true))?;
    runtime
        .registry_mut()
        .stage_decorators(class, [DecoratorTransform::ChangeNominalIdentity])?;

    // When
    let result = runtime.commit_declaration_group();

    // Then
    assert_eq!(
        result,
        Err(ClassError::DecoratorViolation {
            class,
            violation: DecoratorViolation::NominalIdentity
        })
    );
    assert_eq!(
        runtime.registry().class(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        runtime.staged_class_raw_ivar(class, SLOT),
        Err(ConstructionError::Class(ClassError::UnknownClassId(class)))
    );
    let next = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    let revisions = runtime.commit_declaration_group()?;
    assert_eq!(
        (revisions[0].id(), revisions[0].commit_id()),
        (RevisionId::new(0), 1)
    );
    assert!(runtime.class_raw_ivar_names(next)?.is_empty());
    Ok(())
}

#[test]
fn staged_storage_rejects_published_and_unknown_targets() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    runtime.registry_mut().begin_transaction(published)?;

    // When / Then
    for class in [published, ClassId::new(100)] {
        let error = ConstructionError::Class(ClassError::UnknownClassId(class));
        assert_eq!(
            runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Nil),
            Err(error.clone())
        );
        assert_eq!(runtime.staged_class_raw_ivar(class, SLOT), Err(error));
    }
    Ok(())
}

#[test]
fn wrong_commit_route_never_promotes_stale_storage() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Bool(true))?;
    runtime.registry_mut().commit_declaration_group()?;
    runtime.assign_class_raw_ivar(class, SLOT, Value::Bool(false))?;
    runtime.registry_mut().begin_transaction(class)?;

    // When
    let result = runtime.commit_declaration_group();

    // Then
    assert_eq!(result, Err(ClassError::UnknownClassId(class)));
    assert_eq!(runtime.class_raw_ivar(class, SLOT)?, Value::Bool(false));
    assert_eq!(runtime.registry().active(class)?.number(), 1);
    assert!(!runtime.registry().is_staging(class));
    Ok(())
}
