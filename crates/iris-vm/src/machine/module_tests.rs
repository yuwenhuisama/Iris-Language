#![expect(
    clippy::expect_used,
    reason = "tests require compiled runtime fixtures"
)]

use super::{Machine, MachineError, selector_id};
use crate::compile;
use iris_runtime::{MethodBody, StaticSpine, Value};

mod replay;

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |inv: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   (next.call() as Integer) + 10
  })
 }
}
"#;

#[test]
fn mixed_group_rolls_back_when_module_wrapper_capability_is_denied() {
    let program = compile(&format!("{WRAP} module Provider meta deny method_body {{ @Wrap() public fun value() -> Integer {{ 7 }} }} 0")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let classes = machine.register_classes(&program).expect("classes");
    let sibling = machine
        .runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(900), None, &[])
        .expect("sibling");
    let signatures = machine.method_signatures.len();
    let result = machine.publish_module(0, &program, &classes);
    assert!(
        matches!(result, Err(MachineError::Raised(ref error)) if error.0 == Value::Symbol("MetaCapabilityError".into()))
    );
    assert!(machine.runtime.registry().class(sibling).is_err());
    assert!(machine.runtime.registry().staged_origin(sibling).is_err());
    assert!(machine.active_module_id("Provider").is_err());
    assert!(machine.wrapper_chains.is_empty());
    assert_eq!(machine.method_signatures.len(), signatures);
}

#[test]
fn module_and_class_publish_together_when_group_has_independent_candidates() {
    let program = compile(&format!(
        "{WRAP} module Provider {{ @Wrap() public fun value() -> Integer {{ 7 }} }} 0"
    ))
    .expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let classes = machine.register_classes(&program).expect("classes");
    let sibling = machine
        .runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(900), None, &[])
        .expect("sibling");
    machine
        .publish_module(0, &program, &classes)
        .expect("publish");
    let module = machine.active_module_id("Provider").expect("module");
    let revision = machine
        .runtime
        .registry()
        .active_module(module)
        .expect("revision");
    assert_eq!(revision.number(), 1);
    assert_eq!(
        revision.commit_id(),
        machine
            .runtime
            .registry()
            .active(sibling)
            .expect("class")
            .commit_id()
    );
}

#[test]
fn retained_method_keeps_wrapper_when_module_body_is_replaced() {
    let program = compile(&format!("{WRAP} module Provider {{ @Wrap() public fun value() -> Integer {{ 7 }} public fun replacement() -> Integer {{ 9 }} }} 0")).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.execute(&program).expect("execute");
    let module = machine.active_module_id("Provider").expect("module");
    let selector = selector_id(&program, "value").expect("selector");
    let old = machine
        .runtime
        .registry()
        .module_method(module, selector)
        .expect("old");
    let replacement = program
        .functions
        .iter()
        .position(|function| function.name == "Provider.replacement")
        .expect("replacement");
    machine
        .runtime
        .registry_mut()
        .begin_module_transaction(module)
        .expect("candidate");
    machine
        .runtime
        .registry_mut()
        .stage_module_method_body(
            module,
            selector,
            MethodBody::new(u64::try_from(replacement).expect("index")),
        )
        .expect("stage");
    machine.runtime.commit_structural_group().expect("commit");
    let result = machine.invoke_wrapped(old, vec![Value::Symbol("Provider".into())], &program, &[]);
    assert_eq!(result, Ok(Value::Integer(17_u64.into())));
    assert_eq!(
        machine.invoke_module_member(("Provider", "value", None), Vec::new(), &program, &[]),
        Ok(Value::Integer(9_u64.into()))
    );
}
