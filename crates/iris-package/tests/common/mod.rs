#![cfg(test)]

use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

pub fn manifest(identity: &str) -> String {
    format!(
        r#"manifest_version = 1
package_id = "{identity}"
api_major = 1
version = "1.2.0"
iris_major = 1
sources = ["main.iris"]
entry_modules = ["Main"]
[permissions]
required = []
optional = []
"#
    )
}

pub fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .env("GIT_MASTER", "1")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(arguments)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

pub fn repository(text: &str) -> (TempDir, String) {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("iris.toml"), text).unwrap();
    fs::write(repo.path().join("main.iris"), "module Main {}\n").unwrap();
    git(repo.path(), &["init", "--quiet"]);
    let rev = commit(repo.path());
    (repo, rev)
}

pub fn commit(root: &Path) -> String {
    git(root, &["add", "."]);
    git(root, &["commit", "--quiet", "-m", "fixture"]);
    git(root, &["rev-parse", "HEAD"])
}

pub fn dependency(repo: &Path, rev: &str) -> String {
    format!(
        r#"
[dependencies.network]
package_id = "org.iris.network"
api_major = 1
version = "^1.0"
git = "{}"
rev = "{rev}"
"#,
        url::Url::from_directory_path(repo).unwrap()
    )
}

pub fn project(repo: &Path, rev: &str) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("iris.toml"),
        manifest("org.iris.app") + &dependency(repo, rev),
    )
    .unwrap();
    fs::write(root.path().join("main.iris"), "module Main {}\n").unwrap();
    root
}
