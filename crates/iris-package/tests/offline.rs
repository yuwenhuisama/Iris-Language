mod common;
use common::{manifest, project, repository};
use iris_package::{BuildOptions, Grants, InstallOptions, PackageError, build, install, prepare};
use std::process::Command;

#[test]
fn runs_prepare_and_denied_build_when_no_commands_are_available() {
    let (repo, rev) = repository(&manifest("org.iris.network"));
    let root = project(repo.path(), &rev);
    install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    repo.close().unwrap();
    let when = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "offline_child"])
        .env("IRIS_PACKAGE_TEST_ROOT", root.path())
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        when.status.success(),
        "{}{}",
        String::from_utf8_lossy(&when.stdout),
        String::from_utf8_lossy(&when.stderr)
    );
}

#[test]
fn offline_child() {
    let Some(root) = std::env::var_os("IRIS_PACKAGE_TEST_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let when = prepare(root, &Grants::default()).unwrap();
    assert_eq!(when.packages.len(), 2);
    assert!(matches!(
        build(root, BuildOptions::default()),
        Err(PackageError::NativeBuildDenied)
    ));
}
