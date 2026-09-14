use iris_runtime::{
    ClassError, ClassId, ClassRegistry, CompositionEdge, DispatchContext, DispatchError,
    DispatchOutcome, MetaCapabilities, Method, MethodBody, ModuleId, ObjectId, Selector,
    StaticSpine, Visibility,
};

struct Fixture {
    registry: ClassRegistry,
    host: ClassId,
    owner: ModuleId,
    other: ModuleId,
    private: Method,
}

impl Fixture {
    fn new() -> Result<Self, ClassError> {
        let mut registry = ClassRegistry::new();
        let owner = registry.define_module(&[])?;
        let other = registry.define_module(&[])?;
        let private = registry.define_module_method(
            owner,
            Selector::new(1),
            MethodBody::new(10),
            Visibility::Private,
        )?;
        let host = registry.define_class_with_capabilities_and_modules(
            StaticSpine::new(1),
            None,
            MetaCapabilities::all(),
            &[owner, other],
        )?;
        Ok(Self {
            registry,
            host,
            owner,
            other,
            private,
        })
    }
}

#[test]
fn own_private_method_is_accessible_when_module_has_an_ordinary_edge() -> Result<(), ClassError> {
    let fixture = Fixture::new()?;

    for receiver_is_self in [true, false] {
        let outcome = fixture.registry.dispatch_with_context(
            fixture.host,
            fixture.private.selector(),
            DispatchContext::module_implementation(fixture.owner, receiver_is_self),
        );

        assert_eq!(outcome, Ok(DispatchOutcome::Invoke(fixture.private)));
    }
    Ok(())
}

#[test]
fn module_private_is_denied_when_caller_is_external_another_module_or_host()
-> Result<(), ClassError> {
    let fixture = Fixture::new()?;

    for context in [
        DispatchContext::external(),
        DispatchContext::module_implementation(fixture.other, true),
        DispatchContext::implementation(fixture.host, true),
    ] {
        let outcome = fixture.registry.dispatch_with_context(
            fixture.host,
            fixture.private.selector(),
            context,
        );

        assert_eq!(
            outcome,
            Err(DispatchError::VisibilityDenied {
                selector: fixture.private.selector(),
            })
        );
    }
    Ok(())
}

#[test]
fn another_module_private_is_denied_when_caller_has_host_private_grant() -> Result<(), ClassError> {
    let mut fixture = Fixture::new()?;
    let mut candidate = fixture.registry.open(fixture.host)?;
    candidate.remove_module(fixture.other);
    candidate.add_composition_edge(CompositionEdge::new(fixture.other, true));
    fixture.registry.publish(candidate)?;

    let outcome = fixture.registry.dispatch_with_context(
        fixture.host,
        fixture.private.selector(),
        DispatchContext::module_implementation(fixture.other, true),
    );

    assert_eq!(
        outcome,
        Err(DispatchError::VisibilityDenied {
            selector: fixture.private.selector(),
        })
    );
    Ok(())
}

#[test]
fn ancestor_private_is_denied_when_only_receiver_host_grants_access() -> Result<(), ClassError> {
    let mut fixture = Fixture::new()?;
    let selector = Selector::new(2);
    fixture.registry.publish_method(
        fixture.host,
        selector,
        MethodBody::new(20),
        Visibility::Private,
    )?;
    let child = fixture
        .registry
        .define_class(StaticSpine::new(1), Some(fixture.host))?;
    let mut candidate = fixture.registry.open(child)?;
    candidate.add_composition_edge(CompositionEdge::new(fixture.owner, true));
    fixture.registry.publish(candidate)?;

    let outcome = fixture.registry.dispatch_with_context(
        child,
        selector,
        DispatchContext::module_implementation(fixture.owner, true),
    );

    assert_eq!(outcome, Err(DispatchError::VisibilityDenied { selector }));
    Ok(())
}

