use super::{CompileError, declarations::Signature, lowering::Declarations};
use iris_syntax::{MethodDeclaration, TypeExpression};

pub(super) fn complete(
    signature: &Signature<'_>,
    declarations: Declarations<'_, '_>,
) -> Result<Option<MethodDeclaration>, CompileError> {
    let Some(method) = signature.declaration else {
        return Ok(None);
    };
    let Some(Some(qualifier)) = &method.impl_contract else {
        return Ok(None);
    };
    let Some(class) = declarations
        .classes
        .iter()
        .find(|class| class.name == signature.module)
    else {
        return Ok(None);
    };
    let contract_index = declarations
        .contracts
        .iter()
        .rposition(|contract| contract.name == *qualifier)
        .ok_or_else(|| CompileError::new("qualified contract identity"))?;
    let contract = &declarations.contracts[contract_index];
    let Some(requirement) = contract
        .signatures
        .iter()
        .find(|requirement| requirement.selector == method.selector)
    else {
        return Ok(None);
    };
    let arguments = class
        .contract_arguments
        .iter()
        .find(|(index, _)| *index == contract_index)
        .map(|(_, arguments)| arguments.as_slice())
        .unwrap_or_default();
    if arguments.len() != contract.type_parameters.len()
        || requirement.type_parameters.len() != method.type_parameters.len()
        || requirement.parameters.len() != method.parameters.len()
        || requirement.is_async != method.is_async
    {
        return Err(CompileError::new("qualified contract signature"));
    }
    let mut bindings: Vec<_> = requirement
        .type_parameters
        .iter()
        .cloned()
        .zip(
            method
                .type_parameters
                .iter()
                .cloned()
                .map(TypeExpression::Name),
        )
        .collect();
    bindings.extend(
        contract
            .type_parameters
            .iter()
            .cloned()
            .zip(arguments.iter().cloned()),
    );
    let mut selected = super::method_metadata::bodyless(method);
    for (parameter, required) in selected.parameters.iter_mut().zip(&requirement.parameters) {
        if parameter.category != required.category {
            return Err(CompileError::new("qualified contract parameter"));
        }
        let required_type = required
            .annotation
            .as_ref()
            .map(|annotation| substitute(annotation, &bindings));
        if parameter
            .annotation
            .as_ref()
            .zip(required_type.as_ref())
            .is_some_and(|(actual, expected)| actual != expected)
        {
            return Err(CompileError::new("contract signature clash"));
        }
        if parameter.annotation.is_none() {
            parameter.annotation = required_type;
        }
    }
    if selected.return_type.is_none() {
        selected.return_type = requirement
            .return_type
            .as_ref()
            .map(|annotation| substitute(annotation, &bindings));
    }
    Ok(Some(selected))
}

pub(super) fn substitute(
    annotation: &TypeExpression,
    bindings: &[(String, TypeExpression)],
) -> TypeExpression {
    match annotation {
        TypeExpression::Name(name) => bindings
            .iter()
            .find(|(parameter, _)| parameter == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| annotation.clone()),
        TypeExpression::Generic { name, arguments } => TypeExpression::Generic {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute(argument, bindings))
                .collect(),
        },
        TypeExpression::Function { parameters, result } => TypeExpression::Function {
            parameters: parameters
                .iter()
                .map(|parameter| substitute(parameter, bindings))
                .collect(),
            result: Box::new(substitute(result, bindings)),
        },
        TypeExpression::Union(members) => TypeExpression::Union(
            members
                .iter()
                .map(|member| substitute(member, bindings))
                .collect(),
        ),
        TypeExpression::Intersection(members) => TypeExpression::Intersection(
            members
                .iter()
                .map(|member| substitute(member, bindings))
                .collect(),
        ),
        TypeExpression::Typeof(_) => annotation.clone(),
    }
}
