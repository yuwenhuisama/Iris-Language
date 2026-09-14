use iris_syntax::{Declaration, Program, Statement};

use super::super::CompileError;
use super::super::declarations::Signature;
use super::super::ir::{ModuleArtifact, ModuleDeclaration};

pub(in crate::compile) fn prepare_module_replays(
    program: &Program,
    signatures: &mut [Signature<'_>],
    modules: &mut Vec<ModuleDeclaration>,
) -> Result<(), CompileError> {
    let origins = modules.len();
    for index in 0..origins {
        let name = modules[index].name.clone();
        let declarations = program
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Module(module) if module.name == name => Some(module),
                _ => None,
            })
            .collect::<Vec<_>>();
        let decorated = declarations.iter().any(|module| {
            !module.decorators.is_empty()
                || module.body.iter().any(|statement| {
                    matches!(statement,
                Statement::Method(method) if !method.decorators.is_empty())
                })
        });
        if !decorated || !declarations.iter().any(|module| module.reopen) {
            continue;
        }
        if declarations
            .iter()
            .any(|module| !module.parameters.is_empty() || !module.constraints.is_empty())
        {
            return Err(CompileError::new("generic Module decorators"));
        }
        if declarations.iter().filter(|module| !module.reopen).count() != 1 {
            return Err(CompileError::new("decorated Module redeclaration"));
        }
        for module in declarations {
            let functions = signatures
                .iter_mut()
                .enumerate()
                .filter_map(|(function, signature)| {
                    let belongs = signature.module == name
                        && signature.declaration.is_some_and(|method| {
                            module.body.iter().any(|statement| {
                                matches!(statement,
                        Statement::Method(known) if std::ptr::eq(known, method))
                            })
                        });
                    if belongs {
                        signature.receiver = true;
                    }
                    (belongs
                        || (!module.reopen
                            && signature.module == name
                            && signature.declaration.is_none()))
                    .then_some(function)
                })
                .collect();
            let replay = Some(ModuleArtifact {
                reopen: module.reopen,
                functions,
            });
            if module.reopen {
                let mixins = module
                    .mixins
                    .iter()
                    .map(|mixin| match &mixin.target {
                        iris_syntax::TypeExpression::Name(name) => Ok(name.clone()),
                        _ => Err(CompileError::new("module reopen composition")),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                modules.push(ModuleDeclaration {
                    name: name.clone(),
                    meta_deny: module.meta_deny.clone(),
                    mixins,
                    replay,
                });
            } else {
                modules[index].replay = replay;
            }
        }
    }
    Ok(())
}
