use super::{Machine, MachineError};
use crate::compile::ParameterKind;
use iris_runtime::Value;

impl Machine {
    pub(super) fn bind_prepared(
        kinds: &[(ParameterKind, String)],
        optional: &[bool],
        arguments: &[Value],
    ) -> Result<(Vec<Value>, Vec<bool>), MachineError> {
        let CallArguments {
            positional,
            mut keywords,
            block,
        } = CallArguments::scan(arguments)?;
        if block.is_some() && !kinds.iter().any(|(kind, _)| *kind == ParameterKind::Block) {
            return Err(MachineError::ArgumentError);
        }
        let mut bound = Vec::new();
        let mut supplied = Vec::new();
        let mut position = 0;
        for (index, (kind, name)) in kinds.iter().enumerate() {
            let value = match kind {
                ParameterKind::Positional => {
                    let value = positional.get(position).cloned();
                    position += usize::from(value.is_some());
                    value
                }
                ParameterKind::Keyword => keywords
                    .iter()
                    .position(|(held, _)| held == name)
                    .map(|index| keywords.remove(index).1),
                ParameterKind::Rest => {
                    let value =
                        Value::Array(iris_runtime::ArrayRef::new(positional[position..].to_vec()));
                    position = positional.len();
                    Some(value)
                }
                ParameterKind::KeywordRest => Some(Value::Hash(iris_runtime::HashRef::new(
                    std::mem::take(&mut keywords)
                        .into_iter()
                        .map(|(name, value)| (Value::Symbol(name), value))
                        .collect(),
                ))),
                ParameterKind::Block => block.clone(),
            };
            if value.is_none() && !optional.get(index).copied().unwrap_or(false) {
                return Err(MachineError::ArgumentError);
            }
            supplied.push(value.is_some());
            bound.push(value.unwrap_or(Value::Nil));
        }
        if position != positional.len() || !keywords.is_empty() {
            return Err(MachineError::ArgumentError);
        }
        Ok((bound, supplied))
    }
}

pub(super) struct CallArguments {
    pub positional: Vec<Value>,
    pub keywords: Vec<(String, Value)>,
    pub block: Option<Value>,
}

impl CallArguments {
    pub(super) fn scan(arguments: &[Value]) -> Result<Self, MachineError> {
        let mut channels = Self {
            positional: Vec::new(),
            keywords: Vec::new(),
            block: None,
        };
        for argument in arguments {
            match argument {
                Value::BlockArgument(value) => {
                    if channels.block.replace(value.as_ref().clone()).is_some() {
                        return Err(MachineError::ArgumentError);
                    }
                }
                Value::KeywordArgument(name, value) => {
                    if channels.keywords.iter().any(|(held, _)| held == name) {
                        return Err(MachineError::ArgumentError);
                    }
                    channels
                        .keywords
                        .push((name.clone(), value.as_ref().clone()));
                }
                value => channels.positional.push(value.clone()),
            }
        }
        Ok(channels)
    }
}

pub(super) fn native_block_arguments(
    receiver: &Value,
    selector: &str,
    arguments: &[Value],
) -> Result<Option<Vec<Value>>, MachineError> {
    let positional_arity = match receiver {
        Value::Array(_) | Value::ImmutableArray(_) => match selector {
            "map" | "each" | "each_with_index" | "select" | "reject" | "find" | "count"
            | "all?" | "any?" => Some(0..=0),
            "reduce" => Some(0..=1),
            _ => None,
        },
        Value::Hash(_) | Value::ImmutableHash(_) => match selector {
            "map" | "each" | "each_with_iterator" | "select" | "rehash" => Some(0..=0),
            _ => None,
        },
        Value::Class(_) if selector == "define_property" => Some(1..=1),
        Value::Symbol(namespace) if namespace == "Revision" && selector == "subscribe" => {
            Some(0..=1)
        }
        _ => None,
    };
    if !arguments
        .iter()
        .any(|argument| matches!(argument, Value::BlockArgument(_)))
    {
        return Ok(None);
    }
    let Some(positional_arity) = positional_arity else {
        return match receiver {
            Value::Class(_)
            | Value::ClosedClass(..)
            | Value::Object(_)
            | Value::Closure(_)
            | Value::BoundMethod(_) => Ok(None),
            _ => Err(MachineError::ArgumentError),
        };
    };
    let CallArguments {
        mut positional,
        keywords,
        block,
    } = CallArguments::scan(arguments)?;
    if !keywords.is_empty() || !positional_arity.contains(&positional.len()) {
        return Err(MachineError::ArgumentError);
    }
    match block {
        Some(value @ (Value::Closure(_) | Value::BoundMethod(_))) => {
            if matches!(receiver, Value::Symbol(namespace) if namespace == "Revision") {
                positional.insert(0, value);
            } else {
                positional.push(value);
            }
        }
        Some(_) => return Err(MachineError::Kernel(iris_runtime::KernelError::Type)),
        None => return Err(MachineError::ArgumentError),
    }
    Ok(Some(positional))
}
