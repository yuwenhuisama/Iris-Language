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
            dependencies: vec![],
            version: None,
            locked: Vec::new(),
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
    // C003 also lists permission and native-artifact fields. This loader does
    // not model them, so carrying one must neither fail the load nor look like
    // support for it. `version` IS modelled now, since V420 reflects the
    // resolved package identity, so it is retained rather than ignored.
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
            dependencies: vec![],
            version: Some("1.2.3".into()),
            locked: Vec::new(),
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

#[test]
fn c006_orders_a_dependency_before_its_dependent() {
    // IRIS-V1-META-C006 selects dependencies BEFORE initialization and
    // IRIS-V1-META-C017 initializes a dependency before its dependent, so a
    // tree load returns dependencies first.
    let root = tempdir("tree");
    write(
        &root,
        "org.dep/iris.toml",
        "package_id = \"org.dep\"\napi_major = 1\nsources = [\"src/main.ir\"]\n",
    );
    write(&root, "org.dep/src/main.ir", "module Dep { }");
    write(
        &root,
        "org.app/iris.toml",
        "package_id = \"org.app\"\napi_major = 1\nsources = [\"src/main.ir\"]\ndependencies = [\"org.dep\"]\n",
    );
    write(&root, "org.app/src/main.ir", "module App { }");

    // When
    let loaded = super::load_tree(&root, "org.app");

    // Then
    assert_eq!(
        loaded.map(|packages| packages
            .into_iter()
            .map(|package| package.package_id)
            .collect::<Vec<_>>()),
        Ok(vec!["org.dep".to_owned(), "org.app".to_owned()])
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn c007_aborts_on_a_missing_or_cyclic_dependency() {
    // IRIS-V1-META-C007 aborts package linking on a resolution failure, so a
    // dependency that does not exist and a dependency cycle are both reported
    // rather than skipped.
    let missing = tempdir("missing-dep");
    write(
        &missing,
        "org.app/iris.toml",
        "package_id = \"org.app\"\napi_major = 1\ndependencies = [\"org.absent\"]\n",
    );
    let cyclic = tempdir("cyclic-dep");
    write(
        &cyclic,
        "org.a/iris.toml",
        "package_id = \"org.a\"\napi_major = 1\ndependencies = [\"org.b\"]\n",
    );
    write(
        &cyclic,
        "org.b/iris.toml",
        "package_id = \"org.b\"\napi_major = 1\ndependencies = [\"org.a\"]\n",
    );

    // When / Then
    assert!(super::load_tree(&missing, "org.app").is_err());
    assert_eq!(
        super::load_tree(&cyclic, "org.a"),
        Err("package dependency cycle at org.a".into())
    );
    for directory in [missing, cyclic] {
        let _ = std::fs::remove_dir_all(&directory);
    }
}
