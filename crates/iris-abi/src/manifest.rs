//! `IRIS-V1-FFI-C022` native metadata and `C023` load verification.

/// Why a native load was refused.
///
/// `IRIS-V1-FFI-C023` verifies the loaded artifact against the metadata package
/// resolution selected, and a mismatch MUST abort before binding, so each
/// variant names the check that failed rather than a generic failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestRejection {
    /// `C022` requires package identity in the metadata.
    PackageIdentityMissing,
    /// `C022` requires a ReflectionPolicy declaration.
    ReflectionPolicyMissing,
    /// `C023` requires the artifact digest to match the metadata.
    ArtifactDigestMismatch,
    /// `C039` rejects an ABI major mismatch outright.
    AbiMajorMismatch,
}

impl ManifestRejection {
    /// The stable diagnostic code for this rejection.
    #[must_use]
    pub const fn diagnostic(self) -> &'static str {
        match self {
            Self::PackageIdentityMissing => "ffi.native-package-identity-missing",
            Self::ReflectionPolicyMissing => "ffi.native-reflection-policy-missing",
            Self::ArtifactDigestMismatch => "ffi.native-artifact-digest-mismatch",
            Self::AbiMajorMismatch => "ffi.native-abi-major-mismatch",
        }
    }
}

/// Compile-time metadata a native package ships.
///
/// `IRIS-V1-FFI-C022` makes the compiler import this as static API ONLY after
/// validating it is complete, so an absent field is a rejection rather than a
/// default.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeManifest {
    /// Declared package identity, absent when the metadata omits it.
    pub package: Option<String>,
    /// Declared ReflectionPolicy, absent when the metadata omits it.
    pub reflection_policy: Option<String>,
    /// Declared artifact digest.
    pub digest: Option<String>,
    /// Required ABI major.
    pub abi_major: u32,
    /// Required ABI minor.
    pub abi_minor: u32,
}

/// Verifies a manifest against the artifact actually loaded.
///
/// `IRIS-V1-FFI-C023` requires this BEFORE binding, so a rejection means no
/// Method is published and no native code runs.
pub fn verify(
    manifest: &NativeManifest,
    artifact_digest: &str,
    runtime_major: u32,
) -> Result<(), ManifestRejection> {
    // C022 lists package identity and ReflectionPolicy among the metadata a
    // native package MUST ship, so their absence fails before any digest work.
    if manifest.package.is_none() {
        return Err(ManifestRejection::PackageIdentityMissing);
    }
    if manifest.reflection_policy.is_none() {
        return Err(ManifestRejection::ReflectionPolicyMissing);
    }
    if manifest.abi_major != runtime_major {
        return Err(ManifestRejection::AbiMajorMismatch);
    }
    // C023 verifies the artifact MATCHES the selected metadata, so a declared
    // digest that does not describe the loaded bytes aborts the load.
    if manifest.digest.as_deref() != Some(artifact_digest) {
        return Err(ManifestRejection::ArtifactDigestMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ManifestRejection, NativeManifest, verify};

    fn complete() -> NativeManifest {
        NativeManifest {
            package: Some("fixture.static_api".into()),
            reflection_policy: Some("default".into()),
            digest: Some("sha256:00".into()),
            abi_major: 1,
            abi_minor: 0,
        }
    }

    #[test]
    fn c023_a_matching_manifest_verifies() {
        assert_eq!(verify(&complete(), "sha256:00", 1), Ok(()));
    }

    #[test]
    fn c022_missing_package_identity_is_rejected() {
        // Given metadata that omits package identity
        let manifest = NativeManifest {
            package: None,
            ..complete()
        };

        // When / Then
        assert_eq!(
            verify(&manifest, "sha256:00", 1),
            Err(ManifestRejection::PackageIdentityMissing)
        );
        assert_eq!(
            ManifestRejection::PackageIdentityMissing.diagnostic(),
            "ffi.native-package-identity-missing"
        );
    }

    #[test]
    fn c022_missing_reflection_policy_is_rejected() {
        let manifest = NativeManifest {
            reflection_policy: None,
            ..complete()
        };
        assert_eq!(
            verify(&manifest, "sha256:00", 1),
            Err(ManifestRejection::ReflectionPolicyMissing)
        );
    }

    #[test]
    fn c023_a_digest_mismatch_aborts_the_load() {
        // The artifact does not hash to what the metadata declared, so the
        // load stops before any Method is published.
        assert_eq!(
            verify(&complete(), "sha256:11", 1),
            Err(ManifestRejection::ArtifactDigestMismatch)
        );
        assert_eq!(
            ManifestRejection::ArtifactDigestMismatch.diagnostic(),
            "ffi.native-artifact-digest-mismatch"
        );
    }

    #[test]
    fn c039_an_abi_major_mismatch_is_rejected() {
        assert_eq!(
            verify(&complete(), "sha256:00", 2),
            Err(ManifestRejection::AbiMajorMismatch)
        );
    }

    #[test]
    fn c022_identity_is_checked_before_the_digest() {
        // Both are wrong; the identity failure is what the caller sees, so a
        // manifestless artifact is never reported as a digest problem.
        let manifest = NativeManifest {
            package: None,
            digest: Some("sha256:zz".into()),
            ..complete()
        };
        assert_eq!(
            verify(&manifest, "sha256:00", 1),
            Err(ManifestRejection::PackageIdentityMissing)
        );
    }
}
