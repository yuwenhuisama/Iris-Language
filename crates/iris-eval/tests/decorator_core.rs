use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ArrayRef, DecoratorValue, Value};

#[test]
fn changes_are_available_as_core_instances() -> Result<(), EvaluationError> {
    for source in [
        "ArgumentChanges.empty is? ArgumentChanges",
        "ArgumentChanges.new() is? ArgumentChanges",
        "let value: ArgumentChanges = ArgumentChanges.new(); value is? ArgumentChanges",
        "Kernel::ArgumentChanges.new() is? ArgumentChanges",
    ] {
        assert_eq!(evaluate(source)?, Value::Bool(true), "{source}");
    }
    assert_eq!(
        evaluate("ArgumentChanges is? ArgumentChanges")?,
        Value::Bool(false)
    );
    assert_eq!(
        evaluate("ArgumentChanges.empty is? Invocation")?,
        Value::Bool(false)
    );
    assert_eq!(
        evaluate(
            "try { ArgumentChanges.empty as Invocation } catch error { error is? TypeError }"
        )?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn outside_phase_raises_the_actual_core_error() -> Result<(), EvaluationError> {
    for operation in [
        "Plan.empty",
        "Transformation.empty",
        "Transformation.wrap_method(nil)",
        "Transformation.wrap_getter(nil)",
        "Transformation.wrap_setter(nil)",
        "Transformation.add_method(:extra, nil)",
    ] {
        let source = format!(
            "try {{ {operation} }} catch error {{ %[error is? DecoratorProtocolError, error.category, error.owner] }}"
        );
        assert_eq!(
            evaluate(&source)?,
            Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Symbol("outside_phase".into()),
                Value::Nil
            ]))
        );
        assert!(
            matches!(evaluate(operation), Err(EvaluationError::Raised(Value::Decorator(record))) if matches!(*record, DecoratorValue::ProtocolError(_)))
        );
    }
    Ok(())
}

#[test]
fn constructor_preserves_absence_and_explicit_nil() -> Result<(), EvaluationError> {
    let absent = evaluate("ArgumentChanges.new()")?;
    let supplied = evaluate("ArgumentChanges.new(block: nil)")?;
    let Value::Decorator(absent) = absent else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let Value::Decorator(supplied) = supplied else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let DecoratorValue::ArgumentChanges(absent) = *absent else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let DecoratorValue::ArgumentChanges(supplied) = *supplied else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    assert_eq!(absent.block(), None);
    assert_eq!(supplied.block(), Some(&Value::Nil));
    Ok(())
}

#[test]
fn constructor_snapshots_all_five_supplied_fields() -> Result<(), EvaluationError> {
    let value = evaluate(
        "ArgumentChanges.new(positional: %{:first: 1}, keywords: %{:label: nil}, rest: %[2], keyword_rest: %{:extra: 3}, block: nil)",
    )?;
    let Value::Decorator(record) = value else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let DecoratorValue::ArgumentChanges(changes) = *record else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    assert_eq!(
        changes.positional(),
        Some(&[("first".into(), Value::Integer(1u64.into()))][..])
    );
    assert_eq!(
        changes.keywords(),
        Some(&[("label".into(), Value::Nil)][..])
    );
    assert_eq!(changes.rest(), Some(&[Value::Integer(2u64.into())][..]));
    assert_eq!(
        changes.keyword_rest(),
        Some(&[("extra".into(), Value::Integer(3u64.into()))][..])
    );
    assert_eq!(changes.block(), Some(&Value::Nil));
    Ok(())
}

