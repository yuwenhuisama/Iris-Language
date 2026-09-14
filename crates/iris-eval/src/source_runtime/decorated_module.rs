use super::{EvaluationError, SourceEvaluator, meta_capabilities};
use iris_runtime::{CompositionEdge, ModuleCandidateError, RuntimeStructuralError};
use iris_syntax::{ModuleDeclaration, TypeExpression};

pub(super) fn module_error(error: ModuleCandidateError) -> EvaluationError {
    EvaluationError::DecoratorDiagnostic {
        code: match error {
            ModuleCandidateError::CapabilityDenied { .. } => "IRIS-MODULE-CAPABILITY-DENIED",
            _ => "IRIS-MODULE-CANDIDATE",
        },
        phase: "candidate validation",
    }
}

impl SourceEvaluator {
    pub(super) fn decorated_module(
        &mut self,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        if self.open_target.is_some() || self.module_candidate.is_some() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        if declaration.reopen
            && (!declaration.mixins.is_empty() || !declaration.meta_deny.is_empty())
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let existing = declaration
            .reopen
            .then(|| self.module_names.get(&declaration.name).copied())
            .flatten();
        let mut edges = Vec::new();
        for mixin in &declaration.mixins {
            let TypeExpression::Name(name) = &mixin.target else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            edges.push(CompositionEdge::new(
                *self
                    .module_names
                    .get(name)
                    .ok_or(EvaluationError::UnsupportedConstruct)?,
                mixin.private_access,
            ));
        }
        let policy = meta_capabilities(&declaration.meta_deny)?;
        let module = match existing {
            Some(module) => {
                self.runtime
                    .registry_mut()
                    .begin_module_transaction(module)
                    .map_err(module_error)?;
                module
            }
            None => self
                .runtime
                .registry_mut()
                .stage_module_origin(&edges, policy)
                .map_err(module_error)?,
        };
        self.module_candidate = Some(module);
        let outcome = self.module_transaction(module, declaration);
        self.module_candidate = None;
        if outcome.is_err()
            && existing.is_none()
            && !self.failed_modules.contains(&declaration.name)
        {
            self.failed_modules.push(declaration.name.clone());
        }
        outcome
    }

    pub(super) fn publish_module_transaction(&mut self) -> Result<(), EvaluationError> {
        if !self.upgrade_state.active {
            self.runtime
                .commit_structural_group()
                .map_err(|error| match error {
                    RuntimeStructuralError::Class(error) => EvaluationError::Class(error),
                    RuntimeStructuralError::Module(error) => module_error(error),
                })?;
        }
        Ok(())
    }
}
