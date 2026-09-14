use crate::Visibility;

/// A written stored-property accessor block, including an explicitly empty one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyAccessors {
    /// Members in source order. The grammar permits repetition; syntax retains
    /// it rather than selecting a winner or inventing a duplicate diagnostic.
    pub members: Vec<PropertyAccessor>,
}

/// One generated accessor declaration, not a new callable runtime kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PropertyAccessor {
    pub kind: PropertyAccessorKind,
    /// Accessor-local visibility, defaulting to private under D-445.
    pub visibility: Visibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyAccessorKind {
    Get,
    Set,
}
