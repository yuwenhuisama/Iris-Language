use iris_runtime::{ArrayRef, Value};
use iris_vm::{Machine, MachineError, PackageHistory, RevisionArtifact, compile};

fn artifact(source: &str) -> RevisionArtifact {
    RevisionArtifact {
        package_id: "runtime-local".into(),
        api_major: 1,
        logical_owner: "Provider".into(),
        revision: 71,
        artifact: (
            "opaque:not-a-path".into(),
            format!("b3:{}", iris_runtime::artifact_digest(source.as_bytes())),
            source.into(),
        ),
    }
}

fn run(history: &str, current: &str) -> Result<Value, MachineError> {
    let mut machine = Machine::new().unwrap();
    machine.enter_package_history(PackageHistory {
        artifacts: vec![artifact(history)],
    });
    machine.execute(&compile(current).unwrap())
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
   if invocation.slot[0] != Provider { raise :owner }
   if @cached == nil { @cached = next.call() + offset() }; @cached
  })
 }
}
"#;

#[test]
fn fresh_history_when_repeated_preserves_main_and_mixed_in_caches() {
    let given = format!(
        "module Noise {{ public fun collision() {{ 400 }} }}; module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 1 }} }}; {CACHE}; raise :unexecuted"
    );
    let current = format!(
        "{CACHE}; module Other {{ public fun unrelated() {{ 88 }} }}; module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }} }}; class Client mixin Provider {{}}; open class Cache {{ public override fun initialize() {{ @cached = nil; @offset = 99 }}; public override fun offset() -> Integer {{ 99 }} }}; let client = Client.new(); let retained = client.value; let main_old = Reflection::Module.method(Provider, :value); let warm = Provider.value(1); let first = Provider.rollback(71); let historical = client.value; let fresh = client.value(2); let shared_value = Provider.value(99); let second = Provider.rollback(71); let collected = NativeFixture.compact_gc(); %[warm, fresh, shared_value, Provider.value(7), historical.call(99), retained.call(99), Reflection::Module.invoke(main_old, Provider, %[99]), Cache.new().offset(), first == second]"
    );
    let when = run(&given, &current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            vec![110_u64, 5, 5, 10, 5, 110, 110, 99]
                .into_iter()
                .map(|number| Value::Integer(number.into()))
                .chain([Value::Bool(true)])
                .collect()
        )))
    );
}

#[test]
fn plain_history_when_function_order_differs_restores_only_selected_module() {
    let given = "class Noise { public fun collision() { 400 } }; export module Provider { public fun value() -> Integer { 1 } }; raise :unexecuted";
    let when = run(
        given,
        "module Other { public fun collision() { 88 } }; module Provider { public fun value() -> Integer { 9 } }; let identity = Provider; let restored = Provider.rollback(71); %[Provider.value(), Other.collision(), Provider == identity]",
    );
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(88_u64.into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn unavailable_when_record_or_owner_kind_is_wrong() {
    let source = "module Provider { public fun value() -> Integer { 1 } }";
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
        vec![artifact(
            "class Provider { public fun value() -> Integer { 1 } }",
        )],
        vec![artifact(&format!("{source}; class Provider {{}}"))],
        vec![artifact(&format!("{source}; {source}"))],
    ] {
        let mut given = Machine::new().unwrap();
        given.enter_package_history(PackageHistory { artifacts: records });
        let when = given.execute(&compile("module Provider { public fun value() -> Integer { 9 } }; let error = try { Provider.rollback(71) } catch error { error }; %[error, Provider.value()]").unwrap());
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Symbol("RevisionArtifactUnavailableError".into()),
                Value::Integer(9_u64.into())
            ])))
        );
    }
}

#[test]
fn revision_key_when_not_an_explicit_integer_has_no_inferred_alias() {
    let given = "module Provider { public fun value() { 1 } }";
    for argument in ["1", ":old", "", "-1"] {
        let when = run(given, &format!("{given}; Provider.rollback({argument})"));
        assert_eq!(when, Err(MachineError::RevisionArtifactUnavailable));
    }
    assert_eq!(
        run(given, &format!("{given}; Provider.rollback(true)")),
        Err(MachineError::ArgumentError)
    );
}

#[test]
fn incompatible_history_when_promises_change_leaves_current_body() {
    for given in [
        "module Provider { public fun other() -> Integer { 1 } }",
        "module Provider { private fun value() -> Integer { 1 } }",
        "module Provider { public fun value() -> String { \"bad\" } }",
        "module Provider { public fun value(n: Integer) -> Integer { n } }",
        "module Provider { public module fun value() -> Integer { 1 } }",
        "module Provider { public fun value() -> Integer { 1 }; public fun extra() { nil } }",
    ] {
        let when = run(
            given,
            "module Provider { public fun value() -> Integer { 9 } }; let error = try { Provider.rollback(71) } catch error { error }; %[error, Provider.value()]",
        );
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Symbol("TypeContractError".into()),
                Value::Integer(9_u64.into())
            ])))
        );
    }
}

