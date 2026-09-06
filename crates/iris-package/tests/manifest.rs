use iris_package::Manifest;

const VALID: &str = r#"
manifest_version = 1
package_id = "org.iris.network"
api_major = 1
version = "1.2.0"
iris_major = 1
sources = ["src/network.iris"]
entry_modules = ["Network"]
[permissions]
required = ["native.load", "native.blocking", "network.tcp"]
optional = []
[native]
abi_major = 1
minimum_minor = 0
required_features = 0
metadata = "native-api.json"
cargo_manifest = "Cargo.toml"
cargo_package = "iris-network"
cargo_target = "iris_network"
"#;

#[test]
fn parses_nested_native_manifest_when_valid() {
    let given = VALID;
    let when = Manifest::parse(given);
    let then = when.unwrap();
    assert_eq!(then.package_id.as_str(), "org.iris.network");
    assert_eq!(then.permissions.required.len(), 3);
    assert_eq!(then.native.unwrap().cargo_target, "iris_network");
}

#[test]
fn rejects_manifest_when_boundary_is_malformed() {
    for given in [
        VALID.replace("manifest_version = 1", "manifest_version = 2"),
        VALID.replace("iris_major = 1", "iris_major = 2"),
        VALID.replace("org.iris.network", "network"),
        VALID.replace("1.2.0", "not-semver"),
        VALID.replace("src/network.iris", "../network.iris"),
        VALID.replace("src/network.iris", "src\\\\network.iris"),
        VALID.replace("api_major = 1", "api_major = -1"),
        VALID.replace("cargo_target =", "misspelled_target ="),
        VALID.replace("required =", "unexpected ="),
    ] {
        let when = Manifest::parse(&given);
        assert!(when.is_err(), "accepted {given}");
    }
}

#[test]
fn rejects_transport_when_url_or_revision_is_unsafe() {
    for (url, rev) in [
        ("https://user:secret@example.com/repo", "a".repeat(40)),
        ("https://token@example.com/repo", "a".repeat(40)),
        ("ssh://git@example.com/repo", "a".repeat(40)),
        ("http://example.com/repo", "a".repeat(40)),
        ("ext::command", "a".repeat(40)),
        ("../local", "a".repeat(40)),
        ("https://example.com/repo", "main".to_owned()),
        ("https://example.com/repo", "abc123".to_owned()),
    ] {
        let given = format!(
            "{VALID}\n[dependencies.dep]\npackage_id='org.iris.dep'\napi_major=1\nversion='^1'\ngit='{url}'\nrev='{rev}'\n"
        );
        let when = Manifest::parse(&given);
        assert!(when.is_err());
    }
}

#[test]
fn rejects_path_when_platform_spelling_is_unsafe() {
    for path in [
        "/absolute",
        "a/../b",
        "./main.iris",
        "a//b",
        ".git/config",
        ".iris/file",
        "C:/file",
        "NUL",
        "aux.txt",
        "file.",
        "file ",
    ] {
        let given = path.to_owned();
        let when = iris_package::RelativePath::try_from(given);
        assert!(when.is_err(), "{path}");
    }
}
