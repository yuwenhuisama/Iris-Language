use crate::{CandidateRevision, ClassError, ClassId};

/// A restricted declaration-candidate transform applied during publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecoratorTransform {
    Metadata {
        identity: String,
        arguments: Vec<String>,
    },
    ChangeDeclarationKind,
    ChangeNominalIdentity,
    ChangePackageIdentity,
}

impl DecoratorTransform {
    pub fn metadata(
        identity: impl Into<String>,
        arguments: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self::Metadata {
            identity: identity.into(),
            arguments: arguments.into_iter().map(Into::into).collect(),
        }
    }
}

/// A forbidden decorator mutation under IRIS-V1-META-C091.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecoratorViolation {
    DeclarationKind,
    NominalIdentity,
    PackageIdentity,
}

/// Immutable audit metadata for one applied declaration decorator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedDecorator {
    pub(crate) identity: String,
    pub(crate) arguments: Vec<String>,
}

impl AppliedDecorator {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}

pub(crate) fn apply_pending(candidate: &mut CandidateRevision) -> Result<(), ClassError> {
    for decorator in std::mem::take(&mut candidate.pending_decorators) {
        match decorator {
            DecoratorTransform::Metadata {
                identity,
                arguments,
            } => candidate.decorators.push(AppliedDecorator {
                identity,
                arguments,
            }),
            DecoratorTransform::ChangeDeclarationKind => {
                return Err(violation(
                    candidate.owner,
                    DecoratorViolation::DeclarationKind,
                ));
            }
            DecoratorTransform::ChangeNominalIdentity => {
                return Err(violation(
                    candidate.owner,
                    DecoratorViolation::NominalIdentity,
                ));
            }
            DecoratorTransform::ChangePackageIdentity => {
                return Err(violation(
                    candidate.owner,
                    DecoratorViolation::PackageIdentity,
                ));
            }
        }
    }
    Ok(())
}

fn violation(class: ClassId, violation: DecoratorViolation) -> ClassError {
    ClassError::DecoratorViolation { class, violation }
}
