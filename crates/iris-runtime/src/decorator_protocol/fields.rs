use super::{DecoratorValue, Invocation};
use crate::{ImmutableArray, ImmutableHash, Value};

/// Exact backend Type values used for explicit snapshot element storage, not casts.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotTypes {
    pub object: Value,
    pub symbol: Value,
    pub type_type: Value,
    pub invocation_parameter: Value,
    pub symbol_object_tuple: Value,
}

impl SnapshotTypes {
    fn array(&self, elements: Vec<Value>, element_type: &Value) -> Value {
        Value::ImmutableArray(ImmutableArray::new(elements, element_type.clone()))
    }
    fn symbols(&self, names: &[String]) -> Value {
        self.array(
            names.iter().cloned().map(Value::Symbol).collect(),
            &self.symbol,
        )
    }
    fn hash(&self, entries: &[(String, Value)]) -> Value {
        Value::ImmutableHash(ImmutableHash::new(
            entries
                .iter()
                .map(|(name, value)| (Value::Symbol(name.clone()), value.clone()))
                .collect(),
            self.symbol.clone(),
            self.object.clone(),
        ))
    }
}

impl DecoratorValue {
    pub fn read_field(&self, name: &str, types: &SnapshotTypes) -> Option<Value> {
        match self {
            Self::Invocation(record) => invocation_field(record, name, types),
            Self::InvocationSignature(record) => match name {
                "parameters" => Some(
                    types.array(
                        record
                            .parameters()
                            .iter()
                            .cloned()
                            .map(|parameter| Self::InvocationParameter(parameter).into())
                            .collect(),
                        &types.invocation_parameter,
                    ),
                ),
                "result" => Some(record.result().clone()),
                "is_async" => Some(Value::Bool(record.is_async())),
                _ => None,
            },
            Self::InvocationParameter(record) => match name {
                "name" => Some(Value::Symbol(record.name().into())),
                "category" => Some(Value::Symbol(record.category().symbol().into())),
                "type" => Some(record.parameter_type().clone()),
                "optional" => Some(Value::Bool(record.optional())),
                _ => None,
            },
            Self::Context(record) => match name {
                "kind" => Some(Value::Symbol(record.kind().symbol().into())),
                "reason" => Some(Value::Symbol(record.reason().symbol().into())),
                _ => None,
            },
            Self::Plan(record) => match name {
                "kind" => Some(Value::Symbol(record.kind().symbol().into())),
                _ => None,
            },
            Self::Transformation(record) => match name {
                "kind" => Some(Value::Symbol(record.kind().symbol().into())),
                _ => None,
            },
            Self::ProtocolError(record) => match name {
                "category" => Some(Value::Symbol(record.category().symbol().into())),
                "owner" => Some(record.owner_value()),
                _ => None,
            },
            Self::ArgumentChanges(_) => None,
        }
    }
}

fn invocation_field(record: &Invocation, name: &str, types: &SnapshotTypes) -> Option<Value> {
    let selected = record.selected();
    let payload = record.payload();
    let source = record.source();
    Some(match name {
        "receiver" => selected.receiver().clone(),
        "slot" => Value::Tuple(vec![
            selected.slot().declaring().clone(),
            Value::Symbol(selected.slot().selector().into()),
            selected.slot().qualifier().cloned().unwrap_or(Value::Nil),
            Value::Symbol(selected.slot().kind().symbol().into()),
        ]),
        "signature" => DecoratorValue::InvocationSignature(selected.signature().clone()).into(),
        "owner_type_arguments" => {
            types.array(selected.owner_type_arguments().to_vec(), &types.type_type)
        }
        "method_type_arguments" => {
            types.array(selected.method_type_arguments().to_vec(), &types.type_type)
        }
        "positional" => types.hash(payload.positional()),
        "keywords" => types.hash(payload.keywords()),
        "rest" => types.array(payload.rest().to_vec(), &types.object),
        "keyword_rest" => types.hash(payload.keyword_rest()),
        "block" => payload.block().clone(),
        "original_positional" => types.array(source.positional().to_vec(), &types.object),
        "original_keywords" => types.array(
            source
                .keywords()
                .iter()
                .map(|(name, value)| Value::Tuple(vec![Value::Symbol(name.clone()), value.clone()]))
                .collect(),
            &types.symbol_object_tuple,
        ),
        "original_block" => source.block().clone(),
        "block_omitted" => Value::Bool(source.block_omitted()),
        "omitted" => types.symbols(source.omitted()),
        "defaulted" => types.symbols(source.defaulted()),
        "replaced" => types.symbols(payload.replaced()),
        _ => return None,
    })
}
