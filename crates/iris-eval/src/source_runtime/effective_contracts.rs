use super::{EvaluationError, SourceEvaluator, wrapper_generics};
use iris_runtime::{ContractId, MetaCapabilities};
use iris_syntax::{ContractDeclaration, MethodDeclaration, Statement, TypeExpression};
use std::collections::HashMap;

pub(super) fn substitute_requirement(
    requirement: &MethodDeclaration,
    bindings: &HashMap<String, TypeExpression>,
) -> MethodDeclaration {
    let mut requirement = requirement.clone();
    let bindings = bindings
        .iter()
        .filter(|(name, _)| !requirement.type_parameters.contains(name))
        .map(|(name, annotation)| (name.clone(), annotation.clone()))
        .collect();
    for parameter in &mut requirement.parameters {
        parameter.annotation = parameter
            .annotation
            .as_ref()
            .map(|annotation| wrapper_generics::substitute(annotation, &bindings));
    }
    requirement.return_type = requirement
        .return_type
        .as_ref()
        .map(|annotation| wrapper_generics::substitute(annotation, &bindings));
    requirement
}

pub(super) struct EffectiveContract {
    pub requirements: Vec<MethodDeclaration>,
    pub parents: Vec<(ContractId, Vec<TypeExpression>)>,
    pub capabilities: MetaCapabilities,
}

impl SourceEvaluator {
    pub(super) fn effective_contract(
        &self,
        declaration: &ContractDeclaration,
    ) -> Result<EffectiveContract, EvaluationError> {
        let mut effective = EffectiveContract {
            requirements: Vec::new(),
            parents: Vec::new(),
            capabilities: super::meta_capabilities(&declaration.meta_deny)?,
        };
        let mut ancestors = HashMap::new();
        for parent in &declaration.parents {
            let (name, arguments) = match parent {
                TypeExpression::Name(name) => (name, Vec::new()),
                TypeExpression::Generic { name, arguments } => (name, arguments.clone()),
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            let parent = *self
                .contract_names
                .get(name)
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let parameters = self.qualified_contracts.parameters.get(&parent);
            if parameters.is_some_and(|parameters| parameters.len() != arguments.len()) {
                return Err(EvaluationError::TypeContractError);
            }
            let bindings = parameters
                .into_iter()
                .flatten()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect();
            self.validate_parent_arguments((parent, arguments.clone()), &mut ancestors)?;
            effective.parents.push((parent, arguments));
            if let Some(policy) = self.contract_capabilities.get(&parent) {
                effective.capabilities = effective.capabilities.narrowed_by(*policy);
            }
            if let Some(requirements) = self.qualified_contracts.requirements.get(&parent) {
                for requirement in requirements {
                    self.merge_requirement(
                        &mut effective.requirements,
                        substitute_requirement(requirement, &bindings),
                    )?;
                }
            }
        }
        for statement in &declaration.body {
            if let Statement::Method(method) = statement
                && method.body.is_none()
            {
                self.merge_requirement(&mut effective.requirements, method.clone())?;
            }
        }
        Ok(effective)
    }

    fn validate_parent_arguments(
        &self,
        (contract, arguments): (ContractId, Vec<TypeExpression>),
        ancestors: &mut HashMap<ContractId, Vec<TypeExpression>>,
    ) -> Result<(), EvaluationError> {
        if let Some(known) = ancestors.get(&contract) {
            return if *known == arguments {
                Ok(())
            } else {
                Err(EvaluationError::TypeContractError)
            };
        }
        let bindings = self
            .qualified_contracts
            .parameters
            .get(&contract)
            .into_iter()
            .flatten()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect();
        ancestors.insert(contract, arguments);
        for (parent, arguments) in self
            .qualified_contracts
            .parents
            .get(&contract)
            .into_iter()
            .flatten()
        {
            let arguments = arguments
                .iter()
                .map(|argument| wrapper_generics::substitute(argument, &bindings))
                .collect();
            self.validate_parent_arguments((*parent, arguments), ancestors)?;
        }
        Ok(())
    }

    fn merge_requirement(
        &self,
        requirements: &mut Vec<MethodDeclaration>,
        incoming: MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        let Some(existing) = requirements
            .iter_mut()
            .find(|requirement| requirement.selector == incoming.selector)
        else {
            requirements.push(incoming);
            return Ok(());
        };
        if iris_syntax::method_signature_compatible(&incoming, existing, |source, target| {
            self.nominal_subtype(source, target)
        }) {
            *existing = incoming;
        } else if !iris_syntax::method_signature_compatible(
            existing,
            &incoming,
            |source, target| self.nominal_subtype(source, target),
        ) {
            return Err(EvaluationError::TypeContractError);
        }
        Ok(())
    }
}
