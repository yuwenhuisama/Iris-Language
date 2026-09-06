use iris_native_host::{NativeRegistry, PackageSource};
use iris_runtime::Value;
use std::rc::Rc;

fn source(package: &str, text: &str) -> PackageSource {
    PackageSource {
        package_id: package.to_owned(),
        api_major: 1,
        version: "1.0.0".to_owned(),
        path: "fixture.ir".to_owned(),
        source: text.to_owned(),
        allowed_imports: ["Helper".to_owned()].into(),
    }
}

#[test]
fn helper_is_callable_when_units_share_package_identity() -> Result<(), String> {
    let registry = Rc::new(NativeRegistry::new());
    let sources = [
        source(
            "app",
            "module Helper { public module fun value() { 42 } } // end",
        ),
        source("app", "import Helper; Helper.value()"),
    ];
    let program = iris_vm::compile_package_tree_with_natives(&sources, &registry)
        .map_err(|error| error.construct)?;
    iris_vm::verify(&program).map_err(|error| format!("{error:?}"))?;
    let result =
        iris_vm::run_with_natives(&program, registry).map_err(|error| format!("{error:?}"))?;
    assert_eq!(result, Value::Integer(42_u64.into()));
    Ok(())
}

#[test]
fn statements_keep_manifest_order_when_units_share_package_identity() -> Result<(), String> {
    let registry = Rc::new(NativeRegistry::new());
    let sources = [
        source("app", "mut value = 6"),
        source("app", "value = value * 7"),
    ];
    let program = iris_vm::compile_package_tree_with_natives(&sources, &registry)
        .map_err(|error| error.construct)?;
    let result =
        iris_vm::run_with_natives(&program, registry).map_err(|error| format!("{error:?}"))?;
    assert_eq!(result, Value::Integer(42_u64.into()));
    Ok(())
}

#[test]
fn invalid_first_unit_is_rejected_before_joining() {
    let sources = [source("app", "module Helper {"), source("app", "}")];
    let result = iris_vm::compile_package_tree_with_natives(&sources, &NativeRegistry::new());
    assert!(result.is_err());
}

#[test]
fn foreign_packages_are_explicitly_rejected() {
    let sources = [source("helper", "module Helper {}"), source("app", "nil")];
    let result = iris_vm::compile_package_tree_with_natives(&sources, &NativeRegistry::new());
    assert!(
        matches!(result, Err(error) if error.construct.contains("cross-package execution is not implemented"))
    );
}

#[test]
fn import_permission_is_checked_per_unit() {
    let sources = [
        source("app", "module Helper {}"),
        source("app", "import Forbidden; nil"),
    ];
    let result = iris_vm::compile_package_tree_with_natives(&sources, &NativeRegistry::new());
    assert!(matches!(result, Err(error) if error.construct.contains("import permission")));
}

#[test]
fn trusted_tuple_adapter_accepts_same_package_units() -> Result<(), String> {
    let registry = Rc::new(NativeRegistry::new());
    let sources = [
        ("app".to_owned(), "let value = 42".to_owned()),
        ("app".to_owned(), "value".to_owned()),
    ];
    let program = iris_vm::compile_packages_with_natives(&sources, &registry)
        .map_err(|error| error.construct)?;
    let result =
        iris_vm::run_with_natives(&program, registry).map_err(|error| format!("{error:?}"))?;
    assert_eq!(result, Value::Integer(42_u64.into()));
    Ok(())
}

#[test]
fn trusted_tuple_adapter_rejects_invalid_first_unit() {
    let sources = [
        ("app".to_owned(), "module Helper {".to_owned()),
        ("app".to_owned(), "}".to_owned()),
    ];
    let result = iris_vm::compile_packages_with_natives(&sources, &NativeRegistry::new());
    assert!(result.is_err());
}

