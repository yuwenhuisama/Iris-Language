use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::Value;

const SOURCE: &str = r#"
class Cache {
    public fun offset() -> Integer { 1 }
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        @cached = nil
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if invocation.slot[2] != nil { raise :qualifier }
            if @cached == nil { @cached = next.call() + offset() }; @cached
        })
    }
}
impl Cache for MethodDecorator {}
contract Named { fun value(n: Integer) -> Integer }
class Target { @Cache() public fun value(n: Integer) -> Integer { n + 10 } }
impl Target for Named {}
"#;

fn load(history: &str, current: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "qualified.history",
                api_major: 1,
                version: None,
                locked: vec![],
                artifact: None,
                permissions: &[],
                grants: vec![],
            },
            artifacts: vec![RevisionArtifact {
                package_id: "qualified.history".into(),
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
fn rebuilds_shared_cache_when_qualified_history_is_selected() {
    let current = format!(
        "{SOURCE}; open class Target {{ @Cache() public override fun value(n: Integer) -> Integer {{ n + 200 }} }}; open class Cache {{ public override fun offset() -> Integer {{ 99 }} }}"
    );
    let history = format!("{SOURCE}; contract Unrelated {{ fun absent() }}; raise :outside_target");

    let when = load(
        &history,
        &current,
        r#"
        let target = Target.new(); let view = target as Named; let identity = Named.type
        let old = target.value; let warm = old.call(1); let qualified = view..value(99)
        let before = Target.active_revision; let digest = Target.rollback(1)
        %[warm, qualified, target.value(2), view..value(99), target.value(99), old.call(99),
          Named.type == identity, Target.active_revision == before + 1, Cache.new().offset()]
    "#,
    );

    assert_eq!(
        when,
        iris_eval::evaluate("%[300, 300, 13, 13, 13, 300, true, true, 99]").map(Some)
    );
}

#[test]
fn rejects_missing_qualified_member_when_ordinary_selector_still_exists() {
    let history = "class Target { public fun value(n: Integer) -> Integer { n } }";

    let when = load(history, SOURCE, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn rejects_wrong_qualified_signature_when_artifact_is_parseable() {
    let history =
        "class Target { public fun value(n: String) -> Integer { 1 } } impl Target for Named {}";

    let when = load(history, SOURCE, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn rejects_wrong_contract_obligation_when_historical_contract_differs() {
    let history = "contract Other { fun value(n: Integer) -> Integer }; class Target { public fun value(n: Integer) -> Integer { n } }; impl Target for Other {}";

    let when = load(history, SOURCE, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn unified_surface_uses_the_contract_signature() {
    let source = "contract Named { fun value(n: Integer) -> Integer }; class Target { public fun value(n: Integer) -> Integer { n } }; impl Target for Named {}";

    let when = load(
        source,
        source,
        "let target = Target.new(); let digest = Target.rollback(1); target.value(7) == 7 && (target as Named)..value(7) == 7",
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}

#[test]
fn keeps_current_contract_requirements_when_historical_declaration_is_weaker() {
    let history = "contract Named { fun value(n: String) -> Integer }; class Target { public fun value(n: String) -> Integer { 1 } }; impl Target for Named {}";

    let when = load(history, SOURCE, "Target.rollback(1)");

    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn publishes_nothing_when_qualified_requirement_is_missing() {
    let history = "class Target { public fun value(n: Integer) -> Integer { n } }";

    let when = load(
        history,
        SOURCE,
        r#"
        let target = Target.new(); let view = target as Named
        let warm = target.value(1); let qualified = view..value(1); let before = Target.active_revision
        let rejected = try { Target.rollback(1); false } catch error { true }
        rejected && Target.active_revision == before && target.value(99) == warm && view..value(99) == qualified
    "#,
    );

    assert_eq!(when, Ok(Some(Value::Bool(true))));
}
