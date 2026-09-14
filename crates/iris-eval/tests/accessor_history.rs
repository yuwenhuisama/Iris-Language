use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::Value;

const DECORATORS: &str = r#"
class Read {}
impl Read for PropertyDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        mut calls = 0
        Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1; next.call() + calls
        })
    }
}
class Write {}
impl Write for PropertyDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        Transformation.wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            next.call(ArgumentChanges.new(positional: %{:value: 7}))
        })
    }
}
class Count {}
impl Count for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        @cache = nil
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if @cache == nil { @cache = next.call() }; @cache
        })
    }
}
"#;

const TARGET: &str = r#"class Target {
    public fun initialize() { @state = 1 }
    @Read() public property fun value() -> Integer { @state + 10 }
    @Write() public property fun value=(value: Integer) -> Symbol { @state = value; :historical }
    @Count() public class fun count(value: Integer) -> Integer { value + 10 }
    @Count() public fun retained(value: Integer) -> Integer { value + 10 }
}"#;

fn load(history: &str, current: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    let artifact = (
        "opaque-host-key".into(),
        iris_runtime::artifact_digest(history.as_bytes()),
        history.into(),
    );
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "accessor.history",
                api_major: 1,
                version: None,
                locked: vec![],
                artifact: None,
                permissions: &[],
                grants: vec![],
            },
            artifacts: vec![RevisionArtifact {
                package_id: "accessor.history".into(),
                api_major: 1,
                logical_owner: "Target".into(),
                revision: 1,
                artifact,
            }],
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn restores_accessor_bodies_when_existing_raw_state_is_preserved() {
    let given = format!("{DECORATORS} {TARGET}");
    let current = given
        .replace("+ 10", "+ 100")
        .replace(":historical", ":current");
    let when = load(
        &given,
        &current,
        r#"
        let target = Target.new(); let written = target.value = 99
        let revision = Target.active_revision
        let commit = Reflection::Class.revision(Target).fetch(:commit_id)
        let result = Target.rollback(1)
        written == :current && target.value == 18 && (target.value = 2) == :historical && target.value == 19 && Target.count(3) == 13 && Target.active_revision == revision + 1 && Reflection::Class.revision(Target).fetch(:commit_id) == commit + 1
    "#,
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn refreshes_each_slot_when_same_artifact_is_replayed_twice() {
    let given = format!("{DECORATORS} {TARGET}");
    let when = load(
        &given,
        &given,
        r#"
        let target = Target.new(); let old = target.retained
        let warm = old.call(1); let singleton_warm = Target.count(1); let first = Target.rollback(1)
        let retained = target.retained; let once = retained.call(2); let singleton_once = Target.count(2)
        let getter = target.value; let getter_again = target.value
        let second = Target.rollback(1)
        warm == 11 && once == 12 && singleton_warm == 11 && singleton_once == 12 && getter == 12 && getter_again == 13 && target.value == 12 && Target.count(3) == 13 && Target.count(99) == 13 && old.call(99) == 11 && retained.call(99) == 12 && first == second
    "#,
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn uses_property_body_when_method_mutation_is_denied() {
    let given = "class Target { public property fun value() -> Integer { 1 }; public property fun value=(value: Integer) -> Symbol { :written } }";
    for policy in ["method_body", "method_set", "property_set"] {
        let current = given.replace(
            "class Target {",
            &format!("class Target meta deny {policy} {{"),
        );
        let when = load(
            given,
            &current,
            "let result = Target.rollback(1); let target = Target.new(); target.value == 1 && (target.value = 2) == :written",
        );
        assert_eq!(when, Ok(Some(Value::Bool(true))), "{policy}");
    }
}

#[test]
fn denies_property_replay_when_property_body_is_denied() {
    let given = "class Target { public property fun value() -> Integer { 1 } }";
    let current =
        "class Target meta deny property_body { public property fun value() -> Integer { 9 } }";
    let when = load(
        given,
        current,
        "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; rejected && Target.new().value == 9 && Target.active_revision == before",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn checks_setter_result_when_historical_wrapper_skips_body() {
    let current = format!("{DECORATORS} {TARGET}");
    let history = current.replace(
        "next.call(ArgumentChanges.new(positional: %{:value: 7}))",
        "42",
    );
    let when = load(
        &history,
        &current,
        "let result = Target.rollback(1); let target = Target.new(); try { target.value = 3; false } catch error { error is? TypeError }",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn uses_method_body_when_existing_singleton_is_replayed() {
    let given = "class Target { public class fun count() -> Integer { 1 } }";
    let current = "class Target meta deny method_set { public class fun count() -> Integer { 9 } }";
    let when = load(
        given,
        current,
        "let result = Target.rollback(1); Target.count() == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
    let denied = current.replace("method_set", "method_body");
    let when = load(
        given,
        &denied,
        "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; rejected && Target.count() == 9 && Target.active_revision == before",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn matches_member_kind_when_instance_and_singleton_share_selector() {
    let given = "class Target { public fun value() -> Integer { 1 }; public class fun value() -> Integer { 2 } }";
    let current = given.replace("{ 1 }", "{ 8 }").replace("{ 2 }", "{ 9 }");
    let when = load(
        given,
        &current,
        "let result = Target.rollback(1); Target.value() == 2 && Target.new().value() == 1",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn rejects_incomplete_artifact_when_a_property_or_singleton_is_missing() {
    for member in [
        "public property fun value() -> Integer { 9 }",
        "public property fun value=(value: Integer) -> Symbol { :written }",
        "public class fun value() -> Integer { 9 }",
    ] {
        let current = format!("class Target {{ public fun plain() {{ 1 }}; {member} }}");
        let when = load(
            "class Target { public fun plain() { 2 } }",
            &current,
            "Target.rollback(1)",
        );
        assert_eq!(when, Err(EvaluationError::TypeContractError), "{member}");
    }
}

#[test]
fn rejects_class_level_storage_when_artifact_omits_or_redeclares_storage() {
    for current in ["class Target { public class property value: Integer = 1 }"] {
        for history in [current, "class Target { }"] {
            let when = load(history, current, "Target.rollback(1)");
            assert_eq!(when, Err(EvaluationError::UnsupportedConstruct));
        }
    }
}

#[test]
fn preserves_singleton_when_a_later_accessor_transform_fails() {
    let given = format!("{DECORATORS} {TARGET}");
    let history = given
        .replace(
            "next.call(ArgumentChanges.new(positional: %{:value: 7}))",
            "next.call()",
        )
        .replace(
            "Transformation.wrap_setter(",
            "raise :failed; Transformation.wrap_setter(",
        );
    let when = load(
        &history,
        &given,
        "let old = Target.new().retained; let warm = old.call(1); let singleton_warm = Target.count(1); let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { error == :failed }; rejected && Target.count(99) == 11 && old.call(99) == 11 && Target.active_revision == before",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}
