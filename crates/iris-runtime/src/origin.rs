use std::collections::HashSet;

use crate::{
    CandidateRevision, Capability, ClassError, ClassId, ClassRegistry, ClassRevision,
    CompositionEdge, MethodBody, MethodId, MroEntry, PolicyOrigin, RevisionId, Selector,
    StaticSpine, StoredProperty,
};

/// Backend-only, read-only access to an unpublished origin's structural metadata.
///
/// This is not a revision artifact and carries no revision or commit identity.
/// Ordinary Class lookup, dispatch and instance allocation cannot observe it.
#[derive(Debug)]
pub struct StagedOrigin<'registry> {
    candidate: &'registry CandidateRevision,
}

impl StagedOrigin<'_> {
    pub const fn static_spine(&self) -> StaticSpine {
        self.candidate.static_spine
    }

    pub const fn runtime_superclass(&self) -> Option<ClassId> {
        self.candidate.runtime_superclass
    }

    pub fn mro(&self) -> &[MroEntry] {
        &self.candidate.mro
    }

    pub fn composition_edges(&self) -> &[CompositionEdge] {
        &self.candidate.composition_edges
    }

    pub fn method(&self, selector: Selector) -> Option<MethodId> {
        self.candidate.methods.get(&selector).copied()
    }

    pub fn properties(&self) -> &[StoredProperty] {
        &self.candidate.properties
    }
}

impl ClassRegistry {
    pub fn candidate_has_class_var(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<bool, ClassError> {
        match self.staged.get(&class) {
            Some(candidate) => Ok(candidate.class_vars.contains(&name)),
            None => Ok(self.active(class)?.class_vars().contains(&name)),
        }
    }

    /// Reserves a Class identity and stages its origin in the current group.
    ///
    /// The spine carries the declaration's immutable meta policy. Superclasses
    /// must already be published. Modules and decorators are validated again at
    /// commit. Reserving consumes only a Class identity, never a revision or a
    /// commit identity; aborted Class identities are not reused. Modules may be
    /// staged origins; such groups must use `commit_structural_group`.
    ///
    /// `class`, `active`, `active_revision`, `open`, ordinary dispatch and Runtime
    /// allocation reject this identity until the group commits successfully.
    /// Backends can read `staged_origin`, `staged_method`, `visible_properties`
    /// and `is_staging`; they must keep source names private until commit too.
    ///
    /// Install members with `publish_origin_method`, `publish_singleton_method`,
    /// `declare_class_var`, `stage_origin_property` and `stage_decorators`.
    /// These mutate the existing staged candidate without publishing it.
    /// Use `commit_declaration_group`, not the legacy origin-sealing API.
    /// `roll_back_group` and `roll_back_transaction` discard these candidates.
    pub fn stage_class_origin(
        &mut self,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
        modules: &[CompositionEdge],
    ) -> Result<ClassId, ClassError> {
        let class = ClassId::new(self.next_class_id);
        let next_class_id = self
            .next_class_id
            .checked_add(1)
            .filter(|next| *next < crate::core_registration::CORE_ID_DOMAIN)
            .ok_or(ClassError::ClassIdentityExhausted)?;
        if let Some(superclass) = runtime_superclass {
            self.require_meta_capability(superclass, Capability::Subclass)?;
        }
        let capabilities = self.effective_meta_capabilities(static_spine, runtime_superclass)?;
        if !modules.is_empty() && !capabilities.allows(Capability::Modules) {
            return Err(ClassError::MetaCapabilityDenied {
                target: class,
                operation: Capability::Modules,
                policy_origin: PolicyOrigin::Class(class),
                reason: "the origin policy denies Module composition",
            });
        }
        let mut candidate = CandidateRevision::origin(
            class,
            RevisionId::new(0),
            static_spine,
            runtime_superclass,
            self.origin_mro(class, runtime_superclass)?,
            capabilities,
        );
        for edge in modules {
            candidate.add_composition_edge(*edge);
        }
        let (mro, _) = self.overlay_structure(&candidate, &self.candidate_module_overlay())?;
        candidate.mro = mro;
        self.staged.insert(class, candidate);
        self.next_class_id = next_class_id;
        Ok(class)
    }

    /// Reads an origin reserved by `stage_class_origin`, never a published Class.
    ///
    /// Its MRO reflects declaration-time composition; final publication recomputes
    /// it. Method identities remain backend artifacts in `method_by_id`, as for
    /// existing open transactions; they do not make their owner a published Class.
    pub fn staged_origin(&self, class: ClassId) -> Result<StagedOrigin<'_>, ClassError> {
        if self.classes.contains_key(&class) {
            return Err(ClassError::UnknownClassId(class));
        }
        self.staged
            .get(&class)
            .map(|candidate| StagedOrigin { candidate })
            .ok_or(ClassError::UnknownClassId(class))
    }

