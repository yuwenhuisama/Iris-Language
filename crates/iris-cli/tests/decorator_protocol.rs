#![expect(clippy::expect_used, reason = "tests require a running CLI")]

use std::process::Command;

enum Expected {
    Stdout(&'static str),
    Diagnostic {
        code: &'static str,
        stdout: &'static str,
    },
}

fn check(flags: &[&str], source: &str, expected: Expected) {
    let given = iris_parser::parse(source);
    assert!(
        given.program_accepted,
        "invalid fixture syntax: {:?}",
        given.diagnostics
    );

    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", source])
        .output()
        .expect("launch CLI");
    let stdout = String::from_utf8_lossy(&when.stdout);
    let stderr = String::from_utf8_lossy(&when.stderr);

    match expected {
        Expected::Stdout(expected) => {
            assert!(when.status.success(), "stdout: {stdout}\nstderr: {stderr}");
            assert_eq!(stdout, expected, "stderr: {stderr}");
            assert!(stderr.is_empty(), "stderr: {stderr}");
        }
        Expected::Diagnostic {
            code,
            stdout: expected,
        } => {
            assert!(!when.status.success(), "expected {code}; stdout: {stdout}");
            assert_eq!(
                stdout, expected,
                "unexpected phase effects; stderr: {stderr}"
            );
            assert!(stderr.contains(code), "expected {code}; stderr: {stderr}");
        }
    }
}

macro_rules! protocol_cases {
    ($($name:ident: $fixture:literal => $expected:expr;)*) => {
        $(mod $name {
            use super::{Expected, check};

            const SOURCE: &str = include_str!(concat!("decorator_protocol/", $fixture, ".iris"));

            #[test]
            fn reference() {
                check(&[], SOURCE, $expected);
            }

            #[test]
            fn vm() {
                check(&["--vm"], SOURCE, $expected);
            }
        })*
    };
}

protocol_cases! {
    meta_c127_c132_argument_changes_available_outside_phase:
        "argument_changes_available" => Expected::Stdout("true\ntrue\n");
    meta_c127_c132_argument_changes_constructor_accepts_all_public_fields:
        "argument_changes_constructor_fields" => Expected::Stdout("true\ntrue\ntrue\n7\n");
    meta_c132_constructor_rejects_non_symbol_keys:
        "argument_changes_bad_key" => Expected::Stdout("true\n");
    meta_c127_c139_plan_outside_phase_has_typed_category:
        "plan_outside_phase" => Expected::Stdout("true\noutside_phase\nnil\n");
    meta_c127_c139_transformation_outside_phase_has_typed_category:
        "transformation_outside_phase" => Expected::Stdout("true\noutside_phase\nnil\n");
    meta_c127_c131_core_records_and_snapshots_are_immutable:
        "immutable_snapshots" => Expected::Stdout("true\nReadonlyMutationError\ntrue\ntrue\ntrue\nReadonlyMutationError\nReadonlyMutationError\nReadonlyMutationError\n7\n");
    meta_c127_phase_instances_are_fresh_and_arguments_reach_members:
        "phase_instances" => Expected::Stdout("fresh\nmethod\norigin\n23\nmethod\nbody\n7\n");
    meta_c127_plan_executes_and_failure_prevents_runtime_phase:
        "plan_failure" => Expected::Diagnostic { code: "plan_ran", stdout: "" };
    meta_c129_first_written_wrapper_is_outermost:
        "wrapper_order" => Expected::Stdout("transform_A\ntransform_B\nready\nenter_A\nenter_B\nbody\nexit_B\nexit_A\n7\n");
    meta_c138_captured_state_persists_per_chain_not_per_decorator_class:
        "captured_state" => Expected::Stdout("1\n2\n1\n3\n2\n");
    meta_c132_c133_symbol_patches_snapshot_structure_and_keep_channels_separate:
        "parameter_patch" => Expected::Stdout("10\n20\npatched\ntrue\n1\n2\noriginal\nfalse\n30\n");
    meta_c133_bad_patch_type_prevents_every_inner_effect:
        "bad_patch_type" => Expected::Stdout("true\n0\n0\n");
    meta_c130_invalid_initial_input_prevents_zero_attempt_wrapper:
        "bad_initial_type" => Expected::Stdout("true\n0\n0\n");
    meta_c134_c135_zero_attempt_result_still_obeys_target_contract:
        "zero_attempt_result" => Expected::Stdout("true\n0\n");
    meta_c130_c134_c137_retry_repeats_inner_suffix_but_defaults_only_once:
        "retry_defaults" => Expected::Stdout("1\n1\n7\n1\n2\n");
    meta_c127_c139_initialize_is_inside_static_purity_boundary:
        "impure_initialize" => Expected::Diagnostic { code: "IRIS-DECORATOR-NONDETERMINISTIC", stdout: "" };
    meta_c127_c139_plan_call_graph_is_inside_static_purity_boundary:
        "impure_plan" => Expected::Diagnostic { code: "IRIS-DECORATOR-NONDETERMINISTIC", stdout: "" };
    meta_c139_provable_target_kind_mismatch_is_rejected:
        "wrong_target_kind" => Expected::Diagnostic { code: "IRIS-DECORATOR-KIND", stdout: "" };
    meta_c128_c139_denied_capability_rolls_back_members_revision_and_commit:
        "capability_rollback" => Expected::Stdout("true\ntrue\ntrue\ntrue\nold\n");
    meta_c127_c139_retained_wrong_kind_rolls_back_candidate:
        "wrong_kind_rollback" => Expected::Diagnostic { code: "IRIS-DECORATOR-KIND", stdout: "true\ntrue\ntrue\nold\n" };
}
