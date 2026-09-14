use iris_runtime::{ArrayRef, Value};
use iris_vm::{Machine, PackageHistory, RevisionArtifact, compile};

fn artifact(source: &str) -> RevisionArtifact {
    RevisionArtifact {
        package_id: "runtime-local".into(),
        api_major: 1,
        logical_owner: "Target".into(),
        revision: 1,
        artifact: (
            "opaque:old".into(),
            iris_runtime::artifact_digest(source.as_bytes()),
            source.into(),
        ),
    }
}

fn run(history: &str, current: &str) -> Result<Value, iris_vm::MachineError> {
    let mut machine = Machine::new().unwrap();
    machine.enter_package_history(PackageHistory {
        artifacts: vec![artifact(history)],
    });
    machine.execute(&compile(current).unwrap())
}

#[test]
fn historical_body_when_indices_differ_preserves_current_bound_method() {
    let given = "module Unrelated { public fun unused() { 500 } }; class Target { public fun value() -> Integer { 1 } }; raise :unexecuted";
    let when = run(
        given,
        "class Other { public fun collision() { 88 } }; class Target { public fun value() -> Integer { 9 } }; let object = Target.new(); let retained = object.value; let identity = Target; let before = Target.active_revision; let restored = Target.rollback(1); %[object.value(), retained.call(), Target == identity, Target.active_revision == before + 1]",
    );
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Bool(true),
            Value::Bool(true)
        ])))
    );
}

const CACHE: &str = r#"
class Cache {
 public fun initialize() { @cached = nil; @offset = 2 }
 public fun offset() -> Integer { @offset }
}
impl Cache for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   if @cached == nil { @cached = next.call() + offset() }; @cached
  })
 }
}
"#;

#[test]
fn historical_helpers_when_wrappers_escape_keep_fresh_captures() {
    let given = format!(
        "class Target {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; {CACHE}; raise :unexecuted"
    );
    let current = format!(
        "{CACHE}; class Noise {{ public fun collision() {{ 400 }} }}; class Target {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }} }}; open class Cache {{ public override fun initialize() {{ @cached = nil; @offset = 99 }}; public override fun offset() -> Integer {{ 99 }} }}; let object = Target.new(); let retained = object.value; let warm = retained.call(1); let first = Target.rollback(1); let historical = object.value; let old = historical.call(2); let second = Target.rollback(1); %[warm, old, object.value(7), historical.call(99), retained.call(99), Cache.new().offset(), first == second]"
    );
    let when = run(&given, &current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(110_u64.into()),
            Value::Integer(5_u64.into()),
            Value::Integer(10_u64.into()),
            Value::Integer(5_u64.into()),
            Value::Integer(110_u64.into()),
            Value::Integer(99_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn atomic_publication_when_late_transform_fails_preserves_live_wrappers() {
    let given = format!(
        "{CACHE}; class Fail {{}} impl Fail for MethodDecorator {{ public fun plan(d, a) -> Plan {{ Plan.empty }}; public fun transform(d, a, c) -> Transformation {{ raise :late }} }} class Target {{ @Cache() public fun first(n: Integer) -> Integer {{ n + 1 }}; @Fail() public fun last() -> Integer {{ 2 }} }}"
    );
    let current = format!(
        "{CACHE}; class Fail {{}} impl Fail for MethodDecorator {{ public fun plan(d, a) -> Plan {{ Plan.empty }}; public fun transform(d, a, c) -> Transformation {{ Transformation.empty }} }} class Target {{ @Cache() public fun first(n: Integer) -> Integer {{ n + 10 }}; public fun last() -> Integer {{ 9 }} }}; let object = Target.new(); let held = object.first; let warm = held.call(1); let before = Target.active_revision; let rejected = try {{ Target.rollback(1); false }} catch error {{ error == :late }}; %[rejected, object.first(99), held.call(99), object.last(), Target.active_revision == before]"
    );
    let when = run(&given, &current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(13_u64.into()),
            Value::Integer(13_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn rejects_history_when_key_or_digest_is_unavailable() {
    let source = "class Target { public fun value() -> Integer { 1 } }";
    for records in [
        Vec::new(),
        vec![artifact(source), artifact(source)],
        vec![RevisionArtifact {
            package_id: "other".into(),
            ..artifact(source)
        }],
        vec![RevisionArtifact {
            api_major: 2,
            ..artifact(source)
        }],
        vec![RevisionArtifact {
            logical_owner: "Other".into(),
            ..artifact(source)
        }],
        vec![RevisionArtifact {
            artifact: ("opaque".into(), "bad".into(), source.into()),
            ..artifact(source)
        }],
    ] {
        let mut given = Machine::new().unwrap();
        given.enter_package_history(PackageHistory { artifacts: records });
        let when = given.execute(&compile("class Target { public fun value() -> Integer { 9 } }; let before = Target.active_revision; let error = try { Target.rollback(1) } catch error { error }; %[error, Target.new().value(), Target.active_revision == before]").unwrap());
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Symbol("RevisionArtifactUnavailableError".into()),
                Value::Integer(9_u64.into()),
                Value::Bool(true)
            ])))
        );
    }
}

#[test]
fn rejects_symbol_and_missing_revision_when_no_alias_is_supplied() {
    let given = "class Target { public fun value() -> Integer { 1 } }";
    for argument in ["2", ":old", "", "-1"] {
        let when = run(given, &format!("{given}; Target.rollback({argument})"));
        assert_eq!(
            when,
            Err(iris_vm::MachineError::RevisionArtifactUnavailable)
        );
    }
}

#[test]
fn rejects_incompatible_shape_when_history_would_change_promises() {
    for given in [
        "class Target { public fun other() -> Integer { 1 } }",
        "class Target { private fun value() -> Integer { 1 } }",
        "class Target { public fun value() -> String { \"bad\" } }",
        "class Target { public fun value() -> Integer { 1 }; public fun extra() { nil } }",
    ] {
        let when = run(
            given,
            "class Target { public fun value() -> Integer { 9 } }; let before = Target.active_revision; let error = try { Target.rollback(1) } catch error { error }; %[error, Target.new().value(), Target.active_revision == before]",
        );
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Symbol("TypeContractError".into()),
                Value::Integer(9_u64.into()),
                Value::Bool(true)
            ])))
        );
    }
}

