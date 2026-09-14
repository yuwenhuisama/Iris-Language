use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::Value;

const CACHE: &str = r#"class Cache {}
impl Cache for PropertyDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        @cached = nil
        Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if @cached == nil { @cached = next.call() }; @cached
        })
    }
}"#;

fn load(history: &str, current: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "stored.history",
                api_major: 1,
                version: None,
                locked: vec![],
                artifact: None,
                permissions: &[],
                grants: vec![],
            },
            artifacts: vec![RevisionArtifact {
                package_id: "stored.history".into(),
                api_major: 1,
                logical_owner: "Target".into(),
                revision: 1,
                artifact: (
                    "opaque".into(),
                    iris_runtime::artifact_digest(history.as_bytes()),
                    history.into(),
                ),
            }],
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn preserves_existing_state_when_historical_initializer_changes() {
    let given = "class Target { public property value: Integer = 1 }";
    let when = load(
        given,
        &given.replace("= 1", "= 4"),
        "let target = Target.new(); let written = target.value = 9; let before = Target.active_revision; let commit = Reflection::Class.revision(Target).fetch(:commit_id); let result = Target.rollback(1); target.value == 9 && Target.new().value == 1 && Target.active_revision == before + 1 && Reflection::Class.revision(Target).fetch(:commit_id) == commit + 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn rebuilds_fresh_cache_when_stored_accessors_are_restored() {
    let given = format!("{CACHE} class Target {{ @Cache() public property value: Integer = 1 }}");
    let when = load(
        &given,
        &given.replace("= 1", "= 4"),
        "let target = Target.new(); let warm = target.value; let written = target.value = 9; let first = Target.rollback(1); let fresh = target.value; let rewritten = target.value = 12; let cached = target.value; let second = Target.rollback(1); warm == 4 && fresh == 9 && cached == 9 && target.value == 12",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn uses_property_body_when_other_mutation_lanes_are_denied() {
    let given = format!("{CACHE} class Target {{ @Cache() public property value: Integer = 1 }}");
    let current = given.replace(
        "class Target {",
        "class Target meta deny method_body, method_set, property_set {",
    );
    let when = load(
        &given,
        &current,
        "let result = Target.rollback(1); Target.new().value == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn executes_initializers_only_for_new_instances_when_replaying() {
    let given = "class Effects { public class property count: Integer = 0 }; class Target { public property value: Integer = seed(); public fun seed() -> Integer { Effects.count = Effects.count + 1; 1 } }";
    let current = given.replace("; 1 }", "; 4 }");
    let when = load(
        given,
        &current,
        "let target = Target.new(); let written = target.value = 9; let result = Target.rollback(1); let count = Effects.count; let fresh = Target.new(); count == 1 && Effects.count == 2 && target.value == 9 && fresh.value == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn rejects_storage_shape_changes_before_publication() {
    let given = "class Target { property value: Integer = 1 }";
    for history in [
        "class Target { }",
        "class Target { property value: Object = 1 }",
        "class Target { property value: Integer = 1; property added: Integer = 2 }",
    ] {
        let when = load(history, given, "Target.rollback(1)");
        assert_eq!(when, Err(EvaluationError::TypeContractError), "{history}");
    }
}

#[test]
fn preserves_initializer_effects_when_policy_rejects_replay() {
    let given = "class Effects { public class property count: Integer = 0 }; class Target { public property value: Integer = seed(); public fun seed() -> Integer { Effects.count = Effects.count + 1 } }";
    let current = given.replace("class Target {", "class Target meta deny property_body {");
    let when = load(
        given,
        &current,
        "let target = Target.new(); let written = target.value = 9; let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; rejected && Effects.count == 1 && target.value == 9 && Target.active_revision == before",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn restores_storage_when_current_accessor_is_an_explicit_replacement() {
    let given = "class Target { public property value: Integer = 1 }";
    let current = format!(
        "{given}; open class Target {{ public override property fun value() -> Integer {{ @value + 100 }} }}"
    );
    let when = load(
        given,
        &current,
        "let target = Target.new(); let written = target.value = 9; let before = target.value; let result = Target.rollback(1); before == 109 && target.value == 9 && Target.new().value == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn retains_current_initializer_when_historical_transform_fails() {
    let given = format!("{CACHE} class Target {{ @Cache() public property value: Integer = 1 }}");
    let history = given.replace("@cached = nil", "raise :failed; @cached = nil");
    let current = given.replace("= 1", "= 4");
    let when = load(
        &history,
        &current,
        "let target = Target.new(); let written = target.value = 9; let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { error == :failed }; let fresh = Target.new(); rejected && Target.active_revision == before && fresh.value == 4",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_construction_snapshot_when_initializer_triggers_rollback() {
    let given = "class Target { property first: Integer = trigger(); public property second: Integer = 1; public fun trigger() -> Integer { let result = Target.rollback(1); 0 } }";
    let current = given.replace("second: Integer = 1", "second: Integer = 4");
    let when = load(
        given,
        &current,
        "let first = Target.new(); let second = Target.new(); first.second == 4 && second.second == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn rejects_reordered_or_missing_accessors_when_current_spine_requires_them() {
    let given = "class Target { property first: Integer = 1; property second: Integer = 2 }";
    for history in [
        "class Target { property second: Integer = 2; property first: Integer = 1 }",
        "class Target { property first: Integer = 1 { public get; }; property second: Integer = 2 }",
    ] {
        let when = load(history, given, "Target.rollback(1)");
        assert_eq!(when, Err(EvaluationError::TypeContractError));
    }
}
