use iris_runtime::Value;
use iris_vm::{Machine, MachineError, PackageHistory, RevisionArtifact};

fn load(history: &str, current: &str, probe: &str) -> Result<Value, MachineError> {
    let sources = [iris_native_host::PackageSource {
        package_id: "accessor.history".into(),
        api_major: 7,
        version: "7.1.0".into(),
        path: "current.iris".into(),
        allowed_imports: Default::default(),
        source: format!("{current}; {probe}"),
    }];
    let mut machine = Machine::new().unwrap();
    machine.enter_package_history(PackageHistory {
        artifacts: vec![RevisionArtifact {
            package_id: "accessor.history".into(),
            api_major: 7,
            logical_owner: "Target".into(),
            revision: 1,
            artifact: (
                "opaque:accessors".into(),
                iris_runtime::artifact_digest(history.as_bytes()),
                history.into(),
            ),
        }],
    });
    let program = iris_vm::compile_package_tree_with_natives(
        &sources,
        &iris_native_host::NativeRegistry::new(),
    )
    .unwrap();
    machine.execute(&program)
}

const DECORATORS: &str = r#"
class Read {
 public fun offset() -> Integer { 10 }
}
impl Read for PropertyDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1; next.call() + offset() + calls
  })
 }
}
class Write {}
impl Write for PropertyDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
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
"#;

const TARGET: &str = r#"class Target {
 public fun initialize() { @value = 1 }
 @Read() public property fun value() -> Integer { @value + 10 }
 @Write() public property fun value=(value: Integer) -> Symbol { @value = value; :historical }
 @Cache() public class fun count(value: Integer) -> Integer { value + 10 }
 @Cache() public fun retained(value: Integer) -> Integer { value + 10 }
}"#;

