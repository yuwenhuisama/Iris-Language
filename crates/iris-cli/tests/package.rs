#![expect(clippy::expect_used, reason = "tests assert fixture setup")]
mod common;
use common::{TestResult, cli, failure, project, success};
use std::fs;

#[test]
fn executes_pure_package_when_installed() -> TestResult {
    let given = project("print(6 * 7)")?;
    success(&cli(given.path(), "install", &[]));
    for flags in [vec![], vec!["--vm"]] {
        let when = cli(given.path(), "run", &flags);
        success(&when);
        assert_eq!(String::from_utf8(when.stdout)?, "42\n");
    }
    Ok(())
}

#[test]
fn preserves_type_identity_when_manifest_selects_api_major() -> TestResult {
    for major in [1, 2] {
        let given =
            project("class Widget {} print(Widget.type.package); print(Widget.type.hash())")?;
        fs::write(
            given.path().join("iris.toml"),
            common::manifest("org.iris.app")
                .replace("api_major = 1", &format!("api_major = {major}"))
                .replace("1.0.0", &format!("{major}.3.4")),
        )?;
        success(&cli(given.path(), "install", &[]));
        for flags in [vec![], vec!["--vm"]] {
            let when = cli(given.path(), "run", &flags);
            success(&when);
            assert_eq!(
                String::from_utf8(when.stdout)?,
                format!(
                    "org.iris.app\n{}\n",
                    iris_runtime::contract_type_hash("org.iris.app", "Widget", major)
                        .to_u64()
                        .expect("hash is u64")
                ),
            );
        }
    }
    Ok(())
}

#[test]
fn refuses_run_when_lock_missing() -> TestResult {
    let given = project("print(42)")?;
    let when = cli(given.path(), "run", &[]);
    failure(&when, "iris.lock");
    assert!(when.stdout.is_empty());
    Ok(())
}

#[test]
fn refuses_run_when_source_tampered() -> TestResult {
    let given = project("print(42)")?;
    success(&cli(given.path(), "install", &[]));
    fs::write(given.path().join("main.iris"), "print(99)")?;
    let when = cli(given.path(), "run", &[]);
    failure(&when, "integrity");
    assert!(when.stdout.is_empty());
    Ok(())
}

#[test]
fn refuses_build_when_consent_missing() -> TestResult {
    let given = project("print(42)")?;
    let when = cli(given.path(), "build", &[]);
    failure(&when, "allow_native_build");
    assert!(!given.path().join(".iris").exists());
    Ok(())
}

#[test]
fn refuses_load_when_permission_missing() -> TestResult {
    let given = project("print(42)")?;
    let path = given.path().join("iris.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)?.replace("required = []", "required = [\"native.load\"]"),
    )?;
    success(&cli(given.path(), "install", &[]));
    let when = cli(given.path(), "run", &[]);
    failure(&when, "native.load denied");
    assert!(when.stdout.is_empty());
    Ok(())
}

#[test]
fn executes_vm_when_same_package_has_multiple_source_units() -> TestResult {
    let given = project("print(42)")?;
    let path = given.path().join("iris.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)?.replace("[\"main.iris\"]", "[\"first.iris\", \"main.iris\"]"),
    )?;
    fs::write(given.path().join("first.iris"), "print(1)")?;
    success(&cli(given.path(), "install", &[]));
    let when = cli(given.path(), "run", &["--vm"]);
    success(&when);
    assert_eq!(String::from_utf8(when.stdout)?, "1\n42\n");
    Ok(())
}

#[test]
fn refuses_import_when_module_not_declared() -> TestResult {
    let given = project("import Secret; print(42)")?;
    success(&cli(given.path(), "install", &[]));
    for flags in [vec![], vec!["--vm"]] {
        let when = cli(given.path(), "run", &flags);
        failure(&when, "import");
        assert!(when.stdout.is_empty());
    }
    Ok(())
}

#[test]
fn executes_helper_when_same_package_has_multiple_sources() -> TestResult {
    let given = project("import Main; print(Main.answer())")?;
    let path = given.path().join("iris.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)?.replace("[\"main.iris\"]", "[\"first.iris\", \"main.iris\"]"),
    )?;
    fs::write(
        given.path().join("first.iris"),
        "module Main { public module fun answer() { 42 } }",
    )?;
    success(&cli(given.path(), "install", &[]));
    for flags in [vec![], vec!["--vm"]] {
        let when = cli(given.path(), "run", &flags);
        success(&when);
        assert_eq!(String::from_utf8(when.stdout)?, "42\n");
    }
    Ok(())
}
