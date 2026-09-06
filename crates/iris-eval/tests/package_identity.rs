use iris_native_host::{NativeRegistry, PackageSource};
use iris_runtime::Value;
use std::rc::Rc;

fn source(major: u32, text: &str) -> PackageSource {
    PackageSource {
        package_id: "org.iris.app".to_owned(),
        api_major: major,
        version: format!("{major}.3.4"),
        path: "main.iris".to_owned(),
        source: text.to_owned(),
        allowed_imports: Default::default(),
    }
}

#[test]
fn nominal_hash_changes_when_api_major_changes() -> Result<(), String> {
    let mut hashes = Vec::new();
    for major in [1, 2] {
        let given = source(major, "class Widget {} Widget.type.hash()");
        let when =
            iris_eval::evaluate_package_tree_with_natives(&[given], Rc::new(NativeRegistry::new()));
        let expected = Value::Integer(iris_runtime::contract_type_hash(
            "org.iris.app",
            "Widget",
            major.into(),
        ));
        assert_eq!(when, Ok(expected.clone()));
        hashes.push(expected);
    }
    assert_ne!(hashes[0], hashes[1]);
    Ok(())
}

#[test]
fn metadata_is_visible_when_package_is_entered() {
    let given = source(
        2,
        "[Reflection::Package.identity(), Reflection::Package.version()]",
    );
    let when =
        iris_eval::evaluate_package_tree_with_natives(&[given], Rc::new(NativeRegistry::new()));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Array(iris_runtime::ArrayRef::new(vec![
                Value::Symbol("org.iris.app".to_owned()),
                Value::Integer(2_u8.into())
            ])),
            Value::Symbol("2.3.4".to_owned()),
        ])))
    );
}

#[test]
fn conflicting_metadata_is_rejected_when_units_share_package_id() {
    for mismatch in [
        source(2, "nil"),
        PackageSource {
            version: "1.9.0".to_owned(),
            ..source(1, "nil")
        },
        PackageSource {
            allowed_imports: ["Other".to_owned()].into(),
            ..source(1, "nil")
        },
    ] {
        let given = [source(1, "nil"), mismatch];
        let when =
            iris_eval::evaluate_package_tree_with_natives(&given, Rc::new(NativeRegistry::new()));
        assert!(when.is_err());
    }
}

#[test]
fn declaration_keeps_identity_when_another_package_calls_it() {
    let given = [
        source(
            2,
            "class Widget {} module Helper { public module fun value() { [Widget.type.package, Widget.type.hash(), Reflection::Package.version()] } }",
        ),
        PackageSource {
            package_id: "org.iris.caller".to_owned(),
            allowed_imports: ["Helper".to_owned()].into(),
            ..source(1, "import Helper; Helper.value()")
        },
    ];
    let when =
        iris_eval::evaluate_package_tree_with_natives(&given, Rc::new(NativeRegistry::new()));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("org.iris.app".to_owned()),
            Value::Integer(iris_runtime::contract_type_hash(
                "org.iris.app",
                "Widget",
                2
            )),
            Value::Symbol("2.3.4".to_owned()),
        ])))
    );
}
