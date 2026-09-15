use super::{CompilationMode, Program};
use crate::MachineError;
use iris_syntax::{
    ClassDeclaration, Declaration, ExportDeclaration, Expression, MethodKind, Statement,
    TypeExpression,
};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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

pub(crate) fn compile_upgrade(source: &str, current: &Program) -> Result<Program, MachineError> {
    let parsed = iris_parser::parse(source);
    if !parsed.program_accepted || !parsed.program.statements.is_empty() {
        return Err(MachineError::UnsupportedConstruct);
    }
    let all_class_names = current
        .classes
        .iter()
        .map(|class| class.name.clone())
        .collect::<BTreeSet<_>>();
    let module_names = current
        .modules
        .iter()
        .map(|module| module.name.clone())
        .collect::<BTreeSet<_>>();
    let declarations = parsed
        .program
        .declarations
        .iter()
        .filter_map(unwrapped)
        .collect::<Vec<_>>();
    if parsed.program.declarations.iter().any(|declaration| {
        matches!(declaration, Declaration::Export(export) if matches!(export.as_ref(), ExportDeclaration::Names(_)))
    }) {
        return Err(MachineError::UnsupportedConstruct);
    }
    let mut candidate_classes = BTreeSet::new();
    let mut candidate_modules = BTreeSet::new();
    let mut helpers = BTreeSet::new();
    let mut expected_contracts = BTreeMap::new();
    let mut helper_contracts = BTreeMap::new();
    for declaration in &declarations {
        match declaration {
            Declaration::Class(class) if !class.reopen && supported(class, false) => {
                candidate_classes.insert(class.name.clone());
                for decorator in &class.decorators {
                    helpers.insert(decorator.name.clone());
                    if expected_contracts
                        .insert(decorator.name.clone(), "ClassDecorator")
                        .is_some_and(|known| known != "ClassDecorator")
                    {
                        return Err(MachineError::UnsupportedConstruct);
                    }
                }
                for method in class.body.iter().filter_map(|statement| match statement {
                    Statement::Method(method) => Some(method),
                    _ => None,
                }) {
                    let contract = match method.kind {
                        MethodKind::Property => "PropertyDecorator",
                        MethodKind::Instance | MethodKind::Class => "MethodDecorator",
                        MethodKind::Module => return Err(MachineError::UnsupportedConstruct),
                    };
                    for decorator in &method.decorators {
                        helpers.insert(decorator.name.clone());
                        if expected_contracts
                            .insert(decorator.name.clone(), contract)
                            .is_some_and(|known| known != contract)
                        {
                            return Err(MachineError::UnsupportedConstruct);
                        }
                    }
                }
                for decorator in class
                    .body
                    .iter()
                    .filter_map(|statement| match statement {
                        Statement::StoredProperty { decorators, .. } => Some(decorators.iter()),
                        _ => None,
                    })
                    .flatten()
                {
                    helpers.insert(decorator.name.clone());
                    if expected_contracts
                        .insert(decorator.name.clone(), "PropertyDecorator")
                        .is_some_and(|known| known != "PropertyDecorator")
                    {
                        return Err(MachineError::UnsupportedConstruct);
                    }
                }
            }
            Declaration::Module(module) if !module.reopen && modules::supported_upgrade(module) => {
                candidate_modules.insert(module.name.clone());
                for decorator in &module.decorators {
                    helpers.insert(decorator.name.clone());
                    if expected_contracts
                        .insert(decorator.name.clone(), "ModuleDecorator")
                        .is_some_and(|known| known != "ModuleDecorator")
                    {
                        return Err(MachineError::UnsupportedConstruct);
                    }
                }
                for decorator in module
                    .body
                    .iter()
                    .filter_map(|statement| match statement {
                        Statement::Method(method) => Some(method.decorators.iter()),
                        _ => None,
                    })
                    .flatten()
                {
                    helpers.insert(decorator.name.clone());
                    if expected_contracts
                        .insert(decorator.name.clone(), "MethodDecorator")
                        .is_some_and(|known| known != "MethodDecorator")
                    {
                        return Err(MachineError::UnsupportedConstruct);
                    }
                }
            }
            Declaration::Class(_) | Declaration::Module(_) => {
                return Err(MachineError::UnsupportedConstruct);
            }
            Declaration::Impl(_) => {}
            _ => return Err(MachineError::UnsupportedConstruct),
        }
    }
    let current_helpers = current
        .decorator_applications
        .iter()
        .map(|application| current.classes[application.decorator].name.clone())
        .collect::<BTreeSet<_>>();
    if candidate_classes != all_class_names
        || candidate_modules != module_names
        || helpers != current_helpers
        || candidate_classes.is_empty() && candidate_modules.is_empty()
    {
        return Err(MachineError::UnsupportedConstruct);
    }
    for declaration in &declarations {
        match declaration {
            Declaration::Class(class) if helpers.contains(&class.name) => {
                if !supported(class, true) {
                    return Err(MachineError::UnsupportedConstruct);
                }
            }
            Declaration::Class(_) | Declaration::Module(_) => {}
            Declaration::Impl(implementation) => {
                let (TypeExpression::Name(target), TypeExpression::Name(contract)) =
                    (&implementation.target, &implementation.contract)
                else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                if !helpers.contains(target)
                    || !matches!(
                        contract.as_str(),
                        "ClassDecorator"
                            | "MethodDecorator"
                            | "ModuleDecorator"
                            | "PropertyDecorator"
                    )
                    || expected_contracts.get(target) != Some(&contract.as_str())
                    || implementation.constraints.len() > 0
                    || !implementation.methods.iter().all(|method| {
                        method.kind == MethodKind::Instance
                            && !method.is_async
                            && method.type_parameters.is_empty()
                            && method.impl_contract.is_none()
                            && method.body.is_some()
                    })
                {
                    return Err(MachineError::UnsupportedConstruct);
                }
                let plan = implementation
                    .methods
                    .iter()
                    .find(|method| method.selector == "plan")
                    .ok_or(MachineError::UnsupportedConstruct)?;
                if !canonical_empty_plan(plan) {
                    return Err(MachineError::UnsupportedConstruct);
                }
                if helper_contracts
                    .insert(target.clone(), contract.clone())
                    .is_some()
                {
                    return Err(MachineError::UnsupportedConstruct);
                }
            }
            _ => {}
        }
    }
    if helpers.iter().any(|helper| {
        !candidate_classes.contains(helper)
            || !all_class_names.contains(helper)
            || !helper_contracts.contains_key(helper)
    }) {
        return Err(MachineError::UnsupportedConstruct);
    }
    let mut program = super::compile_parsed(
        (
            source,
            &iris_native_host::NativeRegistry::new(),
            CompilationMode::Package,
        ),
        parsed,
    )
    .map_err(|_| MachineError::UnsupportedConstruct)?;
    if program
        .instructions
        .iter()
        .chain(
            program
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter()),
        )
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
    for module in &mut program.modules {
        if module_names.contains(&module.name) {
            let functions = program
                .functions
                .iter_mut()
                .enumerate()
                .filter_map(|(index, function)| {
                    let (owner, _) = function.name.split_once('.')?;
                    (owner == module.name).then(|| {
                        if let Some(signature) = &mut function.signature {
                            signature.is_override = true;
                        }
                        index
                    })
                })
                .collect();
            module.replay = Some(crate::compile::ir::ModuleArtifact {
                reopen: true,
                functions,
            });
        }
    }
    Ok(program)
}

fn canonical_empty_plan(method: &iris_syntax::MethodDeclaration) -> bool {
    matches!(method.body.as_deref(), Some([Statement::Expression(Expression::Member { receiver, selector })])
        if selector == "empty" && matches!(receiver.as_ref(), Expression::Name(name) if name == "Plan"))
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
