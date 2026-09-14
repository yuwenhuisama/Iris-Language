use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::Value;

const DECORATORS: &str = r#"
class Effects {
    public class property phases: Array = %[]
    public class property bodies: Integer = 0
}
class Count {
    public fun initialize() { @phase = 0 }
    public fun offset() -> Integer { 1 }
}
impl Count for MethodDecorator {
    public fun plan(d, a) -> Plan { @phase = 1; Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        if @phase != 0 { raise :reused_instance }
        @phase = 2
        Effects.phases.append(c.reason)
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            let value = next.call()
            if invocation.method_type_arguments[0] != invocation.signature.parameters[0].type { raise :metadata }
            value + calls + offset()
        })
    }
}
class Twice {}
impl Twice for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            next.call() * 2
        })
    }
}
"#;

fn load(history: &str, current: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    load_artifact(
        Some((iris_runtime::artifact_digest(history.as_bytes()), history)),
        current,
        probe,
    )
}

fn load_artifact(
    artifact: Option<(String, &str)>,
    current: &str,
    probe: &str,
) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "generic.history",
                api_major: 1,
                version: None,
                locked: vec![],
                artifact: None,
                permissions: &[],
                grants: vec![],
            },
            artifacts: artifact
                .into_iter()
                .map(|(digest, source)| RevisionArtifact {
                    package_id: "generic.history".into(),
                    api_major: 1,
                    logical_owner: "Target".into(),
                    revision: 1,
                    artifact: ("opaque".into(), digest, source.into()),
                })
                .collect(),
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn rebuilds_class_method_when_current_closed_counters_are_warm() {
    rebuilds_method("class ", "Target");
}

#[test]
fn rebuilds_instance_method_when_current_closed_counters_are_retained() {
    rebuilds_method("", "Target.new()");
}

