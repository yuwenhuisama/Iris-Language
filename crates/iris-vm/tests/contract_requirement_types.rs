use iris_runtime::{ComposedType, TypeAtom, Value};
use iris_vm::{compile, run};

#[test]
fn type_metadata_is_returned_when_annotation_is_explicit() -> Result<(), iris_vm::CompileError> {
    let given = "contract Numbers { fun iterator() -> Iterator<Integer> } \
        Reflection::Contract.requirement(Numbers, :iterator)[:return_type]";
    let when = run(&compile(given)?);
    assert!(
        matches!(when, Ok(Value::ComposedType(ComposedType::Intersection(ref atoms)))
        if matches!(atoms.as_slice(), [TypeAtom::Contract(_, arguments)] if arguments.len() == 1)),
        "{when:?}"
    );
    Ok(())
}

#[test]
fn canonical_identity_matches_when_explicit_and_substituted() -> Result<(), iris_vm::CompileError> {
    let given = "contract Numbers { fun iterator() -> Iterator<Integer> } \
        Reflection::Contract.requirement(Numbers, :iterator)[:return_type] same? \
        Reflection::Contract.requirement(Iterable<Integer>, :iterator)[:return_type]";
    let when = run(&compile(given)?);
    assert_eq!(when, Ok(Value::Bool(true)));
    Ok(())
}

#[test]
fn canonical_identity_repeats_when_reflected_again() -> Result<(), iris_vm::CompileError> {
    let given = "contract Numbers { fun iterator() -> Iterator<Integer> } \
        let first = Reflection::Contract.requirement(Numbers, :iterator)[:return_type]; \
        let second = Reflection::Contract.requirement(Numbers, :iterator)[:return_type]; \
        (first same? second) && first == second";
    let when = run(&compile(given)?);
    assert_eq!(when, Ok(Value::Bool(true)));
    Ok(())
}

#[test]
fn canonical_identity_differs_when_arguments_differ() -> Result<(), iris_vm::CompileError> {
    let given = "contract Values { fun integers() -> Iterator<Integer>; fun strings() -> Iterator<String> } \
        let integers = Reflection::Contract.requirement(Values, :integers)[:return_type]; \
        let strings = Reflection::Contract.requirement(Values, :strings)[:return_type]; \
        !(integers same? strings) && integers != strings && \
        (strings same? Reflection::Contract.requirement(Iterable<String>, :iterator)[:return_type])";
    let when = run(&compile(given)?);
    assert_eq!(when, Ok(Value::Bool(true)));
    Ok(())
}

#[test]
fn contract_object_is_distinct_when_annotation_is_reflected() -> Result<(), iris_vm::CompileError> {
    let given = "contract Numbers { fun iterator() -> Iterator<Integer> } \
        %[Reflection::Contract.requirement(Numbers, :iterator)[:return_type], Iterator<Integer>]";
    let when = run(&compile(given)?);
    assert!(
        matches!(when, Ok(Value::Array(ref values))
        if matches!(values.elements().as_slice(),
            [Value::ComposedType(ComposedType::Intersection(atoms)), Value::Contract(contract, arguments)]
            if matches!(atoms.as_slice(), [TypeAtom::Contract(identity, parameters)]
                if identity == contract && parameters == arguments))),
        "{when:?}"
    );
    Ok(())
}

#[test]
fn contract_object_is_preserved_when_evaluated_directly() -> Result<(), iris_vm::CompileError> {
    let given = "Iterator<Integer>";
    let when = run(&compile(given)?);
    assert!(
        matches!(when, Ok(Value::Contract(_, ref arguments)) if arguments.len() == 1),
        "{when:?}"
    );
    Ok(())
}

#[test]
fn nil_is_returned_when_return_annotation_is_absent() -> Result<(), iris_vm::CompileError> {
    let given = "contract Unannotated { fun value() } \
        Reflection::Contract.requirement(Unannotated, :value)[:return_type]";
    let when = run(&compile(given)?);
    assert_eq!(when, Ok(Value::Nil));
    Ok(())
}

#[test]
fn nil_is_returned_when_requirement_is_absent() -> Result<(), iris_vm::CompileError> {
    let given = "contract Numbers { fun iterator() -> Iterator<Integer> } \
        Reflection::Contract.requirement(Numbers, :absent)";
    let when = run(&compile(given)?);
    assert_eq!(when, Ok(Value::Nil));
    Ok(())
}
