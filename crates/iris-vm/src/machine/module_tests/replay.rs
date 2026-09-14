use super::*;

#[test]
fn mixed_candidates_and_old_captures_survive_when_late_module_transform_fails() {
    let given = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  if context.reason == :open { effects.append(:transform) }
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
  effects.append(:bad)
  Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
mut effects = %[]
module Provider {
 @Wrap() public fun value() -> Integer { 10 }
 public fun late() -> Integer { 30 }
}
let first = Provider.value()
open module Provider {
 @Wrap() public override fun value() -> Integer { 20 }
 @Bad() public override fun late() -> Integer { 40 }
}
0
"#;
    let mut program = compile(given).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core");
    let classes = machine.register_classes(&program).expect("classes");
    machine
        .register_callable_types(&program)
        .expect("callables");
    let boundary = program
        .instructions
        .iter()
        .position(|instruction| {
            matches!(
                instruction,
                crate::Instruction::ApplyModuleDecorators { module: 1 }
            )
        })
        .expect("open");
    machine
        .run_body(
            &program.instructions[..boundary],
            program.registers,
            Vec::new(),
            &program,
            &classes,
        )
        .expect("origin");
    let module = machine.active_module_id("Provider").expect("module");
    let before = machine
        .runtime
        .registry()
        .active_module(module)
        .expect("revision")
        .clone();
    let old = before
        .method(selector_id(&program, "value").expect("selector"))
        .expect("method");
    let signatures = machine.method_signatures.clone();
    let chains = machine.wrapper_chains.len();
    let metadata = machine.decorator_metadata.len();
    let sibling = machine
        .runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(900), None, &[])
        .expect("sibling");

    let when = machine.publish_module(1, &program, &classes);

    assert_eq!(
        when,
        Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"))
    );
    assert_eq!(
        machine
            .runtime
            .registry()
            .active_module(module)
            .expect("revision"),
        &before
    );
    assert!(machine.runtime.registry().staged_origin(sibling).is_err());
    assert!(machine.runtime.registry().class(sibling).is_err());
    assert_eq!(machine.method_signatures, signatures);
    assert_eq!(machine.wrapper_chains.len(), chains);
    assert_eq!(machine.decorator_metadata.len(), metadata);
    assert_eq!(machine.open_depth, 0);
    assert_eq!(
        machine.bindings.get("effects"),
        Some(&Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Array(iris_runtime::ArrayRef::new(vec![
                Value::Symbol("transform".into()),
                Value::Symbol("bad".into())
            ]))
        ])))
    );
    assert_eq!(
        machine.invoke_wrapped(
            old,
            vec![Value::Symbol("Provider".into())],
            &program,
            &classes
        ),
        Ok(Value::Integer(12_u64.into()))
    );

    program.modules[1]
        .replay
        .as_mut()
        .expect("artifact")
        .functions
        .truncate(1);
    program
        .decorator_applications
        .retain(|application| application.decorator != 1);
    machine
        .publish_module(1, &program, &classes)
        .expect("retry");
    assert_eq!(
        machine.invoke_module_member(("Provider", "value", None), Vec::new(), &program, &classes),
        Ok(Value::Integer(21_u64.into()))
    );
    assert_eq!(
        machine.invoke_wrapped(
            old,
            vec![Value::Symbol("Provider".into())],
            &program,
            &classes
        ),
        Ok(Value::Integer(13_u64.into()))
    );
}
