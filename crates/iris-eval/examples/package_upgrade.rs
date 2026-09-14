use iris_eval::{PackageHistory, PackageResolution, PackageUpgrade, ResolvedPackageVersion};

fn main() {
    let current = "class Counter { public fun value() -> Integer { 1 } }";
    let target = "class Counter { public fun value() -> Integer { 2 } }";
    let outcome = iris_eval::load_resolved_package_with_upgrades(
        PackageUpgrade {
            history: PackageHistory {
                resolution: PackageResolution {
                    package_id: "example.counter", api_major: 1, version: Some("1.0".into()),
                    locked: Vec::new(), artifact: None, permissions: &[], grants: Vec::new(),
                },
                artifacts: Vec::new(),
            },
            versions: vec![ResolvedPackageVersion {
                package_id: "example.counter".into(), api_major: 1, version: "1.1".into(),
                artifact: ("host-owned".into(), iris_runtime::artifact_digest(target.as_bytes()), target.into()),
            }],
        },
        &[("counter.iris".into(), current.into())],
        Some("let object = Counter.new(); let retained = object.value; let result = Reflection::Package.upgrade(:\"1.1\"); [object.value(), retained.call(), Reflection::Package.version()]"),
    ).expect("resolved upgrade");
    println!("{:?}", outcome.1);
}
