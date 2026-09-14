#![expect(
    clippy::expect_used,
    reason = "tests require compiled runtime fixtures"
)]

use super::Machine;
use crate::{compile, run};
use iris_runtime::{ClassId, Value};

#[path = "origin_state_tests.rs"]
mod state;

const DECORATORS: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Add {}
impl Add for ClassDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.add_method(:added, { || -> Integer; 42 })
 }
}
"#;

#[test]
fn decorated_origin_is_revision_one_when_only_methods_are_decorated() {
    let program = compile(&format!("{DECORATORS} class Target {{ @Wrap() public fun value() -> Integer {{ 7 }} }} Target.active_revision")).expect("compile");
    assert_eq!(run(&program), Ok(Value::Integer(1_u64.into())));
}

#[test]
fn decorated_origin_is_revision_one_when_class_adds_a_method() {
    let program = compile(&format!("{DECORATORS} @Add() class Target {{ public fun value() -> Integer {{ self.added() }} }} %[Target.active_revision, Target.new().value()]")).expect("compile");
    assert_eq!(
        run(&program),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(42_u64.into())
        ])))
    );
}

#[test]
fn plain_child_inherits_wrapped_parent_when_parent_origin_commits() {
    let program = compile(&format!("{DECORATORS} class Target {{ @Wrap() public fun value() -> Integer {{ 7 }} }} class Child extends Target {{ }} %[Target.active_revision, Child.new().value()]")).expect("compile");
    assert_eq!(
        run(&program),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(7_u64.into())
        ])))
    );
}

#[test]
fn failed_origin_is_hidden_and_consumes_no_publication_when_next_origin_succeeds() {
    let program = compile(&format!("{DECORATORS} @Add() class Target meta deny method_body {{ @Wrap() public fun value() -> Integer {{ 7 }} }} class Child extends Target {{ }} 0")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let classes = machine.register_classes(&program).expect("classes");
    let before = machine
        .runtime
        .registry()
        .active(classes[1])
        .expect("decorator")
        .clone();
    let signatures = machine.method_signatures.len();
    let metadata = machine.decorator_metadata.len();
    let outcome = machine.run_body(
        &program.instructions,
        program.registers,
        Vec::new(),
        &program,
        &classes,
    );
    assert!(outcome.is_err());
    assert!(machine.wrapper_chains.is_empty());
    assert_eq!(machine.method_signatures.len(), signatures);
    assert_eq!(machine.decorator_metadata.len(), metadata);
    let failed = ClassId::new(classes[1].raw() + 1);
    assert!(machine.runtime.registry().class(failed).is_err());
    assert!(machine.runtime.registry().staged_origin(failed).is_err());
    assert!(!machine.bindings.contains_key("Target"));
    let valid = compile("class Valid { public fun value() -> Integer { 9 } } Valid.new().value()")
        .expect("valid program");
    let mut next_classes = Vec::new();
    machine
        .publish_origin(&valid, &mut next_classes)
        .expect("valid origin on same machine");
    let next = next_classes[0];
    let revision = machine.runtime.registry().active(next).expect("revision");
    assert_eq!(revision.number(), 1);
    assert_eq!(revision.id().raw(), before.id().raw() + 1);
    assert_eq!(revision.commit_id(), before.commit_id() + 1);
}

#[test]
fn generated_add_method_signature_is_absent_when_origin_rolls_back() {
    // Given: a Class origin whose decorator adds a method before a later operation fails.
    let program = compile(
        r#"
class AddThenFail {}
impl AddThenFail for ClassDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
   Transformation.add_method(:added, { || -> Integer; 42 }).wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
@AddThenFail()
class Target meta deny method_body { public fun value() -> Integer { 7 } }
0
"#,
    )
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let mut classes = machine.register_classes(&program).expect("classes");
    assert!(
        !machine
            .method_signatures
            .values()
            .any(|signature| signature.selector == "added")
    );

    // When: publishing the origin fails after the generated method is installed.
    assert!(machine.publish_origin(&program, &mut classes).is_err());

    // Then: rollback leaves only resolvable method signature metadata.
    let registry = machine.runtime.registry();
    assert!(
        machine
            .method_signatures
            .keys()
            .all(|method| registry.method_by_id(*method).is_some())
    );
}

#[test]
fn generated_async_class_method_retains_async_signature_metadata() {
    // Given: a Class decorator generates an async method from a closure.
    let program = compile(
        r#"
class Add {}
impl Add for ClassDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.add_method(:added, { async || -> Integer; 42 })
 }
}
@Add()
class Target {}
0
"#,
    )
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let mut classes = machine.register_classes(&program).expect("classes");

    // When: the decorated origin is published.
    machine
        .publish_origin(&program, &mut classes)
        .expect("origin");
    let selector = super::selector_id(&program, "added").expect("selector");
    let method = machine
        .runtime
        .registry()
        .active(classes[1])
        .expect("target revision")
        .methods()
        .get(&selector)
        .copied()
        .expect("generated method");

    // Then: the registered signature declares the generated method async.
    assert!(
        machine
            .method_signatures
            .get(&method)
            .expect("generated signature")
            .is_async
    );
}

