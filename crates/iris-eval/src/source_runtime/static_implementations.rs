use super::{EvaluationError, SourceEvaluator, wrapper_generics};
use iris_runtime::ClassId;
use iris_syntax::{
    Declaration, ExportDeclaration, ImplDeclaration, Program, ProgramEntry, Statement,
    TypeExpression,
};
use std::collections::HashMap;

pub(super) fn prepare(
    program: &Program,
) -> Result<(Program, HashMap<String, Vec<ImplDeclaration>>), EvaluationError> {
    let mut prepared = Program::default();
    let mut implementations: HashMap<String, Vec<ImplDeclaration>> = HashMap::new();
    for entry in &program.entries {
        let mut entry = entry.clone();
        while let ProgramEntry::Declaration(Declaration::Export(export)) = &entry {
            match export.as_ref() {
                ExportDeclaration::Declaration(inner) => {
                    entry = ProgramEntry::Declaration(inner.as_ref().clone());
                }
                ExportDeclaration::Names(_) => break,
            }
        }
        match entry {
            ProgramEntry::Declaration(Declaration::Impl(implementation)) => {
                let target = type_name(&implementation.target)?;
                implementations
                    .entry(target.to_owned())
                    .or_default()
                    .push(implementation);
            }
            entry => prepared.entries.push(entry),
        }
    }
    for (target, contributions) in &implementations {
        let class = prepared
            .entries
            .iter_mut()
            .find_map(|entry| match entry {
                ProgramEntry::Declaration(Declaration::Class(class))
                    if !class.reopen && class.name == *target =>
                {
                    Some(class)
                }
                _ => None,
            })
            .ok_or(EvaluationError::TypeContractError)?;
        for contribution in contributions {
            let contract = type_name(&contribution.contract)?;
            if class
                .implements
                .iter()
                .any(|known| type_name(known).is_ok_and(|name| name == contract))
            {
                return Err(EvaluationError::TypeContractError);
            }
            class.implements.push(contribution.contract.clone());
            for method in &contribution.methods {
                let mut method = method.clone();
                method.impl_contract = Some(None);
                class.body.push(Statement::Method(method));
            }
        }
    }
    for entry in &prepared.entries {
        match entry {
            ProgramEntry::Declaration(declaration) => {
                prepared.declarations.push(declaration.clone())
            }
            ProgramEntry::Statement(statement) => prepared.statements.push(statement.clone()),
        }
    }
    Ok((prepared, implementations))
}

fn type_name(target: &TypeExpression) -> Result<&str, EvaluationError> {
    match target {
        TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => Ok(name),
        _ => Err(EvaluationError::TypeContractError),
    }
}

impl SourceEvaluator {
    pub(super) fn validate_static_implementations(
        &mut self,
        class: ClassId,
        name: &str,
    ) -> Result<(), EvaluationError> {
        let contributions = self
            .static_implementations
            .get(name)
            .cloned()
            .unwrap_or_default();
        for contribution in contributions {
            let contract = *self
                .contract_names
                .get(type_name(&contribution.contract)?)
                .ok_or(EvaluationError::TypeContractError)?;
            let requirements = self
                .contract_requirements
                .get(&contract)
                .cloned()
                .unwrap_or_default();
            if contribution
                .methods
                .iter()
                .any(|method| !requirements.contains(&method.selector) || method.body.is_none())
            {
                return Err(EvaluationError::TypeContractError);
            }
            let bindings = self.qualified_requirement_bindings(class, contract);
            if !self.decorator_contracts.contains(&contract)
                && let Some(requirements) = self.qualified_contracts.requirements.get(&contract)
            {
                for requirement in requirements {
                    let selector = self
                        .selectors
                        .get(&requirement.selector)
                        .copied()
                        .ok_or(EvaluationError::TypeContractError)?;
                    let method = self
                        .candidate_method_declaration(class, selector)
                        .ok_or(EvaluationError::TypeContractError)?;
                    let promise =
                        super::effective_contracts::substitute_requirement(requirement, &bindings);
                    if !iris_syntax::method_signature_compatible(
                        method,
                        &promise,
                        |source, target| self.nominal_subtype(source, target),
                    ) {
                        return Err(EvaluationError::TypeContractError);
                    }
                }
                continue;
            }
            for requirement in requirements {
                let selector = self.selector(&requirement);
                let method = self
                    .candidate_method_declaration(class, selector)
                    .ok_or(EvaluationError::TypeContractError)?;
                let arity = self
                    .contract_requirement_arities
                    .get(&(contract, requirement.clone()))
                    .copied()
                    .unwrap_or_default();
                if method.parameters.len() != arity {
                    return Err(EvaluationError::TypeContractError);
                }
                if let Some(required) = self
                    .contract_requirement_returns
                    .get(&(contract, requirement))
                    && method.return_type != Some(wrapper_generics::substitute(required, &bindings))
                {
                    return Err(EvaluationError::TypeContractError);
                }
            }
        }
        Ok(())
    }
}
