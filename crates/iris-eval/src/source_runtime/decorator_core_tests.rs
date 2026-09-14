use super::{Binding, EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{
    DecoratorKind, DecoratorPhase, DecoratorReason, DecoratorValue, Invocation,
    InvocationParameter, InvocationPayload, InvocationSignature, InvocationSlot, ParameterCategory,
    SelectedCall, SlotKind, SourceCall, Transformation,
};
use iris_runtime::{ArrayRef, ImmutableArray, ImmutableHash, Value};

fn evaluator() -> Result<SourceEvaluator, EvaluationError> {
    SourceEvaluator::new_in_package("decorator.core.tests")
}

fn run(evaluator: &mut SourceEvaluator, source: &str) -> Result<Value, EvaluationError> {
    let parsed = iris_parser::parse(source);
    assert!(
        parsed.program_accepted,
        "{source}: {:?}",
        parsed.diagnostics
    );
    evaluator.set_source(source);
    evaluator.program(&parsed.program)
}

fn bind(evaluator: &mut SourceEvaluator, name: &str, value: Value) {
    evaluator
        .names
        .insert(name.into(), Binding::immutable(value));
}

#[test]
fn readonly_context_fields_keep_their_core_type() -> Result<(), EvaluationError> {
    let mut evaluator = evaluator()?;
    let phase = DecoratorPhase::new(DecoratorKind::Method, DecoratorReason::Open);
    bind(
        &mut evaluator,
        "context",
        DecoratorValue::Context(phase.context()).into(),
    );
    assert_eq!(
        run(
            &mut evaluator,
            "%[context is? DecoratorContext, context.kind, context.reason]"
        )?,
        Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Symbol("method".into()),
            Value::Symbol("open".into())
        ]))
    );
    assert_eq!(
        run(&mut evaluator, "context.kind = :class"),
        Err(EvaluationError::ReadonlyMutation)
    );
    assert_eq!(
        run(
            &mut evaluator,
            "try { context as Invocation } catch error { error is? TypeError }"
        )?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn retained_transformation_does_not_authorize_construction() -> Result<(), EvaluationError> {
    let mut evaluator = evaluator()?;
    let phase = DecoratorPhase::new(DecoratorKind::Property, DecoratorReason::Origin);
    let record =
        Transformation::empty(Some(&phase)).map_err(super::decorator_errors::phase_error)?;
    bind(
        &mut evaluator,
        "saved",
        DecoratorValue::Transformation(record).into(),
    );
    assert_eq!(
        run(&mut evaluator, "saved.kind")?,
        Value::Symbol("property".into())
    );
    assert!(
        matches!(run(&mut evaluator, "saved.wrap_getter(nil)"), Err(EvaluationError::Raised(Value::Decorator(error))) if matches!(*error, DecoratorValue::ProtocolError(_)))
    );
    Ok(())
}

#[test]
fn invocation_fields_materialize_typed_immutable_containers() -> Result<(), EvaluationError> {
    let mut evaluator = evaluator()?;
    let object = evaluator
        .snapshot_types
        .as_ref()
        .ok_or(EvaluationError::UnsupportedConstruct)?
        .object
        .clone();
    let signature = InvocationSignature::new(
        vec![InvocationParameter::new(
            "value",
            ParameterCategory::Positional,
            object.clone(),
        )],
        object.clone(),
        false,
    )
    .map_err(super::decorator_errors::argument_error)?;
    let payload =
        InvocationPayload::from_bindings(&signature, &[Value::Integer(7u64.into())], |_, _| true)
            .map_err(super::decorator_errors::argument_error)?;
    let selected = SelectedCall::new(
        Value::Nil,
        InvocationSlot::new(object, "value", SlotKind::Method),
        signature,
    );
    let invocation = Invocation::new(
        selected,
        SourceCall::new(vec![Value::Integer(7u64.into())], Vec::new(), None),
        payload,
    );
    bind(
        &mut evaluator,
        "invocation",
        DecoratorValue::Invocation(Box::new(invocation)).into(),
    );
    assert_eq!(
        run(
            &mut evaluator,
            "%[invocation.signature is? InvocationSignature, invocation.signature.parameters[0] is? InvocationParameter, invocation.positional[:value], invocation.original_positional[-1], invocation.block_omitted]"
        )?,
        Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Integer(7u64.into()),
            Value::Integer(7u64.into()),
            Value::Bool(true)
        ]))
    );
    assert_eq!(
        run(
            &mut evaluator,
            "let parameters: Array<InvocationParameter> = invocation.signature.parameters; parameters is? Array<InvocationParameter>"
        )?,
        Value::Bool(true)
    );
    assert_eq!(
        run(
            &mut evaluator,
            "invocation.positional is? Hash<Symbol, Object>"
        )?,
        Value::Bool(true)
    );
    assert_eq!(
        run(
            &mut evaluator,
            "invocation.positional is? Hash<Symbol, Integer>"
        )?,
        Value::Bool(false)
    );
    assert_eq!(
        run(
            &mut evaluator,
            "try { let wrong: Array<Object> = invocation.signature.parameters; false } catch error { error is? TypeError }"
        )?,
        Value::Bool(true)
    );
    for source in [
        "invocation.positional[:value] = 8",
        "invocation.signature.parameters[0].name = :bad",
        "invocation.omitted.append(:bad)",
    ] {
        assert_eq!(
            run(&mut evaluator, source),
            Err(EvaluationError::ReadonlyMutation),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn immutable_collections_support_reads_and_iteration_without_mutation()
-> Result<(), EvaluationError> {
    let mut evaluator = evaluator()?;
    let types = evaluator
        .snapshot_types
        .as_ref()
        .ok_or(EvaluationError::UnsupportedConstruct)?
        .clone();
    bind(
        &mut evaluator,
        "array",
        Value::ImmutableArray(ImmutableArray::new(
            vec![Value::Integer(3u64.into()), Value::Integer(4u64.into())],
            types.object.clone(),
        )),
    );
    bind(
        &mut evaluator,
        "hash",
        Value::ImmutableHash(ImmutableHash::new(
            vec![(Value::Symbol("first".into()), Value::Integer(3u64.into()))],
            types.symbol,
            types.object,
        )),
    );
    assert_eq!(
        run(
            &mut evaluator,
            "%[array.length, array[-1], array[99], hash.fetch(:first), hash[:absent], hash.length]"
        )?,
        Value::Array(ArrayRef::new(vec![
            Value::Integer(2u64.into()),
            Value::Integer(4u64.into()),
            Value::Nil,
            Value::Integer(3u64.into()),
            Value::Nil,
            Value::Integer(1u64.into())
        ]))
    );
    assert_eq!(
        run(&mut evaluator, "hash.fetch(:absent)"),
        Err(EvaluationError::KeyError)
    );
    assert_eq!(
        run(
            &mut evaluator,
            "module Walk { public fun sum(values) -> Integer { mut total = 0; for value in values { total = total + value }; total } }; Walk.sum(array)"
        )?,
        Value::Integer(7u64.into())
    );
    assert_eq!(
        run(
            &mut evaluator,
            "let cursor = hash.iterator(); cursor.next().value"
        )?,
        Value::Tuple(vec![
            Value::Symbol("first".into()),
            Value::Integer(3u64.into())
        ])
    );
    for source in [
        "array[0] = 9",
        "array.append(9)",
        "hash[:first] = 9",
        "hash.clear()",
        "hash.rehash()",
    ] {
        assert_eq!(
            run(&mut evaluator, source),
            Err(EvaluationError::ReadonlyMutation),
            "{source}"
        );
    }
    Ok(())
}
