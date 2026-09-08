use crate::{CompileError, Machine, MachineError, Program};
use iris_native_host::NativeRegistry;
use iris_runtime::Value;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageIdentity {
    pub package_id: String,
    pub api_major: u32,
    pub version: Option<String>,
}

impl Program {
    pub fn package_identity(&self) -> Option<&PackageIdentity> {
        self.package.as_ref()
    }

    pub(crate) fn package_id(&self) -> &str {
        self.package
            .as_ref()
            .map_or("runtime-local", |package| package.package_id.as_str())
    }

    pub(crate) fn api_major(&self) -> u64 {
        self.package
            .as_ref()
            .map_or(1, |package| u64::from(package.api_major))
    }
}

pub fn compile_with_native(
    source: &str,
    registry: &NativeRegistry,
) -> Result<Program, CompileError> {
    crate::compile_with_natives(source, registry)
}
pub fn run_with_natives(
    program: &Program,
    registry: Rc<NativeRegistry>,
) -> Result<Value, MachineError> {
    Machine::with_natives(registry)
        .map_err(MachineError::Kernel)?
        .execute(program)
}
pub fn compile_packages_with_natives(
    programs: &[(String, String)],
    registry: &NativeRegistry,
) -> Result<Program, CompileError> {
    for (_, source) in programs {
        if !iris_parser::parse(source).program_accepted {
            return Err(CompileError {
                kind: crate::CompileErrorKind::UnsupportedConstruct,
                construct: "invalid VM package source unit".to_owned(),
            });
        }
    }
    compile_units(
        programs.iter().map(|(package, source)| {
            (
                PackageIdentity {
                    package_id: package.clone(),
                    api_major: 1,
                    version: None,
                },
                source.as_str(),
            )
        }),
        registry,
    )
}

pub fn compile_package_tree_with_natives(
    sources: &[iris_native_host::PackageSource],
    registry: &NativeRegistry,
) -> Result<Program, CompileError> {
    iris_native_host::PackageSource::validate_metadata(sources).map_err(|_| CompileError {
        kind: crate::CompileErrorKind::UnsupportedConstruct,
        construct: "conflicting or invalid VM package metadata".to_owned(),
    })?;
    for source in sources {
        source.validate_imports().map_err(|_| CompileError {
            kind: crate::CompileErrorKind::UnsupportedConstruct,
            construct: "native package import permission".to_owned(),
        })?;
    }
    compile_units(
        sources.iter().map(|source| {
            (
                PackageIdentity {
                    package_id: source.package_id.clone(),
                    api_major: source.api_major,
                    version: Some(source.version.clone()),
                },
                source.source.as_str(),
            )
        }),
        registry,
    )
}

fn compile_units<'a>(
    mut sources: impl Iterator<Item = (PackageIdentity, &'a str)>,
    registry: &NativeRegistry,
) -> Result<Program, CompileError> {
    let Some((package, first)) = sources.next() else {
        return Err(CompileError {
            kind: crate::CompileErrorKind::UnsupportedConstruct,
            construct: "unsupported VM package shape: expected at least one source unit".to_owned(),
        });
    };
    let mut joined = first.to_owned();
    for (identity, source) in sources {
        if identity != package {
            return Err(CompileError {
                kind: crate::CompileErrorKind::UnsupportedConstruct,
                construct:
                    "unsupported VM package shape: cross-package execution is not implemented"
                        .to_owned(),
            });
        }
        joined.push('\n');
        joined.push_str(source);
    }
    let mut program = crate::compile::compile_in_mode(
        &joined,
        registry,
        crate::compile::CompilationMode::Package,
    )?;
    program.package = Some(package);
    Ok(program)
}
