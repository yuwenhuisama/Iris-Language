use iris_syntax::{ClassDeclaration, MethodKind, ParameterCategory, Statement, TypeExpression};

use super::{CompileError, Contract, ContractRequirement};

pub(super) fn decorator_contracts() -> impl Iterator<Item = Contract> {
    crate::core_names::DECORATOR_CONTRACTS
        .iter()
        .map(|name| Contract {
            parent_arguments: Vec::new(),
            type_parameters: Vec::new(),
            signatures: [("plan", 2, "Plan"), ("transform", 3, "Transformation")]
                .into_iter()
                .map(|(selector, arity, result)| {
                    super::effective_contracts::signature(&ContractRequirement {
                        selector: selector.into(),
                        arity,
                        parameter_types: vec![None; arity],
                        return_type: Some(TypeExpression::Name(result.into())),
                    })
                })
                .collect(),
            core_identity: iris_runtime::core_contract_id(name),
            name: (*name).into(),
            parents: Vec::new(),
            meta_deny: Vec::new(),
            requirements: [("plan", 2, "Plan"), ("transform", 3, "Transformation")]
                .into_iter()
                .map(|(selector, arity, result)| ContractRequirement {
                    selector: selector.into(),
                    arity,
                    parameter_types: vec![None; arity],
                    return_type: Some(TypeExpression::Name(result.into())),
                })
                .collect(),
        })
}

impl super::Program {
    pub(crate) fn contract_identity(&self, index: usize) -> iris_runtime::ContractId {
        if let Some(link) = &self.link {
            return link.contracts[index];
        }
        self.contracts[index].core_identity.unwrap_or_else(|| {
            iris_runtime::ContractId::new(
                self.contracts[..index]
                    .iter()
                    .filter(|contract| contract.core_identity.is_none())
                    .count() as u64,
            )
        })
    }

    pub(crate) fn contract_index(&self, identity: iris_runtime::ContractId) -> Option<usize> {
        (0..self.contracts.len()).find(|index| self.contract_identity(*index) == identity)
    }
}

pub(super) fn validate(
    class: &ClassDeclaration,
    contracts: &[Contract],
    conformances: &[usize],
) -> Result<(), CompileError> {
    for contract in conformances.iter().map(|index| &contracts[*index]) {
        if !crate::core_names::DECORATOR_CONTRACTS.contains(&contract.name.as_str()) {
            continue;
        }
        let canonical = decorator_contracts()
            .find(|known| known.name == contract.name)
            .ok_or_else(|| CompileError::new("contract signature clash"))?;
        for requirement in &canonical.requirements {
            let admitted = class.body.iter().any(|statement| {
                matches!(statement, Statement::Method(method)
                    if method.selector == requirement.selector
                    && method.kind == MethodKind::Instance
                    && !method.is_async
                    && method.visibility == iris_syntax::Visibility::Public
                    && method.type_parameters.is_empty()
                    && method.parameters.len() == requirement.arity
                    && method.parameters.iter().all(|parameter|
                        parameter.category == ParameterCategory::Positional && parameter.default.is_none()
                        && parameter.annotation.as_ref().is_none_or(|annotation| matches!(annotation, TypeExpression::Name(name) if name == "Object")))
                    && method.return_type == requirement.return_type)
            });
            if !admitted {
                return Err(CompileError::new("contract signature clash"));
            }
        }
    }
    Ok(())
}
