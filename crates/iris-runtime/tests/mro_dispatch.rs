use iris_runtime::{
    ClassError, ClassRegistry, DispatchError, DispatchOutcome, MethodBody, ModuleId, Selector,
    StaticSpine, Visibility,
};

fn define_class(
    registry: &mut ClassRegistry,
    superclass: Option<iris_runtime::ClassId>,
) -> Result<iris_runtime::ClassId, ClassError> {
    registry.define_class(StaticSpine::new(1), superclass)
}

#[test]
fn mixin_a_b_yields_class_b_a_super_lookup_order() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let superclass = define_class(&mut registry, None)?;
    let module_a = registry.define_module(&[])?;
    let module_b = registry.define_module(&[])?;
    let class = define_class(&mut registry, Some(superclass))?;
    let mut candidate = registry.open(class)?;
    candidate.add_module(module_a);
    candidate.add_module(module_b);

    // When
    registry.publish(candidate)?;

    // Then: Header `mixin A, B` MUST be applied left-to-right and produce lookup order `Class, B, A, Super...` before nested expansion.
    assert_eq!(
        registry.active(class)?.mro(),
        &[
            class.into(),
            module_b.into(),
            module_a.into(),
            superclass.into(),
        ]
    );
    Ok(())
}

#[test]
fn nested_module_composition_deduplicates_at_closest_class_occurrence() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module_a = registry.define_module(&[])?;
    let module_b = registry.define_module(&[module_a])?;
    let class = define_class(&mut registry, None)?;
    let mut candidate = registry.open(class)?;
    candidate.add_module(module_a);
    candidate.add_module(module_b);

    // When
    registry.publish(candidate)?;

    // Then
    assert_eq!(
        registry.active(class)?.mro(),
        &[class.into(), module_b.into(), module_a.into()]
    );
    Ok(())
}

#[test]
fn redefining_complete_selector_replaces_its_method_instead_of_overloading()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None)?;
    let selector = Selector::new(1);
    let first =
        registry.publish_method(class, selector, MethodBody::new(10), Visibility::Public)?;

    // When
    let second =
        registry.publish_method(class, selector, MethodBody::new(20), Visibility::Public)?;

    // Then
    assert_ne!(first.id(), second.id());
    assert_eq!(
        registry.active(class)?.methods().get(&selector),
        Some(&second.id())
    );
    Ok(())
}

#[test]
fn dispatch_reads_the_receiver_current_active_revision_after_reopen() -> Result<(), DispatchError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None).map_err(DispatchError::Class)?;
    let selector = Selector::new(2);
    let old = registry
        .publish_method(class, selector, MethodBody::new(10), Visibility::Public)
        .map_err(DispatchError::Class)?;

    // When
    let new = registry
        .publish_method(class, selector, MethodBody::new(20), Visibility::Public)
        .map_err(DispatchError::Class)?;
    let outcome = registry.dispatch(class, selector)?;

    // Then
    assert_ne!(old.id(), new.id());
    assert_eq!(outcome, DispatchOutcome::Invoke(new));
    Ok(())
}

#[test]
fn captured_method_retains_identity_and_body_after_slot_replacement() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None)?;
    let selector = Selector::new(3);
    let old = registry.publish_method(class, selector, MethodBody::new(10), Visibility::Public)?;

    // When
    let new = registry.publish_method(class, selector, MethodBody::new(20), Visibility::Public)?;

    // Then
    assert_eq!(old.body(), MethodBody::new(10));
    assert_eq!(new.body(), MethodBody::new(20));
    assert_ne!(old.id(), new.id());
    Ok(())
}

#[test]
fn binding_a_method_captures_its_method_identity() -> Result<(), DispatchError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None).map_err(DispatchError::Class)?;
    let selector = Selector::new(30);
    let old = registry
        .publish_method(class, selector, MethodBody::new(10), Visibility::Public)
        .map_err(DispatchError::Class)?;

    // When
    let bound = registry.bind(class, selector)?;
    let second_bound = registry.bind(class, selector)?;
    let new = registry
        .publish_method(class, selector, MethodBody::new(20), Visibility::Public)
        .map_err(DispatchError::Class)?;

    // Then
    assert_eq!(bound.method(), old);
    assert_ne!(bound.method(), new);
    assert_ne!(bound.id(), second_bound.id());
    Ok(())
}

#[test]
fn missing_selector_routes_to_method_missing_but_visibility_denial_is_an_error()
-> Result<(), DispatchError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None).map_err(DispatchError::Class)?;
    let private = Selector::new(4);
    registry
        .publish_method(class, private, MethodBody::new(10), Visibility::Private)
        .map_err(DispatchError::Class)?;

    // When
    let missing = registry.dispatch(class, Selector::new(5))?;
    let denied = registry.dispatch(class, private);

    // Then
    assert_eq!(
        missing,
        DispatchOutcome::WouldInvokeMethodMissing {
            selector: Selector::new(5)
        }
    );
    assert!(
        matches!(denied, Err(DispatchError::VisibilityDenied { selector }) if selector == private)
    );
    Ok(())
}

#[test]
fn removing_module_edge_recomputes_stored_mro() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    let class = define_class(&mut registry, None)?;
    let mut candidate = registry.open(class)?;
    candidate.add_module(module);
    registry.publish(candidate)?;

    // When
    let mut candidate = registry.open(class)?;
    candidate.remove_module(module);
    registry.publish(candidate)?;

    // Then
    assert_eq!(registry.active(class)?.mro(), &[class.into()]);
    Ok(())
}

