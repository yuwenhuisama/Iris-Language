use super::{Binding, EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::DecoratorReason;
use iris_runtime::{ClassId, ObjectId, StaticSpine, Value};
use iris_syntax::{MethodDeclaration, MethodKind, Statement};

#[derive(Default)]
pub(super) struct CandidateDeclarations {
    pub callback: Option<(ObjectId, ClassId)>,
    pub frame: Option<(usize, ClassId)>,
    properties: Vec<MethodDeclaration>,
}

impl SourceEvaluator {
    pub(super) fn candidate_declaration(
        &mut self,
        statement: &Statement,
        frame: usize,
    ) -> Result<(), EvaluationError> {
        let Some((authorized, class)) = self.candidate_declarations.frame else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if authorized != frame {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let builtin = self.is_builtin_class(class);
        match statement {
            Statement::Method(method) => {
                if matches!(method.impl_contract, Some(Some(_))) {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
                self.class_method_as(class, builtin, true, method, false)?;
                match method.kind {
                    MethodKind::Property => {
                        self.candidate_declarations.properties.push(method.clone());
                        Ok(())
                    }
                    MethodKind::Instance | MethodKind::Class | MethodKind::Module => {
                        self.transform_method_decorators(class, method, DecoratorReason::Open)
                    }
                }
            }
            Statement::StoredProperty {
                class_level: false, ..
            } => {
                self.stored_property(class, statement)?;
                let declaration = stored_decorator_declaration(statement)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.candidate_declarations.properties.push(declaration);
                Ok(())
            }
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    pub(super) fn invoke_candidate_callback(
        &mut self,
        block: ObjectId,
        class: ClassId,
    ) -> Result<Value, EvaluationError> {
        let previous = std::mem::take(&mut self.candidate_declarations.properties);
        let result = self.invoke_closure(block, &[Value::Class(class)]);
        let properties = std::mem::replace(&mut self.candidate_declarations.properties, previous);
        let value = result?;
        for declaration in properties {
            self.transform_method_decorators(class, &declaration, DecoratorReason::Open)?;
        }
        Ok(value)
    }

    pub(super) fn candidate_failure(&mut self, error: EvaluationError) -> EvaluationError {
        match error {
            EvaluationError::Class(iris_runtime::ClassError::MetaCapabilityDenied { .. }) => {
                if self
                    .class_name("Kernel::MetaCapabilityError")
                    .ok()
                    .flatten()
                    .is_none()
                    && let Err(error) = self.install_candidate_error()
                {
                    return error;
                }
                let result = self
                    .class_name("Kernel::MetaCapabilityError")
                    .and_then(|class| class.ok_or(EvaluationError::NameError))
                    .and_then(|class| self.construct(class, &[]));
                match result {
                    Ok(object) => EvaluationError::Raised(Value::Object(object)),
                    Err(error) => error,
                }
            }
            error => error,
        }
    }

    pub(super) fn install_candidate_error(&mut self) -> Result<(), EvaluationError> {
        let object = self
            .class_name("Object")?
            .ok_or(EvaluationError::NameError)?;
        let class = self
            .runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), Some(object))
            .map_err(EvaluationError::Class)?;
        for name in ["MetaCapabilityError", "Kernel::MetaCapabilityError"] {
            self.names
                .insert(name.into(), Binding::immutable(Value::Class(class)));
        }
        Ok(())
    }
}

pub(super) fn stored_decorator_declaration(statement: &Statement) -> Option<MethodDeclaration> {
    let Statement::StoredProperty {
        decorators,
        name,
        annotation,
        visibility,
        ..
    } = statement
    else {
        return None;
    };
    Some(MethodDeclaration {
        decorators: decorators.clone(),
        is_async: false,
        is_override: false,
        impl_contract: None,
        kind: MethodKind::Property,
        selector: name.clone(),
        type_parameters: Vec::new(),
        parameters: Vec::new(),
        return_type: Some(annotation.clone()),
        visibility: *visibility,
        body: None,
    })
}
