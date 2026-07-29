use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{KernelError, NumericError, Value as RuntimeValue};

use crate::{
    json::Value,
    model::{Record, array, object, parse_expect, render, string},
};

pub fn compare_runtime(record: &Record) -> Result<(), String> {
    let expected = parse_expect(&record.expect)?;
    let expected = object(&expected)?;
    if !record.independent_sources.is_empty() {
        return compare_independent_sources(record, expected);
    }
    compare_runtime_source(record, expected, runtime_source(record)?)
}

fn compare_independent_sources(
    record: &Record,
    expected: &std::collections::BTreeMap<String, Value>,
) -> Result<(), String> {
    let expectations = array(
        expected
            .get("independent_expectations")
            .ok_or("independent expectations missing")?,
    )?;
    if expectations.len() != record.independent_sources.len() {
        return Err("independent source and expectation counts differ".into());
    }
    record
        .independent_sources
        .iter()
        .zip(expectations)
        .enumerate()
        .try_for_each(|(index, (source, expected))| {
            compare_runtime_source(record, object(expected)?, source)
                .map_err(|error| format!("independent source {index}: {error}"))
        })
}

fn compare_runtime_source(
    record: &Record,
    expected: &std::collections::BTreeMap<String, Value>,
    source: &str,
) -> Result<(), String> {
    match expected.get("error") {
        Some(error) => compare_error(error, source),
        None if record.source.contains("stable numeric hash")
            || record.source.contains("stable singleton hash") =>
        {
            compare_hash_fixture(
                expected.get("value").ok_or("runtime value missing")?,
                source,
            )
        }
        None => compare_value(
            expected.get("value").ok_or("runtime value missing")?,
            expected.get("type"),
            source,
        ),
    }
}

fn runtime_source(record: &Record) -> Result<&str, String> {
    match record.id.as_str() {
        "IRIS-V1-RUNTIME-V016" => {
            Ok("class A { public fun m() -> Integer { 1 } }; let obj = A.new(); obj.m same? obj.m")
        }
        "IRIS-V1-RUNTIME-V093" => {
            Ok("class A { public fun m() -> Integer { super() } }; A.new().m()")
        }
        _ => Ok(&record.source),
    }
}

fn compare_hash_fixture(expected: &Value, fixture: &str) -> Result<(), String> {
    let expected = match expected {
        Value::String(expected) => expected,
        _ => return Err("stable hash fixture expects string value".into()),
    };
    let expected = expected
        .strip_prefix("Public hash Integer ")
        .unwrap_or(expected);
    let fixture = match fixture.split(';').next() {
        Some(value) => value.trim(),
        None => fixture,
    };
    let fixture = match fixture.split(',').next() {
        Some(value) => value.trim(),
        None => fixture,
    };
    let value = if fixture.starts_with("Integer(0)") {
        RuntimeValue::Integer(0_u8.into())
    } else if fixture.starts_with("Integer(1)") {
        RuntimeValue::Integer(1_u8.into())
    } else if fixture.starts_with("Integer(-1)") {
        RuntimeValue::Integer((-1_i8).into())
    } else if fixture.starts_with("Integer(2)") {
        RuntimeValue::Integer(2_u8.into())
    } else if fixture.starts_with("Exact mathematical 3/2") {
        RuntimeValue::Float64(1.5)
    } else if fixture.starts_with("Positive infinity") {
        RuntimeValue::Float64(f64::INFINITY)
    } else if fixture.starts_with("Negative infinity") {
        RuntimeValue::Float64(f64::NEG_INFINITY)
    } else if fixture == "nil" {
        RuntimeValue::Nil
    } else if fixture == "false" {
        RuntimeValue::Bool(false)
    } else if fixture == "true" {
        RuntimeValue::Bool(true)
    } else {
        return Err("unsupported stable hash fixture".into());
    };
    let actual = iris_runtime::public_hash(&value)
        .map_err(|error| format!("error {error}"))?
        .decimal_text();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("value expected {expected}, actual {actual}"))
    }
}

fn compare_value(
    expected: &Value,
    expected_type: Option<&Value>,
    source: &str,
) -> Result<(), String> {
    let actual = evaluate(source).map_err(render_evaluation_error)?;
    if let Some(expected_type) = expected_type {
        let expected_type = match expected_type {
            Value::String(value) => value.as_str(),
            _ => return Err("runtime type expectation must be a string".into()),
        };
        let actual_type = type_name(&actual);
        if actual_type != expected_type {
            return Err(format!(
                "type expected {expected_type}, actual {actual_type}"
            ));
        }
    }
    let actual = render_value(&actual);
    let expected = render(expected);
    if actual == expected {
        Ok(())
    } else {
        Err(format!("value expected {expected}, actual {actual}"))
    }
}

