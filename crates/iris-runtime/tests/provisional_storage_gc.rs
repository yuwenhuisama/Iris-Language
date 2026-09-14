use iris_runtime::{
    ArrayRef, ClassError, ConstructionError, DecoratorTransform, Runtime, Selector, StaticSpine,
    Value,
};

const SLOT: Selector = Selector::new(1);

#[test]
fn transitive_objects_survive_when_only_provisional_storage_roots_them()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let holder = runtime.allocate(published)?;
    let child = runtime.allocate(published)?;
    runtime.assign_raw_ivar(holder, SLOT, Value::Object(child))?;
    let origin = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime.assign_staged_class_raw_ivar(
        origin,
        SLOT,
        Value::Array(ArrayRef::new(vec![Value::Object(holder)])),
    )?;

    // When
    let (freed, _) = runtime.collect_garbage([]);

    // Then
    assert_eq!(freed, 0);
    assert_eq!(runtime.class_of(holder)?, published);
    assert_eq!(runtime.class_of(child)?, published);
    Ok(())
}

#[test]
fn promoted_storage_remains_a_root_when_metadata_commits() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let object = runtime.allocate(published)?;
    let origin = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime.assign_staged_class_raw_ivar(origin, SLOT, Value::Object(object))?;

    // When
    runtime.commit_declaration_group()?;

    // Then
    assert_eq!(runtime.class_raw_ivar(origin, SLOT)?, Value::Object(object));
    assert_eq!(runtime.collect_garbage([]).0, 0);
    assert_eq!(runtime.class_of(object)?, published);
    Ok(())
}

#[test]
fn provisional_roots_are_released_when_group_rolls_back() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let object = runtime.allocate(published)?;
    let origin = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime.assign_staged_class_raw_ivar(origin, SLOT, Value::Object(object))?;

    // When
    runtime.roll_back_group();

    // Then
    assert_eq!(runtime.collect_garbage([]).0, 1);
    assert!(runtime.class_of(object).is_err());
    assert_eq!(
        runtime.registry().class(origin),
        Err(ClassError::UnknownClassId(origin))
    );
    assert!(runtime.commit_declaration_group()?.is_empty());
    Ok(())
}

#[test]
fn failed_group_drops_origin_roots_but_preserves_published_writes() -> Result<(), ConstructionError>
{
    // Given
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let keep = runtime.allocate(published)?;
    let holder = runtime.allocate(published)?;
    let child = runtime.allocate(published)?;
    runtime.assign_raw_ivar(holder, SLOT, Value::Object(child))?;
    runtime.registry_mut().begin_transaction(published)?;
    runtime.assign_class_raw_ivar(published, SLOT, Value::Object(keep))?;
    runtime
        .registry_mut()
        .stage_decorators(published, [DecoratorTransform::ChangePackageIdentity])?;
    let origin = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime.assign_staged_class_raw_ivar(
        origin,
        SLOT,
        Value::Array(ArrayRef::new(vec![Value::Object(holder)])),
    )?;
    let before = runtime.registry().active(published)?.clone();

    // When
    let result = runtime.commit_declaration_group();

    // Then
    assert!(
        matches!(result, Err(ClassError::DecoratorViolation { class, .. }) if class == published)
    );
    assert_eq!(runtime.registry().active(published)?, &before);
    assert_eq!(
        runtime.class_raw_ivar(published, SLOT)?,
        Value::Object(keep)
    );
    assert_eq!(runtime.collect_garbage([]).0, 2);
    assert_eq!(runtime.class_of(keep)?, published);
    let next = runtime
        .registry_mut()
        .define_class(StaticSpine::new(3), None)?;
    assert_eq!(
        runtime.registry().active(next)?.id().raw(),
        before.id().raw() + 1
    );
    assert_eq!(
        runtime.registry().active(next)?.commit_id(),
        before.commit_id() + 1
    );
    Ok(())
}
