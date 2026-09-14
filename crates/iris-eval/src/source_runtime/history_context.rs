use super::rollback_artifact::Artifact;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{Method, MethodId, MethodOwner, Value};
use iris_syntax::{
    ClassDeclaration, Declaration, Decorator, ExportDeclaration, Program, Statement,
};
use std::collections::HashMap;
use std::rc::Rc;

pub(super) fn declarations(program: &Program) -> impl Iterator<Item = &Declaration> {
    program.declarations.iter().filter_map(|declaration| {
        let mut declaration = declaration;
        while let Declaration::Export(export) = declaration {
            match export.as_ref() {
                ExportDeclaration::Declaration(inner) => declaration = inner,
                _ => return None,
            }
        }
        Some(declaration)
    })
}

impl SourceEvaluator {
    pub(super) fn history_source(
        &self,
        owner: (&str, u64, &str),
        arguments: &[Value],
    ) -> Result<(String, &str), EvaluationError> {
        let (package, major, name) = owner;
        let (_, digest, source) = match arguments {
            [] => self.artifact.as_ref(),
            [Value::Integer(number)] => {
                let revision = number
                    .to_u64()
                    .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
                let mut matches = self.rollback_state.artifacts.iter().filter(|record| {
                    record.package_id == package
                        && record.api_major == major
                        && record.logical_owner == name
                        && record.revision == revision
                });
                let selected = matches.next();
                if matches.next().is_some() {
                    return Err(EvaluationError::RevisionArtifactUnavailable);
                }
                selected.map(|record| &record.artifact)
            }
            [Value::Symbol(_)] => None,
            _ => return Err(EvaluationError::ArgumentError),
        }
        .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
        let actual = iris_runtime::artifact_digest(source.as_bytes());
        if actual != digest.strip_prefix("b3:").unwrap_or(digest)
            || package != self.rollback_state.package
        {
            return Err(EvaluationError::RevisionArtifactUnavailable);
        }
        Ok((actual, source))
    }

    pub(super) fn history_decorators(
        program: &Program,
        target: (&[Decorator], &[Statement]),
    ) -> Result<Vec<ClassDeclaration>, EvaluationError> {
        let (program, _) = super::static_implementations::prepare(program)?;
        let mut decorators = Vec::new();
        for application in target.0.iter().chain(
            target
                .1
                .iter()
                .filter_map(|statement| match statement {
                    Statement::Method(method) => Some(method.decorators.iter()),
                    Statement::StoredProperty { decorators, .. } => Some(decorators.iter()),
                    _ => None,
                })
                .flatten(),
        ) {
            if decorators
                .iter()
                .any(|definition: &ClassDeclaration| definition.name == application.name)
            {
                continue;
            }
            let mut definitions =
                declarations(&program).filter_map(|declaration| match declaration {
                    Declaration::Class(class) if class.name == application.name => Some(class),
                    _ => None,
                });
            let definition = definitions
                .next()
                .ok_or(EvaluationError::RevisionArtifactUnavailable)?
                .clone();
            if definitions.next().is_some() || definition.reopen || definition.extends.is_some()
                || !definition.mixins.is_empty() || !definition.parameters.is_empty()
                || !definition.constraints.is_empty() || !definition.decorators.is_empty()
                || definition.body.iter().any(|statement| !matches!(statement, Statement::Method(method) if method.decorators.is_empty())) {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            decorators.push(definition);
        }
        Ok(decorators)
    }

    pub(super) fn install_history_context<T>(
        &mut self,
        artifact: &mut Artifact<T>,
    ) -> Result<(), EvaluationError> {
        let mut methods = HashMap::new();
        for definition in &artifact.decorators {
            let owner = self
                .class_name(&definition.name)?
                .ok_or(EvaluationError::RevisionArtifactUnavailable)?;
            if self.class_identity(owner)
                != (
                    artifact.context.package.as_str(),
                    artifact.context.api_major,
                )
            {
                return Err(EvaluationError::RevisionArtifactUnavailable);
            }
            for statement in &definition.body {
                if let Statement::Method(method) = statement {
                    let selector = self.selector(&method.selector);
                    let body = self.register_body(method.clone());
                    let identity = MethodId::new(self.next_body);
                    self.next_body += 1;
                    methods.insert(
                        (owner, selector),
                        Method::new(
                            identity,
                            MethodOwner::Class(owner),
                            selector,
                            body,
                            super::visibility(method),
                        ),
                    );
                }
            }
        }
        Rc::get_mut(&mut artifact.context)
            .ok_or(EvaluationError::UnsupportedConstruct)?
            .methods = methods;
        for method in artifact.context.methods.values() {
            self.rollback_state
                .bodies
                .insert(method.body().raw(), artifact.context.clone());
        }
        Ok(())
    }
}
