#![cfg(test)]

mod common;
use common::{commit, manifest, project, repository};
use iris_package::{BuildOptions, Grants, InstallOptions, PackageError, build, install, prepare};
use std::{collections::BTreeSet, fs};

fn native_repo() -> (tempfile::TempDir, String) {
    let text = manifest("org.iris.network").replace(
        "required = []",
        "required = [\"native.load\", \"network.tcp\"]",
    ) + r#"
[native]
abi_major = 1
minimum_minor = 0
required_features = 0
metadata = "native-api.json"
cargo_manifest = "Cargo.toml"
cargo_package = "iris-network"
cargo_target = "iris_network"
"#;
    let (repo, _) = repository(&text);
    fs::write(
        repo.path().join("Cargo.toml"),
        r#"[package]
name = "iris-network"
version = "1.2.0"
edition = "2024"
[lib]
crate-type = ["cdylib"]
path = "lib.rs"
"#,
    )
    .unwrap();
    fs::write(
        repo.path().join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"iris-network\"\nversion = \"1.2.0\"\n",
    )
    .unwrap();
    fs::write(
        repo.path().join("lib.rs"),
        "pub fn answer() -> u32 { 42 }\n",
    )
    .unwrap();
    fs::write(
        repo.path().join("build.rs"),
        "fn main() { std::fs::write(\"build-executed\", \"yes\").unwrap(); }\n",
    )
    .unwrap();
    fs::write(repo.path().join("native-api.json"), r#"{"schema_version":1,"package_id":"org.iris.network","api_major":1,"version":"1.2.0","iris_major":1,"abi":{"major":1,"minimum_minor":0,"required_features":0},"permissions":{"required":["native.load","network.tcp"],"optional":[]},"reflection_policy":"deny","modules":[]}"#).unwrap();
    let rev = commit(repo.path());
    (repo, rev)
}

fn grants() -> Grants {
    Grants {
        permissions: BTreeSet::from(["native.load".to_owned(), "network.tcp".to_owned()]),
    }
}

#[test]
fn refuses_build_when_explicit_consent_is_missing() {
    let (repo, rev) = native_repo();
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let when = build(
        root.path(),
        BuildOptions {
            allow_native_build: false,
        },
    );
    assert!(matches!(when, Err(PackageError::NativeBuildDenied)));
    assert!(!root.path().join(".iris/builds.toml").exists());
    assert!(!root.path().join(".iris/work").exists());
    assert!(!repo.path().join("build-executed").exists());
}

#[test]
fn refuses_prepare_when_native_build_is_missing() {
    let (repo, rev) = native_repo();
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let when = prepare(root.path(), &grants());
    assert!(matches!(when, Err(PackageError::MissingBuild(_))));
}

#[test]
fn refuses_prepare_when_grants_are_revoked() {
    let (repo, rev) = native_repo();
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::PermissionDenied { .. })));
}

#[test]
fn builds_and_prepares_native_artifact_when_explicitly_granted() {
    let (repo, rev) = native_repo();
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let receipt = build(
        root.path(),
        BuildOptions {
            allow_native_build: true,
        },
    )
    .unwrap();
    assert_eq!(receipt.artifacts.len(), 1);
    repo.close().unwrap();
    let when = prepare(root.path(), &grants()).unwrap();
    let native = when.packages[0].native.as_ref().unwrap();
    assert!(native.artifact.is_file());
    assert_eq!(native.metadata_sha256, receipt.artifacts[0].metadata_sha256);
    fs::write(&native.artifact, b"tampered").unwrap();
    assert!(matches!(
        prepare(root.path(), &grants()),
        Err(PackageError::Integrity { .. })
    ));
}

#[test]
fn builds_native_artifact_when_consumer_is_nested_in_cargo_workspace() {
    let (repo, rev) = native_repo();
    let workspace = tempfile::tempdir().unwrap();
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\nresolver = \"3\"\n",
    )
    .unwrap();
    let project = project(repo.path(), &rev);
    let workspace_root = fs::canonicalize(workspace.path()).unwrap();
    let root = workspace_root.join("demo/consumer");
    fs::create_dir_all(&root).unwrap();
    for name in ["iris.toml", "main.iris"] {
        fs::copy(project.path().join(name), root.join(name)).unwrap();
    }
    install(
        &root,
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();

    let when = build(
        &root,
        BuildOptions {
            allow_native_build: true,
        },
    )
    .unwrap();

    assert_eq!(when.artifacts.len(), 1);
    assert!(root.join(".iris/builds.toml").is_file());
    let prepared = prepare(&root, &grants()).unwrap();
    let native = prepared.packages[0].native.as_ref().unwrap();
    assert!(native.artifact.is_file());
    assert!(native.artifact.starts_with(root.join(".iris/artifacts")));
}

#[test]
fn builds_native_artifact_when_system_temp_is_inside_cargo_workspace() {
    let workspace = tempfile::tempdir().unwrap();
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\nresolver = \"3\"\n",
    )
    .unwrap();

    let when = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "builds_native_artifact_when_consumer_is_nested_in_cargo_workspace",
            "--nocapture",
        ])
        .env("TMPDIR", workspace.path())
        .env("TMP", workspace.path())
        .env("TEMP", workspace.path())
        .current_dir(workspace.path())
        .output()
        .unwrap();

    assert!(when.status.success(), "{when:?}");
}

#[test]
fn rejects_metadata_when_host_schema_fields_are_missing() {
    let (repo, _) = native_repo();
    let path = repo.path().join("native-api.json");
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace(",\"reflection_policy\":\"deny\",\"modules\":[]", "");
    fs::write(path, text).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    let when = install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    );
    assert!(when.is_err());
}

#[test]
fn rejects_metadata_when_identity_disagrees() {
    let (repo, _) = native_repo();
    let path = repo.path().join("native-api.json");
    let text = fs::read_to_string(&path)
        .unwrap()
        .replace("org.iris.network", "org.iris.other");
    fs::write(path, text).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    let when = install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    );
    assert!(when.is_err());
}

#[test]
fn rejects_build_when_cargo_lock_is_missing() {
    let (repo, _) = native_repo();
    fs::remove_file(repo.path().join("Cargo.lock")).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let when = build(
        root.path(),
        BuildOptions {
            allow_native_build: true,
        },
    );
    assert!(matches!(
        when,
        Err(PackageError::Invalid {
            field: "native build",
            ..
        })
    ));
    assert!(!root.path().join(".iris/builds.toml").exists());
}
