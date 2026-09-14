use iris_runtime::{
    ClassRegistry, CompositionEdge, DispatchOutcome, MetaCapabilities, MethodBody,
    ModuleCandidateError, ModuleMethodDefinition, RuntimeStructuralError, Selector, StaticSpine,
    Visibility,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = ClassRegistry::new();
    let selector = Selector::new(1);
    let module = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    let original = registry.stage_module_origin_method(
        module,
        ModuleMethodDefinition::new(selector, MethodBody::new(10), Visibility::Public),
    )?;
    assert!(registry.module_method(module, selector).is_none());
    let host = registry.stage_class_origin(
        StaticSpine::new(1),
        None,
        &[CompositionEdge::new(module, false)],
    )?;
    assert!(registry.dispatch(host, selector).is_err());
    let origins = registry.commit_structural_group()?;
    assert_eq!(
        origins.classes[0].commit_id(),
        origins.modules[0].commit_id()
    );
    assert_eq!(
        registry.dispatch(host, selector),
        Ok(DispatchOutcome::Invoke(original))
    );
    registry.begin_transaction(host)?;
    registry.begin_module_transaction(module)?;
    let replacement = registry.stage_module_method_body(module, selector, MethodBody::new(20))?;
    assert_eq!(registry.module_method(module, selector), Some(original));
    let receipt = registry.commit_structural_group()?;
    assert_eq!(
        receipt.classes[0].commit_id(),
        receipt.modules[0].commit_id()
    );
    assert_eq!(
        registry.dispatch(host, selector),
        Ok(DispatchOutcome::Invoke(replacement))
    );
    let old_output = registry
        .invoke_reflective(host, original, |method| Ok(method.body().raw()))
        .map_err(|error| format!("{error:?}"))?;
    let new_output = registry
        .invoke_reflective(host, replacement, |method| Ok(method.body().raw()))
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!((old_output, new_output), (10, 20));
    registry.begin_transaction(host)?;
    let invalid = registry.stage_module_origin(&[], MetaCapabilities::all())?;
    registry.stage_module_composition(invalid, &[CompositionEdge::new(invalid, false)])?;
    assert_eq!(
        registry.commit_structural_group(),
        Err(RuntimeStructuralError::Module(
            ModuleCandidateError::CompositionCycle(invalid)
        ))
    );
    assert_eq!(registry.module_method(module, selector), Some(replacement));
    assert!(registry.staged_module(invalid).is_err());
    println!(
        "module candidate probe: hidden origin, mixed commit, retained {old_output}, fresh {new_output}, atomic rollback passed"
    );
    Ok(())
}
