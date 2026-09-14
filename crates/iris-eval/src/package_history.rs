use crate::PackageResolution;

/// A Host-owned revision key and its exact artifact; no Iris format is implied.
#[derive(Clone, Debug)]
pub struct RevisionArtifact {
    /// The target Class or Module's defining package.
    pub package_id: String,
    /// The target owner's API major identity.
    pub api_major: u64,
    /// The canonical logical Class or Module name, not a caller's alias.
    /// Classes and Modules share the package's qualified declaration namespace.
    pub logical_owner: String,
    /// The recorded historical revision number.
    pub revision: u64,
    /// Opaque locator, source digest, and exact historical source bytes.
    pub artifact: (String, String, String),
}

/// Adds explicit revision history without changing existing resolution literals.
///
/// Integer rollback targets select a unique matching record. Missing or ambiguous
/// keys are unavailable. Symbol targets are unavailable: this API supplies no
/// alias mappings. No-argument rollback uses only `resolution.artifact`.
/// Reconstruction retains the single-owner source slice limits documented there.
/// Nongeneric Class owners also rebuild named Contract-qualified instance Methods
/// separately from ordinary Methods, with fresh decorator chains and retained old
/// Method identities. Current Contract declarations and identities remain binding;
/// historical Contract declarations are not installed. Generic owners, generic
/// qualified Methods, and closed generic Contract-qualified history are unsupported.
/// Module rollback accepts one canonical origin Module with the same current
/// method set and compatible signatures/visibility, rebuilding ordered method
/// decorators with fresh captures under the existing Module/main identities.
/// State, composition, generic/qualified/native members, member-set changes and
/// Module-level decorator transformations are unsupported. Unrelated artifact
/// declarations and executable statements are not evaluated.
pub struct PackageHistory<'a> {
    /// The existing manifest, lock, permission, and default artifact resolution.
    pub resolution: PackageResolution<'a>,
    /// Host-resolved artifacts keyed by package, API major, logical owner, revision.
    pub artifacts: Vec<RevisionArtifact>,
}
