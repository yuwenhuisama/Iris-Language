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
        // An IMPORT binds names rather than declaring a callable, so it
        // contributes no signature. The binding happens where the program's
        // statements are lowered, at the import's own source position.
        if matches!(declaration, iris_syntax::Declaration::Import(_)) {
            continue;
        }
        let iris_syntax::Declaration::Module(module) = declaration else {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(CompileError::new(match declaration {
                    iris_syntax::Declaration::Contract(_) => "declaration covered",
                    // A `from S import K` binds names rather than declaring a
                    // callable, so it contributes no signature and is handled
                    // where the program's statements are lowered.
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
        // Type PARAMETERS annotate a module the way they annotate a class or
        // contract: `module Helpers<T> { fun h() { 7 } }` declares the same
        // method either way. A `where Self: T` constraint is such an
        // annotation too - the reference RUNS
        // `module Helpers<T> where Self: T {} 1` and answers `1`, so declining
        // refused a program that runs. It governs a surface the backend has no
        // other support for, so a program that DEPENDS on it fails there
        // rather than here.
        // A module REOPEN adds to the module it names rather than declaring a
        // new one: its methods join the same owner, and a republished selector
        // wins because resolution takes the LAST definition. The target must
        // already be declared, since there is otherwise nothing to add to.
        if module.reopen {
            // A reopen whose target is not declared adds to nothing, and the
            // reference still RUNS the program - `open module Absent {} 1`
            // answers `1` - so its methods are dropped rather than the program
            // being refused. A call to one then fails where it is made.
            if modules
                .iter()
                .any(|known: &crate::compile::ir::ModuleDeclaration| known.name == module.name)
            {
                collect_methods(&module.name, &module.body, false, &mut signatures)?;
            }
            continue;
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
            // A module composed from one that is not DECLARED has nothing to
            // compose: the reference raises when the program runs, so the
            // backend raises rather than declining.
            if !modules
                .iter()
                .any(|known: &crate::compile::ir::ModuleDeclaration| known.name == *name)
            {
                return Err(CompileError::new("module mixin unbound"));
            }
            // A PRIVATE-access mixin carries visibility rules the backend does
            // not model, and the reference runs the declaration either way, so
            // the edge is composed and the visibility surface is where a
            // program depending on it fails.
            mixins.push(name.clone());
        }
        modules.push(crate::compile::ir::ModuleDeclaration {
            name: module.name.clone(),
            mixins,
        });
        // A module's `shared class property` is READ as a member - `M.first`
        // answers it, unlike a `const`, which is visible only lexically. It is
        // module state with no receiver, so it is synthesized as a
        // receiverless reader whose body is the initializer expression.
        for statement in &module.body {
            let Statement::StoredProperty {
                name, initializer, ..
            } = statement
            else {
                continue;
            };
            signatures.push(Signature {
                module: &module.name,
                selector: name,
                parameters: Vec::new(),
                return_type: None,
                body: &[],
                receiver: false,
                class_method: false,
                is_async: false,
                constants: Vec::new(),
                expression_body: Some(initializer),
            });
        }
        collect_methods(&module.name, &module.body, false, &mut signatures)?;
    }
    // `D-173` puts the contract-visible SIGNATURE in the static spine, so a
    // member composed from a MIXIN whose parameter Type contradicts a declared
    // requirement is an incompatible replacement. That is checked here rather
    // than while a class is collected, because the module supplying the member
    // may be declared after the class that mixes it in.
    for class in &classes {
        for contract in &class.contracts {
            for requirement in &contracts[*contract].requirements {
                let composed = class.mixins.iter().find_map(|mixin| {
                    declarations
                        .iter()
                        .find_map(|declaration| match declaration {
                            iris_syntax::Declaration::Module(module) if module.name == *mixin => {
                                module.body.iter().find_map(|statement| match statement {
                                    Statement::Method(method)
                                        if method.selector == requirement.selector =>
                                    {
                                        Some(method)
                                    }
                                    _ => None,
                                })
                            }
                            _ => None,
                        })
                });
                let Some(method) = composed else {
                    continue;
                };
                // An UNANNOTATED position states nothing, so it is left alone
                // rather than treated as a mismatch.
                let clashes = method.parameters.len() == requirement.arity
                    && method
                        .parameters
                        .iter()
                        .zip(&requirement.parameter_types)
                        .any(
                            |(actual, required)| match (actual.annotation.as_ref(), required) {
                                (Some(TypeExpression::Name(actual)), Some(required)) => {
                                    actual != required
                                }
                                _ => false,
                            },
                        );
                if clashes {
                    return Err(CompileError::new("contract signature clash"));
                }
            }
        }
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
    // A decorator, a `where` constraint, a `meta deny` list and type
    // PARAMETERS annotate the declaration without changing its REQUIREMENTS -
    // `contract Comparable<T> {}` declares none either way - so they are
    // accepted the way the class forms are. `open` is such an annotation too:
    // it governs whether the contract may be REOPENED, which is a separate
    // surface, and the requirement set is the same either way.
    //
    // `extends` is NOT an annotation: a child carries its parents'
    // requirements as well as its own, so they are inherited here rather than
    // ignored. Ignoring them would answer a requirement set that is wrong
    // rather than merely incomplete. A parent must already be declared, since
    // its requirements have to exist to be inherited.
    let mut requirements = Vec::new();
    for parent in &declaration.parents {
        let TypeExpression::Name(name) = parent else {
            return Err(CompileError::new("contract declaration form"));
        };
        let Some(parent) = contracts.iter().find(|known| known.name == *name) else {
            return Err(CompileError::new("contract parent unbound"));
        };
        requirements.extend(parent.requirements.iter().cloned());
    }
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
            parameter_types: method
                .parameters
                .iter()
                .map(|parameter| match parameter.annotation.as_ref() {
                    Some(TypeExpression::Name(name)) => Some(name.clone()),
                    _ => None,
                })
                .collect(),
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
        return collect_reopen(class, contracts, signatures, classes, builtin_reopens);
    }
    // A mixin names a MODULE, which the runtime composes into the class's MRO.
    // A generic or private-access mixin carries rules the backend does not
    // model yet, so only the plain form is lowered rather than approximated.
    let mut mixins = Vec::with_capacity(class.mixins.len());
    for mixin in &class.mixins {
        // A GENERIC module mixin names the same module whatever its argument:
        // `mixin Helpers<String>` and `mixin Helpers<_>` both compose
        // `Helpers`, since the backend does not specialise a module per
        // argument any more than the reference publishes one - a wildcard
        // argument therefore reaches the same methods a concrete one does.
        let name = match &mixin.target {
            TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => name,
            _ => return Err(CompileError::new("class mixin")),
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
    //
    // A CONTRACT bound is the exception: `C067` checks it when the class is
    // MATERIALIZED, not where it is declared, so `class Box<T> where T:
    // Comparable<T> {}` declares fine and `Box<String>.new()` is the failure.
    // The bound is recorded by PARAMETER POSITION so a construction naming
    // concrete arguments can decide it.
    let mut contract_bounds = Vec::new();
    for constraint in &class.constraints {
        let named = match &constraint.bound {
            TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => Some(name),
            _ => None,
        };
        let Some(bound) = named else {
            continue;
        };
        let Some(contract) = contracts.iter().position(|known| known.name == *bound) else {
            continue;
        };
        let Some(position) = class
            .parameters
            .iter()
            .position(|parameter| *parameter == constraint.parameter)
        else {
            continue;
        };
        contract_bounds.push((position, contract));
    }
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
    // A subclass INHERITS its superclass's conformances, so an `impl` marker
    // there names a requirement the ancestry declares even when the subclass's
    // own header does not: `class A extends B` may implement `B`'s contract.
    let mut inherited = conformances.clone();
    let mut ancestor = superclass;
    while let Some(index) = ancestor {
        let Some(known) = classes.get(index) else {
            break;
        };
        inherited.extend(known.contracts.iter().copied());
        ancestor = known.superclass;
    }
    validate_contracts(class, contracts, &inherited)?;
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
    // A QUALIFIED `impl fun C::m()` belongs to the contract's VIEW rather than
    // to the class, so it is recorded separately: publishing it as an ordinary
    // method would make `a.m()` answer it too.
    // Signatures are collected in BODY order, so each method is matched to its
    // own by counting the methods before it. Matching by selector alone made
    // two qualified impls of one selector - `impl fun C::m()` beside
    // `impl fun D::m()` - both resolve to the first body.
    let mut qualified_impls = Vec::new();
    let mut position = 0usize;
    for statement in &class.body {
        let Statement::Method(method) = statement else {
            continue;
        };
        if method.body.is_none() {
            continue;
        }
        let function = first_function + position;
        position += 1;
        let Some(Some(qualifier)) = method.impl_contract.as_ref() else {
            continue;
        };
        let Some(contract) = contracts.iter().position(|known| known.name == *qualifier) else {
            return Err(CompileError::new("contract implementation undeclared"));
        };
        qualified_impls.push((contract, method.selector.clone(), function));
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
        contract_bounds,
        qualified_impls,
        property_methods,
        class_variables,
        stored_properties,
    });
    Ok(())
}

fn collect_reopen<'a>(
    class: &'a iris_syntax::ClassDeclaration,
    contracts: &[Contract],
    signatures: &mut Vec<Signature<'a>>,
    classes: &mut [Class],
    builtin_reopens: &mut Vec<crate::compile::ir::BuiltinReopen>,
) -> Result<(), CompileError> {
    // Type PARAMETERS and a `where` constraint on a reopen restate the
    // declaration's own header rather than changing it - the reference runs
    // `class Box<T> { }; open class Box<T> where T: Object { }; 1` and answers
    // `1`. A superclass or a conformance would change what the class IS, so
    // those stay declined; a MIXIN composes a module into the class the same
    // way a declaration's does, and is applied below.
    // A CONFORMANCE names a contract the class satisfies, and is observable
    // through `A.contracts`, so it joins the declaration's own list rather
    // than being ignored. A superclass would change what the class IS, so it
    // stays declined; a MIXIN composes a module the same way a declaration's
    // does, and both are applied below.
    if class.extends.is_some() || !class.meta_deny.is_empty() {
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
        // A BUILT-IN class is the kernel's, so it has no declaration entry a
        // mixin edge could join. A CONFORMANCE needs no entry: it is recorded
        // on the reopen itself, which is what makes `1 as N` a legitimate view.
        if !class.mixins.is_empty() {
            return Err(CompileError::new("class reopen header"));
        }
        let conformances = contract_indices(class, contracts)?;
        let first_function = signatures.len();
        collect_methods(&class.name, &class.body, true, signatures)?;
        let (methods, class_methods) = collected_method_tables(signatures, first_function);
        // A BUILT-IN class's singleton side is the kernel's, not the
        // program's, so a class method on one has no table to publish onto.
        if !class_methods.is_empty() {
            return Err(CompileError::new("class reopen class method"));
        }
        builtin_reopens.push(crate::compile::ir::BuiltinReopen {
            target: class.name.clone(),
            methods,
            contracts: conformances,
        });
        return Ok(());
    }
    let Some(target) = classes.iter().position(|known| known.name == class.name) else {
        return Err(CompileError::new("class reopen target"));
    };
    // A reopen's MIXIN composes into the class the runtime already registers,
    // so the edge joins the declaration's own list rather than needing a
    // second composition path.
    for contract in contract_indices(class, contracts)? {
        if !classes[target].contracts.contains(&contract) {
            classes[target].contracts.push(contract);
        }
    }
    for mixin in &class.mixins {
        let (TypeExpression::Name(name) | TypeExpression::Generic { name, .. }) = &mixin.target
        else {
            return Err(CompileError::new("class reopen header"));
        };
        if mixin.private_access {
            return Err(CompileError::new("class mixin"));
        }
        classes[target].mixins.push(name.clone());
    }
    let first_function = signatures.len();
    collect_methods(&class.name, &class.body, true, signatures)?;
    let (methods, class_methods) = collected_method_tables(signatures, first_function);
    classes[target].reopens.push(ClassReopen {
        methods,
        class_methods,
    });
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
        // A QUALIFIED `impl fun C::m()` names its contract explicitly and is
        // visible only through that view, so it needs no matching requirement:
        // the reference runs `contract C { }` with `impl fun C::m()` and
        // answers `:qualified` through the view.
        if method.impl_contract.is_none() || matches!(method.impl_contract, Some(Some(_))) {
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
        // the caller into the top-level frame, so they are skipped here.
        if !receiver && !matches!(statement, Statement::Method(_)) {
            continue;
        }
        // A CLASS body's `let` or `mut` is NOT an ivar initializer: the
        // reference answers nil for `@done` after `mut done = false`, so the
        // binding declares nothing the object carries. It is accepted and
        // ignored rather than declined, which is what the reference does.
        if receiver && matches!(statement, Statement::Binding { .. }) {
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
