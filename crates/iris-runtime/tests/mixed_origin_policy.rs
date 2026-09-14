use iris_runtime::{
    Capability, ClassError, ClassRegistry, CompositionEdge, DispatchContext, DispatchError,
    DispatchOutcome, MetaCapabilities, MethodBody, ModuleId, ModuleMethodDefinition,
    RuntimeStructuralError, Selector, StaticSpine, Visibility,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn effective_policy_is_final_when_origin_module_components_change() -> TestResult {
    let mut registry = ClassRegistry::new();
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::Native]))?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, false)],
    )?;
    registry.stage_module_composition(module, &[CompositionEdge::new(denial, false)])?;

    let receipt = registry.commit_structural_group()?;

    assert_eq!(
        receipt.classes[0].meta_capabilities(),
        MetaCapabilities::denying(&[Capability::Native])
    );
    assert_eq!(
        registry.active_module(module)?.meta_capabilities(),
        receipt.classes[0].meta_capabilities()
    );
    assert_eq!(
        registry.active(class)?.commit_id(),
        registry.active_module(module)?.commit_id()
    );
    Ok(())
}

#[test]
fn provisional_policy_is_recomputed_when_denying_component_is_removed() -> TestResult {
    let mut registry = ClassRegistry::new();
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodSet]))?;
    let module = registry.stage_module_origin(
        &[CompositionEdge::new(denial, false)],
        MetaCapabilities::all(),
    )?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, false)],
    )?;
    registry.stage_module_composition(module, &[])?;

    registry.publish_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Private,
    )?;
    registry.commit_structural_group()?;

    assert!(
        registry
            .active(class)?
            .meta_capabilities()
            .allows(Capability::MethodSet)
    );
    Ok(())
}

#[test]
fn origin_group_aborts_when_late_policy_denies_a_body_replacement() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, false)],
    )?;
    registry.publish_origin_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    registry.publish_candidate_decorated_method(
        class,
        Selector::new(1),
        MethodBody::new(2),
        Visibility::Public,
        [],
    )?;
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodBody]))?;
    registry.stage_module_composition(module, &[CompositionEdge::new(denial, false)])?;

    let result = registry.commit_structural_group();

    assert!(matches!(
        result,
        Err(RuntimeStructuralError::Class(
            ClassError::MetaCapabilityDenied {
                operation: Capability::MethodBody,
                ..
            }
        ))
    ));
    assert!(registry.active(class).is_err());
    assert!(registry.staged_origin(class).is_err());
    assert!(registry.active_module(module).is_err());
    assert!(registry.staged_module(denial).is_err());
    assert_eq!(registry.commit_structural_group()?.commit_id, None);
    Ok(())
}

#[test]
fn existing_host_dispatch_is_active_only_when_module_origin_is_composed() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.begin_transaction(class)?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let selector = Selector::new(1);
    let method = registry.stage_module_origin_method(
        module,
        ModuleMethodDefinition::new(selector, MethodBody::new(42), Visibility::Public),
    )?;
    registry.recompose_candidate(class, module, true)?;
    assert_eq!(
        registry.dispatch(class, selector),
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    );
    assert!(
        registry
            .invoke_reflective(class, method, |_| Ok(()))
            .is_err()
    );
    assert!(registry.active(class)?.modules().is_empty());

    registry.commit_structural_group()?;

    assert_eq!(
        registry.dispatch(class, selector),
        Ok(DispatchOutcome::Invoke(method))
    );
    Ok(())
}

#[test]
fn private_edge_grants_authority_only_when_mixed_group_commits() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, true)],
    )?;
    let selector = Selector::new(1);
    let method = registry.publish_origin_method(
        class,
        selector,
        MethodBody::new(42),
        Visibility::Private,
    )?;
    let context = DispatchContext::module_implementation(module, true);
    assert!(
        registry
            .dispatch_with_context(class, selector, context)
            .is_err()
    );

    registry.commit_structural_group()?;

    assert_eq!(
        registry.dispatch_with_context(class, selector, context),
        Ok(DispatchOutcome::Invoke(method))
    );
    assert_eq!(
        registry.dispatch(class, selector),
        Err(DispatchError::VisibilityDenied { selector })
    );
    Ok(())
}

#[test]
fn provisional_parent_remains_rejected_when_module_overlay_is_available() -> TestResult {
    let mut registry = ClassRegistry::new();
    let parent = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;

    let result = registry.stage_class_origin(StaticSpine::new(2), Some(parent), &[]);

    assert_eq!(result, Err(ClassError::UnknownClassId(parent)));
    registry.roll_back_group();
    Ok(())
}

#[test]
fn mixed_group_rejects_unknown_class_edge_when_no_module_candidate_exists() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(class)?.clone();
    registry.begin_transaction(class)?;
    let unknown = ModuleId::new(999);
    registry.recompose_candidate(class, unknown, true)?;

    let result = registry.commit_structural_group();

    assert_eq!(
        result,
        Err(RuntimeStructuralError::Class(ClassError::UnknownModuleId(
            unknown
        )))
    );
    assert_eq!(registry.active(class)?, &before);
    assert!(!registry.is_staging(class));
    Ok(())
}
