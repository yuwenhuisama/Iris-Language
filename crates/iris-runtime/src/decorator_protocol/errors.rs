use super::{ChangeField, DecoratorKind, ParameterCategory};
use crate::{ObjectId, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolCategory {
    OutsidePhase,
    Expired,
    ForeignTask,
    Overlap,
    UnfinishedInner,
}

impl ProtocolCategory {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::OutsidePhase => "outside_phase",
            Self::Expired => "expired",
            Self::ForeignTask => "foreign_task",
            Self::Overlap => "overlap",
            Self::UnfinishedInner => "unfinished_inner",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecoratorProtocolError {
    category: ProtocolCategory,
    owner: Option<ObjectId>,
}

impl DecoratorProtocolError {
    pub(super) const fn new(category: ProtocolCategory, owner: Option<ObjectId>) -> Self {
        Self { category, owner }
    }
    pub const fn category(&self) -> ProtocolCategory {
        self.category
    }
    pub const fn owner(&self) -> Option<ObjectId> {
        self.owner
    }
    pub fn owner_value(&self) -> Value {
        self.owner.map_or(Value::Nil, Value::Closure)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KindError {
    expected: DecoratorKind,
    actual: DecoratorKind,
}

impl KindError {
    pub(super) const fn new(expected: DecoratorKind, actual: DecoratorKind) -> Self {
        Self { expected, actual }
    }
    pub const fn expected(&self) -> DecoratorKind {
        self.expected
    }
    pub const fn actual(&self) -> DecoratorKind {
        self.actual
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstructionError {
    Protocol(DecoratorProtocolError),
    Kind(KindError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentError {
    UnknownKeyword(String),
    DuplicateKeyword(String),
    WrongShape(ChangeField),
    NonSymbolKey(ChangeField),
    DuplicateName(String),
    UnknownName(String),
    WrongChannel {
        name: String,
        channel: ParameterCategory,
    },
    MissingChannel(ParameterCategory),
    MissingBinding(String),
    BindingCount {
        expected: usize,
        actual: usize,
    },
    InvalidParameterOrder(String),
    InvalidOptionality(String),
    TypeMismatch(String),
}

impl ArgumentError {
    pub const fn is_type_error(&self) -> bool {
        match self {
            Self::WrongShape(_) | Self::NonSymbolKey(_) | Self::TypeMismatch(_) => true,
            Self::UnknownKeyword(_)
            | Self::DuplicateKeyword(_)
            | Self::DuplicateName(_)
            | Self::UnknownName(_)
            | Self::WrongChannel { .. }
            | Self::MissingChannel(_)
            | Self::MissingBinding(_)
            | Self::BindingCount { .. }
            | Self::InvalidParameterOrder(_)
            | Self::InvalidOptionality(_) => false,
        }
    }
}

macro_rules! debug_error {
    ($($name:ty),+ $(,)?) => { $(
        impl core::fmt::Display for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(formatter, "{self:?}")
            }
        }
        impl std::error::Error for $name {}
    )+ };
}

debug_error!(
    DecoratorProtocolError,
    ConstructionError,
    ArgumentError,
    KindError
);
