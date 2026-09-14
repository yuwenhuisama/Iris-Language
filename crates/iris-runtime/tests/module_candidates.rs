use iris_runtime::{
    Capability, ClassRegistry, CompositionEdge, DecoratorTransform, DispatchContext, DispatchError,
    DispatchOutcome, MetaCapabilities, MethodBody, ModuleCandidateError, ModuleMethodDefinition,
    RuntimeStructuralError, Selector, StaticSpine, Visibility,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn definition(selector: u64, visibility: Visibility) -> ModuleMethodDefinition {
    ModuleMethodDefinition::new(Selector::new(selector), MethodBody::new(10), visibility)
}

#[test]
fn origin_is_hidden_when_staged() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let method = registry.stage_module_origin_method(module, definition(1, Visibility::Public))?;
    assert!(registry.active_module(module).is_err());
    assert_eq!(registry.module_method(module, Selector::new(1)), None);
    assert_eq!(
        registry.staged_module(module)?.method(Selector::new(1)),
        Some(method)
    );
    let receipt = registry.commit_structural_group()?;
    assert_eq!(receipt.modules[0].owner(), module);
    assert_eq!(receipt.modules[0].number(), 1);
    assert_eq!(receipt.commit_id, Some(1));
    assert_eq!(
        registry.module_method(module, Selector::new(1)),
        Some(method)
    );
    Ok(())
}

#[test]
fn checked_addition_is_private_and_collision_free() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    assert!(matches!(
        registry.stage_module_method(module, definition(1, Visibility::Public)),
        Err(ModuleCandidateError::PublicAdditionDenied { .. })
    ));
    let method = registry.stage_module_method(module, definition(1, Visibility::Private))?;
    assert_eq!(method.visibility(), Visibility::Private);
    assert!(matches!(
        registry.stage_module_method(module, definition(1, Visibility::Private)),
        Err(ModuleCandidateError::MethodCollision { .. })
    ));
    Ok(())
}

#[test]
fn mutation_is_denied_when_module_policy_denies_capability() -> TestResult {
    for operation in [Capability::MethodSet, Capability::MethodBody] {
        let mut registry = ClassRegistry::new();
        let module = registry.stage_module_origin(&[], MetaCapabilities::denying(&[operation]))?;
        registry.stage_module_origin_method(module, definition(1, Visibility::Public))?;
        let result = match operation {
            Capability::MethodSet => {
                registry.stage_module_method(module, definition(2, Visibility::Private))
            }
            Capability::MethodBody => {
                registry.stage_module_method_body(module, Selector::new(1), MethodBody::new(20))
            }
            _ => unreachable!(),
        };
        assert_eq!(
            result,
            Err(ModuleCandidateError::CapabilityDenied { module, operation })
        );
    }
    Ok(())
}

#[test]
fn mixed_group_rolls_back_when_module_composition_is_invalid() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let before = registry.active(class)?.clone();
    registry.begin_transaction(class)?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.stage_module_composition(module, &[CompositionEdge::new(module, false)])?;
    assert!(matches!(registry.commit_structural_group(),
        Err(RuntimeStructuralError::Module(ModuleCandidateError::CompositionCycle(found))) if found == module));
    assert_eq!(registry.active(class)?, &before);
    assert!(registry.staged_module(module).is_err());
    registry.begin_transaction(class)?;
    let next = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let receipt = registry.commit_structural_group()?;
    assert_eq!(receipt.commit_id, Some(before.commit_id() + 1));
    assert_eq!(receipt.classes[0].id().raw(), before.id().raw() + 1);
    assert_eq!(registry.active_module(next)?.id().raw(), 0);
    assert_eq!(
        receipt.classes[0].commit_id(),
        receipt.modules[0].commit_id()
    );
    Ok(())
}

#[test]
fn module_origin_rolls_back_when_class_validation_fails() -> TestResult {
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;
    registry.stage_decorators(class, [DecoratorTransform::ChangeNominalIdentity])?;
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    assert!(matches!(
        registry.commit_structural_group(),
        Err(RuntimeStructuralError::Class(_))
    ));
    assert!(registry.active_module(module).is_err());
    assert!(registry.staged_module(module).is_err());
    let next = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let receipt = registry.commit_structural_group()?;
    assert_eq!(receipt.commit_id, Some(1));
    assert_eq!(registry.active_module(next)?.id().raw(), 0);
    Ok(())
}

#[test]
fn old_method_survives_when_body_replacement_publishes() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    let old = registry.define_module_method(
        module,
        Selector::new(1),
        MethodBody::new(10),
        Visibility::Public,
    )?;
    let class = registry.define_class_with_capabilities_and_modules(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
        &[module],
    )?;
    let before = registry.active_module(module)?.clone();
    registry.begin_module_transaction(module)?;
    let new = registry.stage_module_method_body(module, Selector::new(1), MethodBody::new(20))?;
    assert_eq!(registry.module_method(module, Selector::new(1)), Some(old));
    registry.commit_structural_group()?;
    assert_ne!(new.id(), old.id());
    assert_eq!(new.visibility(), old.visibility());
    assert_eq!(registry.method_by_id(old.id()), Some(old));
    assert_eq!(registry.module_revision(before.id())?, &before);
    assert_eq!(
        registry.dispatch(class, Selector::new(1)),
        Ok(DispatchOutcome::Invoke(new))
    );
    assert_eq!(
        registry.invoke_reflective(class, old, |method| Ok(method.body().raw())),
        Ok(10)
    );
    Ok(())
}

#[test]
fn module_mutation_never_grants_host_private_access() -> TestResult {
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    let class = registry.define_class_with_capabilities_and_modules(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
        &[module],
    )?;
    registry.publish_method(
        class,
        Selector::new(1),
        MethodBody::new(10),
        Visibility::Private,
    )?;
    registry.begin_module_transaction(module)?;
    registry.stage_module_method(module, definition(2, Visibility::Private))?;
    registry.commit_structural_group()?;
    assert_eq!(
        registry.dispatch_with_context(
            class,
            Selector::new(1),
            DispatchContext::module_implementation(module, true)
        ),
        Err(DispatchError::VisibilityDenied {
            selector: Selector::new(1)
        })
    );
    Ok(())
}
