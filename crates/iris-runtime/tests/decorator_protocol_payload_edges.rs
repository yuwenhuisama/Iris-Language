use iris_runtime::decorator_protocol::*;
use iris_runtime::{ArrayRef, HashRef, ObjectId, Value};

fn signature(parameters: Vec<InvocationParameter>) -> Result<InvocationSignature, ArgumentError> {
    InvocationSignature::new(parameters, Value::Nil, false)
}

fn rest_signature() -> Result<InvocationSignature, ArgumentError> {
    signature(vec![
        InvocationParameter::new("items", ParameterCategory::Rest, Value::Nil),
        InvocationParameter::new("extras", ParameterCategory::KeywordRest, Value::Nil),
    ])
}

#[test]
fn snapshots_isolate_all_input_maps_but_keep_argument_identity() -> Result<(), ArgumentError> {
    let nested = ArrayRef::new(vec![]);
    let map = HashRef::new(vec![(
        Value::Symbol("x".into()),
        Value::Array(nested.clone()),
    )]);
    let changes = ArgumentChanges::parse(&[
        ("positional".into(), Value::Hash(map.clone())),
        ("keywords".into(), Value::Hash(map.clone())),
        ("keyword_rest".into(), Value::Hash(map.clone())),
    ])?;
    map.clear();
    nested.mutate(|values| values.push(Value::Bool(true)));
    for entries in [
        changes.positional(),
        changes.keywords(),
        changes.keyword_rest(),
    ] {
        let Some([(name, Value::Array(held))]) = entries else {
            unreachable!()
        };
        assert_eq!(name, "x");
        assert!(held.same(&nested));
        assert_eq!(held.elements(), vec![Value::Bool(true)]);
    }
    Ok(())
}

#[test]
fn each_execution_gets_fresh_rest_containers_in_encounter_order() -> Result<(), ArgumentError> {
    let signature = rest_signature()?;
    let source_array = ArrayRef::new(vec![Value::Bool(true)]);
    let source_hash = HashRef::new(vec![
        (Value::Symbol("z".into()), Value::Bool(false)),
        (Value::Symbol("a".into()), Value::Bool(true)),
    ]);
    let payload = InvocationPayload::from_bindings(
        &signature,
        &[
            Value::Array(source_array.clone()),
            Value::Hash(source_hash.clone()),
        ],
        |_, _| true,
    )?;
    let first = payload.bindings(&signature)?;
    let second = payload.bindings(&signature)?;
    let (
        [Value::Array(first_array), Value::Hash(first_hash)],
        [Value::Array(second_array), Value::Hash(second_hash)],
    ) = (first.as_slice(), second.as_slice())
    else {
        unreachable!()
    };
    assert!(!first_array.same(second_array));
    assert!(!first_array.same(&source_array));
    assert!(!first_hash.same(second_hash));
    assert!(!first_hash.same(&source_hash));
    first_array.mutate(Vec::clear);
    first_hash.clear();
    assert_eq!(second_array.elements(), vec![Value::Bool(true)]);
    assert_eq!(second_hash.entries(), source_hash.entries());
    Ok(())
}

#[test]
fn empty_replacements_still_require_declared_channels() -> Result<(), ArgumentError> {
    let signature = signature(vec![])?;
    let payload = InvocationPayload::from_bindings(&signature, &[], |_, _| true)?;
    for (field, value, category) in [
        (
            "rest",
            Value::Array(ArrayRef::new(vec![])),
            ParameterCategory::Rest,
        ),
        (
            "keyword_rest",
            Value::Hash(HashRef::new(vec![])),
            ParameterCategory::KeywordRest,
        ),
        ("block", Value::Nil, ParameterCategory::Block),
    ] {
        let changes = ArgumentChanges::parse(&[(field.into(), value)])?;
        assert_eq!(
            payload.patch(&signature, &changes, |_, _| true),
            Err(ArgumentError::MissingChannel(category))
        );
    }
    Ok(())
}

