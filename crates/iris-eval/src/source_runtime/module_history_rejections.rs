use super::{CURRENT, HISTORY, Session, attach};

#[test]
fn exact_failure_leaves_revisions_unchanged_when_spine_policy_or_artifact_rejects() {
    for (current, history) in [
        (
            CURRENT,
            "module Provider { public fun value() -> String { \"bad\" } }",
        ),
        (
            CURRENT,
            "module Provider { private fun value() -> Integer { 1 } }",
        ),
        (
            CURRENT,
            "module Provider { public module fun value() -> Integer { 1 } }",
        ),
        (
            CURRENT,
            "module Provider { public fun value() -> Integer { 1 }; public class property state: Integer = 2 }",
        ),
        (
            "module Provider meta deny method_body { public fun value() -> Integer { 9 } }; 0",
            HISTORY,
        ),
        (
            "module Provider { public fun value() -> Integer { 9 }; public fun later() -> Integer { 8 } }; 0",
            HISTORY,
        ),
        (
            "module Extra {}; module Provider mixin Extra { public fun value() -> Integer { 9 } }; 0",
            HISTORY,
        ),
        (
            "module Provider { public class property state: Integer = 8; public fun value() -> Integer { 9 } }; 0",
            HISTORY,
        ),
    ] {
        let mut given = Session::new().unwrap();
        given.evaluate(current).unwrap();
        attach(&mut given, history);
        let module = given.evaluator.module_names["Provider"];
        let main = given.evaluator.module_mains[&module];
        let before = given
            .evaluator
            .runtime
            .registry()
            .active_module(module)
            .unwrap()
            .clone();
        let main_revision = given
            .evaluator
            .runtime
            .registry()
            .active(main)
            .unwrap()
            .id();

        let when = given.evaluate("Provider.rollback(1)");

        assert!(when.is_err(), "{current}: {history}");
        assert_eq!(
            given
                .evaluator
                .runtime
                .registry()
                .active_module(module)
                .unwrap(),
            &before
        );
        assert_eq!(
            given
                .evaluator
                .runtime
                .registry()
                .active(main)
                .unwrap()
                .id(),
            main_revision
        );
        assert_eq!(given.evaluate("Provider.value()"), crate::evaluate("9"));
    }
}
