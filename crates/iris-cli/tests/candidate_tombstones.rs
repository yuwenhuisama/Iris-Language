#![expect(clippy::expect_used, reason = "tests require a running CLI")]

use std::process::Command;

const CLASSES: &str = r#"
    class Parent { public fun total() -> Object { "bad" } }
    class Rule extends Parent { public override fun total() -> Integer { 10 } }
"#;

fn check(flags: &[&str], operation: &str) {
    let given = format!(
        "{CLASSES} let changed = Rule.open({{ |candidate|; {operation} }}); print(Rule.method(:total))"
    );
    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", &given])
        .output()
        .expect("launch CLI");
    assert!(when.status.success(), "{when:?}");
    assert_eq!(String::from_utf8_lossy(&when.stdout), "nil\n");
    assert!(when.stderr.is_empty(), "{when:?}");
}

macro_rules! both_engines {
    ($($name:ident: $operation:expr;)*) => {
        $(mod $name {
            use super::*;

            #[test]
            fn reference() {
                check(&[], $operation);
            }

            #[test]
            fn vm() {
                check(&["--vm"], $operation);
            }
        })*
    };
}

both_engines! {
    undef_succeeds_when_callback_blocks_incompatible_ancestor:
        "candidate.undef_method(:total)";
    undef_succeeds_when_callback_discards_intermediate_bad_replacement:
        "candidate.define_method(:total) { \"bad\" }; candidate.undef_method(:total)";
}
