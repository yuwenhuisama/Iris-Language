use iris_syntax::{Statement, TypeExpression};

use super::{
    Class, ClassReopen, ClassVariable, CompileError, Contract, ContractRequirement, LiteralValue,
    StoredProperty,
};

pub(super) struct Signature<'a> {
    pub(super) module: &'a str,
    pub(super) selector: &'a str,
    pub(super) parameters: Vec<&'a iris_syntax::Parameter>,
    pub(super) return_type: Option<&'a TypeExpression>,
    pub(super) body: &'a [Statement],
    pub(super) receiver: bool,
    pub(super) class_method: bool,
    pub(super) is_async: bool,
    /// Constants declared in the OWNER's body, visible lexically in this one.
    ///
    /// `IRIS-V1-CONTROL-D-432` scopes a module `const` to the module rather
    /// than exposing it as a member: `M.K` is a MessageNotFound, but a method
    /// of `M` reads `K` directly. Binding it at frame entry is what gives it
    /// that visibility without making it a selector.
    pub(super) constants: Vec<(&'a str, &'a iris_syntax::Expression)>,
    /// An EXPRESSION body, for a synthesized stored-property initializer.
    ///
    /// A stored property's initializer is an expression rather than a block,
    /// but it needs a frame with `self` bound just as a method does, so it is
    /// lowered as one instead of growing a second lowering path.
    pub(super) expression_body: Option<&'a iris_syntax::Expression>,
}

type MethodTable = Vec<(String, usize)>;

pub(super) struct CollectedDeclarations<'a> {
    pub(super) signatures: Vec<Signature<'a>>,
    pub(super) classes: Vec<Class>,
    pub(super) modules: Vec<crate::compile::ir::ModuleDeclaration>,
    pub(super) builtin_reopens: Vec<crate::compile::ir::BuiltinReopen>,
    pub(super) contracts: Vec<Contract>,
}

pub(super) fn collect_signatures(
    declarations: &[iris_syntax::Declaration],
) -> Result<CollectedDeclarations<'_>, CompileError> {
    let mut signatures = Vec::new();
    let mut classes = Vec::new();
    let mut contracts = Vec::new();
    let mut modules = Vec::new();
    let mut builtin_reopens = Vec::new();
    // A REOPEN is collected after every origin declaration, because it names a
    // class that may be declared later in the source: `open class A { }` ahead
    // of `class A { }` is an ordinary program, and collecting in source order
    // refused it for an ordering the language does not impose.
    let ordered = declarations
        .iter()
        .filter(|declaration| {
            !matches!(declaration,
            iris_syntax::Declaration::Class(class) if class.reopen)
        })
        .chain(declarations.iter().filter(|declaration| {
            matches!(declaration,
            iris_syntax::Declaration::Class(class) if class.reopen)
        }));
    for declaration in ordered {
        if let iris_syntax::Declaration::Contract(contract) = declaration {
            collect_contract(contract, &mut contracts)?;
            continue;
        }
        // A TYPE ALIAS names an existing type and declares nothing the backend
        // acts on: the reference runs `type Alias = Integer; 1` and answers
        // `1`. It is not dropped semantics - the backend has no other support
        // for the type surface either, so a program that DEPENDS on the alias
        // fails on that surface rather than at the declaration.
        if matches!(declaration, iris_syntax::Declaration::TypeAlias(_)) {
            continue;
        }
        let iris_syntax::Declaration::Module(module) = declaration else {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(CompileError::new(match declaration {
                    iris_syntax::Declaration::Contract(_) => "declaration covered",
                    iris_syntax::Declaration::Import(_) => "declaration import",
                    iris_syntax::Declaration::Export(_) => "declaration export",
                    iris_syntax::Declaration::TypeAlias(_) => "declaration type alias",
                    iris_syntax::Declaration::Class(_) | iris_syntax::Declaration::Module(_) => {
                        "declaration covered"
                    }
                }));
            };
            collect_class(
                declarations,
                class,
                &contracts,
                &mut signatures,
                &mut classes,
                &mut builtin_reopens,
            )?;
            continue;
        };
        if module.reopen || !module.parameters.is_empty() {
            return Err(CompileError::new("module"));
        }
        // A module may itself mix in another module, and the runtime composes
        // those edges the same way it does a class's. The composed module must
        // already be DECLARED: the reference refuses a forward reference, so
        // accepting one would answer a value where the language does not.
        let mut mixins = Vec::with_capacity(module.mixins.len());
        for mixin in &module.mixins {
            let TypeExpression::Name(name) = &mixin.target else {
                return Err(CompileError::new("module"));
            };
            if mixin.private_access
                || !modules
                    .iter()
                    .any(|known: &crate::compile::ir::ModuleDeclaration| known.name == *name)
            {
                return Err(CompileError::new("module"));
            }
            mixins.push(name.clone());
        }
        modules.push(crate::compile::ir::ModuleDeclaration {
            name: module.name.clone(),
            mixins,
        });
        collect_methods(&module.name, &module.body, false, &mut signatures)?;
    }
    Ok(CollectedDeclarations {
        signatures,
        classes,
        contracts,
        modules,
        builtin_reopens,
    })
}

