#![expect(
    clippy::expect_used,
    reason = "tests require compiled runtime fixtures"
)]

use super::*;
use iris_runtime::Value;

mod escaped;

const GIVEN: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1
   (next.call() as Integer) + calls
  })
 }
}
class Bad {}
impl Bad for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target {
 @Wrap() public fun value() -> Integer { 10 }
 @Wrap() public fun sibling() -> Integer { 30 }
}
let target = Target.new()
let old = target.value
let first = old.call()
let sibling = target.sibling()
open class Target {
 @Wrap() public override fun value() -> Integer { 20 }
 @Bad() public fun late() -> Integer { 0 }
}
target.value()
"#;

#[test]
fn candidate_artifacts_roll_back_when_late_transform_has_wrong_kind() {
    assert_failed_candidate(
        GIVEN,
        MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"),
    );
}

#[test]
fn candidate_artifacts_roll_back_when_late_signature_is_incompatible() {
    let given = GIVEN.replace(
        "@Bad() public fun late() -> Integer { 0 }",
        "public override fun sibling() -> String { \"bad\" }",
    );
    assert_failed_candidate(&given, MachineError::TypeContractError);
}

#[test]
fn candidate_artifacts_roll_back_when_late_transform_returns_bad_type() {
    let given = GIVEN.replace(
        "Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })",
        "nil",
    );
    assert_failed_candidate(&given, MachineError::TypeContractError);
}

fn assert_failed_candidate(source: &str, expected: MachineError) {
    let mut program = crate::compile(source).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core records");
    let mut classes = machine.register_classes(&program).expect("classes");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let boundary = program
        .instructions
        .iter()
        .position(|instruction| matches!(instruction, crate::Instruction::ApplyReopen { .. }))
        .expect("reopen");
    machine
        .run_body(
            &program.instructions[..boundary],
            program.registers,
            Vec::new(),
            &program,
            &classes,
        )
        .expect("origin");
    let target = match machine.bindings.get("target") {
        Some(Value::Object(object)) => *object,
        _ => unreachable!("target instance"),
    };
    let class = machine.runtime.class_of(target).expect("class");
    classes.push(class);
    let index = program
        .classes
        .iter()
        .position(|known| known.name == "Target")
        .expect("target");
    let before = machine
        .runtime
        .registry()
        .active(class)
        .expect("revision")
        .clone();
    let signatures = machine.method_signatures.clone();
    let chains = machine.wrapper_chains.len();
    let metadata = machine.decorator_metadata.len();
    let closures = machine.closures.keys().copied().collect::<Vec<_>>();
    let counter = machine.next_closure;
    let commit = machine.next_commit;
    let history = machine.revision_history.clone();
    let selector = selector_id(&program, "value").expect("selector");
    let old = machine
        .runtime
        .dispatch_instance(target, selector)
        .expect("old method");
    let sibling = machine
        .runtime
        .dispatch_instance(target, selector_id(&program, "sibling").expect("sibling"))
        .expect("sibling method");

    let when = machine.apply_reopen(&program, &classes, index, 0);

    assert_eq!(when, Err(expected));
    let after = machine.runtime.registry().active(class).expect("revision");
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    assert_eq!(after.commit_id(), before.commit_id());
    assert_eq!(machine.method_signatures, signatures);
    assert_eq!(machine.wrapper_chains.len(), chains);
    assert_eq!(machine.decorator_metadata.len(), metadata);
    assert!(
        closures
            .iter()
            .all(|identity| machine.closures.contains_key(identity))
    );
    assert!(machine.next_closure > counter);
    assert_eq!(machine.next_commit, commit);
    assert_eq!(machine.revision_history, history);
    assert!(machine.pending_replacements.is_empty());
    assert!(machine.decorator_phases.is_empty());
    assert_eq!(machine.open_depth, 0);
    assert_eq!(
        machine.invoke_wrapped(old, vec![Value::Object(target)], &program, &classes),
        Ok(Value::Integer(12_u64.into()))
    );

    program.classes[index].reopens[0].methods.truncate(1);
    program.decorator_applications.retain(|application| {
        application.target
            != (crate::compile::decorators::Target::Reopen {
                class: index,
                artifact: 0,
            })
            || application.method == Some(program.classes[index].reopens[0].methods[0].1)
    });
    machine
        .apply_reopen(&program, &classes, index, 0)
        .expect("retry");
    let current = machine
        .runtime
        .dispatch_instance(target, selector)
        .expect("new method");
    assert_ne!(current.id(), old.id());
    assert_eq!(
        machine
            .runtime
            .registry()
            .active(class)
            .expect("revision")
            .number(),
        before.number() + 1
    );
    assert_eq!(
        machine.invoke_wrapped(current, vec![Value::Object(target)], &program, &classes),
        Ok(Value::Integer(21_u64.into()))
    );
    assert_eq!(
        machine.invoke_wrapped(old, vec![Value::Object(target)], &program, &classes),
        Ok(Value::Integer(13_u64.into()))
    );
    assert_eq!(
        machine.invoke_wrapped(sibling, vec![Value::Object(target)], &program, &classes),
        Ok(Value::Integer(32_u64.into()))
    );
    assert!(machine.runtime.registry().method_by_id(old.id()).is_some());
    assert!(
        machine
            .runtime
            .registry()
            .method_by_id(current.id())
            .is_some()
    );
}
