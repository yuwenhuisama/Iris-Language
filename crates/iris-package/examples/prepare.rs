use iris_package::{BuildOptions, Grants, InstallOptions, build, install, prepare};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = tempfile::tempdir()?;
    fs::write(
        project.path().join("iris.toml"),
        r#"
manifest_version = 1
package_id = "org.iris.example"
api_major = 1
version = "1.0.0"
iris_major = 1
sources = ["main.iris"]
entry_modules = ["Main"]
[permissions]
required = []
optional = []
"#,
    )?;
    fs::write(project.path().join("main.iris"), "module Main {}\n")?;
    let installed = install(project.path(), InstallOptions::default())?;
    let prepared = prepare(project.path(), &Grants::default())?;
    for package in prepared.packages {
        println!(
            "{} {}: {} ordered source(s)",
            package.manifest.package_id,
            package.manifest.version,
            package.sources.len()
        );
        for source in package.sources {
            println!(
                "{}: {}",
                source.path,
                std::str::from_utf8(&source.bytes)?.trim()
            );
        }
    }
    println!("locked packages: {}", installed.packages.len());
    match build(project.path(), BuildOptions::default()) {
        Err(iris_package::PackageError::NativeBuildDenied) => {
            println!("native build denied before command execution")
        }
        other => return Err(format!("unexpected build outcome: {other:?}").into()),
    }
    Ok(())
}