fn collect_contract(
    declaration: &iris_syntax::ContractDeclaration,
    contracts: &mut Vec<Contract>,
) -> Result<(), CompileError> {
    // A decorator, a `where` constraint and a `meta deny` list annotate the
    // declaration without changing its REQUIREMENTS, so they are accepted the
    // way the class forms are. `open`, type parameters and `extends` are not
    // annotations: each changes which requirements the contract carries, and
    // accepting them would answer a requirement set that is wrong rather than
    // merely incomplete.
    if declaration.open || !declaration.parameters.is_empty() || !declaration.parents.is_empty() {
        return Err(CompileError::new("contract declaration form"));
    }
    let mut requirements = Vec::new();
    for statement in &declaration.body {
        let Statement::Method(method) = statement else {
            return Err(CompileError::new("contract body"));
        };
        if matches!(method.impl_contract, Some(Some(_))) {
            return Err(CompileError::new("qualified contract implementation"));
        }
        // A requirement may carry a DEFAULT BODY, which the contract offers to
        // an implementor rather than declaring anything else: the reference
        // runs `contract C { fun m() -> Nil { nil } } 1` and answers `1`. The
        // body is not lowered, because a class satisfying the requirement
        // supplies its own - only the requirement's SHAPE is recorded.
        if method.kind != iris_syntax::MethodKind::Instance
            || method.impl_contract.is_some()
            || method.is_async
            || !method.decorators.is_empty()
            || !method.type_parameters.is_empty()
        {
            return Err(CompileError::new("contract requirement form"));
        }
        if method
            .parameters
            .iter()
            .any(|parameter| parameter.category != iris_syntax::ParameterCategory::Positional)
        {
            return Err(CompileError::new("parameter"));
        }
        requirements.push(ContractRequirement {
            selector: method.selector.clone(),
            arity: method.parameters.len(),
            return_type: method
                .return_type
                .as_ref()
                .and_then(|annotation| match annotation {
                    TypeExpression::Name(name) => Some(name.clone()),
                    _ => None,
                }),
        });
    }
    contracts.push(Contract {
        name: declaration.name.clone(),
        requirements,
    });
    Ok(())
}

