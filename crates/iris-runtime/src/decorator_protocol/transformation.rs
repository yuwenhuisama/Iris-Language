use super::{ConstructionError, DecoratorKind, DecoratorPhase, KindError};
use crate::Value;

/// Inert payloads; actual Closure metadata and publication checks belong to the backend.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    AddMethod { selector: String, body: Value },
    WrapMethod(Value),
    WrapGetter(Value),
    WrapSetter(Value),
}

impl Operation {
    pub const fn callable(&self) -> &Value {
        match self {
            Self::AddMethod { body, .. } => body,
            Self::WrapMethod(wrapper) | Self::WrapGetter(wrapper) | Self::WrapSetter(wrapper) => {
                wrapper
            }
        }
    }
    fn admits(&self, kind: DecoratorKind) -> bool {
        match self {
            Self::AddMethod { .. } => matches!(kind, DecoratorKind::Class | DecoratorKind::Module),
            Self::WrapMethod(_) => kind == DecoratorKind::Method,
            Self::WrapGetter(_) | Self::WrapSetter(_) => kind == DecoratorKind::Property,
        }
    }
    fn required_kind(&self) -> DecoratorKind {
        match self {
            Self::AddMethod { .. } => DecoratorKind::Class,
            Self::WrapMethod(_) => DecoratorKind::Method,
            Self::WrapGetter(_) | Self::WrapSetter(_) => DecoratorKind::Property,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transformation {
    kind: DecoratorKind,
    operations: Vec<Operation>,
}

impl Transformation {
    /// Backend descriptor assembly, not a source factory or phase authorization.
    pub fn from_operations(
        kind: DecoratorKind,
        operations: Vec<Operation>,
    ) -> Result<Self, KindError> {
        for operation in &operations {
            if !operation.admits(kind) {
                return Err(KindError::new(operation.required_kind(), kind));
            }
        }
        Ok(Self { kind, operations })
    }
    pub fn empty(phase: Option<&DecoratorPhase>) -> Result<Self, ConstructionError> {
        Ok(Self {
            kind: DecoratorPhase::kind(phase)?,
            operations: Vec::new(),
        })
    }
    pub const fn kind(&self) -> DecoratorKind {
        self.kind
    }
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    pub fn append(
        &self,
        phase: Option<&DecoratorPhase>,
        operation: Operation,
    ) -> Result<Self, ConstructionError> {
        let kind = DecoratorPhase::kind(phase)?;
        self.validate_kind(kind).map_err(ConstructionError::Kind)?;
        let mut operations = self.operations.clone();
        operations.push(operation);
        Self::from_operations(kind, operations).map_err(ConstructionError::Kind)
    }

    pub fn validate_kind(&self, target: DecoratorKind) -> Result<(), KindError> {
        if self.kind == target {
            Ok(())
        } else {
            Err(KindError::new(target, self.kind))
        }
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        for operation in &self.operations {
            visitor(operation.callable());
        }
    }
}
