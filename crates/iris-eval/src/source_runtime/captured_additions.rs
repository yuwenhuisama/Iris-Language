use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ObjectId, Value};
use iris_syntax::{MethodDeclaration, MethodKind, Visibility};

impl SourceEvaluator {
    pub(super) fn captured_declaration(
        &self,
        selector: &str,
        closure: ObjectId,
    ) -> Result<MethodDeclaration, EvaluationError> {
        let record = self
            .closures
            .get(&closure)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        if super::body_yields(&record.body) {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        Ok(MethodDeclaration {
            decorators: Vec::new(),
            is_async: record.is_async,
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Instance,
            selector: selector.into(),
            type_parameters: Vec::new(),
            parameters: record.full_parameters.clone(),
            return_type: record.return_type.clone(),
            visibility: Visibility::Private,
            body: Some(record.body.clone()),
        })
    }

    pub(super) fn invoke_captured_method(
        &mut self,
        closure: ObjectId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let record = self
            .closures
            .get(&closure)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let parameters = record.full_parameters.clone();
        let result_type = record.return_type.clone();
        let is_async = record.is_async;
        let previous = self.wrapper_context();
        self.restore_wrapper_context(record.lexical_context.clone());
        let outcome = (|| {
            let locals = self.bind_parameters(&parameters, arguments)?;
            let ordered = parameters
                .iter()
                .map(|parameter| {
                    locals
                        .get(&parameter.name)
                        .cloned()
                        .ok_or(EvaluationError::ArgumentError)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let result = self.invoke_closure(closure, &ordered)?;
            if !is_async && let Some(annotation) = &result_type {
                self.check_binding_annotation(&result, annotation)?;
            }
            Ok(result)
        })();
        self.restore_wrapper_context(previous);
        outcome
    }
}
