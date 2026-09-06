use iris_native_host::{NativePolicy, NativeRegistry, TrustedModule, sha256};
use iris_runtime::Value;
use std::{path::PathBuf, rc::Rc};

#[test]
fn both_engines_execute_real_native_module_calls() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../iris-native-host/tests/fixture");
    let target = std::env::temp_dir().join(format!("iris-native-fixture-{}", std::process::id()));
    let status = std::process::Command::new(env!("CARGO"))
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .status()?;
    assert!(status.success());
    let path = target.join("debug").join(format!(
        "{}iris_native_test_module{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    // SAFETY: this test built the trusted fixture, whose counter export has this C signature.
    let library = unsafe { libloading::Library::new(&path) }?;
    // SAFETY: the fixture export is retained by library through the final assertion.
    let destroys = unsafe {
        library.get::<unsafe extern "C" fn() -> u64>(b"iris_native_test_destroy_count\0")
    }?;
    let metadata = std::fs::read(root.join("module.json"))?;
    let registry = Rc::new(NativeRegistry::new());
    registry.load(
        TrustedModule {
            path: &path,
            metadata: &metadata,
            metadata_sha256: sha256(&metadata),
            artifact_sha256: sha256(&std::fs::read(&path)?),
        },
        &NativePolicy {
            allow_native: true,
            permissions: Default::default(),
            max_resources: 32,
        },
    )?;
    for (source, expected) in [
        (
            "import NativeTest; try { Reflection::Module.method(NativeTest, :integer) } catch error { error }",
            Value::Symbol("ReflectionPermissionError".into()),
        ),
        (
            "import NativeTest; NativeTest.integer(-42)",
            Value::Integer("-42".parse()?),
        ),
        ("import NativeTest; NativeTest.nil_value(nil)", Value::Nil),
        (
            "import NativeTest; NativeTest.boolean(true)",
            Value::Bool(true),
        ),
        (
            "import NativeTest; NativeTest.text(\"hello\")",
            Value::Text("hello".into()),
        ),
        (
            "import NativeTest; NativeTest.bytes(b\"abc\")",
            Value::Bytes(b"abc".to_vec()),
        ),
        (
            "import NativeTest; try { NativeTest.fail() } catch error { error }",
            Value::Symbol("NativeTestError".into()),
        ),
        (
            "import NativeTest; try { NativeTest.fail() } catch error, context { context.value == error && context.original_stack.length == 1 }",
            Value::Bool(true),
        ),
        (
            "import NativeTest; try { NativeTest.integer(9223372036854775808) } catch error { error }",
            Value::Symbol("TypeContractError".into()),
        ),
        (
            "import NativeTest; try { NativeTest.integer(9223372036854775808) } catch error, context { context.value == error }",
            Value::Bool(true),
        ),
        (
            "import NativeTest; let resource = NativeTest.create(); let alias = resource; let before = NativeTest.closes(); let ignored = resource.close(); let twice = alias.close(); NativeTest.closes() - before",
            Value::Integer(1_u8.into()),
        ),
        (
            "import NativeTest; let resource = NativeTest.create(); let before = NativeTest.closes(); let ignored = using(resource) { 42 }; NativeTest.closes() - before",
            Value::Integer(1_u8.into()),
        ),
        (
            "import NativeTest; NativeTest.available(NativeTest.create())",
            Value::Bool(true),
        ),
        (
            "import NativeTest; let resource = NativeTest.create(); let ignored = resource.close(); try { NativeTest.available(resource) } catch error { error }",
            Value::Symbol("ClosedResourceError".into()),
        ),
    ] {
        let actual = iris_eval::evaluate_with_natives(source, Rc::clone(&registry));
        assert_eq!(actual, Ok(expected.clone()), "evaluator: {source}");
        let compiled = iris_vm::compile_with_natives(source, &registry);
        assert!(compiled.is_ok(), "compiler: {source}: {compiled:?}");
        let Ok(compiled) = compiled else {
            continue;
        };
        let mut machine = iris_vm::Machine::with_natives(Rc::clone(&registry))
            .map_err(|error| format!("{error:?}"))?;
        assert_eq!(machine.execute(&compiled), Ok(expected), "VM: {source}");
    }
    for source in [
        "import NativeTest; try { NativeTest.fail() } catch error, context { context }",
        "import NativeTest; try { try { NativeTest.fail() } catch error, context { raise } } catch error, context { context }",
        "import NativeTest; let original = try { NativeTest.fail() } catch error, context { context }; try { raise :outer from original } catch error, context { context.cause }",
    ] {
        let evaluated = iris_eval::evaluate_with_natives(source, Rc::clone(&registry))
            .map_err(|error| format!("{error:?}"))?;
        let compiled = iris_vm::compile_with_natives(source, &registry)
            .map_err(|error| format!("{error:?}"))?;
        let executed = iris_vm::run_with_natives(&compiled, Rc::clone(&registry))
            .map_err(|error| format!("{error:?}"))?;
        for context in [evaluated, executed] {
            let Value::ExceptionContext(_, value, cause, suppressed, _, origin) = context else {
                return Err(format!("expected native exception context: {source}").into());
            };
            assert_eq!(*value, Value::Symbol("NativeTestError".into()));
            assert_eq!(*cause, Value::Nil);
            assert!(suppressed.is_empty());
            assert_eq!(origin.original_stack.len(), 1);
            let bridge = origin.native_bridge.ok_or("native bridge was lost")?;
            assert_eq!(bridge.package, "test/native");
            assert_eq!(bridge.code, "NativeTestError");
            assert_eq!(bridge.message, "test failure");
            assert_eq!(bridge.os_code, 42);
        }
    }
    let altered = String::from_utf8(metadata.clone())?.replace("test/native", "other/native");
    let mismatch = NativeRegistry::new().load(
        TrustedModule {
            path: &path,
            metadata: altered.as_bytes(),
            metadata_sha256: sha256(altered.as_bytes()),
            artifact_sha256: sha256(&std::fs::read(&path)?),
        },
        &NativePolicy {
            allow_native: true,
            permissions: Default::default(),
            max_resources: 1,
        },
    );
    assert!(matches!(
        mismatch,
        Err(iris_native_host::NativeError::Digest)
    ));
    let bad_artifact = NativeRegistry::new().load(
        TrustedModule {
            path: &path,
            metadata: &metadata,
            metadata_sha256: sha256(&metadata),
            artifact_sha256: [0; 32],
        },
        &NativePolicy {
            allow_native: true,
            permissions: Default::default(),
            max_resources: 1,
        },
    );
    assert!(matches!(
        bad_artifact,
        Err(iris_native_host::NativeError::Digest)
    ));
    assert!(
        iris_eval::evaluate_with_natives("NativeTest.integer(1)", Rc::clone(&registry)).is_err()
    );
    if let Ok(hidden) = iris_vm::compile_with_natives("NativeTest.integer(1)", &registry) {
        assert!(iris_vm::run_with_natives(&hidden, Rc::clone(&registry)).is_err());
    }
    assert!(iris_vm::compile_with_natives("import NativeTest; module NativeTest { public fun integer(value) { 99 } } NativeTest.integer(1)", &registry).is_err());
    let sources = vec![
        iris_native_host::PackageSource { package_id: "helper".into(), api_major: 1, version: "1.0.0".into(), path: "helper.iris".into(), source: "import NativeTest; module Helper { public module fun value() { NativeTest.integer(23) } }".into(), allowed_imports: ["NativeTest".into()].into() },
        iris_native_host::PackageSource { package_id: "app".into(), api_major: 1, version: "1.0.0".into(), path: "app.iris".into(), source: "import Helper; Helper.value()".into(), allowed_imports: ["Helper".into()].into() },
    ];
    assert_eq!(
        iris_eval::evaluate_package_tree_with_natives(&sources, Rc::clone(&registry)),
        Ok(Value::Integer(23_u8.into()))
    );
    assert!(iris_vm::compile_package_tree_with_natives(&sources, &registry).is_err());
    let mut denied = sources.clone();
    denied[0].allowed_imports.clear();
    assert!(iris_eval::evaluate_package_tree_with_natives(&denied, Rc::clone(&registry)).is_err());
    let one = vec![iris_native_host::PackageSource {
        package_id: "app".into(),
        api_major: 1,
        version: "1.0.0".into(),
        path: "app.iris".into(),
        source: "import NativeTest; NativeTest.integer(24)".into(),
        allowed_imports: ["NativeTest".into()].into(),
    }];
    let one = iris_vm::compile_package_tree_with_natives(&one, &registry)
        .map_err(|error| format!("{error:?}"))?;
    assert_eq!(
        iris_vm::run_with_natives(&one, Rc::clone(&registry)),
        Ok(Value::Integer(24_u8.into()))
    );
    let owned = [iris_native_host::PackageSource {
        package_id: "org.iris.entry".into(),
        api_major: 2,
        version: "2.3.4".into(),
        path: "entry.iris".into(),
        source: "import NativeTest; try { NativeTest.fail() } catch error, context { context }"
            .into(),
        allowed_imports: ["NativeTest".into()].into(),
    }];
    let compiled = iris_vm::compile_package_tree_with_natives(&owned, &registry)
        .map_err(|error| format!("{error:?}"))?;
    let evaluated = iris_eval::evaluate_package_tree_with_natives(&owned, Rc::clone(&registry))
        .map_err(|error| format!("{error:?}"))?;
    let executed = iris_vm::run_with_natives(&compiled, Rc::clone(&registry))
        .map_err(|error| format!("{error:?}"))?;
    for when in [evaluated, executed] {
        let Value::ExceptionContext(_, _, _, _, _, origin) = when else {
            return Err("expected native exception context".into());
        };
        assert_eq!(
            origin.native_bridge.ok_or("missing native owner")?.package,
            "test/native"
        );
    }
    let unsupported = [iris_native_host::PackageSource {
        source: "import NativeTest; NativeTest.type.package".into(),
        ..owned[0].clone()
    }];
    assert!(
        iris_vm::compile_package_tree_with_natives(&unsupported, &registry)
            .map(|program| iris_vm::run_with_natives(&program, Rc::clone(&registry)).is_err())
            .unwrap_or(true)
    );
    drop(registry);
    // SAFETY: counter pointer is still backed by library and takes no arguments.
    assert_eq!(unsafe { destroys() }, 8);
    drop(library);
    std::fs::remove_dir_all(target)?;
    Ok(())
}
