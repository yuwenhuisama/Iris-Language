#![expect(clippy::expect_used, reason = "tests assert CLI launch")]

use std::process::{Command, Output};

enum Expected {
    Success(&'static str),
    TypeFailure(&'static str),
    OpenFailure,
}

fn run(flags: &[&str], source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", source])
        .output()
        .expect("launch Iris CLI")
}

fn assert_outcome(output: &Output, expected: Expected) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    match expected {
        Expected::Success(text) => {
            assert!(output.status.success(), "{output:?}");
            assert_eq!(stdout, text, "{output:?}");
            assert!(stderr.is_empty(), "{output:?}");
        }
        Expected::TypeFailure(text) => {
            assert_type_failure(output);
            assert_eq!(stdout, text, "{output:?}");
        }
        Expected::OpenFailure => {
            assert_type_failure(output);
            assert!(stdout.is_empty() || stdout == "100\n", "{output:?}");
        }
    }
}

fn assert_type_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        ["TypeError", "TypeContractError", "BINDING_FIXED_LOCAL_TYPE"]
            .iter()
            .any(|diagnostic| stderr.contains(diagnostic)),
        "{output:?}"
    );
    let lowercase = stderr.to_ascii_lowercase();
    for rejected in ["parse", "unsupported", "machine does not cover"] {
        assert!(!lowercase.contains(rejected), "{output:?}");
    }
}

macro_rules! both_engines {
    ($($name:ident: $source:expr => $expected:expr;)*) => {
        $(mod $name {
            use super::*;

            #[test]
            fn reference() {
                let given = $source;
                let when = run(&[], &given);
                assert_outcome(&when, $expected);
            }

            #[test]
            fn vm() {
                let given = $source;
                let when = run(&["--vm"], &given);
                assert_outcome(&when, $expected);
            }
        })*
    };
}

const INPUT: &str = "module Input { public module fun read(value) { value } };";
const MARKED_INPUT: &str =
    "module Input { public module fun read(value) { print(\"rhs\"); value } };";
const SERVER: &str = "module Server { public module fun start(port: Integer) -> Integer { print(\"entered start\"); port } };";
const SHIPPING: &str = "class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); print(rule.total(90));";

