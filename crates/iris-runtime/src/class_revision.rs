use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppliedDecorator, Capability, ClassId, DecoratorTransform, MetaCapabilities, MethodBody,
    MethodId, ModuleId, RevisionId, Selector,
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
    mro: Vec<MroEntry>,
    modules: Vec<ModuleId>,
    methods: BTreeMap<Selector, MethodId>,
    singleton_methods: BTreeMap<Selector, MethodId>,
    properties: Vec<StoredProperty>,
    class_vars: BTreeSet<Selector>,
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
            methods: candidate.methods,
            singleton_methods: candidate.singleton_methods,
            properties: candidate.properties,
            class_vars: candidate.class_vars,
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

    /// Returns the inert method-table metadata slot.
    pub const fn methods(&self) -> &BTreeMap<Selector, MethodId> {
        &self.methods
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

    /// Returns this revision's immutable effective meta-operation policy.
    pub const fn meta_capabilities(&self) -> MetaCapabilities {
        self.meta_capabilities
    }

    /// Returns decorators applied to this revision in source order.
    pub fn decorators(&self) -> &[AppliedDecorator] {
        &self.decorators
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
    pub(crate) mro: Vec<MroEntry>,
    pub(crate) modules: Vec<ModuleId>,
    pub(crate) methods: BTreeMap<Selector, MethodId>,
    pub(crate) singleton_methods: BTreeMap<Selector, MethodId>,
    pub(crate) properties: Vec<StoredProperty>,
    pub(crate) class_vars: BTreeSet<Selector>,
    pub(crate) meta_capabilities: MetaCapabilities,
    pub(crate) decorators: Vec<AppliedDecorator>,
    pub(crate) pending_decorators: Vec<DecoratorTransform>,
}

impl CandidateRevision {
    pub(crate) fn origin(
        owner: ClassId,
        base: RevisionId,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
        mro: Vec<MroEntry>,
    ) -> Self {
        Self {
            base,
            owner,
            number: 1,
            static_spine,
            runtime_superclass,
            mro,
            modules: Vec::new(),
            methods: BTreeMap::new(),
            singleton_methods: BTreeMap::new(),
            properties: Vec::new(),
            class_vars: BTreeSet::new(),
            meta_capabilities: MetaCapabilities::all(),
            decorators: Vec::new(),
            pending_decorators: Vec::new(),
        }
    }

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
            singleton_methods: revision.singleton_methods.clone(),
            properties: revision.properties.clone(),
            class_vars: revision.class_vars.clone(),
            meta_capabilities: revision.meta_capabilities,
            decorators: Vec::new(),
            pending_decorators: revision
                .decorators
                .iter()
                .cloned()
                .map(|decorator| DecoratorTransform::Metadata {
                    identity: decorator.identity,
                    arguments: decorator.arguments,
                })
                .collect(),
        })
    }

    /// Adds one Module edge to the candidate metadata.
    pub fn add_module(&mut self, module: ModuleId) {
        self.modules.push(module);
    }

    /// Removes a Module edge from this candidate before publication.
    pub fn remove_module(&mut self, module: ModuleId) {
        self.modules.retain(|candidate| *candidate != module);
    }

    /// Sets an MRO value that publication recomputes before storing.
    pub fn replace_mro(&mut self, mro: Vec<MroEntry>) {
        self.mro = mro;
    }

    pub(crate) fn replace_method(&mut self, selector: Selector, method: MethodId) {
        self.methods.insert(selector, method);
    }

    pub(crate) fn replace_singleton_method(&mut self, selector: Selector, method: MethodId) {
        self.singleton_methods.insert(selector, method);
    }

    pub(crate) fn add_stored_property(&mut self, property: StoredProperty) {
        if let Some(existing) = self
            .properties
            .iter_mut()
            .find(|existing| existing.selector() == property.selector())
        {
            *existing = property;
        } else {
            self.properties.push(property);
        }
    }

    pub(crate) fn restore(&mut self, artifact: &ClassRevision) {
        self.runtime_superclass = artifact.runtime_superclass();
        self.modules = artifact.modules().to_vec();
        self.methods = artifact.methods().clone();
        self.singleton_methods = artifact.singleton_methods().clone();
        self.properties = artifact.properties().to_vec();
        self.class_vars = artifact.class_vars().clone();
    }

    /// Replaces the candidate runtime superclass before validation.
    pub fn replace_runtime_superclass(&mut self, superclass: Option<ClassId>) {
        self.runtime_superclass = superclass;
    }

    /// Replaces the candidate static spine; publication rejects a changed spine.
    pub fn replace_static_spine(&mut self, spine: StaticSpine) {
        self.static_spine = spine;
    }

    /// Applies source-level meta denies, which can only narrow authorization.
    pub fn deny_meta(&mut self, capabilities: &[Capability]) {
        self.meta_capabilities.deny(capabilities);
    }

    pub(crate) fn stage_decorators(
        &mut self,
        decorators: impl IntoIterator<Item = DecoratorTransform>,
    ) {
        self.pending_decorators.extend(decorators);
    }
}
