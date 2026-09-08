#![expect(clippy::expect_used, reason = "tests require a running CLI")]

use std::process::Command;

#[test]
fn property_assignment_initializer_uses_setter_result_in_both_engines() {
    let given = "class Box { public property fun name=(value) -> Symbol { :written } } mut local = 0; let local_result = (local = 1); let property_result = (Box.new().name = 2); print(local_result); print(property_result)";
    for flags in [&[][..], &["--vm"][..]] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(flags)
            .args(["-e", given])
            .output()
            .expect("launch CLI");
        assert!(when.status.success(), "{flags:?}: {when:?}");
        assert_eq!(String::from_utf8_lossy(&when.stdout), "1\nwritten\n");
        assert!(when.stderr.is_empty(), "{when:?}");
    }
}

#[test]
fn property_assignment_to_fixed_symbol_accepts_symbol_setter_result() {
    let given = "class Box { public property fun name=(value) -> Symbol { :written } } mut result = :initial; result = Box.new().name = 2; print(result)";
    for flags in [&[][..], &["--vm"][..]] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(flags)
            .args(["-e", given])
            .output()
            .expect("launch CLI");
        assert!(when.status.success(), "{flags:?}: {when:?}");
        assert_eq!(String::from_utf8_lossy(&when.stdout), "written\n");
        assert!(when.stderr.is_empty(), "{when:?}");
    }
}

#[test]
fn property_assignment_to_fixed_integer_rejects_symbol_setter_result() {
    let given = "class Box { public property fun name=(value) -> Symbol { :written } } mut result = 8; try { result = Box.new().name = 2 } catch error { print(error) }; print(result)";
    for flags in [&[][..], &["--vm"][..]] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(flags)
            .args(["-e", given])
            .output()
            .expect("launch CLI");
        assert!(when.status.success(), "{flags:?}: {when:?}");
        assert_eq!(
            String::from_utf8_lossy(&when.stdout),
            "TypeContractError\n8\n"
        );
        assert!(when.stderr.is_empty(), "{when:?}");
    }
}

#[test]
fn reraising_preserves_context_when_capture_has_explicit_wide_contract() {
    let given = "mut captured: Object = nil; try { try { raise :x } catch _, context { captured = context; raise } } catch _, outer { print(outer same? captured); print(outer.re_raise_sites[0].location.line) }";
    for flags in [&[][..], &["--vm"][..]] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(flags)
            .args(["-e", given])
            .output()
            .expect("launch CLI");
        assert!(when.status.success(), "{flags:?}: {when:?}");
        assert_eq!(String::from_utf8_lossy(&when.stdout), "true\n1\n");
        assert!(when.stderr.is_empty(), "{when:?}");
    }
}

#[test]
fn failed_nil_context_capture_leaves_cell_unchanged() {
    let given = "mut captured = nil; try { try { raise :x } catch _, context { captured = context; raise } } catch error { print(error); print(captured) }";
    for flags in [&[][..], &["--vm"][..]] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(flags)
            .args(["-e", given])
            .output()
            .expect("launch CLI");
        assert!(when.status.success(), "{flags:?}: {when:?}");
        assert_eq!(
            String::from_utf8_lossy(&when.stdout),
            "TypeContractError\nnil\n"
        );
        assert!(when.stderr.is_empty(), "{when:?}");
    }
}
