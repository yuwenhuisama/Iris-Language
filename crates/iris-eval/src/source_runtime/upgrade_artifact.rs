use super::history_context::declarations;
use super::rollback_artifact::{Artifact, BodyContext};
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, ModuleId};
use iris_syntax::{Declaration, ModuleDeclaration, Statement};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub(super) enum UpgradeOwner {
    Class(ClassId, Artifact),
    Module(ModuleId, Artifact<ModuleDeclaration>),
}

impl SourceEvaluator {
    pub(super) fn upgrade_owners(
        &self,
        record: &crate::ResolvedPackageVersion,
    ) -> Result<Vec<UpgradeOwner>, EvaluationError> {
        let parsed = iris_parser::parse(&record.artifact.2);
        if !parsed.program_accepted || !parsed.program.statements.is_empty() {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        if declarations(&parsed.program).count() != parsed.program.declarations.len() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let (program, _) = super::static_implementations::prepare(&parsed.program)?;
        let mut previous = HashMap::new();
        for (_, source) in &self.upgrade_state.sources {
            let parsed = iris_parser::parse(source);
            if declarations(&parsed.program).count() != parsed.program.declarations.len() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            if !parsed.program.statements.is_empty() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            let (previous_program, _) = super::static_implementations::prepare(&parsed.program)?;
            for declaration in declarations(&previous_program) {
                let name = match declaration {
                    Declaration::Class(class) if !class.reopen => &class.name,
                    Declaration::Module(module) if !module.reopen => &module.name,
                    Declaration::Class(_) | Declaration::Module(_) => continue,
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                if previous.insert(name.clone(), declaration.clone()).is_some() {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
            }
        }
        let mut owners = Vec::new();
        let mut names = HashSet::new();
        for declaration in declarations(&program) {
            let context = Rc::new(BodyContext {
                package: self.package.clone(),
                api_major: self.api_major,
                source: record.artifact.2.clone(),
                methods: HashMap::new(),
            });
            match declaration {
                Declaration::Class(target) if !target.reopen && target.decorators.is_empty() => {
                    let Some(Declaration::Class(old)) = previous.get(&target.name) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    if !names.insert(target.name.clone())
                        || old.meta_deny != target.meta_deny
                        || !old.decorators.is_empty()
                    {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let class = self
                        .class_name(&target.name)?
                        .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
                    if self
                        .qualified_methods
                        .keys()
                        .any(|(owner, _, _)| *owner == class)
                    {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let state = |body: &[Statement]| {
                        body.iter()
                            .filter(|statement| !matches!(statement, Statement::Method(_)))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    if state(&old.body) != state(&target.body)
                        || state(&target.body).iter().any(|statement| {
                            !matches!(
                                statement,
                                Statement::StoredProperty {
                                    class_level: true,
                                    ..
                                }
                            )
                        })
                        || target.body.len() != old.body.len()
                    {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let mut target = target.clone();
                    target
                        .body
                        .retain(|statement| matches!(statement, Statement::Method(_)));
                    owners.push(UpgradeOwner::Class(
                        class,
                        Artifact {
                            digest: record.artifact.1.clone(),
                            context,
                            decorators: Self::history_decorators(
                                &program,
                                (&target.decorators, &target.body),
                            )?,
                            target,
                        },
                    ));
                }
                Declaration::Module(target) if !target.reopen => {
                    let Some(Declaration::Module(old)) = previous.get(&target.name) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    if !names.insert(target.name.clone())
                        || old.meta_deny != target.meta_deny
                        || old.mixins != target.mixins
                        || old.parameters != target.parameters
                        || old.constraints != target.constraints
                        || old.contract_for != target.contract_for
                        || old.decorators != target.decorators
                    {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let state = |body: &[Statement]| {
                        body.iter()
                            .filter(|statement| !matches!(statement, Statement::Method(_)))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    if state(&old.body) != state(&target.body)
                        || state(&target.body).iter().any(|statement| {
                            !matches!(
                                statement,
                                Statement::StoredProperty {
                                    class_level: true,
                                    ..
                                } | Statement::Expression(_)
                                    | Statement::Binding { .. }
                                    | Statement::Raise(_)
                            )
                        })
                    {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let mut target = target.clone();
                    target
                        .body
                        .retain(|statement| matches!(statement, Statement::Method(_)));
                    let module = *self
                        .module_names
                        .get(&target.name)
                        .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
                    owners.push(UpgradeOwner::Module(
                        module,
                        Artifact {
                            digest: record.artifact.1.clone(),
                            context,
                            decorators: Self::history_decorators(
                                &program,
                                (&target.decorators, &target.body),
                            )?,
                            target,
                        },
                    ));
                }
                Declaration::Class(_) | Declaration::Module(_) => {}
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        if names.len() != previous.len() || owners.is_empty() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        for definition in &self.decorator_definitions {
            if let Some(class) = self.class_name(&definition.name)?
                && self.class_identity(class) == (self.package.as_str(), self.api_major)
                && !names.contains(&definition.name)
            {
                return Err(EvaluationError::UnsupportedConstruct);
            }
        }
        if self
            .module_packages
            .iter()
            .any(|(name, package)| package == &self.package && !names.contains(name))
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        Ok(owners)
    }
}
