use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppliedDecorator, Capability, ClassId, ClassRevision, CompositionEdge, DecoratorTransform,
    MetaCapabilities, MethodId, ModuleId, MroEntry, RevisionId, Selector, StaticSpine,
    StoredProperty,
};

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
    pub(crate) composition_edges: Vec<CompositionEdge>,
    pub(crate) methods: BTreeMap<Selector, MethodId>,
    pub(crate) tombstones: BTreeSet<Selector>,
    pub(crate) singleton_methods: BTreeMap<Selector, MethodId>,
    pub(crate) properties: Vec<StoredProperty>,
    pub(crate) class_vars: BTreeSet<Selector>,
    pub(crate) immutable_class_vars: BTreeSet<Selector>,
    pub(crate) meta_capabilities: MetaCapabilities,
    pub(crate) decorators: Vec<AppliedDecorator>,
    pub(crate) pending_decorators: Vec<DecoratorTransform>,
    pub(crate) required: Vec<Capability>,
    pub(crate) provisional_methods: BTreeSet<MethodId>,
}

impl CandidateRevision {
    pub(crate) fn origin(
        owner: ClassId,
        base: RevisionId,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
        mro: Vec<MroEntry>,
        meta_capabilities: MetaCapabilities,
    ) -> Self {
        Self {
            base,
            owner,
            number: 1,
            static_spine,
            runtime_superclass,
            mro,
            modules: Vec::new(),
            composition_edges: Vec::new(),
            methods: BTreeMap::new(),
            tombstones: BTreeSet::new(),
            singleton_methods: BTreeMap::new(),
            properties: Vec::new(),
            class_vars: BTreeSet::new(),
            immutable_class_vars: BTreeSet::new(),
            meta_capabilities,
            decorators: Vec::new(),
            pending_decorators: Vec::new(),
            required: Vec::new(),
            provisional_methods: BTreeSet::new(),
        }
    }

    pub(crate) fn from_revision(revision: &ClassRevision) -> Option<Self> {
        revision.number().checked_add(1).map(|number| Self {
            base: revision.id(),
            owner: revision.owner(),
            number,
            static_spine: revision.static_spine(),
            runtime_superclass: revision.runtime_superclass(),
            mro: revision.mro().to_vec(),
            modules: revision.modules().to_vec(),
            composition_edges: revision.composition_edges().to_vec(),
            methods: revision.methods().clone(),
            tombstones: revision.tombstones().clone(),
            singleton_methods: revision.singleton_methods().clone(),
            properties: revision.properties().to_vec(),
            class_vars: revision.class_vars().clone(),
            immutable_class_vars: revision.immutable_class_vars().clone(),
            meta_capabilities: revision.meta_capabilities(),
            decorators: Vec::new(),
            pending_decorators: revision
                .decorators()
                .iter()
                .cloned()
                .map(|decorator| DecoratorTransform::Metadata {
                    identity: decorator.identity,
                    arguments: decorator.arguments,
                })
                .collect(),
            required: Vec::new(),
            provisional_methods: BTreeSet::new(),
        })
    }

    /// The active revision this candidate was opened from.
    pub const fn base_revision(&self) -> RevisionId {
        self.base
    }

    pub fn add_module(&mut self, module: ModuleId) {
        self.add_composition_edge(CompositionEdge::new(module, false));
    }

    pub fn add_composition_edge(&mut self, edge: CompositionEdge) {
        self.modules.push(edge.module());
        self.composition_edges.push(edge);
    }

    /// Removes a Module edge from this candidate before publication.
    pub fn remove_module(&mut self, module: ModuleId) {
        self.modules.retain(|candidate| *candidate != module);
        self.composition_edges
            .retain(|candidate| candidate.module() != module);
    }

    /// Sets an MRO value that publication recomputes before storing.
    pub fn replace_mro(&mut self, mro: Vec<MroEntry>) {
        self.mro = mro;
    }

    pub(crate) fn replace_method(&mut self, selector: Selector, method: MethodId) {
        self.tombstones.remove(&selector);
        self.methods.insert(selector, method);
    }

    pub(crate) fn remove_method(&mut self, selector: Selector) {
        self.methods.remove(&selector);
        self.tombstones.remove(&selector);
    }

    pub(crate) fn undef_method(&mut self, selector: Selector) {
        self.methods.remove(&selector);
        self.tombstones.insert(selector);
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

    pub(crate) fn add_class_var(&mut self, name: Selector, mutable: bool) {
        self.class_vars.insert(name);
        if mutable {
            self.immutable_class_vars.remove(&name);
        } else {
            self.immutable_class_vars.insert(name);
        }
    }

    pub(crate) fn restore(&mut self, artifact: &ClassRevision) {
        self.runtime_superclass = artifact.runtime_superclass();
        self.modules = artifact.modules().to_vec();
        self.composition_edges = artifact.composition_edges().to_vec();
        self.methods = artifact.methods().clone();
        self.tombstones = artifact.tombstones().clone();
        self.singleton_methods = artifact.singleton_methods().clone();
        self.properties = artifact.properties().to_vec();
        self.class_vars = artifact.class_vars().clone();
        self.immutable_class_vars = artifact.immutable_class_vars().clone();
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
