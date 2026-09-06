mod common;
use common::{manifest, project, repository};
use iris_package::{Grants, InstallOptions, PackageError, install, prepare};
use std::fs;

#[test]
fn prepares_offline_when_pinned_revision_was_installed() {
    let (given_repo, rev) = repository(&manifest("org.iris.network"));
    let given_root = project(given_repo.path(), &rev);
    install(
        given_root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let initial = fs::read(given_root.path().join("iris.lock")).unwrap();
    install(
        given_root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    assert_eq!(
        initial,
        fs::read(given_root.path().join("iris.lock")).unwrap()
    );
    given_repo.close().unwrap();
    let when = prepare(given_root.path(), &Grants::default()).unwrap();
    assert_eq!(when.packages.len(), 2);
    assert_eq!(
        when.packages[0].manifest.package_id.as_str(),
        "org.iris.network"
    );
    assert_eq!(when.packages[0].sources[0].bytes, b"module Main {}\n");
}

#[test]
fn rejects_cache_mutation_when_preparing_offline() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    let prepared = prepare(root.path(), &Grants::default()).unwrap();
    fs::write(prepared.packages[0].root.join("main.iris"), "tampered").unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::Integrity { .. })));
}

#[test]
fn rejects_local_transport_when_not_explicitly_enabled() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    let when = install(root.path(), InstallOptions::default());
    assert!(matches!(when, Err(PackageError::LocalGitDenied)));
}

#[test]
fn rejects_lock_when_root_manifest_changes() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    fs::write(root.path().join("iris.toml"), manifest("org.iris.other")).unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(when.is_err());
}
