//! The interactive session.

use std::io::{BufRead, Write};
use std::process::ExitCode;

use iris_eval::EvaluationError;

/// Runs an interactive session until end of input.
pub fn run() -> ExitCode {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    println!("iris {} - type :quit to leave", env!("CARGO_PKG_VERSION"));

    // ONE runtime for the whole session. Re-running earlier lines instead
    // repeats their side effects, and loses any mutation made by a line that is
    // not itself a binding: `account.deposit(30)` would never be re-applied, so
    // a later `account.balance` answered the pre-mutation value.
    let Ok(mut session) = iris_eval::Session::new() else {
        eprintln!("iris: could not start a session");
        return ExitCode::FAILURE;
    };

    loop {
        if write!(stdout, "iris> ").is_err() || stdout.flush().is_err() {
            return ExitCode::FAILURE;
        }
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            // End of input: leave the way a shell does.
            Ok(0) => {
                println!();
                return ExitCode::SUCCESS;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("iris: {error}");
                return ExitCode::FAILURE;
            }
        }
        let entry = line.trim();
        if entry.is_empty() {
            continue;
        }
        if entry == ":quit" || entry == ":q" {
            return ExitCode::SUCCESS;
        }

        // A chunk that ENDS in a binding has no value and answers
        // UnsupportedConstruct, so a trailing `nil` gives it one. Entering
        // `let a = 5` alone otherwise reported an error and left nothing bound.
        let session_state = is_session_state(entry);
        let chunk = if session_state {
            format!("{entry}\nnil")
        } else {
            entry.to_owned()
        };
        match session.evaluate(&chunk) {
            Ok(value) => {
                // A binding or declaration has no useful value to show.
                if !session_state {
                    println!("{}", crate::render::render(&value));
                }
            }
            Err(error) => println!("error: {}", describe(&error)),
        }
    }
}

/// Whether `entry` contributes SESSION STATE rather than an answer.
///
/// Such a line is replayed ahead of later lines so its binding stays visible.
/// Anything else is evaluated once and never replayed, so its side effects
/// happen exactly once.
fn is_session_state(entry: &str) -> bool {
    let parsed = iris_parser::parse(entry);
    if !parsed.program_accepted {
        return false;
    }
    if !parsed.program.declarations.is_empty() {
        return true;
    }
    parsed.program.statements.iter().any(|statement| {
        matches!(
            statement,
            iris_syntax::Statement::Binding { .. }
                | iris_syntax::Statement::GlobalBinding { .. }
                | iris_syntax::Statement::SharedBinding { .. }
                | iris_syntax::Statement::DeferredBinding { .. }
                | iris_syntax::Statement::StoredProperty { .. }
                | iris_syntax::Statement::Method(_)
        )
    })
}

/// Describes an evaluation failure for a human reader.
pub fn describe(error: &EvaluationError) -> String {
    match error {
        EvaluationError::MessageNotFound {
            receiver_class,
            selector,
        } => format!("{receiver_class} has no method `{selector}`"),
        EvaluationError::ParseDiagnostic => "could not parse that".to_owned(),
        EvaluationError::NameError => "unknown name".to_owned(),
        EvaluationError::UnsupportedConstruct => "unsupported construct".to_owned(),
        other => format!("{other:?}"),
    }
}
