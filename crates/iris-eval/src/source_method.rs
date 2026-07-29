use std::collections::HashMap;

use iris_runtime::{ExecutionError, Kernel, Method, Value, Visibility};
use iris_syntax::{BinaryOperator, Expression, MethodDeclaration, Statement};

use crate::EvaluationError;

pub(super) fn invoke(
    bodies: &HashMap<u64, MethodDeclaration>,
    method: Method,
    receiver: Value,
    arguments: &[Value],
) -> Result<Value, ExecutionError> {
    let Some(declaration) = bodies.get(&method.body().raw()) else {
        return Err(ExecutionError::Raised(Value::Nil));
    };
    if declaration.parameters.len() != arguments.len() {
        return Err(ExecutionError::Raised(Value::Nil));
    }
    let mut values = HashMap::new();
    for (parameter, value) in declaration.parameters.iter().zip(arguments) {
        values.insert(parameter.clone(), value.clone());
    }
    let expression = declaration
        .body
        .last()
        .and_then(|statement| match statement {
            Statement::Expression(expression) => Some(expression),
            _ => None,
        });
    let Some(expression) = expression else {
        return Ok(Value::Nil);
    };
    match expression {
        Expression::Symbol(symbol) => Ok(Value::Symbol(symbol.clone())),
        Expression::Binary {
            left,
            operator: BinaryOperator::Multiply,
            right,
        } => multiply(&values, left, right),
        Expression::Name(name) => values
            .get(name)
            .cloned()
            .or(match name.as_str() {
                "nil" => Some(Value::Nil),
                "true" => Some(Value::Bool(true)),
                "false" => Some(Value::Bool(false)),
                _ => None,
            })
            .ok_or(ExecutionError::Raised(Value::Nil)),
        Expression::Literal(value) => {
            literal(value).map_err(|_| ExecutionError::Raised(Value::Nil))
        }
        _ => Err(ExecutionError::Raised(receiver)),
    }
}

fn multiply(
    values: &HashMap<String, Value>,
    left: &Expression,
    right: &Expression,
) -> Result<Value, ExecutionError> {
    let Expression::Name(name) = left else {
        return Err(ExecutionError::Raised(Value::Nil));
    };
    let Some(Value::Integer(value)) = values.get(name).cloned() else {
        return Err(ExecutionError::Raised(Value::Nil));
    };
    let Expression::Literal(right) = right else {
        return Err(ExecutionError::Raised(Value::Nil));
    };
    let multiplier = right
        .parse::<iris_runtime::IntegerValue>()
        .map_err(|_| ExecutionError::Raised(Value::Nil))?;
    Kernel::new()
        .map_err(|_| ExecutionError::Raised(Value::Nil))?
        .send(
            Value::Integer(value),
            iris_runtime::NativeSelector::Multiply,
            &[Value::Integer(multiplier)],
        )
        .map_err(|_| ExecutionError::Raised(Value::Nil))
}

pub(super) fn visibility(method: &MethodDeclaration) -> Visibility {
    match method.visibility {
        iris_syntax::Visibility::Public => Visibility::Public,
        iris_syntax::Visibility::Private => Visibility::Private,
        iris_syntax::Visibility::Protected => Visibility::Protected,
    }
}

pub(super) fn literal(source: &str) -> Result<Value, EvaluationError> {
    match source {
        "nil" => return Ok(Value::Nil),
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    source
        .parse::<iris_runtime::IntegerValue>()
        .map(Value::Integer)
        .map_err(|_| EvaluationError::UnsupportedConstruct)
}

pub(super) fn builtin(name: &str, kernel: &Kernel) -> Option<Value> {
    match name {
        "nil" => Some(Value::Nil),
        "true" => Some(Value::Bool(true)),
        "false" => Some(Value::Bool(false)),
        "Integer" => kernel
            .class(iris_runtime::BuiltinClass::Integer)
            .ok()
            .map(Value::Class),
        "Nil" => kernel
            .class(iris_runtime::BuiltinClass::Nil)
            .ok()
            .map(Value::Class),
        "Bool" => kernel
            .class(iris_runtime::BuiltinClass::Bool)
            .ok()
            .map(Value::Class),
        "Float32" => kernel
            .class(iris_runtime::BuiltinClass::Float32)
            .ok()
            .map(Value::Class),
        "Float64" => kernel
            .class(iris_runtime::BuiltinClass::Float64)
            .ok()
            .map(Value::Class),
        _ => None,
    }
}
