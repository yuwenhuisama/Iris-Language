use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppliedDecorator, CandidateRevision, ClassId, MetaCapabilities, MethodBody, MethodId, ModuleId,
    RevisionId, Selector,
};

/// One revision-owned stored property initializer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoredProperty {
    selector: Selector,
    initializer: MethodBody,
}

impl StoredProperty {
    pub(crate) const fn new(selector: Selector, initializer: MethodBody) -> Self {
        Self {
            selector,
            initializer,
        }
    }

    /// Returns the declared property's selector.
    pub const fn selector(&self) -> Selector {
        self.selector
    }

    /// Returns the evaluator body that initializes this stored property.
    pub const fn initializer(&self) -> MethodBody {
        self.initializer
    }
}

/// One precomputed member in a Class revision's lookup linearization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MroEntry {
    /// A logical Class member.
    Class(ClassId),
    /// A composed Module member.
    Module(ModuleId),
}

impl From<ClassId> for MroEntry {
    fn from(value: ClassId) -> Self {
        Self::Class(value)
    }
}

impl From<ModuleId> for MroEntry {
    fn from(value: ModuleId) -> Self {
        Self::Module(value)
    }
}

/// Immutable nominal promises established by a class origin declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticSpine {
    identity: u64,
    meta_capabilities: MetaCapabilities,
}

/// Immutable authorization metadata for one composition edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompositionEdge {
    module: ModuleId,
    private_access: bool,
}

impl CompositionEdge {
    pub const fn new(module: ModuleId, private_access: bool) -> Self {
        Self {
            module,
            private_access,
        }
    }

    pub const fn module(self) -> ModuleId {
        self.module
    }

    pub const fn private_access(self) -> bool {
        self.private_access
    }
}

impl StaticSpine {
    /// Creates a runtime-owned static-spine identity.
    pub const fn new(raw: u64) -> Self {
        Self {
            identity: raw,
            meta_capabilities: MetaCapabilities::all(),
        }
    }

    /// The spine's own identity.
    ///
    /// `IRIS-V1-META-C097` lists `static_spine` on the Class reflection view,
    /// which V424 reads.
    pub const fn identity(self) -> u64 {
        self.identity
    }

    /// Sets the immutable origin policy before the Class is defined.
    pub const fn with_meta_capabilities(self, meta_capabilities: MetaCapabilities) -> Self {
        Self {
            identity: self.identity,
            meta_capabilities,
        }
    }

    /// Returns the immutable policy declared by this Class origin.
    pub const fn meta_capabilities(self) -> MetaCapabilities {
        self.meta_capabilities
    }
}

/// The stable identity visible to Iris programs for one Class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogicalClass {
    id: ClassId,
    active: RevisionId,
}

impl LogicalClass {
    pub(crate) const fn new(id: ClassId, active: RevisionId) -> Self {
        Self { id, active }
    }

    /// Returns the stable Class identity.
    pub const fn id(&self) -> ClassId {
        self.id
    }

    /// Returns the sole revision currently observed by ordinary Class reads.
    pub const fn active_revision(&self) -> RevisionId {
        self.active
    }

    pub(crate) fn publish(&mut self, revision: RevisionId) {
        self.active = revision;
    }
}

/// Read-only, identity-bearing metadata for one published Class state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassRevision {
    id: RevisionId,
    owner: ClassId,
    number: u64,
    commit_id: u64,
    static_spine: StaticSpine,
    runtime_superclass: Option<ClassId>,
    mro: Vec<MroEntry>,
    modules: Vec<ModuleId>,
    composition_edges: Vec<CompositionEdge>,
    methods: BTreeMap<Selector, MethodId>,
    tombstones: BTreeSet<Selector>,
    singleton_methods: BTreeMap<Selector, MethodId>,
    properties: Vec<StoredProperty>,
    class_vars: BTreeSet<Selector>,
    immutable_class_vars: BTreeSet<Selector>,
    meta_capabilities: MetaCapabilities,
    decorators: Vec<AppliedDecorator>,
}

impl ClassRevision {
    pub(crate) fn from_candidate(
        candidate: CandidateRevision,
        id: RevisionId,
        commit_id: u64,
        meta_capabilities: MetaCapabilities,
    ) -> Self {
        Self {
            id,
            owner: candidate.owner,
            number: candidate.number,
            commit_id,
            static_spine: candidate.static_spine,
            runtime_superclass: candidate.runtime_superclass,
            mro: candidate.mro,
            modules: candidate.modules,
            composition_edges: candidate.composition_edges,
            methods: candidate.methods,
            tombstones: candidate.tombstones,
            singleton_methods: candidate.singleton_methods,
            properties: candidate.properties,
            class_vars: candidate.class_vars,
            immutable_class_vars: candidate.immutable_class_vars,
            meta_capabilities,
            decorators: candidate.decorators,
        }
    }

    /// Returns this revision's opaque runtime identity.
    pub const fn id(&self) -> RevisionId {
        self.id
    }

    /// Returns the logical Class that owns this revision.
    pub const fn owner(&self) -> ClassId {
        self.owner
    }

    /// Returns the monotonically increasing number within its Class history.
    pub const fn number(&self) -> u64 {
        self.number
    }

    /// Returns this structural transaction group's runtime-local commit identifier.
    pub const fn commit_id(&self) -> u64 {
        self.commit_id
    }

    /// Returns the immutable nominal promises this revision preserves.
    pub const fn static_spine(&self) -> StaticSpine {
        self.static_spine
    }

    /// Returns the revision's runtime superclass.
    pub const fn runtime_superclass(&self) -> Option<ClassId> {
        self.runtime_superclass
    }

    /// Returns the precomputed MRO stored when this revision was published.
    pub fn mro(&self) -> &[MroEntry] {
        &self.mro
    }

    /// Returns composed modules in this immutable revision.
    pub fn modules(&self) -> &[ModuleId] {
        &self.modules
    }

    pub fn composition_edges(&self) -> &[CompositionEdge] {
        &self.composition_edges
    }

    /// Returns the inert method-table metadata slot.
    pub const fn methods(&self) -> &BTreeMap<Selector, MethodId> {
        &self.methods
    }

    /// Returns selectors whose local tombstones block ancestor lookup.
    pub const fn tombstones(&self) -> &BTreeSet<Selector> {
        &self.tombstones
    }

    /// Returns singleton Methods installed on this Class object.
    pub const fn singleton_methods(&self) -> &BTreeMap<Selector, MethodId> {
        &self.singleton_methods
    }

    /// Returns the inert property-table metadata slot.
    pub fn properties(&self) -> &[StoredProperty] {
        &self.properties
    }

    /// Returns the inert class-variable-table metadata slot.
    pub const fn class_vars(&self) -> &BTreeSet<Selector> {
        &self.class_vars
    }

    /// Returns immutable hierarchy Class-variable cells.
    pub const fn immutable_class_vars(&self) -> &BTreeSet<Selector> {
        &self.immutable_class_vars
    }

    /// Returns this revision's immutable effective meta-operation policy.
    pub const fn meta_capabilities(&self) -> MetaCapabilities {
        self.meta_capabilities
    }

    /// Returns decorators applied to this revision in source order.
    pub fn decorators(&self) -> &[AppliedDecorator] {
        &self.decorators
    }
}
