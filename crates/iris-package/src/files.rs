use crate::{PackageError, RelativePath, Sha256Digest, error::invalid};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub(crate) fn digest(bytes: &[u8]) -> Sha256Digest {
    Sha256Digest::from_hash(format!("{:x}", Sha256::digest(bytes)))
}

pub(crate) fn safe_path(root: &Path, relative: &Path) -> Result<PathBuf, PackageError> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(PackageError::UnsafePath(relative.to_path_buf()));
        }
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(PackageError::UnsafePath(current));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(current)
}

pub(crate) fn read_regular(root: &Path, relative: &str) -> Result<Vec<u8>, PackageError> {
    let path = safe_path(root, Path::new(relative))?;
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(PackageError::UnsafePath(path));
    }
    Ok(fs::read(path)?)
}

pub(crate) fn text(bytes: &[u8]) -> Result<&str, PackageError> {
    std::str::from_utf8(bytes).map_err(|_| invalid("UTF-8", "invalid text"))
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), PackageError> {
    let parent = path
        .parent()
        .ok_or_else(|| PackageError::UnsafePath(path.to_path_buf()))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

pub(crate) fn write_toml(path: &Path, value: &impl Serialize) -> Result<(), PackageError> {
    atomic_write(path, toml::to_string_pretty(value)?.as_bytes())
}

pub(crate) fn inventory(root: &Path) -> Result<BTreeMap<RelativePath, Vec<u8>>, PackageError> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .map_err(|_| PackageError::UnsafePath(path.clone()))?;
            let relative = relative
                .to_str()
                .ok_or_else(|| PackageError::UnsafePath(path.clone()))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            let name = RelativePath::try_from(relative)?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                files.insert(name, fs::read(path)?);
            } else {
                return Err(PackageError::UnsafePath(path));
            }
        }
    }
    let mut folded = std::collections::BTreeSet::new();
    for path in files.keys() {
        if !folded.insert(path.as_str().to_lowercase()) {
            return Err(invalid("file paths", "case-insensitive collision"));
        }
    }
    Ok(files)
}

pub(crate) fn tree_digest(files: &BTreeMap<RelativePath, Vec<u8>>) -> Sha256Digest {
    let mut hasher = Sha256::new();
    hasher.update(b"iris-package-tree-v1\0");
    for (path, bytes) in files {
        hasher.update(
            u64::try_from(path.as_str().len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        hasher.update(path.as_str().as_bytes());
        hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(bytes);
    }
    Sha256Digest::from_hash(format!("{:x}", hasher.finalize()))
}
