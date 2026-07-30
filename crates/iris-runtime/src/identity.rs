macro_rules! define_id {
    ($name:ident) => {
        /// Opaque runtime identity.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            /// Creates an identity from a runtime-controlled raw value.
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            /// Exposes the raw identity for cross-namespace diagnostics.
            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

define_id!(ObjectId);
define_id!(ClassId);
define_id!(RevisionId);
define_id!(MethodId);
define_id!(ModuleId);
define_id!(ContractId);
define_id!(Selector);

impl Selector {
    /// The reserved selector for `initialize`, dispatched by construction.
    ///
    /// It sits ABOVE every `NativeSelector` id because those start at 1, and a
    /// shared id would make declaring `+` register a method that construction
    /// then invokes with no arguments.
    pub const INITIALIZE: Self = Self::new(900);
}
