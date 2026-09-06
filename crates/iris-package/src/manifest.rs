use crate::{GitRevision, GitUrl, PackageError, PackageId, RelativePath, error::invalid};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: u32,
    pub package_id: PackageId,
    pub api_major: u32,
    pub version: Version,
    pub iris_major: u32,
    pub sources: Vec<RelativePath>,
    pub entry_modules: Vec<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
    pub permissions: Permissions,
    pub native: Option<NativeManifest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub package_id: PackageId,
    pub api_major: u32,
    pub version: VersionReq,
    pub git: GitUrl,
    pub rev: GitRevision,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeManifest {
    pub abi_major: u32,
    pub minimum_minor: u32,
    pub required_features: u64,
    pub metadata: RelativePath,
    pub cargo_manifest: RelativePath,
    pub cargo_package: String,
    pub cargo_target: String,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self, PackageError> {
        let manifest: Self = toml::from_str(text)?;
        if manifest.manifest_version != 1 || manifest.iris_major != 1 {
            return Err(invalid(
                "manifest_version/iris_major",
                "only 1 is supported",
            ));
        }
        let unique_sources: BTreeSet<_> = manifest.sources.iter().collect();
        if unique_sources.len() != manifest.sources.len() {
            return Err(invalid("sources", "duplicate entry"));
        }
        let mut modules = BTreeSet::new();
        for module in &manifest.entry_modules {
            if !module.split("::").all(identifier) || !modules.insert(module) {
                return Err(invalid("entry_modules", module));
            }
        }
        let mut permissions = BTreeSet::new();
        for permission in manifest
            .permissions
            .required
            .iter()
            .chain(&manifest.permissions.optional)
        {
            if !permission.split('.').all(identifier) || !permissions.insert(permission) {
                return Err(invalid("permissions", permission));
            }
        }
        for alias in manifest.dependencies.keys() {
            if !identifier(alias) {
                return Err(invalid("dependency alias", alias));
            }
        }
        if let Some(native) = &manifest.native {
            if !identifier(&native.cargo_target)
                || native.cargo_package.is_empty()
                || !native
                    .cargo_package
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            {
                return Err(invalid(
                    "native cargo target/package",
                    &native.cargo_package,
                ));
            }
            if !manifest
                .permissions
                .required
                .iter()
                .any(|value| value == "native.load")
            {
                return Err(invalid(
                    "permissions.required",
                    "native packages must require native.load",
                ));
            }
        }
        Ok(manifest)
    }
}

fn identifier(value: &str) -> bool {
    value.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
