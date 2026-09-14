use iris_eval::{
    EvaluationError, PackageHistory, PackageResolution, RevisionArtifact,
    load_resolved_package_with_history,
};
use iris_runtime::{ArrayRef, Value};

const DECORATORS: &str = r#"
class Cache {}
impl Cache for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        @cached = nil
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if @cached == nil { @cached = next.call() }; @cached
        })
    }
}
class Twice {}
impl Twice for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        if c.reason == :rollback { @factor = 2 } else { @factor = 3 }
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            next.call() * @factor
        })
    }
}
"#;

#[test]
fn historical_and_retained_chains_survive_when_collection_runs_after_rollback() {
    let history = format!(
        "{DECORATORS} class Target {{ @Cache() @Twice() public fun value(n: Integer) -> Integer {{ n + 1 }} }}"
    );
    let current = format!(
        "{DECORATORS} class Target {{ @Cache() @Twice() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; open class Target {{ @Cache() @Twice() public override fun value(n: Integer) -> Integer {{ n + 10 }} }}; module Collector {{ public fun run() {{ NativeFixture.compact_gc(); nil }} }}"
    );

    let when = load(
        &history,
        &current,
        "let target = Target.new(); let retained = target.value; let warm = retained.call(1); let rollback = Target.rollback(1); let collected = Collector.run(); %[warm, target.value(2), retained.call(99)]",
    );

    assert_eq!(
        when,
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(33u64.into()),
            Value::Integer(6u64.into()),
            Value::Integer(33u64.into())
        ]))))
    );
}

fn load(history: &str, current: &str, probe: &str) -> Result<Option<Value>, EvaluationError> {
    load_digest(
        history,
        current,
        probe,
        iris_runtime::artifact_digest(history.as_bytes()),
    )
}

fn load_digest(
    history: &str,
    current: &str,
    probe: &str,
    digest: String,
) -> Result<Option<Value>, EvaluationError> {
    load_resolved_package_with_history(
        PackageHistory {
            resolution: PackageResolution {
                package_id: "rollback.test",
                api_major: 1,
                version: Some("1.1.0".into()),
                locked: Vec::new(),
                artifact: Some((
                    "rollback.test/old.iris".into(),
                    digest.clone(),
                    history.into(),
                )),
                permissions: &[],
                grants: Vec::new(),
            },
            artifacts: vec![RevisionArtifact {
                package_id: "rollback.test".into(),
                api_major: 1,
                logical_owner: "Target".into(),
                revision: 1,
                artifact: ("rollback.test/old.iris".into(), digest, history.into()),
            }],
        },
        &[("current.iris".into(), current.into())],
        Some(probe),
    )
    .map(|(_, value)| value)
}

#[test]
fn fresh_historical_chain_when_current_cache_and_bound_method_are_retained() {
    let history = format!(
        "{DECORATORS} class Target {{ @Cache() @Twice() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; raise :outside_target"
    );
    let current = format!(
        "{DECORATORS} class Target {{ @Cache() @Twice() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; open class Target {{ @Cache() @Twice() public override fun value(n: Integer) -> Integer {{ n + 10 }} }}; open class Twice {{ public override fun transform(d, a, c) -> Transformation {{ raise :current_decorator }} }}"
    );
    let probe = "let target = Target.new(); let identity = Target; let old = target.value; let warm = old.call(1); let before = Target.active_revision; let digest = Target.rollback(1); %[warm, target.value(2), target.value(99), old.call(99), Target == identity, Target.active_revision == before + 1]";
    assert_eq!(
        load(&history, &current, probe),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(33u64.into()),
            Value::Integer(6u64.into()),
            Value::Integer(6u64.into()),
            Value::Integer(33u64.into()),
            Value::Bool(true),
            Value::Bool(true),
        ]))))
    );
}