#[test]
fn preserves_policy_when_historical_definition_allows_mutation() {
    let given = "class Target { public fun value() -> Integer { 1 } }";
    let when = run(
        given,
        "class Target meta deny method_body { public fun value() -> Integer { 9 } }; let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { true }; %[rejected, Target.new().value(), Target.active_revision == before]",
    );
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(9_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn links_contract_identity_when_history_reorders_contracts() {
    let given = "contract Other { fun other() }; contract Promise { fun value() -> Integer }; class Target {} impl Target for Promise { public fun value() -> Integer { 1 } }";
    let when = run(
        given,
        "contract Promise { fun value() -> Integer }; class Target {} impl Target for Promise { public fun value() -> Integer { 9 } }; let restored = Target.rollback(1); (Target.new() as Promise)..value()",
    );
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn reports_original_offsets_when_historical_body_raises() {
    let given = "module Unrelated { public fun unused() { 88 } }\n\nclass Target {\n public fun value() {\n  raise :historical\n }\n}";
    let when = run(
        given,
        "class Target { public fun value() { 9 } }; let restored = Target.rollback(1); try { Target.new().value() } catch error, context { context.raise_location }",
    );
    assert_eq!(when, Ok(Value::SourceLocation("<source>".into(), 5, 3)));
}

#[test]
fn rollback_reason_when_rebuilt_is_not_origin_or_open() {
    let given = "class Reason {} impl Reason for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { let reason = c.reason; Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; reason }) } } class Target { @Reason() public fun value() { nil } }";
    let when = run(
        given,
        &format!("{given}; let restored = Target.rollback(1); Target.new().value()"),
    );
    assert_eq!(when, Ok(Value::Symbol("rollback".into())));
}

#[test]
fn rejects_unsupported_history_before_publishing() {
    for given in [
        "class Target<T> { public fun value() { 1 } }",
        "class Target { public property value: Integer = 1 }",
        "class Target { public fun value<T>(n: T) { n } }",
        "class Target { public fun value() { NativeFixture.compact_gc() } }",
    ] {
        let when = run(
            given,
            "class Target { public fun value() { 9 } }; Target.rollback(1)",
        );
        assert_eq!(when, Err(iris_vm::MachineError::UnsupportedConstruct));
    }
}

#[test]
fn wrappers_survive_when_collection_runs_after_history_rebuild() {
    let given = format!(
        "{CACHE}; class Target {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 1 }} }}"
    );
    let when = run(
        &given,
        &format!(
            "{given}; let object = Target.new(); let current = object.value; let warm = current.call(1); let restored = Target.rollback(1); let collected = NativeFixture.compact_gc(); %[object.value(7), current.call(99)]"
        ),
    );
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(10_u64.into()),
            Value::Integer(4_u64.into())
        ])))
    );
}

