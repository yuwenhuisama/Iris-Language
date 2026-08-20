use iris_syntax::{Statement, TypeExpression};

use super::{Class, ClassReopen, CompileError};

pub(super) struct Signature<'a> {
    pub(super) module: &'a str,
    pub(super) selector: &'a str,
    pub(super) parameters: Vec<&'a str>,
    pub(super) body: &'a [Statement],
    pub(super) receiver: bool,
    pub(super) class_method: bool,
}

type MethodTable = Vec<(String, usize)>;

pub(super) fn collect_signatures(
    declarations: &[iris_syntax::Declaration],
) -> Result<(Vec<Signature<'_>>, Vec<Class>), CompileError> {
    let mut signatures = Vec::new();
    let mut classes = Vec::new();
    for declaration in declarations {
        let iris_syntax::Declaration::Module(module) = declaration else {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(CompileError::new(match declaration {
                    iris_syntax::Declaration::Contract(_) => "declaration contract",
                    iris_syntax::Declaration::Import(_) => "declaration import",
                    iris_syntax::Declaration::Export(_) => "declaration export",
                    iris_syntax::Declaration::TypeAlias(_) => "declaration type alias",
                    iris_syntax::Declaration::Class(_) | iris_syntax::Declaration::Module(_) => {
                        "declaration covered"
                    }
                }));
            };
            collect_class(declarations, class, &mut signatures, &mut classes)?;
            continue;
        };
        if module.reopen
            || !module.mixins.is_empty()
            || !module.parameters.is_empty()
            || !module.decorators.is_empty()
        {
            return Err(CompileError::new("module"));
        }
        collect_methods(&module.name, &module.body, false, &mut signatures)?;
    }
    Ok((signatures, classes))
}

fn collect_class<'a>(
    declarations: &'a [iris_syntax::Declaration],
    class: &'a iris_syntax::ClassDeclaration,
    signatures: &mut Vec<Signature<'a>>,
    classes: &mut Vec<Class>,
) -> Result<(), CompileError> {
    if !class.decorators.is_empty() {
        return Err(CompileError::new("class decorator"));
    }
    if class.reopen {
        return collect_reopen(class, signatures, classes);
    }
    if !class.implements.is_empty() {
        return Err(CompileError::new("class implements"));
    }
    if !class.mixins.is_empty() {
        return Err(CompileError::new("class mixin"));
    }
    if !class.constraints.is_empty() {
        return Err(CompileError::new("class constraints"));
    }
    if !class.parameters.is_empty() {
        return Err(CompileError::new("class generics"));
    }
    if !class.meta_deny.is_empty() {
        return Err(CompileError::new("class meta deny"));
    }
    let superclass_name = match &class.extends {
        Some(TypeExpression::Name(name)) => Some(name.as_str()),
        Some(_) => return Err(CompileError::new("class superclass")),
        None => None,
    };
    let first_function = signatures.len();
    collect_methods(&class.name, &class.body, true, signatures)?;
    let superclass = match superclass_name {
        Some("Object") | None => None,
        Some(name) => declarations
            .iter()
            .filter_map(|declaration| match declaration {
                iris_syntax::Declaration::Class(candidate) => Some(&candidate.name),
                _ => None,
            })
            .position(|candidate| candidate == name)
            .ok_or_else(|| CompileError::new("class superclass"))?
            .into(),
    };
    let (methods, class_methods) = collected_method_tables(signatures, first_function);
    classes.push(Class {
        name: class.name.clone(),
        superclass,
        methods,
        class_methods,
        reopens: Vec::new(),
    });
    Ok(())
}

fn collect_reopen<'a>(
    class: &'a iris_syntax::ClassDeclaration,
    signatures: &mut Vec<Signature<'a>>,
    classes: &mut [Class],
) -> Result<(), CompileError> {
    if class.extends.is_some()
        || !class.implements.is_empty()
        || !class.mixins.is_empty()
        || !class.constraints.is_empty()
        || !class.parameters.is_empty()
        || !class.meta_deny.is_empty()
    {
        return Err(CompileError::new("class reopen header"));
    }
    let Some(target) = classes.iter().position(|known| known.name == class.name) else {
        return Err(CompileError::new("class reopen target"));
    };
    let first_function = signatures.len();
    collect_methods(&class.name, &class.body, true, signatures)?;
    let (methods, class_methods) = collected_method_tables(signatures, first_function);
    if !class_methods.is_empty() {
        return Err(CompileError::new("class reopen class method"));
    }
    classes[target].reopens.push(ClassReopen { methods });
    Ok(())
}

fn collected_method_tables(
    signatures: &[Signature<'_>],
    first_function: usize,
) -> (MethodTable, MethodTable) {
    let mut methods = Vec::new();
    let mut class_methods = Vec::new();
    for (offset, signature) in signatures[first_function..].iter().enumerate() {
        let entry = (signature.selector.to_owned(), first_function + offset);
        if signature.class_method {
            class_methods.push(entry);
        } else {
            methods.push(entry);
        }
    }
    (methods, class_methods)
}

fn collect_methods<'a>(
    owner: &'a str,
    body: &'a [Statement],
    receiver: bool,
    signatures: &mut Vec<Signature<'a>>,
) -> Result<(), CompileError> {
    for statement in body {
        let Statement::Method(method) = statement else {
            return Err(CompileError::new(if receiver {
                "class body"
            } else {
                "module body"
            }));
        };
        if method.is_async
            || method.impl_contract.is_some()
            || !method.decorators.is_empty()
            || !method.type_parameters.is_empty()
            || !matches!(
                method.kind,
                iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class
            )
            || (!receiver && method.kind != iris_syntax::MethodKind::Instance)
        {
            return Err(method_error(method));
        }
        let Some(body) = method.body.as_deref() else {
            return Err(CompileError::new("abstract method"));
        };
        let mut parameters = Vec::with_capacity(method.parameters.len());
        for parameter in &method.parameters {
            if parameter.category != iris_syntax::ParameterCategory::Positional {
                return Err(CompileError::new("parameter"));
            }
            parameters.push(parameter.name.as_str());
        }
        signatures.push(Signature {
            module: owner,
            selector: &method.selector,
            parameters,
            body,
            receiver,
            class_method: method.kind == iris_syntax::MethodKind::Class,
        });
    }
    Ok(())
}

fn method_error(method: &iris_syntax::MethodDeclaration) -> CompileError {
    CompileError::new(if method.is_async {
        "method async"
    } else if method.impl_contract.is_some() {
        "method contract implementation"
    } else if !method.decorators.is_empty() {
        "method decorator"
    } else if !method.type_parameters.is_empty() {
        "method generics"
    } else {
        match method.kind {
            iris_syntax::MethodKind::Module => "method module",
            iris_syntax::MethodKind::Property => "method property",
            iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class => "method kind",
        }
    })
}
