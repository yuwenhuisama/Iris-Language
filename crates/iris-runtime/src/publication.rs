use std::collections::HashSet;

use crate::{CandidateRevision, ClassError, ClassRegistry, ClassRevision, RevisionId};

impl ClassRegistry {
    pub fn publish_group<const N: usize>(
        &mut self,
        mut candidates: [CandidateRevision; N],
    ) -> Result<Vec<ClassRevision>, ClassError> {
        for candidate in &mut candidates {
            crate::decorator::apply_pending(candidate)?;
        }
        self.validate_group(&candidates)?;
        let candidates = candidates
            .into_iter()
            .map(|mut candidate| {
                candidate.mro = self.compute_mro(&candidate)?;
                Ok(candidate)
            })
            .collect::<Result<Vec<_>, ClassError>>()?;
        let commit_id = self
            .next_commit_id
            .checked_add(1)
            .ok_or(ClassError::CommitIdentityExhausted)?;
        let revision_count = u64::try_from(N).map_err(|_| ClassError::RevisionIdentityExhausted)?;
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

    fn validate_group<const N: usize>(
        &self,
        candidates: &[CandidateRevision; N],
    ) -> Result<(), ClassError> {
        let mut owners = HashSet::with_capacity(N);
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