fn rebuilds_method(kind: &str, receiver: &str) {
    let given = format!(
        "{DECORATORS} class Target {{ @Count() @Twice() public {kind}fun echo<T>(value: T) -> Integer {{ Effects.bodies = Effects.bodies + 1; 1 }} }}; raise :outside_target"
    );
    let current = format!(
        "{}; open class Target {{ @Count() @Twice() public override {kind}fun echo<T>(value: T) -> Integer {{ Effects.bodies = Effects.bodies + 1; 10 }} }}",
        given.replace("; raise :outside_target", "")
    );
    let probe = format!(
        r#"
        let target = {receiver}; let identity = Target; let old = target.echo
        let integer = old.call(7); let string = old.call("s"); let again = old.call(9)
        let before = Target.active_revision
        let commit = Reflection::Class.revision(Target).fetch(:commit_id)
        let phases = Effects.phases.length
        let result = Target.rollback(1)
        let rollback_phase = Effects.phases[phases] == :rollback
        let fresh = target.echo<Integer>(1); let fresh_string = target.echo<String>("fresh")
        let reused = target.echo<Integer>(2); let retained = old.call(99); let retained_string = old.call("old")
        integer == 22 && string == 22 && again == 23 && fresh == 4 && fresh_string == 4 && reused == 5 && retained == 24 && retained_string == 23 &&
        Effects.bodies == 8 && Effects.phases.length == phases + 3 && rollback_phase &&
        Effects.phases[phases + 1] == :closed_materialization && Effects.phases[phases + 2] == :closed_materialization &&
        Target == identity && Target.active_revision == before + 1 && Reflection::Class.revision(Target).fetch(:commit_id) == commit + 1
    "#
    );

    let probe = if kind.is_empty() {
        probe
    } else {
        probe
            .replace("let old = target.echo", "let old = nil")
            .replace("old.call(", "target.echo(")
            .replace(
                "retained == 24 && retained_string == 23",
                "retained == 6 && retained_string == 5",
            )
    };
    let when = load(&given, &current, &probe);

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_template_when_historical_generic_arity_differs() {
    rejects_signature("public fun echo<T, U>(value: T) -> Integer { 1 }");
}

#[test]
fn preserves_template_when_historical_parameter_narrows() {
    rejects_signature("public fun echo<T>(value: Integer) -> Integer { 1 }");
}

#[test]
fn preserves_template_when_historical_result_widens() {
    rejects_signature("public fun echo<T>(value: T) -> Object { 1 }");
}

fn rejects_signature(method: &str) {
    let given = format!("{DECORATORS} class Target {{ @Count() {method} }}");
    let current = format!(
        "{DECORATORS} class Target {{ @Count() public fun echo<T>(value: T) -> Integer {{ 10 }} }}"
    );

    let when = load(
        &given,
        &current,
        r#"
        let target = Target.new(); let old = target.echo; let warm = old.call(1); let phases = Effects.phases.length
        let before = Target.active_revision; let commit = Reflection::Class.revision(Target).fetch(:commit_id)
        let rejected = try { Target.rollback(1); false } catch error { true }
        let untouched = Effects.phases.length == phases
        rejected && untouched && warm == 12 && target.echo<Integer>(2) == 13 && old.call(3) == 14 && target.echo<String>("s") == 12 &&
        Target.active_revision == before && Reflection::Class.revision(Target).fetch(:commit_id) == commit
    "#,
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
    assert_eq!(
        load(&given, &current, "Target.rollback(1)"),
        Err(EvaluationError::TypeContractError)
    );
}

#[test]
fn uses_historical_helper_when_closed_chain_materializes_after_rollback() {
    let given = format!(
        "{DECORATORS} class Target {{ @Count() public class fun echo<T>(value: T) -> Integer {{ 1 }} }}"
    );
    let current = format!(
        "{given}; open class Count {{ public override fun initialize() {{ @phase = 99 }}; public override fun offset() -> Integer {{ 99 }}; public override fun transform(d, a, c) -> Transformation {{ raise :current_transform }} }}"
    );

    let when = load(
        &given,
        &current,
        "let result = Target.rollback(1); Target.echo<Integer>(7) == 3 && Target.echo<String>(\"s\") == 3",
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn rebuilds_again_when_same_generic_artifact_is_requested_twice() {
    let given = format!(
        "{DECORATORS} class Target {{ @Count() public fun echo<T>(value: T) -> Integer {{ 1 }} }}"
    );

    let when = load(
        &given,
        &given,
        "let target = Target.new(); let first = Target.rollback(1); let old = target.echo; let warm = old.call(1); let second = Target.rollback(1); warm == 3 && target.echo<Integer>(2) == 3 && old.call(3) == 4 && first == second",
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_generic_method_when_artifact_digest_is_invalid() {
    let given = "class Target { public class fun echo<T>(value: T) -> T { value } }";

    let when = load_artifact(Some(("bad".into(), given)), given, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn preserves_generic_method_when_artifact_is_missing() {
    let given = "class Target { public class fun echo<T>(value: T) -> T { value } }";

    let when = load_artifact(None, given, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn retains_pending_generic_bindings_when_rollback_replaces_the_template() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                    await next.call()
                })
            }
        }
        class Target { @Wrap() public async class fun echo<T>(value: T, gate) -> T { await gate; let typed: T = value; typed } }
    "#;
    let current = format!(
        "{given}; open class Target {{ @Wrap() public override async class fun echo<T>(value: T, gate) -> T {{ await gate; let typed: T = value; raise :old_body }} }}"
    );

    let when = load(
        given,
        &current,
        r#"
        let old_gate = Gate.new(); let new_gate = Gate.new(); let string_gate = Gate.new()
        let old = Target.echo<Integer>(1, old_gate)
        let result = Target.rollback(1)
        let fresh = Target.echo<Integer>(7, new_gate); let string = Target.echo<String>("s", string_gate)
        let posted_string = Gate.complete(string_gate, nil); let text = Host.run(string)
        let posted_old = Gate.complete(old_gate, nil); let retained = try { Host.run(old); false } catch error { error == :old_body }
        let posted_new = Gate.complete(new_gate, nil)
        retained && text == "s" && Host.run(fresh) == 7
    "#,
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}
