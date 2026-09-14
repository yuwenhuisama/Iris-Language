#![expect(
    clippy::expect_used,
    reason = "tests require compiled runtime fixtures"
)]

use super::{Machine, selector_id};
use crate::compile;
use iris_runtime::{ClassId, Value};

const GIVEN: &str = r#"
class Wrap {}
impl Wrap for PropertyDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() }).wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Target { @Wrap() property value: Integer = 7 { public get; public set; } }
Target.new()
"#;

#[test]
fn generated_methods_retain_signatures_when_wrapped_origin_commits_once() {
    let program = compile(GIVEN).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let when = machine.execute(&program).expect("execute");
    let object = match when {
        Value::Object(object) => Some(object),
        _ => None,
    }
    .expect("object");
    let class = machine.runtime.class_of(object).expect("class");
    let registry = machine.runtime.registry();
    let revision = registry.active(class).expect("revision");
    let decorator = registry
        .active(ClassId::new(class.raw() - 1))
        .expect("decorator");
    assert_eq!(revision.number(), 1);
    assert_eq!(revision.commit_id(), decorator.commit_id() + 1);
    assert_eq!(revision.properties().len(), 1);
    for name in ["value", "value="] {
        let selector = selector_id(&program, name).expect("selector");
        let current = machine
            .runtime
            .dispatch_instance(object, selector)
            .expect("accessor");
        let retained: Vec<_> = machine
            .method_signatures
            .iter()
            .filter(|(_, signature)| signature.selector == name)
            .collect();
        assert_eq!(retained.len(), 2);
        for (identity, signature) in retained {
            let method = registry.method_by_id(*identity).expect("retained MethodID");
            assert_eq!(method.owner(), current.owner());
            assert_eq!(method.body(), current.body());
            assert_eq!(signature.kind, iris_syntax::MethodKind::Property);
        }
        assert!(machine.wrapper_chains.contains_key(&current.id()));
    }
    let slot = selector_id(&program, "value").expect("slot");
    assert_eq!(
        machine.runtime.raw_ivar(object, slot).expect("raw slot"),
        Value::Integer(7_u64.into())
    );
}

#[test]
fn generated_method_artifacts_are_tombstoned_when_origin_fails() {
    let given = GIVEN.replace("class Target {", "class Target meta deny property_body {");
    let program = compile(&given).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let when = machine.execute(&program);
    assert!(matches!(
        when,
        Err(super::MachineError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied {
                operation: iris_runtime::Capability::PropertyBody,
                ..
            }
        ))
    ));
    assert!(machine.wrapper_chains.is_empty());
    assert!(
        machine
            .method_signatures
            .values()
            .all(|signature| signature.selector != "value" && signature.selector != "value=")
    );
    assert!(!machine.bindings.contains_key("Target"));
}
