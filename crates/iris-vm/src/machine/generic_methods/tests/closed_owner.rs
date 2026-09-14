#![expect(clippy::panic, reason = "tests destructure required fixture values")]

use crate::{compile, machine::Machine};
use iris_runtime::Value;

#[test]
fn closed_chain_survives_collection_when_original_program_is_dropped() {
    let given = format!(
        r#"{}
let box = Box<Integer>.new()
let first = box.value(7)
box.value
"#,
        include_str!("../../../../tests/closed_owner.iris")
    );
    let original = compile(&given).expect("compile");
    let mut machine = Machine::new().expect("machine");
    let Value::BoundMethod(bound) = machine.execute(&original).expect("bound method") else {
        panic!("bound method");
    };
    let current = compile("class Unrelated { public fun value() { 99 } } Unrelated.new().value()")
        .expect("current");
    machine.active_values.push(Value::BoundMethod(bound));
    machine.execute(&current).expect("current executes");
    drop(original);
    machine.collect_engine();

    let when =
        machine.invoke_bound_callable(&bound, &[Value::Integer(7_u64.into())], (&current, &[]));

    assert_eq!(when, Ok(Value::Integer(9_u64.into())));
}

#[test]
fn closed_pending_frames_survive_collection_when_caller_program_changes() {
    let source = include_str!("../../../../tests/closed_owner_async.iris");
    let (declarations, _) = source.split_once("let integer =").expect("entry");
    let original = compile(&format!(
        r#"{declarations}
let gate = Gate.new()
%[gate, Box<String>.new().value("retained", gate)]
"#
    ))
    .expect("original");
    let mut machine = Machine::new().expect("machine");
    let Value::Array(values) = machine.execute(&original).expect("pending") else {
        panic!("array");
    };
    let values = values.elements();
    let Value::Gate(gate) = values[0] else {
        panic!("gate");
    };
    machine.active_values.extend(values.iter().cloned());
    let current = compile("class Unrelated { public fun value() { 99 } } Unrelated.new().value()")
        .expect("current");
    machine.execute(&current).expect("current executes");
    drop(original);
    machine.collect_engine();
    machine.gates.insert(gate, Some(Value::Nil));

    let when = machine.observe_task(values[1].clone(), &current, &[]);

    assert_eq!(when, Ok(Value::Text("retained".into())));
}
