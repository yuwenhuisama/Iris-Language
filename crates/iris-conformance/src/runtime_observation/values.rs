use iris_eval::{EvaluationError, evaluate, evaluate_with_class_publication};
use iris_runtime::{KernelError, NumericError, Value as RuntimeValue};

use crate::{
    json::Value,
    model::{object, render, string},
};

/// Compares an ALREADY EVALUATED outcome against a value expectation.
///
/// A multi-package program is evaluated by the caller, because `D-431` needs
/// its packages to share one runtime, so the outcome arrives here rather than
/// a source string.
pub(super) fn compare_evaluated(
    expected: &std::collections::BTreeMap<String, Value>,
    outcome: Result<iris_runtime::Value, iris_eval::EvaluationError>,
) -> Result<(), String> {
    let actual = outcome.map_err(render_evaluation_error)?;
    compare_value(
        expected.get("value").ok_or("runtime value missing")?,
        expected.get("type"),
        actual,
    )
}

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

/// Compares an ALREADY EVALUATED outcome against an error expectation.
pub(super) fn compare_evaluated_error(
    expected: &Value,
    outcome: Result<iris_runtime::Value, iris_eval::EvaluationError>,
) -> Result<(), String> {
    let expected = string(object(expected)?, "code")?;
    match outcome {
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
        // A ReadonlyArray is an ORDERED readable collection under C079, so it
        // renders as an array. Only its mutation is rejected, and a distinct
        // rendering would make every existing suppressed-list expectation
        // opaque rather than comparable.
        RuntimeValue::Array(values) | RuntimeValue::ReadonlyArray(values) => format!(
            "{{\"array\":[{}]}}",
            values
                .iter()
                .map(render_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        // IRIS-V1-COLLECTIONS-C033 leaves Hash iteration order UNSPECIFIED, so
        // only the entry count is rendered. Rendering the entries in stored
        // order would let a vector depend on an order the clause refuses to
        // promise.
        RuntimeValue::Hash(entries) => format!("{{\"hash\":\"{}\"}}", entries.len()),
        // IRIS-V1-CONFORMANCE-C027 names `string` as the expectation key for an
        // observable String value.
        RuntimeValue::Text(value) => format!("{{\"string\":\"{value}\"}}"),
        RuntimeValue::Symbol(value) => format!("{{\"symbol\":\"{value}\"}}"),
        RuntimeValue::Class(_)
        | RuntimeValue::Type(..)
        | RuntimeValue::ComposedType(_)
        | RuntimeValue::Contract(_)
        | RuntimeValue::Closure(_)
        | RuntimeValue::KeywordArgument(_, _)
        | RuntimeValue::IterationYield(_)
        | RuntimeValue::SourceLocation(..)
        | RuntimeValue::StackFrame(..)
        | RuntimeValue::RaiseSite(_)
        | RuntimeValue::ArrayIterator(_)
        | RuntimeValue::IterationDone
        | RuntimeValue::ExceptionContext(..)
        | RuntimeValue::ContractView(_, _)
        | RuntimeValue::Object(_)
        | RuntimeValue::BoundMethod(_)
        | RuntimeValue::Method(_)
        | RuntimeValue::Transformation { .. } => "{\"opaque\":true}".into(),
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
        RuntimeValue::Hash(_) => "Hash",
        RuntimeValue::ReadonlyArray(_) => "ReadonlyArray",
        RuntimeValue::SourceLocation(..) => "SourceLocation",
        RuntimeValue::StackFrame(..) => "StackFrame",
        RuntimeValue::RaiseSite(_) => "RaiseSite",
        RuntimeValue::Text(_) => "String",
        RuntimeValue::Symbol(_) => "Symbol",
        RuntimeValue::Class(_) => "Class",
        RuntimeValue::Type(..) | RuntimeValue::ComposedType(_) => "Type",
        RuntimeValue::Contract(_) => "Contract",
        RuntimeValue::Closure(_) => "Closure",
        RuntimeValue::KeywordArgument(_, _) | RuntimeValue::IterationYield(_) => "Iteration",
        RuntimeValue::ArrayIterator(..) | RuntimeValue::IterationDone => "Iteration",
        RuntimeValue::ExceptionContext(..) => "ExceptionContext",
        RuntimeValue::ContractView(_, _) => "ContractView",
        RuntimeValue::Object(_) => "Object",
        RuntimeValue::BoundMethod(_) => "BoundMethod",
        RuntimeValue::Method(_) => "Method",
        RuntimeValue::Transformation { .. } => "Transformation",
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
        // C033 forbids both targets and V439 requires the same name the
        // declarative spelling reports statically.
        EvaluationError::ClosedGenericOpenForbidden => "CLOSED_GENERIC_OPEN_FORBIDDEN".into(),
        // C049 requires import-site replacement authorization; V349 names the
        // link-phase diagnostic.
        EvaluationError::ImportReplacementAuthorization => {
            "IRIS-IMPORT-REPLACEMENT-AUTHORIZATION".into()
        }
        EvaluationError::InvalidInstanceVariableName => "InvalidInstanceVariableNameError".into(),
        // C006 makes `(package_id, api_major)` one identity; V352 names the
        // link failure when a lock selects two implementations of that pair.
        EvaluationError::PackageVersionUnification => "PackageVersionUnificationError".into(),
        // C045 requires a DIRECT import to activate a static extension member;
        // V346, V418 and V438 name the diagnostic.
        EvaluationError::StaticMemberNotFound => "IRIS-STATIC-MEMBER-NOT-FOUND".into(),
        // C066 and D-271 verify the stored digest before reconstruction;
        // V357 names the failure.
        EvaluationError::RevisionArtifactUnavailable => "RevisionArtifactUnavailableError".into(),
        // C003 refuses a load whose REQUIRED permission is ungranted; C102 and
        // C103 refuse an ungranted or out-of-scope reflection call.
        EvaluationError::PermissionDenied => "PermissionDenied".into(),
        EvaluationError::ReflectionAccess => "ReflectionAccessError".into(),
        // C037 raises MetaTransactionError on a dynamic suspension attempt.
        EvaluationError::MetaTransactionSuspension => "MetaTransactionError".into(),
        EvaluationError::IdentityError => "IdentityError".into(),
        EvaluationError::ComparisonContractError => "ComparisonContractError".into(),
        EvaluationError::ArgumentError => "ArgumentError".into(),
        EvaluationError::PatternMatchError => "PatternMatchError".into(),
        EvaluationError::NoActiveExceptionError => "NoActiveExceptionError".into(),
        EvaluationError::ExceptionChainError => "ExceptionChainError".into(),
        EvaluationError::ModuleInitializationCycleError => "ModuleInitializationCycleError".into(),
        EvaluationError::InstanceVariableNotFoundError => "InstanceVariableNotFoundError".into(),
        // A  that escaped every callable boundary has no target, which
        // is the same control-target failure a stray break reports.
        EvaluationError::LoopBreak(..)
        | EvaluationError::LoopContinue(_)
        | EvaluationError::Return(_) => "ControlTargetError".into(),
        EvaluationError::Runtime(error) => kernel_error_code(error).into(),
        // IRIS-V1-META-C125 and the V430 row fix this name as
        // `MetaCapabilityError`, which is also what `catchable_name` already
        // reports to Iris source. `MetaOperationError` appeared nowhere in the
        // specification set and made an uncaught meta-capability failure
        // observe a different name than a caught one.
        EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
            | iris_runtime::ClassError::ProtectedSuperclass { .. },
        ) => "MetaCapabilityError".into(),
        // IRIS-V1-CONTROL-C078 names this code, so a class-variable
        // redeclaration is distinguishable from any other Class failure.
        EvaluationError::Class(iris_runtime::ClassError::DuplicateClassVariable { .. }) => {
            "CLASS_VARIABLE_REDECLARATION".into()
        }
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
        EvaluationError::NameError => "NameError".into(),
        EvaluationError::ReadonlyProperty => "ReadonlyPropertyError".into(),
        EvaluationError::ReadonlyMutation => "ReadonlyMutationError".into(),
        // A harness limit rather than an Iris error, reported so a
        // non-terminating vector is visible evidence instead of a hang.
        EvaluationError::StepBudgetExhausted => "StepBudgetExhausted".into(),
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
