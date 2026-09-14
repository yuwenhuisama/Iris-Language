use super::decorator_phases::PhaseTarget;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{DecoratorKind, DecoratorReason};
use iris_runtime::{ClassId, Value};
use iris_syntax::{MethodKind, Statement};

impl SourceEvaluator {
    pub(super) fn rollback_class(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let mut artifact = self.rollback_artifact(class, arguments)?;
        self.replay_class_artifact(class, &mut artifact)?;
        Ok(Value::Symbol(artifact.digest))
    }

    pub(super) fn replay_class_artifact(
        &mut self,
        class: ClassId,
        artifact: &mut super::rollback_artifact::Artifact,
    ) -> Result<(), EvaluationError> {
        let reason = if self.upgrade_state.active {
            DecoratorReason::Upgrade
        } else {
            DecoratorReason::Rollback
        };
        self.validate_rollback_spine(class, &artifact.target)?;
        let methods = super::rollback_artifact::replay_methods(&artifact.target.body)?;
        if self.runtime.registry().is_staging(class) || self.open_target.is_some() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let mut planner = Self::new_in_package(&artifact.context.package)?;
        planner.decorator_planning = true;
        planner.source = artifact.context.source.clone();
        planner.inherit_planning_contracts(self);
        for definition in &artifact.decorators {
            planner.class(definition)?;
        }
        let mut plain = artifact.target.clone();
        plain.decorators.clear();
        plain.meta_deny.clear();
        for statement in &mut plain.body {
            if let Statement::Method(method) = statement {
                method.decorators.clear();
            } else if let Statement::StoredProperty { decorators, .. } = statement {
                decorators.clear();
            }
        }
        planner.class(&plain)?;
        let planned = planner
            .class_name(&plain.name)?
            .ok_or(EvaluationError::NameError)?;
        for decorator in &artifact.target.decorators {
            let metadata = planner.class_decorator_metadata(planned)?;
            planner.execute_decorator_phase(
                decorator,
                PhaseTarget {
                    kind: DecoratorKind::Class,
                    reason,
                    metadata,
                    candidate: None,
                },
            )?;
        }
        planner.plan_class_members(planned, (&artifact.target.body, reason))?;
        for method in &methods {
            if matches!(method.impl_contract, Some(Some(_))) {
                continue;
            }
            let selector = self.selector(&method.selector);
            let active = self
                .runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?;
            let capability = match method.kind {
                MethodKind::Property => {
                    if active.methods().contains_key(&selector) {
                        iris_runtime::Capability::PropertyBody
                    } else {
                        iris_runtime::Capability::PropertySet
                    }
                }
                MethodKind::Instance if active.methods().contains_key(&selector) => {
                    iris_runtime::Capability::MethodBody
                }
                MethodKind::Class if active.singleton_methods().contains_key(&selector) => {
                    iris_runtime::Capability::MethodBody
                }
                MethodKind::Instance | MethodKind::Class => iris_runtime::Capability::MethodSet,
                MethodKind::Module => return Err(EvaluationError::UnsupportedConstruct),
            };
            self.runtime
                .registry()
                .require_candidate_meta_capability(class, capability)
                .map_err(EvaluationError::Class)?;
        }
        self.install_history_context(artifact)?;
        let source = std::mem::replace(&mut self.source, artifact.context.source.clone());
        let package = std::mem::replace(&mut self.package, artifact.context.package.clone());
        let context = self
            .rollback_state
            .context
            .replace(artifact.context.clone());
        let chains = self.wrapper_chains.clone();
        let qualified = self.qualified_methods.clone();
        let properties = self.property_methods.clone();
        let singletons = self.singleton_identities.clone();
        let declarations = self.singleton_declarations.clone();
        let metadata = self.decorator_metadata.len();
        let outcome = (|| {
            self.runtime
                .registry_mut()
                .begin_transaction(class)
                .map_err(EvaluationError::Class)?;
            self.stage_history_initializers(class, &artifact.target.body)?;
            for method in &methods {
                let mut method = method.clone();
                let selector = self.selector(&method.selector);
                let active = self
                    .runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?;
                let table = match method.kind {
                    MethodKind::Class => active.singleton_methods(),
                    MethodKind::Instance | MethodKind::Property => active.methods(),
                    MethodKind::Module => return Err(EvaluationError::UnsupportedConstruct),
                };
                method.is_override = table.contains_key(&selector);
                self.class_method_as(
                    class,
                    false,
                    true,
                    &method,
                    method.kind == MethodKind::Property,
                )?;
            }
            self.publish_decorators(class, &artifact.target.decorators, reason)?;
            for statement in &artifact.target.body {
                if let Statement::Method(method) = statement {
                    self.transform_method_decorators(class, method, reason)?;
                } else if let Some(method) =
                    super::candidate_declarations::stored_decorator_declaration(statement)
                {
                    self.transform_method_decorators(class, &method, reason)?;
                }
            }
            self.validate_candidate_contracts(class)?;
            if self.upgrade_state.active {
                return Ok(());
            }
            self.runtime
                .registry_mut()
                .commit_transaction(class)
                .map_err(EvaluationError::Class)
        })();
        self.source = source;
        self.package = package;
        self.rollback_state.context = context;
        if let Err(error) = outcome {
            self.runtime.registry_mut().roll_back_transaction(class);
            self.wrapper_chains = chains;
            self.qualified_methods = qualified;
            self.property_methods = properties;
            self.singleton_identities = singletons;
            self.singleton_declarations = declarations;
            self.decorator_metadata.truncate(metadata);
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "accessor_history_tests.rs"]
mod accessor_tests;

#[cfg(test)]
#[path = "qualified_history_tests.rs"]
mod qualified_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn consumes_one_commit_when_historical_rebuild_succeeds() {
        let mut given = crate::Session::new().unwrap();
        let source = "class Target { public fun value() -> Integer { 1 } }; 0";
        given.evaluate(source).unwrap();
        given.evaluator.enter_artifact(None);
        given
            .evaluator
            .enter_revision_artifacts(vec![crate::RevisionArtifact {
                package_id: super::super::LOCAL_PACKAGE.into(),
                api_major: 1,
                logical_owner: "Target".into(),
                revision: 1,
                artifact: (
                    "opaque".into(),
                    iris_runtime::artifact_digest(source.as_bytes()),
                    source.into(),
                ),
            }]);
        let target = given.evaluator.class_name("Target").unwrap().unwrap();
        let before = given
            .evaluator
            .runtime
            .registry()
            .active(target)
            .unwrap()
            .clone();
        let old = given
            .evaluator
            .runtime
            .registry()
            .dispatch(target, given.evaluator.selectors["value"])
            .unwrap();
        given.evaluate("Target.rollback(1)").unwrap();
        let after = given.evaluator.runtime.registry().active(target).unwrap();
        assert_eq!(after.commit_id(), before.commit_id() + 1);
        assert_eq!(after.number(), before.number() + 1);
        assert_ne!(
            given
                .evaluator
                .runtime
                .registry()
                .dispatch(target, given.evaluator.selectors["value"])
                .unwrap(),
            old
        );
    }
}
