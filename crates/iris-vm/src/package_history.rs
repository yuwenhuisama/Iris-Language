/// Host-resolved source for one canonical Class or Module revision. The locator is opaque.
#[derive(Clone, Debug)]
pub struct RevisionArtifact {
    pub package_id: String,
    pub api_major: u64,
    pub logical_owner: String,
    pub revision: u64,
    /// Locator, BLAKE3 digest (optionally prefixed `b3:`), and exact source.
    pub artifact: (String, String, String),
}

/// Explicit revision keys supplied by the host, not a package discovery API.
///
/// Integer targets must uniquely match package, API major, owner, and revision.
/// Symbols and no-argument rollback are unavailable; no alias/default is inferred.
/// Reconstruction supports canonical non-generic, parentless instance-method Classes and
/// method-only Modules, and their method-only decorator Classes. State, composition, native/property and
/// qualified members, and member-set changes are rejected. Historical top-level
/// statements and unrelated declarations are never executed.
#[derive(Clone, Debug, Default)]
pub struct PackageHistory {
    pub artifacts: Vec<RevisionArtifact>,
}

/// Exact host-resolved source for one same-major package upgrade.
#[derive(Clone, Debug)]
pub struct ResolvedPackageVersion {
    pub package_id: String,
    pub api_major: u64,
    pub version: String,
    /// Opaque locator, BLAKE3 digest (optionally prefixed `b3:`), and exact source.
    pub artifact: (String, String, String),
}

/// Explicit host candidates for `Reflection::Package.upgrade`.
#[derive(Clone, Debug, Default)]
pub struct PackageUpgrade {
    pub versions: Vec<ResolvedPackageVersion>,
}

impl crate::Machine {
    /// Replaces the host history input. Digests are verified at selection time.
    pub fn enter_package_history(&mut self, history: PackageHistory) {
        self.replace_package_history(history);
    }

    /// Replaces the host's exact same-major package-upgrade candidates.
    pub fn enter_package_upgrades(&mut self, upgrades: PackageUpgrade) {
        self.replace_package_upgrades(upgrades);
    }
}