both_engines! {
    inferred_literal_fails_when_assignment_changes_type:
        "print(\"before\"); mut port = 8080; port = \"9090\"; print(port is String);"
        => Expected::TypeFailure("");
    annotated_literal_fails_when_assignment_changes_type:
        "print(\"before\"); mut port: Integer = 8080; port = \"9090\"; print(port is String);"
        => Expected::TypeFailure("");
    union_literal_fails_when_assignment_leaves_union:
        "print(\"before\"); mut port: Integer | String = 8080; port = true; print(port is Bool);"
        => Expected::TypeFailure("");

    inferred_method_local_fails_when_method_is_uncalled:
        "print(\"before\"); class Uncalled { public fun check() { mut port = 8080; port = \"9090\"; port } };"
        => Expected::TypeFailure("");
    annotated_method_local_fails_when_method_is_uncalled:
        "print(\"before\"); class Uncalled { public fun check() { mut port: Integer = 8080; port = \"9090\"; port } };"
        => Expected::TypeFailure("");
    union_method_local_fails_when_method_is_uncalled:
        "print(\"before\"); class Uncalled { public fun check() { mut port: Integer | String = 8080; port = true; port } };"
        => Expected::TypeFailure("");
    inferred_closure_local_fails_when_closure_is_uncalled:
        "print(\"before\"); let uncalled = { || mut port = 8080; port = \"9090\"; port };"
        => Expected::TypeFailure("");
    annotated_closure_local_fails_when_closure_is_uncalled:
        "print(\"before\"); let uncalled = { || mut port: Integer = 8080; port = \"9090\"; port };"
        => Expected::TypeFailure("");
    union_closure_local_fails_when_closure_is_uncalled:
        "print(\"before\"); let uncalled = { || mut port: Integer | String = 8080; port = true; port };"
        => Expected::TypeFailure("");

    open_fails_when_return_signature_is_incompatible:
        format!("{SHIPPING} open class ShippingRule {{ public override fun total(subtotal: Integer) -> String {{ \"free shipping\" }} }}; print(rule.total(90)); print(\"after\");")
        => Expected::OpenFailure;
    open_succeeds_when_return_signature_is_compatible:
        format!("{SHIPPING} open class ShippingRule {{ public override fun total(subtotal: Integer) -> Integer {{ subtotal }} }}; print(rule.total(90));")
        => Expected::Success("100\n90\n");

    inferred_local_succeeds_when_integer_updated:
        "mut port = 8080; port = 9090; print(port);"
        => Expected::Success("9090\n");
    annotated_local_succeeds_when_integer_updated:
        "mut port: Integer = 8080; port = 9090; print(port);"
        => Expected::Success("9090\n");
    union_local_succeeds_when_member_changes:
        "mut port: Integer | String = 8080; port = \"9090\"; print(port);"
        => Expected::Success("9090\n");
    object_local_succeeds_when_updates_are_heterogeneous:
        "mut port: Object = 8080; port = \"9090\"; print(port is String); port = true; print(port is Bool);"
        => Expected::Success("true\ntrue\n");
    dynamic_local_succeeds_when_updates_are_heterogeneous:
        "mut port: Dynamic<Object> = 8080; port = \"9090\"; print(port is String); port = true; print(port is Bool);"
        => Expected::Success("true\ntrue\n");

    inferred_local_fails_when_dynamic_rhs_changes_type:
        format!("{MARKED_INPUT} mut port = 8080; port = Input.read(\"9090\"); print(port); print(\"after\");")
        => Expected::TypeFailure("rhs\n");
    annotated_local_fails_when_dynamic_rhs_changes_type:
        format!("{MARKED_INPUT} mut port: Integer = 8080; port = Input.read(\"9090\"); print(port); print(\"after\");")
        => Expected::TypeFailure("rhs\n");
    union_local_fails_when_dynamic_rhs_leaves_union:
        format!("{MARKED_INPUT} mut port: Integer | String = 8080; port = Input.read(true); print(port); print(\"after\");")
        => Expected::TypeFailure("rhs\n");
    helper_succeeds_when_returning_heterogeneous_values:
        format!("{MARKED_INPUT} print(Input.read(\"9090\")); print(Input.read(true));")
        => Expected::Success("rhs\n9090\nrhs\ntrue\n");

    inferred_cell_retains_value_when_dynamic_assignment_fails:
        format!("{MARKED_INPUT} mut port = 8080; try {{ port = Input.read(\"9090\"); print(\"stored\"); }} catch error {{ print(\"caught\"); }}; print(port);")
        => Expected::Success("rhs\ncaught\n8080\n");
    annotated_cell_retains_value_when_dynamic_assignment_fails:
        format!("{MARKED_INPUT} mut port: Integer = 8080; try {{ port = Input.read(\"9090\"); print(\"stored\"); }} catch error {{ print(\"caught\"); }}; print(port);")
        => Expected::Success("rhs\ncaught\n8080\n");
    union_cell_retains_value_when_dynamic_assignment_fails:
        format!("{MARKED_INPUT} mut port: Integer | String = 8080; try {{ port = Input.read(true); print(\"stored\"); }} catch error {{ print(\"caught\"); }}; print(port);")
        => Expected::Success("rhs\ncaught\n8080\n");

    parameter_guard_succeeds_when_dynamic_input_is_integer:
        format!("{INPUT} {SERVER} print(Server.start(Input.read(8080))); print(Server.start(Input.read(-1)));")
        => Expected::Success("entered start\n8080\nentered start\n-1\n");
    parameter_guard_fails_when_dynamic_input_is_string:
        format!("{INPUT} {SERVER} print(Server.start(Input.read(\"8080\")));")
        => Expected::TypeFailure("");
    return_guard_fails_when_dynamic_result_is_string:
        format!("{INPUT} module Settings {{ public module fun port(raw) -> Integer {{ print(\"entered port\"); Input.read(raw) }} }}; print(Settings.port(\"8080\"));")
        => Expected::TypeFailure("entered port\n");
}
