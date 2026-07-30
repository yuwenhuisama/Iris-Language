use iris_eval::{EvaluationError, evaluate, evaluate_with_class_publication};
use iris_runtime::{KernelError, NumericError, Value as RuntimeValue};

use crate::{
    json::Value,
    model::{object, render, string},
};

pub(super) fn compare_observation(
    expected: &std::collections::BTreeMap<String, Value>,
    source: &str,
) -> Result<(), String> {
    let actual = evaluate(source).map_err(render_evaluation_error)?;
    match expected.get("side_effects") {
        Some(side_effects) => compare_side_effects(expected, side_effects, actual),
        None => compare_value(
            expected.get("value").ok_or("runtime value missing")?,
            expected.get("type"),
            actual,
        ),
    }
}

fn compare_side_effects(
    expected: &std::collections::BTreeMap<String, Value>,
    expected_side_effects: &Value,
    actual: RuntimeValue,
) -> Result<(), String> {
    let actual = observation_values(actual)?;
    match expected.get("value") {
        Some(expected_value) => {
            let [actual_value, actual_side_effects] = actual.as_slice() else {
                return Err("value and side_effects require a two-item observation array".into());
            };
            compare_value(expected_value, expected.get("type"), actual_value.clone())?;
            compare_rendered("side_effects", expected_side_effects, actual_side_effects)
        }
        None => {
            let [actual_side_effects] = actual.as_slice() else {
                return Err(
                    "side_effects without value require a one-item observation array".into(),
                );
            };
            compare_rendered("side_effects", expected_side_effects, actual_side_effects)
        }
    }
}

fn observation_values(actual: RuntimeValue) -> Result<Vec<RuntimeValue>, String> {
    match actual {
        RuntimeValue::Array(values) => Ok(values),
        _ => Err("side_effects require an observation array".into()),
    }
}

fn compare_value(
    expected: &Value,
    expected_type: Option<&Value>,
    actual: RuntimeValue,
) -> Result<(), String> {
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
    compare_rendered("value", expected, &actual)
}

fn compare_rendered(label: &str, expected: &Value, actual: &RuntimeValue) -> Result<(), String> {
    let actual = render_value(actual);
    let expected = render(expected);
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label} expected {expected}, actual {actual}"))
    }
}

pub(super) fn compare_error(expected: &Value, source: &str) -> Result<(), String> {
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

pub(super) fn compare_error_and_side_effects(
    expected_error: &Value,
    expected_side_effects: &Value,
    source: &str,
) -> Result<(), String> {
    let class = unpublished_class(expected_side_effects)?;
    let expected = string(object(expected_error)?, "code")?;
    let (outcome, published) = evaluate_with_class_publication(source, class);
    match outcome {
        Ok(value) => Err(format!(
            "error expected {expected}, actual value {}",
            render_value(&value)
        )),
        Err(error) if error_code(&error) != expected => Err(format!(
            "error expected {expected}, actual {}",
            error_code(&error)
        )),
        Err(_) if published => Err(format!("side_effects expected {class} is not published")),
        Err(_) => Ok(()),
    }
}

fn unpublished_class(expected: &Value) -> Result<&str, String> {
    let Value::String(expected) = expected else {
        return Err("error side_effects must name an unpublished Class".into());
    };
    expected
        .strip_suffix(" is not published.")
        .ok_or_else(|| "error side_effects must use '<Class> is not published.'".into())
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
        RuntimeValue::Class(_)
        | RuntimeValue::Type(_)
        | RuntimeValue::Contract(_)
        | RuntimeValue::Closure(_)
        | RuntimeValue::ContractView(_, _)
        | RuntimeValue::Object(_)
        | RuntimeValue::BoundMethod(_)
        | RuntimeValue::Method(_) => "{\"opaque\":true}".into(),
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
        RuntimeValue::Type(_) => "Type",
        RuntimeValue::Contract(_) => "Contract",
        RuntimeValue::Closure(_) => "Closure",
        RuntimeValue::ContractView(_, _) => "ContractView",
        RuntimeValue::Object(_) => "Object",
        RuntimeValue::BoundMethod(_) => "BoundMethod",
        RuntimeValue::Method(_) => "Method",
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
        EvaluationError::ComparisonContractError => "ComparisonContractError".into(),
        EvaluationError::Runtime(error) => kernel_error_code(error).into(),
        EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
            | iris_runtime::ClassError::ProtectedSuperclass { .. },
        ) => "MetaOperationError".into(),
        EvaluationError::Class(_) => "RuntimeError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::NoSuperMethod { .. },
        )) => "NoSuperMethodError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::VisibilityDenied { .. },
        )) => "MethodVisibilityError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::ContractDispatch { .. },
        )) => "ContractDispatchError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::MethodBinding { .. },
        )) => "MethodBindingError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::InvalidSuper { .. },
        )) => "InvalidSuperError".into(),
        EvaluationError::Construction(iris_runtime::ConstructionError::InstanceState {
            ..
        }) => "InstanceStateError".into(),
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
        KernelError::Dispatch(iris_runtime::DispatchError::VisibilityDenied { .. }) => {
            "MethodVisibilityError"
        }
        KernelError::Dispatch(iris_runtime::DispatchError::ContractDispatch { .. }) => {
            "ContractDispatchError"
        }
        KernelError::Dispatch(iris_runtime::DispatchError::MethodBinding { .. }) => {
            "MethodBindingError"
        }
        KernelError::Dispatch(iris_runtime::DispatchError::InvalidSuper { .. }) => {
            "InvalidSuperError"
        }
        KernelError::StableHash(iris_runtime::StableHashError::InvalidNumericKey) => {
            "InvalidKeyError"
        }
        KernelError::Class(_)
        | KernelError::Dispatch(_)
        | KernelError::Numeric(_)
        | KernelError::StableHash(_)
        | KernelError::Arity => "RuntimeError",
    }
}
