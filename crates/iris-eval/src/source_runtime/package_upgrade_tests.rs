use crate::{ResolvedPackageVersion, Session};

#[test]
fn all_revisions_share_one_commit_when_mixed_package_upgrades() {
    let current = "class First { public fun value() -> Integer { 1 } }; class Last { public fun value() -> Integer { 2 } }; module Provider { public fun value() -> Integer { 3 } }";
    let target = current
        .replace("{ 1 }", "{ 10 }")
        .replace("{ 2 }", "{ 20 }")
        .replace("{ 3 }", "{ 30 }");
    let mut given = Session::new().unwrap();
    given.evaluate(&format!("{current}; 0")).unwrap();
    let first = given.evaluator.class_name("First").unwrap().unwrap();
    let last = given.evaluator.class_name("Last").unwrap().unwrap();
    let module = given.evaluator.module_names["Provider"];
    let main = given.evaluator.module_mains[&module];
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(main)
        .unwrap()
        .commit_id();
    given.evaluator.enter_package_upgrades(
        vec![ResolvedPackageVersion {
            package_id: super::super::LOCAL_PACKAGE.into(),
            api_major: 1,
            version: "next".into(),
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(target.as_bytes()),
                target,
            ),
        }],
        &[("current".into(), current.into())],
    );

    given
        .evaluate("Reflection::Package.upgrade(:next)")
        .unwrap();

    for class in [first, last, main] {
        assert_eq!(
            given
                .evaluator
                .runtime
                .registry()
                .active(class)
                .unwrap()
                .commit_id(),
            before + 1
        );
    }
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active_module(module)
            .unwrap()
            .commit_id(),
        before + 1
    );
}
