use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{
    Invocation, InvocationParameter, InvocationPayload, InvocationSignature, InvocationSlot,
    ParameterCategory, SelectedCall, SlotKind, SourceCall,
};
use iris_runtime::{ArrayRef, HashRef, MethodOwner, Value};
use iris_syntax::{
    MethodDeclaration, MethodKind, ParameterCategory as SyntaxCategory, TypeExpression,
};
use std::collections::HashMap;

impl SourceEvaluator {
    pub(super) fn prepare_wrapper_invocation(
        &mut self,
        chain: &super::wrapper_chain::WrapperChain,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Invocation, EvaluationError> {
        let declaration = &chain.declaration;
        let owner_arguments = self.wrapper_owner_arguments(chain, &receiver)?;
        let owner_types = owner_arguments
            .iter()
            .map(|argument| Value::Type(argument.class(), argument.arguments().to_vec()))
            .collect();
        self.current_contract = chain.qualifier;
        let mut positional = Vec::new();
        let mut keywords = Vec::new();
        let block = super::call_channels::block(arguments)?.cloned();
        for argument in arguments {
            match super::call_channels::ArgumentChannel::of(argument) {
                super::call_channels::ArgumentChannel::Block(_) => {}
                super::call_channels::ArgumentChannel::Keyword(name, value) => {
                    if keywords.iter().any(|(held, _)| held == name) {
                        return Err(EvaluationError::ArgumentError);
                    }
                    keywords.push((name.to_owned(), value.clone()));
                }
                super::call_channels::ArgumentChannel::Positional(value) => {
                    positional.push(value.clone())
                }
            }
        }
        let has_block = declaration
            .parameters
            .iter()
            .any(|parameter| parameter.category == SyntaxCategory::Block);
        if block.is_some() && !has_block {
            return Err(EvaluationError::ArgumentError);
        }
        let source = SourceCall::new(positional.clone(), keywords.clone(), block.clone());
        let mut next = 0;
        let mut omitted = Vec::new();
        let mut defaulted = Vec::new();
        let mut bindings = Vec::new();
        let locals = self.rooted(std::cell::RefCell::new(HashMap::new()));
        for parameter in &declaration.parameters {
            let supplied = match parameter.category {
                SyntaxCategory::Positional => {
                    let value = positional.get(next).cloned();
                    next += usize::from(value.is_some());
                    value
                }
                SyntaxCategory::Keyword => keywords
                    .iter()
                    .position(|(name, _)| name == &parameter.name)
                    .map(|position| keywords.remove(position).1),
                SyntaxCategory::Rest => {
                    let rest = positional[next..].to_vec();
                    next = positional.len();
                    Some(Value::Array(ArrayRef::new(rest)))
                }
                SyntaxCategory::KeywordRest => Some(Value::Hash(HashRef::new(
                    std::mem::take(&mut keywords)
                        .into_iter()
                        .map(|(name, value)| (Value::Symbol(name), value))
                        .collect(),
                ))),
                SyntaxCategory::Block => block.clone(),
            };
            let value = match supplied {
                Some(value) => value,
                None => {
                    omitted.push(parameter.name.clone());
                    let Some(default) = &parameter.default else {
                        return Err(EvaluationError::ArgumentError);
                    };
                    defaulted.push(parameter.name.clone());
                    self.expression(default, &locals.borrow(), Some(receiver.clone()))?
                }
            };
            if let Some(annotation) = &parameter.annotation {
                let annotation =
                    super::wrapper_generics::substitute(annotation, &self.method_type_bindings);
                let accepted = match (&parameter.category, &value) {
                    (SyntaxCategory::Rest, Value::Array(array)) => {
                        array.elements().iter().all(|value| {
                            self.wrapper_annotation_admits(value, &annotation)
                                .unwrap_or(false)
                        })
                    }
                    (SyntaxCategory::KeywordRest, Value::Hash(hash)) => {
                        hash.entries().iter().all(|(_, value)| {
                            self.wrapper_annotation_admits(value, &annotation)
                                .unwrap_or(false)
                        })
                    }
                    (SyntaxCategory::Block, Value::Nil) if parameter.default.is_some() => true,
                    _ => self.wrapper_annotation_admits(&value, &annotation)?,
                };
                if !accepted {
                    return Err(self.wrapper_type_error());
                }
            }
            locals
                .borrow_mut()
                .insert(parameter.name.clone(), value.clone());
            bindings.push(value);
        }
        if next < positional.len() || !keywords.is_empty() {
            return Err(EvaluationError::ArgumentError);
        }
        let signature = self.wrapper_signature(declaration)?;
        let payload =
            InvocationPayload::from_bindings(&signature, &bindings, |annotation, value| {
                self.wrapper_accepts(annotation, value)
            })
            .map_err(|error| {
                self.core_boundary_error(super::decorator_errors::argument_error(error))
            })?;
        let declaring = match chain.original.owner() {
            MethodOwner::Class(class) => match chain.qualifier {
                Some(_) if owner_arguments.is_empty() => Value::Class(class),
                Some(_) => Value::Type(class, owner_arguments.clone()),
                None if declaration.kind == MethodKind::Class => Value::Class(class),
                None => Value::Type(class, owner_arguments.clone()),
            },
            MethodOwner::Module(module) => self.module_symbol(module),
        };
        let (selector, kind) = match declaration.kind {
            MethodKind::Property => match declaration.selector.strip_suffix('=') {
                Some(name) => (name, SlotKind::Setter),
                None => (declaration.selector.as_str(), SlotKind::Getter),
            },
            MethodKind::Instance | MethodKind::Class | MethodKind::Module => {
                (declaration.selector.as_str(), SlotKind::Method)
            }
        };
        let slot = InvocationSlot::new(declaring, selector, kind);
        let slot = match chain.qualifier {
            Some(contract) => {
                let MethodOwner::Class(class) = chain.original.owner() else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                slot.qualified(self.closed_qualified_contract((class, contract), &owner_arguments)?)
            }
            None => slot,
        };
        let method_types = declaration
            .type_parameters
            .iter()
            .map(|name| self.wrapper_type_value(&TypeExpression::Name(name.clone())))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Invocation::new(
            SelectedCall::new(receiver, slot, signature)
                .with_type_arguments(owner_types, method_types),
            source.with_provenance(omitted, defaulted),
            payload,
        ))
    }

    pub(super) fn wrapper_signature(
        &mut self,
        method: &MethodDeclaration,
    ) -> Result<InvocationSignature, EvaluationError> {
        let mut parameters = Vec::new();
        for parameter in &method.parameters {
            let category = match parameter.category {
                SyntaxCategory::Positional => ParameterCategory::Positional,
                SyntaxCategory::Keyword => ParameterCategory::Keyword,
                SyntaxCategory::Rest => ParameterCategory::Rest,
                SyntaxCategory::KeywordRest => ParameterCategory::KeywordRest,
                SyntaxCategory::Block => ParameterCategory::Block,
            };
            let annotation = parameter
                .annotation
                .clone()
                .unwrap_or(TypeExpression::Name("Object".into()));
            parameters.push(
                InvocationParameter::new(
                    &parameter.name,
                    category,
                    self.wrapper_type_value(&annotation)?,
                )
                .with_optional(parameter.default.is_some()),
            );
        }
        let result = method
            .return_type
            .clone()
            .unwrap_or(TypeExpression::Name("Object".into()));
        InvocationSignature::new(
            parameters,
            self.wrapper_type_value(&result)?,
            method.is_async,
        )
        .map_err(super::decorator_errors::argument_error)
    }
}
