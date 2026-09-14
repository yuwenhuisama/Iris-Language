use super::super::stored_properties::stored_property_bodies;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::ClassId;
use iris_syntax::{ClassDeclaration, MethodDeclaration, Statement};

pub(in super::super) fn replay_methods(
    body: &[Statement],
) -> Result<Vec<MethodDeclaration>, EvaluationError> {
    let mut methods = Vec::new();
    for statement in body {
        match statement {
            Statement::Method(method) => methods.push(method.clone()),
            Statement::StoredProperty {
                class_level: false, ..
            } => {
                methods.extend(stored_property_bodies(statement)?.0);
            }
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
    }
    Ok(methods)
}

impl SourceEvaluator {
    pub(in super::super) fn validate_history_storage(
        &self,
        class: ClassId,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        let mut current = Vec::new();
        for definition in &self.decorator_definitions {
            if self.class_name(&definition.name)? != Some(class) {
                continue;
            }
            for statement in &definition.body {
                if let Statement::StoredProperty {
                    name,
                    annotation,
                    class_level: false,
                    ..
                } = statement
                {
                    if let Some(existing) =
                        current.iter_mut().find(|(existing, _)| existing == &name)
                    {
                        *existing = (name, annotation);
                    } else {
                        current.push((name, annotation));
                    }
                }
            }
        }
        let historical: Vec<_> = declaration
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::StoredProperty {
                    name,
                    annotation,
                    class_level: false,
                    ..
                } => Some((name, annotation)),
                _ => None,
            })
            .collect();
        let active = self
            .runtime
            .registry()
            .active(class)
            .map_err(EvaluationError::Class)?;
        if current != historical || active.properties().len() != current.len() {
            return Err(EvaluationError::TypeContractError);
        }
        for (property, (name, annotation)) in active.properties().iter().zip(current) {
            if self.selectors.get(&format!("@{name}")) != Some(&property.selector())
                || self.property_types.get(&(class, property.selector())) != Some(annotation)
            {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }

    pub(in super::super) fn stage_history_initializers(
        &mut self,
        class: ClassId,
        body: &[Statement],
    ) -> Result<(), EvaluationError> {
        for statement in body {
            if let Statement::StoredProperty { name, .. } = statement {
                let (_, initializer) = stored_property_bodies(statement)?;
                let body = self.register_body(initializer);
                let selector = self.selector(&format!("@{name}"));
                self.runtime
                    .registry_mut()
                    .publish_stored_property(class, selector, body)
                    .map_err(EvaluationError::Class)?;
            }
        }
        Ok(())
    }
}
