use iris_syntax::{MethodDeclaration, MethodKind, Parameter, ParameterCategory, Visibility};

pub(super) fn bodyless(method: &MethodDeclaration) -> MethodDeclaration {
    MethodDeclaration {
        body: None,
        decorators: Vec::new(),
        parameters: method.parameters.clone(),
        type_parameters: method.type_parameters.clone(),
        return_type: method.return_type.clone(),
        selector: method.selector.clone(),
        impl_contract: method.impl_contract.clone(),
        is_async: method.is_async,
        is_override: method.is_override,
        kind: method.kind,
        visibility: method.visibility,
    }
}

pub(super) fn dynamic(parameters: &[String]) -> MethodDeclaration {
    MethodDeclaration {
        body: None,
        decorators: Vec::new(),
        parameters: parameters
            .iter()
            .map(|name| Parameter {
                name: name.clone(),
                category: ParameterCategory::Positional,
                annotation: None,
                default: None,
            })
            .collect(),
        type_parameters: Vec::new(),
        return_type: None,
        selector: String::new(),
        impl_contract: None,
        is_async: false,
        is_override: false,
        kind: MethodKind::Instance,
        visibility: Visibility::Public,
    }
}
