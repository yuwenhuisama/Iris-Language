use crate::{ClassId, MethodId, ModuleId, ObjectId, Selector};

/// Opaque executable-body identity supplied by the evaluator in a later milestone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MethodBody(u64);

impl MethodBody {
    /// Creates a runtime-controlled body identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(&self) -> u64 {
        self.0
    }
}

/// Access policy declared for a Method slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// The Method may be invoked by ordinary external dispatch.
    Public,
    /// The Method is found but unavailable to ordinary external dispatch.
    Private,
    /// The Method is available only to implementation code in its nominal hierarchy.
    Protected,
}

/// Immutable identity-bearing Method definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Method {
    id: MethodId,
    owner: MethodOwner,
    selector: Selector,
    body: MethodBody,
    visibility: Visibility,
}

impl Method {
    pub(crate) const fn new(
        id: MethodId,
        owner: MethodOwner,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
    ) -> Self {
        Self {
            id,
            owner,
            selector,
            body,
            visibility,
        }
    }

    /// Returns this Method's runtime identity.
    pub const fn id(&self) -> MethodId {
        self.id
    }

    /// Returns the lexical Class or Module owner retained by this Method.
    pub const fn owner(&self) -> MethodOwner {
        self.owner
    }

    /// Returns the complete ordinary selector installed by this Method.
    pub const fn selector(&self) -> Selector {
        self.selector
    }

    /// Returns the immutable evaluator body selected at call entry.
    pub const fn body(&self) -> MethodBody {
        self.body
    }

    pub(crate) const fn visibility(&self) -> Visibility {
        self.visibility
    }
}

/// The lexical definition owner retained by a Method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodOwner {
    /// A Class definition owns the Method.
    Class(ClassId),
    /// A Module definition owns the Method.
    Module(ModuleId),
}

/// Identity-bearing pairing of an object receiver and one exact Method.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundMethod {
    id: ObjectId,
    receiver: ClassId,
    method: Method,
}

impl BoundMethod {
    pub(crate) const fn new(id: ObjectId, receiver: ClassId, method: Method) -> Self {
        Self {
            id,
            receiver,
            method,
        }
    }

    /// Returns this BoundMethod's distinct runtime identity.
    pub const fn id(&self) -> ObjectId {
        self.id
    }

    /// Returns the captured receiver identity.
    pub const fn receiver(&self) -> ClassId {
        self.receiver
    }

    /// Returns the exact Method identity selected at binding time.
    pub const fn method(&self) -> Method {
        self.method
    }
}
