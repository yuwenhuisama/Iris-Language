#![expect(clippy::expect_used, reason = "tests require a running CLI")]

use std::process::Command;

fn check(flags: &[&str], given: &str, expected: &str) {
    let parsed = iris_parser::parse(given);
    assert!(
        parsed.program_accepted,
        "invalid fixture syntax: {:?}",
        parsed.diagnostics
    );

    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", given])
        .output()
        .expect("launch CLI");

    let stdout = String::from_utf8_lossy(&when.stdout);
    let stderr = String::from_utf8_lossy(&when.stderr);
    assert!(
        when.status.success(),
        "expected stdout: {expected:?}\nactual stdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(stdout, expected, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr: {stderr}");
}

macro_rules! target_cases {
    ($($name:ident: $fixture:literal => $expected:literal;)*) => {
        $(mod $name {
            use super::check;

            pub(super) const GIVEN: &str = include_str!(concat!("decorator_targets/", $fixture, ".iris"));

            #[test]
            fn reference() {
                check(&[], GIVEN, $expected);
            }

            #[test]
            fn vm() {
                check(&["--vm"], GIVEN, $expected);
            }
        })*

        #[test]
        fn fixtures_are_accepted_when_parsed_without_execution() {
            $(let parsed = iris_parser::parse($name::GIVEN);
            assert!(parsed.program_accepted, "{}: {:?}", $fixture, parsed.diagnostics);)*
        }
    };
}

target_cases! {
    meta_c128_class_addition_when_body_captures_private_state:
        "class_addition" => "class\nTarget\norigin\n1\n42\n2\n44\nMethodVisibilityError\n";
    meta_c128_module_addition_when_member_is_mixed_into_class:
        "module_addition" => "module\nProvider\norigin\n1\n43\n2\n45\nMethodVisibilityError\n";
    meta_c128_contract_empty_when_requirements_and_order_are_preserved:
        "contract_empty" => "first\ncontract\nNamed\n2\nsecond\ncontract\nNamed\n2\n2\nFirst\nSecond\nname\n9\n";
    meta_c131_c138_class_object_when_method_generic_is_closed:
        "class_object_generic" => "method\npublic\nmethod\npublic\n1\ntrue\necho\ntrue\nmethod\n0\n1\ntrue\ntrue\ntrue\n7\n7\n";
    meta_c128_c138_module_member_when_wrapper_keeps_declaring_slot:
        "module_member" => "method\nvalue\n1\ntrue\nvalue\ntrue\nmethod\n7\n17\n";
    meta_c128_c129_computed_property_when_accessor_chains_are_independent:
        "computed_property" => "0\n0\nget\ngetter\n1\n5\nset\nsetter\n1\n8\nwritten\nget\ngetter\n2\n8\n2\n1\n";
    meta_c128_stored_property_when_both_generated_accessors_are_wrapped:
        "stored_property" => "property\nvalue\ntrue\ngetter\n4\nsetter\n9\n9\ngetter\n9\n2\n1\n";
    meta_c128_c139_property_candidate_when_setter_is_absent:
        "missing_accessor_rollback" => "true\ntrue\ntrue\ntrue\nold\n0\n";
    meta_c131_c138_qualified_method_when_one_ordinary_surface_is_reused:
        "qualified_method" => "1\ntrue\nname\nfalse\nmethod\nname\n1\n2\ntrue\nname\nfalse\nmethod\nname\n2\n3\ntrue\nname\nfalse\nmethod\nname\n3\n";
    meta_c128_c138_protected_method_when_visibility_is_not_runtime_protection:
        "protected_method" => "protected\n1\n7\n17\nMethodVisibilityError\n1\n1\n";
}
