use iris_runtime::decorator_protocol::*;
use iris_runtime::{ArrayRef, HashRef, Value};

fn hash(entries: &[(&str, Value)]) -> Value {
    Value::Hash(HashRef::new(
        entries
            .iter()
            .map(|(name, value)| (Value::Symbol((*name).into()), value.clone()))
            .collect(),
    ))
}

#[test]
fn changes_distinguish_absence_and_nil_and_snapshot_structure() -> Result<(), ArgumentError> {
    let rest = ArrayRef::new(vec![Value::Bool(true)]);
    let changes = ArgumentChanges::parse(&[
        ("rest".into(), Value::Array(rest.clone())),
        ("block".into(), Value::Nil),
    ])?;
    rest.mutate(Vec::clear);
    assert_eq!(changes.rest(), Some([Value::Bool(true)].as_slice()));
    assert_eq!(changes.block(), Some(&Value::Nil));
    assert_eq!(ArgumentChanges::empty().block(), None);
    Ok(())
}

#[test]
fn changes_reject_wrong_shapes_unknown_and_duplicate_keywords() {
    for field in ["positional", "keywords", "rest", "keyword_rest"] {
        assert!(matches!(
            ArgumentChanges::parse(&[(field.into(), Value::Nil)]),
            Err(ArgumentError::WrongShape(_))
        ));
    }
    assert!(matches!(
        ArgumentChanges::parse(&[("wat".into(), Value::Nil)]),
        Err(ArgumentError::UnknownKeyword(_))
    ));
    assert!(matches!(
        ArgumentChanges::parse(&[("block".into(), Value::Nil), ("block".into(), Value::Nil)]),
        Err(ArgumentError::DuplicateKeyword(_))
    ));
    assert!(matches!(
        ArgumentChanges::parse(&[(
            "keywords".into(),
            Value::Hash(HashRef::new(vec![(Value::Text("x".into()), Value::Nil)]))
        )]),
        Err(ArgumentError::NonSymbolKey(_))
    ));
}

fn signature() -> Result<InvocationSignature, ArgumentError> {
    InvocationSignature::new(
        vec![
            InvocationParameter::new(
                "first",
                ParameterCategory::Positional,
                Value::Text("bool".into()),
            ),
            InvocationParameter::new("items", ParameterCategory::Rest, Value::Text("bool".into())),
            InvocationParameter::new(
                "flag",
                ParameterCategory::Keyword,
                Value::Text("bool".into()),
            )
            .with_optional(true),
            InvocationParameter::new(
                "extras",
                ParameterCategory::KeywordRest,
                Value::Text("bool".into()),
            ),
            InvocationParameter::new(
                "body",
                ParameterCategory::Block,
                Value::Text("block".into()),
            )
            .with_optional(true),
        ],
        Value::Text("bool".into()),
        false,
    )
}

fn accepts(ty: &Value, value: &Value) -> bool {
    match ty {
        Value::Text(name) if name == "bool" => matches!(value, Value::Bool(_)),
        Value::Text(name) if name == "block" => matches!(value, Value::Closure(_)),
        _ => false,
    }
}

#[test]
fn patches_bind_names_keep_provenance_and_retry_from_incoming() -> Result<(), ArgumentError> {
    let signature = signature()?;
    let payload = InvocationPayload::from_bindings(
        &signature,
        &[
            Value::Bool(true),
            Value::Array(ArrayRef::new(vec![])),
            Value::Bool(false),
            hash(&[]),
            Value::Nil,
        ],
        accepts,
    )?;
    let source = SourceCall::new(vec![Value::Bool(true)], vec![], None)
        .with_provenance(vec!["flag".into()], vec!["flag".into()]);
    let selected = SelectedCall::new(
        Value::Nil,
        InvocationSlot::new(Value::Nil, "run", SlotKind::Method),
        signature,
    );
    let invocation = Invocation::new(selected, source, payload);
    let patch = ArgumentChanges::parse(&[
        ("keywords".into(), hash(&[("flag", Value::Bool(true))])),
        ("positional".into(), hash(&[("first", Value::Bool(false))])),
    ])?;
    let inner = invocation.patch(&patch, accepts)?;
    assert_eq!(inner.payload().replaced(), &["first", "flag"]);
    assert_eq!(inner.source(), invocation.source());
    assert_eq!(invocation.payload().positional()[0].1, Value::Bool(true));
    let retry = invocation.patch(&ArgumentChanges::empty(), accepts)?;
    assert!(retry.payload().replaced().is_empty());
    assert_eq!(retry.payload().keywords()[0].1, Value::Bool(false));
    Ok(())
}

#[test]
fn patches_validate_all_values_and_reject_channel_smuggling() -> Result<(), ArgumentError> {
    let signature = signature()?;
    let payload = InvocationPayload::from_bindings(
        &signature,
        &[
            Value::Bool(true),
            Value::Array(ArrayRef::new(vec![])),
            Value::Bool(false),
            hash(&[]),
            Value::Nil,
        ],
        accepts,
    )?;
    for changes in [
        vec![("positional".into(), hash(&[("flag", Value::Bool(true))]))],
        vec![("keyword_rest".into(), hash(&[("first", Value::Bool(true))]))],
    ] {
        assert!(matches!(
            payload.patch(&signature, &ArgumentChanges::parse(&changes)?, accepts),
            Err(ArgumentError::WrongChannel { .. })
        ));
    }
    let changes =
        ArgumentChanges::parse(&[("rest".into(), Value::Array(ArrayRef::new(vec![Value::Nil])))])?;
    assert!(matches!(
        payload.patch(&signature, &changes, accepts),
        Err(ArgumentError::TypeMismatch(_))
    ));
    assert!(matches!(
        payload.patch(&signature, &ArgumentChanges::empty(), |_, _| false),
        Err(ArgumentError::TypeMismatch(_))
    ));
    Ok(())
}
