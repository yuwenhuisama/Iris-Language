use iris_syntax::TypeExpression;

use super::{Admission, callable_name};

pub(super) fn equivalent(
    admission: &Admission<'_>,
    argument: &TypeExpression,
    bound: &TypeExpression,
) -> bool {
    match (callable_signature(argument), callable_signature(bound)) {
        (Some((argument_kind, argument)), Some((bound_kind, bound))) => {
            argument_kind == bound_kind
                && argument
                    .parameters
                    .iter()
                    .zip(bound.parameters)
                    .all(|(left, right)| admission.signature_type_equivalent(left, right))
                && argument.parameters.len() == bound.parameters.len()
                && admission.signature_type_equivalent(argument.result, bound.result)
        }
        (None, None) => argument == bound,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

struct Signature<'a> {
    parameters: &'a [TypeExpression],
    result: &'a TypeExpression,
}

fn callable_signature(expression: &TypeExpression) -> Option<(&str, Signature<'_>)> {
    let TypeExpression::Generic { name, arguments } = expression else {
        return None;
    };
    let [TypeExpression::Function { parameters, result }] = arguments.as_slice() else {
        return None;
    };
    callable_name(name).then_some((name, Signature { parameters, result }))
}

impl Admission<'_> {
    fn signature_type_equivalent(&self, left: &TypeExpression, right: &TypeExpression) -> bool {
        let left = self.normalize_signature_type(left);
        let right = self.normalize_signature_type(right);
        self.normalized_type_equivalent(&left, &right)
    }

    fn normalize_signature_type(&self, expression: &TypeExpression) -> TypeExpression {
        match expression {
            TypeExpression::Generic { name, arguments } if callable_name(name) => {
                let Some((_, signature)) = callable_signature(expression) else {
                    return expression.clone();
                };
                TypeExpression::Generic {
                    name: name.clone(),
                    arguments: vec![TypeExpression::Function {
                        parameters: signature
                            .parameters
                            .iter()
                            .map(|parameter| self.normalize_signature_type(parameter))
                            .collect(),
                        result: Box::new(self.normalize_signature_type(signature.result)),
                    }],
                }
            }
            TypeExpression::Generic { name, arguments } => TypeExpression::Generic {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.normalize_signature_type(argument))
                    .collect(),
            },
            TypeExpression::Union(members) => {
                let mut normalized = Vec::new();
                for member in members {
                    match self.normalize_signature_type(member) {
                        TypeExpression::Name(name) if name == "Never" => {}
                        TypeExpression::Union(nested) => {
                            normalized.extend(nested);
                        }
                        member => normalized.push(member),
                    }
                }
                self.normalize_members(normalized, true)
            }
            TypeExpression::Intersection(members) => {
                let mut normalized = Vec::new();
                for member in members {
                    match self.normalize_signature_type(member) {
                        TypeExpression::Name(name) if name == "Never" => {
                            return TypeExpression::Name(name);
                        }
                        TypeExpression::Intersection(nested) => normalized.extend(nested),
                        member => normalized.push(member),
                    }
                }
                self.normalize_members(normalized, false)
            }
            TypeExpression::Name(_)
            | TypeExpression::Function { .. }
            | TypeExpression::Typeof(_) => expression.clone(),
        }
    }

    fn normalize_members(&self, members: Vec<TypeExpression>, union: bool) -> TypeExpression {
        let mut unique = Vec::new();
        for member in members {
            if !unique
                .iter()
                .any(|existing| self.normalized_type_equivalent(&member, existing))
            {
                unique.push(member);
            }
        }
        if !union && unique.iter().any(is_non_nil) {
            if unique.iter().any(is_nil) {
                return TypeExpression::Name("Never".into());
            }
            unique = unique
                .into_iter()
                .map(|member| match member {
                    TypeExpression::Union(mut nested) => {
                        nested.retain(|member| !is_nil(member));
                        self.normalize_members(nested, true)
                    }
                    member => member,
                })
                .flat_map(|member| match member {
                    TypeExpression::Intersection(nested) => nested,
                    member => vec![member],
                })
                .collect();
            if unique.iter().any(|member| {
                matches!(member, TypeExpression::Name(name) if name == "Nil" || name == "Never")
            }) {
                return TypeExpression::Name("Never".into());
            }
            if unique
                .iter()
                .any(|member| !is_non_nil(member) && self.provably_non_nil(member))
            {
                unique.retain(|member| !is_non_nil(member));
            }
        }
        let mut kept = Vec::new();
        for member in unique {
            let mut absorbed = false;
            kept.retain(|existing| {
                let member_subtype = self.signature_subtype(&member, existing);
                let existing_subtype = self.signature_subtype(existing, &member);
                if self.normalized_type_equivalent(&member, existing)
                    || (union && member_subtype)
                    || (!union && existing_subtype)
                {
                    absorbed = true;
                    true
                } else {
                    !((union && existing_subtype) || (!union && member_subtype))
                }
            });
            if !absorbed {
                kept.push(member);
            }
        }
        match kept.as_slice() {
            [] => TypeExpression::Name(if union { "Never" } else { "Object" }.into()),
            [member] => member.clone(),
            _ if union => TypeExpression::Union(kept),
            _ => TypeExpression::Intersection(kept),
        }
    }

    fn signature_subtype(&self, argument: &TypeExpression, bound: &TypeExpression) -> bool {
        match bound {
            TypeExpression::Name(name) if name == "NonNil" => self.provably_non_nil(argument),
            TypeExpression::Union(members) => members
                .iter()
                .any(|member| self.signature_subtype(argument, member)),
            TypeExpression::Intersection(members) => members
                .iter()
                .all(|member| self.signature_subtype(argument, member)),
            TypeExpression::Name(_)
            | TypeExpression::Generic { .. }
            | TypeExpression::Function { .. }
            | TypeExpression::Typeof(_) => self.bound_admits(argument, bound),
        }
    }

    fn provably_non_nil(&self, expression: &TypeExpression) -> bool {
        match expression {
            TypeExpression::Name(name) if name == "NonNil" => true,
            TypeExpression::Name(name) if name == "Nil" || name == "Object" => false,
            TypeExpression::Name(_) | TypeExpression::Generic { .. } => {
                self.known_nominal(expression)
            }
            TypeExpression::Union(members) => {
                members.iter().all(|member| self.provably_non_nil(member))
            }
            TypeExpression::Intersection(members) => {
                members.iter().any(|member| self.provably_non_nil(member))
            }
            TypeExpression::Function { .. } | TypeExpression::Typeof(_) => false,
        }
    }

    fn normalized_type_equivalent(&self, left: &TypeExpression, right: &TypeExpression) -> bool {
        match (left, right) {
            (TypeExpression::Union(left), TypeExpression::Union(right))
            | (TypeExpression::Intersection(left), TypeExpression::Intersection(right)) => {
                left.len() == right.len()
                    && left.iter().all(|member| {
                        right
                            .iter()
                            .any(|candidate| self.normalized_type_equivalent(member, candidate))
                    })
            }
            (
                TypeExpression::Generic {
                    name: left_name,
                    arguments: left_arguments,
                },
                TypeExpression::Generic {
                    name: right_name,
                    arguments: right_arguments,
                },
            ) if callable_name(left_name) && callable_name(right_name) => {
                left_name == right_name
                    && left_arguments.len() == right_arguments.len()
                    && equivalent(self, left, right)
            }
            _ => left == right,
        }
    }
}

fn is_non_nil(expression: &TypeExpression) -> bool {
    matches!(expression, TypeExpression::Name(name) if name == "NonNil")
}

fn is_nil(expression: &TypeExpression) -> bool {
    matches!(expression, TypeExpression::Name(name) if name == "Nil")
}

#[cfg(test)]
#[path = "callable_tests.rs"]
mod tests;
