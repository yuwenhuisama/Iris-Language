use super::{EvaluationError, SourceEvaluator};
use iris_runtime::Value;
use iris_syntax::{Expression, MethodDeclaration, ParameterCategory, TypeExpression};
use std::collections::HashMap;

pub(super) fn substitute(
    annotation: &TypeExpression,
    bindings: &HashMap<String, TypeExpression>,
) -> TypeExpression {
    match annotation {
        TypeExpression::Name(name) => bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| annotation.clone()),
        TypeExpression::Generic { name, arguments } => TypeExpression::Generic {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|item| substitute(item, bindings))
                .collect(),
        },
        TypeExpression::Function { parameters, result } => TypeExpression::Function {
            parameters: parameters
                .iter()
                .map(|item| substitute(item, bindings))
                .collect(),
            result: Box::new(substitute(result, bindings)),
        },
        TypeExpression::Union(items) => TypeExpression::Union(
            items
                .iter()
                .map(|item| substitute(item, bindings))
                .collect(),
        ),
        TypeExpression::Intersection(items) => TypeExpression::Intersection(
            items
                .iter()
                .map(|item| substitute(item, bindings))
                .collect(),
        ),
        TypeExpression::Typeof(_) => annotation.clone(),
    }
}

impl SourceEvaluator {
    pub(super) fn explicit_generic_call(
        &mut self,
        (callee, operands, types): (&Expression, &[Expression], &[TypeExpression]),
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let (target, selector, qualified) = match callee {
            Expression::Member { receiver, selector } => (receiver, selector, false),
            Expression::ContractView { receiver, selector } => (receiver, selector, true),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        let target = self.expression(target, locals, receiver.clone())?;
        let target = self.rooted(target);
        let arguments = self.evaluate_arguments(operands, (locals, receiver))?;
        let arguments = self.rooted(arguments);
        let types = types
            .iter()
            .map(|annotation| substitute(annotation, &self.method_type_bindings))
            .collect();
        let previous = std::mem::replace(&mut self.explicit_method_types, types);
        let target = std::rc::Rc::unwrap_or_clone(target);
        let result = if qualified {
            self.qualified_send(target, selector, &arguments)
        } else {
            self.send(target, selector, &arguments)
        };
        self.explicit_method_types = previous;
        result
    }

    pub(super) fn select_wrapper_types(
        &mut self,
        method: &MethodDeclaration,
        arguments: &[Value],
    ) -> Result<(), EvaluationError> {
        super::call_channels::block(arguments)?;
        let explicit = std::mem::take(&mut self.explicit_method_types);
        self.method_type_bindings.clear();
        if method.type_parameters.is_empty() {
            return Ok(());
        }
        if !explicit.is_empty() && explicit.len() != method.type_parameters.len() {
            return Err(EvaluationError::ArgumentError);
        }
        for (name, annotation) in method.type_parameters.iter().zip(explicit) {
            if annotation != TypeExpression::Name("_".into()) {
                self.reify_type(&annotation)?;
                self.method_type_bindings.insert(name.clone(), annotation);
            }
        }
        let mut position = 0;
        for parameter in &method.parameters {
            let value = match parameter.category {
                ParameterCategory::Positional => {
                    let value = arguments
                        .iter()
                        .filter(|value| {
                            matches!(
                                super::call_channels::ArgumentChannel::of(value),
                                super::call_channels::ArgumentChannel::Positional(_)
                            )
                        })
                        .nth(position);
                    position += 1;
                    value
                }
                ParameterCategory::Keyword => arguments.iter().find_map(|value| match value {
                    Value::KeywordArgument(name, value) if name == &parameter.name => {
                        Some(value.as_ref())
                    }
                    _ => None,
                }),
                ParameterCategory::Rest
                | ParameterCategory::KeywordRest
                | ParameterCategory::Block => None,
            };
            if let (Some(TypeExpression::Name(name)), Some(value)) = (&parameter.annotation, value)
                && method.type_parameters.contains(name)
                && !self.method_type_bindings.contains_key(name)
            {
                let name_type = match value {
                    Value::Object(_) => {
                        let class = self.class_of_value(value)?;
                        self.names
                            .iter()
                            .find_map(|(name, binding)| {
                                (binding.value() == Value::Class(class))
                                    .then(|| TypeExpression::Name(name.clone()))
                            })
                            .ok_or(EvaluationError::UnsupportedConstruct)?
                    }
                    Value::Nil
                    | Value::Bool(_)
                    | Value::Integer(_)
                    | Value::Float32(_)
                    | Value::Float64(_)
                    | Value::Text(_)
                    | Value::Symbol(_) => {
                        TypeExpression::Name(super::receiver_class_name(value).into())
                    }
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                self.method_type_bindings.insert(name.clone(), name_type);
            }
        }
        if method
            .type_parameters
            .iter()
            .any(|name| !self.method_type_bindings.contains_key(name))
        {
            return Err(self.wrapper_type_error());
        }
        Ok(())
    }

    pub(super) fn open_annotation_metadata(
        &mut self,
        annotation: &TypeExpression,
        parameters: &[String],
    ) -> Result<Value, EvaluationError> {
        match annotation {
            TypeExpression::Name(name) if parameters.contains(name) => self.metadata_record(vec![
                ("kind", Value::Symbol("parameter".into())),
                ("name", Value::Symbol(name.clone())),
            ]),
            TypeExpression::Name(_) | TypeExpression::Typeof(_) => {
                self.wrapper_type_value(annotation)
            }
            TypeExpression::Generic { name, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|item| self.open_annotation_metadata(item, parameters))
                    .collect::<Result<Vec<_>, _>>()?;
                self.metadata_record(vec![
                    ("kind", Value::Symbol("generic".into())),
                    ("name", Value::Symbol(name.clone())),
                    ("arguments", Value::ReadonlyArray(arguments)),
                ])
            }
            TypeExpression::Function {
                parameters: inputs,
                result,
            } => {
                let inputs = inputs
                    .iter()
                    .map(|item| self.open_annotation_metadata(item, parameters))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.open_annotation_metadata(result, parameters)?;
                self.metadata_record(vec![
                    ("kind", Value::Symbol("function".into())),
                    ("parameters", Value::ReadonlyArray(inputs)),
                    ("result", result),
                ])
            }
            TypeExpression::Union(items) | TypeExpression::Intersection(items) => {
                let kind = match annotation {
                    TypeExpression::Union(_) => "union",
                    _ => "intersection",
                };
                let items = items
                    .iter()
                    .map(|item| self.open_annotation_metadata(item, parameters))
                    .collect::<Result<Vec<_>, _>>()?;
                self.metadata_record(vec![
                    ("kind", Value::Symbol(kind.into())),
                    ("members", Value::ReadonlyArray(items)),
                ])
            }
        }
    }
}
