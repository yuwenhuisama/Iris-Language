mod args;
mod sources;
#[cfg(test)]
mod tests;

use crate::Engine;
use iris_native_host::{NativePolicy, NativeRegistry, TrustedModule};
use iris_package::{BuildOptions, Grants, InstallOptions};
use std::{path::Path, process::ExitCode, rc::Rc};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    Package(#[from] iris_package::PackageError),
    #[error("{0}")]
    Native(#[from] iris_native_host::NativeError),
    #[error("{0}")]
    Arguments(&'static str),
    #[error("invalid permission: {0}")]
    Permission(String),
    #[error("cannot prepare {path}: {source}")]
    Source {
        path: String,
        source: std::string::FromUtf8Error,
    },
    #[error("package import denied or invalid in {0}")]
    Import(String),
    #[error("invalid SHA256 hex digest")]
    Digest,
    #[error("cannot read {}: {source}; run iris package install first", path.display())]
    Lock {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("the machine does not cover {0}")]
    Compile(String),
    #[error("TypeContractError: {0}")]
    StaticDiagnostic(&'static str),
    #[error("machine defect: {0}")]
    Verify(String),
    #[error("machine execution failed: {0}")]
    Machine(String),
    #[error("{0}")]
    Evaluation(String),
}

pub(super) fn run(arguments: &[String]) -> ExitCode {
    if matches!(arguments, [flag] if flag == "--help" || flag == "-h") {
        println!("{}", crate::USAGE);
        return ExitCode::SUCCESS;
    }
    match args::parse(arguments).and_then(execute) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("iris: package: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(command: args::Command<'_>) -> Result<(), Error> {
    match command {
        args::Command::Install {
            root,
            allow_local_git,
        } => {
            let locked = iris_package::install(root, InstallOptions { allow_local_git })?;
            println!(
                "Installed {} package(s); lock: {}",
                locked.packages.len(),
                root.join("iris.lock").display()
            );
            Ok(())
        }
        args::Command::Build {
            root,
            allow_native_build,
        } => {
            let receipts = iris_package::build(root, BuildOptions { allow_native_build })?;
            println!("Built {} native artifact(s)", receipts.artifacts.len());
            Ok(())
        }
        args::Command::Run {
            root,
            engine,
            grants,
        } => run_package(root, engine, &grants),
    }
}

fn run_package(root: &Path, engine: Engine, grants: &Grants) -> Result<(), Error> {
    let lock = root.join("iris.lock");
    std::fs::metadata(&lock).map_err(|source| Error::Lock { path: lock, source })?;
    let tree = iris_package::prepare(root, grants)?;
    let sources = sources::collect(&tree)?;
    if matches!(engine, Engine::Machine)
        && !sources.first().is_some_and(|first| {
            sources
                .iter()
                .all(|source| source.package_id == first.package_id)
        })
    {
        return Err(Error::Compile("unsupported VM package shape: expected source units from one package; cross-package execution is not implemented".to_owned()));
    }
    for source in &sources {
        source
            .validate_imports()
            .map_err(|_| Error::Import(source.path.clone()))?;
    }
    let registry = Rc::new(NativeRegistry::new());
    let policy = NativePolicy {
        allow_native: grants.permissions.contains("native.load"),
        permissions: grants.permissions.clone(),
        max_resources: 4096,
    };
    for package in &tree.packages {
        if let Some(native) = &package.native {
            registry.load(
                TrustedModule {
                    path: &native.artifact,
                    metadata: &native.metadata,
                    metadata_sha256: sources::digest(native.metadata_sha256.as_str())?,
                    artifact_sha256: sources::digest(native.artifact_sha256.as_str())?,
                },
                &policy,
            )?;
        }
    }
    match engine {
        Engine::Reference => {
            iris_eval::evaluate_package_tree_with_natives(&sources, registry)
                .map_err(|error| Error::Evaluation(crate::repl::describe(&error)))?;
        }
        Engine::Machine => {
            let program = iris_vm::compile_package_tree_with_natives(&sources, &registry).map_err(
                |error| match error.kind {
                    iris_vm::CompileErrorKind::UnsupportedConstruct => {
                        Error::Compile(error.construct)
                    }
                    iris_vm::CompileErrorKind::StaticDiagnostic { code } => {
                        Error::StaticDiagnostic(code)
                    }
                },
            )?;
            iris_vm::verify(&program).map_err(|error| Error::Verify(format!("{error:?}")))?;
            iris_vm::run_with_natives(&program, registry)
                .map_err(|error| Error::Machine(format!("{error:?}")))?;
        }
    }
    Ok(())
}
