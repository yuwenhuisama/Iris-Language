use iris_syntax::TypeExpression;

use super::super::{Class, Contract};

mod callable;

pub(crate) struct Admission<'a> {
    classes: &'a [Class],
    contracts: &'a [Contract],
}

impl<'a> Admission<'a> {
    pub(crate) const fn new(classes: &'a [Class], contracts: &'a [Contract]) -> Self {
        Self { classes, contracts }
    }

    pub(crate) fn accepts(&self, class: usize, arguments: &[TypeExpression]) -> bool {
        let declaration = &self.classes[class];
        if declaration.type_parameters.len() != arguments.len()
            || !arguments
                .iter()
                .all(|argument| self.closed_argument(argument))
        {
            return false;
        }
        let bindings: Vec<_> = declaration.type_parameters.iter().zip(arguments).collect();
        declaration.constraints.iter().all(|constraint| {
            let Some((_, argument)) = bindings
                .iter()
                .find(|(name, _)| **name == constraint.parameter)
            else {
                return false;
            };
            self.bound_admits(argument, &substitute(&constraint.bound, &bindings))
        })
    }

    fn closed_argument(&self, argument: &TypeExpression) -> bool {
        match argument {
            TypeExpression::Name(name) if callable_name(name) => false,
            TypeExpression::Name(name) => self
                .classes
                .iter()
                .position(|class| class.name == *name)
                .is_none_or(|class| self.accepts(class, &[])),
            TypeExpression::Generic { name, arguments } if callable_name(name) => {
                matches!(arguments.as_slice(), [TypeExpression::Function { parameters, result }]
                    if parameters.iter().all(|parameter| self.closed_signature_type(parameter))
                        && self.closed_signature_type(result))
            }
            TypeExpression::Generic { name, arguments } => {
                match self.classes.iter().position(|class| class.name == *name) {
                    Some(class) => self.accepts(class, arguments),
                    None => arguments
                        .iter()
                        .all(|argument| self.closed_argument(argument)),
                }
            }
            TypeExpression::Function { .. }
            | TypeExpression::Typeof(_)
            | TypeExpression::Union(_)
            | TypeExpression::Intersection(_) => false,
        }
    }

    fn closed_signature_type(&self, expression: &TypeExpression) -> bool {
        match expression {
            TypeExpression::Union(members) | TypeExpression::Intersection(members) => members
                .iter()
                .all(|member| self.closed_signature_type(member)),
            TypeExpression::Function { .. } | TypeExpression::Typeof(_) => false,
            _ => self.closed_argument(expression),
        }
    }

    fn bound_admits(&self, argument: &TypeExpression, bound: &TypeExpression) -> bool {
        match bound {
            TypeExpression::Union(members) => members
                .iter()
                .any(|bound| self.bound_admits(argument, bound)),
            TypeExpression::Intersection(members) => members
                .iter()
                .all(|bound| self.bound_admits(argument, bound)),
            TypeExpression::Name(name) if name == "Never" => false,
            TypeExpression::Name(name) if name == "NonNil" => {
                self.known_nominal(argument) && *argument != TypeExpression::Name("Nil".into())
            }
            TypeExpression::Name(name) if name == "Object" => self.known_nominal(argument),
            TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => {
                if let Some(contract) = self
                    .contracts
                    .iter()
                    .rposition(|contract| contract.name == *name)
                {
                    return self.contract_admits(argument, (contract, bound));
                }
                if !self.known_nominal(argument) || !self.known_nominal(bound) {
                    return false;
                }
                if callable::equivalent(self, argument, bound) {
                    return true;
                }
                if matches!(bound, TypeExpression::Generic { .. }) {
                    return false;
                }
                let Some((class, _)) = self.class_arguments(argument) else {
                    return false;
                };
                let mut ancestor = self.classes[class].superclass;
                for _ in 0..self.classes.len() {
                    let Some(class) = ancestor else {
                        return false;
                    };
                    if self.classes[class].name == *name {
                        return true;
                    }
                    ancestor = self.classes[class].superclass;
                }
                false
            }
            TypeExpression::Function { .. } | TypeExpression::Typeof(_) => false,
        }
    }

    fn contract_admits(
        &self,
        argument: &TypeExpression,
        (contract, bound): (usize, &TypeExpression),
    ) -> bool {
        let required = match bound {
            TypeExpression::Generic { arguments, .. } => Some(arguments.as_slice()),
            TypeExpression::Name(_) => None,
            TypeExpression::Function { .. }
            | TypeExpression::Typeof(_)
            | TypeExpression::Union(_)
            | TypeExpression::Intersection(_) => return false,
        };
        if required.is_some_and(|arguments| {
            arguments.len() != self.contracts[contract].type_parameters.len()
        }) {
            return false;
        }
        let Some((mut class, mut arguments)) = self.class_arguments(argument) else {
            return false;
        };
        for _ in 0..self.classes.len() {
            let declaration = &self.classes[class];
            let bindings: Vec<_> = declaration.type_parameters.iter().zip(arguments).collect();
            for (implemented, supplied) in &declaration.contract_arguments {
                if *implemented != contract {
                    continue;
                }
                if required.is_none_or(|required| {
                    supplied.len() == required.len()
                        && supplied
                            .iter()
                            .zip(required)
                            .all(|(actual, expected)| substitute(actual, &bindings) == *expected)
                }) {
                    return true;
                }
            }
            let Some(parent) = declaration.superclass else {
                return false;
            };
            class = parent;
            arguments = &[];
        }
        false
    }

    fn class_arguments<'b>(
        &self,
        expression: &'b TypeExpression,
    ) -> Option<(usize, &'b [TypeExpression])> {
        let (name, arguments) = match expression {
            TypeExpression::Name(name) => (name, &[][..]),
            TypeExpression::Generic { name, arguments } => (name, arguments.as_slice()),
            TypeExpression::Function { .. }
            | TypeExpression::Typeof(_)
            | TypeExpression::Union(_)
            | TypeExpression::Intersection(_) => return None,
        };
        self.classes
            .iter()
            .position(|class| class.name == *name)
            .map(|class| (class, arguments))
    }

    fn known_nominal(&self, expression: &TypeExpression) -> bool {
        if matches!(expression, TypeExpression::Generic { name, .. } if callable_name(name)) {
            return self.closed_argument(expression);
        }
        if self.class_arguments(expression).is_some() {
            return true;
        }
        match expression {
            TypeExpression::Name(name) => super::is_builtin_receiver(name),
            TypeExpression::Generic { name, arguments } => {
                super::is_builtin_receiver(name)
                    && arguments
                        .iter()
                        .all(|argument| self.known_nominal(argument))
            }
            TypeExpression::Function { .. }
            | TypeExpression::Typeof(_)
            | TypeExpression::Union(_)
            | TypeExpression::Intersection(_) => false,
        }
    }
}

fn callable_name(name: &str) -> bool {
    matches!(name, "Block" | "Closure" | "BoundMethod")
}

fn substitute(
    expression: &TypeExpression,
    bindings: &[(&String, &TypeExpression)],
) -> TypeExpression {
    match expression {
        TypeExpression::Name(name) => bindings
            .iter()
            .find(|(parameter, _)| *parameter == name)
            .map_or_else(|| expression.clone(), |(_, value)| (*value).clone()),
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
        TypeExpression::Typeof(_) => expression.clone(),
    }
}
