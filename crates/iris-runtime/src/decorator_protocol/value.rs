use super::{
    ArgumentChanges, DecoratorContext, DecoratorProtocolError, Invocation, InvocationParameter,
    InvocationSignature, Plan, Transformation,
};
use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum DecoratorValue {
    Invocation(Box<Invocation>),
    InvocationSignature(InvocationSignature),
    InvocationParameter(InvocationParameter),
    ArgumentChanges(ArgumentChanges),
    Context(DecoratorContext),
    Plan(Plan),
    Transformation(Transformation),
    ProtocolError(DecoratorProtocolError),
}

impl DecoratorValue {
    pub const fn core_name(&self) -> &'static str {
        match self {
            Self::Invocation(_) => "Invocation",
            Self::InvocationSignature(_) => "InvocationSignature",
            Self::InvocationParameter(_) => "InvocationParameter",
            Self::ArgumentChanges(_) => "ArgumentChanges",
            Self::Context(_) => "DecoratorContext",
            Self::Plan(_) => "Plan",
            Self::Transformation(_) => "Transformation",
            Self::ProtocolError(_) => "DecoratorProtocolError",
        }
    }

    pub fn visit_values(&self, visitor: &mut impl FnMut(&Value)) {
        match self {
            Self::Invocation(record) => record.visit_values(visitor),
            Self::InvocationSignature(record) => record.visit_values(visitor),
            Self::InvocationParameter(record) => visitor(record.parameter_type()),
            Self::ArgumentChanges(record) => record.visit_values(visitor),
            Self::Transformation(record) => record.visit_values(visitor),
            Self::ProtocolError(record) => visitor(&record.owner_value()),
            Self::Context(_) | Self::Plan(_) => {}
        }
    }
}

impl From<DecoratorValue> for Value {
    fn from(record: DecoratorValue) -> Self {
        Self::Decorator(Box::new(record))
    }
}
