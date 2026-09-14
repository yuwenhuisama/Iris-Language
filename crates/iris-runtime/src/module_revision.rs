use std::collections::HashMap;

use crate::{
    CompositionEdge, MetaCapabilities, Method, MethodBody, ModuleId, Selector, Visibility,
};

/// Runtime-local Module revision identity, distinct from Class revision identities.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ModuleRevisionId(pub(crate) u64);

impl ModuleRevisionId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Immutable published Module metadata. Legacy registration uses commit zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleRevision {
    pub(crate) id: ModuleRevisionId,
    pub(crate) owner: ModuleId,
    pub(crate) number: u64,
    pub(crate) commit_id: u64,
    pub(crate) components: Vec<CompositionEdge>,
    pub(crate) methods: HashMap<Selector, Method>,
    pub(crate) meta_capabilities: MetaCapabilities,
}

impl ModuleRevision {
    pub const fn id(&self) -> ModuleRevisionId {
        self.id
    }
    pub const fn owner(&self) -> ModuleId {
        self.owner
    }
    pub const fn number(&self) -> u64 {
        self.number
    }
    pub const fn commit_id(&self) -> u64 {
        self.commit_id
    }
    pub const fn meta_capabilities(&self) -> MetaCapabilities {
        self.meta_capabilities
    }
    pub fn composition_edges(&self) -> &[CompositionEdge] {
        &self.components
    }
    pub fn method(&self, selector: Selector) -> Option<Method> {
        self.methods.get(&selector).copied()
    }
}

/// Backend-only candidate view, not a published revision or reflection grant.
#[derive(Debug)]
pub struct CandidateModule {
    pub(crate) owner: ModuleId,
    pub(crate) base: Option<ModuleRevisionId>,
    pub(crate) number: u64,
    pub(crate) components: Vec<CompositionEdge>,
    pub(crate) methods: HashMap<Selector, Method>,
    pub(crate) meta_capabilities: MetaCapabilities,
    pub(crate) required: Vec<crate::Capability>,
    pub(crate) additions: std::collections::HashSet<Selector>,
    pub(crate) provisional_methods: std::collections::HashSet<crate::MethodId>,
}

impl CandidateModule {
    pub const fn owner(&self) -> ModuleId {
        self.owner
    }
    pub const fn base_revision(&self) -> Option<ModuleRevisionId> {
        self.base
    }
    pub const fn meta_capabilities(&self) -> MetaCapabilities {
        self.meta_capabilities
    }
    pub fn composition_edges(&self) -> &[CompositionEdge] {
        &self.components
    }
    pub fn method(&self, selector: Selector) -> Option<Method> {
        self.methods.get(&selector).copied()
    }
}

/// Backend-validated slot definition; signatures and executable bodies remain backend-owned.
#[derive(Clone, Copy, Debug)]
pub struct ModuleMethodDefinition {
    pub(crate) selector: Selector,
    pub(crate) body: MethodBody,
    pub(crate) visibility: Visibility,
}

impl ModuleMethodDefinition {
    pub const fn new(selector: Selector, body: MethodBody, visibility: Visibility) -> Self {
        Self {
            selector,
            body,
            visibility,
        }
    }
}
