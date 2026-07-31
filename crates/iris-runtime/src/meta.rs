/// One operation-level authorization in a Class revision policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    MethodSet,
    MethodBody,
    PropertySet,
    PropertyBody,
    Modules,
    Superclass,
    Subclass,
    Shape,
    ClassStateSet,
    ClassStateWrite,
    InstanceState,
    Native,
}

/// Stable built-in Class categories with protected runtime superclasses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltinClass {
    Object,
    Nil,
    Bool,
    Integer,
    Float32,
    Float64,
    /// `IRIS-V1-COLLECTIONS-C041` makes a String an identity-less immutable
    /// sequence of Unicode scalar values, so it is a value Class alongside the
    /// numeric ones rather than an ordinary Object.
    String,
}

/// Immutable effective meta-operation policy stored on each revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetaCapabilities(u16);

impl MetaCapabilities {
    const METHOD_SET: u16 = 1;
    const METHOD_BODY: u16 = 2;
    const PROPERTY_SET: u16 = 4;
    const PROPERTY_BODY: u16 = 8;
    const MODULES: u16 = 16;
    const SUPERCLASS: u16 = 32;
    const SUBCLASS: u16 = 64;
    const SHAPE: u16 = 128;
    const CLASS_STATE_SET: u16 = 256;
    const CLASS_STATE_WRITE: u16 = 512;
    const INSTANCE_STATE: u16 = 1024;
    const NATIVE: u16 = 2048;

    /// Returns the default policy allowing all currently modeled operations.
    pub const fn all() -> Self {
        Self(
            Self::METHOD_SET
                | Self::METHOD_BODY
                | Self::PROPERTY_SET
                | Self::PROPERTY_BODY
                | Self::MODULES
                | Self::SUPERCLASS
                | Self::SUBCLASS
                | Self::SHAPE
                | Self::CLASS_STATE_SET
                | Self::CLASS_STATE_WRITE
                | Self::INSTANCE_STATE
                | Self::NATIVE,
        )
    }

    /// Builds an immutable policy by removing source-declared capabilities.
    pub fn denying(capabilities: &[Capability]) -> Self {
        let mut policy = Self::all();
        policy.deny(capabilities);
        policy
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

    /// Combines independent denials without granting authorization.
    pub(crate) const fn narrowed_by(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    const fn bit(capability: Capability) -> u16 {
        match capability {
            Capability::MethodSet => Self::METHOD_SET,
            Capability::MethodBody => Self::METHOD_BODY,
            Capability::PropertySet => Self::PROPERTY_SET,
            Capability::PropertyBody => Self::PROPERTY_BODY,
            Capability::Modules => Self::MODULES,
            Capability::Superclass => Self::SUPERCLASS,
            Capability::Subclass => Self::SUBCLASS,
            Capability::Shape => Self::SHAPE,
            Capability::ClassStateSet => Self::CLASS_STATE_SET,
            Capability::ClassStateWrite => Self::CLASS_STATE_WRITE,
            Capability::InstanceState => Self::INSTANCE_STATE,
            Capability::Native => Self::NATIVE,
        }
    }
}
