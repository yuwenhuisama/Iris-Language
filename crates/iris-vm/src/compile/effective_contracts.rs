use super::{Class, CompileError, Contract, ContractRequirement};
use iris_syntax::{ContractDeclaration, MethodDeclaration, Statement, TypeExpression};

pub(crate) fn substitute_requirement(
    requirement: &MethodDeclaration,
    bindings: &[(String, TypeExpression)],
) -> MethodDeclaration {
    let mut selected = requirement.clone();
    let bindings: Vec<_> = bindings
        .iter()
        .filter(|(name, _)| !requirement.type_parameters.contains(name))
        .cloned()
        .collect();
    for parameter in &mut selected.parameters {
        parameter.annotation = parameter
            .annotation
            .as_ref()
            .map(|annotation| super::qualified::substitute(annotation, &bindings));
    }
    selected.return_type = selected
        .return_type
        .as_ref()
        .map(|annotation| super::qualified::substitute(annotation, &bindings));
    selected
}

pub(super) fn signature(requirement: &ContractRequirement) -> MethodDeclaration {
    MethodDeclaration {
        decorators: Vec::new(),
        is_async: false,
        is_override: false,
        impl_contract: None,
        kind: iris_syntax::MethodKind::Instance,
        selector: requirement.selector.clone(),
        type_parameters: Vec::new(),
        parameters: requirement
            .parameter_types
            .iter()
            .enumerate()
            .map(|(index, annotation)| iris_syntax::Parameter {
                name: format!("argument{index}"),
                category: iris_syntax::ParameterCategory::Positional,
                annotation: annotation
                    .as_ref()
                    .map(|name| TypeExpression::Name(name.clone())),
                default: None,
            })
            .collect(),
        return_type: requirement.return_type.clone(),
        visibility: iris_syntax::Visibility::Public,
        body: None,
    }
}

fn requirement(signature: &MethodDeclaration) -> ContractRequirement {
    ContractRequirement {
        selector: signature.selector.clone(),
        arity: signature.parameters.len(),
        parameter_types: signature
            .parameters
            .iter()
            .map(|parameter| match &parameter.annotation {
                Some(TypeExpression::Name(name)) => Some(name.clone()),
                _ => None,
            })
            .collect(),
        return_type: signature.return_type.clone(),
    }
}

pub(super) fn collect(
    declaration: &ContractDeclaration,
    contracts: &mut Vec<Contract>,
) -> Result<(), CompileError> {
    let mut effective = Contract {
        parent_arguments: Vec::new(),
        type_parameters: declaration.parameters.clone(),
        signatures: Vec::new(),
        core_identity: None,
        parents: Vec::new(),
        name: declaration.name.clone(),
        requirements: Vec::new(),
        meta_deny: declaration.meta_deny.clone(),
    };
    for parent in &declaration.parents {
        let (name, arguments) = match parent {
            TypeExpression::Name(name) => (name, &[][..]),
            TypeExpression::Generic { name, arguments } => (name, arguments.as_slice()),
            _ => return Err(CompileError::new("contract declaration form")),
        };
        let parent = contracts
            .iter()
            .rfind(|contract| contract.name == *name)
            .ok_or_else(|| CompileError::new("contract parent unbound"))?;
        if parent.type_parameters.len() != arguments.len() {
            return Err(CompileError::new("contract signature clash"));
        }
        let bindings: Vec<_> = parent
            .type_parameters
            .iter()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect();
        for (name, arguments) in std::iter::once((name.clone(), arguments.to_vec())).chain(
            parent.parent_arguments.iter().map(|(name, arguments)| {
                (
                    name.clone(),
                    arguments
                        .iter()
                        .map(|argument| super::qualified::substitute(argument, &bindings))
                        .collect(),
                )
            }),
        ) {
            match effective
                .parent_arguments
                .iter()
                .find(|(known, _)| *known == name)
            {
                Some((_, known)) if *known != arguments => {
                    return Err(CompileError::new("contract signature clash"));
                }
                Some(_) => {}
                None => {
                    effective.parents.push(name.clone());
                    effective.parent_arguments.push((name, arguments));
                }
            }
        }
        for denied in &parent.meta_deny {
            if !effective.meta_deny.contains(denied) {
                effective.meta_deny.push(denied.clone());
            }
        }
        for signature in &parent.signatures {
            merge(
                &mut effective.signatures,
                substitute_requirement(signature, &bindings),
                contracts,
            )?;
        }
    }
    for statement in &declaration.body {
        let Statement::Method(method) = statement else {
            return Err(CompileError::new("contract body"));
        };
        if method.kind != iris_syntax::MethodKind::Instance
            || method.impl_contract.is_some()
            || !method.decorators.is_empty()
        {
            return Err(CompileError::new("contract requirement form"));
        }
        merge(
            &mut effective.signatures,
            super::method_metadata::bodyless(method),
            contracts,
        )?;
    }
    effective.requirements = effective.signatures.iter().map(requirement).collect();
    contracts.push(effective);
    Ok(())
}

fn merge(
    requirements: &mut Vec<MethodDeclaration>,
    incoming: MethodDeclaration,
    contracts: &[Contract],
) -> Result<(), CompileError> {
    let Some(existing) = requirements
        .iter_mut()
        .find(|known| known.selector == incoming.selector)
    else {
        requirements.push(incoming);
        return Ok(());
    };
    let subtype = |source: &str, target: &str| {
        contracts.iter().any(|contract| {
            contract.name == source && contract.parents.iter().any(|parent| parent == target)
        })
    };
    if iris_syntax::method_signature_compatible(&incoming, existing, subtype) {
        *existing = incoming;
    } else if !iris_syntax::method_signature_compatible(existing, &incoming, subtype) {
        return Err(CompileError::new("contract signature clash"));
    }
    Ok(())
}

pub(super) fn expand_conformances(
    classes: &mut [Class],
    contracts: &[Contract],
) -> Result<(), CompileError> {
    for class in classes {
        for (contract, arguments) in class.contract_arguments.clone() {
            let bindings: Vec<_> = contracts[contract]
                .type_parameters
                .iter()
                .cloned()
                .zip(arguments)
                .collect();
            for (name, arguments) in &contracts[contract].parent_arguments {
                let parent = contracts
                    .iter()
                    .rposition(|known| known.name == *name)
                    .ok_or_else(|| CompileError::new("contract parent unbound"))?;
                let arguments: Vec<_> = arguments
                    .iter()
                    .map(|argument| super::qualified::substitute(argument, &bindings))
                    .collect();
                match class
                    .contract_arguments
                    .iter()
                    .find(|(known, _)| *known == parent)
                {
                    Some((_, known)) if *known != arguments => {
                        return Err(CompileError::new("static impl"));
                    }
                    Some(_) => {}
                    None => {
                        class.contracts.push(parent);
                        class.contract_arguments.push((parent, arguments));
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn closed_requirement(
    contract: &Contract,
    arguments: &[TypeExpression],
    promise: &MethodDeclaration,
) -> MethodDeclaration {
    let bindings: Vec<_> = contract
        .type_parameters
        .iter()
        .cloned()
        .zip(arguments.iter().cloned())
        .collect();
    substitute_requirement(promise, &bindings)
}
