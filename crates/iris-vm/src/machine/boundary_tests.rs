#![expect(
    clippy::unwrap_used,
    reason = "private harness asserts valid setup and retained runtime state"
)]
#![expect(clippy::panic, reason = "test fixture must construct an instance")]

use super::{Machine, MachineError, selector_id};
use crate::{Instruction, compile};
use iris_runtime::Value;

#[test]
fn active_state_is_unchanged_when_reopen_validation_fails() {
    let program = compile(
        r#"
        class Rule { public fun total() -> Integer { 10 } }
        let rule = Rule.new()
        open class Rule {
            public fun sibling() -> Integer { 7 }
            public override fun total() -> String { "bad" }
        }
        rule.total()
    "#,
    )
    .unwrap();
    let mut machine = Machine::new().unwrap();
    let classes = machine.register_classes(&program).unwrap();
    let boundary = program
        .instructions
        .iter()
        .position(|instruction| matches!(instruction, Instruction::ApplyReopen { .. }))
        .unwrap();
    machine
        .run_body(
            &program.instructions[..boundary],
            program.registers,
            Vec::new(),
            &program,
            &classes,
        )
        .unwrap();
    let class = classes[0];
    let before = machine.runtime.registry().active(class).unwrap().clone();
    let Value::Object(instance) = machine.bindings["rule"] else {
        panic!("instance missing");
    };

    let outcome = machine.apply_reopen(&program, &classes, 0, 0);

    assert_eq!(outcome, Err(MachineError::TypeContractError));
    let after = machine.runtime.registry().active(class).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    assert_eq!(after.singleton_methods(), before.singleton_methods());
    let sibling = selector_id(&program, "sibling").unwrap();
    assert!(!machine.runtime.registry().staged_has_method(class, sibling));
    let selector = selector_id(&program, "total").unwrap();
    let method = machine
        .runtime
        .dispatch_instance(instance, selector)
        .unwrap();
    let result = machine.invoke_function(
        usize::try_from(method.body().raw()).unwrap(),
        vec![Value::Object(instance)],
        &program,
        &classes,
    );
    assert_eq!(result, Ok(Value::Integer(10_u64.into())));
}

#[test]
fn group_is_unchanged_when_callback_replacement_is_incompatible() {
    let program = compile(
        r#"
        class Rule { public fun total() -> Integer { 10 } }
        class Other { }
        let rule = Rule.new()
        Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            Other.open({ |other|; other.define_method(:sibling) { 8 } })
            candidate.define_method(:total) { "bad" }
        })
    "#,
    )
    .unwrap();
    let mut machine = Machine::new().unwrap();
    let classes = machine.register_classes(&program).unwrap();
    let before: Vec<_> = classes
        .iter()
        .map(|class| machine.runtime.registry().active(*class).unwrap().clone())
        .collect();

    let outcome = machine.run_body(
        &program.instructions,
        program.registers,
        Vec::new(),
        &program,
        &classes,
    );

    assert_eq!(outcome, Err(MachineError::TypeContractError));
    for (class, before) in classes.iter().zip(before) {
        let after = machine.runtime.registry().active(*class).unwrap();
        assert_eq!(after.id(), before.id());
        assert_eq!(after.methods(), before.methods());
    }
    assert!(machine.revision_history.is_empty());
}

#[test]
fn candidate_is_discarded_when_staging_fails_after_sibling() {
    let mut program = compile(
        r#"
        class Rule { public fun total() -> Integer { 10 } }
        open class Rule { public fun sibling() -> Integer { 7 } }
        1
    "#,
    )
    .unwrap();
    let mut machine = Machine::new().unwrap();
    let classes = machine.register_classes(&program).unwrap();
    let before = machine
        .runtime
        .registry()
        .active(classes[0])
        .unwrap()
        .clone();
    program.classes[0].reopens[0]
        .methods
        .push(("absent-selector".to_owned(), 0));

    let outcome = machine.apply_reopen(&program, &classes, 0, 0);

    assert!(matches!(outcome, Err(MachineError::UnknownSelector(_))));
    let after = machine.runtime.registry().active(classes[0]).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    let sibling = selector_id(&program, "sibling").unwrap();
    assert!(
        !machine
            .runtime
            .registry()
            .staged_has_method(classes[0], sibling)
    );
}

#[test]
fn singleton_table_is_unchanged_when_reopen_validation_fails() {
    let program = compile(
        r#"
        class Rule { public class fun total() -> Integer { 10 } }
        open class Rule {
            public class fun sibling() -> Integer { 7 }
            public override class fun total() -> String { "bad" }
        }
        Rule.total()
    "#,
    )
    .unwrap();
    let mut machine = Machine::new().unwrap();
    let classes = machine.register_classes(&program).unwrap();
    let before = machine
        .runtime
        .registry()
        .active(classes[0])
        .unwrap()
        .clone();

    let outcome = machine.apply_reopen(&program, &classes, 0, 0);

    assert_eq!(outcome, Err(MachineError::TypeContractError));
    let after = machine.runtime.registry().active(classes[0]).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.singleton_methods(), before.singleton_methods());
}
