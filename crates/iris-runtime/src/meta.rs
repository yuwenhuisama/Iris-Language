/// One operation-level authorization in a Class revision policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    MethodSet,
    MethodBody,
    PropertySet,
    PropertyBody,
}

/// Stable built-in Class categories with protected runtime superclasses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltinClass {
    Nil,
    Bool,
    Integer,
    Float32,
    Float64,
}

/// Immutable effective meta-operation policy stored on each revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetaCapabilities(u8);

impl MetaCapabilities {
    const METHOD_SET: u8 = 1;
    const METHOD_BODY: u8 = 2;
    const PROPERTY_SET: u8 = 4;
    const PROPERTY_BODY: u8 = 8;

    /// Returns the default policy allowing all currently modeled operations.
    pub const fn all() -> Self {
        Self(Self::METHOD_SET | Self::METHOD_BODY | Self::PROPERTY_SET | Self::PROPERTY_BODY)
    }

    /// Returns whether one operation is permitted.
    pub const fn allows(self, capability: Capability) -> bool {
        self.0 & Self::bit(capability) != 0
    }

    /// Narrows this policy without any ability to grant authorization.
    pub(crate) fn deny(&mut self, capabilities: &[Capability]) {
        for capability in capabilities {
            self.0 &= !Self::bit(*capability);
        }
    }

    const fn bit(capability: Capability) -> u8 {
        match capability {
            Capability::MethodSet => Self::METHOD_SET,
            Capability::MethodBody => Self::METHOD_BODY,
            Capability::PropertySet => Self::PROPERTY_SET,
            Capability::PropertyBody => Self::PROPERTY_BODY,
        }
    }
}