#[test]
fn preserves_target_when_digest_mismatches() {
    let current = "class Target { public fun value() -> Integer { 9 } }";
    let probe = "let before = Target.active_revision; let ignored = try { Target.rollback(1) } catch error { nil }; %[Target.new().value(), Target.active_revision == before]";
    assert_eq!(
        load_digest(
            "class Target { public fun value() -> Integer { 1 } }",
            current,
            probe,
            "bad".into()
        ),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(9u64.into()),
            Value::Bool(true)
        ]))))
    );
}

#[test]
fn rejects_missing_member_even_when_an_unrelated_class_declares_it() {
    let history = "class Target { public fun value() -> Integer { 1 } }; class Other { public fun later() -> Integer { 2 } }";
    let current =
        "class Target { public fun value() -> Integer { 9 }; public fun later() -> Integer { 8 } }";
    let probe = "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; %[rejected, Target.new().value(), Target.active_revision == before]";
    assert_eq!(
        load(history, current, probe),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(9u64.into()),
            Value::Bool(true)
        ]))))
    );
}

#[test]
fn rejects_unavailable_requested_revision() {
    let source = "class Target { public fun value() -> Integer { 1 } }";
    assert_eq!(
        load(source, source, "Target.rollback(2)"),
        Err(EvaluationError::RevisionArtifactUnavailable)
    );
}

#[test]
fn preserves_current_policy_when_history_allows_replacement() {
    let history = "class Target { public fun value() -> Integer { 1 } }";
    let current = "class Target meta deny method_body { public fun value() -> Integer { 9 } }";
    let probe = "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; %[rejected, Target.new().value(), Target.active_revision == before]";
    assert_eq!(
        load(history, current, probe),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(9u64.into()),
            Value::Bool(true)
        ]))))
    );
}

#[test]
fn rebuilds_again_when_same_artifact_is_requested_twice() {
    let history = format!(
        "{DECORATORS} class Target {{ @Cache() public fun value(n: Integer) -> Integer {{ n }} }}"
    );
    let probe = "let target = Target.new(); let first = Target.rollback(1); let old = target.value; let warm = old.call(4); let second = Target.rollback(1); %[warm, target.value(7), old.call(9), first == second]";
    assert_eq!(
        load(&history, &history, probe),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Integer(4u64.into()),
            Value::Integer(7u64.into()),
            Value::Integer(4u64.into()),
            Value::Bool(true)
        ]))))
    );
}

#[test]
fn publishes_nothing_when_historical_transform_fails_after_staging_bodies() {
    let history = "class Fail {} impl Fail for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :failed } }; class Target { public fun first() -> Integer { 1 }; @Fail() public fun value() -> Integer { 2 } }";
    let current = "class Fail {} impl Fail for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.empty } }; class Target { public fun first() -> Integer { 8 }; public fun value() -> Integer { 9 } }";
    let probe = "let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; %[rejected, Target.new().first(), Target.new().value(), Target.active_revision == before]";
    assert_eq!(
        load(history, current, probe),
        Ok(Some(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(8u64.into()),
            Value::Integer(9u64.into()),
            Value::Bool(true)
        ]))))
    );
}

#[test]
fn preserves_digest_return_when_single_artifact_is_selected_without_argument() {
    let history = "class Target { public fun value() -> Integer { 1 } }";
    assert_eq!(
        load(history, history, "Target.rollback()"),
        Ok(Some(Value::Symbol(iris_runtime::artifact_digest(
            history.as_bytes()
        ))))
    );
}

#[test]
fn uses_historical_constructor_and_helper_after_wrapper_escapes() {
    let history = "class Add { public fun initialize() { @offset = 2 }; public fun offset() -> Integer { @offset } } impl Add for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + offset() }) } }; class Target { @Add() public fun value() -> Integer { 1 } }";
    let current = format!(
        "{history}; open class Add {{ public override fun initialize() {{ @offset = 99 }}; public override fun offset() -> Integer {{ 99 }} }}"
    );
    assert_eq!(
        load(
            history,
            &current,
            "let ignored = Target.rollback(1); Target.new().value()"
        ),
        Ok(Some(Value::Integer(3u64.into())))
    );
}
