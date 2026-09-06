use crate::{
    Dependency, LockedPackage, PackageError, RelativePath, SourceLock, files, git, lock, resolver,
};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) fn publish(
    root: &Path,
    package: &LockedPackage,
    content: &BTreeMap<RelativePath, Vec<u8>>,
) -> Result<(), PackageError> {
    let destination = lock::package_root(root, package)?;
    if destination.exists() {
        lock::verify_package(root, package)?;
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| PackageError::UnsafePath(destination.clone()))?;
    let temporary = tempfile::tempdir_in(parent)?;
    for (relative, bytes) in content {
        let path = temporary.path().join(relative.as_str());
        let parent = path
            .parent()
            .ok_or_else(|| PackageError::UnsafePath(path.clone()))?;
        fs::create_dir_all(parent)?;
        fs::write(path, bytes)?;
    }
    fs::rename(temporary.path(), &destination)?;
    Ok(())
}

pub(crate) fn restore(root: &Path, locked: &SourceLock, local: bool) -> Result<(), PackageError> {
    resolver::verify_graph(locked)?;
    let cache = files::safe_path(root, Path::new(".iris/sources"))?;
    fs::create_dir_all(cache)?;
    for package in &locked.packages {
        let directory = lock::package_root(root, package)?;
        match &package.source {
            Some(source) if !directory.exists() => {
                let dependency = Dependency {
                    package_id: package.manifest.package_id.clone(),
                    api_major: package.manifest.api_major,
                    version: semver::VersionReq::STAR,
                    git: source.git.clone(),
                    rev: source.rev.clone(),
                };
                let content = git::fetch(&dependency, local)?;
                if lock::snapshot(&content, Some(source.clone()))? != *package {
                    return Err(PackageError::Integrity { path: directory });
                }
                publish(root, package, &content)?;
            }
            Some(_) | None => {
                lock::verify_package(root, package)?;
            }
        }
    }
    Ok(())
}
