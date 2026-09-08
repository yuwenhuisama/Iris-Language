use crate::{MethodDeclaration, Parameter, ParameterCategory, TypeExpression};

/// Checks declaration replacement, not invariant first-class callable arguments.
/// `nominal_subtype(source, target)` supplies the caller's nominal hierarchy;
/// structural types, Object, Never, and omitted Dynamic<Object> are handled here.
pub fn method_signature_compatible(
    implementation: &MethodDeclaration,
    promise: &MethodDeclaration,
    nominal_subtype: impl Fn(&str, &str) -> bool,
) -> bool {
    if implementation.kind != promise.kind
        || implementation.is_async != promise.is_async
        || implementation.type_parameters != promise.type_parameters
    {
        return false;
    }
    let dynamic = TypeExpression::Generic {
        name: "Dynamic".into(),
        arguments: vec![TypeExpression::Name("Object".into())],
    };
    let relation = Relation {
        nominal: &nominal_subtype,
        dynamic: &dynamic,
    };
    relation.parameters(&implementation.parameters, &promise.parameters)
        && relation.subtype(
            implementation.return_type.as_ref().unwrap_or(&dynamic),
            promise.return_type.as_ref().unwrap_or(&dynamic),
        )
}

struct Relation<'a, Nominal> {
    nominal: &'a Nominal,
    dynamic: &'a TypeExpression,
}

impl<Nominal: Fn(&str, &str) -> bool> Relation<'_, Nominal> {
    fn parameter(&self, implementation: &Parameter, promise: &Parameter) -> bool {
        self.subtype(
            promise.annotation.as_ref().unwrap_or(self.dynamic),
            implementation.annotation.as_ref().unwrap_or(self.dynamic),
        )
    }

    fn parameters(&self, implementation: &[Parameter], promise: &[Parameter]) -> bool {
        let positional = |parameters: &[Parameter]| {
            parameters
                .iter()
                .filter(|parameter| parameter.category == ParameterCategory::Positional)
                .count()
        };
        let provided = positional(promise);
        let accepted = positional(implementation);
        let rest = implementation
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::Rest);
        let promised_rest = promise
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::Rest);
        let required = |parameters: &[Parameter]| {
            parameters
                .iter()
                .filter(|parameter| parameter.category == ParameterCategory::Positional)
                .enumerate()
                .filter(|(_, parameter)| parameter.default.is_none())
                .map(|(index, _)| index + 1)
                .max()
                .unwrap_or(0)
        };
        if required(implementation) > required(promise)
            || (provided > accepted && rest.is_none())
            || (promised_rest.is_some() && rest.is_none())
        {
            return false;
        }
        let mut targets = implementation
            .iter()
            .filter(|parameter| parameter.category == ParameterCategory::Positional);
        for source in promise
            .iter()
            .filter(|parameter| parameter.category == ParameterCategory::Positional)
        {
            let Some(target) = targets.next().or(rest) else {
                return false;
            };
            if !self.parameter(target, source) {
                return false;
            }
        }
        if let Some(source) = promised_rest {
            for target in targets.chain(rest) {
                if !self.parameter(target, source) {
                    return false;
                }
            }
        }
        let keyword_rest = implementation
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::KeywordRest);
        let promised_keyword_rest = promise
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::KeywordRest);
        for source in promise
            .iter()
            .filter(|parameter| parameter.category == ParameterCategory::Keyword)
        {
            let target = implementation
                .iter()
                .find(|parameter| {
                    parameter.category == ParameterCategory::Keyword
                        && parameter.name == source.name
                })
                .or(keyword_rest);
            if !target.is_some_and(|target| self.parameter(target, source)) {
                return false;
            }
        }
        for target in implementation
            .iter()
            .filter(|parameter| parameter.category == ParameterCategory::Keyword)
        {
            let source = promise.iter().find(|parameter| {
                parameter.category == ParameterCategory::Keyword && parameter.name == target.name
            });
            if target.default.is_none()
                && !source.is_some_and(|parameter| parameter.default.is_none())
            {
                return false;
            }
            if source.is_none()
                && promised_keyword_rest.is_some_and(|source| !self.parameter(target, source))
            {
                return false;
            }
        }
        if let Some(source) = promised_keyword_rest
            && !keyword_rest.is_some_and(|target| self.parameter(target, source))
        {
            return false;
        }
        let block = implementation
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::Block);
        let promised_block = promise
            .iter()
            .find(|parameter| parameter.category == ParameterCategory::Block);
        match promised_block {
            Some(source) => block.is_some_and(|target| self.parameter(target, source)),
            None => true,
        }
    }

    fn subtype(&self, source: &TypeExpression, target: &TypeExpression) -> bool {
        if source == target {
            return true;
        }
        if matches!(target, TypeExpression::Name(name) if name == "Object")
            || matches!(source, TypeExpression::Name(name) if name == "Never")
        {
            return true;
        }
        match (source, target) {
            (TypeExpression::Generic { name, arguments }, _) if name == "Dynamic" => {
                matches!(arguments.as_slice(), [bound] if self.subtype(bound, target))
            }
            (_, TypeExpression::Generic { name, arguments }) if name == "Dynamic" => {
                matches!(arguments.as_slice(), [bound] if self.subtype(source, bound))
            }
            (TypeExpression::Union(members), _) => {
                members.iter().all(|member| self.subtype(member, target))
            }
            (_, TypeExpression::Union(members)) => {
                members.iter().any(|member| self.subtype(source, member))
            }
            (_, TypeExpression::Intersection(members)) => {
                members.iter().all(|member| self.subtype(source, member))
            }
            (TypeExpression::Intersection(members), _) => {
                members.iter().any(|member| self.subtype(member, target))
            }
            (
                TypeExpression::Function {
                    parameters: source_parameters,
                    result: source_result,
                },
                TypeExpression::Function {
                    parameters: target_parameters,
                    result: target_result,
                },
            ) => {
                source_parameters.len() == target_parameters.len()
                    && source_parameters
                        .iter()
                        .zip(target_parameters)
                        .all(|(source, target)| self.subtype(target, source))
                    && self.subtype(source_result, target_result)
            }
            (TypeExpression::Name(source), TypeExpression::Name(target)) => {
                (self.nominal)(source, target)
            }
            (
                TypeExpression::Generic {
                    name: source_name,
                    arguments: source_arguments,
                },
                TypeExpression::Generic {
                    name: target_name,
                    arguments: target_arguments,
                },
            ) => source_name == target_name && source_arguments == target_arguments,
            (
                TypeExpression::Name(_)
                | TypeExpression::Typeof(_)
                | TypeExpression::Generic { .. }
                | TypeExpression::Function { .. },
                _,
            ) => false,
        }
    }
}
