use std::process::Command;

#[test]
fn uncaught_core_errors_report_nominal_codes_when_reference_runtime_exits() {
    for (given, expected) in [
        ("\"value\".split(1)", "TypeError"),
        (
            "module M { public async fun run() -> Object { await 7 } } Host.run(M.run())",
            "TypeError",
        ),
        (
            "try { let callback = { |value| value }; callback.call() } catch error: Symbol { error }",
            "ArgumentError",
        ),
        ("raise :ArgumentError", "Raised(Symbol(\"ArgumentError\"))"),
    ] {
        let when = Command::new(env!("CARGO_BIN_EXE_iris"))
            .args(["-e", given])
            .output()
            .unwrap();

        assert!(!when.status.success());
        assert_eq!(
            String::from_utf8(when.stderr).unwrap(),
            format!("iris: -e: {expected}\n")
        );
    }
}

#[test]
fn deferred_binding_static_diagnostics_exit_before_either_engine_runs() {
    for (given, expected) in [
        ("let value: Integer", "BINDING_LET_REQUIRES_INITIALIZER"),
        ("mut value", "BINDING_MISSING_TYPE_FOR_DEFERRED_INIT"),
    ] {
        for flags in [&["-e", given][..], &["--vm", "-e", given][..]] {
            let when = Command::new(env!("CARGO_BIN_EXE_iris"))
                .args(flags)
                .output()
                .unwrap();

            assert!(!when.status.success());
            assert_eq!(
                String::from_utf8(when.stderr).unwrap(),
                format!("iris: -e: {expected}\n"),
                "{flags:?}"
            );
        }
    }
}
