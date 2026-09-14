#![expect(clippy::expect_used, reason = "tests require a running CLI")]

use std::process::Command;

fn check(flags: &[&str], source: &str, expected: &str) {
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
    assert!(
        when.status.success(),
        "expected stdout: {expected:?}\nactual stdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(stdout, expected, "stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr: {stderr}");
}

macro_rules! async_cases {
    ($($name:ident: $fixture:literal => $expected:literal;)*) => {
        $(mod $name {
            use super::check;

            const SOURCE: &str = include_str!(concat!("decorator_async/", $fixture, ".iris"));

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

async_cases! {
    meta_c136_eager_forwarding_when_inner_is_ready:
        "eager_ready" => "0\n0\n0\n1\n1\n1\n7\n7\n1\n1\n1\n";
    meta_c136_c137_prefix_once_when_owner_resumes_then_calls_sync_helper:
        "pending_owner" => "1\n0\n0\n7\n1\n1\n1\n";
    meta_c136_fresh_bridges_and_outer_when_forwarding_repeatedly:
        "bridge_identity" => "false\nfalse\nfalse\nfalse\ntrue\ntrue\n4\n";
    meta_c136_failed_outer_when_initial_input_is_invalid:
        "invalid_initial" => "returned\n0\n0\ntrue\n0\n0\n";
    meta_c136_c139_failed_bridge_when_patch_is_invalid:
        "bad_patch" => "returned\ntrue\n0\n0\n7\n0\n0\n";
    meta_c137_c139_failed_bridge_when_owner_has_expired:
        "expired" => "7\nreturned\nfalse\ntrue\nexpired\ntrue\ntrue\nexpired\n0\n";
    meta_c137_c139_foreign_helper_rejected_when_eager_prefix_calls_next:
        "foreign_eager" => "returned\ntrue\nforeign_task\ntrue\n0\n7\n1\n1\n";
    meta_c137_c139_overlap_rejected_when_first_bridge_is_pending:
        "overlap" => "returned\ntrue\noverlap\ntrue\n1\n0\n7\n2\n2\n";
    meta_c137_c139_inner_continues_when_owner_returns_unfinished:
        "unfinished_normal" => "returned\ntrue\nunfinished_inner\ntrue\n1\n0\n7\n1\n1\n0\n";
    meta_c135_c136_failed_admission_when_zero_attempt_result_violates_r:
        "zero_attempt_result" => "returned\n1\n0\ntrue\ntrue\n1\n0\n";
}
