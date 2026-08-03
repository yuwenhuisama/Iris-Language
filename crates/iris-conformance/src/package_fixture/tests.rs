use super::{Package, load};

#[test]
fn it_loads_a_manifest_and_its_ordered_sources() {
    // IRIS-V1-META-C003 requires `package_id`, `api_major` and ordered source
    // entries, and IRIS-V1-META-C017 initializes in manifest-declared source
    // order, so the order is load-bearing rather than incidental.
    let directory = tempdir("loads");
    write(
        &directory,
        "iris.toml",
        "package_id = \"org.iris.test\"\napi_major = 1\nsources = [\"src/a.ir\", \"src/b.ir\"]\n",
    );
    write(&directory, "src/a.ir", "module A { }");
    write(&directory, "src/b.ir", "module B { }");

    // When
    let package = load(&directory);

    // Then
    assert_eq!(
        package,
        Ok(Package {
            package_id: "org.iris.test".into(),
            api_major: 1,
            sources: vec![
                ("src/a.ir".into(), "module A { }".into()),
                ("src/b.ir".into(), "module B { }".into()),
            ],
        })
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_missing_or_incomplete_manifest_aborts_the_load() {
    // IRIS-V1-META-C010: a missing or invalid manifest ABORTS the load, so a
    // defaulted identity would be worse than a reported failure.
    let absent = tempdir("absent");
    let incomplete = tempdir("incomplete");
    write(&incomplete, "iris.toml", "api_major = 1\n");
    let missing_source = tempdir("missing-source");
    write(
        &missing_source,
        "iris.toml",
        "package_id = \"org.iris.test\"\napi_major = 1\nsources = [\"src/gone.ir\"]\n",
    );

    // When / Then
    assert!(load(&absent).is_err());
    assert_eq!(
        load(&incomplete),
        Err("manifest declares no package_id".into())
    );
    assert!(load(&missing_source).is_err());
    for directory in [absent, incomplete, missing_source] {
        let _ = std::fs::remove_dir_all(&directory);
    }
}

#[test]
fn an_unmodelled_manifest_key_is_ignored_rather_than_honoured() {
    // C003 also lists dependency, permission and native-artifact fields. This
    // loader does not model them, so carrying one must neither fail the load
    // nor look like support for it.
    let directory = tempdir("extra-keys");
    write(
        &directory,
        "iris.toml",
        "package_id = \"org.iris.test\"\napi_major = 1\nversion = \"1.2.3\"\n[dependencies]\norg.dep = \"^2.0\"\n",
    );

    // When / Then
    assert_eq!(
        load(&directory),
        Ok(Package {
            package_id: "org.iris.test".into(),
            api_major: 1,
            sources: vec![],
        })
    );
    let _ = std::fs::remove_dir_all(&directory);
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("iris-package-fixture-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    let _ = std::fs::create_dir_all(&directory);
    directory
}

fn write(directory: &std::path::Path, relative: &str, contents: &str) {
    let path = directory.join(relative);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, contents);
}
