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
        [flag, source] if flag == "-e" => run_source(source, "-e", Engine::Reference),
        [engine, flag, source] if engine == "--vm" && flag == "-e" => {
            run_source(source, "-e", Engine::Machine)
        }
        [flag, path] if flag == "--vm" => run_file(path, Engine::Machine),
        [path] if !path.starts_with('-') => run_file(path, Engine::Reference),
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
  iris --vm <file>     Run a script on the register machine
  iris --help          Show this message
  iris --version       Show the version";

/// Which engine runs the program.
///
/// The register machine is a SECOND implementation of the same language, and
/// it declines a construct it does not fully cover rather than approximating
/// one - so a program it refuses is reported as refused instead of being
/// silently rerouted, which would hide what the machine cannot yet do.
#[derive(Clone, Copy)]
enum Engine {
    Reference,
    Machine,
}

fn run_file(path: &str, engine: Engine) -> ExitCode {
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("iris: cannot read {path}");
        return ExitCode::FAILURE;
    };
    run_source(&source, path, engine)
}

/// Runs `source` and reports the result.
///
/// A script's value is NOT printed. A program communicates through what it
/// does, and echoing the last expression would put noise on stdout that a
/// caller then has to filter. The REPL prints values because that is its
/// purpose; a script runner does not.
fn run_source(source: &str, origin: &str, engine: Engine) -> ExitCode {
    match engine {
        Engine::Reference => match iris_eval::evaluate(source) {
            Ok(_) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("iris: {origin}: {}", repl::describe(&error));
                ExitCode::FAILURE
            }
        },
        Engine::Machine => run_on_machine(source, origin),
    }
}

/// Runs `source` on the register machine.
///
/// A construct the machine does not cover is reported as REFUSED rather than
/// rerouted to the reference: falling back silently would report success for a
/// program the machine never ran.
fn run_on_machine(source: &str, origin: &str) -> ExitCode {
    let program = match iris_vm::compile(source) {
        Ok(program) => program,
        Err(error) => {
            eprintln!(
                "iris: {origin}: the machine does not cover {}",
                error.construct
            );
            return ExitCode::FAILURE;
        }
    };
    // Verification proves every operand is in range and written before use, so
    // a machine defect is caught here rather than becoming a wrong answer.
    if let Err(error) = iris_vm::verify(&program) {
        eprintln!("iris: {origin}: machine defect: {error:?}");
        return ExitCode::FAILURE;
    }
    match iris_vm::run(&program) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("iris: {origin}: {error:?}");
            ExitCode::FAILURE
        }
    }
}
