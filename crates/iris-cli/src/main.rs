//! The `iris` command: runs a script file, or an interactive session.

mod render;
mod repl;

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => repl::run(),
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [flag] if flag == "--version" || flag == "-V" => {
            println!("iris {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [flag, source] if flag == "-e" => run_source(source, "-e"),
        [path] if !path.starts_with('-') => run_file(path),
        _ => {
            eprintln!("iris: unrecognized arguments\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "\
Usage:
  iris                 Start an interactive session
  iris <file.iris>     Run a script
  iris -e <source>     Run source given on the command line
  iris --help          Show this message
  iris --version       Show the version";

fn run_file(path: &str) -> ExitCode {
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("iris: cannot read {path}");
        return ExitCode::FAILURE;
    };
    run_source(&source, path)
}

/// Runs `source` and reports the result.
///
/// A script's value is NOT printed. A program communicates through what it
/// does, and echoing the last expression would put noise on stdout that a
/// caller then has to filter. The REPL prints values because that is its
/// purpose; a script runner does not.
fn run_source(source: &str, origin: &str) -> ExitCode {
    match iris_eval::evaluate(source) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("iris: {origin}: {}", repl::describe(&error));
            ExitCode::FAILURE
        }
    }
}
