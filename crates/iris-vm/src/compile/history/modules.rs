use super::{MachineError, Program, unwrapped};
use iris_syntax::{Declaration, MethodKind, ModuleDeclaration, Statement};

pub(super) fn selected_module<'a>(
    declarations: &[&'a Declaration],
    name: &str,
) -> Result<&'a ModuleDeclaration, MachineError> {
    let mut matching = declarations
        .iter()
        .copied()
        .filter(|declaration| match declaration {
            Declaration::Module(target) => target.name == name,
            Declaration::Class(target) => target.name == name,
            _ => false,
        });
    let Some(Declaration::Module(target)) = matching.next() else {
        return Err(MachineError::RevisionArtifactUnavailable);
    };
    if matching.next().is_some() || target.reopen {
        return Err(MachineError::RevisionArtifactUnavailable);
    }
    Ok(target)
}

pub(super) fn supported(module: &ModuleDeclaration) -> bool {
    module.parameters.is_empty() && module.constraints.is_empty()
        && module.contract_for.is_empty() && module.mixins.is_empty() && module.decorators.is_empty()
        && module.body.iter().all(|statement| matches!(statement,
            Statement::Method(method) if method.type_parameters.is_empty() && method.impl_contract.is_none()
                && method.body.is_some() && matches!(method.kind, MethodKind::Instance | MethodKind::Module)))
}

pub(crate) fn validate_module_source(source: &str, name: &str) -> Result<(), MachineError> {
    let parsed = iris_parser::parse(source);
    if !parsed.program_accepted {
        return Err(MachineError::RevisionArtifactUnavailable);
    }
    let mut origins = 0;
    for declaration in parsed.program.declarations.iter().filter_map(unwrapped) {
        if let Declaration::Module(module) = declaration
            && module.name == name
        {
            if !supported(module) {
                return Err(MachineError::UnsupportedConstruct);
            }
            origins += usize::from(!module.reopen);
        }
    }
    if origins != 1 {
        return Err(MachineError::RevisionArtifactUnavailable);
    }
    Ok(())
}

pub(crate) fn compile_module(source: &str, name: &str) -> Result<Program, MachineError> {
    let mut program = super::compile_target(source, name, true)?;
    let functions = program
        .functions
        .iter_mut()
        .enumerate()
        .filter_map(|(index, function)| {
            let (owner, _) = function.name.split_once('.')?;
            if owner != name {
                return None;
            }
            if let Some(signature) = &mut function.signature {
                signature.is_override = true;
            }
            Some(index)
        })
        .collect();
    let target = program
        .modules
        .iter_mut()
        .find(|module| module.name == name)
        .ok_or(MachineError::RevisionArtifactUnavailable)?;
    target.replay = Some(crate::compile::ir::ModuleArtifact {
        reopen: true,
        functions,
    });
    Ok(program)
}