    /// Installs a static origin property without requiring reflective authority.
    pub fn stage_origin_property(
        &mut self,
        class: ClassId,
        selector: Selector,
        initializer: MethodBody,
    ) -> Result<(), ClassError> {
        self.staged_origin(class)?;
        self.mutate_candidate(class, |candidate| {
            candidate.add_stored_property(StoredProperty::new(selector, initializer));
        })
    }

    /// Consumes the entire staged group, publishing origins and opens atomically.
    ///
    /// All validation precedes revision/commit identity allocation. On failure
    /// every candidate is discarded, existing active revisions are unchanged,
    /// and reserved origins remain unknown to ordinary lookup. Success numbers
    /// origins 1, advances opens once, and assigns one shared commit identity.
    /// Empty groups consume no identities. Backend-owned state and Method body
    /// artifacts are not rolled back; this transaction owns Class metadata only.
    pub fn commit_declaration_group(&mut self) -> Result<Vec<ClassRevision>, ClassError> {
        let staged = std::mem::take(&mut self.staged);
        if staged.is_empty() {
            return Ok(Vec::new());
        }
        let mut origins = HashSet::new();
        for (class, candidate) in &staged {
            match self.classes.get(class) {
                Some(active) => {
                    if active.active_revision() != candidate.base_revision() {
                        return Err(ClassError::MetaTransactionConflict { class: *class });
                    }
                }
                None => {
                    origins.insert(*class);
                }
            }
        }
        let mut candidates: Vec<_> = staged.into_iter().collect();
        candidates.sort_by_key(|(class, _)| *class);
        self.publish_candidates(
            candidates
                .into_iter()
                .map(|(_, candidate)| candidate)
                .collect(),
            &origins,
        )
    }

    pub(crate) fn prepare_group_origins(
        &self,
        candidates: &mut [CandidateRevision],
        origins: &HashSet<ClassId>,
    ) -> Result<(), ClassError> {
        for index in 0..candidates.len() {
            if !origins.contains(&candidates[index].owner) {
                continue;
            }
            if let Some(superclass) = candidates[index].runtime_superclass {
                let (mro, policy) =
                    match candidates.iter().find(|parent| parent.owner == superclass) {
                        Some(parent) => (parent.mro.clone(), parent.meta_capabilities),
                        None => {
                            let parent = self.active(superclass)?;
                            (parent.mro().to_vec(), parent.meta_capabilities())
                        }
                    };
                if !policy.allows(Capability::Subclass) {
                    return Err(ClassError::MetaCapabilityDenied {
                        target: superclass,
                        operation: Capability::Subclass,
                        policy_origin: PolicyOrigin::Class(superclass),
                        reason: "the final superclass policy denies subclassing",
                    });
                }
                let candidate = &mut candidates[index];
                let local_len = candidate
                    .mro
                    .iter()
                    .position(|entry| *entry == MroEntry::Class(superclass))
                    .unwrap_or(candidate.mro.len());
                candidate.mro.truncate(local_len);
                candidate.mro.extend(mro);
                candidate.meta_capabilities = candidate.meta_capabilities.narrowed_by(policy);
                if !candidate.modules.is_empty() && !policy.allows(Capability::Modules) {
                    return Err(ClassError::MetaCapabilityDenied {
                        target: candidate.owner,
                        operation: Capability::Modules,
                        policy_origin: PolicyOrigin::Class(superclass),
                        reason: "the final superclass policy denies Module composition",
                    });
                }
            }
        }
        Ok(())
    }
}
