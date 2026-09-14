use super::candidate_declarations::stored_decorator_declaration;
use super::decorator_phases::{PhaseTarget, method_kind};
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, decorator_protocol::DecoratorReason};
use iris_syntax::Statement;

impl SourceEvaluator {
    pub(super) fn plan_class_members(
        &mut self,
        class: ClassId,
        (body, reason): (&[Statement], DecoratorReason),
    ) -> Result<(), EvaluationError> {
        for statement in body {
            let stored = stored_decorator_declaration(statement);
            let method = match statement {
                Statement::Method(method) => Some(method),
                _ => stored.as_ref(),
            };
            if let Some(method) = method {
                for decorator in &method.decorators {
                    let metadata = self.method_decorator_metadata(class, method)?;
                    self.execute_decorator_phase(
                        decorator,
                        PhaseTarget {
                            kind: method_kind(method),
                            reason,
                            metadata,
                            candidate: None,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }
}