fn collect_class<'a>(
    declarations: &'a [iris_syntax::Declaration],
    class: &'a iris_syntax::ClassDeclaration,
    contracts: &[Contract],
    signatures: &mut Vec<Signature<'a>>,
    classes: &mut Vec<Class>,
    builtin_reopens: &mut Vec<crate::compile::ir::BuiltinReopen>,
) -> Result<(), CompileError> {
    if class.reopen {
        return collect_reopen(class, signatures, classes, builtin_reopens);
    }
    // A mixin names a MODULE, which the runtime composes into the class's MRO.
    // A generic or private-access mixin carries rules the backend does not
    // model yet, so only the plain form is lowered rather than approximated.
    let mut mixins = Vec::with_capacity(class.mixins.len());
    for mixin in &class.mixins {
        let TypeExpression::Name(name) = &mixin.target else {
            return Err(CompileError::new("class mixin"));
        };
        if mixin.private_access {
            return Err(CompileError::new("class mixin"));
        }
        mixins.push(name.clone());
    }
    // A DECORATOR, a `where` constraint and a `meta deny` list are all
    // declaration-time annotations the reference accepts and this backend does
    // not act on: `class A<T> where T: Object { } 1` answers `1` there, so
    // declining refused a program that runs. They are not silently dropped
    // semantics - each governs a surface (decoration, generic bounds, meta
    // capability) the backend has no other support for either, so a program
    // that DEPENDS on one fails on that surface rather than here.
    let superclass_name = match &class.extends {
        Some(TypeExpression::Name(name)) => Some(name.as_str()),
        Some(_) => return Err(CompileError::new("class superclass")),
        None => None,
    };
    let first_function = signatures.len();
    let conformances = contract_indices(class, contracts)?;
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
    validate_contracts(class, contracts, &conformances)?;
    let property_methods = class
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::Method(method) if method.kind == iris_syntax::MethodKind::Property => {
                Some(method.selector.clone())
            }
            _ => None,
        })
        .collect();
    let mut class_variables = class
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::SharedBinding {
                mutable,
                name,
                value,
                ..
            } => Some(class_variable(name, *mutable, value)),
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?;
    // A stored property whose initializer is not a literal gets a synthesized
    // FRAME with `self` bound, appended after the class's own methods so its
    // index is stable once every declaration has been collected.
    let mut stored_properties = Vec::new();
    for statement in &class.body {
        let Statement::StoredProperty {
            class_level,
            shared,
            name,
            initializer,
            ..
        } = statement
        else {
            continue;
        };
        // A CLASS-level property is class state rather than per-instance
        // storage: `Cache.n` reads it and `Cache.n = 9` writes it, which is
        // exactly what a class variable already does. An absent initializer
        // starts it at nil, since the reference lets `Cache.value` be written
        // before it is ever read.
        if *class_level {
            // `IRIS-V1-TYPES-C064` puts a `shared class property` on the
            // UNAPPLIED generic definition, while a plain one belongs to each
            // closed construction - so on a generic class the bare name does
            // not reach it, and the reference answers MessageNotFound. Storing
            // it as one class variable would answer a value where the language
            // has none.
            if !class.parameters.is_empty() && !shared {
                return Err(CompileError::new("class-level stored property"));
            }
            let initializer = literal_value(initializer, "class-level stored property")
                .unwrap_or(LiteralValue::Nil);
            class_variables.push(ClassVariable {
                name: name.clone(),
                mutable: true,
                initializer,
            });
            continue;
        }
        if let Ok(literal) = literal_value(initializer, "stored property initializer") {
            stored_properties.push(StoredProperty {
                name: name.clone(),
                initializer: literal,
                initializer_function: None,
            });
            continue;
        }
        signatures.push(Signature {
            module: &class.name,
            selector: name,
            parameters: Vec::new(),
            return_type: None,
            body: &[],
            receiver: true,
            class_method: false,
            is_async: false,
            constants: Vec::new(),
            expression_body: Some(initializer),
        });
        stored_properties.push(StoredProperty {
            name: name.clone(),
            initializer: LiteralValue::Nil,
            initializer_function: Some(signatures.len() - 1),
        });
    }
    classes.push(Class {
        name: class.name.clone(),
        generic: !class.parameters.is_empty(),
        superclass,
        methods,
        class_methods,
        reopens: Vec::new(),
        contracts: conformances,
        mixins,
        property_methods,
        class_variables,
        stored_properties,
    });
    Ok(())
}

