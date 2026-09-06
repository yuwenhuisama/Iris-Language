#![expect(clippy::unwrap_used, reason = "tests assert fixture setup")]
use iris_native_host::{NativePolicy, NativeRegistry, TrustedModule, sha256};

#[test]
fn load_refuses_unauthorized_native_code_before_opening_artifact() {
    let registry = NativeRegistry::new();
    let metadata = br#"{}"#;
    let trusted = TrustedModule {
        path: std::path::Path::new("does-not-exist"),
        metadata,
        metadata_sha256: sha256(metadata),
        artifact_sha256: sha256(b""),
    };
    let result = registry.load(trusted, &NativePolicy::default());
    assert!(matches!(result, Err(iris_native_host::NativeError::Denied)));
}

#[test]
fn metadata_digest_is_computed_not_assumed() {
    let metadata = include_bytes!("fixture/module.json");
    let registry = NativeRegistry::new();
    let result = registry.load(
        TrustedModule {
            path: std::path::Path::new("does-not-exist"),
            metadata,
            metadata_sha256: [0; 32],
            artifact_sha256: [0; 32],
        },
        &NativePolicy {
            allow_native: true,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(iris_native_host::NativeError::Digest)));
}

#[test]
fn required_permissions_are_checked_before_artifact_access() {
    let metadata = String::from_utf8(include_bytes!("fixture/module.json").to_vec())
        .unwrap()
        .replace("\"required\":[]", "\"required\":[\"network\"]");
    let registry = NativeRegistry::new();
    let result = registry.load(
        TrustedModule {
            path: std::path::Path::new("does-not-exist"),
            metadata: metadata.as_bytes(),
            metadata_sha256: sha256(metadata.as_bytes()),
            artifact_sha256: [0; 32],
        },
        &NativePolicy {
            allow_native: true,
            ..Default::default()
        },
    );
    assert!(matches!(result, Err(iris_native_host::NativeError::Denied)));
}
