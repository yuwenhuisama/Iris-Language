use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

const EMPTY: &str = "class Stamp { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } } impl Stamp for ContractDecorator {}; 0";

#[test]
fn reference_fixture_executes_when_contract_is_decorated() {
    let given = include_str!("../../iris-cli/tests/decorator_targets/contract_empty.iris");
    let when = evaluate(given);
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn planning_is_required_when_contract_is_decorated() {
    let given = "class Bad { public fun transform(d,a,c) -> Transformation { Transformation.empty } } impl Bad for ContractDecorator {}; @Bad() contract Named {}; 0";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn transitive_effects_are_rejected_when_exported_decorator_is_planned() {
    let given = "export class Bad { fun helper() { print(1) } public fun plan(d,a) -> Plan { self.helper(); Plan.empty } public fun transform(d,a,c) -> Transformation { Transformation.empty } } impl Bad for ContractDecorator {}; @Bad() contract Named {}; 0";
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
fn wrong_target_kind_is_rejected_when_contract_is_planned() {
    let given = "class Bad { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } } impl Bad for ModuleDecorator {}; @Bad() contract Named {}; 0";
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "static"
        })
    );
}

#[test]
fn operations_are_rejected_when_contract_is_transformed() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Bad { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.add_method(:added, { || -> Integer; 1 }) } } impl Bad for ContractDecorator {}; 0").unwrap();
    let when = given.evaluate("@Bad() contract Named { fun name() -> Symbol }; 0");
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "candidate validation"
        })
    );
    assert_eq!(given.evaluate("Named"), Err(EvaluationError::NameError));
}

#[test]
fn instances_are_fresh_when_each_contract_phase_executes() {
    let given = "class Fresh { fun initialize() { @phase = 0 } public fun plan(d,a) -> Plan { if @phase != 0 { raise :reused }; @phase = 1; Plan.empty } public fun transform(d,a,c) -> Transformation { if @phase != 0 { raise :reused }; @phase = 2; Transformation.empty } } impl Fresh for ContractDecorator {}; @Fresh() @Fresh() contract Named {}; 7";
    let when = evaluate(given);
    assert_eq!(when, evaluate("7"));
}

#[test]
fn retained_metadata_is_immutable_when_phase_has_finished() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Effects { public class property held: Object = nil } class Keep { public fun plan(d,a) -> Plan { if d.requirements[0] != :zeta { raise :order }; Plan.empty } public fun transform(d,a,c) -> Transformation { Effects.held = d; Transformation.empty } } impl Keep for ContractDecorator {}; @Keep() contract Named { fun zeta() -> Symbol; fun alpha() -> Integer }; 0").unwrap();
    let when = given.evaluate("Effects.held.requirements[0] = :changed");
    assert_eq!(when, Err(EvaluationError::ReadonlyMutation));
    assert_eq!(
        given.evaluate("Effects.held.name = :Changed"),
        Err(EvaluationError::ReadonlyMutation)
    );
    assert_eq!(
        given.evaluate("Effects.held.requirements[0]"),
        Ok(Value::Symbol("zeta".into()))
    );
    assert_eq!(
        given.evaluate("Named.reflect.decorators[0]"),
        Ok(Value::Symbol("Keep".into()))
    );
}

#[test]
fn origin_name_is_hidden_when_transform_attempts_lookup() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Lookup { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Named; Transformation.empty } } impl Lookup for ContractDecorator {}; 0").unwrap();
    let when = given.evaluate("@Lookup() contract Named {}; 0");
    assert_eq!(when, Err(EvaluationError::NameError));
    assert_eq!(given.evaluate("Named"), Err(EvaluationError::NameError));
}

#[test]
fn existing_contract_is_preserved_when_replacement_transform_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate(EMPTY).unwrap();
    given.evaluate("@Stamp() contract Named { fun old() -> Integer }; class Stop { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { raise :abort } } impl Stop for ContractDecorator {}; 0").unwrap();
    let original = given.evaluate("Named").unwrap();
    let when = given.evaluate("@Stop() contract Named { fun replacement() -> Symbol }; 0");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluate("Named"), Ok(original));
    assert_eq!(
        given.evaluate("Named.reflect.requirements[0]"),
        Ok(Value::Symbol("old".into()))
    );
}

#[test]
fn retained_wrong_kind_is_rejected_when_contract_transform_returns_it() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Effects { public class property held: Object = nil } class Keep { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Effects.held = Transformation.empty; Transformation.empty } } impl Keep for ClassDecorator {}; @Keep() class Target {}; class Wrong { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Effects.held } } impl Wrong for ContractDecorator {}; 0").unwrap();
    let when = given.evaluate("@Wrong() contract Named {}; 0");
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "candidate validation"
        })
    );
    assert_eq!(given.evaluate("Named"), Err(EvaluationError::NameError));
}
