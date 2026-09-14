use iris_eval::{
    PackageHistory, PackageResolution, PackageUpgrade, ResolvedPackageVersion,
    load_resolved_package_with_upgrades,
};
use iris_runtime::Value;

fn load(
    current: &str,
    target: &str,
    probe: &str,
) -> Result<Option<Value>, iris_eval::EvaluationError> {
    load_versions(
        current,
        vec![ResolvedPackageVersion {
            package_id: "upgrade.test".into(),
            api_major: 1,
            version: "1.1".into(),
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(target.as_bytes()),
                target.into(),
            ),
        }],
        probe,
    )
}

fn load_versions(
    current: &str,
    versions: Vec<ResolvedPackageVersion>,
    probe: &str,
) -> Result<Option<Value>, iris_eval::EvaluationError> {
    load_resolved_package_with_upgrades(
        PackageUpgrade {
            history: PackageHistory {
                resolution: PackageResolution {
                    package_id: "upgrade.test",
                    api_major: 1,
                    version: Some("1.0".into()),
                    locked: vec![],
                    artifact: None,
                    permissions: &[],
                    grants: vec![],
                },
                artifacts: vec![],
            },
            versions,
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn replaces_all_owners_when_exact_artifact_is_selected() {
    let current = "class First { public fun value() -> Integer { 1 } }; module Second { public fun value() -> Integer { 2 } }";
    let target = "class First { public fun value() -> Integer { 10 } }; module Second { public fun value() -> Integer { 20 } }";
    let when = load(
        current,
        target,
        "let old = First.new().value; let identity = First; let result = Reflection::Package.upgrade(:\"1.1\"); First.new().value() == 10 && Second.value() == 20 && old.call() == 1 && First == identity && Reflection::Package.version() == :\"1.1\"",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

const CACHE: &str = r#"class Cache {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        @cached = nil
        if c.reason == :upgrade { @factor = 2 } else { @factor = 3 }
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if @cached == nil { @cached = next.call() * @factor }; @cached
        })
    }
}
impl Cache for MethodDecorator {}"#;

#[test]
fn rebuilds_fresh_captures_when_old_method_is_retained() {
    let current = format!(
        "{CACHE} class First {{ @Cache() public fun value(n: Integer) -> Integer {{ n }} }}"
    );
    let target = format!(
        "{CACHE} class First {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }} }}"
    );
    let when = load(
        &current,
        &target,
        "let instance = First.new(); let old = instance.value; let warm = old.call(1); let result = Reflection::Package.upgrade(:\"1.1\"); warm == 3 && instance.value(2) == 24 && instance.value(99) == 24 && old.call(99) == 3",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_first_owner_when_late_transform_fails() {
    let bad = "class Bad { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :late } } impl Bad for MethodDecorator {}";
    let current = format!(
        "{CACHE} {bad} class First {{ @Cache() public fun value(n: Integer) -> Integer {{ n }} }}; class Last {{ public fun value() -> Integer {{ 9 }} }}"
    );
    let target = format!(
        "{CACHE} {bad} class First {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }} }}; class Last {{ @Bad() public fun value() -> Integer {{ 2 }} }}"
    );
    let when = load(
        &current,
        &target,
        "let instance = First.new(); let old = instance.value; let warm = old.call(1); let before = First.active_revision; let rejected = try { Reflection::Package.upgrade(:\"1.1\"); false } catch error { error == :late }; rejected && instance.value(99) == 3 && old.call(99) == 3 && First.active_revision == before && Last.new().value() == 9 && Reflection::Package.version() == :\"1.0\"",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_root_and_external_effect_when_candidate_hook_fails() {
    let state = "class State { public shared class property root: Array = %[1]; public shared class property log: Array = %[]; public shared class property count: Integer = 0 }; module Init {}; open module Init { State.count = State.count + 1 };";
    let current = format!(
        "{state} class First {{ public fun value() -> Integer {{ 1 }} }}; module Upgrade {{ public fun upgrade(older, context) {{ nil }} }}"
    );
    let target = format!(
        "{state} class First {{ public fun value() -> Integer {{ 2 }} }}; module Upgrade {{ public fun upgrade(older, context) {{ State.root = %[9]; State.log.append(:effect); raise :hook }} }}"
    );
    let when = load(
        &current,
        &target,
        "let root = State.root; let before = First.active_revision; let result = try { Reflection::Package.upgrade(:\"1.1\") } catch error { error }; result == :hook && State.root.same?(root) && State.root[0] == 1 && State.log.length() == 1 && State.log[0] == :effect && State.count == 1 && First.new().value() == 1 && First.active_revision == before && Reflection::Package.version() == :\"1.0\"",
    );
    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn skips_existing_initializer_when_upgrade_succeeds() {
    let state = "class State { public shared class property count: Integer = 0 }; module Init {}; open module Init { State.count = State.count + 1 };";
    let current = format!("{state} class First {{ public fun value() {{ 1 }} }}");
    let target = format!("{state} class First {{ public fun value() {{ 2 }} }}");
    assert_eq!(
        load(
            &current,
            &target,
            "let result = Reflection::Package.upgrade(:\"1.1\"); State.count == 1 && First.new().value() == 2"
        ),
        Ok(Some(Value::Bool(true)))
    );
}

#[test]
fn rejects_unsupported_owner_when_other_owner_is_compatible() {
    let current = "class First { public fun value() -> Integer { 1 } }; class Last { public property state: Integer = 1 }";
    let target = "class First { public fun value() -> Integer { 2 } }; class Last { public property state: Integer = 1 }";
    assert_eq!(
        load(current, target, "Reflection::Package.upgrade(:\"1.1\")"),
        Err(iris_eval::EvaluationError::UnsupportedConstruct)
    );
}

#[test]
fn rejects_duplicate_version_before_hook_effects() {
    let source = "module Upgrade { public fun upgrade(older, newer) { raise :effect } }";
    let record = ResolvedPackageVersion {
        package_id: "upgrade.test".into(),
        api_major: 1,
        version: "1.1".into(),
        artifact: (
            "opaque".into(),
            iris_runtime::artifact_digest(source.as_bytes()),
            source.into(),
        ),
    };
    assert_eq!(
        load_versions(
            source,
            vec![record.clone(), record],
            "Reflection::Package.upgrade(:\"1.1\")"
        ),
        Err(iris_eval::EvaluationError::RevisionArtifactUnavailable)
    );
}

#[test]
fn rejects_bad_digest_before_hook_effects() {
    let source = "module Upgrade { public fun upgrade(older, newer) { raise :effect } }";
    let record = ResolvedPackageVersion {
        package_id: "upgrade.test".into(),
        api_major: 1,
        version: "1.1".into(),
        artifact: ("opaque".into(), "bad".into(), source.into()),
    };
    assert_eq!(
        load_versions(
            source,
            vec![record],
            "Reflection::Package.upgrade(:\"1.1\")"
        ),
        Err(iris_eval::EvaluationError::RevisionArtifactUnavailable)
    );
}

#[test]
fn module_chain_uses_upgrade_reason_when_replayed() {
    let current = format!(
        "{CACHE} module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n }} }}"
    );
    let target = format!(
        "{CACHE} module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }} }}"
    );
    assert_eq!(
        load(
            &current,
            &target,
            "let warm = Provider.value(1); let result = Reflection::Package.upgrade(:\"1.1\"); warm == 3 && Provider.value(2) == 24 && Provider.value(99) == 24"
        ),
        Ok(Some(Value::Bool(true)))
    );
}

#[test]
fn publishes_candidate_root_when_hook_succeeds() {
    let current = "class State { public shared class property root: Array = %[1] }; module Upgrade { public fun upgrade(older, newer) { nil } }";
    let target = current.replace("{ nil }", "{ State.root = %[9]; State.root[0] }");
    assert_eq!(
        load(
            current,
            &target,
            "let old = State.root; let result = Reflection::Package.upgrade(:\"1.1\"); result == 9 && State.root[0] == 9 && old[0] == 1 && Reflection::Package.version() == :\"1.1\""
        ),
        Ok(Some(Value::Bool(true)))
    );
}
