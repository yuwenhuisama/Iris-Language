use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact, Session,
    load_resolved_package_with_history,
};
use iris_runtime::{ArrayRef, Value};

const WRAP: &str = "class Wrap {} impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + 1 })
    }
}";

fn replay(history: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    let artifact = (
        "history.iris".into(),
        iris_runtime::artifact_digest(history.as_bytes()),
        history.into(),
    );
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "impl.history",
                api_major: 1,
                version: Some("1.0.0".into()),
                locked: Vec::new(),
                artifact: Some(artifact.clone()),
                permissions: &[],
                grants: Vec::new(),
            },
            artifacts: vec![RevisionArtifact {
                package_id: "impl.history".into(),
                api_major: 1,
                logical_owner: "Target".into(),
                revision: 1,
                artifact,
            }],
        },
        &[("current.iris".into(), history.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn impl_methods_and_conformance_survive_when_package_history_is_replayed() {
    let given = format!(
        "{WRAP} contract Read {{ fun value() -> Integer }} export class Target {{}} impl Target for Read {{ @Wrap() public fun value() -> Integer {{ 7 }} }}"
    );

    let when = replay(
        &given,
        "let retained = Target.new().value; let before = Target.active_revision; let restored = Target.rollback(1); %[(Target.new() as Read)..value(), retained.call(), Target.active_revision == before + 1]",
    );

    assert_eq!(
        when,
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(8u64.into()),
            Value::Integer(8u64.into()),
            Value::Bool(true),
        ]))))
    );
}

#[test]
fn decorator_kind_survives_when_session_reopens_after_an_unrelated_chunk() {
    let mut given = Session::new().unwrap();
    given.evaluate(&format!("{WRAP}; 0")).unwrap();
    given
        .evaluate("class Target { @Wrap() public fun value() -> Integer { 7 } }; 0")
        .unwrap();
    given.evaluate("class Unrelated {}; 0").unwrap();

    let when = given.evaluate(
        "open class Target { @Wrap() public override fun value() -> Integer { 9 } }; Target.new().value()",
    );

    assert_eq!(when, Ok(Value::Integer(10u64.into())));
}

#[test]
fn repeated_replay_preserves_impl_members_when_decorator_has_current_replacement() {
    let given = format!(
        "{WRAP} contract Read {{ fun value() -> Integer }} class Target {{}} impl Target for Read {{ @Wrap() public fun value() -> Integer {{ 7 }} }}"
    );

    let when = replay(
        &given,
        "open class Wrap { public override fun transform(d, a, c) -> Transformation { raise :current } }; let first = Target.rollback(1); let second = Target.rollback(1); (Target.new() as Read)..value()",
    );

    assert_eq!(when, Ok(Some(Value::Integer(8u64.into()))));
}

#[test]
fn static_impl_origin_keeps_lazy_initialization_when_no_decorator_is_applied() {
    let mut given = Session::new().unwrap();

    let when = given.evaluate(
        "let seed = 7; contract Read { fun value() -> Integer } class Target { public class property seed: Integer = seed } impl Target for Read { public fun value() -> Integer { 1 } } Target.seed",
    );

    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn decorated_origin_rejects_candidate_when_class_initializer_raises() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stamp {} impl Stamp for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } }; 0").unwrap();

    let when = given.evaluate(
        "@Stamp() class Target { public class property broken: Object = { raise :initializer }.call() }; 0",
    );

    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("initializer".into())))
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}
