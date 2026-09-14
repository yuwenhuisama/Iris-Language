use iris_runtime::decorator_protocol::*;
use iris_runtime::{
    ArrayRef, ClassId, ImmutableArray, ImmutableHash, ObjectId, Runtime, StaticSpine, Value,
};

fn types() -> SnapshotTypes {
    SnapshotTypes {
        object: Value::Type(ClassId::new(1), vec![]),
        symbol: Value::Type(ClassId::new(2), vec![]),
        type_type: Value::Type(ClassId::new(3), vec![]),
        invocation_parameter: Value::Type(ClassId::new(4), vec![]),
        symbol_object_tuple: Value::Type(ClassId::new(5), vec![]),
    }
}

fn array(elements: Vec<Value>, element_type: &Value) -> Value {
    Value::ImmutableArray(ImmutableArray::new(elements, element_type.clone()))
}

#[test]
fn invocation_field_projection_preserves_all_channels_and_type_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let types = types();
    let signature = InvocationSignature::new(
        vec![
            InvocationParameter::new("arg", ParameterCategory::Positional, types.object.clone())
                .with_optional(true),
        ],
        types.object.clone(),
        false,
    )?;
    let selected = SelectedCall::new(
        Value::Object(ObjectId::new(10)),
        InvocationSlot::new(Value::Class(ClassId::new(6)), "work", SlotKind::Method)
            .qualified(types.object.clone()),
        signature.clone(),
    )
    .with_type_arguments(vec![types.symbol.clone()], vec![types.object.clone()]);
    let payload =
        InvocationPayload::from_bindings(&signature, &[Value::Integer(7_u8.into())], |_, _| true)?;
    let source = SourceCall::new(vec![], vec![], Some(Value::Nil))
        .with_provenance(vec!["arg".into()], vec!["arg".into()]);
    let invocation = Invocation::new(selected, source, payload);
    let changes = ArgumentChanges::parse(&[(
        "positional".into(),
        Value::ImmutableHash(ImmutableHash::new(
            vec![(Value::Symbol("arg".into()), Value::Integer(9_u8.into()))],
            types.symbol.clone(),
            types.object.clone(),
        )),
    )])?;
    let patched = DecoratorValue::Invocation(Box::new(invocation.patch(&changes, |_, _| true)?));
    let expected = [
        ("receiver", Value::Object(ObjectId::new(10))),
        (
            "slot",
            Value::Tuple(vec![
                Value::Class(ClassId::new(6)),
                Value::Symbol("work".into()),
                types.object.clone(),
                Value::Symbol("method".into()),
            ]),
        ),
        (
            "signature",
            DecoratorValue::InvocationSignature(signature).into(),
        ),
        (
            "owner_type_arguments",
            array(vec![types.symbol.clone()], &types.type_type),
        ),
        (
            "method_type_arguments",
            array(vec![types.object.clone()], &types.type_type),
        ),
        (
            "positional",
            Value::ImmutableHash(ImmutableHash::new(
                vec![(Value::Symbol("arg".into()), Value::Integer(9_u8.into()))],
                types.symbol.clone(),
                types.object.clone(),
            )),
        ),
        (
            "keywords",
            Value::ImmutableHash(ImmutableHash::new(
                vec![],
                types.symbol.clone(),
                types.object.clone(),
            )),
        ),
        ("rest", array(vec![], &types.object)),
        (
            "keyword_rest",
            Value::ImmutableHash(ImmutableHash::new(
                vec![],
                types.symbol.clone(),
                types.object.clone(),
            )),
        ),
        ("block", Value::Nil),
        ("original_positional", array(vec![], &types.object)),
        (
            "original_keywords",
            array(vec![], &types.symbol_object_tuple),
        ),
        ("original_block", Value::Nil),
        ("block_omitted", Value::Bool(false)),
        (
            "omitted",
            array(vec![Value::Symbol("arg".into())], &types.symbol),
        ),
        (
            "defaulted",
            array(vec![Value::Symbol("arg".into())], &types.symbol),
        ),
        (
            "replaced",
            array(vec![Value::Symbol("arg".into())], &types.symbol),
        ),
    ];
    for (name, expected) in expected {
        assert_eq!(patched.read_field(name, &types), Some(expected), "{name}");
    }
    assert_eq!(patched.read_field("selected", &types), None);
    assert_eq!(
        invocation.payload().positional()[0].1,
        Value::Integer(7_u8.into())
    );
    Ok(())
}

#[test]
fn boxed_invocation_roots_survive_collection_through_snapshot_cycles()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let held = runtime.allocate(class)?;
    let dead = runtime.allocate(class)?;
    let cycle = ArrayRef::new(vec![Value::Object(held)]);
    let signature = InvocationSignature::new(
        vec![InvocationParameter::new(
            "arg",
            ParameterCategory::Positional,
            Value::Type(class, vec![]),
        )],
        Value::Type(class, vec![]),
        false,
    )?;
    let payload =
        InvocationPayload::from_bindings(&signature, &[Value::Array(cycle.clone())], |_, _| true)?;
    let selected = SelectedCall::new(
        Value::Nil,
        InvocationSlot::new(Value::Class(class), "work", SlotKind::Method),
        signature,
    );
    let record: Value = DecoratorValue::Invocation(Box::new(Invocation::new(
        selected,
        SourceCall::new(vec![], vec![], None),
        payload,
    )))
    .into();
    let root = array(vec![record], &types().object);
    cycle.mutate(|values| values.push(root.clone()));
    let (freed, _) = runtime.collect_garbage([&root]);
    assert_eq!(freed, 1);
    assert!(runtime.identity_hash(held).is_ok());
    assert!(runtime.identity_hash(dead).is_err());
    cycle.mutate(Vec::clear);
    assert_eq!(runtime.collect_garbage([]).0, 1);
    Ok(())
}

#[test]
fn changes_reject_readonly_views_and_bad_immutable_keys() {
    let types = types();
    let bad = Value::ImmutableHash(ImmutableHash::new(
        vec![(Value::Text("arg".into()), Value::Nil)],
        types.symbol,
        types.object,
    ));
    assert_eq!(
        ArgumentChanges::parse(&[("positional".into(), bad)]),
        Err(ArgumentError::NonSymbolKey(ChangeField::Positional))
    );
    assert_eq!(
        ArgumentChanges::parse(&[("rest".into(), Value::ReadonlyArray(vec![]))]),
        Err(ArgumentError::WrongShape(ChangeField::Rest))
    );
}
