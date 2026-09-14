use super::{EvaluationError, SourceEvaluator};
use iris_runtime::Value;
use iris_syntax::{Expression, TypeExpression};

pub(super) fn operand(expression: &Expression) -> &Expression {
    match expression {
        Expression::BlockArgument { value } => value,
        expression => expression,
    }
}

pub(super) enum ArgumentChannel<'a> {
    Positional(&'a Value),
    Keyword(&'a str, &'a Value),
    Block(&'a Value),
}

impl<'a> ArgumentChannel<'a> {
    pub(super) fn of(value: &'a Value) -> Self {
        match value {
            Value::BlockArgument(value) => Self::Block(value),
            Value::KeywordArgument(name, value) => Self::Keyword(name, value),
            value => Self::Positional(value),
        }
    }
}

pub(super) fn block(arguments: &[Value]) -> Result<Option<&Value>, EvaluationError> {
    let mut block = None;
    for argument in arguments {
        match ArgumentChannel::of(argument) {
            ArgumentChannel::Block(value) if block.replace(value).is_some() => {
                return Err(EvaluationError::ArgumentError);
            }
            ArgumentChannel::Block(_)
            | ArgumentChannel::Positional(_)
            | ArgumentChannel::Keyword(_, _) => {}
        }
    }
    Ok(block)
}

pub(super) fn native_arguments(
    arguments: &[Value],
) -> Result<std::borrow::Cow<'_, [Value]>, EvaluationError> {
    if block(arguments)?.is_some() {
        Ok(std::borrow::Cow::Owned(
            arguments
                .iter()
                .map(|value| match ArgumentChannel::of(value) {
                    ArgumentChannel::Block(value) => value.clone(),
                    ArgumentChannel::Positional(_) | ArgumentChannel::Keyword(_, _) => {
                        value.clone()
                    }
                })
                .collect(),
        ))
    } else {
        Ok(std::borrow::Cow::Borrowed(arguments))
    }
}

impl SourceEvaluator {
    pub(super) fn callable_admits(
        &mut self,
        value: &Value,
        expected: &Value,
    ) -> Result<bool, EvaluationError> {
        let actual = match value {
            Value::BoundMethod(bound) => self.bound_method_type(*bound)?,
            Value::Closure(identity) => self.closure_type(*identity)?,
            _ => return Ok(false),
        };
        Ok(actual == *expected)
    }

    pub(super) fn bound_method_type(
        &mut self,
        bound: iris_runtime::BoundMethod,
    ) -> Result<Value, EvaluationError> {
        let declaration = match self.wrapper_chains.get(&bound.method().id()) {
            Some(chain) => &chain.declaration,
            None => self
                .bodies
                .get(&bound.method().body().raw())
                .ok_or(EvaluationError::UnsupportedConstruct)?,
        };
        if declaration
            .type_parameters
            .iter()
            .any(|name| !self.method_type_bindings.contains_key(name))
            || declaration
                .body
                .as_ref()
                .is_some_and(|body| super::body_yields(body))
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                parameter
                    .annotation
                    .clone()
                    .unwrap_or(TypeExpression::Name("Object".into()))
            })
            .collect::<Vec<_>>();
        let result = declaration
            .return_type
            .clone()
            .unwrap_or(TypeExpression::Name("Object".into()));
        let result = if declaration.is_async {
            TypeExpression::Generic {
                name: "Task".into(),
                arguments: vec![result],
            }
        } else {
            result
        };
        self.intern_source_callable(
            "BoundMethod",
            &[TypeExpression::Function {
                parameters,
                result: Box::new(result),
            }],
        )
    }
}
