use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

const MEMBER: &str = include_str!("../../iris-cli/tests/decorator_targets/module_member.iris");
const ADDITION: &str = include_str!("../../iris-cli/tests/decorator_targets/module_addition.iris");
const ADD: &str = "class Add {}
impl Add for ModuleDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        let held = 40; mut calls = 0
        Transformation.add_method(:added, { |delta: Integer| -> Integer;
            calls = calls + 1; held + delta + calls
        })
    }
}; 0";

#[test]
fn module_member_fixture_executes_when_wrapped() {
    let given = MEMBER;
    let mut session = Session::new().unwrap();
    session.evaluate(given).unwrap();
    let when = session.evaluate("Target.new().value(9)");
    assert_eq!(when, evaluate("19"));
}

#[test]
fn module_addition_fixture_executes_when_captured() {
    let given = ADDITION;
    let when = evaluate(given);
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn origin_name_is_absent_when_transform_raises() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stop {} impl Stop for ModuleDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { raise :abort } }; 0").unwrap();
    let when = given.evaluate("@Stop() module Provider { public fun value() { 7 } }; 0");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluate("Provider"), Err(EvaluationError::NameError));
}

#[test]
fn addition_is_denied_when_method_set_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate(ADD).unwrap();
    let when = given.evaluate("@Add() module Provider meta deny method_set { }; 0");
    assert!(when.is_err(), "{when:?}");
    assert_eq!(given.evaluate("Provider"), Err(EvaluationError::NameError));
}

#[test]
fn wrapper_is_denied_when_method_body_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Wrap {} impl Wrap for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.wrap_method({ |inv: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() }) } }; 0").unwrap();
    let when = given
        .evaluate("module Provider meta deny method_body { @Wrap() public fun value() { 7 } }; 0");
    assert!(when.is_err(), "{when:?}");
    assert_eq!(given.evaluate("Provider"), Err(EvaluationError::NameError));
}

#[test]
fn private_addition_is_denied_when_called_externally() {
    let mut given = Session::new().unwrap();
    given.evaluate(ADD).unwrap();
    given
        .evaluate("@Add() module Provider {}; class Target mixin Provider {}; 0")
        .unwrap();
    let when = given.evaluate("try { Target.new().added(1) } catch error { error }");
    assert_eq!(when, Ok(Value::Symbol("MethodVisibilityError".into())));
}

#[test]
fn class_addition_preserves_captures_when_installed() {
    let given = include_str!("../../iris-cli/tests/decorator_targets/class_addition.iris");
    let when = evaluate(given);
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn planner_rejects_transitive_effects_when_module_is_decorated() {
    let given = "class Bad { fun helper() { print(1) } } impl Bad for ModuleDecorator { public fun plan(d,a) -> Plan { self.helper(); Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } }; @Bad() module Provider {}; 0";
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-NONDETERMINISTIC",
            phase: "static"
        })
    );
}

#[test]
fn metadata_rejects_mutation_when_module_is_transformed() {
    let given = "class Bad {} impl Bad for ModuleDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { d.name = :Changed; Transformation.empty } }; @Bad() module Provider {}; 0";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::ReadonlyMutation));
}

#[test]
fn phases_construct_fresh_instances_when_module_is_decorated() {
    let given = "class Fresh { fun initialize() { @phase = 0 } } impl Fresh for ModuleDecorator { public fun plan(d,a) -> Plan { @phase = 1; Plan.empty }; public fun transform(d,a,c) -> Transformation { if @phase != 0 { raise :reused }; Transformation.empty } }; @Fresh() module Provider {}; 7";
    let when = evaluate(given);
    assert_eq!(when, evaluate("7"));
}
