use iris_runtime::Value;

#[test]
fn retains_stored_getter_cache_when_replay_replaces_initializer_handle() {
    let mut given = crate::Session::new().unwrap();
    let source = "class Cache {}; impl Cache for PropertyDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { @cached = nil; Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; if @cached == nil { @cached = next.call() }; @cached }) } }; class Target { @Cache() public property value: Integer = 1 }; 0";
    given.evaluate(source).unwrap();
    given.evaluator.enter_artifact(None);
    given
        .evaluator
        .enter_revision_artifacts(vec![crate::RevisionArtifact {
            package_id: super::super::LOCAL_PACKAGE.into(),
            api_major: 1,
            logical_owner: "Target".into(),
            revision: 1,
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(source.as_bytes()),
                source.into(),
            ),
        }]);
    let class = given.evaluator.class_name("Target").unwrap().unwrap();
    let object = given.evaluator.construct(class, &[]).unwrap();
    let method = given
        .evaluator
        .runtime
        .dispatch_instance(object, given.evaluator.selectors["value"])
        .unwrap();
    let bound = given
        .evaluator
        .runtime
        .registry_mut()
        .bind_retained_instance(object, method)
        .unwrap();
    let retained = Value::BoundMethod(bound);
    assert_eq!(
        given.evaluator.send(retained.clone(), "call", &[]),
        Ok(Value::Integer(1u64.into()))
    );
    given
        .evaluator
        .runtime
        .assign_raw_ivar(
            object,
            given.evaluator.selectors["@value"],
            Value::Integer(9u64.into()),
        )
        .unwrap();
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(class)
        .unwrap()
        .clone();

    given.evaluate("Target.rollback(1)").unwrap();

    let after = given.evaluator.runtime.registry().active(class).unwrap();
    assert_eq!(after.commit_id(), before.commit_id() + 1);
    assert_ne!(
        after.properties()[0].initializer(),
        before.properties()[0].initializer()
    );
    assert_ne!(
        after.methods()[&given.evaluator.selectors["value"]],
        method.id()
    );
    assert_eq!(
        given.evaluator.send(Value::Object(object), "value", &[]),
        Ok(Value::Integer(9u64.into()))
    );
    assert_eq!(
        given.evaluator.send(retained, "call", &[]),
        Ok(Value::Integer(1u64.into()))
    );
}

#[test]
fn retains_all_old_slot_identities_when_replay_commits_once() {
    let mut given = crate::Session::new().unwrap();
    let source = "class Cache {}; impl Cache for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { @cached = nil; Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; if @cached == nil { @cached = next.call() }; @cached }) } }; class Target { public property fun value() -> Integer { 1 }; public property fun value=(value: Integer) -> Symbol { :written }; @Cache() public class fun count(value: Integer) -> Integer { value } }; 0";
    given.evaluate(source).unwrap();
    given.evaluator.enter_artifact(None);
    given
        .evaluator
        .enter_revision_artifacts(vec![crate::RevisionArtifact {
            package_id: super::super::LOCAL_PACKAGE.into(),
            api_major: 1,
            logical_owner: "Target".into(),
            revision: 1,
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(source.as_bytes()),
                source.into(),
            ),
        }]);
    let class = given.evaluator.class_name("Target").unwrap().unwrap();
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(class)
        .unwrap()
        .clone();
    let identity = before.singleton_methods()[&given.evaluator.selectors["count"]];
    let old = given
        .evaluator
        .runtime
        .registry()
        .method_by_id(identity)
        .unwrap();
    let retained = Value::BoundMethod(
        given
            .evaluator
            .runtime
            .registry_mut()
            .bind_retained_class(class, old)
            .unwrap(),
    );
    assert_eq!(
        given
            .evaluator
            .send(retained.clone(), "call", &[Value::Integer(4u64.into())]),
        Ok(Value::Integer(4u64.into()))
    );

    given.evaluate("Target.rollback(1)").unwrap();

    let after = given.evaluator.runtime.registry().active(class).unwrap();
    assert_eq!(after.number(), before.number() + 1);
    assert_eq!(after.commit_id(), before.commit_id() + 1);
    for selector in ["value", "value="] {
        let selector = &given.evaluator.selectors[selector];
        assert_ne!(after.methods()[selector], before.methods()[selector]);
    }
    assert_ne!(
        after.singleton_methods()[&given.evaluator.selectors["count"]],
        identity
    );
    assert_eq!(
        given.evaluate("Target.count(7)"),
        Ok(Value::Integer(7u64.into()))
    );
    assert_eq!(
        given
            .evaluator
            .send(retained, "call", &[Value::Integer(99u64.into())]),
        Ok(Value::Integer(4u64.into()))
    );
}
