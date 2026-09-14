use std::process::Command;

enum Expected {
    Success(&'static str),
    WrongKind(&'static str),
}

fn check(flags: &[&str], source: &str, expected: Expected) {
    let given = iris_parser::parse(source);
    assert!(given.program_accepted, "{:?}", given.diagnostics);

    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", source])
        .output();
    let output = when.unwrap_or_else(|error| panic!("launch Iris CLI: {error}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    match expected {
        Expected::Success(text) => {
            assert_eq!(output.status.code(), Some(0), "{stdout}\n{stderr}");
            assert_eq!(stdout, text, "stderr: {stderr}");
            assert!(stderr.is_empty(), "{stderr}");
        }
        Expected::WrongKind(text) => {
            assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
            assert_eq!(stdout, text, "stderr: {stderr}");
            assert!(stderr.contains("IRIS-DECORATOR-KIND"), "{stderr}");
            assert!(
                !stderr.to_ascii_lowercase().contains("unsupported"),
                "{stderr}"
            );
        }
    }
}

macro_rules! replay_cases {
    ($($name:ident: $fixture:literal => $expected:expr;)*) => {
        $(mod $name {
            use super::{Expected, check};

            const SOURCE: &str = include_str!(concat!("decorator_replay/", $fixture, ".iris"));

            #[test]
            fn parser_precheck() {
                let given = SOURCE;
                let when = iris_parser::parse(given);
                assert!(when.program_accepted, "{:?}", when.diagnostics);
                assert!(when.diagnostics.is_empty(), "{:?}", when.diagnostics);
            }

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

replay_cases! {
    meta_c138_fresh_chain_when_declarative_open_replaces_decorated_body:
        "decorated_open" => Expected::Success(concat!(
            "assemble_count\norigin\nassemble_trace\norigin\n1\n",
            "1\nenter_trace\nold_body\nexit_trace\n101\n",
            "2\nenter_trace\nold_body\nexit_trace\n102\n",
            "assemble_count\nopen\nassemble_trace\nopen\n2\ntrue\nfalse\n",
            "1\nenter_trace\nnew_body\nexit_trace\n201\n",
            "3\nenter_trace\nold_body\nexit_trace\n103\n",
            "2\nenter_trace\nnew_body\nexit_trace\n202\n"
        ));
    meta_c138_sibling_cache_persists_when_open_replaces_only_other_declaration:
        "undecorated_open" => Expected::Success(concat!(
            "assemble\nassemble\n1\n1\nold_body\n10\n1\nsibling_body\n30\n",
            "2\n20\n2\n10\n2\n30\n3\n30\n"
        ));
    meta_c138_captured_suffix_survives_when_slot_is_removed_then_undefined:
        "retained_suffix" => Expected::Success(
            "1\nold_body\n11\n2\nancestor_body\n7\nold_body\n12\n3\ntrue\nold_body\n13\n"
        );
    meta_c138_active_cache_survives_when_wrong_kind_aborts_open:
        "failed_open" => Expected::WrongKind(
            "1\nold_body\n11\ntrue\ntrue\ntrue\ntrue\ntrue\n12\n13\n14\nold_body\n25\n"
        );
}
