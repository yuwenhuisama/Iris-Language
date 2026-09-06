use super::Error;
use crate::Engine;
use iris_package::Grants;
use std::path::Path;

pub(super) enum Command<'a> {
    Install {
        root: &'a Path,
        allow_local_git: bool,
    },
    Build {
        root: &'a Path,
        allow_native_build: bool,
    },
    Run {
        root: &'a Path,
        engine: Engine,
        grants: Grants,
    },
}

pub(super) fn parse(arguments: &[String]) -> Result<Command<'_>, Error> {
    let [operation, directory, flags @ ..] = arguments else {
        return Err(Error::Arguments(
            "expected package install/build/run <dir>; see iris --help",
        ));
    };
    if directory.starts_with('-') {
        return Err(Error::Arguments("expected package directory before flags"));
    }
    let root = Path::new(directory);
    match operation.as_str() {
        "install" => match flags {
            [] => Ok(Command::Install {
                root,
                allow_local_git: false,
            }),
            [flag] if flag == "--allow-local-git" => Ok(Command::Install {
                root,
                allow_local_git: true,
            }),
            _ => Err(Error::Arguments("install accepts only --allow-local-git")),
        },
        "build" => match flags {
            [] => Ok(Command::Build {
                root,
                allow_native_build: false,
            }),
            [flag] if flag == "--allow-native-build" => Ok(Command::Build {
                root,
                allow_native_build: true,
            }),
            _ => Err(Error::Arguments("build accepts only --allow-native-build")),
        },
        "run" => {
            let mut engine = Engine::Reference;
            let mut permissions = None;
            let mut remaining = flags.iter();
            while let Some(flag) = remaining.next() {
                match flag.as_str() {
                    "--vm" if matches!(engine, Engine::Reference) => engine = Engine::Machine,
                    "--allow" if permissions.is_none() => {
                        let value = remaining.next().ok_or(Error::Arguments(
                            "--allow requires comma-separated permissions",
                        ))?;
                        let mut parsed = std::collections::BTreeSet::new();
                        for permission in value.split(',') {
                            if !permission.split('.').all(|part| {
                                part.starts_with(|character: char| {
                                    character.is_ascii_alphabetic() || character == '_'
                                }) && part
                                    .bytes()
                                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                            }) {
                                return Err(Error::Permission(permission.to_owned()));
                            }
                            parsed.insert(permission.to_owned());
                        }
                        permissions = Some(parsed);
                    }
                    _ => {
                        return Err(Error::Arguments(
                            "run accepts only --vm and --allow <permissions>, each once",
                        ));
                    }
                }
            }
            Ok(Command::Run {
                root,
                engine,
                grants: Grants {
                    permissions: permissions.unwrap_or_default(),
                },
            })
        }
        _ => Err(Error::Arguments("expected package install, build, or run")),
    }
}