#[test]
fn block_nil_is_optional_only_and_replacement_uses_exact_predicate() -> Result<(), ArgumentError> {
    let exact_type = Value::Symbol("exact Block<(Integer) -> String>".into());
    let parameter = InvocationParameter::new("body", ParameterCategory::Block, exact_type.clone());
    let required = signature(vec![parameter.clone()])?;
    let optional = signature(vec![parameter.with_optional(true)])?;
    let exact_block = Value::Closure(ObjectId::new(10));
    let accepts = |ty: &Value, value: &Value| ty == &exact_type && value == &exact_block;
    let payload = InvocationPayload::from_bindings(&optional, &[Value::Nil], accepts)?;
    let changes = ArgumentChanges::parse(&[("block".into(), exact_block.clone())])?;
    let patched = payload.patch(&optional, &changes, accepts)?;
    assert_eq!(patched.block(), &exact_block);
    assert_eq!(patched.replaced(), &["body"]);
    assert!(matches!(
        InvocationPayload::from_bindings(&required, &[Value::Nil], |_, _| true),
        Err(ArgumentError::TypeMismatch(_))
    ));
    let wrong = ArgumentChanges::parse(&[("block".into(), Value::Closure(ObjectId::new(11)))])?;
    assert!(matches!(
        payload.patch(&optional, &wrong, accepts),
        Err(ArgumentError::TypeMismatch(_))
    ));
    Ok(())
}

#[test]
fn source_block_absence_is_distinct_from_explicit_nil() {
    let omitted = SourceCall::new(vec![], vec![], None);
    let explicit = SourceCall::new(vec![], vec![], Some(Value::Nil));
    assert!(omitted.block_omitted());
    assert!(!explicit.block_omitted());
    assert_eq!(omitted.block(), explicit.block());
}

#[test]
fn nested_patches_accumulate_unique_names_in_declaration_order() -> Result<(), ArgumentError> {
    let signature = rest_signature()?;
    let payload = InvocationPayload::from_bindings(
        &signature,
        &[
            Value::Array(ArrayRef::new(vec![])),
            Value::Hash(HashRef::new(vec![])),
        ],
        |_, _| true,
    )?;
    let first =
        ArgumentChanges::parse(&[("keyword_rest".into(), Value::Hash(HashRef::new(vec![])))])?;
    let inner = payload.patch(&signature, &first, |_, _| true)?;
    let second = ArgumentChanges::parse(&[
        ("rest".into(), Value::Array(ArrayRef::new(vec![]))),
        ("keyword_rest".into(), Value::Hash(HashRef::new(vec![]))),
    ])?;
    let inner = inner.patch(&signature, &second, |_, _| true)?;
    assert_eq!(inner.replaced(), &["items", "extras"]);
    assert!(payload.replaced().is_empty());
    Ok(())
}

#[test]
fn signature_rejects_bad_order_repeated_channels_and_optional_rest() {
    let parameter = |name, category| InvocationParameter::new(name, category, Value::Nil);
    for parameters in [
        vec![
            parameter("key", ParameterCategory::Keyword),
            parameter("arg", ParameterCategory::Positional),
        ],
        vec![
            parameter("first", ParameterCategory::Rest),
            parameter("second", ParameterCategory::Rest),
        ],
        vec![
            parameter("first", ParameterCategory::Block),
            parameter("second", ParameterCategory::Block),
        ],
    ] {
        assert!(matches!(
            signature(parameters),
            Err(ArgumentError::InvalidParameterOrder(_))
        ));
    }
    assert!(matches!(
        signature(vec![
            parameter("items", ParameterCategory::Rest).with_optional(true)
        ]),
        Err(ArgumentError::InvalidOptionality(_))
    ));
    assert!(matches!(
        signature(vec![
            parameter("x", ParameterCategory::Positional),
            parameter("x", ParameterCategory::Keyword)
        ]),
        Err(ArgumentError::DuplicateName(_))
    ));
}