#[test]
fn provisional_class_cannot_be_loaded_when_transform_reads_its_name() {
    let source = "class Inspect {} impl Inspect for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { let leaked = Target; Transformation.empty } } @Inspect() class Target { } 0";
    let program = compile(source).expect("compile");
    let mut machine = Machine::new().expect("machine");
    assert!(machine.execute(&program).is_err());
    assert!(machine.decorator_metadata.is_empty());
}

#[test]
fn declaration_metadata_remains_readonly_when_origin_is_private() {
    let source = "class Inspect {} impl Inspect for ClassDecorator { public fun plan(d, a) -> Plan { if d.name != :Target { raise :wrong }; Plan.empty } public fun transform(d, a, c) -> Transformation { let error = try { d[:name] = :Other } catch error { error }; if error != :ReadonlyMutationError { raise :mutable }; Transformation.empty } } @Inspect() class Target { } Target.active_revision";
    assert_eq!(
        run(&compile(source).expect("compile")),
        Ok(Value::Integer(1_u64.into()))
    );
}

#[test]
fn mutable_class_payload_is_initialized_when_decorated_origin_commits() {
    let program = compile(&format!("{DECORATORS} @Add() class Target {{ public class property count: Integer = 9 }} Target.count")).expect("compile");
    assert_eq!(run(&program), Ok(Value::Integer(9_u64.into())));
}

#[test]
fn origin_takes_one_commit_when_transform_succeeds() {
    let program = compile(&format!(
        "{DECORATORS} class Target {{ @Wrap() public fun value() -> Integer {{ 7 }} }} 0"
    ))
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let mut classes = machine.register_classes(&program).expect("classes");
    let before = machine
        .runtime
        .registry()
        .active(classes[1])
        .expect("decorator")
        .clone();
    machine
        .publish_origin(&program, &mut classes)
        .expect("origin");
    let after = machine
        .runtime
        .registry()
        .active(classes[2])
        .expect("target");
    assert_eq!(after.number(), 1);
    assert_eq!(after.id().raw(), before.id().raw() + 1);
    assert_eq!(after.commit_id(), before.commit_id() + 1);
}

#[test]
fn author_effects_survive_when_transform_raises_before_publication() {
    let source = "class Effects { public class property count: Integer = 0 } class Fail {} impl Fail for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Effects.count = Effects.count + 1; raise :failed } } @Fail() class Target { } 0";
    let program = compile(source).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    let mut classes = machine.register_classes(&program).expect("classes");
    assert!(machine.publish_origin(&program, &mut classes).is_err());
    let count = super::selector_id(&program, "count").expect("count");
    assert_eq!(
        machine
            .runtime
            .class_var(classes[0], count)
            .expect("author state"),
        Some(Value::Integer(1_u64.into()))
    );
    assert!(machine.decorator_metadata.is_empty());
}