fn collect_reopen<'a>(
    class: &'a iris_syntax::ClassDeclaration,
    signatures: &mut Vec<Signature<'a>>,
    classes: &mut [Class],
    builtin_reopens: &mut Vec<crate::compile::ir::BuiltinReopen>,
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
    // A BUILT-IN class is created by the kernel and has no entry here, so a
    // reopen of one is recorded by NAME and published onto the kernel's class
    // at load. Only the classes the kernel actually defines are accepted: a
    // name that is neither declared nor built in has no target at all.
    if !classes.iter().any(|known| known.name == class.name) {
        if !matches!(
            class.name.as_str(),
            "Object" | "Nil" | "Bool" | "Integer" | "Float32" | "Float64" | "String"
        ) {
            return Err(CompileError::new("class reopen target"));
        }
        let first_function = signatures.len();
        collect_methods(&class.name, &class.body, true, signatures)?;
        let (methods, class_methods) = collected_method_tables(signatures, first_function);
        if !class_methods.is_empty() {
            return Err(CompileError::new("class reopen class method"));
        }
        builtin_reopens.push(crate::compile::ir::BuiltinReopen {
            target: class.name.clone(),
            methods,
        });
        return Ok(());
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

fn contract_indices(
    class: &iris_syntax::ClassDeclaration,
    contracts: &[Contract],
) -> Result<Vec<usize>, CompileError> {
    class
        .implements
        .iter()
        .map(|target| {
            let TypeExpression::Name(name) = target else {
                return Err(CompileError::new("class contract type"));
            };
            contracts
                .iter()
                .position(|contract| contract.name == *name)
                .ok_or_else(|| CompileError::new("class contract unbound"))
        })
        .collect()
}

fn validate_contracts(
    class: &iris_syntax::ClassDeclaration,
    contracts: &[Contract],
    conformances: &[usize],
) -> Result<(), CompileError> {
    for statement in &class.body {
        let Statement::Method(method) = statement else {
            continue;
        };
        if method.impl_contract.is_none() {
            continue;
        }
        let matches = conformances.iter().any(|contract| {
            contracts[*contract].requirements.iter().any(|requirement| {
                requirement.selector == method.selector
                    && requirement.arity == method.parameters.len()
            })
        });
        if !matches {
            return Err(CompileError::new("contract implementation undeclared"));
        }
    }
    // A conformance is NOT enforced when the class is declared: the reference
    // runs `class X for C { }` with `C`'s requirement unimplemented, and a
    // plain method satisfies a requirement without an `impl` marker. Refusing
    // the declaration declined programs that run, and demanding the marker
    // refused the ordinary form as well.
    let _ = conformances;
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
    // A module body may declare CONSTANTS alongside its methods, and they are
    // visible lexically inside those methods rather than as members, so they
    // are collected before any method is lowered.
    let constants = body
        .iter()
        .filter_map(|statement| match statement {
            Statement::Binding {
                constant: true,
                name,
                value,
                ..
            } => Some((name.as_str(), value)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for statement in body {
        if matches!(
            statement,
            Statement::SharedBinding { .. } | Statement::StoredProperty { .. }
        ) {
            continue;
        }
        // A `const` is state rather than a callable, so it contributes no
        // signature and is skipped here after being collected above.
        if matches!(statement, Statement::Binding { constant: true, .. }) {
            continue;
        }
        // A MODULE body may also hold ordinary statements, which run when the
        // program loads rather than declaring anything. They are lowered by
        // the caller into the top-level frame, so they are skipped here. A
        // CLASS body has no such phase, so the same statement stays declined.
        if !receiver && !matches!(statement, Statement::Method(_)) {
            continue;
        }
        let Statement::Method(method) = statement else {
            return Err(CompileError::new(if receiver {
                "class body"
            } else {
                "module body"
            }));
        };
        // A `module fun` is a MODULE-level callable, which is exactly how a
        // module's ordinary `fun` is already reached: `M.f()`. On a class it
        // is a different shape, so it stays declined there.
        let module_level = !receiver && method.kind == iris_syntax::MethodKind::Module;
        if !method.decorators.is_empty()
            || !matches!(
                method.kind,
                iris_syntax::MethodKind::Instance
                    | iris_syntax::MethodKind::Class
                    | iris_syntax::MethodKind::Property
                    | iris_syntax::MethodKind::Module
            )
            || (!receiver && !module_level && method.kind != iris_syntax::MethodKind::Instance)
            || (receiver && method.kind == iris_syntax::MethodKind::Module)
        {
            return Err(method_error(method));
        }
        let Some(body) = method.body.as_deref() else {
            return Err(CompileError::new("abstract method"));
        };
        let mut parameters = Vec::with_capacity(method.parameters.len());
        for parameter in &method.parameters {
            parameters.push(parameter);
        }
        signatures.push(Signature {
            module: owner,
            selector: &method.selector,
            parameters,
            return_type: method.return_type.as_ref(),
            body,
            receiver,
            class_method: method.kind == iris_syntax::MethodKind::Class,
            is_async: method.is_async,
            constants: constants.clone(),
            expression_body: None,
        });
    }
    Ok(())
}

fn class_variable(
    name: &str,
    mutable: bool,
    value: &iris_syntax::Expression,
) -> Result<ClassVariable, CompileError> {
    let initializer = literal_value(value, "class variable initializer")?;
    Ok(ClassVariable {
        name: name.to_owned(),
        mutable,
        initializer,
    })
}

fn literal_value(
    value: &iris_syntax::Expression,
    construct: &str,
) -> Result<LiteralValue, CompileError> {
    match value {
        iris_syntax::Expression::Literal(text) if text == "nil" => Ok(LiteralValue::Nil),
        iris_syntax::Expression::Literal(text) if text == "true" => Ok(LiteralValue::Bool(true)),
        iris_syntax::Expression::Literal(text) if text == "false" => Ok(LiteralValue::Bool(false)),
        iris_syntax::Expression::Literal(text) if text.starts_with('"') => {
            Ok(LiteralValue::Text(text.clone()))
        }
        iris_syntax::Expression::Literal(text) => Ok(LiteralValue::Integer(text.clone())),
        _ => Err(CompileError::new(construct)),
    }
}

fn method_error(method: &iris_syntax::MethodDeclaration) -> CompileError {
    CompileError::new(if method.is_async {
        "method async"
    } else if method.impl_contract.is_some() {
        "method contract implementation"
    } else if !method.decorators.is_empty() {
        "method decorator"
    } else {
        match method.kind {
            iris_syntax::MethodKind::Module => "method module",
            iris_syntax::MethodKind::Property => "method property",
            iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class => "method kind",
        }
    })
}
