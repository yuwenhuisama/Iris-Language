use iris_runtime::{
    Capability, ClassRegistry, CompositionEdge, MetaCapabilities, MethodBody, ModuleCandidateError,
    ModuleMethodDefinition, Runtime, RuntimeStructuralError, Selector, StaticSpine, Value,
    Visibility,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn whole_group_is_discarded_when_explicitly_aborted() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.roll_back_group();
    assert!(registry.staged_module(module).is_err());
    assert_eq!(registry.commit_structural_group()?.commit_id, None);
    Ok(())
}

#[test]
fn publication_rechecks_authority_when_later_composition_denies_it() -> TestResult {
    let mut registry = ClassRegistry::new();
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodSet]))?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.stage_module_method(
        module,
        ModuleMethodDefinition::new(Selector::new(1), MethodBody::new(1), Visibility::Private),
    )?;
    registry.stage_module_composition(module, &[CompositionEdge::new(denial, false)])?;
    assert_eq!(
        registry.commit_structural_group(),
        Err(RuntimeStructuralError::Module(
            ModuleCandidateError::CapabilityDenied {
                module,
                operation: Capability::MethodSet
            }
        ))
    );
    Ok(())
}

#[test]
fn publication_rechecks_collisions_when_composition_changes_after_addition() -> TestResult {
    let mut registry = ClassRegistry::new();
    let component = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let definition =
        ModuleMethodDefinition::new(Selector::new(1), MethodBody::new(1), Visibility::Private);
    registry.stage_module_origin_method(component, definition)?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.stage_module_method(module, definition)?;
    registry.stage_module_composition(module, &[CompositionEdge::new(component, false)])?;
    assert_eq!(
        registry.commit_structural_group(),
        Err(RuntimeStructuralError::Module(
            ModuleCandidateError::MethodCollision {
                module,
                selector: Selector::new(1)
            }
        ))
    );
    Ok(())
}

#[test]
fn old_candidate_conflicts_when_legacy_api_changes_active_module() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    registry.begin_module_transaction(module)?;
    let replacement = registry.define_module_method(
        module,
        Selector::new(1),
        MethodBody::new(2),
        Visibility::Public,
    )?;
    assert_eq!(
        registry.commit_structural_group(),
        Err(RuntimeStructuralError::Module(
            ModuleCandidateError::Conflict(module)
        ))
    );
    assert_eq!(
        registry.module_method(module, Selector::new(1)),
        Some(replacement)
    );
    Ok(())
}

#[test]
fn existing_module_composition_and_policy_are_frozen() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    registry.begin_module_transaction(module)?;
    assert_eq!(
        registry.stage_module_composition(module, &[]),
        Err(ModuleCandidateError::CompositionFrozen(module))
    );
    assert_eq!(
        registry.stage_module_policy(module, MetaCapabilities::all()),
        Err(ModuleCandidateError::PolicyFrozen(module))
    );
    Ok(())
}

#[test]
fn legacy_module_policy_stays_frozen_when_body_only_transaction_publishes() -> TestResult {
    let mut registry = ClassRegistry::new();
    let component = registry.define_module_with_capabilities(
        &[],
        MetaCapabilities::denying(&[Capability::MethodSet]),
    )?;
    let module = registry.define_module(&[component])?;
    let before = registry.active_module(module)?.meta_capabilities();
    registry.begin_module_transaction(module)?;
    registry.commit_structural_group()?;
    assert_eq!(registry.active_module(module)?.meta_capabilities(), before);
    Ok(())
}

#[test]
fn staged_component_policy_is_inherited_when_origin_group_publishes() -> TestResult {
    let mut registry = ClassRegistry::new();
    let component =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodBody]))?;
    let module = registry.stage_module_origin(
        &[CompositionEdge::new(component, false)],
        MetaCapabilities::all(),
    )?;
    registry.commit_structural_group()?;
    assert!(
        !registry
            .active_module(module)?
            .meta_capabilities()
            .allows(Capability::MethodBody)
    );
    assert_eq!(registry.module_components(module), vec![component]);
    Ok(())
}

#[test]
fn runtime_storage_joins_mixed_metadata_commit() -> TestResult {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(1), None, &[])?;
    let module = runtime
        .registry_mut()
        .stage_module_origin(&[], MetaCapabilities::all())?;
    runtime.assign_staged_class_raw_ivar(class, Selector::new(1), Value::Nil)?;
    let receipt = runtime.commit_structural_group()?;
    assert_eq!(
        receipt.classes[0].commit_id(),
        receipt.modules[0].commit_id()
    );
    assert_eq!(receipt.modules[0].owner(), module);
    assert!(
        runtime
            .staged_class_raw_ivar(class, Selector::new(1))
            .is_err()
    );
    assert_eq!(runtime.class_raw_ivar_names(class)?, vec![Selector::new(1)]);
    assert_eq!(runtime.class_raw_ivar(class, Selector::new(1))?, Value::Nil);
    Ok(())
}

#[test]
fn provisional_module_class_overlay_publishes_when_group_is_valid() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(class)?.clone();
    registry.begin_transaction(class)?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.recompose_candidate(class, module, true)?;
    let receipt = registry.commit_structural_group()?;
    assert_eq!(registry.active(class)?.number(), before.number() + 1);
    assert_eq!(registry.active(class)?.modules(), &[module]);
    assert_eq!(
        receipt.classes[0].commit_id(),
        receipt.modules[0].commit_id()
    );
    Ok(())
}
