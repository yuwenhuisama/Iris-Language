use iris_runtime::ObjectId;

#[test]
fn rejected_calls_preserve_the_original_bridge_and_settlement_allows_retry()
-> Result<(), DecoratorProtocolError> {
    let owner = TaskId::new(ObjectId::new(1));
    let bridge = TaskId::new(ObjectId::new(2));
    let mut scope = NextScope::asynchronous(ObjectId::new(3), owner);
    scope.begin_attempt(Some(owner), Some(bridge))?;
    assert!(
        scope
            .begin_attempt(None, Some(TaskId::new(ObjectId::new(4))))
            .is_err()
    );
    assert_eq!(scope.bridge(), Some(bridge));
    assert!(scope.in_flight());
    scope.finish_attempt();
    scope.begin_attempt(Some(owner), Some(TaskId::new(ObjectId::new(5))))?;
    scope.finish_attempt();
    assert_eq!(scope.finish_owner(), None);
    Ok(())
}

#[test]
fn unfinished_owner_invalidation_keeps_inner_work_tracking() -> Result<(), DecoratorProtocolError> {
    let owner = TaskId::new(ObjectId::new(1));
    let bridge = TaskId::new(ObjectId::new(2));
    let next = ObjectId::new(3);
    let mut scope = NextScope::asynchronous(next, owner);
    scope.begin_attempt(Some(owner), Some(bridge))?;
    let Some(error) = scope.finish_owner() else {
        unreachable!()
    };
    assert_eq!(error.category(), ProtocolCategory::UnfinishedInner);
    assert_eq!(error.owner(), Some(next));
    assert_eq!(scope.owner(), ScopeOwner::Asynchronous(owner));
    assert_eq!(scope.bridge(), Some(bridge));
    assert!(scope.in_flight());
    scope.finish_attempt();
    assert_eq!(
        scope
            .begin_attempt(Some(owner), Some(bridge))
            .map_err(|error| error.category()),
        Err(ProtocolCategory::Expired)
    );
    Ok(())
}
use iris_runtime::decorator_protocol::*;

#[test]
fn scope_checks_expired_then_foreign_then_overlap() -> Result<(), DecoratorProtocolError> {
    let owner = TaskId::new(ObjectId::new(1));
    let foreign = TaskId::new(ObjectId::new(2));
    let mut scope = NextScope::asynchronous(ObjectId::new(3), owner);
    scope.begin_attempt(Some(owner), Some(TaskId::new(ObjectId::new(4))))?;
    assert_eq!(
        scope
            .begin_attempt(Some(foreign), None)
            .map_err(|error| error.category()),
        Err(ProtocolCategory::ForeignTask)
    );
    assert_eq!(
        scope
            .begin_attempt(Some(owner), None)
            .map_err(|error| error.category()),
        Err(ProtocolCategory::Overlap)
    );
    let unfinished = scope.finish_owner();
    assert_eq!(
        unfinished.map(|error| error.category()),
        Some(ProtocolCategory::UnfinishedInner)
    );
    assert_eq!(
        scope
            .begin_attempt(Some(foreign), None)
            .map_err(|error| error.category()),
        Err(ProtocolCategory::Expired)
    );
    Ok(())
}

#[test]
fn sequential_sync_attempts_are_allowed_until_owner_finishes() -> Result<(), DecoratorProtocolError>
{
    let mut scope =
        NextScope::synchronous(ObjectId::new(1), ActivationId::new(ObjectId::new(2)), None);
    scope.begin_attempt(None, None)?;
    scope.finish_attempt();
    scope.begin_attempt(None, None)?;
    scope.finish_attempt();
    assert_eq!(scope.finish_owner(), None);
    assert_eq!(
        scope
            .begin_attempt(None, None)
            .map_err(|error| error.category()),
        Err(ProtocolCategory::Expired)
    );
    Ok(())
}
