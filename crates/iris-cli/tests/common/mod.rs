use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn manifest(identity: &str) -> String {
    format!(
        r#"manifest_version = 1
package_id = "{identity}"
api_major = 1
version = "1.0.0"
iris_major = 1
sources = ["main.iris"]
entry_modules = ["Main"]
[permissions]
required = []
optional = []
"#
    )
}

pub fn project(source: &str) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("iris.toml"), manifest("org.iris.app"))?;
    fs::write(root.path().join("main.iris"), source)?;
    Ok(root)
}

pub fn cli(root: &Path, operation: &str, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(["package", operation])
        .arg(root)
        .args(flags)
        .output()
        .expect("launch CLI")
}

pub fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn failure(output: &Output, diagnostic: &str) {
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
