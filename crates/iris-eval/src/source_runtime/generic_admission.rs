use super::{EvaluationError, SourceEvaluator, wrapper_generics};
use iris_runtime::{BuiltinClass, ClassId, NominalType, Value};
use iris_syntax::TypeExpression;
use std::collections::HashMap;

impl SourceEvaluator {
    pub(super) fn nominal_type(
        &mut self,
        expression: &TypeExpression,
    ) -> Result<NominalType, EvaluationError> {
        let expression = wrapper_generics::substitute(expression, &self.method_type_bindings);
        let (name, arguments) = match &expression {
            TypeExpression::Generic { name, .. } if name == "Closure" || name == "Block" => {
                return match self.reify_type(&expression)? {
                    Value::Type(class, arguments) => Ok(NominalType::new(class, arguments)),
                    _ => Err(EvaluationError::UnsupportedConstruct),
                };
            }
            TypeExpression::Name(name) => (name, Vec::new()),
            TypeExpression::Generic { name, arguments } => {
                (name, self.nominal_arguments(arguments)?)
            }
            TypeExpression::Typeof(_)
            | TypeExpression::Intersection(_)
            | TypeExpression::Union(_)
            | TypeExpression::Function { .. } => return Err(EvaluationError::UnsupportedConstruct),
        };
        let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
        self.validate_class_arguments(class, &arguments)?;
        Ok(NominalType::new(class, arguments))
    }

    pub(super) fn validate_class_arguments(
        &mut self,
        class: ClassId,
        arguments: &[NominalType],
    ) -> Result<(), EvaluationError> {
        if self
            .wrapper_owner_parameters
            .get(&class)
            .is_some_and(|parameters| parameters.len() != arguments.len())
        {
            return Err(EvaluationError::TypeContractError);
        }
        let name = self
            .generic_bounds
            .keys()
            .find(|name| self.class_name(name) == Ok(Some(class)))
            .cloned();
        if let Some(name) = name {
            self.check_nominal_generic_bounds(&name, arguments)?;
        }
        Ok(())
    }

    pub(super) fn check_generic_bounds(
        &mut self,
        name: &str,
        arguments: &[TypeExpression],
    ) -> Result<(), EvaluationError> {
        let arguments = self.nominal_arguments(arguments)?;
        let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
        self.validate_class_arguments(class, &arguments)
    }

    pub(super) fn check_nominal_generic_bounds(
        &mut self,
        name: &str,
        arguments: &[NominalType],
    ) -> Result<(), EvaluationError> {
        let Some((parameters, constraints)) = self.generic_bounds.get(name).cloned() else {
            return Ok(());
        };
        if parameters.len() != arguments.len() {
            return Err(EvaluationError::TypeContractError);
        }
        let bindings = parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| {
                Ok((
                    parameter.clone(),
                    self.wrapper_nominal_annotation(argument)?,
                ))
            })
            .collect::<Result<HashMap<_, _>, EvaluationError>>()?;
        for constraint in &constraints {
            let position = parameters
                .iter()
                .position(|parameter| *parameter == constraint.parameter)
                .ok_or(EvaluationError::TypeContractError)?;
            let bound = wrapper_generics::substitute(&constraint.bound, &bindings);
            if !self.nominal_bound_admits(&arguments[position], &bound)? {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }

    fn nominal_bound_admits(
        &mut self,
        argument: &NominalType,
        bound: &TypeExpression,
    ) -> Result<bool, EvaluationError> {
        match bound {
            TypeExpression::Union(members) => {
                for member in members {
                    if self.nominal_bound_admits(argument, member)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TypeExpression::Intersection(members) => {
                for member in members {
                    if !self.nominal_bound_admits(argument, member)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TypeExpression::Name(name) if name == "NonNil" => Ok(argument.class()
                != self
                    .kernel
                    .class(BuiltinClass::Nil)
                    .map_err(EvaluationError::Runtime)?),
            TypeExpression::Name(name) if name == "Never" => Ok(false),
            TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => {
                if let Some(contract) = self.contract_names.get(name).copied() {
                    if !self.conforms_through_ancestry(argument.class(), contract)? {
                        return Ok(false);
                    }
                    return match self.check_closed_contract_bound(argument, bound) {
                        Ok(()) => Ok(true),
                        Err(EvaluationError::TypeContractError) => Ok(false),
                        Err(error) => Err(error),
                    };
                }
                let target = self.nominal_type(bound)?;
                if !target.arguments().is_empty() {
                    return Ok(argument == &target);
                }
                Ok(self.subtype(argument.class(), target.class())? == Value::Bool(true))
            }
            TypeExpression::Typeof(_) | TypeExpression::Function { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
        }
    }
}
