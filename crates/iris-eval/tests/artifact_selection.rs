use iris_eval::{EvaluationError, PackageResolution, load_resolved_package_with_artifact};
use iris_eval::{PackageHistory, RevisionArtifact, load_resolved_package_with_history};
use iris_runtime::Value;

const HISTORY: &str = "class Target { public fun value() -> Integer { 1 } }";
const CURRENT: &str = "class Target { public fun value() -> Integer { 9 } }";

fn resolution() -> PackageResolution<'static> {
    PackageResolution {
        package_id: "rollback.test",
        api_major: 1,
        version: None,
        locked: Vec::new(),
        artifact: Some((
            "opaque@1".into(),
            iris_runtime::artifact_digest(HISTORY.as_bytes()),
            HISTORY.into(),
        )),
        permissions: &[],
        grants: Vec::new(),
    }
}

fn record(locator: &str) -> RevisionArtifact {
    RevisionArtifact {
        package_id: "rollback.test".into(),
        api_major: 1,
        logical_owner: "Target".into(),
        revision: 1,
        artifact: (
            locator.into(),
            iris_runtime::artifact_digest(HISTORY.as_bytes()),
            HISTORY.into(),
        ),
    }
}

fn load(records: Vec<RevisionArtifact>, probe: &str) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: resolution(),
            artifacts: records,
        },
        &[("current.iris".into(), CURRENT.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn rejects_explicit_revision_when_only_unkeyed_artifact_is_available() {
    let given = resolution();

    let actual = load_resolved_package_with_artifact(
        given,
        &[("current.iris".into(), CURRENT.into())],
        Some("Target.rollback(1)"),
    );

    assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn reconstructs_revision_when_locator_has_no_revision_syntax() {
    let given = vec![record("opaque/store/blob")];

    let actual = load(
        given,
        "let digest = Target.rollback(1); Target.new().value()",
    );

    assert_eq!(actual, Ok(Some(Value::Integer(1u64.into()))));
}

#[test]
fn selects_metadata_revision_when_locator_suffix_disagrees() {
    let given = vec![record("opaque@99")];

    let actual = load(given, "Target.rollback(1)");

    assert_eq!(
        actual,
        Ok(Some(Value::Symbol(iris_runtime::artifact_digest(
            HISTORY.as_bytes()
        ))))
    );
}

#[test]
fn rejects_locator_suffix_when_it_is_not_a_metadata_revision() {
    let given = vec![record("opaque@99")];

    let actual = load(given, "Target.rollback(99)");

    assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn rejects_history_when_package_owner_or_major_mismatches() {
    let mut package = record("opaque");
    package.package_id = "other.test".into();
    let mut owner = record("opaque");
    owner.logical_owner = "Other".into();
    let mut major = record("opaque");
    major.api_major = 2;
    for given in [package, owner, major] {
        let actual = load(vec![given], "Target.rollback(1)");

        assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
    }
}

#[test]
fn rejects_missing_or_unmapped_targets_when_history_is_available() {
    for target in ["2", "-1", "18446744073709551616", ":old", ":\"1\""] {
        let given = vec![record("opaque@old")];

        let actual = load(given, &format!("Target.rollback({target})"));

        assert_eq!(
            actual,
            Err(EvaluationError::RevisionArtifactUnavailable),
            "{target}"
        );
    }
}

#[test]
fn selects_exact_record_when_multiple_revisions_are_available() {
    let mut second = record("same-opaque-locator");
    second.revision = 2;
    second.artifact.2 = "class Target { public fun value() -> Integer { 2 } }".into();
    second.artifact.1 = iris_runtime::artifact_digest(second.artifact.2.as_bytes());
    let given = vec![record("same-opaque-locator"), second];

    let actual = load(
        given,
        "let digest = Target.rollback(2); Target.new().value()",
    );

    assert_eq!(actual, Ok(Some(Value::Integer(2u64.into()))));
}

#[test]
fn publishes_nothing_when_keyed_digest_is_malformed() {
    let mut given = record("opaque");
    given.artifact.1 = "b3:malformed".into();

    let actual = load(
        vec![given],
        "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; rejected && Target.new().value() == 9 && Target.active_revision == before",
    );

    assert_eq!(actual, Ok(Some(Value::Bool(true))));
}

#[test]
fn preserves_legacy_digest_when_rollback_has_no_argument() {
    let given = resolution();

    let actual = load_resolved_package_with_artifact(
        given,
        &[("current.iris".into(), CURRENT.into())],
        Some("Target.rollback()"),
    );

    assert_eq!(
        actual,
        Ok((
            vec![],
            Some(Value::Symbol(iris_runtime::artifact_digest(
                HISTORY.as_bytes()
            )))
        ))
    );
}

#[test]
fn preserves_v357_when_loading_the_frozen_unkeyed_fixture() {
    let source = include_str!("../../../conformance/iris-v1/fixtures/meta/v357/src/owner.ir");
    let mut given = resolution();
    given.package_id = "org.iris.v357";
    given.artifact = Some((
        "org.iris.v357/src/owner.ir@1".into(),
        "b3:7513f737323f64751a41ef3215b226725c0eef9c54d89918085ad5b0ee9a9360".into(),
        "class Owner {\n  public fun status() -> Symbol { :ok }\n}\n".into(),
    ));

    let actual = load_resolved_package_with_artifact(
        given,
        &[("src/owner.ir".into(), source.into())],
        Some("Owner.rollback()"),
    );

    assert_eq!(
        actual,
        Ok((
            vec![],
            Some(Value::Symbol(
                "7513f737323f64751a41ef3215b226725c0eef9c54d89918085ad5b0ee9a9360".into()
            ))
        ))
    );
}

#[test]
fn rejects_ambiguous_keys_when_two_records_name_the_same_revision() {
    let given = vec![record("first"), record("second")];

    let actual = load(given, "Target.rollback(1)");

    assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn preserves_digest_error_when_keyed_artifact_is_malformed() {
    let mut given = record("opaque");
    given.artifact.1 = "b3:malformed".into();

    let actual = load(vec![given], "Target.rollback(1)");

    assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
}

#[test]
fn rejects_no_argument_when_only_keyed_history_exists() {
    let mut given = resolution();
    given.artifact = None;

    let actual = load_resolved_package_with_history(
        PackageHistory {
            resolution: given,
            artifacts: vec![record("opaque")],
        },
        &[("current.iris".into(), CURRENT.into())],
        Some("Target.rollback()"),
    );

    assert_eq!(actual, Err(EvaluationError::RevisionArtifactUnavailable));
}
