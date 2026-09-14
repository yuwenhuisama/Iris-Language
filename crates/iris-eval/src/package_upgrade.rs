use crate::PackageHistory;

/// Exact Host-resolved implementation keyed independently of its opaque locator.
#[derive(Clone, Debug)]
pub struct ResolvedPackageVersion {
    pub package_id: String,
    pub api_major: u64,
    pub version: String,
    /// Opaque locator, BLAKE3 digest, and exact UTF-8 package source.
    pub artifact: (String, String, String),
}

/// Additive Host metadata for explicit same-major package upgrades.
///
/// Every existing Class/Module must occur exactly once in the artifact. This
/// initial slice replaces compatible ordinary method bodies and method decorators.
/// Existing initialization statements and class-level storage declarations must remain
/// identical and are not executed again. New owners/storage, changed member sets,
/// composition, generics, native bodies, and other declaration kinds are rejected.
/// Hook slot assignments are private candidates; effects through referenced objects
/// (including external logs) are not compensated. No locator conventions or new
/// Iris package format are defined by this API.
/// Legacy loaders without version metadata retain failure-hook evaluation, but
/// cannot publish an upgrade: a successful hook reports artifact unavailable.
pub struct PackageUpgrade<'a> {
    pub history: PackageHistory<'a>,
    pub versions: Vec<ResolvedPackageVersion>,
}
