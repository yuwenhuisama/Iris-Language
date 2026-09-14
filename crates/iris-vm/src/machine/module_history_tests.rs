use super::*;
use crate::{PackageHistory, RevisionArtifact, compile};

fn history(source: &str) -> PackageHistory {
    PackageHistory {
        artifacts: vec![RevisionArtifact {
            package_id: "runtime-local".into(),
            api_major: 1,
            logical_owner: "Provider".into(),
            revision: 71,
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(source.as_bytes()),
                source.into(),
            ),
        }],
    }
}

#[test]
fn one_structural_commit_when_history_replays_keeps_module_identity() {
    let mut given = Machine::new().unwrap();
    given.execute(&compile("module Provider { public fun value() { 9 } }; class Client mixin Provider {}; Client.new().value()").unwrap()).unwrap();
    let module = given.active_module_id("Provider").unwrap();
    let before = given
        .runtime
        .registry()
        .active_module(module)
        .unwrap()
        .clone();
    let class = given
        .code
        .module_owner(module)
        .unwrap()
        .0
        .link
        .as_ref()
        .unwrap()
        .classes
        .borrow()[0];
    let host = given.runtime.registry().active(class).unwrap().clone();
    given.enter_package_history(history("module Provider { public fun value() { 1 } }"));

    let when = given.rollback_module(module, &[Value::Integer(71_u64.into())]);

    assert!(when.is_ok());
    let after = given.runtime.registry().active_module(module).unwrap();
    assert_eq!(after.owner(), before.owner());
    assert_eq!(after.number(), before.number() + 1);
    assert_eq!(after.commit_id(), host.commit_id() + 1);
    assert_eq!(given.runtime.registry().active(class).unwrap(), &host);
}

#[test]
fn older_module_when_observer_bundle_collides_uses_its_own_package_and_selectors() {
    let mut given = Machine::new().unwrap();
    let original =
        compile("module Provider { public fun value() { 9 } }; Provider.value()").unwrap();
    given.execute(&original).unwrap();
    let module = given.active_module_id("Provider").unwrap();
    let observer = compile("module Provider { public fun apple() { 80 }; public fun value() { 90 } }; Provider.apple()").unwrap();
    given.execute(&observer).unwrap();
    let observer_module = given.active_module_id("Provider").unwrap();
    let observer = given.code.owner(&observer).unwrap();
    given.enter_package_history(history("module Unrelated { public fun apple() { 100 } }; module Provider { public fun value() { 1 } }"));

    let when = given.rollback_module(module, &[Value::Integer(71_u64.into())]);

    assert!(when.is_ok());
    assert_eq!(given.active_module_id("Provider").unwrap(), observer_module);
    assert_eq!(
        given.invoke_module_member(("Provider", "value", None), Vec::new(), &observer, &[]),
        Ok(Value::Integer(90_u64.into()))
    );
    let owner = given.code.module_owner(module).unwrap().0;
    let method = given
        .runtime
        .registry()
        .module_method(module, selector_id(&owner, "value").unwrap())
        .unwrap();
    let body = given.resolve_method_body(method.body(), &observer).unwrap();
    assert_eq!(
        given.invoke_function(body.function, Vec::new(), &body.program, &body.classes),
        Ok(Value::Integer(1_u64.into()))
    );
}

#[test]
fn late_failure_when_transform_has_external_effects_restores_only_structural_state() {
    let decorator = "class Effect {} impl Effect for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { print(:external_effect); raise { || -> Integer; 42 } } }";
    let mut given = Machine::new().unwrap();
    given.execute(&compile(&format!("{decorator}; module Provider {{ public fun first() {{ 8 }}; public fun value() {{ 9 }} }}; Provider.value()")).unwrap()).unwrap();
    let module = given.active_module_id("Provider").unwrap();
    let before = given
        .runtime
        .registry()
        .active_module(module)
        .unwrap()
        .clone();
    let chains = given.wrapper_chains.len();
    let metadata = given.decorator_metadata.len();
    given.enter_package_history(history(&format!("{decorator}; module Provider {{ public fun first() {{ 1 }}; @Effect() public fun value() {{ 2 }} }}")));

    let when = given.rollback_module(module, &[Value::Integer(71_u64.into())]);

    assert!(matches!(when, Err(MachineError::Raised(_))));
    assert_eq!(
        given.runtime.registry().active_module(module).unwrap(),
        &before
    );
    assert!(given.runtime.registry().staged_module(module).is_err());
    assert_eq!(given.wrapper_chains.len(), chains);
    assert_eq!(given.decorator_metadata.len(), metadata);
    assert_eq!(given.open_depth, 0);
}
