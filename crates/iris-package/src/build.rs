use crate::{
    BuildReceipt, BuildReceipts, LockedPackage, PackageError, RelativePath, error::invalid, files,
    git, lock, platform, resolver,
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct BuildOptions {
    pub allow_native_build: bool,
}

pub fn build(root: &Path, options: BuildOptions) -> Result<BuildReceipts, PackageError> {
    if !options.allow_native_build {
        return Err(PackageError::NativeBuildDenied);
    }
    let root = fs::canonicalize(root)?;
    let locked = lock::read_lock(&root)?;
    resolver::verify_lock(&root, &locked)?;
    let work = files::safe_path(&root, Path::new(".iris/work"))?;
    fs::create_dir_all(&work)?;
    let mut receipts = BuildReceipts {
        receipt_version: 1,
        artifacts: Vec::new(),
    };
    for package in &locked.packages {
        if package.manifest.native.is_some() {
            receipts
                .artifacts
                .push(build_package(&root, &work, package)?);
        }
    }
    resolver::verify_lock(&root, &locked)?;
    let path = files::safe_path(&root, Path::new(".iris/builds.toml"))?;
    files::write_toml(&path, &receipts)?;
    Ok(receipts)
}

fn build_package(
    root: &Path,
    work: &Path,
    package: &LockedPackage,
) -> Result<BuildReceipt, PackageError> {
    let native = package
        .manifest
        .native
        .as_ref()
        .ok_or_else(|| invalid("native build", "missing declaration"))?;
    let content = lock::verify_package(root, package)?;
    if !content.keys().any(|path| path.as_str() == "Cargo.lock") {
        return Err(invalid(
            "native build",
            "repository must track root Cargo.lock",
        ));
    }
    let temporary = tempfile::tempdir_in(work)?;
    let source = temporary.path().join("source");
    fs::create_dir(&source)?;
    for (relative, bytes) in &content {
        let path = source.join(relative.as_str());
        let parent = path
            .parent()
            .ok_or_else(|| PackageError::UnsafePath(path.clone()))?;
        fs::create_dir_all(parent)?;
        fs::write(path, bytes)?;
    }
    let manifest_path = source.join(native.cargo_manifest.as_str());
    let output = git::run(
        Command::new("cargo")
            .current_dir(&source)
            .args([
                "build",
                "--locked",
                "--release",
                "--lib",
                "--package",
                &native.cargo_package,
                "--target",
                platform(),
                "--message-format=json-render-diagnostics",
                "--manifest-path",
            ])
            .arg(&manifest_path)
            .arg("--target-dir")
            .arg(temporary.path().join("target")),
        "cargo",
    )?;
    let mut candidates = Vec::new();
    for line in output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let message: CargoMessage = serde_json::from_slice(line)
            .map_err(|error| invalid("Cargo JSON", error.to_string()))?;
        match message {
            CargoMessage::Artifact {
                manifest_path: actual_manifest,
                target,
                filenames,
            } => {
                if actual_manifest == manifest_path
                    && target.name == native.cargo_target
                    && target.crate_types.iter().any(|kind| kind == "cdylib")
                {
                    candidates.extend(filenames.into_iter().filter(|path| {
                        path.extension()
                            .is_some_and(|extension| extension == std::env::consts::DLL_EXTENSION)
                    }));
                }
            }
            CargoMessage::Other => {}
        }
    }
    let artifact = match candidates.as_slice() {
        [artifact] => artifact,
        _ => {
            return Err(invalid(
                "native build",
                "Cargo must emit exactly one selected cdylib",
            ));
        }
    };
    let relative = artifact
        .strip_prefix(temporary.path())
        .map_err(|_| PackageError::UnsafePath(artifact.clone()))?;
    let relative = relative
        .to_str()
        .ok_or_else(|| PackageError::UnsafePath(artifact.clone()))?;
    let bytes = files::read_regular(temporary.path(), relative)?;
    let artifact_sha256 = files::digest(&bytes);
    let relative = RelativePath::try_from(format!(
        "{}/{}/{}.{}",
        platform(),
        package.tree_sha256,
        artifact_sha256,
        std::env::consts::DLL_EXTENSION
    ))?;
    let destination = files::safe_path(root, Path::new(&format!(".iris/artifacts/{relative}")))?;
    let parent = destination
        .parent()
        .ok_or_else(|| PackageError::UnsafePath(destination.clone()))?;
    fs::create_dir_all(parent)?;
    files::atomic_write(&destination, &bytes)?;
    let metadata_sha256 = package
        .metadata_sha256
        .clone()
        .ok_or_else(|| invalid("native build", "missing metadata checksum"))?;
    Ok(BuildReceipt {
        package_id: package.manifest.package_id.clone(),
        api_major: package.manifest.api_major,
        source_sha256: package.tree_sha256.clone(),
        metadata_sha256,
        platform: platform().to_owned(),
        artifact: relative,
        artifact_sha256,
    })
}

#[derive(Deserialize)]
#[serde(tag = "reason")]
enum CargoMessage {
    #[serde(rename = "compiler-artifact")]
    Artifact {
        manifest_path: PathBuf,
        target: CargoTarget,
        filenames: Vec<PathBuf>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct CargoTarget {
    name: String,
    crate_types: Vec<String>,
}
