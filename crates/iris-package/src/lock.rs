use crate::{
    GitRevision, GitUrl, Manifest, PackageError, RelativePath, Sha256Digest, error::invalid, files,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLock {
    pub lock_version: u32,
    pub packages: Vec<LockedPackage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    pub manifest: Manifest,
    pub source: Option<GitSource>,
    pub tree_sha256: Sha256Digest,
    pub metadata_sha256: Option<Sha256Digest>,
    pub files: BTreeMap<RelativePath, Sha256Digest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitSource {
    pub git: GitUrl,
    pub rev: GitRevision,
}

pub(crate) fn root_files(root: &Path) -> Result<BTreeMap<RelativePath, Vec<u8>>, PackageError> {
    let bytes = files::read_regular(root, "iris.toml")?;
    let manifest = Manifest::parse(files::text(&bytes)?)?;
    if manifest.native.is_some() {
        return Err(invalid(
            "root native",
            "native packages must be pinned Git dependencies",
        ));
    }
    let mut result = BTreeMap::new();
    result.insert(RelativePath::try_from("iris.toml".to_owned())?, bytes);
    for source in &manifest.sources {
        result.insert(source.clone(), files::read_regular(root, source.as_str())?);
    }
    Ok(result)
}

pub(crate) fn snapshot(
    content: &BTreeMap<RelativePath, Vec<u8>>,
    source: Option<GitSource>,
) -> Result<LockedPackage, PackageError> {
    let manifest_path = RelativePath::try_from("iris.toml".to_owned())?;
    let bytes = content
        .get(&manifest_path)
        .ok_or_else(|| invalid("package", "missing tracked iris.toml"))?;
    let manifest = Manifest::parse(files::text(bytes)?)?;
    for path in &manifest.sources {
        if !content.contains_key(path) {
            return Err(invalid("sources", path.as_str()));
        }
    }
    let metadata_sha256 = match &manifest.native {
        Some(native) => {
            if !content.contains_key(&native.cargo_manifest) {
                return Err(invalid(
                    "native.cargo_manifest",
                    native.cargo_manifest.as_str(),
                ));
            }
            let metadata = content
                .get(&native.metadata)
                .ok_or_else(|| invalid("native.metadata", native.metadata.as_str()))?;
            crate::metadata::verify(metadata, &manifest)?;
            Some(files::digest(metadata))
        }
        None => None,
    };
    Ok(LockedPackage {
        manifest,
        source,
        metadata_sha256,
        tree_sha256: files::tree_digest(content),
        files: content
            .iter()
            .map(|(path, bytes)| (path.clone(), files::digest(bytes)))
            .collect(),
    })
}

pub(crate) fn package_root(root: &Path, package: &LockedPackage) -> Result<PathBuf, PackageError> {
    match &package.source {
        Some(_) => files::safe_path(
            root,
            Path::new(&format!(".iris/sources/{}", package.tree_sha256)),
        ),
        None => Ok(root.to_path_buf()),
    }
}

pub(crate) fn verify_package(
    root: &Path,
    package: &LockedPackage,
) -> Result<BTreeMap<RelativePath, Vec<u8>>, PackageError> {
    let directory = package_root(root, package)?;
    let content = match &package.source {
        Some(_) => files::inventory(&directory)?,
        None => root_files(root)?,
    };
    if snapshot(&content, package.source.clone())? != *package {
        return Err(PackageError::Integrity { path: directory });
    }
    Ok(content)
}

pub(crate) fn read_lock(root: &Path) -> Result<SourceLock, PackageError> {
    let bytes = files::read_regular(root, "iris.lock")?;
    let lock: SourceLock = toml::from_str(files::text(&bytes)?)?;
    if lock.lock_version != 1 {
        return Err(invalid("lock_version", "only 1 is supported"));
    }
    Ok(lock)
}
