use super::{
    ConstructionError, DecoratorKind, DecoratorProtocolError, DecoratorReason, KindError,
    ProtocolCategory,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecoratorContext {
    kind: DecoratorKind,
    reason: DecoratorReason,
}

impl DecoratorContext {
    pub const fn kind(&self) -> DecoratorKind {
        self.kind
    }
    pub const fn reason(&self) -> DecoratorReason {
        self.reason
    }
}

/// Backend-owned execution extent. A context snapshot is not this authority.
#[derive(Debug)]
pub struct DecoratorPhase {
    context: DecoratorContext,
    active: bool,
}

impl DecoratorPhase {
    pub const fn new(kind: DecoratorKind, reason: DecoratorReason) -> Self {
        Self {
            context: DecoratorContext { kind, reason },
            active: true,
        }
    }
    pub const fn context(&self) -> DecoratorContext {
        self.context
    }
    pub fn finish(&mut self) {
        self.active = false;
    }

    pub(super) fn kind(phase: Option<&Self>) -> Result<DecoratorKind, ConstructionError> {
        match phase {
            Some(phase) if phase.active => Ok(phase.context.kind),
            Some(_) | None => Err(ConstructionError::Protocol(DecoratorProtocolError::new(
                ProtocolCategory::OutsidePhase,
                None,
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    kind: DecoratorKind,
}

impl Plan {
    pub fn empty(phase: Option<&DecoratorPhase>) -> Result<Self, ConstructionError> {
        Ok(Self {
            kind: DecoratorPhase::kind(phase)?,
        })
    }
    pub const fn kind(&self) -> DecoratorKind {
        self.kind
    }
    pub fn validate_kind(&self, target: DecoratorKind) -> Result<(), KindError> {
        if self.kind == target {
            Ok(())
        } else {
            Err(KindError::new(target, self.kind))
        }
    }
}
