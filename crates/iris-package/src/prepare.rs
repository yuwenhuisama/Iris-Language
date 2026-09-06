use crate::{
    BuildReceipts, Manifest, NativeManifest, PackageError, RelativePath, Sha256Digest, files, lock,
    platform, resolver,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default)]
pub struct Grants {
    pub permissions: BTreeSet<String>,
}

#[derive(Debug)]
pub struct PreparedPackageTree {
    pub packages: Vec<PreparedPackage>,
}

#[derive(Debug)]
pub struct PreparedPackage {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub sources: Vec<PreparedSource>,
    pub native: Option<NativeLoadDeclaration>,
}

#[derive(Debug)]
pub struct PreparedSource {
    pub path: RelativePath,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct NativeLoadDeclaration {
    pub artifact: PathBuf,
    pub artifact_sha256: Sha256Digest,
    pub metadata: Vec<u8>,
    pub metadata_sha256: Sha256Digest,
    pub requirements: NativeManifest,
}

pub fn prepare(root: &Path, grants: &Grants) -> Result<PreparedPackageTree, PackageError> {
    let root = fs::canonicalize(root)?;
    let locked = lock::read_lock(&root)?;
    resolver::verify_lock(&root, &locked)?;
    for package in &locked.packages {
        for permission in &package.manifest.permissions.required {
            if !grants.permissions.contains(permission) {
                return Err(PackageError::PermissionDenied {
                    package: package.manifest.package_id.to_string(),
                    permission: permission.clone(),
                });
            }
        }
    }
    let receipts = if locked
        .packages
        .iter()
        .any(|package| package.manifest.native.is_some())
    {
        let path = files::safe_path(&root, Path::new(".iris/builds.toml"))?;
        if !path.exists() {
            return Err(PackageError::MissingBuild(
                "run an explicitly authorized native build".to_owned(),
            ));
        }
        let bytes = files::read_regular(&root, ".iris/builds.toml")?;
        let receipt: BuildReceipts = toml::from_str(files::text(&bytes)?)?;
        if receipt.receipt_version != 1 {
            return Err(crate::error::invalid(
                "receipt_version",
                "only 1 is supported",
            ));
        }
        receipt
    } else {
        BuildReceipts {
            receipt_version: 1,
            artifacts: Vec::new(),
        }
    };
    let mut packages = Vec::new();
    for package in &locked.packages {
        let content = lock::verify_package(&root, package)?;
        let native = match &package.manifest.native {
            Some(requirements) => {
                let matching: Vec<_> = receipts
                    .artifacts
                    .iter()
                    .filter(|receipt| {
                        receipt.package_id == package.manifest.package_id
                            && receipt.api_major == package.manifest.api_major
                            && receipt.platform == platform()
                    })
                    .collect();
                let receipt = match matching.as_slice() {
                    [receipt]
                        if receipt.source_sha256 == package.tree_sha256
                            && Some(&receipt.metadata_sha256)
                                == package.metadata_sha256.as_ref() =>
                    {
                        *receipt
                    }
                    _ => {
                        return Err(PackageError::MissingBuild(
                            package.manifest.package_id.to_string(),
                        ));
                    }
                };
                let artifact_relative = format!(".iris/artifacts/{}", receipt.artifact);
                let artifact = files::safe_path(&root, Path::new(&artifact_relative))?;
                if files::digest(&files::read_regular(&root, &artifact_relative)?)
                    != receipt.artifact_sha256
                {
                    return Err(PackageError::Integrity { path: artifact });
                }
                let metadata = content
                    .get(&requirements.metadata)
                    .ok_or_else(|| PackageError::Integrity {
                        path: artifact.clone(),
                    })?
                    .clone();
                Some(NativeLoadDeclaration {
                    artifact,
                    artifact_sha256: receipt.artifact_sha256.clone(),
                    metadata,
                    metadata_sha256: receipt.metadata_sha256.clone(),
                    requirements: requirements.clone(),
                })
            }
            None => None,
        };
        let mut sources = Vec::new();
        for path in &package.manifest.sources {
            let bytes = content
                .get(path)
                .ok_or_else(|| PackageError::Integrity {
                    path: root.join(path.as_str()),
                })?
                .clone();
            sources.push(PreparedSource {
                path: path.clone(),
                bytes,
            });
        }
        packages.push(PreparedPackage {
            root: lock::package_root(&root, package)?,
            manifest: package.manifest.clone(),
            sources,
            native,
        });
    }
    Ok(PreparedPackageTree { packages })
}
