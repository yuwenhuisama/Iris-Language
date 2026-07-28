use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{KernelError, NumericError, Value as RuntimeValue};

use crate::{
    json::Value,
    model::{Record, object, parse_expect, render, string},
};

pub fn compare_runtime(record: &Record) -> Result<(), String> {
    let expected = parse_expect(&record.expect)?;
    let expected = object(&expected)?;
    match expected.get("error") {
        Some(error) => compare_error(error, &record.source),
        None => compare_value(
            expected.get("value").ok_or("runtime value missing")?,
            expected.get("type"),
            &record.source,
        ),
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
        RuntimeValue::Class(_) | RuntimeValue::Object(_) => "{\"opaque\":true}".into(),
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
        RuntimeValue::Class(_) => "Class",
        RuntimeValue::Object(_) => "Object",
    }
}

fn render_evaluation_error(error: EvaluationError) -> String {
    format!("error {}", error_code(&error))
}

fn error_code(error: &EvaluationError) -> &'static str {
    match error {
        EvaluationError::LexicalDiagnostic(code) => code,
        EvaluationError::UnsupportedConstruct => "UnsupportedConstruct",
        EvaluationError::ParseDiagnostic => "ParseDiagnostic",
        EvaluationError::Runtime(error) => kernel_error_code(error),
    }
}

fn kernel_error_code(error: &KernelError) -> &'static str {
    match error {
        KernelError::Numeric(NumericError::DivisionByZero) => "DivisionByZeroError",
        KernelError::Numeric(NumericError::Range) => "RangeError",
        KernelError::Type => "TypeError",
        KernelError::Class(_)
        | KernelError::Dispatch(_)
        | KernelError::Numeric(_)
        | KernelError::Arity
        | KernelError::MissingMethod => "RuntimeError",
    }
}

#[cfg(test)]
mod tests {
    use super::compare_runtime;
    use crate::model::Record;

    #[test]
    fn expected_division_by_zero_is_accepted() {
        // Given
        let record = Record {
            id: "test".into(),
            source: "1 div 0".into(),
            expect: "{\"error\":{\"code\":\"DivisionByZeroError\"}}".into(),
            tags: vec![],
        };

        // When
        let result = compare_runtime(&record);

        // Then
        assert_eq!(result, Ok(()));
    }
}
