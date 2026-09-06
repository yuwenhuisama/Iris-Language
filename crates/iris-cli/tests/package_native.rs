#![expect(clippy::expect_used, reason = "tests assert fixture setup")]
mod common;
use common::{TestResult, cli, failure, manifest, project, success};
use std::{fs, path::Path, process::Command};

fn git(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .current_dir(root)
        .output()
        .expect("run fixture git");
    success(&output);
    String::from_utf8(output.stdout)
        .expect("git UTF-8")
        .trim()
        .to_owned()
}

#[test]
fn loads_real_native_when_built_and_granted() -> TestResult {
    let given_repo = tempfile::tempdir()?;
    let repo = given_repo.path();
    fs::create_dir_all(repo.join("src"))?;
    fs::create_dir_all(repo.join("sdk/src"))?;
    fs::write(
        repo.join("src/lib.rs"),
        include_bytes!("../../iris-native-host/tests/fixture/src/lib.rs"),
    )?;
    fs::write(
        repo.join("build.rs"),
        include_bytes!("../../iris-native-host/tests/fixture/build.rs"),
    )?;
    fs::write(
        repo.join("sdk/src/lib.rs"),
        include_bytes!("../../iris-native-sdk/src/lib.rs"),
    )?;
    fs::write(
        repo.join("sdk/Cargo.toml"),
        include_bytes!("../../iris-native-sdk/Cargo.toml"),
    )?;
    fs::write(
        repo.join("Cargo.toml"),
        include_str!("../../iris-native-host/tests/fixture/Cargo.toml")
            .replace("../../../iris-native-sdk", "sdk"),
    )?;
    fs::write(
        repo.join("module.json"),
        include_str!("../../iris-native-host/tests/fixture/module.json")
            .replace("test/native", "org.iris.native")
            .replace("\"required\":[]", "\"required\":[\"native.load\"]"),
    )?;
    fs::write(
        repo.join("iris.toml"),
        manifest("org.iris.native")
            .replace("sources = [\"main.iris\"]", "sources = []")
            .replace(
                "entry_modules = [\"Main\"]",
                "entry_modules = [\"NativeTest\"]",
            )
            .replace("required = []", "required = [\"native.load\"]")
            + r#"
[native]
abi_major = 1
minimum_minor = 0
required_features = 0
metadata = "module.json"
cargo_manifest = "Cargo.toml"
cargo_package = "iris-native-test-module"
cargo_target = "iris_native_test_module"
"#,
    )?;
    let lock = Command::new(env!("CARGO"))
        .args(["generate-lockfile", "--offline"])
        .current_dir(repo)
        .output()?;
    success(&lock);
    git(repo, &["init", "--quiet"]);
    git(repo, &["add", "."]);
    git(repo, &["commit", "--quiet", "-m", "fixture"]);
    let revision = git(repo, &["rev-parse", "HEAD"]);
    let given = project(
        "import NativeTest; print(NativeTest.integer(42)); let resource = NativeTest.create(); print(NativeTest.available(resource)); resource.close()",
    )?;
    fs::write(
        given.path().join("iris.toml"),
        manifest("org.iris.app")
            + &format!(
                r#"
[dependencies.native]
package_id = "org.iris.native"
api_major = 1
version = "^1.0"
git = "file://{}"
rev = "{revision}"
"#,
                repo.display()
            ),
    )?;
    failure(&cli(given.path(), "install", &[]), "allow_local_git");
    success(&cli(given.path(), "install", &["--allow-local-git"]));
    failure(&cli(given.path(), "build", &[]), "allow_native_build");
    failure(&cli(given.path(), "run", &[]), "native.load denied");
    failure(
        &cli(given.path(), "run", &["--allow", "native.load"]),
        "build",
    );
    success(&cli(given.path(), "build", &["--allow-native-build"]));
    given_repo.close()?;
    for flags in [
        vec!["--allow", "native.load"],
        vec!["--vm", "--allow", "native.load"],
    ] {
        let when = cli(given.path(), "run", &flags);
        success(&when);
        assert_eq!(String::from_utf8(when.stdout)?, "42\ntrue\n");
    }
    let prepared = iris_package::prepare(
        given.path(),
        &iris_package::Grants {
            permissions: ["native.load".to_owned()].into(),
        },
    )?;
    let artifact = &prepared.packages[0]
        .native
        .as_ref()
        .expect("native artifact")
        .artifact;
    fs::write(artifact, b"tampered")?;
    let when = cli(given.path(), "run", &["--allow", "native.load"]);
    failure(&when, "integrity");
    assert!(when.stdout.is_empty());
    Ok(())
}
