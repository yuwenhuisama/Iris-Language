use std::collections::HashSet;

use crate::{CandidateRevision, ClassError, ClassRegistry, ClassRevision, RevisionId};

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
        mut candidates: Vec<CandidateRevision>,
    ) -> Result<Vec<ClassRevision>, ClassError> {
        let count = candidates.len();
        for candidate in &mut candidates {
            crate::decorator::apply_pending(candidate)?;
        }
        self.validate_group(&candidates)?;
        let candidates = candidates
            .into_iter()
            .map(|mut candidate| {
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
        let commit_id = self
            .next_commit_id
            .checked_add(1)
            .ok_or(ClassError::CommitIdentityExhausted)?;
        let revision_count =
            u64::try_from(count).map_err(|_| ClassError::RevisionIdentityExhausted)?;
        let final_revision_id = self
            .next_revision_id
            .checked_add(revision_count)
            .ok_or(ClassError::RevisionIdentityExhausted)?;
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
                .get_mut(&revision.owner())
                .ok_or(ClassError::UnknownClassId(revision.owner()))?
                .publish(revision.id());
        }
        self.next_revision_id = final_revision_id;
        self.next_commit_id = commit_id;
        Ok(published)
    }

    fn validate_group(&self, candidates: &[CandidateRevision]) -> Result<(), ClassError> {
        let mut owners = HashSet::with_capacity(candidates.len());
        for candidate in candidates {
            let active = self.active(candidate.owner)?;
            if !owners.insert(candidate.owner) {
                return Err(ClassError::DuplicateClassInGroup {
                    class: candidate.owner,
                });
            }
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
