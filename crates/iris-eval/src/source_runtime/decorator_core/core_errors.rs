use super::{CoreClass, EvaluationError, SourceEvaluator, Value};

impl SourceEvaluator {
    pub(in crate::source_runtime) fn core_boundary_error(
        &mut self,
        error: EvaluationError,
    ) -> EvaluationError {
        let class = match error {
            EvaluationError::Runtime(iris_runtime::KernelError::Type) => CoreClass::TypeError,
            EvaluationError::ArgumentError => CoreClass::ArgumentError,
            error => return error,
        };
        match self.construct(class.id(), &[]) {
            Ok(object) => {
                let value = Value::Object(object);
                self.active_context = Some(Value::ExceptionContext(
                    self.next_context_identity(),
                    Box::new(value.clone()),
                    Box::new(Value::Nil),
                    Vec::new(),
                    Vec::new(),
                    Box::new(self.source_location(0).into()),
                ));
                EvaluationError::Raised(value)
            }
            Err(error) => error,
        }
    }

    pub(in crate::source_runtime) fn core_error_send(
        &self,
        object: iris_runtime::ObjectId,
        selector: &str,
    ) -> Result<Option<Value>, EvaluationError> {
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        let name = match class {
            class if class == CoreClass::ArgumentError.id() => "ArgumentError",
            class if class == CoreClass::TypeError.id() => "TypeError",
            _ => return Ok(None),
        };
        Ok(match selector {
            "class" => Some(Value::Class(class)),
            "class_name" => Some(Value::Symbol(name.into())),
            "to_string" | "inspect" => Some(Value::Text(name.into())),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_core_error_keeps_context_metadata_when_crossing_boundary() {
        let mut given = SourceEvaluator::new_in_package("test").unwrap();
        let original = Value::Symbol("ArgumentError".into());
        let identity = given.next_context_identity();
        let origin: iris_runtime::ExceptionOrigin = given.source_location(0).into();
        let error = iris_native_host::NativeError::Context {
            error: Box::new(iris_native_host::NativeError::Raised {
                code: "ArgumentError".into(),
                message: "bad native arguments".into(),
                os_code: 42,
            }),
            context: Box::new(Value::ExceptionContext(
                identity,
                Box::new(original),
                Box::new(Value::Nil),
                Vec::new(),
                Vec::new(),
                Box::new(origin.clone()),
            )),
        };

        let when = given.native_error(error);

        let EvaluationError::Raised(Value::Object(object)) = when else {
            panic!("expected boxed native ArgumentError: {when:?}")
        };
        assert_eq!(
            given.runtime.class_of(object).unwrap(),
            CoreClass::ArgumentError.id()
        );
        assert_eq!(
            given.active_context,
            Some(Value::ExceptionContext(
                identity,
                Box::new(Value::Object(object)),
                Box::new(Value::Nil),
                Vec::new(),
                Vec::new(),
                Box::new(origin),
            ))
        );
    }
}
