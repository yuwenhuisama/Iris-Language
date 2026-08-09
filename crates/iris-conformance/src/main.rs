use std::process::ExitCode;

fn main() -> ExitCode {
    let chapter = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [flag, chapter] if flag == "--chapter" && chapter == "GRAMMAR" => {
            iris_conformance::Chapter::Grammar
        }
        [flag, chapter] if flag == "--chapter" && chapter == "RUNTIME" => {
            iris_conformance::Chapter::Runtime
        }
        [flag, chapter] if flag == "--chapter" && chapter == "CONTROL" => {
            iris_conformance::Chapter::Control
        }
        [flag, chapter] if flag == "--chapter" && chapter == "TYPES" => {
            iris_conformance::Chapter::Types
        }
        [flag, chapter] if flag == "--chapter" && chapter == "META" => {
            iris_conformance::Chapter::Meta
        }
        [flag, chapter] if flag == "--chapter" && chapter == "ASYNC" => {
            iris_conformance::Chapter::Async
        }
        [flag, chapter] if flag == "--chapter" && chapter == "COLLECTIONS" => {
            iris_conformance::Chapter::Collections
        }
        [flag, chapter] if flag == "--chapter" && chapter == "FFI" => {
            iris_conformance::Chapter::Ffi
        }
        [flag, chapter] if flag == "--chapter" && chapter == "IDENTITY" => {
            iris_conformance::Chapter::Identity
        }
        _ => {
            eprintln!(
                "usage: iris-conformance --chapter GRAMMAR|RUNTIME|CONTROL|TYPES|META|ASYNC|COLLECTIONS|FFI|IDENTITY"
            );
            return ExitCode::from(2);
        }
    };
    match iris_conformance::Corpus::workspace()
        .and_then(|corpus| corpus.records_for(chapter))
        .map(|records| match chapter {
            iris_conformance::Chapter::Grammar => iris_conformance::execute(&records),
            // CONTROL vectors observe values, errors and diagnostics exactly as
            // RUNTIME ones do, so they share the runtime execution path.
            // TYPES vectors observe values, errors and diagnostics exactly as
            // RUNTIME and CONTROL ones do, so they share the same path.
            // META vectors observe a package load's result the same way, so
            // they share the runtime execution path too.
            iris_conformance::Chapter::Runtime
            | iris_conformance::Chapter::Control
            | iris_conformance::Chapter::Types
            // ASYNC vectors observe values and diagnostics the same way, so
            // they share the runtime execution path too.
            | iris_conformance::Chapter::Meta
            | iris_conformance::Chapter::Async
            | iris_conformance::Chapter::Collections
                | iris_conformance::Chapter::Ffi
                | iris_conformance::Chapter::Identity => iris_conformance::execute_runtime(&records),
        }) {
        Ok(outcomes) => {
            let report = iris_conformance::report(&outcomes);
            match chapter {
                iris_conformance::Chapter::Grammar => println!(
                    "passed: {}, failed: {}, deferred: {}, authored_expect: {}, unrunnable_source: {}",
                    report.passed,
                    report.failed,
                    report.deferred,
                    report.authored_expect,
                    report.unrunnable_source
                ),
                iris_conformance::Chapter::Runtime
                | iris_conformance::Chapter::Control
                | iris_conformance::Chapter::Types
                | iris_conformance::Chapter::Meta
                | iris_conformance::Chapter::Async
                | iris_conformance::Chapter::Collections
                | iris_conformance::Chapter::Ffi
                | iris_conformance::Chapter::Identity => {
                    println!(
                        "passed: {}, failed: {}, needs_subsystem: {}, no_fixture: {}, differential: {}",
                        report.passed,
                        report.failed,
                        report.needs_subsystem,
                        report.no_fixture,
                        report.differential
                    )
                }
            }
            for outcome in outcomes {
                match outcome {
                    iris_conformance::Outcome::Failed {
                        id,
                        expected,
                        actual,
                    } => println!("failed: {id}\n  expected: {expected}\n  actual: {actual}"),
                    iris_conformance::Outcome::UnrunnableSource { id } => {
                        println!("unrunnable_source: {id}")
                    }
                    iris_conformance::Outcome::NeedsSubsystem { id } => {
                        println!("needs_subsystem: {id}")
                    }
                    iris_conformance::Outcome::NoFixture { id } => println!("no_fixture: {id}"),
                    iris_conformance::Outcome::Differential { id } => {
                        println!("differential: {id}")
                    }
                    iris_conformance::Outcome::Passed { .. }
                    | iris_conformance::Outcome::Deferred { .. }
                    | iris_conformance::Outcome::AuthoredExpect { .. } => {}
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("conformance error: {error}");
            ExitCode::from(2)
        }
    }
}
