use iris_package::{Grants, InstallOptions, install, prepare};
mod common;
use common::{commit, manifest, project, repository};
use std::fs;

#[test]
fn accepts_complete_host_metadata_when_native_resource_is_declared() {
    let text = manifest("org.iris.network").replace("required = []", "required = ['native.load']")
        + r#"
[native]
abi_major=1
minimum_minor=0
required_features=0
metadata='native-api.json'
cargo_manifest='Cargo.toml'
cargo_package='iris-network'
cargo_target='iris_network'
"#;
    let (repo, _) = repository(&text);
    fs::write(repo.path().join("Cargo.toml"), "").unwrap();
    fs::write(repo.path().join("native-api.json"), r#"{
"schema_version":1,"package_id":"org.iris.network","version":"1.2.0","api_major":1,"iris_major":1,
"abi":{"major":1,"minimum_minor":0,"required_features":0},
"permissions":{"required":["native.load"],"optional":[]},"reflection_policy":"deny",
"modules":[{"name":"Network","functions":[{"name":"close","parameters":[{"name":"socket","type":"resource:Socket"}],"returns":"Nil"}],
"resources":[{"name":"Socket","contracts":["Closeable"],"managed_roots":false,"storage":"external-cookie-v1"}]}]}
"#).unwrap();
    let rev = commit(repo.path());
    let root = project(repo.path(), &rev);
    let when = install(
        root.path(),
        InstallOptions {
            allow_local_git: true,
        },
    )
    .unwrap();
    assert!(when.packages[0].metadata_sha256.is_some());
    assert!(prepare(root.path(), &Grants::default()).is_err());
}