#[test]
fn dispatch_uses_mro_stored_on_the_active_revision() -> Result<(), DispatchError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[]).map_err(DispatchError::Class)?;
    let selector = Selector::new(6);
    let module_method = registry
        .define_module_method(module, selector, MethodBody::new(30), Visibility::Public)
        .map_err(DispatchError::Class)?;
    let class = define_class(&mut registry, None).map_err(DispatchError::Class)?;
    let mut candidate = registry.open(class).map_err(DispatchError::Class)?;
    candidate.add_module(module);
    registry.publish(candidate).map_err(DispatchError::Class)?;

    // When
    let outcome = registry.dispatch(class, selector)?;

    // Then
    assert_eq!(outcome, DispatchOutcome::Invoke(module_method));
    Ok(())
}

#[test]
fn missing_qualified_slot_returns_contract_dispatch_error() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None)?;

    // When
    let result = registry.dispatch_contract(class, ModuleId::new(99), Selector::new(7));

    // Then
    assert!(matches!(
        result,
        Err(DispatchError::ContractDispatch { .. })
    ));
    Ok(())
}

#[test]
fn alias_method_preserves_the_original_method_identity() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = define_class(&mut registry, None)?;
    let original = Selector::new(8);
    let alias = Selector::new(9);
    let method =
        registry.publish_method(class, original, MethodBody::new(80), Visibility::Public)?;

    // When
    registry.alias_method(class, alias, original)?;

    // Then
    assert_eq!(
        registry.dispatch(class, alias),
        Ok(DispatchOutcome::Invoke(method))
    );
    Ok(())
}

#[test]
fn removing_a_local_slot_exposes_the_ancestor_implementation() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = define_class(&mut registry, None)?;
    let child = define_class(&mut registry, Some(parent))?;
    let selector = Selector::new(10);
    let inherited =
        registry.publish_method(parent, selector, MethodBody::new(100), Visibility::Public)?;
    registry.publish_method(child, selector, MethodBody::new(101), Visibility::Public)?;

    // When
    registry.remove_method(child, selector)?;

    // Then
    assert_eq!(
        registry.dispatch(child, selector),
        Ok(DispatchOutcome::Invoke(inherited))
    );
    Ok(())
}

#[test]
fn undefining_a_local_slot_blocks_the_ancestor_implementation() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = define_class(&mut registry, None)?;
    let child = define_class(&mut registry, Some(parent))?;
    let selector = Selector::new(11);
    registry.publish_method(parent, selector, MethodBody::new(110), Visibility::Public)?;

    // When
    registry.undef_method(child, selector)?;

    // Then
    assert_eq!(
        registry.dispatch(child, selector),
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    );
    Ok(())
}

#[test]
fn reflective_invocation_requires_the_lexical_owner_in_the_current_mro() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let owner = define_class(&mut registry, None)?;
    let receiver = define_class(&mut registry, None)?;
    let method = registry.publish_method(
        owner,
        Selector::new(12),
        MethodBody::new(120),
        Visibility::Public,
    )?;

    // When
    let result = registry.validate_method_binding(receiver, method);

    // Then
    assert!(matches!(result, Err(DispatchError::MethodBinding { .. })));
    Ok(())
}

#[test]
fn reflective_invocation_accepts_an_owner_in_the_current_mro() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let owner = define_class(&mut registry, None)?;
    let receiver = define_class(&mut registry, Some(owner))?;
    let method = registry.publish_method(
        owner,
        Selector::new(13),
        MethodBody::new(130),
        Visibility::Public,
    )?;

    // When
    let result = registry.validate_method_binding(receiver, method);

    // Then
    assert_eq!(result, Ok(()));
    Ok(())
}

#[test]
fn reflective_invocation_requires_a_module_owner_in_the_current_mro() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    let receiver = define_class(&mut registry, None)?;
    let method = registry.define_module_method(
        module,
        Selector::new(15),
        MethodBody::new(150),
        Visibility::Public,
    )?;

    // When
    let result = registry.validate_method_binding(receiver, method);

    // Then
    assert!(matches!(result, Err(DispatchError::MethodBinding { .. })));
    Ok(())
}

#[test]
fn reflective_invocation_accepts_a_module_owner_in_the_current_mro() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let module = registry.define_module(&[])?;
    let receiver = registry.define_class_with_capabilities_and_modules(
        StaticSpine::new(1),
        None,
        iris_runtime::MetaCapabilities::all(),
        &[module],
    )?;
    let method = registry.define_module_method(
        module,
        Selector::new(16),
        MethodBody::new(160),
        Visibility::Public,
    )?;

    // When
    let result = registry.validate_method_binding(receiver, method);

    // Then
    assert_eq!(result, Ok(()));
    Ok(())
}

#[test]
fn invalid_reflective_binding_prevents_the_method_body_from_executing() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let owner = define_class(&mut registry, None)?;
    let receiver = define_class(&mut registry, None)?;
    let method = registry.publish_method(
        owner,
        Selector::new(14),
        MethodBody::new(140),
        Visibility::Public,
    )?;
    let mut body_executed = false;

    // When
    let result = registry.invoke_reflective(receiver, method, |_| {
        body_executed = true;
        Ok(())
    });

    // Then
    assert!(matches!(result, Err(DispatchError::MethodBinding { .. })));
    assert!(!body_executed);
    Ok(())
}
