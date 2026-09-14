use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, ModuleId, Value};
use iris_syntax::{ModuleDeclaration, Statement};
use std::collections::HashMap;

impl SourceEvaluator {
    pub(super) fn module_candidate_body(
        &mut self,
        module: ModuleId,
        main: ClassId,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        let executable = declaration
            .body
            .iter()
            .filter(|statement| {
                matches!(
                    statement,
                    Statement::Expression(_) | Statement::Binding { .. } | Statement::Raise(_)
                )
            })
            .collect::<Vec<_>>();
        if executable.is_empty() {
            return Ok(());
        }
        let shadowed = executable
            .iter()
            .filter_map(|statement| match statement {
                Statement::Binding { name, .. } => {
                    Some((name.clone(), self.names.get(name).cloned()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let origin = self.runtime.registry().staged_origin(main).is_ok();
        let receiver = if origin {
            Value::Symbol(declaration.name.clone())
        } else {
            Value::Object(self.construct(main, &[])?)
        };
        let previous = self.module_body_main.replace(main);
        if origin {
            self.origin_names.insert(main, declaration.name.clone());
        }
        let outcome = executable.into_iter().try_for_each(|statement| {
            if let Statement::Expression(expression) = statement
                && let Some(selector) = self.candidate_define_method(main, expression)?
            {
                return self.publish_module_copy(module, main, selector);
            }
            self.statement(statement, &HashMap::new(), Some(receiver.clone()))?;
            if let Statement::Binding {
                constant: true,
                name,
                ..
            } = statement
            {
                let binding = self.names.get(name).ok_or(EvaluationError::NameError)?;
                self.module_constants
                    .insert((module, name.clone()), binding.value());
            }
            Ok(())
        });
        self.module_body_main = previous;
        if origin {
            self.origin_names.remove(&main);
        }
        for (name, outer) in shadowed {
            match outer {
                Some(binding) => {
                    self.names.insert(name, binding);
                }
                None => {
                    self.names.remove(&name);
                }
            }
        }
        outcome
    }
}
