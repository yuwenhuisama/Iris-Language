use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

fn decorator(contract: &str, initialize: &str, plan: &str, transform: &str) -> String {
    format!(
        "class Stamp {{ {initialize} }} impl Stamp for {contract} {{ public fun plan(declaration, arguments) -> Plan {{ {plan} }}; public fun transform(declaration, arguments, context) -> Transformation {{ {transform} }} }}"
    )
}

#[test]
fn construction_is_fresh_for_each_phase() {
    let source = decorator(
        "ClassDecorator",
        "fun initialize() { @seen = 0; @kind = Plan.empty.kind };",
        "@seen = 1; Plan.empty",
        "if @seen != 0 { raise :shared }; if @kind != :class { raise :kind }; Transformation.empty",
    );
    assert_eq!(
        evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7")),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn planning_raise_precedes_runtime_statements() {
    let source = decorator("ClassDecorator", "", "raise :planning", "raise :runtime");
    assert_eq!(
        evaluate(&format!(
            "{source}; raise :earlier_runtime; @Stamp() class Target {{ }}; 7"
        )),
        Err(EvaluationError::Raised(Value::Symbol("planning".into())))
    );
}

#[test]
fn method_planning_raise_precedes_runtime_statements() {
    let source = decorator("MethodDecorator", "", "raise :planning", "raise :runtime");
    assert_eq!(
        evaluate(&format!(
            "{source}; raise :earlier_runtime; class Target {{ @Stamp() public fun value() {{ 7 }} }}; 7"
        )),
        Err(EvaluationError::Raised(Value::Symbol("planning".into())))
    );
}

#[test]
fn each_runtime_application_constructs_a_fresh_instance() {
    let source = decorator(
        "ClassDecorator",
        "fun initialize() { @used = false };",
        "Plan.empty",
        "if @used { raise :shared }; @used = true; Transformation.empty",
    );
    assert_eq!(
        evaluate(&format!(
            "{source}; @Stamp() @Stamp() class Target {{ }}; 7"
        )),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn planning_rejects_impure_constructors_and_transitive_helpers() {
    for initialize in [
        "fun initialize() { print(:leak) };",
        "fun initialize() { self.helper() }; fun helper() { print(:leak) };",
        "fun initialize() { @value = $ambient };",
        "fun initialize() { File.read(\"missing\") };",
    ] {
        let source = decorator(
            "ClassDecorator",
            initialize,
            "Plan.empty",
            "Transformation.empty",
        );
        let error = evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7")).unwrap_err();
        assert!(
            format!("{error:?}").contains("IRIS-DECORATOR-NONDETERMINISTIC"),
            "{error:?}"
        );
    }
}

#[test]
fn method_transform_receives_runtime_context() {
    let source = decorator(
        "MethodDecorator",
        "",
        "if declaration.selector != :value { raise :metadata }; Plan.empty",
        "if context.kind != :method { raise :kind }; if context.reason != :origin { raise :reason }; Transformation.empty",
    );
    assert_eq!(
        evaluate(&format!(
            "{source}; class Target {{ @Stamp() public fun value() {{ 7 }} }}; Target.new().value()"
        )),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn wrong_target_contract_is_rejected_before_runtime() {
    let source = decorator("MethodDecorator", "", "Plan.empty", "Transformation.empty");
    let error = evaluate(&format!(
        "{source}; raise :runtime; @Stamp() class Target {{ }}; 7"
    ))
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("IRIS-DECORATOR-KIND"),
        "{error:?}"
    );
}

#[test]
fn class_transform_receives_open_reason() {
    let source = decorator(
        "ClassDecorator",
        "",
        "Plan.empty",
        "if context.kind != :class { raise :kind }; if context.reason != :open { raise :reason }; Transformation.empty",
    );
    assert_eq!(
        evaluate(&format!(
            "{source}; class Target {{ }}; @Stamp() open class Target {{ }}; 7"
        )),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn plan_factory_helpers_inherit_constructor_scope() {
    let source = decorator(
        "ClassDecorator",
        "fun initialize() { @plan = self.helper() }; fun helper() { Plan.empty };",
        "@plan",
        "Transformation.empty",
    );
    assert_eq!(
        evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7")),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn failed_transform_does_not_retain_phase_authority() {
    let mut session = iris_eval::Session::new().unwrap();
    let source = decorator("ClassDecorator", "", "Plan.empty", "raise :failure");
    assert!(
        session
            .evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7"))
            .is_err()
    );
    assert_eq!(
        session.evaluate("try { Plan.empty } catch error { error.category }"),
        Ok(Value::Symbol("outside_phase".into()))
    );
}

#[test]
fn wrong_phase_result_is_rejected() {
    let source = decorator("ClassDecorator", "", "Plan.empty", "nil");
    assert!(evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7")).is_err());
}

#[test]
fn metadata_cannot_mutate_candidate() {
    let source = decorator(
        "ClassDecorator",
        "",
        "Plan.empty",
        "declaration.name = :Changed; Transformation.empty",
    );
    assert!(matches!(
        evaluate(&format!("{source}; @Stamp() class Target {{ }}; 7")),
        Err(EvaluationError::ReadonlyMutation)
    ));
}

#[test]
fn runtime_output_is_absent_when_planning_fails() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "planning_output_child", "--nocapture"])
        .env("IRIS_PHASE_OUTPUT_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("EFFECT_LEAK"));
}

#[test]
fn planning_output_child() {
    if std::env::var_os("IRIS_PHASE_OUTPUT_CHILD").is_none() {
        return;
    }
    for initialize in [
        "fun initialize() { print(:EFFECT_LEAK) };",
        "fun initialize() { self.helper() }; fun helper() { print(:EFFECT_LEAK) };",
    ] {
        let source = decorator(
            "ClassDecorator",
            initialize,
            "Plan.empty",
            "Transformation.empty",
        );
        assert!(
            evaluate(&format!(
                "print(:EFFECT_LEAK); {source}; @Stamp() class Target {{ }}; 7"
            ))
            .is_err()
        );
    }
    let source = decorator(
        "ClassDecorator",
        "",
        "raise :planning",
        "Transformation.empty",
    );
    assert!(
        evaluate(&format!(
            "print(:EFFECT_LEAK); {source}; @Stamp() class Target {{ }}; 7"
        ))
        .is_err()
    );
}
