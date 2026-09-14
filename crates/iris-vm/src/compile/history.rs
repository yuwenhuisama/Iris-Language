use super::{CompilationMode, Program};
use crate::MachineError;
use iris_syntax::{ClassDeclaration, Declaration, ExportDeclaration, Statement, TypeExpression};

mod modules;
pub(crate) use modules::{compile_module, validate_module_source};

fn unwrapped(declaration: &Declaration) -> Option<&Declaration> {
    match declaration {
        Declaration::Export(export) => match export.as_ref() {
            ExportDeclaration::Declaration(inner) => unwrapped(inner),
            ExportDeclaration::Names(_) => None,
        },
        declaration => Some(declaration),
    }
}

fn selected_class<'a>(
    declarations: &[&'a Declaration],
    name: &str,
) -> Result<&'a ClassDeclaration, MachineError> {
    let mut matching = declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Class(class) if class.name == name => Some(class),
            _ => None,
        });
    let class = matching
        .next()
        .ok_or(MachineError::RevisionArtifactUnavailable)?;
    if matching.next().is_some() || class.reopen {
        return Err(MachineError::RevisionArtifactUnavailable);
    }
    Ok(class)
}

fn supported(class: &ClassDeclaration, decorator: bool) -> bool {
    class.parameters.is_empty() && class.constraints.is_empty()
        && class.extends.is_none() && class.mixins.is_empty()
        && (!decorator || class.decorators.is_empty())
        && class.body.iter().all(|statement| matches!(statement,
            Statement::Method(method) if (method.kind == iris_syntax::MethodKind::Instance
                || (!decorator && matches!(method.kind, iris_syntax::MethodKind::Property | iris_syntax::MethodKind::Class)))
                && method.type_parameters.is_empty() && method.body.is_some()
                && !method.impl_contract.as_ref().is_some_and(Option::is_some)
                && (!decorator || method.decorators.is_empty())))
}

pub(crate) fn compile(source: &str, name: &str) -> Result<Program, MachineError> {
    compile_target(source, name, false)
}

fn compile_target(source: &str, name: &str, module: bool) -> Result<Program, MachineError> {
    let mut parsed = iris_parser::parse(source);
    if !parsed.program_accepted {
        return Err(MachineError::RevisionArtifactUnavailable);
    }
    let declarations = parsed
        .program
        .declarations
        .iter()
        .filter_map(unwrapped)
        .collect::<Vec<_>>();
    let (decorators, body) = if module {
        let target = modules::selected_module(&declarations, name)?;
        if !modules::supported(target) {
            return Err(MachineError::UnsupportedConstruct);
        }
        (&target.decorators, &target.body)
    } else {
        let target = selected_class(&declarations, name)?;
        if !supported(target, false) {
            return Err(MachineError::UnsupportedConstruct);
        }
        (&target.decorators, &target.body)
    };
    let mut names = if module {
        Vec::new()
    } else {
        vec![name.to_owned()]
    };
    for application in decorators.iter().chain(
        body.iter()
            .filter_map(|statement| match statement {
                Statement::Method(method) => Some(method.decorators.iter()),
                _ => None,
            })
            .flatten(),
    ) {
        if application.name == name {
            return Err(MachineError::UnsupportedConstruct);
        }
        if !names.contains(&application.name) {
            let decorator = selected_class(&declarations, &application.name)?;
            if !supported(decorator, true) {
                return Err(MachineError::UnsupportedConstruct);
            }
            names.push(application.name.clone());
        }
    }
    let mut selected = Vec::new();
    let mut contracts = Vec::new();
    for declaration in &declarations {
        if module && matches!(declaration, Declaration::Module(target) if target.name == name) {
            selected.push((*declaration).clone());
        }
        if let Declaration::Class(class) = declaration
            && names.contains(&class.name)
        {
            for contract in &class.implements {
                let TypeExpression::Name(name) = contract else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                if !contracts.contains(name) {
                    contracts.push(name.clone());
                }
            }
            selected.push((*declaration).clone());
        }
    }
    for declaration in &declarations {
        let Declaration::Impl(implementation) = declaration else {
            continue;
        };
        let TypeExpression::Name(target) = &implementation.target else {
            return Err(MachineError::UnsupportedConstruct);
        };
        if !names.contains(target) {
            continue;
        }
        let TypeExpression::Name(contract) = &implementation.contract else {
            return Err(MachineError::UnsupportedConstruct);
        };
        if !contracts.contains(contract) {
            contracts.push(contract.clone());
        }
        selected.push((*declaration).clone());
    }
    for name in contracts {
        if iris_runtime::core_contract_id(&name).is_some() {
            continue;
        }
        let mut matching = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Contract(contract) if contract.name == name => Some(contract),
                _ => None,
            });
        let contract = matching
            .next()
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        if matching.next().is_some()
            || contract.open
            || !contract.decorators.is_empty()
            || !contract.parents.is_empty()
            || !contract.parameters.is_empty()
            || !contract.constraints.is_empty()
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        selected.insert(0, Declaration::Contract(contract.clone()));
    }
    parsed.program = iris_syntax::Program {
        entries: selected
            .iter()
            .cloned()
            .map(iris_syntax::ProgramEntry::Declaration)
            .collect(),
        declarations: selected,
        statements: Vec::new(),
    };
    let program = super::compile_parsed(
        (
            source,
            &iris_native_host::NativeRegistry::new(),
            CompilationMode::Package,
        ),
        parsed,
    )
    .map_err(|_| MachineError::UnsupportedConstruct)?;
    if program
        .classes
        .iter()
        .any(|class| class.contract_signature_clash)
        || (!module && program.classes.iter().all(|class| class.name != name))
    {
        return Err(MachineError::TypeContractError);
    }
    if program
        .functions
        .iter()
        .flat_map(|function| &function.instructions)
        .any(|instruction| {
            matches!(
                instruction,
                super::Instruction::NativeCall { .. }
                    | super::Instruction::NativeFixture { .. }
                    | super::Instruction::FfiOpen { .. }
            )
        })
    {
        return Err(MachineError::UnsupportedConstruct);
    }
    Ok(program)
}