#[test]
fn factories_reject_wrong_shapes_before_phase_construction() -> Result<(), EvaluationError> {
    for source in [
        "Transformation.wrap_method()",
        "Transformation.wrap_method(nil, nil)",
        "Transformation.wrap_getter(wrapper: nil)",
        "Transformation.add_method(:extra)",
        "ArgumentChanges.new() { || 1 }",
    ] {
        assert_eq!(
            evaluate(&format!(
                "try {{ {source} }} catch error {{ error is? ArgumentError }}"
            ))?,
            Value::Bool(true),
            "{source}"
        );
    }
    assert_eq!(
        evaluate("try { Transformation.add_method(1, nil) } catch error { error is? TypeError }")?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn constructor_rejects_invalid_arguments() -> Result<(), EvaluationError> {
    for arguments in ["1", "unknown: 1", "block: nil, block: nil", "{ || 1 }"] {
        assert_eq!(
            evaluate(&format!(
                "try {{ ArgumentChanges.new({arguments}) }} catch error {{ error is? ArgumentError }}"
            ))?,
            Value::Bool(true),
            "{arguments}"
        );
    }
    for arguments in [
        "positional: nil",
        "keywords: %[]",
        "rest: nil",
        "keyword_rest: %[]",
        "positional: %{\"bad\": 1}",
    ] {
        assert_eq!(
            evaluate(&format!(
                "try {{ ArgumentChanges.new({arguments}) }} catch error {{ error is? TypeError }}"
            ))?,
            Value::Bool(true),
            "{arguments}"
        );
    }
    Ok(())
}

#[test]
fn constructor_failures_are_catchable_by_type() -> Result<(), EvaluationError> {
    assert_eq!(
        evaluate(
            "try { ArgumentChanges.new(positional: %{\"bad\": 1}) } catch error { error is? TypeError }"
        )?,
        Value::Bool(true)
    );
    assert_eq!(
        evaluate(
            "try { ArgumentChanges.new(unknown: nil) } catch error { error is? ArgumentError }"
        )?,
        Value::Bool(true)
    );
    Ok(())
}

#[test]
fn changes_snapshot_structure_but_keep_shared_values() -> Result<(), EvaluationError> {
    let source = "mut held = %[1]; mut fixed = %{:value: held}; mut rest = %[held]; let changes = ArgumentChanges.new(positional: fixed, rest: rest); let ignored = fixed[:other] = 9; let ignored = rest.append(9); let ignored = held.append(2); changes";
    let Value::Decorator(record) = evaluate(source)? else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let DecoratorValue::ArgumentChanges(changes) = *record else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let shared = Value::Array(ArrayRef::new(vec![
        Value::Integer(1u64.into()),
        Value::Integer(2u64.into()),
    ]));
    assert_eq!(
        changes.positional(),
        Some(&[("value".into(), shared.clone())][..])
    );
    assert_eq!(changes.rest(), Some(&[shared][..]));
    Ok(())
}

#[test]
fn runtime_owned_records_cannot_be_constructed() {
    for name in [
        "Invocation",
        "InvocationSignature",
        "InvocationParameter",
        "DecoratorContext",
        "DecoratorProtocolError",
        "Plan",
        "Transformation",
    ] {
        assert!(
            matches!(evaluate(&format!("{name}.new()")), Err(EvaluationError::MessageNotFound { receiver_class, selector }) if receiver_class == name && selector == "new"),
            "{name}"
        );
    }
}

#[test]
fn all_decorator_contracts_require_both_members() {
    for name in [
        "ClassDecorator",
        "ModuleDecorator",
        "ContractDecorator",
        "MethodDecorator",
        "PropertyDecorator",
    ] {
        let source = format!(
            "class Missing {{}} impl Missing for {name} {{ public fun plan(declaration, arguments) -> Plan {{ Plan.empty }} }}; 1"
        );
        assert!(
            matches!(evaluate(&source), Err(EvaluationError::TypeContractError)),
            "{name}"
        );
    }
}

#[test]
fn both_phase_members_admit_an_ordinary_decorator_class() -> Result<(), EvaluationError> {
    let source = "class Complete {} impl Complete for MethodDecorator { public fun plan(declaration, arguments) -> Plan { Plan.empty }; public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty } }; Complete.new()";
    assert!(matches!(evaluate(source)?, Value::Object(_)));
    Ok(())
}

#[test]
fn missing_plan_and_async_phase_members_are_rejected() {
    for members in [
        "public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }",
        "public async fun plan(declaration, arguments) -> Plan { Plan.empty }; public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }",
    ] {
        assert!(matches!(
            evaluate(&format!(
                "class Invalid {{}} impl Invalid for MethodDecorator {{ {members} }}; 1"
            )),
            Err(EvaluationError::TypeContractError)
        ));
    }
}
