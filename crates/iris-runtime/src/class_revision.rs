use std::collections::{BTreeMap, BTreeSet};

use crate::{ClassId, MethodId, ModuleId, RevisionId, Selector};

/// Immutable nominal promises established by a class origin declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticSpine(u64);

impl StaticSpine {
    /// Creates a runtime-owned static-spine identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
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
    mro: Vec<ClassId>,
    modules: Vec<ModuleId>,
    methods: BTreeMap<Selector, MethodId>,
    properties: BTreeSet<Selector>,
    class_vars: BTreeSet<Selector>,
}

impl ClassRevision {
    pub(crate) fn from_candidate(
        candidate: CandidateRevision,
        id: RevisionId,
        commit_id: u64,
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
            methods: candidate.methods,
            properties: candidate.properties,
            class_vars: candidate.class_vars,
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

    /// Returns the computed MRO slot; its linearization is deferred.
    pub fn mro(&self) -> &[ClassId] {
        &self.mro
    }

    /// Returns composed modules in this immutable revision.
    pub fn modules(&self) -> &[ModuleId] {
        &self.modules
    }

    /// Returns the inert method-table metadata slot.
    pub const fn methods(&self) -> &BTreeMap<Selector, MethodId> {
        &self.methods
    }

    /// Returns the inert property-table metadata slot.
    pub const fn properties(&self) -> &BTreeSet<Selector> {
        &self.properties
    }

    /// Returns the inert class-variable-table metadata slot.
    pub const fn class_vars(&self) -> &BTreeSet<Selector> {
        &self.class_vars
    }
}

/// Unpublished, mutable class metadata derived from one active revision.
#[derive(Clone, Debug)]
pub struct CandidateRevision {
    pub(crate) base: RevisionId,
    pub(crate) owner: ClassId,
    pub(crate) number: u64,
    pub(crate) static_spine: StaticSpine,
    pub(crate) runtime_superclass: Option<ClassId>,
    pub(crate) mro: Vec<ClassId>,
    pub(crate) modules: Vec<ModuleId>,
    pub(crate) methods: BTreeMap<Selector, MethodId>,
    pub(crate) properties: BTreeSet<Selector>,
    pub(crate) class_vars: BTreeSet<Selector>,
}

impl CandidateRevision {
    pub(crate) fn from_revision(revision: &ClassRevision) -> Option<Self> {
        revision.number.checked_add(1).map(|number| Self {
            base: revision.id,
            owner: revision.owner,
            number,
            static_spine: revision.static_spine,
            runtime_superclass: revision.runtime_superclass,
            mro: revision.mro.clone(),
            modules: revision.modules.clone(),
            methods: revision.methods.clone(),
            properties: revision.properties.clone(),
            class_vars: revision.class_vars.clone(),
        })
    }

    /// Adds one Module edge to the candidate metadata.
    pub fn add_module(&mut self, module: ModuleId) {
        self.modules.push(module);
    }

    /// Sets the deferred MRO result for a later MRO computation step.
    pub fn replace_mro(&mut self, mro: Vec<ClassId>) {
        self.mro = mro;
    }

    /// Replaces the candidate runtime superclass before validation.
    pub fn replace_runtime_superclass(&mut self, superclass: Option<ClassId>) {
        self.runtime_superclass = superclass;
    }

    /// Replaces the candidate static spine; publication rejects a changed spine.
    pub fn replace_static_spine(&mut self, spine: StaticSpine) {
        self.static_spine = spine;
    }
}
