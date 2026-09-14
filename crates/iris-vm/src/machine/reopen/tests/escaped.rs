use super::*;

const GIVEN: &str = r#"
class State {
 public class property saved: Object = nil
 public class property advance: Object = nil
 public class property transforms: Integer = 0
 public class fun fresh() -> Object { { || -> Integer; 99 } }
}
class Payload { public fun value() -> Integer { 1 } }
class Escape {}
impl Escape for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  mut captured = 41
  let payload = Payload.new()
  State.saved = { || -> Integer; captured + payload.value() }
  State.advance = { || -> Integer; captured = captured + 1 }
  State.transforms = State.transforms + 1
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
 }
}
class Bad {}
impl Bad for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation { FAILURE }
}
class Target { public fun value() -> Integer { 7 } }
let target = Target.new()
open class Target {
 @Escape() public override fun value() -> Integer { 700 }
 @Bad() public fun late() -> Integer { 900 }
}
"#;

enum Failure {
    BadType,
    Raised,
}

#[test]
fn escaped_closures_remain_callable_when_late_transform_returns_bad_type() {
    assert_escaped_effects(Failure::BadType);
}

#[test]
fn escaped_closures_remain_callable_when_late_transform_raises() {
    assert_escaped_effects(Failure::Raised);
}

fn assert_escaped_effects(failure: Failure) {
    let given = GIVEN.replace(
        "FAILURE",
        match failure {
            Failure::BadType => "nil",
            Failure::Raised => "raise 123",
        },
    );
    let parsed = iris_parser::parse(&given);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let program = crate::compile(&given).expect("compile");
    let mut machine = Machine::new().expect("machine");
    machine.register_core_records().expect("core records");
    let classes = machine.register_classes(&program).expect("classes");
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
    let index = program
        .classes
        .iter()
        .position(|class| class.name == "Target")
        .expect("target");
    let class = classes[index];
    let state = classes[program
        .classes
        .iter()
        .position(|class| class.name == "State")
        .expect("state")];
    let Value::Object(target) = machine.bindings["target"] else {
        unreachable!("target")
    };
    let before = machine
        .runtime
        .registry()
        .active(class)
        .expect("revision")
        .clone();
    let old = machine
        .runtime
        .dispatch_instance(target, selector_id(&program, "value").expect("value"))
        .expect("method");

    let when = machine.apply_reopen(&program, &classes, index, 0);

    match failure {
        Failure::BadType => assert_eq!(when, Err(MachineError::TypeContractError)),
        Failure::Raised => {
            let Err(MachineError::Raised(raised)) = when else {
                unreachable!("expected raise, got {when:?}")
            };
            assert_eq!(raised.0, Value::Integer(123_u64.into()));
        }
    }
    let after = machine.runtime.registry().active(class).expect("revision");
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    assert_eq!(after.commit_id(), before.commit_id());
    assert_eq!(
        machine.invoke_selected_method(old, vec![Value::Object(target)], &program, &classes),
        Ok(Value::Integer(7_u64.into()))
    );
    assert_eq!(
        machine
            .runtime
            .class_var(
                state,
                selector_id(&program, "transforms").expect("transforms")
            )
            .expect("state"),
        Some(Value::Integer(1_u64.into()))
    );
    let saved = machine
        .runtime
        .class_var(state, selector_id(&program, "saved").expect("saved"))
        .expect("state")
        .expect("saved value");
    let Value::Closure(saved) = saved else {
        unreachable!("escaped closure")
    };
    assert_eq!(
        machine.invoke_closure_value(saved, &[], &program, &classes),
        Ok(Value::Integer(42_u64.into()))
    );

    machine
        .native_fixture("compact_gc", &[])
        .expect("collection");
    assert_eq!(
        machine.invoke_closure_value(saved, &[], &program, &classes),
        Ok(Value::Integer(42_u64.into()))
    );
    let advance = machine
        .runtime
        .class_var(state, selector_id(&program, "advance").expect("advance"))
        .expect("state")
        .expect("advance value");
    let Value::Closure(advance) = advance else {
        unreachable!("shared capture writer")
    };
    assert_eq!(
        machine.invoke_closure_value(advance, &[], &program, &classes),
        Ok(Value::Integer(42_u64.into()))
    );
    assert_eq!(
        machine.invoke_closure_value(saved, &[], &program, &classes),
        Ok(Value::Integer(43_u64.into()))
    );

    let allocated = machine.next_closure;
    assert!(allocated > saved.raw());
    let later = machine
        .class_method_value(state, "fresh", &[], &program, &classes)
        .expect("fresh closure");
    let Value::Closure(later) = later else {
        unreachable!("later closure")
    };
    assert!(later.raw() >= allocated);
    assert_ne!(saved, later);
    assert_eq!(
        machine.invoke_closure_value(later, &[], &program, &classes),
        Ok(Value::Integer(99_u64.into()))
    );
    assert_eq!(
        machine.invoke_closure_value(saved, &[], &program, &classes),
        Ok(Value::Integer(43_u64.into()))
    );
}
