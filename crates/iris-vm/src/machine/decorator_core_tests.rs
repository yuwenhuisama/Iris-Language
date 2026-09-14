#![expect(clippy::expect_used, reason = "tests construct typed runtime fixtures")]

use super::{Machine, MachineError};
use iris_runtime::decorator_protocol::{
    DecoratorKind, DecoratorPhase, DecoratorReason, InvocationParameter, ParameterCategory,
};
use iris_runtime::{DecoratorValue, ImmutableArray, ImmutableHash, Value};

fn probe(body: &str, input: impl FnOnce(&Machine) -> Value) -> Result<Value, MachineError> {
    let source =
        format!("module Probe {{ public module fun read(value) {{ {body} }} }} Probe.read(nil)");
    let program = crate::compile(&source).expect("probe compiles");
    let mut machine = Machine::new().expect("machine");
    let classes = machine.register_classes(&program)?;
    machine.register_core_records()?;
    let value = input(&machine);
    machine.invoke_function(0, vec![value], &program, &classes)
}

#[test]
fn typed_record_fields_are_read_by_compiled_source() {
    let outcome = probe(
        "%[value.kind, value.reason, value is? DecoratorContext]",
        |_| {
            DecoratorValue::Context(
                DecoratorPhase::new(DecoratorKind::Method, DecoratorReason::Origin).context(),
            )
            .into()
        },
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("method".into()),
            Value::Symbol("origin".into()),
            Value::Bool(true)
        ])))
    );
}

#[test]
fn parameter_fields_retain_exact_type_and_readonly_behavior() {
    let outcome = probe(
        "let error = try { value.name = :other } catch error { error }; %[value.name, value.optional, error]",
        |machine| {
            DecoratorValue::InvocationParameter(InvocationParameter::new(
                "item",
                ParameterCategory::Positional,
                Value::Type(
                    machine.builtin_class("Integer").expect("integer"),
                    Vec::new(),
                ),
            ))
            .into()
        },
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("item".into()),
            Value::Bool(false),
            Value::Symbol("ReadonlyMutationError".into())
        ])))
    );
}

fn array(machine: &Machine) -> Value {
    Value::ImmutableArray(ImmutableArray::new(
        vec![Value::Integer(7_u64.into())],
        Value::Type(
            machine.builtin_class("Integer").expect("integer"),
            Vec::new(),
        ),
    ))
}

fn hash(machine: &Machine) -> Value {
    Value::ImmutableHash(ImmutableHash::new(
        vec![(Value::Symbol("item".into()), Value::Integer(7_u64.into()))],
        Value::Type(machine.builtin_class("Symbol").expect("symbol"), Vec::new()),
        Value::Type(
            machine.builtin_class("Integer").expect("integer"),
            Vec::new(),
        ),
    ))
}

#[test]
fn immutable_array_supports_reads_and_invariant_annotations() {
    let outcome = probe(
        "let typed: Array<Integer> = value; %[typed.length, typed[0], typed.fetch(0), typed is? Array<Integer>, typed is? Array<Object>]",
        array,
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Bool(true),
            Value::Bool(false)
        ])))
    );
}

#[test]
fn immutable_hash_supports_reads_and_invariant_annotations() {
    let outcome = probe(
        "let typed: Hash<Symbol, Integer> = value; %[typed.length, typed[:item], typed.fetch(:item), typed is? Hash<Symbol, Integer>, typed is? Hash<Symbol, Object>]",
        hash,
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Bool(true),
            Value::Bool(false)
        ])))
    );
}

#[test]
fn immutable_containers_reject_writes_through_index_and_methods() {
    for body in [
        "value[0] = 9",
        "value.append(9)",
        "value.clear()",
        "value.reverse!()",
    ] {
        assert_eq!(
            probe(body, array),
            Err(MachineError::ReadonlyMutation),
            "{body}"
        );
    }
    for body in [
        "value[:item] = 9",
        "value.delete(:item)",
        "value.clear()",
        "value.rehash()",
    ] {
        assert_eq!(
            probe(body, hash),
            Err(MachineError::ReadonlyMutation),
            "{body}"
        );
    }
}

