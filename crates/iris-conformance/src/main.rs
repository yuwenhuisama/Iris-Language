use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::args().skip(1).collect::<Vec<_>>().as_slice() != ["--chapter", "GRAMMAR"] {
        eprintln!("usage: iris-conformance --chapter GRAMMAR");
        return ExitCode::from(2);
    }
    match iris_conformance::Corpus::workspace()
        .and_then(|corpus| corpus.records())
        .map(|records| iris_conformance::execute(&records))
    {
        Ok(outcomes) => {
            let report = iris_conformance::report(&outcomes);
            println!(
                "passed: {}, failed: {}, deferred: {}, authored_expect: {}, unrunnable_source: {}",
                report.passed,
                report.failed,
                report.deferred,
                report.authored_expect,
                report.unrunnable_source
            );
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
