use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use crate::{CandidateRevision, ClassId, ClassRevision, LogicalClass, RevisionId, StaticSpine};

/// A recoverable failure from class revision management.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassError {
    /// The requested logical Class is absent from this runtime.
    UnknownClassId(ClassId),
    /// The requested revision is absent from this runtime.
    UnknownRevisionId(RevisionId),
    /// A candidate was based on a revision that is no longer active.
    StaleCandidate { class: ClassId },
    /// A candidate attempted to alter immutable nominal promises.
    StaticSpineDowngrade { class: ClassId },
    /// A transaction group includes multiple candidates for one Class.
    DuplicateClassInGroup { class: ClassId },
    /// A historical revision cannot supply a valid rollback artifact.
    RevisionArtifactUnavailable(RevisionId),
    /// The runtime cannot issue another opaque revision identity.
    RevisionIdentityExhausted,
    /// The runtime cannot issue another structural transaction identifier.
    CommitIdentityExhausted,
    /// A Class has exhausted its per-Class revision number sequence.
    RevisionNumberExhausted { class: ClassId },
    /// The runtime cannot issue another logical Class identity.
    ClassIdentityExhausted,
}

impl fmt::Display for ClassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownClassId(_) => formatter.write_str("unknown Iris Class identity"),
            Self::UnknownRevisionId(_) => {
                formatter.write_str("unknown Iris Class revision identity")
            }
            Self::StaleCandidate { .. } => {
                formatter.write_str("stale Iris Class revision candidate")
            }
            Self::StaticSpineDowngrade { .. } => {
                formatter.write_str("Iris Class static spine changed")
            }
            Self::DuplicateClassInGroup { .. } => {
                formatter.write_str("transaction group repeats an Iris Class")
            }
            Self::RevisionArtifactUnavailable(_) => {
                formatter.write_str("Iris revision artifact is unavailable for rollback")
            }
            Self::RevisionIdentityExhausted => {
                formatter.write_str("Iris revision identity space exhausted")
            }
            Self::CommitIdentityExhausted => {
                formatter.write_str("Iris commit identity space exhausted")
            }
            Self::RevisionNumberExhausted { .. } => {
                formatter.write_str("Iris Class revision number space exhausted")
            }
            Self::ClassIdentityExhausted => {
                formatter.write_str("Iris Class identity space exhausted")
            }
        }
    }
}

impl Error for ClassError {}

/// Runtime-owned registry of logical Classes and every published revision.
#[derive(Debug, Default)]
pub struct ClassRegistry {
    classes: HashMap<ClassId, LogicalClass>,
    revisions: HashMap<RevisionId, ClassRevision>,
    next_class_id: u64,
    next_revision_id: u64,
    next_commit_id: u64,
}

