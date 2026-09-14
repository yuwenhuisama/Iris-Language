use std::process::Command;

fn check(flags: &[&str], given: &str, expected: &str) {
    let parsed = iris_parser::parse(given);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", given])
        .output()
        .unwrap_or_else(|error| panic!("launch Iris CLI: {error}"));
    let stdout = String::from_utf8_lossy(&when.stdout);
    let stderr = String::from_utf8_lossy(&when.stderr);
    assert_eq!(when.status.code(), Some(0), "{stdout}\n{stderr}");
    assert_eq!(stdout, expected, "stderr: {stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

macro_rules! channel_cases {
    ($($name:ident: $fixture:literal => $expected:expr;)*) => {
        $(mod $name {
            use super::check;

            const GIVEN: &str = include_str!(concat!("decorator_channels/", $fixture, ".iris"));

            #[test]
            fn parser_precheck() {
                let when = iris_parser::parse(GIVEN);
                assert!(when.program_accepted, "{:?}", when.diagnostics);
                assert!(when.diagnostics.is_empty(), "{:?}", when.diagnostics);
            }

            #[test]
            fn reference() {
                check(&[], GIVEN, $expected);
            }

            #[test]
            fn vm() {
                check(&["--vm"], GIVEN, $expected);
            }
        })*
    };
}

channel_cases! {
    meta_c130_positional_closure_when_block_channel_is_absent:
        "positional_only" => "true\ntrue\ntrue\n42\n";
    meta_c130_positional_closure_when_separate_block_is_required:
        "required_block" => "true\n0\n0\ntrue\nfalse\n47\n1\n1\n";
    meta_c131_c133_block_identity_when_forwarded_cleared_and_replaced:
        "block_identity" => concat!(
            "true\ntrue\ntrue\ntrue\n0\ntrue\ntrue\n1\ntrue\ntrue\n40\ntrue\n",
            "false\nfalse\ntrue\ntrue\n5\ntrue\ntrue\n1\ntrue\ntrue\n40\ntrue\n"
        );
    meta_c133_exact_block_signature_when_patch_precedes_inner_effects:
        "block_rejection" => "true\n0\n0\ntrue\n0\n0\ntrue\n0\n0\n7\n1\n1\n";
    meta_c133_c134_channels_when_absent_or_next_arguments_are_invalid:
        "channel_rejection" => "true\ntrue\ntrue\ntrue\ntrue\ntrue\n0\n0\n7\n1\n1\n";
    meta_c133_nested_patch_basis_when_failure_is_caught_before_retry:
        "nested_retry" => "10\n20\ntrue\n10\n10\n7\n20\ntrue\n7\n7\n7\n";
    meta_c133_c134_fresh_attempt_storage_when_replacements_share_objects:
        "fresh_storage" => concat!(
            "1\n1\ntrue\ntrue\n0\n1\n1\ntrue\ntrue\n1\n",
            "false\nfalse\nfalse\nfalse\n10\n20\n10\n20\n1\n1\n2\n"
        );
    meta_c130_inputs_and_defaults_when_attempts_reuse_prepared_values:
        "prepared_once" => "receiver\noperand\nkeyword\nfirst_default\nsecond_default\n1\n1\ntrue\ntrue\n0\n1\n1\ntrue\ntrue\n1\n7\n1\n1\n1\n1\n1\n2\n";
}
