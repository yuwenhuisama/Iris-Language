use iris_runtime::{ArrayRef, Value};
use iris_vm::{CompileError, MachineError, compile, run};

#[test]
fn array_cell_admits_readonly_representation_when_assigned_properties() -> Result<(), CompileError>
{
    let source = r#"
        class Sample { public property value: Integer = 1 }
        mut properties = []
        let assigned = properties = Sample.properties
        properties
    "#;
    let program = compile(source)?;

    let outcome = run(&program);

    assert_eq!(
        outcome,
        Ok(Value::ReadonlyArray(vec![Value::Symbol("@value".into())]))
    );
    Ok(())
}

#[test]
fn array_cell_retains_old_value_when_dynamic_integer_is_rejected() -> Result<(), CompileError> {
    let source = r#"
        module Input { public module fun read(value) { value } }
        mut properties = [:original]
        let failure = try { properties = Input.read(7) } catch error { error }
        [failure, properties]
    "#;
    let program = compile(source)?;

    let outcome = run(&program);

    assert_eq!(
        outcome,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("TypeContractError".into()),
            Value::Array(ArrayRef::new(vec![Value::Symbol("original".into())])),
        ])))
    );
    Ok(())
}

#[test]
fn readonly_mutation_is_rejected_when_properties_pass_array_guard() -> Result<(), CompileError> {
    let source = r#"
        class Sample { public property value: Integer = 1 }
        mut properties = []
        let assigned = properties = Sample.properties
        properties.append(:extra)
    "#;
    let program = compile(source)?;

    let outcome = run(&program);

    assert_eq!(outcome, Err(MachineError::ReadonlyMutation));
    Ok(())
}

#[test]
fn mutable_array_stays_mutable_when_replacement_passes_array_guard() -> Result<(), CompileError> {
    let source = r#"
        module Input { public module fun read(value) { value } }
        mut properties = []
        let assigned = properties = Input.read([:replacement])
        let appended = properties.append(:extra)
        properties
    "#;
    let program = compile(source)?;

    let outcome = run(&program);

    assert_eq!(
        outcome,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("replacement".into()),
            Value::Symbol("extra".into()),
        ])))
    );
    Ok(())
}