#[test]
fn immutable_containers_iterate_through_source_protocol() {
    assert_eq!(
        probe(
            "let iterator = value.iterator(); iterator.next().value",
            array
        ),
        Ok(Value::Integer(7_u64.into()))
    );
    assert_eq!(
        probe(
            "let iterator = value.iterator(); iterator.next().value",
            hash
        ),
        Ok(Value::Tuple(vec![
            Value::Symbol("item".into()),
            Value::Integer(7_u64.into())
        ]))
    );
}

#[test]
fn immutable_containers_iterate_in_for_loops() {
    assert_eq!(
        probe(
            "mut total = 0; for item in value { total = total + item }; total",
            array
        ),
        Ok(Value::Integer(7_u64.into()))
    );
}

#[test]
fn immutable_annotations_reject_unknown_types_and_invariant_widening() {
    assert!(probe("let invalid: Array<Object> = value; invalid", array).is_err());
    assert!(probe("let invalid: Missing = value; invalid", array).is_err());
}

#[test]
fn immutable_collection_copy_operations_do_not_mutate_the_source() {
    assert_eq!(
        probe(
            "let copied = value.reverse(); %[copied[0], value[0]]",
            array
        ),
        Ok(Value::Array(iris_runtime::ArrayRef::new(
            vec![Value::Integer(7_u64.into()); 2]
        )))
    );
}

#[test]
fn immutable_each_calls_source_closures() {
    assert_eq!(
        probe(
            "mut total = 0; let walked = value.each({ |item| total = total + item }); total",
            array
        ),
        Ok(Value::Integer(7_u64.into()))
    );
    assert_eq!(
        probe(
            "mut total = 0; let walked = value.each({ |entry_key, item| total = total + item }); total",
            hash
        ),
        Ok(Value::Integer(7_u64.into()))
    );
}

#[test]
fn retained_transformation_does_not_authorize_fluent_operations() {
    let result = probe(
        "try { value.wrap_method(nil) } catch error { %[error.category, error.owner] }",
        |_| {
            let phase = DecoratorPhase::new(DecoratorKind::Method, DecoratorReason::Origin);
            DecoratorValue::Transformation(
                iris_runtime::decorator_protocol::Transformation::empty(Some(&phase))
                    .expect("phase"),
            )
            .into()
        },
    );
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("outside_phase".into()),
            Value::Nil
        ])))
    );
}

#[test]
fn invocation_fields_build_typed_immutable_snapshots() {
    use iris_runtime::decorator_protocol::{
        Invocation, InvocationPayload, InvocationSignature, InvocationSlot, SelectedCall, SlotKind,
        SourceCall,
    };
    let outcome = probe(
        "%[value.positional.fetch(:item), value.signature.parameters[0].name, value.signature.parameters is? Array<InvocationParameter>, value.original_positional is? Array<Object>, value.original_keywords is? Array<Tuple<Symbol, Object>>]",
        |machine| {
            let integer = Value::Type(
                machine.builtin_class("Integer").expect("integer"),
                Vec::new(),
            );
            let signature = InvocationSignature::new(
                vec![InvocationParameter::new(
                    "item",
                    ParameterCategory::Positional,
                    integer.clone(),
                )],
                integer,
                false,
            )
            .expect("signature");
            let arguments = vec![Value::Integer(7_u64.into())];
            let payload = InvocationPayload::from_bindings(&signature, &arguments, |_, _| true)
                .expect("payload");
            let selected = SelectedCall::new(
                Value::Nil,
                InvocationSlot::new(
                    Value::Class(machine.builtin_class("Object").expect("object")),
                    "read",
                    SlotKind::Method,
                ),
                signature,
            );
            DecoratorValue::Invocation(Box::new(Invocation::new(
                selected,
                SourceCall::new(arguments, Vec::new(), None),
                payload,
            )))
            .into()
        },
    );
    assert_eq!(
        outcome,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Symbol("item".into()),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true)
        ])))
    );
}
