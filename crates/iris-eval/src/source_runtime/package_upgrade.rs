use super::decorated_module::module_error;
use super::upgrade_artifact::UpgradeOwner;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, RuntimeStructuralError, Selector, Value};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct UpgradeState {
    pub versions: Vec<crate::ResolvedPackageVersion>,
    pub sources: Vec<(String, String)>,
    pub active: bool,
    pub slots: HashMap<(ClassId, Selector), Value>,
}

impl SourceEvaluator {
    pub(crate) fn enter_package_upgrades(
        &mut self,
        versions: Vec<crate::ResolvedPackageVersion>,
        sources: &[(String, String)],
    ) {
        self.upgrade_state.versions = versions;
        self.upgrade_state.sources = sources.to_vec();
    }

    pub(super) fn upgrade_package(&mut self, target: &str) -> Result<Value, EvaluationError> {
        if self.upgrade_state.active
            || self.open_target.is_some()
            || self.module_candidate.is_some()
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        if self.upgrade_state.versions.is_empty() {
            self.upgrade_state.active = true;
            let outcome = self.upgrade_hook(target);
            self.upgrade_state.active = false;
            self.upgrade_state.slots.clear();
            return outcome.and(Err(EvaluationError::RevisionArtifactUnavailable));
        }
        let mut matches = self.upgrade_state.versions.iter().filter(|record| {
            record.package_id == self.package
                && record.api_major == self.api_major
                && record.version == target
        });
        let record = matches
            .next()
            .ok_or(EvaluationError::RevisionArtifactUnavailable)?
            .clone();
        if matches.next().is_some()
            || iris_runtime::artifact_digest(record.artifact.2.as_bytes())
                != record
                    .artifact
                    .1
                    .strip_prefix("b3:")
                    .unwrap_or(&record.artifact.1)
        {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        let mut owners = self.upgrade_owners(&record)?;
        let chains = self.wrapper_chains.clone();
        let metadata = self.decorator_metadata.len();
        let overrides = self.module_method_overrides.clone();
        self.upgrade_state.active = true;
        let outcome = (|| {
            for owner in &owners {
                match owner {
                    UpgradeOwner::Class(class, artifact) => {
                        self.validate_rollback_spine(*class, &artifact.target)?
                    }
                    UpgradeOwner::Module(module, artifact) => {
                        self.validate_module_history(*module, &artifact.target)?
                    }
                }
            }
            for owner in &mut owners {
                match owner {
                    UpgradeOwner::Class(class, artifact) => {
                        self.replay_class_artifact(*class, artifact)?
                    }
                    UpgradeOwner::Module(module, artifact) => {
                        self.replay_module_artifact(*module, artifact)?
                    }
                }
            }
            let value = self.upgrade_hook(target)?;
            for owner in &owners {
                if let UpgradeOwner::Class(class, _) = owner {
                    self.validate_candidate_contracts(*class)?;
                }
            }
            self.runtime
                .commit_structural_group()
                .map_err(|error| match error {
                    RuntimeStructuralError::Class(error) => EvaluationError::Class(error),
                    RuntimeStructuralError::Module(error) => module_error(error),
                })?;
            Ok(value)
        })();
        self.upgrade_state.active = false;
        match outcome {
            Ok(value) => {
                for ((class, slot), value) in std::mem::take(&mut self.upgrade_state.slots) {
                    self.runtime
                        .assign_class_raw_ivar(class, slot, value)
                        .map_err(EvaluationError::Construction)?;
                }
                for owner in owners {
                    if let UpgradeOwner::Module(module, _) = owner {
                        let active = self
                            .runtime
                            .registry()
                            .active_module(module)
                            .map_err(module_error)?;
                        for selector in self.selectors.values() {
                            if let Some(method) = active.method(*selector) {
                                self.module_methods.insert((module, *selector), method);
                            }
                        }
                    }
                }
                self.package_version = Some(target.into());
                self.upgrade_state.sources = vec![(record.artifact.0, record.artifact.2)];
                Ok(value)
            }
            Err(error) => {
                self.runtime.roll_back_group();
                self.upgrade_state.slots.clear();
                self.wrapper_chains = chains;
                self.decorator_metadata.truncate(metadata);
                self.module_method_overrides = overrides;
                Err(error)
            }
        }
    }

    fn upgrade_hook(&mut self, target: &str) -> Result<Value, EvaluationError> {
        let Some(module) = self.module_names.get("Upgrade").copied() else {
            return Ok(Value::Nil);
        };
        let hook = self.selector("upgrade");
        let method = match self.runtime.registry().staged_module(module) {
            Ok(candidate) => candidate.method(hook),
            Err(_) if self.upgrade_state.versions.is_empty() => {
                self.runtime.registry().module_method(module, hook)
            }
            Err(error) => return Err(module_error(error)),
        };
        let Some(method) = method else {
            return Ok(Value::Nil);
        };
        let main = self.module_main(module)?;
        let receiver = Value::Object(
            self.runtime
                .allocate(main)
                .map_err(EvaluationError::Construction)?,
        );
        self.invoke_method(
            method,
            receiver,
            &[
                Value::Symbol(self.package_version.clone().unwrap_or_default()),
                Value::Symbol(target.into()),
            ],
        )
    }
}

#[cfg(test)]
#[path = "package_upgrade_tests.rs"]
mod tests;
