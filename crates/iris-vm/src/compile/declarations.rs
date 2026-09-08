use iris_syntax::{Statement, TypeExpression};

use super::{
    Class, ClassReopen, ClassVariable, CompileError, Contract, ContractRequirement, LiteralValue,
    StoredProperty,
};

pub(super) struct Signature<'a> {
    pub(super) declaration: Option<&'a iris_syntax::MethodDeclaration>,
    pub(super) module: &'a str,
    pub(super) selector: &'a str,
    pub(super) parameters: Vec<&'a iris_syntax::Parameter>,
    pub(super) return_type: Option<&'a TypeExpression>,
    pub(super) body: &'a [Statement],
    pub(super) receiver: bool,
    pub(super) class_method: bool,
    /// Whether the source wrote `private` before the method.
    ///
    /// `IRIS-V1-RUNTIME-C077` refuses a private call from every path but the
    /// declaring class - or a module composed with `private` access - so the
    /// marker has to reach the registry rather than being dropped here.
    pub(super) private: bool,
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

fn written_constructions(
    declarations: &[iris_syntax::Declaration],
    top_level: &[Statement],
    class: &str,
) -> Vec<Vec<TypeExpression>> {
    let mut found = Vec::new();
    fn visit_expression(
        expression: &iris_syntax::Expression,
        class: &str,
        found: &mut Vec<Vec<TypeExpression>>,
    ) {
        match expression {
            iris_syntax::Expression::ClosedGeneric { name, arguments } => {
                if name == class
                    && crate::compile::lowering::Lowering::type_selector(arguments).is_some()
                    && !found.contains(arguments)
                {
                    found.push(arguments.clone());
                }
            }
            iris_syntax::Expression::Member { receiver, .. }
            | iris_syntax::Expression::ContractView { receiver, .. }
            | iris_syntax::Expression::Await(receiver)
            | iris_syntax::Expression::Grouped(receiver)
            | iris_syntax::Expression::Unary {
                operand: receiver, ..
            }
            | iris_syntax::Expression::KeywordArgument {
                value: receiver, ..
            } => visit_expression(receiver, class, found),
            iris_syntax::Expression::Call {
                callee, arguments, ..
            } => {
                visit_expression(callee, class, found);
                for argument in arguments {
                    visit_expression(argument, class, found);
                }
            }
            iris_syntax::Expression::Index { receiver, index } => {
                visit_expression(receiver, class, found);
                visit_expression(index, class, found);
            }
            iris_syntax::Expression::Array(values) | iris_syntax::Expression::Tuple(values) => {
                for value in values {
                    visit_expression(value, class, found);
                }
            }
            iris_syntax::Expression::Hash(entries) => {
                for (key, value) in entries {
                    visit_expression(key, class, found);
                    visit_expression(value, class, found);
                }
            }
            iris_syntax::Expression::Binary { left, right, .. }
            | iris_syntax::Expression::Assignment { left, right, .. } => {
                visit_expression(left, class, found);
                visit_expression(right, class, found);
            }
            iris_syntax::Expression::Closure { body, .. } => visit_statements(body, class, found),
            iris_syntax::Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                visit_expression(condition, class, found);
                visit_statements(then_body, class, found);
                if let Some(body) = else_body {
                    visit_statements(body, class, found);
                }
            }
            iris_syntax::Expression::While {
                condition, body, ..
            } => {
                visit_expression(condition, class, found);
                visit_statements(body, class, found);
            }
            iris_syntax::Expression::Try {
                body,
                catches,
                finally,
            } => {
                visit_statements(body, class, found);
                for catch in catches {
                    visit_type(catch.filter.as_ref(), class, found);
                    visit_statements(&catch.body, class, found);
                }
                if let Some(body) = finally {
                    visit_statements(body, class, found);
                }
            }
            iris_syntax::Expression::Yield(Some(value)) => visit_expression(value, class, found),
            iris_syntax::Expression::ReifiedType(expression) => {
                visit_type(Some(expression), class, found)
            }
            iris_syntax::Expression::Yield(None)
            | iris_syntax::Expression::Name(_)
            | iris_syntax::Expression::Literal(_)
            | iris_syntax::Expression::Symbol(_)
            | iris_syntax::Expression::RawIvar(_)
            | iris_syntax::Expression::ClassVar(_)
            | iris_syntax::Expression::GlobalVar(_) => {}
        }
    }
    fn visit_type(
        expression: Option<&TypeExpression>,
        class: &str,
        found: &mut Vec<Vec<TypeExpression>>,
    ) {
        let Some(expression) = expression else { return };
        match expression {
            TypeExpression::Typeof(value) => visit_expression(value, class, found),
            TypeExpression::Intersection(values) | TypeExpression::Union(values) => {
                for value in values {
                    visit_type(Some(value), class, found);
                }
            }
            TypeExpression::Generic { arguments, .. } => {
                for value in arguments {
                    visit_type(Some(value), class, found);
                }
            }
            TypeExpression::Function { parameters, result } => {
                for parameter in parameters {
                    visit_type(Some(parameter), class, found);
                }
                visit_type(Some(result), class, found);
            }
            TypeExpression::Name(_) => {}
        }
    }
    fn visit_statements(values: &[Statement], class: &str, found: &mut Vec<Vec<TypeExpression>>) {
        for statement in values {
            match statement {
                Statement::Binding { value, .. }
                | Statement::SharedBinding { value, .. }
                | Statement::GlobalBinding { value, .. }
                | Statement::Expression(value) => visit_expression(value, class, found),
                Statement::DeferredBinding { .. } | Statement::Continue(_) => {}
                Statement::StoredProperty {
                    annotation,
                    initializer,
                    ..
                } => {
                    visit_type(Some(annotation), class, found);
                    visit_expression(initializer, class, found);
                }
                Statement::Method(method) => {
                    visit_type(method.return_type.as_ref(), class, found);
                    for parameter in &method.parameters {
                        visit_type(parameter.annotation.as_ref(), class, found);
                        if let Some(value) = &parameter.default {
                            visit_expression(value, class, found);
                        }
                    }
                    if let Some(body) = &method.body {
                        visit_statements(body, class, found);
                    }
                }
                Statement::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    visit_expression(condition, class, found);
                    visit_statements(then_body, class, found);
                    if let Some(body) = else_body {
                        visit_statements(body, class, found);
                    }
                }
                Statement::While {
                    condition, body, ..
                } => {
                    visit_expression(condition, class, found);
                    visit_statements(body, class, found);
                }
                Statement::For { iterable, body, .. } => {
                    visit_expression(iterable, class, found);
                    visit_statements(body, class, found);
                }
                Statement::Return(Some(value))
                | Statement::Break {
                    value: Some(value), ..
                } => visit_expression(value, class, found),
                Statement::Return(None) | Statement::Break { value: None, .. } => {}
                Statement::Match {
                    subject,
                    arms,
                    fallback,
                } => {
                    visit_expression(subject, class, found);
                    for arm in arms {
                        if let Some(guard) = &arm.guard {
                            visit_expression(guard, class, found);
                        }
                        match &arm.body {
                            iris_syntax::MatchBody::Expression(value) => {
                                visit_expression(value, class, found)
                            }
                            iris_syntax::MatchBody::Block(body) => {
                                visit_statements(body, class, found)
                            }
                        }
                    }
                    if let Some(body) = fallback {
                        match body {
                            iris_syntax::MatchBody::Expression(value) => {
                                visit_expression(value, class, found)
                            }
                            iris_syntax::MatchBody::Block(body) => {
                                visit_statements(body, class, found)
                            }
                        }
                    }
                }
                Statement::Raise(Some(raise)) => {
                    visit_expression(&raise.value, class, found);
                    if let Some(cause) = &raise.cause {
                        visit_expression(cause, class, found);
                    }
                }
                Statement::Raise(None) => {}
                Statement::Try {
                    body,
                    catches,
                    finally,
                    ..
                } => {
                    visit_statements(body, class, found);
                    for catch in catches {
                        visit_type(catch.filter.as_ref(), class, found);
                        visit_statements(&catch.body, class, found);
                    }
                    if let Some(body) = finally {
                        visit_statements(body, class, found);
                    }
                }
            }
        }
    }
    for declaration in declarations {
        match declaration {
            iris_syntax::Declaration::Class(declaration) => {
                visit_statements(&declaration.body, class, &mut found)
            }
            iris_syntax::Declaration::Module(declaration) => {
                visit_statements(&declaration.body, class, &mut found)
            }
            _ => {}
        }
    }
    visit_statements(top_level, class, &mut found);
    found
}

