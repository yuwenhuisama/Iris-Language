use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::{ArrayRef, Value};

const DECORATORS: &str = r#"
class Cache {}
impl Cache for MethodDecorator {
    public fun plan(d,a) -> Plan { Plan.empty }
    public fun transform(d,a,c) -> Transformation {
        @cached = nil
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if @cached == nil { @cached = next.call() }; @cached
        })
    }
}
class Count {
    public fun offset() -> Integer { 2 }
}
impl Count for MethodDecorator {
    public fun plan(d,a) -> Plan { Plan.empty }
    public fun transform(d,a,c) -> Transformation {
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1; next.call() + calls + offset()
        })
    }
}
"#;

fn record(source: &str) -> RevisionArtifact {
    RevisionArtifact {
        package_id: "module.history".into(),
        api_major: 1,
        logical_owner: "Provider".into(),
        revision: 1,
        artifact: (
            "opaque:anything".into(),
            iris_runtime::artifact_digest(source.as_bytes()),
            source.into(),
        ),
    }
}

fn load(
    history: RevisionArtifact,
    current: &str,
    probe: &str,
) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "module.history",
                api_major: 1,
                version: None,
                locked: Vec::new(),
                artifact: Some(history.artifact.clone()),
                permissions: &[],
                grants: Vec::new(),
            },
            artifacts: vec![history],
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn fresh_chains_when_current_module_and_mixin_methods_are_retained() {
    let history = format!(
        "{DECORATORS} module Provider {{ @Cache() @Count() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; raise :unrelated"
    );
    let current = format!(
        "{DECORATORS} module Provider {{ @Cache() @Count() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; open module Provider {{ @Cache() @Count() public override fun value(n: Integer) -> Integer {{ n + 10 }} }}; class Client mixin Provider {{}}; open class Count {{ public override fun offset() -> Integer {{ 90 }} }}"
    );
    let when = load(
        record(&history),
        &current,
        "let client = Client.new(); let old = client.value; let warm = old.call(1); let module_old = Provider.method(:value); let first = Provider.rollback(1); let fresh = client.value(2); let module_fresh = Provider.value(99); let retained = old.call(9); let module_retained = Provider.invoke(module_old, Provider, %[9]); let second = Provider.rollback(1); %[warm, fresh, module_fresh, retained, module_retained, Provider.value(7), first == second]",
    );
    assert_eq!(
        when,
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(102u64.into()),
            Value::Integer(6u64.into()),
            Value::Integer(6u64.into()),
            Value::Integer(102u64.into()),
            Value::Integer(102u64.into()),
            Value::Integer(11u64.into()),
            Value::Bool(true),
        ]))))
    );
}

#[test]
fn unavailable_when_history_identity_or_digest_mismatches() {
    let source = "module Provider { public fun value() -> Integer { 1 } }";
    let mut records = vec![record(source); 5];
    records[0].artifact.1 = "bad".into();
    records[1].logical_owner = "Other".into();
    records[2].package_id = "other.package".into();
    records[3].api_major = 2;
    records[4].revision = 2;
    for history in records {
        let when = load(history, source, "Provider.rollback(1)");
        assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
    }
}

#[test]
fn current_body_survives_when_required_member_or_policy_rejects_history() {
    let history = "module Provider { public fun value() -> Integer { 1 } }";
    for current in [
        "module Provider meta deny method_body { public fun value() -> Integer { 9 } }",
        "module Provider { public fun value() -> Integer { 9 }; public fun later() -> Integer { 8 } }",
        "module Provider { public fun value(n: Integer) -> Integer { 9 } }",
    ] {
        let when = load(
            record(history),
            current,
            "try { Provider.rollback(1); false } catch error { true }",
        );
        assert_eq!(when, Ok(Some(Value::Bool(true))));
    }
}

#[test]
fn unavailable_when_exact_artifact_has_wrong_owner_kind() {
    let when = load(
        record("class Provider { public fun value() -> Integer { 1 } }"),
        "module Provider { public fun value() -> Integer { 9 } }",
        "Provider.rollback(1)",
    );
    assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn rollback_reason_and_module_method_when_historical_chain_is_rebuilt() {
    let decorator = "class Reason {} impl Reason for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { let reason = c.reason; Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; if reason == :rollback { next.call() + 10 } else { next.call() } }) } }";
    let history = format!(
        "{decorator} module Provider {{ @Reason() public module fun value() -> Integer {{ 1 }} }}"
    );
    let when = load(
        record(&history),
        &history,
        "let before = Provider.value(); let digest = Provider.rollback(); %[before, Provider.value()]",
    );
    assert_eq!(
        when,
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(1u64.into()),
            Value::Integer(11u64.into())
        ]))))
    );
}

#[test]
fn rejects_symbol_and_ambiguous_canonical_declarations_without_substitution() {
    let source = "module Provider { public fun value() -> Integer { 1 } }";
    for (history, probe) in [
        (source.to_owned(), "Provider.rollback(:old)"),
        (format!("{source}; {source}"), "Provider.rollback(1)"),
        (
            format!("{source}; class Provider {{}}"),
            "Provider.rollback(1)",
        ),
    ] {
        let when = load(record(&history), source, probe);
        assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
    }
}
