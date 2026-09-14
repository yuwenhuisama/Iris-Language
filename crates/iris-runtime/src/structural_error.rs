use core::fmt;
use std::error::Error;

use crate::{Capability, ClassError, ModuleId, ModuleRevisionId, Selector};

/// Module transaction failures never encode a Module as a Class identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleCandidateError {
    UnknownModule(ModuleId),
    UnknownRevision(ModuleRevisionId),
    NotStaged(ModuleId),
    OriginRequired(ModuleId),
    IdentityExhausted,
    RevisionIdentityExhausted,
    RevisionNumberExhausted(ModuleId),
    MethodIdentityExhausted,
    Conflict(ModuleId),
    CompositionCycle(ModuleId),
    CompositionFrozen(ModuleId),
    PolicyFrozen(ModuleId),
    CapabilityDenied {
        module: ModuleId,
        operation: Capability,
    },
    PublicAdditionDenied {
        module: ModuleId,
        selector: Selector,
    },
    MethodCollision {
        module: ModuleId,
        selector: Selector,
    },
    MethodNotFound {
        module: ModuleId,
        selector: Selector,
    },
}

impl fmt::Display for ModuleCandidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Module candidate error: {self:?}")
    }
}
impl Error for ModuleCandidateError {}

/// Additive group error boundary; existing ClassError consumers remain unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeStructuralError {
    Class(ClassError),
    Module(ModuleCandidateError),
}

impl From<ClassError> for RuntimeStructuralError {
    fn from(error: ClassError) -> Self {
        Self::Class(error)
    }
}
impl From<ModuleCandidateError> for RuntimeStructuralError {
    fn from(error: ModuleCandidateError) -> Self {
        Self::Module(error)
    }
}
impl fmt::Display for RuntimeStructuralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(error) => error.fmt(formatter),
            Self::Module(error) => error.fmt(formatter),
        }
    }
}
impl Error for RuntimeStructuralError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Class(error) => Some(error),
            Self::Module(error) => Some(error),
        }
    }
}
