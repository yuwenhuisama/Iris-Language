use iris_runtime::{
    BuiltinClass, Capability, ClassError, ClassRegistry, ComparisonError, ComparisonProtocol,
    ComparisonSlot, DispatchContext, DispatchError, DispatchOutcome, MetaCapabilities, MethodBody,
    Selector, StaticSpine, Truthiness, TruthinessError, TruthinessMethod, Value, Visibility,
};

fn class(
    registry: &mut ClassRegistry,
    parent: Option<iris_runtime::ClassId>,
) -> Result<iris_runtime::ClassId, ClassError> {
    registry.define_class(StaticSpine::new(1), parent)
}

#[test]
fn protected_selector_requires_lexical_subclass_authority_and_self_receiver()
-> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = class(&mut registry, None)?;
    let child = class(&mut registry, Some(parent))?;
    let selector = Selector::new(10);
    let protected =
        registry.publish_method(parent, selector, MethodBody::new(1), Visibility::Protected)?;

    // When
    let external = registry.dispatch(child, selector);
    let permitted = registry.dispatch_with_context(
        child,
        selector,
        DispatchContext::implementation(child, true),
    );
    let wrong_receiver = registry.dispatch_with_context(
        child,
        selector,
        DispatchContext::implementation(child, false),
    );

    // Then
    assert!(matches!(
        external,
        Err(DispatchError::VisibilityDenied { .. })
    ));
    assert_eq!(permitted, Ok(DispatchOutcome::Invoke(protected)));
    assert!(matches!(
        wrong_receiver,
        Err(DispatchError::VisibilityDenied { .. })
    ));
    Ok(())
}

#[test]
fn default_spaceship_returns_nil_without_object_identity_ordering() {
    // Given
    let comparison = ComparisonProtocol::default();

    // When
    let first = comparison.compare(&Value::Object(iris_runtime::ObjectId::new(1)));
    let second = comparison.compare(&Value::Object(iris_runtime::ObjectId::new(2)));

    // Then
    assert_eq!(first, Ok(Value::Nil));
    assert_eq!(second, Ok(Value::Nil));
}

#[test]
fn default_comparisons_accept_only_unit_ordering_or_nil() {
    // Given
    let mut comparison = ComparisonProtocol::default();

    // When
    comparison.replace_spaceship(Value::Integer(2_u8.into()));
    let invalid = comparison.call(ComparisonSlot::Less, false);
    comparison.replace_spaceship(Value::Integer((-1_i8).into()));
    let less = comparison.call(ComparisonSlot::Less, false);
    comparison.replace_spaceship(Value::Integer(0_u8.into()));
    let equal = comparison.call(ComparisonSlot::Equal, false);
    comparison.replace_spaceship(Value::Integer(1_u8.into()));
    let greater = comparison.call(ComparisonSlot::Greater, false);
    comparison.replace_spaceship(Value::Nil);
    let unordered = comparison.call(ComparisonSlot::NotEqual, false);

    // Then
    assert_eq!(invalid, Err(ComparisonError::Contract));
    assert_eq!(less, Ok(Value::Bool(true)));
    assert_eq!(equal, Ok(Value::Bool(true)));
    assert_eq!(greater, Ok(Value::Bool(true)));
    assert_eq!(unordered, Ok(Value::Bool(true)));
}

#[test]
fn replacing_spaceship_changes_default_slots_but_not_directly_replaced_slot() {
    // Given
    let mut comparison = ComparisonProtocol::default();
    comparison.replace_spaceship(Value::Integer(1_u8.into()));

    // When
    let delegated = comparison.call(ComparisonSlot::Greater, false);
    comparison.replace_slot(ComparisonSlot::Greater, Value::Bool(false));
    comparison.replace_spaceship(Value::Integer((-1_i8).into()));
    let direct = comparison.call(ComparisonSlot::Greater, false);

    // Then
    assert_eq!(delegated, Ok(Value::Bool(true)));
    assert_eq!(direct, Ok(Value::Bool(false)));
}

#[test]
fn logical_and_returns_original_falsy_value_without_evaluating_rhs() {
    // Given
    let mut evaluated = false;

    // When
    let result = Truthiness::logical_and(Value::Nil, TruthinessMethod::Default, || {
        evaluated = true;
        Ok(Value::Bool(true))
    });

    // Then
    assert_eq!(result, Ok(Value::Nil));
    assert!(!evaluated);
}

#[test]
fn truthiness_rejects_non_bool_and_preserves_raised_error() {
    // Given
    let non_bool = TruthinessMethod::Returns(Value::Integer(1_u8.into()));
    let raised = TruthinessMethod::Raises(Value::Integer(9_u8.into()));

    // When
    let wrong_type = Truthiness::test(&Value::Object(iris_runtime::ObjectId::new(1)), non_bool);
    let propagated = Truthiness::test(&Value::Object(iris_runtime::ObjectId::new(1)), raised);

    // Then
    assert_eq!(wrong_type, Err(TruthinessError::TypeContract));
    assert_eq!(
        propagated,
        Err(TruthinessError::Raised(Value::Integer(9_u8.into())))
    );
}