#[test]
fn restores_accessors_when_current_raw_state_must_survive() {
    let given = format!("{DECORATORS} {TARGET}; raise :unexecuted");
    let current = format!(
        "{DECORATORS} {}",
        TARGET
            .replace("+ 10", "+ 100")
            .replace(":historical", ":current")
    );
    let when = load(
        &given,
        &current,
        r#"
     let target = Target.new(); let written = target.value = 9
     let revision = Target.active_revision; let commit = Reflection::Class.revision(Target).fetch(:commit_id)
     let restored = Target.rollback(1)
     written == :current && target.value == 30 && (target.value = 2) == :historical && target.value == 24 && Target.count(3) == 13 && Target.active_revision == revision + 1 && Reflection::Class.revision(Target).fetch(:commit_id) == commit + 1
    "#,
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn refreshes_wrappers_when_retained_methods_keep_their_own_bundles() {
    let given = format!("{TARGET} {DECORATORS}; raise :unexecuted");
    let current = format!("{DECORATORS} {}", TARGET.replace("+ 10", "+ 100"));
    let when = load(
        &given,
        &current,
        r#"
     let target = Target.new(); let old = target.retained
     let warm = old.call(1); let singleton_warm = Target.count(1)
     open class Read { public override fun offset() -> Integer { 99 } }
     let first = Target.rollback(1); let retained = target.retained
     let once = retained.call(2); let singleton_once = Target.count(2)
     let getter = target.value; let again = target.value; let second = Target.rollback(1)
     let collected = NativeFixture.compact_gc()
     warm == 101 && singleton_warm == 101 && once == 12 && singleton_once == 12 && getter == 22 && again == 23 && target.value == 22 && Target.count(3) == 13 && Target.count(99) == 13 && old.call(99) == 101 && retained.call(99) == 12 && Read.new().offset() == 99 && first == second
    "#,
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn restores_only_body_when_property_set_and_method_capabilities_are_denied() {
    let given = "class Target { public property fun value() -> Integer { 1 }; public property fun value=(value: Integer) -> Symbol { :written } }";
    for policy in [
        "method_body",
        "method_set",
        "property_set",
        "method_body, method_set, property_set",
    ] {
        let current = given
            .replace(
                "class Target {",
                &format!("class Target meta deny {policy} {{"),
            )
            .replace("{ 1 }", "{ 9 }");
        let when = load(
            given,
            &current,
            "let restored = Target.rollback(1); let target = Target.new(); target.value == 1 && (target.value = 2) == :written",
        );
        assert_eq!(when, Ok(Value::Bool(true)), "{policy}");
    }
}

#[test]
fn rejects_history_atomically_when_accessor_promises_differ() {
    let current = "class Target { public property fun value() -> Integer { 9 }; public property fun value=(value: Integer) -> Symbol { :current }; public class fun count() -> Integer { 9 } }";
    for given in [
        current.replace(
            "public property fun value=(value: Integer) -> Symbol { :current };",
            "",
        ),
        current.replace("public property fun value() -> Integer { 9 };", ""),
        current.replace("public class fun count() -> Integer { 9 }", ""),
        current.replace("public property", "private property"),
        current.replace("-> Symbol { :current }", "-> Integer { 1 }"),
        current.replace("value: Integer", "value: String"),
        current.replace("public property fun value()", "public fun value()"),
    ] {
        let when = load(
            &given,
            current,
            "let target = Target.new(); let before = Target.active_revision; let rejected = try { Target.rollback(1); false } catch error { error == :TypeContractError }; rejected && target.value == 9 && (target.value = 3) == :current && Target.count() == 9 && Target.active_revision == before",
        );
        assert_eq!(when, Ok(Value::Bool(true)), "{given}");
    }
}

#[test]
fn rejects_body_policy_when_history_would_replace_a_denied_slot() {
    for (member, denied, probe) in [
        (
            "public property fun value() -> Integer",
            "property_body",
            "Target.new().value",
        ),
        (
            "public class fun value() -> Integer",
            "method_body",
            "Target.value()",
        ),
    ] {
        let given = format!("class Target {{ {member} {{ 1 }} }}");
        let current = format!("class Target meta deny {denied} {{ {member} {{ 9 }} }}");
        let when = load(
            &given,
            &current,
            &format!(
                "let before = Target.active_revision; let rejected = try {{ Target.rollback(1); false }} catch error {{ true }}; rejected && {probe} == 9 && Target.active_revision == before"
            ),
        );
        assert_eq!(when, Ok(Value::Bool(true)), "{denied}");
    }
}

#[test]
fn checks_setter_result_when_historical_wrapper_short_circuits() {
    let current = format!("{DECORATORS} {TARGET}");
    let given = current.replace("-> Object; next.call()", "-> Object; 42");
    let when = load(
        &given,
        &current,
        "let restored = Target.rollback(1); let target = Target.new(); try { target.value = 3; false } catch error { error is? TypeError }",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn matches_slot_kind_when_instance_and_singleton_share_a_selector() {
    let given = "class Target { public fun value() -> Integer { 1 }; public class fun value() -> Integer { 2 } }";
    let current = given
        .replace("{ 1 }", "{ 8 }")
        .replace("{ 2 }", "{ 9 }")
        .replace("class Target {", "class Target meta deny method_set {");
    let when = load(
        given,
        &current,
        "let restored = Target.rollback(1); Target.value() == 2 && Target.new().value() == 1",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn preserves_all_slots_when_late_accessor_transform_fails() {
    let current = format!("{DECORATORS} {TARGET}");
    let given = current.replace(
        "Transformation.wrap_setter(",
        "raise :failed; Transformation.wrap_setter(",
    );
    let when = load(
        &given,
        &current,
        r#"
     let target = Target.new(); let old = target.retained; let warm = old.call(1)
     let singleton_warm = Target.count(1); let getter_warm = target.value; let before = Target.active_revision
     let rejected = try { Target.rollback(1); false } catch error { error == :failed }
     rejected && Target.count(99) == 11 && old.call(99) == 11 && target.value == 23 && (target.value = 9) == :historical && Target.active_revision == before
    "#,
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn rejects_stored_state_when_replay_cannot_restore_storage() {
    for current in [
        "class Target { public property value: Integer = 9 }",
        "class Target { public class property value: Integer = 9 }",
    ] {
        for given in [current, "class Target { }"] {
            let when = load(given, current, "Target.rollback(1)");
            assert_eq!(when, Err(MachineError::UnsupportedConstruct));
        }
    }
}

#[test]
fn restores_exact_chains_when_both_operations_address_one_property() {
    let given = r#"
     class Both {}
     impl Both for PropertyDecorator {
      public fun plan(d, a) -> Plan { Plan.empty }
      public fun transform(d, a, c) -> Transformation {
       let digit = a[0] as Integer
       Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() * 10 + digit }).wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() * 10 + digit })
      }
     }
     class Target {
      @Both(1) @Both(2) public property fun value() -> Integer { 7 }
      public property fun value=(value: Integer) -> Integer { value + 1 }
     }
    "#;
    let current = given.replace("@Both(1) @Both(2)", "@Both(3)");
    let when = load(
        given,
        &current,
        "let target = Target.new(); let warm = target.value; let restored = Target.rollback(1); warm == 73 && target.value == 721 && (target.value = 8) == 921",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn removes_current_wrappers_when_historical_slots_are_undecorated() {
    let current = format!("{DECORATORS} {TARGET}");
    let given = current
        .replace("@Read()", "")
        .replace("@Write()", "")
        .replace("@Cache()", "");
    let when = load(
        &given,
        &current,
        "let target = Target.new(); let old = target.retained; let warm = old.call(1); let restored = Target.rollback(1); target.value == 11 && target.retained(2) == 12 && target.retained(3) == 13 && Target.count(2) == 12 && Target.count(3) == 13 && old.call(99) == 11",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
}
