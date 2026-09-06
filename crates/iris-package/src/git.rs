use crate::{Dependency, PackageError, RelativePath, error::invalid, files::text};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

pub(crate) fn run(command: &mut Command, program: &'static str) -> Result<Vec<u8>, PackageError> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(PackageError::Command {
            program,
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}

fn command(directory: &Path, local: bool) -> Command {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .current_dir(directory)
        .env("GIT_MASTER", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "-c",
            "protocol.ssh.allow=always",
            "-c",
            "fetch.fsckObjects=true",
            "-c",
            "transfer.fsckObjects=true",
            "-c",
            "http.followRedirects=false",
            "-c",
            "credential.helper=",
        ]);
    if local {
        command.args(["-c", "protocol.file.allow=always"]);
    }
    command
}

pub(crate) fn fetch(
    dependency: &Dependency,
    local: bool,
) -> Result<BTreeMap<RelativePath, Vec<u8>>, PackageError> {
    if dependency.git.is_local() && !local {
        return Err(PackageError::LocalGitDenied);
    }
    let temporary = tempfile::tempdir()?;
    let root = temporary.path();
    let format = if dependency.rev.as_str().len() == 64 {
        "--object-format=sha256"
    } else {
        "--object-format=sha1"
    };
    run(
        command(root, local).args(["init", "--bare", "--template=", format, "."]),
        "git",
    )?;
    run(
        command(root, local).args([
            "fetch",
            "--no-tags",
            "--depth=1",
            "--no-recurse-submodules",
            "--",
            dependency.git.as_str(),
            dependency.rev.as_str(),
        ]),
        "git",
    )?;
    let commit = run(
        command(root, local).args([
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", dependency.rev),
        ]),
        "git",
    )?;
    if text(&commit)?.trim() != dependency.rev.as_str() {
        return Err(invalid(
            "Git revision",
            "revision must identify a commit, not a tag",
        ));
    }
    let tree = run(
        command(root, local).args(["ls-tree", "-rz", "--full-tree", dependency.rev.as_str()]),
        "git",
    )?;
    let mut files = BTreeMap::new();
    let mut folded = BTreeSet::new();
    for record in tree
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let (header, path) = text(record)?
            .split_once('\t')
            .ok_or_else(|| invalid("Git tree", "missing tab"))?;
        let mut fields = header.split(' ');
        let mode = fields.next();
        let kind = fields.next();
        let object = fields
            .next()
            .ok_or_else(|| invalid("Git tree", "missing object"))?;
        if !matches!(mode, Some("100644" | "100755")) || kind != Some("blob") {
            return Err(PackageError::UnsafePath(path.into()));
        }
        let path = RelativePath::try_from(path.to_owned())?;
        if !folded.insert(path.as_str().to_lowercase()) {
            return Err(invalid("Git tree", "case-insensitive collision"));
        }
        let bytes = run(
            command(root, local).args(["cat-file", "blob", object]),
            "git",
        )?;
        if files.insert(path, bytes).is_some() {
            return Err(invalid("Git tree", "duplicate path"));
        }
    }
    Ok(files)
}
