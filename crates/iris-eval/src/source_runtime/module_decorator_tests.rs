use crate::{EvaluationError, Session, evaluate};
use iris_runtime::{MethodOwner, Value};

#[test]
fn backing_revisions_are_unpublished_when_later_transform_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stop {}; impl Stop for ModuleDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { raise :abort } }; class Stamp {}; impl Stamp for ModuleDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } }; 0").unwrap();
    let before = given.evaluator.class_name("Stamp").unwrap().unwrap();
    let revision = given
        .evaluator
        .runtime
        .registry()
        .active(before)
        .unwrap()
        .clone();
    let mains = given.evaluator.module_mains.len();
    let classes = given.evaluator.module_classes.len();
    let when = given.evaluate("@Stop() module Broken { public class property state: Integer = 7; public fun value() { 1 } }; 0");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluator.module_mains.len(), mains);
    assert_eq!(given.evaluator.module_classes.len(), classes);
    given
        .evaluate("@Stamp() module Good { public fun value() { 2 } }; 0")
        .unwrap();
    let module = given.evaluator.module_names["Good"];
    let backing = given.evaluator.module_classes[&module];
    let main = given.evaluator.module_mains[&module];
    let published = given
        .evaluator
        .runtime
        .registry()
        .active_module(module)
        .unwrap();
    assert_eq!(published.commit_id(), revision.commit_id() + 1);
    for class in [backing, main] {
        let active = given.evaluator.runtime.registry().active(class).unwrap();
        assert_eq!(active.commit_id(), published.commit_id());
        assert!(active.id().raw() <= revision.id().raw() + 2);
    }
}

#[test]
fn captured_addition_executes_with_original_receiver_when_bound_by_backend() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Add { fun initialize() { @seed = 30 } }; impl Add for ModuleDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { let held = 10; mut calls = 0; Transformation.add_method(:added, { |delta: Integer| -> Integer; calls = calls + 1; @seed + held + delta + calls }) } }; @Add() module Provider {}; 0").unwrap();
    let module = given.evaluator.module_names["Provider"];
    let selector = given.evaluator.selector("added");
    let method = given
        .evaluator
        .runtime
        .registry()
        .module_method(module, selector)
        .unwrap();
    assert_eq!(method.owner(), MethodOwner::Module(module));
    let when = given
        .evaluator
        .invoke_method(method, Value::Nil, &[Value::Integer(2_u8.into())]);
    assert_eq!(when, evaluate("43"));
    assert_eq!(
        given
            .evaluator
            .invoke_method(method, Value::Nil, &[Value::Integer(2_u8.into())]),
        evaluate("44")
    );
}
