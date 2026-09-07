use iris_parser::parse;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn accepts_many_statements_when_each_expression_is_small() {
    let source = "1\n".repeat(2000);
    let result = parse(&source);
    assert!(result.program_accepted, "{:?}", result.diagnostics);
    assert_eq!(result.program.statements.len(), 2000);
}

#[test]
fn rejects_hostile_source_within_a_bounded_process() -> std::io::Result<()> {
    if let Ok(case) = std::env::var("IRIS_PARSER_PROGRESS_CASE") {
        let source = match case.as_str() {
            "brace" => "}".repeat(1000),
            "unary" => format!("{}1", "!".repeat(10000)),
            "group" => format!("{}1{}", "(".repeat(1000), ")".repeat(1000)),
            "body" => format!("{}1{}", "fun f() {".repeat(1000), "}".repeat(1000)),
            "type" => format!("let x: {}A{} = nil", "Box<".repeat(1000), ">".repeat(1000)),
            "chain" => format!("1{}", " + 1".repeat(10000)),
            "postfix" => format!("x{}", ".x".repeat(10000)),
            _ => unreachable!(),
        };
        let result = parse(&source);
        assert!(!result.program_accepted);
        assert!(!result.diagnostics.is_empty());
        return Ok(());
    }
    for case in [
        "brace", "unary", "group", "body", "type", "chain", "postfix",
    ] {
        let mut child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "rejects_hostile_source_within_a_bounded_process",
                "--nocapture",
            ])
            .env("IRIS_PARSER_PROGRESS_CASE", case)
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
        assert!(status.success(), "{case}: {status}");
    }
    Ok(())
}
