use iris_runtime::{
    ClassError, ClassId, ConstructionError, MetaCapabilities, Runtime, Selector, StaticSpine, Value,
};

const SLOT: Selector = Selector::new(1);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn immutable_payload_publishes_when_declaration_group_commits() -> TestResult {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime
        .registry_mut()
        .declare_class_var(class, SLOT, false)?;
    assert_eq!(
        runtime.initialize_staged_class_var(class, SLOT, Value::Bool(true))?,
        Value::Bool(true)
    );
    assert_eq!(
        runtime.class_var(class, SLOT),
        Err(ConstructionError::Class(ClassError::UnknownClassId(class)))
    );
    assert!(runtime.assign_class_var(class, SLOT, Value::Nil).is_err());

    let revisions = runtime.commit_declaration_group()?;

    assert_eq!(
        (
            revisions[0].id().raw(),
            revisions[0].number(),
            revisions[0].commit_id()
        ),
        (0, 1, 1)
    );
    assert!(revisions[0].immutable_class_vars().contains(&SLOT));
    assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Bool(true)));
    assert_eq!(
        runtime.assign_class_var(class, SLOT, Value::Nil),
        Err(ConstructionError::ImmutableClassVariable { class, name: SLOT })
    );
    Ok(())
}

#[test]
fn nil_has_presence_when_mixed_group_commits() -> TestResult {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    let module = runtime
        .registry_mut()
        .stage_module_origin(&[], MetaCapabilities::all())?;
    runtime
        .registry_mut()
        .declare_class_var(class, SLOT, true)?;
    let absent = Selector::new(2);
    runtime
        .registry_mut()
        .declare_class_var(class, absent, false)?;
    runtime.initialize_staged_class_var(class, SLOT, Value::Nil)?;

    let receipt = runtime.commit_structural_group()?;

    assert_eq!(receipt.commit_id, Some(1));
    assert_eq!(
        receipt.classes[0].commit_id(),
        runtime.registry().active_module(module)?.commit_id()
    );
    assert!(!receipt.classes[0].immutable_class_vars().contains(&SLOT));
    assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Nil));
    assert_eq!(runtime.class_var(class, absent)?, None);
    assert_eq!(
        runtime.assign_class_var(class, SLOT, Value::Bool(true))?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn initialization_rejects_when_target_is_unknown_or_published() -> TestResult {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    runtime.declare_class_var(class, SLOT, Value::Bool(true), false)?;
    runtime.registry_mut().begin_transaction(class)?;

    for target in [class, ClassId::new(100)] {
        assert_eq!(
            runtime.initialize_staged_class_var(target, SLOT, Value::Nil),
            Err(ConstructionError::Class(ClassError::UnknownClassId(target)))
        );
    }

    assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Bool(true)));
    Ok(())
}

#[test]
fn initialization_requires_local_slot_when_ancestor_declares_same_name() -> TestResult {
    let mut runtime = Runtime::new();
    let parent = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    runtime.declare_class_var(parent, SLOT, Value::Bool(true), false)?;
    let class =
        runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;

    let result = runtime.initialize_staged_class_var(class, SLOT, Value::Nil);

    assert_eq!(
        result,
        Err(ConstructionError::MissingDeclaredClassVariable { class, name: SLOT })
    );
    runtime.commit_declaration_group()?;
    assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Bool(true)));
    assert_eq!(
        runtime.assign_class_var(class, SLOT, Value::Nil),
        Err(ConstructionError::ImmutableClassVariable {
            class: parent,
            name: SLOT
        })
    );
    Ok(())
}

#[test]
fn initialization_rejects_when_slot_is_undeclared() -> TestResult {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;

    let result = runtime.initialize_staged_class_var(class, SLOT, Value::Nil);

    assert_eq!(
        result,
        Err(ConstructionError::MissingDeclaredClassVariable { class, name: SLOT })
    );
    Ok(())
}

#[test]
fn initialization_rejects_when_payload_was_already_staged() -> TestResult {
    for mutable in [false, true] {
        let mut runtime = Runtime::new();
        let class = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(1), None, &[])?;
        runtime
            .registry_mut()
            .declare_class_var(class, SLOT, mutable)?;
        runtime.initialize_staged_class_var(class, SLOT, Value::Nil)?;

        let result = runtime.initialize_staged_class_var(class, SLOT, Value::Bool(true));

        assert_eq!(
            result,
            Err(ConstructionError::Class(
                ClassError::DuplicateClassVariable { class, name: SLOT }
            ))
        );
        runtime.commit_declaration_group()?;
        assert_eq!(runtime.class_var(class, SLOT)?, Some(Value::Nil));
    }
    Ok(())
}

#[test]
fn mutable_cell_uses_declaring_owner_when_assigned_through_child() -> TestResult {
    let mut runtime = Runtime::new();
    let parent = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    runtime
        .registry_mut()
        .declare_class_var(parent, SLOT, true)?;
    runtime.initialize_staged_class_var(parent, SLOT, Value::Bool(true))?;
    runtime.commit_declaration_group()?;
    let child =
        runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;
    runtime.commit_declaration_group()?;

    runtime.assign_class_var(child, SLOT, Value::Bool(false))?;

    assert_eq!(runtime.class_var(parent, SLOT)?, Some(Value::Bool(false)));
    assert_eq!(runtime.class_var(child, SLOT)?, Some(Value::Bool(false)));
    Ok(())
}

#[test]
fn local_cell_has_distinct_owner_when_shadowing_ancestor() -> TestResult {
    let mut runtime = Runtime::new();
    let parent = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    runtime.declare_class_var(parent, SLOT, Value::Bool(true), false)?;
    let child =
        runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(2), Some(parent), &[])?;
    runtime
        .registry_mut()
        .declare_class_var(child, SLOT, false)?;
    runtime.initialize_staged_class_var(child, SLOT, Value::Nil)?;

    runtime.commit_declaration_group()?;

    assert_eq!(runtime.class_var(parent, SLOT)?, Some(Value::Bool(true)));
    assert_eq!(runtime.class_var(child, SLOT)?, Some(Value::Nil));
    assert_eq!(
        runtime.assign_class_var(child, SLOT, Value::Bool(false)),
        Err(ConstructionError::ImmutableClassVariable {
            class: child,
            name: SLOT
        })
    );
    Ok(())
}
