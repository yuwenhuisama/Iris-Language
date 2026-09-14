use iris_parser::{parse, parse_editor, parse_with_source};
use iris_syntax::{Expression, Statement};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const CASES: &[(&str, bool)] = &[
    ("print(f() { :v })", true),
    (
        "class A { public fun m(&b) { b.call() } }; print(A.new().m() { :v }); print(try { A.new().m({ :v }); false } catch error { error is? ArgumentError })",
        true,
    ),
    ("print(f() { :v }", false),
    ("print(f() { :v", false),
    ("print(f() { :v }) }", false),
    ("print(f() { :v },)", true),
];

#[test]
fn terminates_when_trailing_blocks_are_nested_in_call_arguments() -> std::io::Result<()> {
    if let Ok(case) = std::env::var("IRIS_NESTED_BLOCK_CASE") {
        let index: usize = case
            .parse()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
        let (given, accepted) = CASES[index];

        let when = parse(given);
        let sourced = parse_with_source(given);
        let editor = parse_editor(given);

        for result in [&when, &sourced.parse, &editor.parse] {
            assert_eq!(result.program_accepted, accepted, "{given}: {result:?}");
            assert_eq!(result.diagnostics.is_empty(), accepted, "{given}");
        }
        if accepted {
            assert_eq!(when.program, sourced.parse.program);
            assert_eq!(when.program, editor.parse.program);
        }
        if index == 0 {
            let Statement::Expression(Expression::Call { arguments, .. }) =
                &when.program.statements[0]
            else {
                return Err(std::io::Error::other("outer call required"));
            };
            let [Expression::Call { arguments, .. }] = arguments.as_slice() else {
                return Err(std::io::Error::other(
                    "inner call must remain the outer positional argument",
                ));
            };
            let [Expression::BlockArgument { value }] = arguments.as_slice() else {
                return Err(std::io::Error::other(
                    "trailing closure must belong to the inner block channel",
                ));
            };
            assert!(matches!(value.as_ref(), Expression::Closure { body, .. }
                if body == &[Statement::Expression(Expression::Symbol("v".into()))]));
        }
        return Ok(());
    }

    for (index, (given, _)) in CASES.iter().enumerate() {
        let mut child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "terminates_when_trailing_blocks_are_nested_in_call_arguments",
                "--nocapture",
            ])
            .env("IRIS_NESTED_BLOCK_CASE", index.to_string())
            .stdout(Stdio::null())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill()?;
                break child.wait()?;
            }
            std::thread::yield_now();
        };
        assert!(status.success(), "{given}: {status}");
    }
    Ok(())
}
