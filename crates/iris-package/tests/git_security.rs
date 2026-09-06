mod common;
use common::{commit, dependency, git, manifest, project, repository};
use iris_package::{Grants, InstallOptions, PackageError, SourceLock, install, prepare};
use std::fs;

const LOCAL: InstallOptions = InstallOptions {
    allow_local_git: true,
};

#[test]
fn restores_missing_cache_when_existing_lock_is_trusted() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(root.path(), LOCAL).unwrap();
    let original = fs::read(root.path().join("iris.lock")).unwrap();
    fs::remove_dir_all(root.path().join(".iris")).unwrap();
    let when = install(root.path(), LOCAL);
    assert!(when.is_ok(), "{when:?}");
    assert_eq!(fs::read(root.path().join("iris.lock")).unwrap(), original);
}

#[test]
fn rejects_gitlink_when_repository_contains_submodule() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    git(
        repo.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{rev},submodule"),
        ],
    );
    git(repo.path(), &["commit", "--quiet", "-m", "gitlink"]);
    let rev = git(repo.path(), &["rev-parse", "HEAD"]);
    let root = project(repo.path(), &rev);
    let when = install(root.path(), LOCAL);
    assert!(matches!(when, Err(PackageError::UnsafePath(_))));
    assert!(!root.path().join("iris.lock").exists());
}

#[cfg(unix)]
#[test]
fn rejects_symlink_when_repository_tracks_one() {
    let (repo, _) = repository(&manifest("org.iris.network"));
    std::os::unix::fs::symlink("main.iris", repo.path().join("linked.iris")).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    let when = install(root.path(), LOCAL);
    assert!(matches!(when, Err(PackageError::UnsafePath(_))));
}

#[cfg(unix)]
#[test]
fn rejects_symlink_when_cache_directory_is_replaced() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(root.path(), LOCAL).unwrap();
    let prepared = prepare(root.path(), &Grants::default()).unwrap();
    let cache = &prepared.packages[0].root;
    let moved = root.path().join("moved");
    fs::rename(cache, &moved).unwrap();
    std::os::unix::fs::symlink(moved, cache).unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::UnsafePath(_))));
}

#[test]
fn rejects_untracked_addition_when_cache_was_modified() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(root.path(), LOCAL).unwrap();
    let prepared = prepare(root.path(), &Grants::default()).unwrap();
    fs::write(prepared.packages[0].root.join("extra.rs"), "new code").unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::Integrity { .. })));
}

#[test]
fn selects_exact_commit_when_remote_head_has_moved() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    fs::write(repo.path().join("main.iris"), "module Changed {}\n").unwrap();
    commit(repo.path());
    let root = project(repo.path(), &rev);
    install(root.path(), LOCAL).unwrap();
    let when = prepare(root.path(), &Grants::default()).unwrap();
    assert_eq!(when.packages[0].sources[0].bytes, b"module Main {}\n");
}

#[test]
fn rejects_identity_version_and_api_when_repository_disagrees() {
    for text in [
        manifest("org.iris.wrong"),
        manifest("org.iris.network").replace("1.2.0", "2.0.0"),
        manifest("org.iris.network").replace("api_major = 1", "api_major = 2"),
    ] {
        let (repo, rev) = repository(&text);
        let root = project(repo.path(), &rev);
        let when = install(root.path(), LOCAL);
        assert!(matches!(when, Err(PackageError::Conflict(_))));
    }
}

#[test]
fn rejects_duplicate_identity_when_two_revisions_are_requested() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    fs::write(repo.path().join("main.iris"), "module Changed {}\n").unwrap();
    let next = commit(repo.path());
    let root = project(repo.path(), &rev);
    let text = fs::read_to_string(root.path().join("iris.toml")).unwrap()
        + &dependency(repo.path(), &next).replace("dependencies.network", "dependencies.other");
    fs::write(root.path().join("iris.toml"), text).unwrap();
    let when = install(root.path(), LOCAL);
    assert!(matches!(when, Err(PackageError::Conflict(_))));
}

#[test]
fn rejects_cycle_when_dependency_points_back_to_active_root() {
    let (repo, first) = repository(&manifest("org.iris.network"));
    let text = manifest("org.iris.network")
        + &dependency(repo.path(), &first).replace(
            "package_id = \"org.iris.network\"",
            "package_id = \"org.iris.app\"",
        );
    fs::write(repo.path().join("iris.toml"), text).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    let when = install(root.path(), LOCAL);
    assert!(matches!(when, Err(PackageError::Cycle(_))));
}

#[test]
fn rejects_changed_selection_when_lock_revision_is_tampered() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    let mut locked = install(root.path(), LOCAL).unwrap();
    locked.packages[0].source.as_mut().unwrap().rev = "0".repeat(40).try_into().unwrap();
    fs::write(
        root.path().join("iris.lock"),
        toml::to_string(&locked).unwrap(),
    )
    .unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::Conflict(_))));
}

#[test]
fn rejects_duplicate_lock_package_when_same_identity_is_repeated() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    let mut locked: SourceLock = install(root.path(), LOCAL).unwrap();
    locked.packages.insert(0, locked.packages[0].clone());
    fs::write(
        root.path().join("iris.lock"),
        toml::to_string(&locked).unwrap(),
    )
    .unwrap();
    let when = prepare(root.path(), &Grants::default());
    assert!(matches!(when, Err(PackageError::Conflict(_))));
}