impl ClassRegistry {
    /// Creates an empty class registry.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            revisions: HashMap::new(),
            next_class_id: 0,
            next_revision_id: 0,
            next_commit_id: 0,
        }
    }

    /// Defines a logical Class with its origin revision numbered one.
    pub fn define_class(
        &mut self,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
    ) -> Result<ClassId, ClassError> {
        let class = ClassId::new(self.next_class_id);
        let revision = RevisionId::new(self.next_revision_id);
        let next_class_id = self
            .next_class_id
            .checked_add(1)
            .ok_or(ClassError::ClassIdentityExhausted)?;
        let next_revision_id = self
            .next_revision_id
            .checked_add(1)
            .ok_or(ClassError::RevisionIdentityExhausted)?;
        let next_commit_id = self
            .next_commit_id
            .checked_add(1)
            .ok_or(ClassError::CommitIdentityExhausted)?;
        let candidate = CandidateRevision {
            base: revision,
            owner: class,
            number: 1,
            static_spine,
            runtime_superclass,
            mro: Vec::new(),
            modules: Vec::new(),
            methods: Default::default(),
            properties: Default::default(),
            class_vars: Default::default(),
        };
        self.revisions.insert(
            revision,
            ClassRevision::from_candidate(candidate, revision, next_commit_id),
        );
        self.classes
            .insert(class, LogicalClass::new(class, revision));
        self.next_class_id = next_class_id;
        self.next_revision_id = next_revision_id;
        self.next_commit_id = next_commit_id;
        Ok(class)
    }

    /// Returns stable logical Class state; revision metadata is read separately through `active`.
    pub fn class(&self, class: ClassId) -> Result<LogicalClass, ClassError> {
        self.classes
            .get(&class)
            .copied()
            .ok_or(ClassError::UnknownClassId(class))
    }

    /// Returns the current active revision identity for a logical Class.
    pub fn active_revision(&self, class: ClassId) -> Result<RevisionId, ClassError> {
        Ok(self.class(class)?.active_revision())
    }

    /// Returns read-only metadata for a specific retained revision.
    pub fn revision(&self, revision: RevisionId) -> Result<&ClassRevision, ClassError> {
        self.revisions
            .get(&revision)
            .ok_or(ClassError::UnknownRevisionId(revision))
    }

    /// Returns class metadata by resolving only its current active revision.
    pub fn active(&self, class: ClassId) -> Result<&ClassRevision, ClassError> {
        self.revision(self.active_revision(class)?)
    }

    /// Opens a candidate derived from the Class's sole current active revision.
    pub fn open(&self, class: ClassId) -> Result<CandidateRevision, ClassError> {
        let active = self.active(class)?;
        CandidateRevision::from_revision(active)
            .ok_or(ClassError::RevisionNumberExhausted { class })
    }

    /// Validates and atomically publishes one candidate as a one-Class transaction group.
    pub fn publish(&mut self, candidate: CandidateRevision) -> Result<ClassRevision, ClassError> {
        let mut revisions = self.publish_group([candidate])?;
        revisions.pop().ok_or(ClassError::RevisionIdentityExhausted)
    }

    /// Validates every candidate before persisting revisions and swapping any active pointer.
    pub fn publish_group<const N: usize>(
        &mut self,
        candidates: [CandidateRevision; N],
    ) -> Result<Vec<ClassRevision>, ClassError> {
        self.validate_group(&candidates)?;
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
                ClassRevision::from_candidate(candidate, id, commit_id)
            })
            .collect::<Vec<_>>();
        for revision in &published {
            self.revisions.insert(revision.id(), revision.clone());
        }
        for revision in &published {
            if let Some(class) = self.classes.get_mut(&revision.owner()) {
                class.publish(revision.id());
            }
        }
        self.next_revision_id = final_revision_id;
        self.next_commit_id = commit_id;
        Ok(published)
    }

    /// Rebuilds a candidate from a retained artifact and publishes it as new history.
    pub fn rollback(
        &mut self,
        class: ClassId,
        target: RevisionId,
    ) -> Result<ClassRevision, ClassError> {
        let artifact = self
            .revision(target)
            .map_err(|_| ClassError::RevisionArtifactUnavailable(target))?;
        if artifact.owner() != class {
            return Err(ClassError::RevisionArtifactUnavailable(target));
        }
        let mut candidate = self.open(class)?;
        candidate.runtime_superclass = artifact.runtime_superclass();
        candidate.mro = artifact.mro().to_vec();
        candidate.modules = artifact.modules().to_vec();
        candidate.methods = artifact.methods().clone();
        candidate.properties = artifact.properties().clone();
        candidate.class_vars = artifact.class_vars().clone();
        self.publish(candidate)
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
            if active.id() != candidate.base {
                return Err(ClassError::StaleCandidate {
                    class: candidate.owner,
                });
            }
            if active.static_spine() != candidate.static_spine {
                return Err(ClassError::StaticSpineDowngrade {
                    class: candidate.owner,
                });
            }
            if candidate.number
                != active
                    .number()
                    .checked_add(1)
                    .ok_or(ClassError::RevisionNumberExhausted {
                        class: candidate.owner,
                    })?
            {
                return Err(ClassError::StaleCandidate {
                    class: candidate.owner,
                });
            }
        }
        Ok(())
    }
}