#[test]
fn failed_transform_when_closure_escapes_keeps_historical_helpers_and_live_methods() {
    let given = "class Escape { public fun offset() -> Integer { 42 } } impl Escape for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise { || -> Integer; offset() } } } module Provider { public fun first() { 1 }; @Escape() public fun value() { 2 } }";
    let current = "class Escape { public fun offset() -> Integer { 99 } } impl Escape for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.empty } } module Provider { public fun first() { 8 }; public fun value() { 9 } }; class Client mixin Provider {}; let held = Client.new().value; let escaped = try { Provider.rollback(71) } catch error { error }; let collected = NativeFixture.compact_gc(); %[escaped.call(), held.call(), Provider.first(), Escape.new().offset()]";
    let when = run(given, current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            vec![42_u64, 9, 8, 99]
                .into_iter()
                .map(|number| Value::Integer(number.into()))
                .collect()
        )))
    );
}

#[test]
fn module_method_when_rebuilt_receives_rollback_reason() {
    let given = "class Reason {} impl Reason for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { let reason = c.reason; Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; if reason == :rollback { next.call() + 10 } else { next.call() } }) } } module Provider { @Reason() public fun value() -> Integer { 1 } }";
    let when = run(
        given,
        &format!(
            "{given}; let before = Provider.value(); let restored = Provider.rollback(71); %[before, Provider.value()]"
        ),
    );
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(11_u64.into())
        ])))
    );
}

#[test]
fn unsupported_history_when_state_composition_or_generics_change_is_rejected() {
    for given in [
        "module Provider<T> { public fun value() { 1 } }",
        "module Provider { public fun value<T>(item: T) { item } }",
        "module Provider { const state = 1; public fun value() { state } }",
        "module Other {}; module Provider mixin Other { public fun value() { 1 } }",
        "module Provider { public fun value() { NativeFixture.compact_gc() } }",
    ] {
        let when = run(
            given,
            "module Provider { public fun value() { 9 } }; Provider.rollback(71)",
        );
        assert_eq!(when, Err(MachineError::UnsupportedConstruct));
    }
}

#[test]
fn unsupported_current_module_when_state_or_composition_exists_is_rejected() {
    let given = "module Provider { public fun value() { 1 } }";
    for current in [
        "module Provider<T> { public fun value() { 9 } }",
        "module Provider { const state = 9; public fun value() { state } }",
        "module Other {}; module Provider mixin Other { public fun value() { 9 } }",
    ] {
        let when = run(given, &format!("{current}; Provider.rollback(71)"));
        assert_eq!(when, Err(MachineError::UnsupportedConstruct));
    }
}

#[test]
fn policy_when_history_changes_or_current_denies_bodies_stays_frozen() {
    for (historical, current) in [
        ("", "meta deny method_body"),
        ("meta deny method_body", ""),
        ("meta deny method_body", "meta deny method_body"),
    ] {
        let given = format!("module Provider {historical} {{ public fun value() {{ 1 }} }}");
        let when = run(
            &given,
            &format!(
                "module Provider {current} {{ public fun value() {{ 9 }} }}; let rejected = try {{ Provider.rollback(71); false }} catch error {{ true }}; %[rejected, Provider.value()]"
            ),
        );
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Integer(9_u64.into())
            ])))
        );
    }
}

#[test]
fn plain_module_fun_when_history_is_compatible_rebuilds_body() {
    let given = "module Provider { public module fun value() -> Integer { 1 } }";
    let when = run(
        given,
        "module Provider { public module fun value() -> Integer { 9 } }; let restored = Provider.rollback(71); Provider.value()",
    );
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn late_transform_when_current_cache_is_warm_does_not_replace_it() {
    let given = format!(
        "{CACHE}; class Fail {{}} impl Fail for MethodDecorator {{ public fun plan(d,a) -> Plan {{ Plan.empty }}; public fun transform(d,a,c) -> Transformation {{ raise :late }} }} module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 1 }}; @Fail() public fun last() {{ 2 }} }}"
    );
    let current = format!(
        "{CACHE}; class Fail {{}} impl Fail for MethodDecorator {{ public fun plan(d,a) -> Plan {{ Plan.empty }}; public fun transform(d,a,c) -> Transformation {{ Transformation.empty }} }} module Provider {{ @Cache() public fun value(n: Integer) -> Integer {{ n + 10 }}; public fun last() {{ 9 }} }}; class Client mixin Provider {{}}; let held = Client.new().value; let warm = held.call(1); let rejected = try {{ Provider.rollback(71); false }} catch error {{ error == :late }}; %[rejected, Provider.value(99), held.call(99), Provider.last()]"
    );
    let when = run(&given, &current);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(13_u64.into()),
            Value::Integer(13_u64.into()),
            Value::Integer(9_u64.into())
        ])))
    );
}

#[test]
fn host_package_when_history_is_loaded_retains_package_major_identity() {
    let source = "export module Provider { public fun value() -> Integer { 1 } }";
    let sources = [iris_native_host::PackageSource {
        package_id: "host.example".into(), api_major: 7, version: "7.1.0".into(), path: "current.iris".into(),
        allowed_imports: Default::default(),
        source: "module Provider { public fun value() -> Integer { 9 } }; let restored = Provider.rollback(71); Provider.value()".into(),
    }];
    let mut given = Machine::new().unwrap();
    given.enter_package_history(PackageHistory {
        artifacts: vec![RevisionArtifact {
            package_id: "host.example".into(),
            api_major: 7,
            ..artifact(source)
        }],
    });
    let program = iris_vm::compile_package_tree_with_natives(
        &sources,
        &iris_native_host::NativeRegistry::new(),
    )
    .unwrap();
    let when = given.execute(&program);
    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}