#[test]
fn super_resolves_after_lexical_owner_in_current_mro() -> Result<(), DispatchError> {
    // Given
    let mut registry = ClassRegistry::new();
    let parent = class(&mut registry, None).map_err(DispatchError::Class)?;
    let child = class(&mut registry, Some(parent)).map_err(DispatchError::Class)?;
    let selector = Selector::new(20);
    let parent_method = registry
        .publish_method(parent, selector, MethodBody::new(1), Visibility::Public)
        .map_err(DispatchError::Class)?;
    let child_method = registry
        .publish_method(child, selector, MethodBody::new(2), Visibility::Public)
        .map_err(DispatchError::Class)?;

    // When
    let resolved = registry.dispatch_super(child, child_method)?;

    // Then
    assert_eq!(resolved, parent_method);
    Ok(())
}

#[test]
fn protected_builtin_superclass_cannot_be_changed() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let root = class(&mut registry, None)?;
    let integer =
        registry.define_builtin_class(BuiltinClass::Integer, StaticSpine::new(2), Some(root))?;
    let before = registry.active_revision(integer)?;
    let mut candidate = registry.open(integer)?;
    candidate.replace_runtime_superclass(None);

    // When
    let result = registry.publish(candidate);

    // Then
    assert_eq!(
        result,
        Err(ClassError::ProtectedSuperclass { class: integer })
    );
    assert_eq!(registry.active_revision(integer)?, before);
    Ok(())
}

#[test]
fn meta_deny_only_narrows_and_method_capabilities_are_independent() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class_with_capabilities(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
    )?;
    registry.publish_method(
        class,
        Selector::new(31),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    let mut candidate = registry.open(class)?;
    candidate.deny_meta(&[Capability::MethodSet]);
    registry.publish(candidate)?;

    // When
    let add = registry.publish_method(
        class,
        Selector::new(30),
        MethodBody::new(1),
        Visibility::Public,
    );
    let body = registry.publish_method(
        class,
        Selector::new(31),
        MethodBody::new(2),
        Visibility::Public,
    );
    let caps = registry.active_meta_capabilities(class)?;

    // Then
    assert_eq!(
        add,
        Err(ClassError::MetaCapabilityDenied {
            target: class,
            operation: Capability::MethodSet,
            policy_origin: iris_runtime::PolicyOrigin::Class(class),
            reason: "the active effective policy denies this meta operation",
        })
    );
    assert!(body.is_ok());
    assert!(!caps.allows(Capability::MethodSet));
    assert!(caps.allows(Capability::MethodBody));
    Ok(())
}

#[test]
fn property_set_and_property_body_are_independently_deniable() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class_with_capabilities(
        StaticSpine::new(1),
        None,
        MetaCapabilities::all(),
    )?;
    let property = Selector::new(40);
    registry.publish_stored_property(class, property, MethodBody::new(1))?;
    let mut candidate = registry.open(class)?;
    candidate.deny_meta(&[Capability::PropertySet]);
    registry.publish(candidate)?;

    // When
    let add = registry.publish_stored_property(class, Selector::new(41), MethodBody::new(1));
    let replace = registry.publish_stored_property(class, property, MethodBody::new(2));

    // Then
    assert_eq!(
        add,
        Err(ClassError::MetaCapabilityDenied {
            target: class,
            operation: Capability::PropertySet,
            policy_origin: iris_runtime::PolicyOrigin::Class(class),
            reason: "the active effective policy denies this meta operation",
        })
    );
    assert!(replace.is_ok());
    Ok(())
}

#[test]
fn decorated_method_publication_respects_method_set_capability() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let mut candidate = registry.open(class)?;
    candidate.deny_meta(&[Capability::MethodSet]);
    registry.publish(candidate)?;

    // When
    let result = registry.publish_decorated_method(
        class,
        Selector::new(99),
        MethodBody::new(1),
        Visibility::Public,
        [iris_runtime::DecoratorTransform::metadata(
            "logged",
            ["trace"],
        )],
    );

    // Then
    assert_eq!(
        result,
        Err(ClassError::MetaCapabilityDenied {
            target: class,
            operation: Capability::MethodSet,
            policy_origin: iris_runtime::PolicyOrigin::Class(class),
            reason: "the active effective policy denies this meta operation",
        })
    );
    Ok(())
}

#[test]
fn meta_capability_error_reports_target_operation_origin_and_reason() -> Result<(), ClassError> {
    // Given
    let mut registry = ClassRegistry::new();
    let policy = MetaCapabilities::denying(&[Capability::MethodSet]);
    let base = registry.define_class_with_capabilities(StaticSpine::new(1), None, policy)?;
    let child = registry.define_class(StaticSpine::new(1), Some(base))?;

    // When
    let result = registry.publish_method(
        child,
        Selector::new(120),
        MethodBody::new(1),
        Visibility::Public,
    );

    // Then
    assert_eq!(
        result,
        Err(ClassError::MetaCapabilityDenied {
            target: child,
            operation: Capability::MethodSet,
            policy_origin: iris_runtime::PolicyOrigin::Class(base),
            reason: "the active effective policy denies this meta operation",
        })
    );
    Ok(())
}
