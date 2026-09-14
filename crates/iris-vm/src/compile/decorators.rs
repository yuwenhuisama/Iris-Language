use iris_runtime::decorator_protocol::DecoratorKind;
use iris_syntax::{Declaration, Expression, Statement};

use super::lowering::{Declarations, Lowering, ProgramBinding};
use super::{CompileError, Function, Instruction};

mod exports;
mod modules;
mod targets;
pub(super) use exports::prepare_exports;
pub(super) use modules::prepare_module_replays;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    Class(usize),
    Reopen { class: usize, artifact: usize },
    Module(usize),
    Contract(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application {
    pub target: Target,
    pub method: Option<usize>,
    pub decorator: usize,
    pub arguments: usize,
    pub kind: DecoratorKind,
    pub property: Option<PropertyDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyDeclaration {
    pub name: String,
    pub annotation: iris_syntax::TypeExpression,
    pub visibility: iris_syntax::Visibility,
}

pub(super) fn lower_applications(
    program: &iris_syntax::Program,
    declarations: Declarations<'_, '_>,
    functions: &mut Vec<Function>,
    bindings: &[ProgramBinding],
) -> Result<Vec<Application>, CompileError> {
    let mut applications = Vec::new();
    let mut reopens = std::collections::BTreeMap::new();
    let mut module_ordinals = std::collections::BTreeMap::<String, usize>::new();
    for declaration in &program.declarations {
        let (target, reopen, mut targets) = match declaration {
            Declaration::Module(module) => {
                let decorated = !module.decorators.is_empty()
                    || module.body.iter().any(|statement| match statement {
                        Statement::Method(method) => !method.decorators.is_empty(),
                        Statement::StoredProperty { decorators, .. } => !decorators.is_empty(),
                        _ => false,
                    });
                if decorated && (!module.parameters.is_empty() || !module.constraints.is_empty()) {
                    return Err(CompileError::new("generic Module decorators"));
                }
                let ordinal = module_ordinals.entry(module.name.clone()).or_default();
                let index = declarations
                    .modules
                    .iter()
                    .enumerate()
                    .filter(|(_, known)| known.name == module.name)
                    .nth(*ordinal)
                    .map(|(index, _)| index);
                if declarations
                    .modules
                    .iter()
                    .any(|known| known.name == module.name && known.replay.is_some())
                {
                    *ordinal += 1;
                }
                if !decorated {
                    continue;
                }
                let index = index.ok_or_else(|| CompileError::new("module decorator target"))?;
                let mut targets = vec![(None, DecoratorKind::Module, &module.decorators)];
                for statement in &module.body {
                    if matches!(statement, Statement::StoredProperty { decorators, .. } if !decorators.is_empty())
                    {
                        return Err(CompileError::new("Module stored property decorators"));
                    }
                    if let Statement::Method(method) = statement {
                        if !method.decorators.is_empty()
                            && (method.kind != iris_syntax::MethodKind::Instance
                                || !method.type_parameters.is_empty()
                                || method.impl_contract.is_some())
                        {
                            return Err(CompileError::new("module decorator member kind"));
                        }
                        let function = declarations.signatures.iter().position(|signature| {
                            signature
                                .declaration
                                .is_some_and(|known| std::ptr::eq(known, method))
                                && signature.module == module.name
                        });
                        targets.push((function, DecoratorKind::Method, &method.decorators));
                    }
                }
                (Target::Module(index), false, targets)
            }
            Declaration::Contract(contract) => {
                let index = declarations
                    .contracts
                    .iter()
                    .position(|known| known.name == contract.name)
                    .ok_or_else(|| CompileError::new("contract parent unbound"))?;
                (
                    Target::Contract(index),
                    false,
                    vec![(None, DecoratorKind::Contract, &contract.decorators)],
                )
            }
            Declaration::Class(class) => {
                let Some(class_index) = declarations
                    .classes
                    .iter()
                    .position(|known| known.name == class.name)
                else {
                    continue;
                };
                let targets = targets::class_targets(class, declarations)?;
                let target = if class.reopen {
                    let artifact = reopens.entry(class_index).or_insert(0);
                    let target = Target::Reopen {
                        class: class_index,
                        artifact: *artifact,
                    };
                    *artifact += 1;
                    target
                } else {
                    Target::Class(class_index)
                };
                (target, false, targets)
            }
            Declaration::Impl(implementation) => {
                let (iris_syntax::TypeExpression::Name(owner)
                | iris_syntax::TypeExpression::Generic { name: owner, .. }) =
                    &implementation.target
                else {
                    return Err(CompileError::new("static impl"));
                };
                let class = declarations
                    .classes
                    .iter()
                    .position(|class| class.name == *owner)
                    .ok_or_else(|| CompileError::new("static impl"))?;
                let generic = !declarations.classes[class].type_parameters.is_empty();
                let targets = implementation
                    .methods
                    .iter()
                    .map(|method| targets::method_target(method, (owner, generic), declarations))
                    .collect::<Result<Vec<_>, _>>()?;
                (Target::Class(class), false, targets)
            }
            _ => continue,
        };
        for (method, kind, decorators) in targets.drain(..) {
            for decorator in decorators {
                if reopen {
                    return Err(CompileError::new("decorator replay"));
                }
                let decorator_index = declarations
                    .classes
                    .iter()
                    .position(|known| known.name == decorator.name)
                    .ok_or_else(|| CompileError::diagnostic("IRIS-DECORATOR-KIND"))?;
                let contract = format!(
                    "{}Decorator",
                    match kind {
                        DecoratorKind::Class => "Class",
                        DecoratorKind::Method => "Method",
                        DecoratorKind::Property => "Property",
                        DecoratorKind::Module => "Module",
                        DecoratorKind::Contract => "Contract",
                    }
                );
                if !declarations.classes[decorator_index]
                    .contracts
                    .iter()
                    .any(|index| declarations.contracts[*index].name == contract)
                {
                    return Err(CompileError::diagnostic("IRIS-DECORATOR-KIND"));
                }
                let mut nested = Vec::new();
                let mut lowering =
                    Lowering::new(declarations, functions.len(), &mut nested, bindings, false);
                let value = lowering.expression(&Expression::Array(decorator.arguments.clone()))?;
                lowering.instructions.push(Instruction::Return { value });
                let function = Function {
                    body_entry: 0,
                    signature: None,
                    name: "<decorator-arguments>".into(),
                    parameters: 0,
                    captures: 0,
                    fixed_arity: Some(0),
                    parameter_types: Vec::new(),
                    return_type: "Array".into(),
                    is_async: false,
                    registers: usize::from(lowering.next_register),
                    instructions: std::mem::take(&mut lowering.instructions),
                };
                drop(lowering);
                functions.extend(nested);
                let arguments = functions.len();
                functions.push(function);
                applications.push(Application {
                    property: match declaration {
                        Declaration::Class(class) => {
                            class.body.iter().find_map(|statement| match statement {
                                Statement::StoredProperty {
                                    decorators,
                                    name,
                                    annotation,
                                    visibility,
                                    ..
                                } if decorators
                                    .iter()
                                    .any(|known| std::ptr::eq(known, decorator)) =>
                                {
                                    Some(PropertyDeclaration {
                                        name: name.clone(),
                                        annotation: annotation.clone(),
                                        visibility: *visibility,
                                    })
                                }
                                _ => None,
                            })
                        }
                        _ => None,
                    },
                    target: target.clone(),
                    method,
                    decorator: decorator_index,
                    arguments,
                    kind,
                });
            }
        }
    }
    Ok(applications)
}
