#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use crate::{Instruction, compile, verify};
use iris_syntax::TypeExpression;

mod closed_owner;

#[test]
fn generic_origin_publishes_nothing_when_transform_or_wrapper_validation_fails() {
    for transform in [
        "raise :failed",
        "Transformation.wrap_method({ |invocation, next| next.call() })",
        "7",
    ] {
        let given = compile(&format!(
            r#"
class Effects {{ public class property count: Integer = 0 }}
class Fail {{}}
impl Fail for MethodDecorator {{
 public fun plan(d, a) -> Plan {{ Plan.empty }}
 public fun transform(d, a, c) -> Transformation {{
  Effects.count = Effects.count + 1
  {transform}
 }}
}}
class Target {{ @Fail() public fun echo<Element>(value: Element) -> Element {{ value }} }}
0
"#
        ))
        .expect("compile");
        let mut machine = super::Machine::new().expect("machine");
        machine.register_core_records().expect("core");
        machine.register_callable_types(&given).expect("callables");
        let mut classes = machine.register_classes(&given).expect("classes");
        let before = machine
            .runtime
            .registry()
            .active(classes[1])
            .expect("decorator")
            .clone();

        let when = machine.publish_origin(&given, &mut classes);

        assert!(when.is_err(), "{transform}");
        assert_eq!(classes.len(), 2);
        let failed = iris_runtime::ClassId::new(classes[1].raw() + 1);
        assert!(machine.runtime.registry().class(failed).is_err());
        assert!(machine.runtime.registry().staged_origin(failed).is_err());
        assert!(machine.closed_methods.is_empty());
        assert!(machine.wrapper_chains.is_empty());
        assert!(machine.decorator_metadata.is_empty());
        let count = crate::machine::selector_id(&given, "count").expect("count");
        assert_eq!(
            machine
                .runtime
                .class_var(classes[0], count)
                .expect("effect"),
            Some(iris_runtime::Value::Integer(1_u64.into()))
        );
        let valid = compile("class Valid { } 0").expect("valid");
        let mut next_classes = Vec::new();
        machine
            .publish_origin(&valid, &mut next_classes)
            .expect("valid origin");
        let after = machine
            .runtime
            .registry()
            .active(next_classes[0])
            .expect("revision");
        assert_eq!(after.commit_id(), before.commit_id() + 1);
        assert_eq!(after.id().raw(), before.id().raw() + 1);
    }
}

#[test]
fn generic_ir_preserves_types_when_lowering_class_send() {
    let given = compile("class Target { public class fun echo<Element>(value: Element) -> Element { value } } Target.echo<Integer>(7)").expect("compile");
    let when = given
        .instructions
        .iter()
        .find_map(|instruction| match instruction {
            Instruction::GenericCall {
                call,
                type_arguments,
            } => Some((call.as_ref(), type_arguments)),
            _ => None,
        })
        .expect("generic call");
    assert!(matches!(when.0, Instruction::SendClass { .. }));
    assert_eq!(when.1, &[TypeExpression::Name("Integer".into())]);
}

#[test]
fn generic_verifier_rejects_operand_when_call_reads_out_of_range() {
    let mut given = compile("nil").expect("compile");
    given.instructions.push(Instruction::GenericCall {
        type_arguments: vec![TypeExpression::Name("Integer".into())],
        call: Box::new(Instruction::SendClass {
            destination: 0,
            class: 0,
            selector: "echo".into(),
            first: u16::MAX,
            count: 1,
        }),
    });
    let when = verify(&given);
    assert!(matches!(
        when,
        Err(crate::VerifyError::RegisterOutOfRange { .. })
    ));
}

#[test]
fn generic_verifier_rejects_instruction_when_payload_is_not_a_call() {
    let mut given = compile("nil").expect("compile");
    given.instructions.push(Instruction::GenericCall {
        type_arguments: Vec::new(),
        call: Box::new(Instruction::LoadNil { destination: 0 }),
    });
    let when = verify(&given);
    assert!(matches!(
        when,
        Err(crate::VerifyError::UnknownFunction { .. })
    ));
}

#[test]
fn closed_chain_keeps_identity_when_canonical_slot_is_replaced() {
    use iris_runtime::{DispatchOutcome, Value};
    let given = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; calls = calls + 1; (next.call() as Integer) + calls })
 }
}
class Target { @Wrap() public class fun echo<Element>(value: Element) -> Element { value } }
Target.echo<Integer>(7)
"#;
    let program = compile(given).expect("compile");
    let mut machine = super::Machine::new().expect("machine");
    assert_eq!(machine.execute(&program), Ok(Value::Integer(8_u64.into())));
    let old = machine.closed_methods[0].method;
    let iris_runtime::MethodOwner::Class(owner) = old.owner() else {
        panic!("class owner");
    };
    let DispatchOutcome::Invoke(canonical) = machine
        .runtime
        .registry()
        .dispatch_class_object(owner, old.selector())
        .expect("dispatch")
    else {
        panic!("method");
    };
    let current = machine
        .runtime
        .registry_mut()
        .publish_singleton_method(owner, old.selector(), old.body(), old.visibility())
        .expect("replace");
    let when = machine.invoke_wrapped(
        old,
        vec![Value::Class(owner), Value::Integer(7_u64.into())],
        &program,
        &[],
    );
    assert_ne!(old.id(), canonical.id());
    assert_ne!(old.id(), current.id());
    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn instance_closed_chain_survives_when_compatible_open_replaces_canonical_method() {
    use iris_runtime::{ArrayRef, Value};
    let given = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  mut calls = 0
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   calls = calls + 1; (next.call() as Integer) + calls
  })
 }
}
class Target { @Wrap() public fun echo<Element>(value: Element) -> Element { value } }
let target = Target.new()
let first = target.echo<Integer>(7)
open class Target { @Wrap() public override fun echo<Element>(value: Element) -> Element { (value as Integer) + 10 } }
%[first, target.echo<Integer>(7)]
"#;
    let program = compile(given).expect("compile");
    let mut machine = super::Machine::new().expect("machine");
    assert_eq!(
        machine.execute(&program),
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(8_u64.into()),
            Value::Integer(18_u64.into())
        ])))
    );
    let old = machine.closed_methods[0].method;
    let receiver = machine.bindings.get("target").expect("target").clone();
    let when = machine.invoke_wrapped(
        old,
        vec![receiver, Value::Integer(7_u64.into())],
        &program,
        &[],
    );
    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}
