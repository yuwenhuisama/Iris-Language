use super::{Error, args, sources};
use iris_package::{Manifest, PreparedPackage, PreparedPackageTree, PreparedSource, RelativePath};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn rejects_digest_when_not_exact_sha256_hex() {
    for given in [
        "00".repeat(31),
        "00".repeat(33),
        "gg".repeat(32),
        "+1".repeat(32),
        "é".repeat(32),
    ] {
        let when = sources::digest(&given);
        assert!(matches!(when, Err(Error::Digest)), "{given}");
    }
}

#[test]
fn decodes_digest_when_exact_sha256_hex() -> TestResult {
    let given = "0123456789abcdef".repeat(4);
    let when = sources::digest(&given)?;
    assert_eq!(
        when,
        [1, 35, 69, 103, 137, 171, 205, 239].repeat(4).as_slice()
    );
    Ok(())
}

#[test]
fn refuses_arguments_when_ambiguous_or_malformed() {
    for given in [
        vec!["run", ".", "--allow"],
        vec!["run", ".", "--allow", "native.load,,network.tcp"],
        vec!["run", ".", "--vm", "--vm"],
        vec![
            "run",
            ".",
            "--allow",
            "native.load",
            "--allow",
            "network.tcp",
        ],
        vec!["run", ".", "--allow-native-build"],
        vec!["install", ".", "--allow", "native.load"],
        vec!["build", ".", "--vm"],
    ] {
        let given = given.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let when = args::parse(&given);
        assert!(when.is_err());
    }
}

#[test]
fn collects_sources_when_direct_dependencies_declare_entry_modules() -> TestResult {
    let mut direct = package("org.iris.direct", "Network")?;
    direct.manifest.dependencies = Manifest::parse(&format!(
        "{}{}",
        text("org.iris.direct", "Network"),
        dependency("org.iris.transitive")
    ))?
    .dependencies;
    let mut root = package("org.iris.app", "Main")?;
    root.manifest.dependencies = Manifest::parse(&format!(
        "{}{}",
        text("org.iris.app", "Main"),
        dependency("org.iris.direct")
    ))?
    .dependencies;
    root.sources.push(PreparedSource {
        path: RelativePath::try_from("second.iris".to_owned())?,
        bytes: b"print(2)".to_vec(),
    });
    let given = PreparedPackageTree {
        packages: vec![package("org.iris.transitive", "Secret")?, direct, root],
    };
    let when = sources::collect(&given)?;
    assert_eq!(
        when.iter()
            .map(|source| source.package_id.as_str())
            .collect::<Vec<_>>(),
        [
            "org.iris.transitive",
            "org.iris.direct",
            "org.iris.app",
            "org.iris.app"
        ]
    );
    assert_eq!(
        when[2].allowed_imports,
        ["Main".to_owned(), "Network".to_owned()].into()
    );
    assert_eq!(when[3].source, "print(2)");
    assert!(when[3].path.ends_with("second.iris"));
    Ok(())
}

fn text(identity: &str, module: &str) -> String {
    format!(
        r#"manifest_version = 1
package_id = "{identity}"
api_major = 1
version = "1.0.0"
iris_major = 1
sources = ["main.iris"]
entry_modules = ["{module}"]
[permissions]
required = []
optional = []
"#
    )
}

fn dependency(identity: &str) -> String {
    format!(
        r#"
[dependencies.direct]
package_id = "{identity}"
api_major = 1
version = "^1.0"
git = "https://example.invalid/package"
rev = "0123456789012345678901234567890123456789"
"#
    )
}

fn package(identity: &str, module: &str) -> Result<PreparedPackage, Box<dyn std::error::Error>> {
    Ok(PreparedPackage {
        root: identity.into(),
        manifest: Manifest::parse(&text(identity, module))?,
        sources: vec![PreparedSource {
            path: RelativePath::try_from("main.iris".to_owned())?,
            bytes: b"print(1)".to_vec(),
        }],
        native: None,
    })
}
