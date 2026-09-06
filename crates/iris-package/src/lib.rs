//! Pinned Git package resolution, explicit native builds, and offline admission.
//!
//! `install(root, InstallOptions)` creates `iris.lock` or verifies an existing
//! lock and restores only missing pinned snapshots. It never runs package code.
//! `build(root, BuildOptions)` requires `allow_native_build = true` before any
//! filesystem access or process execution. Cargo build scripts are arbitrary
//! trusted code, not sandboxed code. `prepare(root, &Grants)` starts no processes
//! and returns dependency-first packages, manifest-ordered source bytes, and
//! digest-verified native load declarations. The native host must recheck the
//! artifact digest immediately before loading and enforce its own ABI policy.
//!
//! # Manifest version 1
//! Top-level fields are `manifest_version = 1`, reverse-domain `package_id`,
//! `api_major: u32`, SemVer `version`, `iris_major = 1`, relative `sources`, and
//! `entry_modules`. `[permissions]` requires `required` and `optional` arrays.
//! `[dependencies.alias]` requires `package_id`, `api_major`, SemVer range
//! `version`, credential-free `git` URL, and full lowercase 40/64-digit `rev`.
//! HTTPS and SSH URL syntax are supported; userinfo is forbidden, including SSH
//! usernames (configure the SSH user in local SSH configuration). `file:` URLs
//! require `InstallOptions::allow_local_git`; raw paths and SCP syntax are not
//! accepted. No branches, tags, registry resolution, or implicit updates exist.
//!
//! `[native]` requires `abi_major`, `minimum_minor`, `required_features: u64`,
//! `metadata`, `cargo_manifest`, `cargo_package`, and `cargo_target`. Current host
//! packages use feature mask zero. Metadata JSON uses the native-host version-1
//! schema, including reflection policy, modules, functions, and resources.
//! Native dependencies must require `native.load` and track root `Cargo.lock`.
//! The local application root is source-only; native packages are Git dependencies.
//!
//! # Persisted records
//! `iris.lock` is TOML `{ lock_version = 1, packages = [...] }`. Each package
//! stores its complete typed manifest, optional Git source `{ git, rev }`,
//! `tree_sha256`, optional `metadata_sha256`, and a sorted `files` map of relative
//! path to exact-byte SHA256. Dependencies are DFS postorder by sorted alias;
//! the source-only root is last. One implementation per package ID is admitted.
//! Root checksums cover its manifest and declared sources; dependency checksums
//! cover every tracked regular file. Git modes are checked, not hashed.
//! Tree hashing is SHA256 of `iris-package-tree-v1\0` followed by sorted UTF-8
//! path/content records: little-endian u64 path length, path bytes, little-endian
//! u64 content length, exact content bytes. Digests are lowercase hex, no prefix.
//!
//! `.iris/builds.toml` stores `{ receipt_version = 1, artifacts = [...] }`.
//! Each artifact records package ID/API, `source_sha256`, `metadata_sha256`,
//! compile-time target-triple `platform`, relative `artifact` path, and
//! `artifact_sha256`. A build replaces receipts for this project/platform;
//! snapshots live under `.iris/sources/<tree_sha256>` and artifacts under
//! `.iris/artifacts/<platform>/<tree_sha256>/<artifact_sha256>.<extension>`.
//! Writes are atomic; callers must serialize operations on a project. Lockfiles
//! and receipts are trusted local inputs, not signatures. Concurrent hostile
//! filesystem mutation and malicious authorized build scripts are not isolated.
mod build;
mod cache;
mod error;
mod files;
mod git;
mod lock;
mod manifest;
mod metadata;
mod prepare;
mod receipt;
mod resolver;
mod types;

pub use build::{BuildOptions, build};
pub use error::PackageError;
pub use lock::{GitSource, LockedPackage, SourceLock};
pub use manifest::{Dependency, Manifest, NativeManifest, Permissions};
pub use prepare::{
    Grants, NativeLoadDeclaration, PreparedPackage, PreparedPackageTree, PreparedSource, prepare,
};
pub use receipt::{BuildReceipt, BuildReceipts, platform};
pub use resolver::{InstallOptions, install};
pub use types::{GitRevision, GitUrl, PackageId, RelativePath, Sha256Digest};
