use iris_runtime::decorator_protocol::*;
use iris_runtime::{ArrayRef, ClassId, HeapPayload, ObjectId, RuntimeHeap, Value, reachable_from};

#[test]
fn invocation_visitor_reaches_receiver_source_payload_types_and_cycles()
-> Result<(), Box<dyn std::error::Error>> {
    let mut heap = RuntimeHeap::new();
    let hidden = heap.alloc(ClassId::new(1), HeapPayload::InstanceFields(vec![]))?;
    let cycle = ArrayRef::new(vec![Value::Object(hidden)]);
    cycle.mutate(|values| values.push(Value::Array(cycle.clone())));
    let signature = InvocationSignature::new(
        vec![InvocationParameter::new(
            "arg",
            ParameterCategory::Positional,
            Value::Object(ObjectId::new(10)),
        )],
        Value::Object(ObjectId::new(11)),
        true,
    )?;
    let payload =
        InvocationPayload::from_bindings(&signature, &[Value::Array(cycle.clone())], |_, _| true)?;
    let selected = SelectedCall::new(
        Value::Object(ObjectId::new(12)),
        InvocationSlot::new(Value::Object(ObjectId::new(13)), "work", SlotKind::Getter)
            .qualified(Value::Object(ObjectId::new(14))),
        signature,
    )
    .with_type_arguments(
        vec![Value::Object(ObjectId::new(15))],
        vec![Value::Object(ObjectId::new(16))],
    );
    let source = SourceCall::new(
        vec![Value::Object(ObjectId::new(17))],
        vec![("key".into(), Value::Object(ObjectId::new(18)))],
        Some(Value::Closure(ObjectId::new(19))),
    );
    let invocation = Invocation::new(selected, source, payload);
    let mut roots = Vec::new();
    invocation.visit_values(&mut |value| roots.push(value));
    let reached = reachable_from(&heap, &Default::default(), roots);
    assert!(reached.contains(hidden));
    for raw in 10..=19 {
        assert!(reached.contains(ObjectId::new(raw)));
    }
    cycle.mutate(Vec::clear);
    Ok(())
}

#[test]
fn operation_and_changes_visitors_keep_captured_values_reachable()
-> Result<(), Box<dyn std::error::Error>> {
    let phase = DecoratorPhase::new(DecoratorKind::Class, DecoratorReason::Origin);
    let transformation = Transformation::empty(Some(&phase))?.append(
        Some(&phase),
        Operation::AddMethod {
            selector: "work".into(),
            body: Value::Closure(ObjectId::new(21)),
        },
    )?;
    let changes = ArgumentChanges::parse(&[("block".into(), Value::Closure(ObjectId::new(22)))])?;
    let mut roots = Vec::new();
    transformation.visit_values(&mut |value| roots.push(value));
    changes.visit_values(&mut |value| roots.push(value));
    let reached = reachable_from(&RuntimeHeap::new(), &Default::default(), roots);
    assert_eq!(reached.ids(), vec![ObjectId::new(21), ObjectId::new(22)]);
    Ok(())
}