#[test]
fn nominal_hash_changes_when_api_major_changes() -> Result<(), String> {
    let mut hashes = Vec::new();
    for major in [1, 2] {
        let given = PackageSource {
            api_major: major,
            version: format!("{major}.3.4"),
            ..source("org.iris.app", "class Widget {} Widget.type.hash()")
        };
        let registry = NativeRegistry::new();
        let program = iris_vm::compile_package_tree_with_natives(&[given], &registry)
            .map_err(|error| error.construct)?;
        assert_eq!(
            program.package_identity(),
            Some(&iris_vm::PackageIdentity {
                package_id: "org.iris.app".to_owned(),
                api_major: major,
                version: Some(format!("{major}.3.4")),
            })
        );
        let when = iris_vm::run(&program).map_err(|error| format!("{error:?}"))?;
        assert_eq!(
            when,
            Value::Integer(iris_runtime::contract_type_hash(
                "org.iris.app",
                "Widget",
                major.into()
            ))
        );
        hashes.push(when);
    }
    assert_ne!(hashes[0], hashes[1]);
    Ok(())
}

#[test]
fn conflicting_metadata_is_rejected_when_units_share_package_id() {
    for mismatch in [
        PackageSource {
            api_major: 2,
            ..source("app", "nil")
        },
        PackageSource {
            version: "1.9.0".to_owned(),
            ..source("app", "nil")
        },
        PackageSource {
            allowed_imports: Default::default(),
            ..source("app", "nil")
        },
    ] {
        let given = [source("app", "nil"), mismatch];
        let when = iris_vm::compile_package_tree_with_natives(&given, &NativeRegistry::new());
        assert!(when.is_err());
    }
}

#[test]
fn contract_hash_uses_package_identity_when_compiled_as_package() -> Result<(), String> {
    for major in [1, 2] {
        let given = PackageSource {
            api_major: major,
            ..source("org.iris.app", "contract Named {} Named.hash()")
        };
        let program = iris_vm::compile_package_tree_with_natives(&[given], &NativeRegistry::new())
            .map_err(|error| error.construct)?;
        let when = iris_vm::run(&program);
        assert_eq!(
            when,
            Ok(Value::Integer(iris_runtime::contract_type_hash(
                "org.iris.app",
                "Named",
                major.into()
            )))
        );
    }
    Ok(())
}

#[test]
fn type_identity_stays_local_when_compiled_as_script() -> Result<(), String> {
    let given = iris_vm::compile("class Widget {} [Widget.type.package, Widget.type.hash()]")
        .map_err(|error| error.construct)?;
    let when = iris_vm::run(&given);
    assert_eq!(given.package_identity(), None);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("runtime-local".to_owned()),
            Value::Integer(iris_runtime::contract_type_hash(
                "runtime-local",
                "Widget",
                1
            ))
        ])))
    );
    Ok(())
}

#[test]
fn package_reflection_is_explicitly_unsupported_in_vm() {
    let given = [source("app", "Reflection::Package.version()")];
    let when = iris_vm::compile_package_tree_with_natives(&given, &NativeRegistry::new());
    assert!(when.is_err());
}

#[test]
fn builtin_type_identity_is_refused_instead_of_using_entry_package() -> Result<(), String> {
    for text in ["Integer.type.package", "Integer.type.hash()"] {
        let given = [source("app", text)];
        let program = iris_vm::compile_package_tree_with_natives(&given, &NativeRegistry::new())
            .map_err(|error| error.construct)?;
        let when = iris_vm::run(&program);
        assert_eq!(when, Err(iris_vm::MachineError::UnsupportedConstruct));
    }
    Ok(())
}

#[test]
fn contract_view_hash_uses_declared_major() -> Result<(), String> {
    let given = [PackageSource {
        api_major: 2,
        version: "2.3.4".to_owned(),
        ..source(
            "app",
            "contract Named {} class Widget for Named { public fun hash() { 17 } } (Widget.new() as Named).hash()",
        )
    }];
    let program = iris_vm::compile_package_tree_with_natives(&given, &NativeRegistry::new())
        .map_err(|error| error.construct)?;
    let when = iris_vm::run(&program);
    let contract = iris_runtime::contract_type_hash("app", "Named", 2)
        .to_u64()
        .ok_or("hash must be u64")?;
    assert_eq!(
        when,
        Ok(Value::Integer(iris_runtime::contract_view_hash(
            17, contract
        )))
    );
    Ok(())
}
