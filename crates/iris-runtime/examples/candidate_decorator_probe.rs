use iris_runtime::{
    Capability, ClassError, ClassRegistry, DispatchContext, DispatchOutcome, MetaCapabilities,
    MethodBody, PolicyOrigin, RevisionId, Selector, StaticSpine, Visibility,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = ClassRegistry::new();
    let class = registry.stage_class_origin(StaticSpine::new(1), None, &[])?;
    let selector = Selector::new(1);
    let original =
        registry.publish_origin_method(class, selector, MethodBody::new(10), Visibility::Public)?;
    registry.require_candidate_meta_capability(class, Capability::MethodBody)?;
    let wrapper = registry.publish_candidate_decorated_method(
        class,
        selector,
        MethodBody::new(20),
        Visibility::Public,
        [],
    )?;
    let private = registry.publish_candidate_decorated_method(
        class,
        Selector::new(2),
        MethodBody::new(30),
        Visibility::Private,
        [],
    )?;
    assert_eq!(
        registry.active(class),
        Err(ClassError::UnknownClassId(class))
    );
    assert_eq!(
        registry.require_meta_capability(class, Capability::MethodBody),
        Err(ClassError::UnknownClassId(class))
    );
    assert!(registry.dispatch(class, selector).is_err());
    registry.commit_declaration_group()?;
    let DispatchOutcome::Invoke(selected) = registry
        .dispatch(class, selector)
        .map_err(|error| format!("{error:?}"))?
    else {
        return Err("wrapped slot was not published".into());
    };
    let output = registry
        .invoke_reflective(class, selected, |method| {
            assert_eq!(method, wrapper);
            Ok(original.body().raw() + method.body().raw())
        })
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(output, 30);
    assert_eq!(
        registry
            .dispatch_with_context(
                class,
                private.selector(),
                DispatchContext::implementation(class, true)
            )
            .map_err(|error| format!("{error:?}"))?,
        DispatchOutcome::Invoke(private)
    );
    assert_eq!(registry.active(class)?.number(), 1);

    let denied = registry.stage_class_origin(
        StaticSpine::new(2)
            .with_meta_capabilities(MetaCapabilities::denying(&[Capability::MethodBody])),
        None,
        &[],
    )?;
    registry.publish_origin_method(denied, selector, MethodBody::new(1), Visibility::Public)?;
    let result = registry.publish_candidate_decorated_method(
        denied,
        selector,
        MethodBody::new(2),
        Visibility::Public,
        [],
    );
    assert!(matches!(result, Err(ClassError::MetaCapabilityDenied {
        target, operation: Capability::MethodBody, policy_origin: PolicyOrigin::Class(origin), ..
    }) if target == denied && origin == denied));
    registry.roll_back_group();
    let next = registry.define_class(StaticSpine::new(3), None)?;
    assert_eq!(
        (
            registry.active(next)?.id(),
            registry.active(next)?.commit_id()
        ),
        (RevisionId::new(1), 2)
    );
    println!("candidate decorator probe: add + wrap({output}) + deny + rollback passed");
    Ok(())
}
