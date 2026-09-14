use crate::{EvaluationError, Session};
use iris_runtime::Value;

#[test]
fn module_is_unpublished_when_class_property_initializer_raises() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("module M { public class fun fail(){raise :stop}; public class property x: Integer = M.fail() }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert!(!given.evaluator.module_names.contains_key("M"));
    assert!(!given.evaluator.module_packages.contains_key("M"));
    assert!(given.evaluator.module_classes.is_empty());
    assert!(given.evaluator.module_mains.is_empty());
    assert!(given.evaluator.module_methods.is_empty());
    assert!(given.evaluator.class_level_properties.is_empty());
    assert!(given.evaluator.class_property_accessors.is_empty());
    assert!(given.evaluator.origin_names.is_empty());
    assert!(given.evaluator.origin_bodies.is_empty());
}

#[test]
fn module_publishes_once_when_initializers_succeed() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("module M { public class fun value(){7}; public class property x: Integer = M.value(); public class property y: Integer = M.x + 1 } M.y");
    assert_eq!(when, Ok(Value::Integer(8_u8.into())));
    let module = given.evaluator.module_names["M"];
    let backing = given.evaluator.module_classes[&module];
    let registry = given.evaluator.runtime.registry();
    let revision = registry.active_module(module).unwrap();
    assert_eq!(revision.number(), 1);
    assert_eq!(
        registry.active(backing).unwrap().commit_id(),
        revision.commit_id()
    );
    assert!(registry.staged_module(module).is_err());
    assert!(registry.staged_origin(backing).is_err());
}

#[test]
fn module_revision_survives_when_open_body_raises() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("module M { public fun value(){:old} } M")
        .unwrap();
    let module = given.evaluator.module_names["M"];
    let registry = given.evaluator.runtime.registry();
    let revision = registry.active_module(module).unwrap().clone();
    let when = given.evaluate("open module M { override public fun value(){:new}; raise :stop }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert_eq!(given.evaluator.module_names["M"], module);
    let registry = given.evaluator.runtime.registry();
    assert_eq!(registry.active_module(module).unwrap(), &revision);
    assert!(!given.evaluator.failed_modules.contains(&"M".into()));
    assert_eq!(given.evaluate("M.value()"), Ok(Value::Symbol("old".into())));
}

#[test]
fn initializer_failure_consumes_no_commit_or_revision_when_next_module_publishes() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("module Before { public fun value(){1} } Before")
        .unwrap();
    let before = given.evaluator.module_names["Before"];
    let registry = given.evaluator.runtime.registry();
    let revision = registry.active_module(before).unwrap().clone();
    let main = given.evaluator.module_mains[&before];
    let class_revision = registry.active(main).unwrap().clone();
    let bodies = given.evaluator.bodies.len();
    let when = given.evaluate("module Broken { public class fun fail(){raise :stop}; public fun value(){2}; shared class property first: Integer = 1; shared class property second: Integer = Broken.fail() }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert_eq!(given.evaluator.bodies.len(), bodies);
    given
        .evaluate("module After { public fun value(){3} } After")
        .unwrap();
    let after = given.evaluator.module_names["After"];
    let registry = given.evaluator.runtime.registry();
    let published = registry.active_module(after).unwrap();
    assert_eq!(published.id().raw(), revision.id().raw() + 1);
    assert_eq!(published.commit_id(), revision.commit_id() + 1);
    let main = given.evaluator.module_mains[&after];
    assert_eq!(
        registry.active(main).unwrap().id().raw(),
        class_revision.id().raw() + 2
    );
}

#[test]
fn module_is_unpublished_when_shared_binding_initializer_raises() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("module M { public class fun fail(){raise :stop}; shared let @@first = 1; shared let @@second = M.fail() }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert!(!given.evaluator.module_names.contains_key("M"));
    assert!(given.evaluator.module_classes.is_empty());
}

#[test]
fn module_state_survives_when_open_property_initializer_raises() {
    let mut given = Session::new().unwrap();
    given.evaluate("module M { public class fun fail(){raise :stop}; public class property x: Integer = 7 } M").unwrap();
    let module = given.evaluator.module_names["M"];
    let registry = given.evaluator.runtime.registry();
    let revision = registry.active_module(module).unwrap().clone();
    let when = given.evaluate("open module M { public class property x: Integer = 9; public class property y: Integer = M.fail() }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert_eq!(given.evaluate("M.x"), Ok(Value::Integer(7_u8.into())));
    let registry = given.evaluator.runtime.registry();
    assert_eq!(registry.active_module(module).unwrap(), &revision);
}

#[test]
fn initializer_failure_cleans_up_when_methods_are_instance_members() {
    let mut given = Session::new().unwrap();
    let bodies = given.evaluator.bodies.len();
    let when = given.evaluate(
        "module M { public fun fail(){raise :stop}; public class property x: Integer = M.fail() }",
    );
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert!(!given.evaluator.module_names.contains_key("M"));
    assert!(given.evaluator.module_mains.is_empty());
    assert_eq!(given.evaluator.bodies.len(), bodies);
}
