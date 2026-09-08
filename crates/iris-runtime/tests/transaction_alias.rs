use iris_runtime::{
    ClassError, ClassRegistry, DispatchOutcome, MethodBody, Selector, StaticSpine, Visibility,
};

const GOOD: Selector = Selector::new(1);
const COPY: Selector = Selector::new(2);

#[test]
fn alias_selects_replacement_when_source_is_staged() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let active = registry.publish_method(class, GOOD, MethodBody::new(1), Visibility::Public)?;
    let revision = registry.active_revision(class)?;
    registry.begin_transaction(class)?;
    let staged = registry.publish_method(class, GOOD, MethodBody::new(2), Visibility::Public)?;

    registry.alias_method(class, COPY, GOOD)?;

    assert_eq!(registry.staged_method(class, COPY), Some(staged.id()));
    assert_eq!(registry.active_revision(class)?, revision);
    assert_eq!(
        registry.dispatch(class, GOOD),
        Ok(DispatchOutcome::Invoke(active))
    );
    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector: COPY })
    );
    registry.commit_transaction(class)?;
    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::Invoke(staged))
    );
    assert_eq!(
        registry.dispatch(class, GOOD),
        Ok(DispatchOutcome::Invoke(staged))
    );
    Ok(())
}

#[test]
fn alias_retains_identity_when_staged_source_is_replaced_again() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.publish_method(class, GOOD, MethodBody::new(1), Visibility::Public)?;
    registry.begin_transaction(class)?;
    let selected = registry.publish_method(class, GOOD, MethodBody::new(2), Visibility::Public)?;
    registry.alias_method(class, COPY, GOOD)?;

    let replacement =
        registry.publish_method(class, GOOD, MethodBody::new(3), Visibility::Public)?;
    registry.commit_transaction(class)?;

    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::Invoke(selected))
    );
    assert_eq!(
        registry.dispatch(class, GOOD),
        Ok(DispatchOutcome::Invoke(replacement))
    );
    Ok(())
}

#[test]
fn alias_resolves_new_source_and_alias_chain_when_staged() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.begin_transaction(class)?;
    let selected = registry.publish_method(class, GOOD, MethodBody::new(2), Visibility::Public)?;
    let second_copy = Selector::new(3);

    registry.alias_method(class, COPY, GOOD)?;
    registry.alias_method(class, second_copy, COPY)?;
    registry.commit_transaction(class)?;

    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::Invoke(selected))
    );
    assert_eq!(
        registry.dispatch(class, second_copy),
        Ok(DispatchOutcome::Invoke(selected))
    );
    Ok(())
}

#[test]
fn active_methods_stay_unchanged_when_staged_alias_is_rolled_back() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    let active = registry.publish_method(class, GOOD, MethodBody::new(1), Visibility::Public)?;
    let revision = registry.active_revision(class)?;
    registry.begin_transaction(class)?;
    registry.publish_method(class, GOOD, MethodBody::new(2), Visibility::Public)?;
    registry.alias_method(class, COPY, GOOD)?;

    registry.roll_back_transaction(class);

    assert_eq!(registry.active_revision(class)?, revision);
    assert_eq!(
        registry.dispatch(class, GOOD),
        Ok(DispatchOutcome::Invoke(active))
    );
    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector: COPY })
    );
    Ok(())
}

#[test]
fn alias_selects_staged_ancestor_when_local_source_is_removed() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class(StaticSpine::new(1), None)?;
    let child = registry.define_class(StaticSpine::new(1), Some(parent))?;
    registry.publish_method(parent, GOOD, MethodBody::new(1), Visibility::Public)?;
    let local = registry.publish_method(child, GOOD, MethodBody::new(3), Visibility::Public)?;
    registry.begin_transaction(parent)?;
    registry.begin_transaction(child)?;
    let inherited =
        registry.publish_method(parent, GOOD, MethodBody::new(2), Visibility::Public)?;
    registry.remove_method(child, GOOD)?;

    registry.alias_method(child, COPY, GOOD)?;

    assert_eq!(registry.staged_method(child, COPY), Some(inherited.id()));
    assert_eq!(
        registry.dispatch(child, GOOD),
        Ok(DispatchOutcome::Invoke(local))
    );
    registry.commit_group()?;
    assert_eq!(
        registry.dispatch(child, COPY),
        Ok(DispatchOutcome::Invoke(inherited))
    );
    Ok(())
}

#[test]
fn alias_rejects_source_when_staged_tombstone_blocks_ancestors() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let parent = registry.define_class(StaticSpine::new(1), None)?;
    let child = registry.define_class(StaticSpine::new(1), Some(parent))?;
    registry.publish_method(parent, GOOD, MethodBody::new(1), Visibility::Public)?;
    registry.publish_method(child, GOOD, MethodBody::new(2), Visibility::Public)?;
    registry.begin_transaction(child)?;
    registry.undef_method(child, GOOD)?;

    let outcome = registry.alias_method(child, COPY, GOOD);

    assert_eq!(
        outcome,
        Err(ClassError::MethodSlotNotFound {
            class: child,
            selector: GOOD
        })
    );
    assert_eq!(registry.staged_method(child, COPY), None);
    Ok(())
}

#[test]
fn alias_rejects_source_when_removed_without_ancestor() -> Result<(), ClassError> {
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.publish_method(class, GOOD, MethodBody::new(1), Visibility::Public)?;
    registry.begin_transaction(class)?;
    registry.remove_method(class, GOOD)?;

    let outcome = registry.alias_method(class, COPY, GOOD);

    assert_eq!(
        outcome,
        Err(ClassError::MethodSlotNotFound {
            class,
            selector: GOOD
        })
    );
    Ok(())
}

#[test]
fn alias_selects_source_when_staged_replacement_clears_active_tombstone() -> Result<(), ClassError>
{
    let mut registry = ClassRegistry::new();
    let class = registry.define_class(StaticSpine::new(1), None)?;
    registry.undef_method(class, GOOD)?;
    registry.begin_transaction(class)?;
    let selected = registry.publish_method(class, GOOD, MethodBody::new(2), Visibility::Public)?;

    registry.alias_method(class, COPY, GOOD)?;
    registry.commit_transaction(class)?;

    assert_eq!(
        registry.dispatch(class, COPY),
        Ok(DispatchOutcome::Invoke(selected))
    );
    Ok(())
}