#[test]
fn host_private_access_tracks_current_composition_grant() -> Result<(), ClassError> {
    for (keep_edge, private_access) in [(true, true), (false, false), (true, false)] {
        let mut fixture = Fixture::new()?;
        let selector = Selector::new(2);
        let method = fixture.registry.publish_method(
            fixture.host,
            selector,
            MethodBody::new(20),
            Visibility::Private,
        )?;
        let mut granted = fixture.registry.open(fixture.host)?;
        granted.remove_module(fixture.owner);
        granted.add_composition_edge(CompositionEdge::new(fixture.owner, true));
        fixture.registry.publish(granted)?;
        let mut current = fixture.registry.open(fixture.host)?;
        current.remove_module(fixture.owner);
        if keep_edge {
            current.add_composition_edge(CompositionEdge::new(fixture.owner, private_access));
        }
        fixture.registry.publish(current)?;

        let outcome = fixture.registry.dispatch_with_context(
            fixture.host,
            selector,
            DispatchContext::module_implementation(fixture.owner, true),
        );

        let expected = if private_access {
            Ok(DispatchOutcome::Invoke(method))
        } else {
            Err(DispatchError::VisibilityDenied { selector })
        };
        assert_eq!(outcome, expected);
    }
    Ok(())
}

#[test]
fn alias_cannot_preserve_private_authority_when_module_owner_leaves_mro() -> Result<(), ClassError>
{
    let mut fixture = Fixture::new()?;
    let alias = Selector::new(3);
    fixture
        .registry
        .alias_method(fixture.host, alias, fixture.private.selector())?;
    let mut candidate = fixture.registry.open(fixture.host)?;
    candidate.remove_module(fixture.owner);
    fixture.registry.publish(candidate)?;

    let outcome = fixture.registry.dispatch_with_context(
        fixture.host,
        alias,
        DispatchContext::module_implementation(fixture.owner, true),
    );

    assert_eq!(
        outcome,
        Err(DispatchError::VisibilityDenied { selector: alias })
    );
    Ok(())
}

#[test]
fn context_binding_captures_own_private_method_when_owner_is_present() -> Result<(), ClassError> {
    let mut fixture = Fixture::new()?;
    let receiver = ObjectId::new(1);

    let outcome = fixture.registry.bind_instance_with_context(
        receiver,
        fixture.host,
        fixture.private.selector(),
        DispatchContext::module_implementation(fixture.owner, true),
    );

    assert_eq!(outcome.map(|bound| bound.method()), Ok(fixture.private));
    Ok(())
}

#[test]
fn context_binding_denies_other_module_even_with_host_grant() -> Result<(), ClassError> {
    let mut fixture = Fixture::new()?;
    let mut candidate = fixture.registry.open(fixture.host)?;
    candidate.remove_module(fixture.other);
    candidate.add_composition_edge(CompositionEdge::new(fixture.other, true));
    fixture.registry.publish(candidate)?;

    let outcome = fixture.registry.bind_instance_with_context(
        ObjectId::new(1),
        fixture.host,
        fixture.private.selector(),
        DispatchContext::module_implementation(fixture.other, true),
    );

    assert_eq!(
        outcome,
        Err(DispatchError::VisibilityDenied {
            selector: fixture.private.selector(),
        })
    );
    Ok(())
}

#[test]
fn retained_private_binding_rejects_entry_when_module_owner_is_removed() -> Result<(), DispatchError>
{
    let mut fixture = Fixture::new().map_err(DispatchError::Class)?;
    let bound = fixture.registry.bind_instance_with_context(
        ObjectId::new(1),
        fixture.host,
        fixture.private.selector(),
        DispatchContext::module_implementation(fixture.owner, true),
    )?;
    let mut candidate = fixture
        .registry
        .open(fixture.host)
        .map_err(DispatchError::Class)?;
    candidate.remove_module(fixture.owner);
    fixture
        .registry
        .publish(candidate)
        .map_err(DispatchError::Class)?;
    let mut entered = false;

    let outcome = fixture
        .registry
        .invoke_reflective(fixture.host, bound.method(), |_| {
            entered = true;
            Ok(())
        });

    assert_eq!(
        outcome,
        Err(DispatchError::MethodBinding {
            selector: fixture.private.selector(),
        })
    );
    assert!(!entered);
    Ok(())
}
