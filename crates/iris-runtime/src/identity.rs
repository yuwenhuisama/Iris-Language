macro_rules! define_id {
    ($name:ident) => {
        /// Opaque runtime identity.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name(u64);

        impl $name {
            /// Creates an identity from a runtime-controlled raw value.
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }
        }
    };
}

define_id!(ObjectId);
define_id!(ClassId);
define_id!(RevisionId);
define_id!(MethodId);
define_id!(ModuleId);
define_id!(Selector);
