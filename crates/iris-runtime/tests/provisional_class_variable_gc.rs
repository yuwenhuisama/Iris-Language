use iris_runtime::{
    ArrayRef, ClassError, CompositionEdge, DecoratorTransform, MetaCapabilities, Runtime, Selector,
    StaticSpine, Value,
};

const SLOT: Selector = Selector::new(1);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn transitive_payload_survives_when_staged_and_promoted() -> TestResult {
    for structural in [false, true] {
        let mut runtime = Runtime::new();
        let published = runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)?;
        let holder = runtime.allocate(published)?;
        let child = runtime.allocate(published)?;
        runtime.assign_raw_ivar(holder, SLOT, Value::Object(child))?;
        let class = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(class, SLOT, false)?;
        runtime.initialize_staged_class_var(
            class,
            SLOT,
            Value::Array(ArrayRef::new(vec![Value::Object(holder)])),
        )?;
        assert_eq!(runtime.collect_garbage([]).0, 0);

        if structural {
            runtime.commit_structural_group()?;
        } else {
            runtime.commit_declaration_group()?;
        }

        assert_eq!(runtime.collect_garbage([]).0, 0);
        assert_eq!(runtime.raw_ivar(holder, SLOT)?, Value::Object(child));
        assert_eq!(runtime.class_of(child)?, published);
    }
    Ok(())
}

#[test]
fn failed_group_releases_payload_when_published_writes_survive() -> TestResult {
    for structural in [false, true] {
        let mut runtime = Runtime::new();
        let published = runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)?;
        runtime.declare_class_var(published, SLOT, Value::Nil, true)?;
        let keep = runtime.allocate(published)?;
        let holder = runtime.allocate(published)?;
        let child = runtime.allocate(published)?;
        runtime.assign_raw_ivar(holder, SLOT, Value::Object(child))?;
        let before = runtime.registry().active(published)?.clone();
        runtime.registry_mut().begin_transaction(published)?;
        runtime.assign_class_var(published, SLOT, Value::Object(keep))?;
        let class = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(class, SLOT, false)?;
        runtime.initialize_staged_class_var(
            class,
            SLOT,
            Value::Array(ArrayRef::new(vec![Value::Object(holder)])),
        )?;
        runtime.assign_staged_class_raw_ivar(class, SLOT, Value::Object(child))?;
        runtime
            .registry_mut()
            .stage_decorators(class, [DecoratorTransform::ChangeNominalIdentity])?;

        if structural {
            assert!(runtime.commit_structural_group().is_err());
        } else {
            assert!(runtime.commit_declaration_group().is_err());
        }

        assert_eq!(runtime.registry().active(published)?, &before);
        assert_eq!(
            runtime.class_var(published, SLOT)?,
            Some(Value::Object(keep))
        );
        assert!(runtime.class_var(class, SLOT).is_err());
        assert_eq!(runtime.collect_garbage([]).0, 2);
        assert_eq!(runtime.class_of(keep)?, published);
        let next = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(3), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(next, SLOT, false)?;
        let revisions = runtime.commit_declaration_group()?;
        assert_eq!(revisions[0].id().raw(), before.id().raw() + 1);
        assert_eq!(revisions[0].commit_id(), before.commit_id() + 1);
        assert_eq!(runtime.class_var(next, SLOT)?, None);
    }
    Ok(())
}

#[test]
fn payload_is_discarded_when_module_validation_fails() -> TestResult {
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let object = runtime.allocate(published)?;
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime
        .registry_mut()
        .declare_class_var(class, SLOT, false)?;
    runtime.initialize_staged_class_var(class, SLOT, Value::Object(object))?;
    let module = runtime
        .registry_mut()
        .stage_module_origin(&[], MetaCapabilities::all())?;
    runtime
        .registry_mut()
        .stage_module_composition(module, &[CompositionEdge::new(module, false)])?;

    let result = runtime.commit_structural_group();

    assert!(result.is_err());
    assert!(runtime.class_var(class, SLOT).is_err());
    assert_eq!(runtime.collect_garbage([]).0, 1);
    let next = runtime
        .registry_mut()
        .stage_module_origin(&[], MetaCapabilities::all())?;
    let receipt = runtime.commit_structural_group()?;
    assert_eq!(receipt.commit_id, Some(2));
    assert_eq!(runtime.registry().active_module(next)?.id().raw(), 0);
    Ok(())
}

#[test]
fn payload_root_is_released_when_runtime_or_registry_rolls_back() -> TestResult {
    for direct in [false, true] {
        let mut runtime = Runtime::new();
        let published = runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)?;
        let object = runtime.allocate(published)?;
        let class = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(class, SLOT, false)?;
        runtime.initialize_staged_class_var(class, SLOT, Value::Object(object))?;

        if direct {
            runtime.registry_mut().roll_back_group();
        } else {
            runtime.roll_back_group();
        }

        assert_eq!(runtime.collect_garbage([]).0, 1);
        assert!(runtime.class_var(class, SLOT).is_err());
        assert!(runtime.commit_declaration_group()?.is_empty());
    }
    Ok(())
}

#[test]
fn stale_payload_is_rejected_when_registry_publishes_without_runtime() -> TestResult {
    for structural in [false, true] {
        let mut runtime = Runtime::new();
        let class = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(1), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(class, SLOT, true)?;
        runtime.initialize_staged_class_var(class, SLOT, Value::Bool(true))?;
        runtime.registry_mut().commit_declaration_group()?;
        runtime.assign_class_var(class, SLOT, Value::Bool(false))?;
        runtime.registry_mut().begin_transaction(class)?;

        if structural {
            assert_eq!(
                runtime.commit_structural_group(),
                Err(ClassError::UnknownClassId(class).into())
            );
        } else {
            assert_eq!(
                runtime.commit_declaration_group(),
                Err(ClassError::UnknownClassId(class))
            );
        }

        assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Bool(false)));
        assert_eq!(runtime.registry().active(class)?.number(), 1);
        assert!(!runtime.registry().is_staging(class));
    }
    Ok(())
}

#[test]
fn combined_declaration_discards_payload_when_origin_rolls_back() -> TestResult {
    let mut runtime = Runtime::new();
    let published = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let object = runtime.allocate(published)?;
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])?;
    runtime.declare_class_var(class, SLOT, Value::Object(object), false)?;

    runtime.roll_back_group();

    assert_eq!(runtime.collect_garbage([]).0, 1);
    Ok(())
}
