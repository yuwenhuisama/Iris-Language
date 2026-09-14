use iris_runtime::decorator_protocol::*;
use iris_runtime::{
    ArrayRef, ClassId, ImmutableArray, ImmutableHash, ObjectId, RuntimeHeap, Value, reachable_from,
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

#[test]
fn context_fields_expose_description_without_phase_authority() {
    let phase = DecoratorPhase::new(DecoratorKind::Property, DecoratorReason::Open);
    let record = DecoratorValue::Context(phase.context());
    assert_eq!(record.core_name(), "DecoratorContext");
    assert_eq!(
        record.read_field("kind", &types()),
        Some(Value::Symbol("property".into()))
    );
    assert_eq!(
        record.read_field("reason", &types()),
        Some(Value::Symbol("open".into()))
    );
    for name in ["candidate", "phase", "next", "operations", "original_body"] {
        assert_eq!(record.read_field(name, &types()), None);
    }
}

#[test]
fn signature_fields_have_exact_immutable_element_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let types = types();
    let signature = InvocationSignature::new(
        vec![InvocationParameter::new(
            "arg",
            ParameterCategory::Positional,
            types.object.clone(),
        )],
        types.symbol.clone(),
        false,
    )?;
    let record = DecoratorValue::InvocationSignature(signature);
    let Some(Value::ImmutableArray(parameters)) = record.read_field("parameters", &types) else {
        unreachable!("immutable parameter array required")
    };
    assert_eq!(parameters.element_type(), &types.invocation_parameter);
    let Value::Decorator(parameter) = &parameters.elements()[0] else {
        unreachable!("typed parameter required")
    };
    assert_eq!(
        parameter.read_field("type", &types),
        Some(types.object.clone())
    );
    assert_eq!(record.read_field("result", &types), Some(types.symbol));
    Ok(())
}

#[test]
fn immutable_changes_copy_structure_but_keep_nested_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let types = types();
    let nested = ArrayRef::new(vec![Value::Nil]);
    let hash = ImmutableHash::new(
        vec![(Value::Symbol("arg".into()), Value::Array(nested.clone()))],
        types.symbol.clone(),
        types.object.clone(),
    );
    let rest = ImmutableArray::new(vec![Value::Array(nested.clone())], types.object);
    let changes = ArgumentChanges::parse(&[
        ("positional".into(), Value::ImmutableHash(hash)),
        ("rest".into(), Value::ImmutableArray(rest)),
    ])?;
    nested.mutate(|elements| elements.push(Value::Bool(true)));
    let Some([Value::Array(held)]) = changes.rest() else {
        unreachable!("nested identity required")
    };
    assert!(held.same(&nested));
    assert_eq!(changes.positional().map(<[_]>::len), Some(1));
    assert_eq!(held.len(), 2);
    Ok(())
}

#[test]
fn boxed_records_and_immutable_metadata_are_gc_roots() -> Result<(), Box<dyn std::error::Error>> {
    let next = ObjectId::new(11);
    let mut scope = NextScope::synchronous(next, ActivationId::new(ObjectId::new(12)), None);
    scope.finish_owner();
    let Err(error) = scope.begin_attempt(None, None) else {
        unreachable!("finished owner rejects next")
    };
    let root = Value::ImmutableHash(ImmutableHash::new(
        vec![(
            Value::Object(ObjectId::new(13)),
            Value::Decorator(Box::new(DecoratorValue::ProtocolError(error))),
        )],
        Value::Object(ObjectId::new(14)),
        Value::ImmutableArray(ImmutableArray::new(
            vec![Value::Object(ObjectId::new(15))],
            Value::Object(ObjectId::new(16)),
        )),
    ));
    let reached = reachable_from(&RuntimeHeap::new(), &Default::default(), [&root]);
    assert_eq!(
        reached.ids(),
        vec![
            next,
            ObjectId::new(13),
            ObjectId::new(14),
            ObjectId::new(15),
            ObjectId::new(16)
        ]
    );
    Ok(())
}

#[test]
fn decorator_values_remain_send() {
    let record = Value::Decorator(Box::new(DecoratorValue::ArgumentChanges(
        ArgumentChanges::empty(),
    )));
    let Ok(returned) = std::thread::spawn(move || record).join() else {
        unreachable!("value transfer does not panic")
    };
    assert!(matches!(returned, Value::Decorator(_)));
}

#[test]
fn kernel_registration_classifies_exact_record_tags_without_changing_legacy_startup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut registry = iris_runtime::ClassRegistry::new();
    let mut kernel = iris_runtime::Kernel::new(&mut registry)?;
    let record: Value = DecoratorValue::ArgumentChanges(ArgumentChanges::empty()).into();
    assert_eq!(
        kernel.class_of(&record),
        Err(iris_runtime::KernelError::Type)
    );
    kernel.register_decorator_classes(&mut registry)?;
    assert_eq!(
        Some(kernel.class_of(&record)?),
        kernel.core_class("ArgumentChanges")
    );
    assert_ne!(
        kernel.core_class("Invocation"),
        kernel.core_class("InvocationSignature")
    );
    let immutable = Value::ImmutableArray(ImmutableArray::new(vec![], types().object));
    assert_eq!(
        kernel.class_of(&immutable)?,
        kernel.class_of(&Value::Array(ArrayRef::new(vec![])))?
    );
    assert_eq!(
        kernel.send(&registry, record, iris_runtime::NativeSelector::ToBool, &[])?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn immutable_array_metadata_uses_exact_value_equality() {
    let first = ImmutableArray::new(vec![], Value::Type(ClassId::new(1), vec![]));
    let second = ImmutableArray::new(vec![], Value::Type(ClassId::new(2), vec![]));
    assert_ne!(first, second);
    assert_eq!(first, first.clone());
    assert_eq!(
        iris_runtime::public_hash(&Value::ImmutableArray(first)),
        Err(iris_runtime::StableHashError::UnsupportedValue)
    );
}

#[test]
fn argument_changes_expose_no_unlisted_presence_or_payload_fields() {
    let record = DecoratorValue::ArgumentChanges(ArgumentChanges::empty());
    for name in [
        "positional",
        "keywords",
        "rest",
        "keyword_rest",
        "block",
        "present",
    ] {
        assert_eq!(record.read_field(name, &types()), None);
    }
}