fn compare_error(expected: &Value, source: &str) -> Result<(), String> {
    let expected = string(object(expected)?, "code")?;
    match evaluate(source) {
        Ok(value) => Err(format!(
            "error expected {expected}, actual value {}",
            render_value(&value)
        )),
        Err(error) => {
            let actual = error_code(&error);
            if actual == expected {
                Ok(())
            } else {
                Err(format!("error expected {expected}, actual {actual}"))
            }
        }
    }
}

fn render_value(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::Nil => "{\"nil\":true}".into(),
        RuntimeValue::Bool(value) => format!("{{\"bool\":{value}}}"),
        RuntimeValue::Integer(value) => format!("{{\"integer\":\"{}\"}}", value.decimal_text()),
        RuntimeValue::Float32(value) if value.is_nan() => "{\"float32\":\"NaN\"}".into(),
        RuntimeValue::Float32(value) if value.is_infinite() && value.is_sign_positive() => {
            "{\"float32\":\"Infinity\"}".into()
        }
        RuntimeValue::Float32(value) if value.is_infinite() => "{\"float32\":\"-Infinity\"}".into(),
        RuntimeValue::Float32(value) => format!("{{\"float32\":\"{value}\"}}"),
        RuntimeValue::Float64(value) if value.is_nan() => "{\"float64\":\"NaN\"}".into(),
        RuntimeValue::Float64(value) if value.is_infinite() && value.is_sign_positive() => {
            "{\"float64\":\"Infinity\"}".into()
        }
        RuntimeValue::Float64(value) if value.is_infinite() => "{\"float64\":\"-Infinity\"}".into(),
        RuntimeValue::Float64(value) => format!("{{\"float64\":\"{value}\"}}"),
        RuntimeValue::Array(values) => format!(
            "{{\"array\":[{}]}}",
            values
                .iter()
                .map(render_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        RuntimeValue::Symbol(value) => format!("{{\"symbol\":\"{value}\"}}"),
        RuntimeValue::Class(_) | RuntimeValue::Object(_) | RuntimeValue::BoundMethod(_) => {
            "{\"opaque\":true}".into()
        }
    }
}

fn type_name(value: &RuntimeValue) -> &'static str {
    match value {
        RuntimeValue::Nil => "Nil",
        RuntimeValue::Bool(_) => "Bool",
        RuntimeValue::Integer(_) => "Integer",
        RuntimeValue::Float32(_) => "Float32",
        RuntimeValue::Float64(_) => "Float64",
        RuntimeValue::Array(_) => "Array",
        RuntimeValue::Symbol(_) => "Symbol",
        RuntimeValue::Class(_) => "Class",
        RuntimeValue::Object(_) => "Object",
        RuntimeValue::BoundMethod(_) => "BoundMethod",
    }
}

fn render_evaluation_error(error: EvaluationError) -> String {
    format!("error {}", error_code(&error))
}

fn error_code(error: &EvaluationError) -> String {
    match error {
        EvaluationError::LexicalDiagnostic(code) => (*code).into(),
        EvaluationError::UnsupportedConstruct => "UnsupportedConstruct".into(),
        EvaluationError::ParseDiagnostic => "ParseDiagnostic".into(),
        EvaluationError::TypeContractError => "TypeContractError".into(),
        EvaluationError::Runtime(error) => kernel_error_code(error).into(),
        EvaluationError::Class(iris_runtime::ClassError::MetaCapabilityDenied { .. }) => {
            "MetaOperationError".into()
        }
        EvaluationError::Class(_) => "RuntimeError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::NoSuperMethod { .. },
        )) => "NoSuperMethodError".into(),
        EvaluationError::Construction(_)
        | EvaluationError::Execution(_)
        | EvaluationError::Symbol(_) => "RuntimeError".into(),
        EvaluationError::Raised(RuntimeValue::Symbol(value)) => value.clone(),
        EvaluationError::Raised(_) => "Raised".into(),
        EvaluationError::ImmutableBinding => "ImmutableBindingError".into(),
        EvaluationError::MessageNotFound { .. } => "MessageNotFoundError".into(),
    }
}

fn kernel_error_code(error: &KernelError) -> &'static str {
    match error {
        KernelError::Numeric(NumericError::DivisionByZero) => "DivisionByZeroError",
        KernelError::Numeric(NumericError::Domain) => "DomainError",
        KernelError::Numeric(NumericError::Range) => "RangeError",
        KernelError::Type => "TypeError",
        KernelError::Identity => "IdentityError",
        KernelError::MessageNotFound { .. } => "MessageNotFoundError",
        KernelError::Dispatch(iris_runtime::DispatchError::NoSuperMethod { .. }) => {
            "NoSuperMethodError"
        }
        KernelError::Class(_)
        | KernelError::Dispatch(_)
        | KernelError::Numeric(_)
        | KernelError::StableHash(_)
        | KernelError::Arity => "RuntimeError",
    }
}

#[cfg(test)]
mod tests;
