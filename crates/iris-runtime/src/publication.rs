use std::collections::HashSet;

use crate::{
    CandidateRevision, ClassError, ClassId, ClassRegistry, ClassRevision, LogicalClass, RevisionId,
};

impl ClassRegistry {
    pub fn publish_group<const N: usize>(
        &mut self,
        candidates: [CandidateRevision; N],
    ) -> Result<Vec<ClassRevision>, ClassError> {
        self.publish_all(candidates.into())
    }

    /// Publishes every candidate in one transaction group atomically.
    ///
    /// `IRIS-V1-META-C038` validates and publishes all candidates in a group
    /// together, or rolls them all back, and `IRIS-V1-META-C041` forbids a
    /// thread from observing a partial structural mix. They therefore share one
    /// commit identity rather than being published one at a time.
    pub fn publish_all(
        &mut self,
        candidates: Vec<CandidateRevision>,
    ) -> Result<Vec<ClassRevision>, ClassError> {
        self.publish_candidates(candidates, &HashSet::new())
    }

    pub(crate) fn publish_candidates(
        &mut self,
        candidates: Vec<CandidateRevision>,
        origins: &HashSet<ClassId>,
    ) -> Result<Vec<ClassRevision>, ClassError> {
        let candidates = self.prepare_candidates(candidates, origins, None)?;
        let commit_id = self
            .next_commit_id
            .checked_add(1)
            .ok_or(ClassError::CommitIdentityExhausted)?;
        self.check_revision_capacity(candidates.len())?;
        Ok(self.install_candidates(candidates, commit_id))
    }

    pub(crate) fn prepare_candidates(
        &self,
        mut candidates: Vec<CandidateRevision>,
        origins: &HashSet<ClassId>,
        modules: Option<&crate::module_overlay::ModuleOverlay<'_>>,
    ) -> Result<Vec<CandidateRevision>, ClassError> {
        for candidate in &mut candidates {
            crate::decorator::apply_pending(candidate)?;
        }
        self.validate_group(&candidates, origins, modules)?;
        let mut candidates = candidates
            .into_iter()
            .map(|mut candidate| {
                if let Some(modules) = modules {
                    let (mro, policy) = self.overlay_structure(&candidate, modules)?;
                    candidate.mro = mro;
                    candidate.meta_capabilities = policy;
                    return Ok(candidate);
                }
                candidate.mro = self.compute_mro(&candidate)?;
                candidate.meta_capabilities = self
                    .effective_meta_capabilities(
                        candidate.static_spine,
                        candidate.runtime_superclass,
                    )?
                    .narrowed_by(candidate.meta_capabilities)
                    .narrowed_by(self.mro_meta_capabilities(&candidate.mro)?);
                Ok(candidate)
            })
            .collect::<Result<Vec<_>, ClassError>>()?;
        self.prepare_group_origins(&mut candidates, origins)?;
        for candidate in &candidates {
            for operation in &candidate.required {
                if !candidate.meta_capabilities.allows(*operation) {
                    return Err(ClassError::MetaCapabilityDenied {
                        target: candidate.owner,
                        operation: *operation,
                        policy_origin: crate::PolicyOrigin::Class(candidate.owner),
                        reason: "the final candidate policy denies a staged operation",
                    });
                }
            }
        }
        Ok(candidates)
    }

    pub(crate) fn check_revision_capacity(&self, count: usize) -> Result<(), ClassError> {
        let revision_count =
            u64::try_from(count).map_err(|_| ClassError::RevisionIdentityExhausted)?;
        self.next_revision_id
            .checked_add(revision_count)
            .filter(|next| *next < crate::core_registration::CORE_ID_DOMAIN)
            .ok_or(ClassError::RevisionIdentityExhausted)?;
        Ok(())
    }

    pub(crate) fn install_candidates(
        &mut self,
        candidates: Vec<CandidateRevision>,
        commit_id: u64,
    ) -> Vec<ClassRevision> {
        let mut next_revision_id = self.next_revision_id;
        let published = candidates
            .into_iter()
            .map(|candidate| {
                let id = RevisionId::new(next_revision_id);
                next_revision_id += 1;
                let capabilities = candidate.meta_capabilities;
                ClassRevision::from_candidate(candidate, id, commit_id, capabilities)
            })
            .collect::<Vec<_>>();
        for revision in &published {
            self.revisions.insert(revision.id(), revision.clone());
        }
        for revision in &published {
            self.classes
                .entry(revision.owner())
                .and_modify(|class| class.publish(revision.id()))
                .or_insert_with(|| LogicalClass::new(revision.owner(), revision.id()));
        }
        self.next_revision_id = next_revision_id;
        self.next_commit_id = commit_id;
        published
    }

    fn validate_group(
        &self,
        candidates: &[CandidateRevision],
        origins: &HashSet<ClassId>,
        modules: Option<&crate::module_overlay::ModuleOverlay<'_>>,
    ) -> Result<(), ClassError> {
        let mut owners = HashSet::with_capacity(candidates.len());
        for candidate in candidates {
            if !owners.insert(candidate.owner) {
                return Err(ClassError::DuplicateClassInGroup {
                    class: candidate.owner,
                });
            }
            if origins.contains(&candidate.owner) {
                if let Some(superclass) = candidate.runtime_superclass {
                    self.require_meta_capability(superclass, crate::Capability::Subclass)?;
                }
                for module in &candidate.modules {
                    if modules.is_none() && !self.modules.contains(*module) {
                        return Err(ClassError::UnknownModuleId(*module));
                    }
                }
                continue;
            }
            let active = self.active(candidate.owner)?;
            if active.id() != candidate.base
                || candidate.number
                    != active.number().checked_add(1).ok_or(
                        ClassError::RevisionNumberExhausted {
                            class: candidate.owner,
                        },
                    )?
            {
                return Err(ClassError::StaleCandidate {
                    class: candidate.owner,
                });
            }
            if active.static_spine() != candidate.static_spine {
                return Err(ClassError::StaticSpineDowngrade {
                    class: candidate.owner,
                });
            }
            if self.builtins.contains_key(&candidate.owner)
                && active.runtime_superclass() != candidate.runtime_superclass
            {
                return Err(ClassError::ProtectedSuperclass {
                    class: candidate.owner,
                });
            }
        }
        Ok(())
    }
}
