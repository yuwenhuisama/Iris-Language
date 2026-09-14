use super::{EvaluationError, SourceEvaluator, Value};
use crate::{CoreErrorKind, HostException};
use iris_runtime::CoreClass;

impl SourceEvaluator {
    pub(crate) fn program(
        &mut self,
        program: &iris_syntax::Program,
    ) -> Result<Value, EvaluationError> {
        if let Some(code) = iris_parser::analyze(program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .find(|code| {
                matches!(
                    *code,
                    "BINDING_LET_REQUIRES_INITIALIZER" | "BINDING_MISSING_TYPE_FOR_DEFERRED_INIT"
                )
            })
        {
            return Err(EvaluationError::StaticDiagnostic(code));
        }
        self.program_inner(program)
            .map_err(|error| self.host_exception(error))
    }

    fn host_exception(&self, error: EvaluationError) -> EvaluationError {
        let EvaluationError::Raised(Value::Object(object)) = &error else {
            return error;
        };
        let class = match self.runtime.class_of(*object) {
            Ok(class) => class,
            Err(error) => return EvaluationError::Construction(error),
        };
        let kind = match class {
            class if class == CoreClass::ArgumentError.id() => CoreErrorKind::ArgumentError,
            class if class == CoreClass::TypeError.id() => CoreErrorKind::TypeError,
            _ => return error,
        };
        let value = Value::Object(*object);
        let context = match &self.active_context {
            Some(context @ Value::ExceptionContext(_, carried, ..)) if **carried == value => {
                Some(context.clone())
            }
            _ => None,
        };
        EvaluationError::HostException(Box::new(HostException {
            kind,
            value,
            context,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_metadata_survives_when_evaluator_is_dropped() {
        let mut given = SourceEvaluator::new_in_package("test").unwrap();
        let native = iris_native_host::NativeError::Raised {
            code: "ArgumentError".into(),
            message: "bad arguments".into(),
            os_code: 42,
        };
        let raised = given.native_error(native);
        let context = given.active_context.clone();

        let when = given.host_exception(raised);
        drop(given);

        let EvaluationError::HostException(exception) = when else {
            panic!("expected owned native error metadata")
        };
        assert_eq!(exception.kind, CoreErrorKind::ArgumentError);
        assert_eq!(exception.context, context);
        let Some(Value::ExceptionContext(_, value, ..)) = exception.context else {
            panic!("expected native context")
        };
        assert_eq!(*value, exception.value);
    }
}