#[test]
fn rejects_historical_additions_when_class_transform_changes_member_set() {
    let given = "class Add {} impl Add for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.add_method(:extra, { || -> Integer; 4 }) } } @Add() class Target { public fun value() { 1 } }";
    let current = "class Add {} impl Add for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.empty } } class Target { public fun value() { 9 } }; Target.rollback(1)";
    let when = run(given, current);
    assert_eq!(when, Err(iris_vm::MachineError::UnsupportedConstruct));
}

#[test]
fn escaped_history_closure_when_transform_raises_survives_failed_publication() {
    let given = "class Escape { public fun offset() -> Integer { 42 } } impl Escape for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise { || -> Integer; offset() } } } class Target { public fun first() { 1 }; @Escape() public fun value() { 2 } }";
    let current = "class Escape { public fun offset() -> Integer { 99 } } impl Escape for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.empty } } class Target { public fun first() { 8 }; public fun value() { 9 } }; let held = Target.new().value; let before = Target.active_revision; let escaped = try { Target.rollback(1) } catch error { error }; let collected = NativeFixture.compact_gc(); %[escaped.call(), held.call(), Target.new().first(), Target.active_revision == before, Escape.new().offset()]";
    let when = run(given, current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(42_u64.into()),
            Value::Integer(9_u64.into()),
            Value::Integer(8_u64.into()),
            Value::Bool(true),
            Value::Integer(99_u64.into())
        ])))
    );
}

#[test]
fn historical_source_when_loaded_by_host_keeps_package_major_and_one_commit() {
    let history = "export class Target { public fun value() -> Integer { 1 } }";
    let sources = [iris_native_host::PackageSource {
        package_id: "host.example".into(), api_major: 7, version: "7.1.0".into(), path: "current.iris".into(),
        allowed_imports: Default::default(),
        source: "class Target { public fun value() -> Integer { 9 } }; let before = Reflection::Class.revision(Target); let restored = Target.rollback(1); let after = Reflection::Class.revision(Target); %[Target.new().value(), Reflection::Class.method(Target, :value).source[0], after[:commit_id] == before[:commit_id] + 1]".into(),
    }];
    let mut given = Machine::new().unwrap();
    let mut record = artifact(history);
    record.package_id = "host.example".into();
    record.api_major = 7;
    given.enter_package_history(PackageHistory {
        artifacts: vec![record],
    });
    let program = iris_vm::compile_package_tree_with_natives(
        &sources,
        &iris_native_host::NativeRegistry::new(),
    )
    .unwrap();
    let when = given.execute(&program);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Symbol("host.example".into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn absent_historical_constructor_when_current_added_one_does_not_run_live_code() {
    let given = "class Empty {} impl Empty for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.empty } } class Target { @Empty() public fun value() { 1 } }";
    let when = run(
        given,
        &format!(
            "{given}; open class Empty {{ public fun initialize() {{ raise :current_constructor }} }}; let restored = Target.rollback(1); Target.new().value()"
        ),
    );
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}
