use super::EvaluationError;
use iris_runtime::decorator_protocol::{ArgumentError, ConstructionError, DecoratorValue};

pub(super) fn argument_error(error: ArgumentError) -> EvaluationError {
    if error.is_type_error() {
        EvaluationError::Runtime(iris_runtime::KernelError::Type)
    } else {
        EvaluationError::ArgumentError
    }
}

pub(super) fn phase_error(error: ConstructionError) -> EvaluationError {
    match error {
        ConstructionError::Protocol(error) => {
            EvaluationError::Raised(DecoratorValue::ProtocolError(error).into())
        }
        ConstructionError::Kind(_) => EvaluationError::TypeContractError,
    }
}
