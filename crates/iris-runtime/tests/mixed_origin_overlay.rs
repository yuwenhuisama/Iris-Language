use iris_runtime::{
    Capability, ClassError, ClassRegistry, CompositionEdge, DispatchOutcome, MetaCapabilities,
    MethodBody, ModuleCandidateError, ModuleId, ModuleMethodDefinition, MroEntry,
    RuntimeStructuralError, Selector, StaticSpine, Visibility,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn class_origin_composes_when_module_origin_is_unpublished() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let selector = Selector::new(1);
    let method = registry.stage_module_origin_method(
        module,
        ModuleMethodDefinition::new(selector, MethodBody::new(42), Visibility::Public),
    )?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, true)],
    )?;
    assert!(registry.active(class).is_err());
    assert!(registry.module_method(module, selector).is_none());
    assert!(registry.dispatch(class, selector).is_err());

    let receipt = registry.commit_structural_group()?;

    assert_eq!(
        receipt.classes[0].commit_id(),
        receipt.modules[0].commit_id()
    );
    assert_eq!(
        registry.active(class)?.mro(),
        &[MroEntry::Class(class), MroEntry::Module(module)]
    );
    assert_eq!(
        registry.dispatch(class, selector),
        Ok(DispatchOutcome::Invoke(method))
    );
    Ok(())
}

#[test]
fn final_mro_orders_and_deduplicates_when_nested_edges_change_after_class_staging() -> TestResult {
    let mut registry = ClassRegistry::new();
    let active = registry.define_module(&[])?;
    let left = registry.stage_module_origin(
        &[CompositionEdge::new(active, false)],
        MetaCapabilities::all(),
    )?;
    let right = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[
            CompositionEdge::new(left, false),
            CompositionEdge::new(right, false),
        ],
    )?;
    registry.stage_module_composition(
        right,
        &[
            CompositionEdge::new(left, false),
            CompositionEdge::new(active, false),
        ],
    )?;

    registry.commit_structural_group()?;

    assert_eq!(
        registry.active(class)?.mro(),
        &[
            MroEntry::Class(class),
            MroEntry::Module(right),
            MroEntry::Module(active),
            MroEntry::Module(left),
        ]
    );
    Ok(())
}

#[test]
fn checked_mutation_is_denied_when_staged_transitive_module_denies_it() -> TestResult {
    let mut registry = ClassRegistry::new();
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodSet]))?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let class = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, false)],
    )?;
    registry.stage_module_composition(module, &[CompositionEdge::new(denial, false)])?;

    let result = registry.publish_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Private,
    );

    assert!(matches!(
        result,
        Err(ClassError::MetaCapabilityDenied {
            operation: Capability::MethodSet,
            ..
        })
    ));
    registry.roll_back_group();
    assert_eq!(registry.commit_structural_group()?.commit_id, None);
    Ok(())
}

#[test]
fn entire_group_aborts_when_late_module_policy_denies_checked_class_mutation() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(class)?.clone();
    registry.begin_transaction(class)?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.recompose_candidate(class, module, true)?;
    registry.publish_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Private,
    )?;
    let denial =
        registry.stage_module_origin(&[], MetaCapabilities::denying(&[Capability::MethodSet]))?;
    registry.stage_module_composition(module, &[CompositionEdge::new(denial, false)])?;

    let result = registry.commit_structural_group();

    assert!(matches!(
        result,
        Err(RuntimeStructuralError::Class(
            ClassError::MetaCapabilityDenied {
                operation: Capability::MethodSet,
                ..
            }
        ))
    ));
    assert_eq!(registry.active(class)?, &before);
    assert!(!registry.is_staging(class));
    assert!(registry.active_module(module).is_err());
    assert!(registry.staged_module(denial).is_err());
    registry.begin_transaction(class)?;
    let next_module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let receipt = registry.commit_structural_group()?;
    assert_eq!(receipt.commit_id, Some(before.commit_id() + 1));
    assert_eq!(receipt.classes[0].id().raw(), before.id().raw() + 1);
    assert_eq!(registry.active_module(next_module)?.id().raw(), 0);
    Ok(())
}

#[test]
fn entire_group_aborts_when_late_module_graph_is_invalid() -> TestResult {
    for unknown in [false, true] {
        let mut registry = ClassRegistry::new();
        let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
        let class = registry.stage_class_origin(
            StaticSpine::new(1),
            None,
            &[CompositionEdge::new(module, false)],
        )?;
        let target = if unknown { ModuleId::new(999) } else { module };
        registry.stage_module_composition(module, &[CompositionEdge::new(target, false)])?;

        let result = registry.commit_structural_group();

        let error = if unknown {
            ModuleCandidateError::UnknownModule(target)
        } else {
            ModuleCandidateError::CompositionCycle(module)
        };
        assert_eq!(result, Err(RuntimeStructuralError::Module(error)));
        assert!(registry.active(class).is_err());
        assert!(registry.staged_origin(class).is_err());
        assert!(registry.active_module(module).is_err());
        assert!(registry.staged_module(module).is_err());
        let next = registry.stage_class_origin(StaticSpine::new(2), None, &[])?;
        registry.stage_module_origin(&[], MetaCapabilities::all())?;
        let receipt = registry.commit_structural_group()?;
        assert_eq!(receipt.commit_id, Some(1));
        assert_eq!(registry.active(next)?.id().raw(), 0);
        assert_eq!(receipt.modules[0].id().raw(), 0);
    }
    Ok(())
}
