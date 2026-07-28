use core::fmt;
use std::{collections::HashMap, error::Error};

use crate::module_registry::ModuleRegistry;
use crate::{
    BuiltinClass, CandidateRevision, Capability, ClassId, ClassRevision, DecoratorTransform,
    DecoratorViolation, LogicalClass, MetaCapabilities, Method, MethodId, RevisionId, StaticSpine,
};

/// A recoverable failure from Class revision management.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassError {
    UnknownClassId(ClassId),
    UnknownRevisionId(RevisionId),
    StaleCandidate {
        class: ClassId,
    },
    StaticSpineDowngrade {
        class: ClassId,
    },
    DuplicateClassInGroup {
        class: ClassId,
    },
    RevisionArtifactUnavailable(RevisionId),
    RevisionIdentityExhausted,
    CommitIdentityExhausted,
    RevisionNumberExhausted {
        class: ClassId,
    },
    ClassIdentityExhausted,
    MethodIdentityExhausted,
    UnknownModuleId(crate::ModuleId),
    ModuleCompositionCycle(crate::ModuleId),
    ProtectedSuperclass {
        class: ClassId,
    },
    MetaCapabilityDenied {
        class: ClassId,
        capability: Capability,
    },
    DecoratorViolation {
        class: ClassId,
        violation: DecoratorViolation,
    },
}

impl fmt::Display for ClassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnknownClassId(_) => "unknown Iris Class identity",
            Self::UnknownRevisionId(_) => "unknown Iris Class revision identity",
            Self::StaleCandidate { .. } => "stale Iris Class revision candidate",
            Self::StaticSpineDowngrade { .. } => "Iris Class static spine changed",
            Self::DuplicateClassInGroup { .. } => "transaction group repeats an Iris Class",
            Self::RevisionArtifactUnavailable(_) => {
                "Iris revision artifact is unavailable for rollback"
            }
            Self::RevisionIdentityExhausted => "Iris revision identity space exhausted",
            Self::CommitIdentityExhausted => "Iris commit identity space exhausted",
            Self::RevisionNumberExhausted { .. } => "Iris Class revision number space exhausted",
            Self::ClassIdentityExhausted => "Iris Class identity space exhausted",
            Self::MethodIdentityExhausted => "Iris Method identity space exhausted",
            Self::UnknownModuleId(_) => "unknown Iris Module identity",
            Self::ModuleCompositionCycle(_) => "cyclic Iris Module composition",
            Self::ProtectedSuperclass { .. } => "Iris built-in Class superclass is protected",
            Self::MetaCapabilityDenied { .. } => "Iris Class meta capability denied",
            Self::DecoratorViolation { .. } => {
                "Iris decorator changed forbidden declaration metadata"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ClassError {}

/// Runtime-owned registry of logical Classes and every published revision.
#[derive(Debug, Default)]
pub struct ClassRegistry {
    pub(crate) classes: HashMap<ClassId, LogicalClass>,
    pub(crate) revisions: HashMap<RevisionId, ClassRevision>,
    pub(crate) modules: ModuleRegistry,
    pub(crate) methods: HashMap<MethodId, Method>,
    pub(crate) next_class_id: u64,
    pub(crate) next_revision_id: u64,
    pub(crate) next_commit_id: u64,
    pub(crate) next_method_id: u64,
    pub(crate) next_bound_method_id: u64,
    pub(crate) builtins: HashMap<ClassId, BuiltinClass>,
}

impl ClassRegistry {
    /// Creates an empty Class registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a logical Class with origin revision number one.
    pub fn define_class(
        &mut self,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
    ) -> Result<ClassId, ClassError> {
        self.define_class_with_capabilities(
            static_spine,
            runtime_superclass,
            MetaCapabilities::all(),
        )
    }

    /// Defines a Class with an immutable origin meta-operation policy.
    pub fn define_class_with_capabilities(
        &mut self,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
        capabilities: MetaCapabilities,
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
        let candidate = CandidateRevision::origin(
            class,
            revision,
            static_spine,
            runtime_superclass,
            self.origin_mro(class, runtime_superclass)?,
        );
        self.revisions.insert(
            revision,
            ClassRevision::from_candidate(candidate, revision, next_commit_id, capabilities),
        );
        self.classes
            .insert(class, LogicalClass::new(class, revision));
        self.next_class_id = next_class_id;
        self.next_revision_id = next_revision_id;
        self.next_commit_id = next_commit_id;
        Ok(class)
    }

    /// Defines a protected built-in value Class.
    pub fn define_builtin_class(
        &mut self,
        kind: BuiltinClass,
        static_spine: StaticSpine,
        runtime_superclass: Option<ClassId>,
    ) -> Result<ClassId, ClassError> {
        let class = self.define_class(static_spine, runtime_superclass)?;
        self.builtins.insert(class, kind);
        Ok(class)
    }

    /// Returns stable logical Class state.
    pub fn class(&self, class: ClassId) -> Result<LogicalClass, ClassError> {
        self.classes
            .get(&class)
            .copied()
            .ok_or(ClassError::UnknownClassId(class))
    }

    /// Returns the active revision identity for a logical Class.
    pub fn active_revision(&self, class: ClassId) -> Result<RevisionId, ClassError> {
        Ok(self.class(class)?.active_revision())
    }

    /// Returns retained metadata for a specific revision.
    pub fn revision(&self, revision: RevisionId) -> Result<&ClassRevision, ClassError> {
        self.revisions
            .get(&revision)
            .ok_or(ClassError::UnknownRevisionId(revision))
    }

    /// Returns metadata for the Class's current active revision.
    pub fn active(&self, class: ClassId) -> Result<&ClassRevision, ClassError> {
        self.revision(self.active_revision(class)?)
    }

    /// Returns the current revision's immutable effective meta policy.
    pub fn active_meta_capabilities(&self, class: ClassId) -> Result<MetaCapabilities, ClassError> {
        Ok(self.active(class)?.meta_capabilities())
    }

    /// Opens a candidate from the Class's sole active revision.
    pub fn open(&self, class: ClassId) -> Result<CandidateRevision, ClassError> {
        CandidateRevision::from_revision(self.active(class)?)
            .ok_or(ClassError::RevisionNumberExhausted { class })
    }

    /// Publishes a single candidate transaction.
    pub fn publish(&mut self, candidate: CandidateRevision) -> Result<ClassRevision, ClassError> {
        let mut revisions = self.publish_group([candidate])?;
        revisions.pop().ok_or(ClassError::RevisionIdentityExhausted)
    }

    /// Applies declaration decorators to an unpublished candidate before atomic publication.
    pub fn publish_decorated(
        &mut self,
        mut candidate: CandidateRevision,
        decorators: impl IntoIterator<Item = DecoratorTransform>,
    ) -> Result<ClassRevision, ClassError> {
        candidate.stage_decorators(decorators);
        self.publish(candidate)
    }

    /// Rebuilds a candidate from a retained artifact and publishes new history.
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
        candidate.restore(artifact);
        self.publish(candidate)
    }
}
