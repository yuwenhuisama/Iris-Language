use iris_native_host::{NativeError, NativePolicy, NativeRegistry, TrustedModule, sha256};
use iris_runtime::Value;
use iris_syntax::{MethodKind, ParameterCategory, Statement, TypeExpression, Visibility};
use std::path::PathBuf;

fn fixture() -> Result<NativeRegistry, Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let executable = std::env::current_exe()?;
    let target = executable
        .ancestors()
        .nth(3)
        .ok_or("missing target directory")?;
    let status = std::process::Command::new(env!("CARGO"))
        .arg("build")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target)
        .status()?;
    assert!(status.success());
    let path = target.join("debug").join(format!(
        "{}iris_native_test_module{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let metadata = include_bytes!("fixture/module.json");
    let registry = NativeRegistry::new();
    registry.load(
        TrustedModule {
            path: &path,
            metadata,
            metadata_sha256: sha256(metadata),
            artifact_sha256: sha256(&std::fs::read(&path)?),
        },
        &NativePolicy {
            allow_native: true,
            max_resources: 4,
            ..Default::default()
        },
    )?;
    Ok(registry)
}

#[test]
fn signatures_are_typed_when_real_native_metadata_is_admitted()
-> Result<(), Box<dyn std::error::Error>> {
    let given = fixture()?;
    let expected = [
        ("nil_value", Some(("value", "Nil")), "Nil"),
        ("boolean", Some(("value", "Bool")), "Bool"),
        ("integer", Some(("value", "Integer")), "Integer"),
        ("text", Some(("value", "String")), "String"),
        ("bytes", Some(("value", "Bytes")), "Bytes"),
        ("fail", None, "Nil"),
        ("create", None, "Object"),
        ("closes", None, "Integer"),
        ("available", Some(("resource", "Object")), "Bool"),
    ];

    let when = given.declarations()?;

    assert_eq!(when.len(), 1);
    let (package, module) = &when[0];
    assert_eq!(package, "test/native");
    assert_eq!(module.name, "NativeTest");
    assert_eq!(module.body.len(), expected.len());
    for (statement, (selector, parameter, result)) in module.body.iter().zip(expected) {
        let Statement::Method(method) = statement else {
            return Err("expected native method declaration".into());
        };
        assert_eq!(method.selector, selector);
        assert_eq!(method.kind, MethodKind::Module);
        assert_eq!(method.visibility, Visibility::Public);
        assert!(!method.is_async);
        let actual = method
            .parameters
            .iter()
            .map(|parameter| {
                assert_eq!(parameter.category, ParameterCategory::Positional);
                assert!(parameter.default.is_none());
                (parameter.name.clone(), parameter.annotation.clone())
            })
            .collect::<Vec<_>>();
        let expected = parameter
            .into_iter()
            .map(|(name, annotation)| {
                (
                    name.to_owned(),
                    Some(TypeExpression::Name(annotation.to_owned())),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{selector}");
        assert_eq!(
            method.return_type,
            Some(TypeExpression::Name(result.into())),
            "{selector}"
        );
    }
    let skeleton = iris_parser::parse(
        "module NativeTest meta deny method_set, method_body, property_set, property_body, modules, superclass, subclass, shape, class_state_set, class_state_write, instance_state, native { public module fun placeholder() { nil } }",
    );
    let iris_syntax::Declaration::Module(protected) = &skeleton.program.declarations[0] else {
        return Err("expected protected module".into());
    };
    assert_eq!(module.meta_deny, protected.meta_deny);
    let Statement::Method(placeholder) = &protected.body[0] else {
        return Err("expected placeholder method".into());
    };
    for statement in &module.body {
        let Statement::Method(method) = statement else {
            return Err("expected native method".into());
        };
        assert_eq!(method.body, placeholder.body);
    }
    Ok(())
}

#[test]
fn exact_abi_contract_survives_when_source_declarations_are_generated()
-> Result<(), Box<dyn std::error::Error>> {
    let given = fixture()?;
    let metadata = serde_json::to_value(given.metadata())?;

    let _declarations = given.declarations()?;

    assert_eq!(serde_json::to_value(given.metadata())?, metadata);
    for integer in ["-9223372036854775808", "9223372036854775807"] {
        let value = Value::Integer(integer.parse()?);
        assert_eq!(
            given.call("NativeTest.integer", std::slice::from_ref(&value))?,
            value
        );
    }
    for integer in ["-9223372036854775809", "9223372036854775808"] {
        assert!(matches!(
            given.call("NativeTest.integer", &[Value::Integer(integer.parse()?)]),
            Err(NativeError::Type)
        ));
    }
    assert!(matches!(
        given.call("NativeTest.integer", &[Value::Bool(true)]),
        Err(NativeError::Type)
    ));
    assert!(matches!(
        given.call("NativeTest.integer", &[]),
        Err(NativeError::Type)
    ));
    let resource = given.call("NativeTest.create", &[])?;
    assert_eq!(
        given.call("NativeTest.available", &[resource])?,
        Value::Bool(true)
    );
    assert!(matches!(
        given.call("NativeTest.available", &[Value::Nil]),
        Err(NativeError::Type)
    ));
    Ok(())
}
