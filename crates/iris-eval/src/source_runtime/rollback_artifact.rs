use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, Value};
use iris_syntax::{ClassDeclaration, Declaration, ExportDeclaration, Statement, TypeExpression};
use std::collections::HashMap;
use std::rc::Rc;

#[path = "stored_history.rs"]
mod stored_history;
pub(super) use stored_history::replay_methods;

#[path = "qualified_history.rs"]
mod qualified_history;

#[derive(Default)]
pub(super) struct RollbackState {
    pub package: String,
    pub artifacts: Vec<crate::RevisionArtifact>,
    pub context: Option<Rc<BodyContext>>,
    pub bodies: HashMap<u64, Rc<BodyContext>>,
    pub modules: HashMap<iris_runtime::ModuleId, Vec<iris_syntax::ModuleDeclaration>>,
}

pub(super) struct BodyContext {
    pub package: String,
    pub api_major: u64,
    pub source: String,
    pub methods: HashMap<(ClassId, iris_runtime::Selector), iris_runtime::Method>,
}

pub(super) struct Artifact<T = ClassDeclaration> {
    pub digest: String,
    pub context: Rc<BodyContext>,
    pub target: T,
    pub decorators: Vec<ClassDeclaration>,
}

impl SourceEvaluator {
    pub(crate) fn enter_revision_artifacts(&mut self, artifacts: Vec<crate::RevisionArtifact>) {
        self.rollback_state.artifacts = artifacts;
    }

    pub(super) fn rollback_artifact(
        &self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Artifact, EvaluationError> {
        let canonical = self
            .decorator_definitions
            .iter()
            .find(|declaration| {
                !declaration.reopen
                    && self.class_name(&declaration.name).ok().flatten() == Some(class)
            })
            .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
        let (package, major) = self.class_identity(class);
        let (actual, source) = self.history_source((package, major, &canonical.name), arguments)?;
        let parsed = iris_parser::parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        let (program, _) = super::static_implementations::prepare(&parsed.program)?;
        let declarations: Vec<_> = program
            .declarations
            .iter()
            .filter_map(|declaration| {
                let mut declaration = declaration;
                while let Declaration::Export(export) = declaration {
                    match export.as_ref() {
                        ExportDeclaration::Declaration(inner) => declaration = inner,
                        _ => return None,
                    }
                }
                match declaration {
                    Declaration::Class(class) => Some(class),
                    _ => None,
                }
            })
            .collect();
        let mut targets = declarations
            .iter()
            .filter(|declaration| declaration.name == canonical.name);
        let target = (*targets
            .next()
            .ok_or(EvaluationError::RevisionArtifactUnavailable)?)
        .clone();
        if targets.next().is_some() || target.reopen {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        let decorators = Self::history_decorators(&program, (&target.decorators, &target.body))?;
        Ok(Artifact {
            digest: actual,
            context: Rc::new(BodyContext {
                package: self.rollback_state.package.clone(),
                api_major: major,
                source: source.to_owned(),
                methods: HashMap::new(),
            }),
            target,
            decorators,
        })
    }

    pub(super) fn validate_rollback_spine(
        &self,
        class: ClassId,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        let methods = replay_methods(&declaration.body)?;
        if self.generic_definitions.contains(&class)
            || !declaration.parameters.is_empty()
            || !declaration.constraints.is_empty()
            || !declaration.mixins.is_empty()
            || methods.iter().any(|method| {
                !((method.type_parameters.is_empty()
                    || matches!(
                        method.kind,
                        iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class
                    ))
                    && match &method.impl_contract {
                        None | Some(None) => true,
                        Some(Some(_)) => {
                            method.kind == iris_syntax::MethodKind::Instance
                                && method.type_parameters.is_empty()
                        }
                    }
                    && matches!(
                        method.kind,
                        iris_syntax::MethodKind::Instance
                            | iris_syntax::MethodKind::Property
                            | iris_syntax::MethodKind::Class
                    ))
            })
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let superclass = match &declaration.extends {
            None => self.class_name("Object")?,
            Some(TypeExpression::Name(name)) => Some(
                self.class_name(name)?
                    .ok_or(EvaluationError::RevisionArtifactUnavailable)?,
            ),
            Some(_) => return Err(EvaluationError::UnsupportedConstruct),
        };
        if self.static_superclasses.get(&class).copied().flatten() != superclass {
            return Err(EvaluationError::TypeContractError);
        }
        for contract in self.class_contracts.get(&class).into_iter().flatten() {
            if !declaration.implements.iter().any(|target| matches!(target, TypeExpression::Name(name) if self.contract_names.get(name) == Some(contract))) {
                return Err(EvaluationError::TypeContractError);
            }
        }
        if declaration.implements.iter().any(|target| !matches!(target, TypeExpression::Name(name) if self.contract_names.get(name).is_some_and(|contract| self.class_contracts.get(&class).is_some_and(|contracts| contracts.contains(contract))))) {
            return Err(EvaluationError::TypeContractError);
        }
        self.validate_qualified_history(class, &methods)?;
        let active = self
            .runtime
            .registry()
            .active(class)
            .map_err(EvaluationError::Class)?;
        self.validate_history_storage(class, declaration)?;
        if !active.modules().is_empty()
            || !active.class_vars().is_empty()
            || (!self.upgrade_state.active
                && self
                    .class_level_properties
                    .get(&class)
                    .is_some_and(|slots| !slots.is_empty()))
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        for (selector, identity) in active.methods().iter().chain(active.singleton_methods()) {
            if active.methods().get(selector) == Some(identity) && self.selectors.get("to_bool") == Some(selector) && !self.decorator_definitions.iter().filter(|definition| self.class_name(&definition.name).ok().flatten() == Some(class)).any(|definition| definition.body.iter().any(|statement| matches!(statement, Statement::Method(method) if method.selector == "to_bool"))) { continue; }
            let method = self
                .runtime
                .registry()
                .method_by_id(*identity)
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            if self.native_bodies.contains_key(&method.body().raw()) {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            let promise = self
                .bodies
                .get(&method.body().raw())
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let replacement = methods
                .iter()
                .rev()
                .find(|method| {
                    method.selector == promise.selector
                        && method.kind == promise.kind
                        && !matches!(method.impl_contract, Some(Some(_)))
                })
                .ok_or(EvaluationError::TypeContractError)?;
            if !iris_syntax::method_signature_compatible(replacement, promise, |source, target| {
                self.nominal_subtype(source, target)
            }) || replacement.visibility != promise.visibility
            {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }
}
