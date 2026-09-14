use super::decorated_module::module_error;
use super::history_context::declarations;
use super::rollback_artifact::{Artifact, BodyContext};
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ModuleId, Value};
use iris_syntax::{Declaration, MethodKind, ModuleDeclaration, Statement};
use std::collections::HashMap;
use std::rc::Rc;

fn supported(declaration: &ModuleDeclaration) -> bool {
    declaration.parameters.is_empty()
        && declaration.constraints.is_empty()
        && declaration.contract_for.is_empty()
        && declaration.mixins.is_empty()
        && declaration.decorators.is_empty()
        && declaration.body.iter().all(|statement| {
            matches!(statement,
            Statement::Method(method) if method.type_parameters.is_empty()
                && method.impl_contract.is_none() && method.body.is_some()
                && matches!(method.kind, MethodKind::Instance | MethodKind::Module))
        })
}

impl SourceEvaluator {
    pub(super) fn rollback_module(
        &mut self,
        module: ModuleId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        if self.open_target.is_some() || self.module_candidate.is_some() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let mut artifact = self.module_history_artifact(module, arguments)?;
        self.replay_module_artifact(module, &mut artifact)?;
        Ok(Value::Symbol(artifact.digest))
    }

    pub(super) fn replay_module_artifact(
        &mut self,
        module: ModuleId,
        artifact: &mut Artifact<ModuleDeclaration>,
    ) -> Result<(), EvaluationError> {
        self.validate_module_history(module, &artifact.target)?;
        let mut planner = Self::new_in_package(&artifact.context.package)?;
        planner.decorator_planning = true;
        planner.upgrade_state.active = self.upgrade_state.active;
        planner.source = artifact.context.source.clone();
        planner.inherit_planning_contracts(self);
        for definition in &artifact.decorators {
            planner.class(definition)?;
        }
        let mut plain = artifact.target.clone();
        plain.meta_deny.clear();
        for statement in &mut plain.body {
            if let Statement::Method(method) = statement {
                method.decorators.clear();
            }
        }
        planner.module(&plain)?;
        planner.rollback_state.context = Some(artifact.context.clone());
        planner.module_decorator_phases(&artifact.target)?;
        drop(planner);
        self.install_history_context(artifact)?;
        let source = std::mem::replace(&mut self.source, artifact.context.source.clone());
        let package = std::mem::replace(&mut self.package, artifact.context.package.clone());
        let context = self
            .rollback_state
            .context
            .replace(artifact.context.clone());
        let chains = self.wrapper_chains.clone();
        let metadata = self.decorator_metadata.len();
        let overrides = self.module_method_overrides.clone();
        artifact.target.reopen = true;
        artifact.target.meta_deny.clear();
        let outcome = self.decorated_module(&artifact.target);
        self.source = source;
        self.package = package;
        self.rollback_state.context = context;
        if let Err(error) = outcome {
            self.wrapper_chains = chains;
            self.decorator_metadata.truncate(metadata);
            self.module_method_overrides = overrides;
            return Err(error);
        }
        Ok(())
    }

    fn module_history_artifact(
        &self,
        module: ModuleId,
        arguments: &[Value],
    ) -> Result<Artifact<ModuleDeclaration>, EvaluationError> {
        let canonical = self
            .rollback_state
            .modules
            .get(&module)
            .and_then(|definitions| definitions.iter().find(|definition| !definition.reopen))
            .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
        let package = self
            .module_packages
            .get(&canonical.name)
            .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
        let major = self
            .package_contexts
            .packages
            .get(package)
            .map_or(self.api_major, |(major, _)| *major);
        let (digest, source) = self.history_source((package, major, &canonical.name), arguments)?;
        let parsed = iris_parser::parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        let mut targets = declarations(&parsed.program).filter(|declaration| match declaration {
            Declaration::Module(target) => target.name == canonical.name,
            Declaration::Class(target) => target.name == canonical.name,
            _ => false,
        });
        let Some(Declaration::Module(target)) = targets.next() else {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        };
        if targets.next().is_some() || target.reopen {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        let decorators =
            Self::history_decorators(&parsed.program, (&target.decorators, &target.body))?;
        Ok(Artifact {
            digest,
            target: target.clone(),
            decorators,
            context: Rc::new(BodyContext {
                package: package.clone(),
                api_major: major,
                source: source.into(),
                methods: HashMap::new(),
            }),
        })
    }

    pub(super) fn validate_module_history(
        &self,
        module: ModuleId,
        target: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        if !supported(target)
            || self
                .rollback_state
                .modules
                .get(&module)
                .is_none_or(|definitions| {
                    !self.upgrade_state.active
                        && definitions.iter().any(|definition| !supported(definition))
                })
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let active = self
            .runtime
            .registry()
            .active_module(module)
            .map_err(module_error)?;
        if !active.composition_edges().is_empty() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        for selector in self.selectors.values() {
            let Some(method) = active.method(*selector) else {
                continue;
            };
            if self.native_bodies.contains_key(&method.body().raw()) {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            let promise = self
                .bodies
                .get(&method.body().raw())
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let replacement = target
                .body
                .iter()
                .find_map(|statement| match statement {
                    Statement::Method(method) if method.selector == promise.selector => {
                        Some(method)
                    }
                    _ => None,
                })
                .ok_or(EvaluationError::TypeContractError)?;
            if replacement.kind != promise.kind
                || replacement.visibility != promise.visibility
                || !iris_syntax::method_signature_compatible(
                    replacement,
                    promise,
                    |source, target| source == target,
                )
            {
                return Err(EvaluationError::TypeContractError);
            }
        }
        for statement in &target.body {
            if let Statement::Method(method) = statement {
                let existing = self
                    .selectors
                    .get(&method.selector)
                    .and_then(|selector| active.method(*selector));
                if existing.is_none() {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "module_history_tests.rs"]
mod tests;
