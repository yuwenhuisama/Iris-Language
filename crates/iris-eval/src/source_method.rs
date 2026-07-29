use iris_runtime::{Kernel, Value, Visibility};
use iris_syntax::MethodDeclaration;

use crate::EvaluationError;

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