pub(super) fn collect_signatures<'a>(
    declarations: &'a [iris_syntax::Declaration],
    statements: &[Statement],
) -> Result<CollectedDeclarations<'a>, CompileError> {
    let mut signatures = Vec::new();
    let mut classes = Vec::new();
    let mut contracts = traversal_contracts();
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
                statements,
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
        // A REDECLARED module REPLACES the earlier one rather than merging
        // with it: the reference answers MessageNotFound for the first
        // declaration's method afterwards, so keeping both would answer a
        // value the language does not have.
        signatures.retain(|signature| signature.module != module.name);
        modules.retain(|known: &crate::compile::ir::ModuleDeclaration| known.name != module.name);
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
                declaration: None,
                module: &module.name,
                selector: name,
                parameters: Vec::new(),
                return_type: None,
                body: &[],
                receiver: false,
                class_method: false,
                private: false,
                is_async: false,
                constants: Vec::new(),
                expression_body: Some(initializer),
            });
        }
        // A module method is lowered WITH a receiver when the module is mixed
        // into a class: dispatch prepends the composing object, so the frame
        // must bind `self` to it or the object would land in the first
        // parameter's register. A module never composed keeps the receiverless
        // form, where `M.f()` passes only its arguments.
        let composed = declarations.iter().any(|declaration| match declaration {
            iris_syntax::Declaration::Class(class) => class.mixins.iter().any(|mixin| {
                matches!(&mixin.target,
                    TypeExpression::Name(name) | TypeExpression::Generic { name, .. }
                        if *name == module.name)
            }),
            _ => false,
        });
        collect_methods(&module.name, &module.body, composed, &mut signatures)?;
    }
    // `D-173` puts the contract-visible SIGNATURE in the static spine, so a
    // member composed from a MIXIN whose parameter Type contradicts a declared
    // requirement is an incompatible replacement. That is checked here rather
    // than while a class is collected, because the module supplying the member
    // may be declared after the class that mixes it in.
    for class in &classes {
        for contract in &class.contracts {
            for requirement in &contracts[*contract].requirements {
                let composed = class.mixins.iter().find_map(|(mixin, _)| {
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
                // A class's OWN member is checked the same way: an `impl`
                // whose parameter Type contradicts the requirement does not
                // implement it, whatever the marker claims. Only the mixin
                // case was looked at, so a class stating the mismatch
                // directly passed unexamined.
                // A REOPEN's member is checked too: it REPLACES the one the
                // origin declared, so a replacement that no longer fits the
                // requirement leaves the conformance unmet just as an
                // original mismatch would.
                let declared = declarations
                    .iter()
                    .filter_map(|declaration| match declaration {
                        iris_syntax::Declaration::Class(candidate)
                            if candidate.name == class.name =>
                        {
                            candidate.body.iter().find_map(|statement| match statement {
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
                    .next_back();
                let Some(method) = composed.or(declared) else {
                    continue;
                };
                // A different ARITY is a mismatch on its own: a member taking
                // a different number of parameters cannot be called the way
                // the requirement states, whatever its Types say. An
                // UNANNOTATED position states nothing, so only two stated
                // Types can contradict each other.
                let clashes = method.parameters.len() != requirement.arity
                    || method
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
    // A denial written on a CONTRACT narrows every class that declares it:
    // the promise carries the restriction with it, so a conforming class is
    // registered with the contract's denials as well as its own.
    for class in &mut classes {
        for contract in &class.contracts {
            for denied in &contracts[*contract].meta_deny {
                if !class.meta_deny.contains(denied) {
                    class.meta_deny.push(denied.clone());
                }
            }
        }
    }
    // A class variable is ANCHORED once per ancestry, so a subclass
    // redeclaring one its superclass already anchors is a duplicate rather
    // than a fresh slot. The ancestry is walked here, after every class is
    // collected, because a superclass may be declared after its subclass.
    for index in 0..classes.len() {
        let mut ancestor = classes[index].superclass;
        let mut duplicate = None;
        while let Some(parent) = ancestor {
            if let Some(found) = classes[index].class_variables.iter().find(|variable| {
                classes[parent]
                    .class_variables
                    .iter()
                    .any(|anchored| anchored.name == variable.name)
            }) {
                duplicate = Some(found.name.clone());
                break;
            }
            ancestor = classes[parent].superclass;
        }
        classes[index].duplicate_class_variable = duplicate;
    }
    // `D-173` also settles a class conforming to TWO contracts that require one
    // selector with CONTRADICTING signatures: the static spine holds one
    // signature per selector, so NO single member can satisfy both. The clash
    // is in the CONFORMANCE rather than in any member, so it is decided from
    // the requirements alone, before any member is even looked at.
    for class in &mut classes {
        let conformances = class.contracts.clone();
        for (position, first) in conformances.iter().enumerate() {
            for second in &conformances[position + 1..] {
                for left in &contracts[*first].requirements {
                    for right in &contracts[*second].requirements {
                        if left.selector != right.selector {
                            continue;
                        }
                        // An UNANNOTATED position states nothing, so only two
                        // stated Types can contradict each other.
                        let parameters_differ = left.arity != right.arity
                            || left.parameter_types.iter().zip(&right.parameter_types).any(
                                |(left, right)| match (left, right) {
                                    (Some(left), Some(right)) => left != right,
                                    _ => false,
                                },
                            );
                        let returns_differ = match (&left.return_type, &right.return_type) {
                            (Some(left), Some(right)) => left != right,
                            _ => false,
                        };
                        if parameters_differ || returns_differ {
                            class.contract_signature_clash = true;
                        }
                    }
                }
            }
        }
    }
    for class in &classes {
        if class.reopens.is_empty() {
            continue;
        }
        for arguments in written_constructions(declarations, statements, &class.name) {
            for position in &class.non_nil_bounds {
                if matches!(arguments.get(*position), Some(TypeExpression::Name(name)) if name == "Nil")
                {
                    return Err(CompileError::new("closed construction bound"));
                }
            }
            for (position, contract) in &class.contract_bounds {
                let Some(argument) = arguments.get(*position) else {
                    continue;
                };
                let name = match argument {
                    TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => name,
                    _ => continue,
                };
                if !classes
                    .iter()
                    .find(|candidate| candidate.name == *name)
                    .is_some_and(|candidate| candidate.contracts.contains(contract))
                {
                    return Err(CompileError::new("closed construction bound"));
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
    let mut parents = Vec::new();
    for parent in &declaration.parents {
        // A GENERIC parent names the same contract as a bare one: the backend
        // interns one contract per definition rather than per construction, so
        // `extends P<Integer>` inherits `P`'s requirements.
        let (TypeExpression::Name(name) | TypeExpression::Generic { name, .. }) = parent else {
            return Err(CompileError::new("contract declaration form"));
        };
        let Some(parent) = contracts.iter().rfind(|known| known.name == *name) else {
            // `Iterable` and `Iterator` are KERNEL contracts rather than
            // program declarations, so a child extending one inherits nothing
            // this backend records - the reference runs the declaration, and a
            // program depending on those requirements fails on the collection
            // surface rather than here.
            if matches!(name.as_str(), "Iterable" | "Iterator" | "Comparable") {
                continue;
            }
            return Err(CompileError::new("contract parent unbound"));
        };
        parents.push(name.clone());
        parents.extend(parent.parents.iter().cloned());
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
            || matches!(method.impl_contract, Some(Some(_)))
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
            return_type: method.return_type.clone(),
        });
    }
    contracts.push(Contract {
        parents,
        name: declaration.name.clone(),
        requirements,
        meta_deny: declaration.meta_deny.clone(),
    });
    Ok(())
}

fn traversal_contracts() -> Vec<Contract> {
    vec![
        Contract {
            name: "Iterable".to_owned(),
            parents: Vec::new(),
            requirements: vec![ContractRequirement {
                selector: "iterator".to_owned(),
                arity: 0,
                return_type: Some(TypeExpression::Generic {
                    name: "Iterator".to_owned(),
                    arguments: vec![TypeExpression::Name("T".to_owned())],
                }),
                parameter_types: Vec::new(),
            }],
            meta_deny: Vec::new(),
        },
        Contract {
            name: "Iterator".to_owned(),
            parents: Vec::new(),
            requirements: vec![
                ContractRequirement {
                    selector: "next".to_owned(),
                    arity: 0,
                    return_type: Some(TypeExpression::Generic {
                        name: "Iteration".to_owned(),
                        arguments: vec![TypeExpression::Name("T".to_owned())],
                    }),
                    parameter_types: Vec::new(),
                },
                ContractRequirement {
                    selector: "close".to_owned(),
                    arity: 0,
                    return_type: Some(TypeExpression::Name("Nil".to_owned())),
                    parameter_types: Vec::new(),
                },
            ],
            meta_deny: Vec::new(),
        },
        Contract {
            name: "Iteration".to_owned(),
            parents: Vec::new(),
            requirements: Vec::new(),
            meta_deny: Vec::new(),
        },
    ]
}

fn collect_class<'a>(
    declarations: &'a [iris_syntax::Declaration],
    statements: &[Statement],
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
    // A `private` marker travels with the edge, since it grants the module
    // reach into the class's private methods.
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
        mixins.push((name.clone(), mixin.private_access));
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
    let mut non_nil_bounds = Vec::new();
    for constraint in &class.constraints {
        let named = match &constraint.bound {
            TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => Some(name),
            _ => None,
        };
        let Some(bound) = named else {
            continue;
        };
        // `NonNil` is not a CONTRACT, so it has no entry to look up - it names
        // the one argument it excludes instead, and is recorded by position.
        if bound == "NonNil" {
            if let Some(position) = class
                .parameters
                .iter()
                .position(|parameter| *parameter == constraint.parameter)
            {
                non_nil_bounds.push(position);
            }
            continue;
        }
        let Some(contract) = contracts.iter().rposition(|known| known.name == *bound) else {
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
    let mut shared_class_variables = Vec::new();
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
            // A class-level initializer that is NOT a literal is an ordinary
            // expression evaluated the first time the property is read, so it
            // needs a frame of its own rather than being flattened to nil.
            let literal = literal_value(initializer, "class-level stored property");
            let initializer_function = if literal.is_ok() {
                None
            } else {
                signatures.push(Signature {
                    declaration: None,
                    module: &class.name,
                    selector: name,
                    parameters: Vec::new(),
                    return_type: None,
                    body: &[],
                    // The frame takes the CLASS as its receiver: a
                    // receiverless signature is reachable as a bare module
                    // function, which would run the initializer on EVERY read
                    // instead of only the first.
                    receiver: true,
                    class_method: false,
                    private: false,
                    is_async: false,
                    constants: Vec::new(),
                    expression_body: Some(initializer),
                });
                Some(signatures.len() - 1)
            };
            let initializer = literal.unwrap_or(LiteralValue::Nil);
            // `IRIS-V1-TYPES-C064` puts a `shared class property` on the
            // UNAPPLIED generic definition, while a plain one belongs to each
            // closed CONSTRUCTION. The runtime keys class state by
            // `(ClassId, Selector)` and a generic class has one ClassId, so
            // each construction gets its own SELECTOR instead - one slot per
            // construction, without a class per construction. The bare name is
            // registered too, which is what keeps a non-generic class working.
            // On a GENERIC class the BARE name reaches no slot: the property
            // belongs to each construction, so `C.n` is a MessageNotFound
            // while `C<String>.n` answers. Registering the bare name too would
            // answer a value the language does not have there.
            if class.parameters.is_empty() || *shared {
                // A SHARED property on a generic class belongs to the
                // unapplied definition only, so a closed construction must not
                // reach it - answering the definition's value there gave
                // `C<String>.n` a slot the language does not give it.
                if *shared && !class.parameters.is_empty() {
                    shared_class_variables.push(name.clone());
                    // A construction still needs its OWN slot for a write to
                    // land in: `C<String>.n = 3` answers 3 while leaving the
                    // definition's `C.n` untouched, so the two must not share
                    // one slot.
                    for arguments in written_constructions(declarations, statements, &class.name) {
                        let construction =
                            crate::compile::lowering::Lowering::type_selector(&arguments)
                                .ok_or_else(|| CompileError::new("class property construction"))?;
                        class_variables.push(ClassVariable {
                            name: format!("{name}<{construction}>"),
                            mutable: true,
                            initializer: initializer.clone(),
                            initializer_function,
                        });
                    }
                }
                class_variables.push(ClassVariable {
                    name: name.clone(),
                    mutable: true,
                    initializer: initializer.clone(),
                    initializer_function,
                });
            } else {
                for arguments in written_constructions(declarations, statements, &class.name) {
                    let construction =
                        crate::compile::lowering::Lowering::type_selector(&arguments)
                            .ok_or_else(|| CompileError::new("class property construction"))?;
                    class_variables.push(ClassVariable {
                        name: format!("{name}<{construction}>"),
                        mutable: true,
                        initializer: initializer.clone(),
                        initializer_function,
                    });
                }
            }
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
            declaration: None,
            selector: name,
            parameters: Vec::new(),
            return_type: None,
            body: &[],
            receiver: true,
            class_method: false,
            private: false,
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
        let Some(contract) = contracts.iter().rposition(|known| known.name == *qualifier) else {
            return Err(CompileError::new("contract implementation undeclared"));
        };
        qualified_impls.push((contract, method.selector.clone(), function));
    }
    let private_methods = signatures[first_function..]
        .iter()
        .filter(|signature| signature.private)
        .map(|signature| signature.selector.to_owned())
        .collect();
    // `C024` forbids an OVERLOAD SET: one selector maps to at most one method
    // per revision, so a second declaration of a selector REPLACES the first
    // and must write `override` to say so. A declaration that does not is
    // recorded here and refused at load, where the class identity the failure
    // names exists.
    let mut override_required = Vec::new();
    let mut declared: Vec<(iris_syntax::MethodKind, &str)> = Vec::new();
    for statement in &class.body {
        let Statement::Method(method) = statement else {
            continue;
        };
        // A QUALIFIED `impl fun C::m()` lives in the contract's view rather
        // than the class's own table, so it replaces nothing there.
        if matches!(method.impl_contract, Some(Some(_))) {
            continue;
        }
        let seen = (method.kind, method.selector.as_str());
        if declared.contains(&seen) && !method.is_override {
            override_required.push(method.selector.clone());
        }
        declared.push(seen);
    }
    classes.push(Class {
        name: class.name.clone(),
        private_methods,
        override_required,
        contract_signature_clash: false,
        non_nil_bounds,
        shared_class_variables,
        duplicate_class_variable: None,
        meta_deny: class.meta_deny.clone(),
        generic: !class.parameters.is_empty(),
        superclass,
        methods,
        decorators: class
            .decorators
            .iter()
            .map(|decorator| {
                (
                    decorator.name.clone(),
                    decorator.arguments.iter().map(decorator_literal).collect(),
                )
            })
            .collect(),
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

fn decorator_literal(argument: &iris_syntax::Expression) -> String {
    match argument {
        iris_syntax::Expression::Literal(text) => text.clone(),
        iris_syntax::Expression::Symbol(name) => format!(":{name}"),
        _ => String::new(),
    }
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
    // A reopen's `where` constraint NARROWS what the class admits: `C067`
    // checks a bound at materialization, so a bound added by a reopen governs
    // every construction the program names - including one written before the
    // reopen, since the class has only one set of bounds. Dropping it let a
    // construction the language refuses stand.
    for constraint in &class.constraints {
        let (TypeExpression::Name(bound) | TypeExpression::Generic { name: bound, .. }) =
            &constraint.bound
        else {
            continue;
        };
        // A reopen RESTATES the header, so its own parameter list names the
        // same positions the declaration used.
        let Some(position) = class
            .parameters
            .iter()
            .position(|parameter| *parameter == constraint.parameter)
        else {
            continue;
        };
        if bound == "NonNil" {
            if !classes[target].non_nil_bounds.contains(&position) {
                classes[target].non_nil_bounds.push(position);
            }
            continue;
        }
        let Some(contract) = contracts.iter().rposition(|known| known.name == *bound) else {
            continue;
        };
        if !classes[target]
            .contract_bounds
            .contains(&(position, contract))
        {
            classes[target].contract_bounds.push((position, contract));
        }
    }
    for mixin in &class.mixins {
        let (TypeExpression::Name(name) | TypeExpression::Generic { name, .. }) = &mixin.target
        else {
            return Err(CompileError::new("class reopen header"));
        };
        classes[target]
            .mixins
            .push((name.clone(), mixin.private_access));
    }
    // A REOPEN replaces what the origin declared, so `C024` requires
    // `override` there too: the origin is a static fact and passes unchecked,
    // while a reopen is a meta operation on a class that already has the
    // selector. The check names the class the reopen targets.
    for statement in &class.body {
        let Statement::Method(method) = statement else {
            continue;
        };
        if matches!(method.impl_contract, Some(Some(_))) || method.is_override {
            continue;
        }
        let replaces = match method.kind {
            iris_syntax::MethodKind::Class => classes[target]
                .class_methods
                .iter()
                .any(|(name, _)| *name == method.selector),
            _ => classes[target]
                .methods
                .iter()
                .any(|(name, _)| *name == method.selector),
        };
        if replaces {
            classes[target]
                .override_required
                .push(method.selector.clone());
        }
    }
    // `D-173` puts the contract-visible SIGNATURE in the static spine, so a
    // REPLACEMENT whose return Type contradicts a declared requirement is an
    // incompatible member rather than a new one. An UNANNOTATED position
    // states nothing and is left alone. The refusal is recorded rather than
    // returned, because it names a class identity that exists only at load.
    for contract in &classes[target].contracts {
        for requirement in &contracts[*contract].requirements {
            let Some(required) = requirement.return_type.as_ref() else {
                continue;
            };
            let clashes = class.body.iter().any(|statement| match statement {
                Statement::Method(method) if method.selector == requirement.selector => method
                    .return_type
                    .as_ref()
                    .is_some_and(|actual| actual != required),
                _ => false,
            });
            if clashes {
                classes[target].contract_signature_clash = true;
            }
        }
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
                .rposition(|contract| contract.name == *name)
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
        // A class body may hold ordinary STATEMENTS, which run with `self`
        // bound to the class when the declaration is reached - that is how
        // `class A { if true { self.define_method(:x) { .. } } }` publishes a
        // method. They declare nothing, so they contribute no signature and
        // are lowered by the caller.
        if receiver
            && matches!(
                statement,
                Statement::If { .. } | Statement::Expression(_) | Statement::Match { .. }
            )
        {
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
            declaration: Some(method),
            selector: &method.selector,
            parameters,
            return_type: method.return_type.as_ref(),
            body,
            receiver,
            class_method: method.kind == iris_syntax::MethodKind::Class,
            // A QUALIFIED `impl fun C::m()` is reached only through the
            // contract's view, which consults it directly, so its visibility
            // governs nothing - marking it private would deny the class's own
            // method of that name.
            private: method.visibility == iris_syntax::Visibility::Private
                && !matches!(method.impl_contract, Some(Some(_))),
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
        initializer_function: None,
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
