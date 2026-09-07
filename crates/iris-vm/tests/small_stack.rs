use iris_runtime::{ArrayRef, Value};
use std::{fmt::Write, process::Command, thread};

const CHILD_SCENARIO: &str = "IRIS_VM_SMALL_STACK_CHILD";

enum Scenario {
    NestedCalls,
    CatchFinally,
}

fn assert_child_success(scenario: &str) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;

    let output = Command::new(executable)
        .args(["--exact", "small_stack_child", "--nocapture"])
        .env(CHILD_SCENARIO, scenario)
        .output()
        .map_err(|error| error.to_string())?;

    assert!(
        output.status.success(),
        "{scenario} child exited with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    Ok(())
}

#[test]
fn nested_module_calls_return_42_when_stack_is_one_mib() -> Result<(), String> {
    assert_child_success("nested_calls")
}

#[test]
fn catch_return_runs_finally_when_stack_is_one_mib() -> Result<(), String> {
    assert_child_success("catch_finally")
}

#[test]
fn small_stack_child() -> Result<(), String> {
    let scenario = match std::env::var(CHILD_SCENARIO) {
        Ok(marker) => match marker.as_str() {
            "nested_calls" => Scenario::NestedCalls,
            "catch_finally" => Scenario::CatchFinally,
            _ => return Err(format!("unknown child scenario: {marker}")),
        },
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(error @ std::env::VarError::NotUnicode(_)) => return Err(error.to_string()),
    };
    let source = match scenario {
        Scenario::NestedCalls => {
            let mut source = String::from("module Main {\n");
            for depth in 0..15 {
                writeln!(
                    source,
                    "public module fun call{depth}() -> Integer {{ Main.call{}() }}",
                    depth + 1,
                )
                .map_err(|error| error.to_string())?;
            }
            source.push_str("public module fun call15() -> Integer { 42 }\n}\nMain.call0()");
            source
        }
        Scenario::CatchFinally => r#"
            module Main {
                public module fun run(log) -> Integer {
                    try {
                        try { raise :caught } catch error {
                            log.append(error)
                            return 42
                        }
                    } finally {
                        log.append(:closed)
                    }
                    0
                }
            }
            let log = []
            let result = Main.run(log)
            let recorded = log.append(result)
            log
        "#
        .to_owned(),
    };
    let program = iris_vm::compile(&source).map_err(|error| error.construct)?;
    iris_vm::verify(&program).map_err(|error| format!("{error:?}"))?;

    let worker = thread::Builder::new()
        .name("iris-vm-small-stack".to_owned())
        .stack_size(1024 * 1024)
        .spawn(move || {
            let result = iris_vm::run(&program);

            let expected = match scenario {
                Scenario::NestedCalls => Value::Integer(42_u64.into()),
                Scenario::CatchFinally => Value::Array(ArrayRef::new(vec![
                    Value::Symbol("caught".to_owned()),
                    Value::Symbol("closed".to_owned()),
                    Value::Integer(42_u64.into()),
                ])),
            };
            assert_eq!(result, Ok(expected));
        })
        .map_err(|error| error.to_string())?;

    worker
        .join()
        .map_err(|_| "small-stack worker panicked".to_owned())
}
