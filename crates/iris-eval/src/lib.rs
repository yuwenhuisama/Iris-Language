//! Minimal literal evaluation.

mod source_method;
mod source_runtime;

use iris_lexer::{Literal, convert_literals};
use iris_parser::parse;
use iris_runtime::{BuiltinClass, Kernel, KernelError, NativeSelector, Value as RuntimeValue};
use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

/// Observable literal values supported by the Iris v1 grammar vectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// An arbitrary-precision integer represented in its canonical decimal form.
    Integer(String),
    /// Exact IEEE-754 binary32 bits.
    Float32Bits(u32),
    /// Exact IEEE-754 binary64 bits.
    Float64Bits(u64),
    /// A Unicode string.
    String(String),
    /// A literal array.
    Array(Vec<Value>),
}

/// Evaluation failure for a source form outside the literal-only evaluator.
#[derive(Clone, Debug, PartialEq)]
pub enum EvaluationError {
    /// Literal conversion emitted a stable lexical diagnostic.
    LexicalDiagnostic(&'static str),
    /// The source did not produce a literal value this evaluator can observe.
    UnsupportedConstruct,
    /// Source parsing rejected the requested expression.
    ParseDiagnostic,
    /// The runtime rejected a native send.
    Runtime(KernelError),
    /// The declaration publisher rejected a Class mutation.
    Class(iris_runtime::ClassError),
    /// Construction or dynamic instance dispatch failed.
    Construction(iris_runtime::ConstructionError),
    /// A source Method raised during execution.
    Execution(iris_runtime::ExecutionError),
    /// An Iris raise escaped the current source evaluation.
    Raised(RuntimeValue),
    /// Source code attempted to write an immutable lexical binding.
    ImmutableBinding,
    /// Evaluation exceeded its step budget and was abandoned.
    ///
    /// This is a HARNESS limit, not an Iris semantic. It exists so a vector
    /// that fails to terminate is reported as evidence instead of hanging the
    /// conformance suite indefinitely.
    StepBudgetExhausted,
    /// A mutation was attempted against a runtime-owned read-only collection.
    ///
    /// `IRIS-V1-CONTROL-V285` names this for `ExceptionContext.suppressed`,
    /// which `D-142` lets user code iterate and copy but never modify.
    ReadonlyMutation,
    /// A write was attempted against a read-only property.
    ///
    /// `IRIS-V1-CONTROL-V302A` names this for an `ExceptionContext` value,
    /// which `D-159` makes readable but never assignable.
    ReadonlyProperty,
    /// An unqualified name resolved to no binding.
    ///
    /// `IRIS-V1-CONTROL-C009` requires assignment to an absent ordinary local to
    /// fail as `NameError` and to create NO binding, which is what stops a typo
    /// from silently introducing a local.
    NameError,
    /// Source symbols are not yet representable as runtime Values.
    Symbol(String),
    /// A truthiness `to_bool` Method returned a value other than Bool.
    TypeContractError,
    /// A programmatic open targeted a Contract or a closed generic Class.
    ///
    /// `IRIS-V1-META-C033` forbids both targets, and `IRIS-V1-META-V439`
    /// requires the SAME `CLOSED_GENERIC_OPEN_FORBIDDEN` name the declarative
    /// spelling reports statically, so the two entry points agree.
    ClosedGenericOpenForbidden,
    /// Two direct imports replaced one member without authorization.
    ///
    /// `IRIS-V1-META-C049` requires the import-site `override` marker before a
    /// direct import may replace an already merged static member.
    ImportReplacementAuthorization,
    /// Raw ivar reflection was given a name that is not an instance ivar.
    ///
    /// `IRIS-V1-RUNTIME-C073` makes `@@name` a hierarchy binding cell rather
    /// than an instance ivar, and a name with no `@` sigil is an ordinary
    /// selector. `IRIS-V1-META-V362` names the error.
    InvalidInstanceVariableName,
    /// A lock selected two implementations of one package at one API major.
    ///
    /// `IRIS-V1-META-C006` requires an EXACT selection and one identity per
    /// `(package_id, api_major)`, so two entries unifying to that pair have no
    /// single Type identity. `IRIS-V1-META-V352` names the link failure.
    PackageVersionUnification,
    /// `same?` was applied to a Contract view.
    ///
    /// `IRIS-V1-TYPES-C050` makes Contract views immutable identity-LESS
    /// capability values, so an identity question about one has no answer to
    /// give and must raise rather than compare the underlying receiver.
    IdentityError,
    /// A `<=>` Method returned a value outside `Integer(-1|0|1)` and `nil`.
    ComparisonContractError,
    /// A send supplied the wrong number of arguments for the selected Method.
    ArgumentError,
    /// A binding-only destructuring context did not match its value.
    PatternMatchError,
    /// A bare `raise` occurred outside any catch dynamic extent.
    NoActiveExceptionError,
    /// A cause or suppressed edge would have formed a cycle.
    ExceptionChainError,
    /// A package's Module dependencies form a cycle.
    ///
    /// `IRIS-V1-META-C017` makes Module initialization an ACYCLIC deterministic
    /// DAG and requires a dependency or initialization cycle to be a compile or
    /// link error, so a package whose sources import each other never loads.
    ModuleInitializationCycleError,
    /// A raw ivar removal named a slot the receiver does not own.
    ///
    /// `IRIS-V1-META-C100` removes an EXISTING slot and returns its old value,
    /// and `IRIS-V1-META-V362` names the absent case.
    InstanceVariableNotFoundError,
    /// A `continue` is unwinding to start the next iteration of its target loop.
    ///
    /// `IRIS-V1-CONTROL-C043` gives `continue` NO value, so unlike `LoopBreak`
    /// it carries only the target label.
    LoopContinue(Option<String>),
    /// A `break` is unwinding to its target loop, carrying the loop result.
    ///
    /// `IRIS-V1-CONTROL-C043` makes `break expr` exit the target loop with
    /// `expr` as the LOOP result, so this travels as a control signal rather
    /// than an ordinary value and is consumed by the loop that catches it.
    LoopBreak(Option<String>, RuntimeValue),
    /// A `return` unwinding to its nearest callable boundary.
    ///
    /// `IRIS-V1-CONTROL-D-421` makes `return` inside a Closure end only THAT
    /// Closure invocation, so this is caught at the nearest Method or Closure
    /// boundary rather than propagating to an enclosing one.
    Return(RuntimeValue),
    /// An ordinary selector was absent and the default `method_missing` applied.
    MessageNotFound {
        receiver_class: String,
        selector: String,
    },
}

/// Evaluates a source expression by sending every supported operator through the runtime kernel.
pub fn evaluate(source: &str) -> Result<RuntimeValue, EvaluationError> {
    let parsed = parse(source);
    if !parsed.program_accepted {
        return Err(EvaluationError::ParseDiagnostic);
    }
    if !parsed.program.declarations.is_empty()
        || parsed
            .program
            .statements
            .iter()
            .any(source_runtime_statement)
    {
        return source_runtime::evaluate(&parsed.program, source);
    }
    let mut registry = iris_runtime::ClassRegistry::new();
    let kernel = Kernel::new(&mut registry).map_err(EvaluationError::Runtime)?;
    let mut evaluator = Evaluator { kernel, registry };
    let values = parsed
        .program
        .statements
        .iter()
        .map(|statement| evaluator.statement(statement))
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [] => Err(EvaluationError::UnsupportedConstruct),
        [value] => Ok(value.clone()),
        _ => Ok(RuntimeValue::Array(values)),
    }
}

/// Evaluates ordered per-package programs against ONE runtime.
///
/// `IRIS-V1-META-C003` lets a manifestless local script run with runtime-local
/// package identity only, and `D-431` makes a global's true identity
/// `(package_id, $name)`, unique within a package and separately instantiated
/// per runtime, with NO flat cross-package namespace and no auto-merge.
/// Observing that requires two packages sharing one runtime, which a single
/// program cannot express.
///
/// Each entry is a `(package_id, source)` pair. The programs run in order and
/// the LAST value is reported, so a later package can read what an earlier one
/// published through its own package identity.
pub fn evaluate_packages(programs: &[(String, String)]) -> Result<RuntimeValue, EvaluationError> {
    let mut evaluator = source_runtime::SourceEvaluator::new_in_package("")?;
    let mut last = RuntimeValue::Nil;
    for (package, source) in programs {
        let parsed = parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::ParseDiagnostic);
        }
        evaluator.enter_package(package, source);
        last = evaluator.program(&parsed.program)?;
    }
    Ok(last)
}

/// Loads one package's ordered source files and reports its initialized Modules.
///
/// `IRIS-V1-META-C017` initializes a package in manifest-declared source order,
/// and `IRIS-V1-META-C011` puts every executable statement inside a Module
/// body, so what a package load OBSERVES is which Modules were initialized and
/// in what order rather than a trailing expression value.
///
/// Each entry is a `(path, source)` pair in manifest order. A parse failure
/// aborts the load, which `IRIS-V1-META-C010` requires of an invalid package.
pub fn load_package(
    package_id: &str,
    sources: &[(String, String)],
) -> Result<Vec<String>, EvaluationError> {
    load_package_with_probe(package_id, sources, None).map(|(modules, _)| modules)
}

/// Loads a package and evaluates one probe expression against it afterwards.
///
/// A package source file is declarations only under `IRIS-V1-META-C011`, so a
/// row observing a Module member, such as `IRIS-V1-META-V342`'s
/// `M.answer()`, needs a send made AFTER the load rather than a trailing
/// statement inside the package.
pub fn load_package_with_probe(
    package_id: &str,
    sources: &[(String, String)],
    probe: Option<&str>,
) -> Result<(Vec<String>, Option<RuntimeValue>), EvaluationError> {
    load_package_at_major(package_id, 1, sources, probe)
}

/// Loads one package at a declared `api_major`.
///
/// `D-242` makes major-version contract identity part of a named Contract
/// Type's hash, so the manifest's major must reach the evaluator rather than
/// being assumed. V260 observes `pkg@1::C` and `pkg@2::C` hashing differently.
pub fn load_package_at_major(
    package_id: &str,
    api_major: u64,
    sources: &[(String, String)],
    probe: Option<&str>,
) -> Result<(Vec<String>, Option<RuntimeValue>), EvaluationError> {
    load_resolved_package(package_id, api_major, None, Vec::new(), sources, probe)
}

/// Loads one package together with the identity a lock file already resolved.
///
/// `IRIS-V1-META-C006` makes the dependency selection EXACT, so a recorded
/// selection is carried in rather than re-resolved, and `IRIS-V1-META-V420`
/// observes it without any resolver fetch occurring.
pub fn load_resolved_package(
    package_id: &str,
    api_major: u64,
    version: Option<String>,
    locked: Vec<(String, u64, String, String)>,
    sources: &[(String, String)],
    probe: Option<&str>,
) -> Result<(Vec<String>, Option<RuntimeValue>), EvaluationError> {
    let mut evaluator = source_runtime::SourceEvaluator::new_in_package(package_id)?;
    evaluator.enter_api_major(api_major);
    // C006 makes `(package_id, api_major)` one identity, so a lock selecting
    // two implementations of that pair cannot be unified and fails the LINK
    // before any Module body runs. V352 observes that no hidden identity is
    // created in its place.
    for (index, (name, major, ..)) in locked.iter().enumerate() {
        if locked
            .iter()
            .skip(index + 1)
            .any(|(other, other_major, ..)| other == name && other_major == major)
        {
            return Err(EvaluationError::PackageVersionUnification);
        }
    }
    evaluator.enter_package_resolution(version, locked);
    let mut initialized = Vec::new();
    // C017 makes Module initialization an acyclic deterministic DAG and makes a
    // cycle a LINK error, so the dependency graph is checked before any Module
    // body runs rather than after a half-initialized package is published.
    reject_import_cycles(sources)?;
    reject_unauthorized_replacements(sources)?;
    for (_, source) in sources {
        let parsed = parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::ParseDiagnostic);
        }
        evaluator.enter_package(package_id, source);
        // C011 keeps every executable statement inside a Module body, so a
        // package source file is DECLARATIONS ONLY and yields no program
        // value. That absence is the normal case here rather than a failure,
        // which is what V415 means by "no top-level executable statement".
        // The tolerance is limited to a file that HAS no statements, so an
        // unsupported construct inside one is still reported.
        let declarations_only = parsed.program.statements.is_empty();
        match evaluator.program(&parsed.program) {
            Ok(_) => {}
            Err(EvaluationError::UnsupportedConstruct) if declarations_only => {}
            Err(error) => return Err(error),
        }
        initialized.extend(declared_modules(&parsed.program));
    }
    let observed = match probe {
        Some(probe) => {
            let parsed = parse(probe);
            if !parsed.program_accepted {
                return Err(EvaluationError::ParseDiagnostic);
            }
            evaluator.enter_package(package_id, probe);
            Some(evaluator.program(&parsed.program)?)
        }
        None => None,
    };
    Ok((initialized, observed))
}

/// Loads ordered packages onto ONE runtime and probes the last one.
///
/// `IRIS-V1-META-C006` selects dependencies before initialization and
/// `IRIS-V1-META-C017` initializes a dependency before its dependent, so the
/// packages arrive dependencies-first and share one runtime. `D-431` keys a
/// global by `(package_id, $name)`, so each package's own declarations stay
/// distinct while its exports remain reachable to a consumer that imports them.
///
/// Each entry is `(package_id, sources)`. The probe runs under the LAST
/// package's identity, which is the consumer.
pub fn load_package_tree(
    packages: &[(String, Vec<(String, String)>)],
    probe: Option<&str>,
) -> Result<(Vec<String>, Option<RuntimeValue>), EvaluationError> {
    let Some((entry, _)) = packages.last() else {
        return Ok((Vec::new(), None));
    };
    let mut evaluator = source_runtime::SourceEvaluator::new_in_package(entry)?;
    let mut initialized = Vec::new();
    // C049 authorizes a replacement at the IMPORT site, so a second extension
    // contributed by a DIFFERENT package needs the marker exactly as one in the
    // same package does. Checking per package let a cross-package replacement
    // through unauthorized, which V348's tamper exposed.
    let across_tree: Vec<(String, String)> = packages
        .iter()
        .flat_map(|(_, sources)| sources.iter().cloned())
        .collect();
    reject_unauthorized_replacements(&across_tree)?;
    for (package_id, sources) in packages {
        // C017 makes Module initialization an acyclic deterministic DAG within
        // a package too, so each package's own graph is checked before any of
        // its Module bodies run.
        reject_import_cycles(sources)?;
        for (_, source) in sources {
            let parsed = parse(source);
            if !parsed.program_accepted {
                return Err(EvaluationError::ParseDiagnostic);
            }
            evaluator.enter_package(package_id, source);
            let declarations_only = parsed.program.statements.is_empty();
            match evaluator.program(&parsed.program) {
                Ok(_) => {}
                Err(EvaluationError::UnsupportedConstruct) if declarations_only => {}
                Err(error) => return Err(error),
            }
            initialized.extend(declared_modules(&parsed.program));
        }
    }
    let observed = match probe {
        Some(probe) => {
            let parsed = parse(probe);
            if !parsed.program_accepted {
                return Err(EvaluationError::ParseDiagnostic);
            }
            evaluator.enter_package(entry, probe);
            Some(evaluator.program(&parsed.program)?)
        }
        None => None,
    };
    Ok((initialized, observed))
}

/// Rejects a package whose source files import each other in a cycle.
///
/// `IRIS-V1-META-C017` makes Module initialization an ACYCLIC deterministic DAG
/// and requires a dependency or initialization cycle to be a compile or link
/// error. The graph is per SOURCE FILE, since that is what a package's manifest
/// orders and what an `import` in one file names in another.
/// Rejects two extensions replacing one member without import authorization.
///
/// `IRIS-V1-META-C049` requires import-site replacement authorization before a
/// direct import may replace an already merged static member, and `D-230`
/// authorizes only the replacements the source marked. `IRIS-V1-META-V349`
/// observes the link-phase diagnostic when two compatible extensions contribute
/// the same member and no import carries the `override` marker.
fn reject_unauthorized_replacements(sources: &[(String, String)]) -> Result<(), EvaluationError> {
    // `(class, selector)` pairs an earlier source already contributed. The
    // FIRST reopen of a member establishes it here; a later one replaces it.
    let mut established: Vec<(String, String)> = Vec::new();
    for (_, source) in sources {
        let parsed = parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::ParseDiagnostic);
        }
        // Authorization is granted at THIS source's own import site, so a
        // marker in one file cannot authorize another file's replacement.
        let authorized = parsed.program.declarations.iter().any(|declaration| {
            matches!(
                declaration,
                iris_syntax::Declaration::Import(value) if value.replacement_authorized
            )
        });
        let mut contributed: Vec<(String, String)> = Vec::new();
        for declaration in &parsed.program.declarations {
            let iris_syntax::Declaration::Class(value) = declaration else {
                continue;
            };
            if !value.reopen {
                continue;
            }
            for statement in &value.body {
                if let iris_syntax::Statement::Method(method) = statement {
                    let entry = (value.name.clone(), method.selector.clone());
                    if established.contains(&entry) && !authorized {
                        return Err(EvaluationError::ImportReplacementAuthorization);
                    }
                    contributed.push(entry);
                }
            }
        }
        established.extend(contributed);
    }
    Ok(())
}

fn reject_import_cycles(sources: &[(String, String)]) -> Result<(), EvaluationError> {
    let mut declares: Vec<(usize, Vec<String>)> = Vec::new();
    let mut imports: Vec<(usize, Vec<String>)> = Vec::new();
    for (index, (_, source)) in sources.iter().enumerate() {
        let parsed = parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::ParseDiagnostic);
        }
        declares.push((index, declared_modules(&parsed.program)));
        imports.push((index, imported_targets(&parsed.program)));
    }
    // An edge runs from the file that imports a name to the file declaring it.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (from, targets) in &imports {
        for target in targets {
            for (to, names) in &declares {
                if from != to && names.iter().any(|name| name == target) {
                    edges.push((*from, *to));
                }
            }
        }
    }
    // A cycle exists when repeatedly removing a file with no outgoing edge
    // cannot empty the graph.
    let mut remaining: Vec<usize> = (0..sources.len()).collect();
    loop {
        let Some(position) = remaining.iter().position(|file| {
            !edges
                .iter()
                .any(|(from, to)| from == file && remaining.contains(to))
        }) else {
            return Err(EvaluationError::ModuleInitializationCycleError);
        };
        remaining.remove(position);
        if remaining.is_empty() {
            return Ok(());
        }
    }
}

/// Names the Modules a program's imports target.
fn imported_targets(program: &iris_syntax::Program) -> Vec<String> {
    program
        .declarations
        .iter()
        .filter_map(|entry| match entry {
            iris_syntax::Declaration::Import(import) => Some(import.target.clone()),
            _ => None,
        })
        .collect()
}

/// Names the Modules a program declares, in source order.
fn declared_modules(program: &iris_syntax::Program) -> Vec<String> {
    program
        .declarations
        .iter()
        .filter_map(|entry| match entry {
            iris_syntax::Declaration::Module(module) => Some(module.name.clone()),
            _ => None,
        })
        .collect()
}

/// Evaluates source and reports whether a named Class was published before failure.
pub fn evaluate_with_class_publication(
    source: &str,
    class_name: &str,
) -> (Result<RuntimeValue, EvaluationError>, bool) {
    let parsed = parse(source);
    if !parsed.program_accepted {
        return (Err(EvaluationError::ParseDiagnostic), false);
    }
    let mut evaluator =
        match source_runtime::SourceEvaluator::new_in_package(source_runtime::LOCAL_PACKAGE) {
            Ok(evaluator) => evaluator,
            Err(error) => return (Err(error), false),
        };
    let outcome = evaluator.program(&parsed.program);
    // D-212 leaves a Module whose initializer raised with status
    // `not_published`, which V239 observes the same way a failed Class
    // candidate is observed, so the name is looked up in either registry.
    let published = matches!(evaluator.class_name(class_name), Ok(Some(_)))
        || evaluator.module_published(class_name);
    (outcome, published)
}

/// Evaluates source and reports whether a named Class responds to a selector.
///
/// `IRIS-V1-META-C022` publishes NOTHING from a failed candidate. A failure
/// halts the program, so a member the failed transaction staged cannot be read
/// from Iris source afterwards; this observes the published revision directly.
pub fn evaluate_with_member_probe(
    source: &str,
    class_name: &str,
    selector: &str,
) -> (Result<RuntimeValue, EvaluationError>, bool) {
    let parsed = parse(source);
    if !parsed.program_accepted {
        return (Err(EvaluationError::ParseDiagnostic), false);
    }
    let mut evaluator =
        match source_runtime::SourceEvaluator::new_in_package(source_runtime::LOCAL_PACKAGE) {
            Ok(evaluator) => evaluator,
            Err(error) => return (Err(error), false),
        };
    let outcome = evaluator.program(&parsed.program);
    let responds = evaluator.class_responds_to(class_name, selector);
    (outcome, responds)
}

struct Evaluator {
    kernel: Kernel,
    registry: iris_runtime::ClassRegistry,
}

enum Evaluated {
    Value(RuntimeValue),
    Member(RuntimeValue, String),
    UnresolvedClass(String),
    UnresolvedClassMember {
        class_name: String,
        selector: String,
    },
}

impl Evaluator {
    fn statement(&mut self, statement: &Statement) -> Result<RuntimeValue, EvaluationError> {
        match statement {
            Statement::GlobalBinding { .. }
            | Statement::SharedBinding { .. }
            | Statement::Binding { .. }
            | Statement::DeferredBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_) => Err(EvaluationError::UnsupportedConstruct),
            Statement::Expression(expression) => self
                .expression(expression)
                .and_then(|value| self.value(value)),
            Statement::If { .. }
            | Statement::Return(_)
            | Statement::Break { .. }
            | Statement::Continue(_)
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Match { .. } => Err(EvaluationError::UnsupportedConstruct),
            Statement::Raise(_) | Statement::Try { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
        }
    }

    fn expression(&mut self, expression: &Expression) -> Result<Evaluated, EvaluationError> {
        match expression {
            // A keyword argument is meaningless outside a call the literal
            // evaluator cannot make, so it is routed rather than evaluated.
            Expression::KeywordArgument { .. }
            | Expression::Index { .. }
            | Expression::Try { .. }
            // A closed generic construction and a reified Type both need the
            // Class registry the literal evaluator does not have.
            | Expression::GlobalVar(_)
        | Expression::ClosedGeneric { .. }
            | Expression::ReifiedType(_)
            | Expression::While { .. } => Err(EvaluationError::UnsupportedConstruct),
            Expression::Name(name) => self.name(name),
            Expression::Literal(source) => self.literal(source).map(Evaluated::Value),
            Expression::Array(expressions) => expressions
                .iter()
                .map(|expression| {
                    self.expression(expression)
                        .and_then(|value| self.value(value))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(RuntimeValue::Array)
                .map(Evaluated::Value),
            Expression::Grouped(expression) => self.expression(expression),
            Expression::Member { receiver, selector } => match self.expression(receiver)? {
                Evaluated::Value(receiver) => Ok(Evaluated::Member(receiver, selector.clone())),
                Evaluated::Member(receiver, previous_selector) => {
                    let receiver = self.value(Evaluated::Member(receiver, previous_selector))?;
                    Ok(Evaluated::Member(receiver, selector.clone()))
                }
                Evaluated::UnresolvedClass(class_name) => Ok(Evaluated::UnresolvedClassMember {
                    class_name,
                    selector: selector.clone(),
                }),
                Evaluated::UnresolvedClassMember { .. } => {
                    Err(EvaluationError::UnsupportedConstruct)
                }
            },
            Expression::Call {
            callee,
            arguments,
            ..
        } => match self.expression(callee)? {
                Evaluated::Member(receiver, selector) => {
                    let arguments = self.arguments(arguments, None)?;
                    self.send(receiver, &selector, &arguments)
                }
                Evaluated::Value(RuntimeValue::Class(class)) => {
                    let arguments = self.arguments(arguments, Some(class))?;
                    self.kernel
                        .construct(class, &arguments)
                        .map(Evaluated::Value)
                        .map_err(EvaluationError::Runtime)
                }
                Evaluated::UnresolvedClassMember {
                    class_name,
                    selector,
                } => Err(EvaluationError::MessageNotFound {
                    receiver_class: class_name,
                    selector,
                }),
                Evaluated::Value(_) | Evaluated::UnresolvedClass(_) => {
                    Err(EvaluationError::UnsupportedConstruct)
                }
            },
            Expression::Unary { operator, operand } => {
                let operand = self.expression(operand)?;
                let operand = self.value(operand)?;
                let selector = match operator {
                    UnaryOperator::Negate => NativeSelector::Negate,
                    UnaryOperator::BitwiseNot => NativeSelector::BitwiseNot,
                    UnaryOperator::Not => {
                        return self
                            .truthy(operand)
                            .map(|value| Evaluated::Value(RuntimeValue::Bool(!value)));
                    }
                    UnaryOperator::Plus => {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                };
                self.kernel
                    .send(&self.registry, operand, selector, &[])
                    .map(Evaluated::Value)
                    .map_err(EvaluationError::Runtime)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.expression(left)?;
                let left = self.value(left)?;
                if matches!(operator, BinaryOperator::LogicalAnd) {
                    return if self.truthy(left.clone())? {
                        self.expression(right)
                    } else {
                        Ok(Evaluated::Value(left))
                    };
                }
                if matches!(operator, BinaryOperator::LogicalOr) {
                    return if self.truthy(left.clone())? {
                        Ok(Evaluated::Value(left))
                    } else {
                        self.expression(right)
                    };
                }
                let right = self.expression(right)?;
                let right = self.value(right)?;
                match operator {
                    BinaryOperator::Identity => Kernel::same_identity(&left, &right)
                        .map(RuntimeValue::Bool)
                        .map(Evaluated::Value)
                        .map_err(EvaluationError::Runtime),
                    _ => self.binary(left, operator, right),
                }
            }
            Expression::Symbol(_)
            | Expression::RawIvar(_)
            | Expression::ClassVar(_)
            | Expression::ContractView { .. }
            | Expression::Closure { .. }
            | Expression::Hash(_)
            | Expression::Assignment { .. }
            | Expression::If { .. } => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn name(&self, name: &str) -> Result<Evaluated, EvaluationError> {
        let value = match name {
            "nil" => RuntimeValue::Nil,
            "true" => RuntimeValue::Bool(true),
            "false" => RuntimeValue::Bool(false),
            "Integer" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Integer)
                    .map_err(EvaluationError::Runtime)?,
            ),
            "String" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::String)
                    .map_err(EvaluationError::Runtime)?,
            ),
            "Float32" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Float32)
                    .map_err(EvaluationError::Runtime)?,
            ),
            "Float64" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Float64)
                    .map_err(EvaluationError::Runtime)?,
            ),
            _ if name.chars().next().is_some_and(char::is_uppercase) => {
                return Ok(Evaluated::UnresolvedClass(name.into()));
            }
            // IRIS-V1-CONTROL-C011 makes a non-call unresolved bare name a
            // `NameError`, and this evaluator has no bindings at all, so any
            // lowercase name reaching here is unresolved.
            _ => return Err(EvaluationError::NameError),
        };
        Ok(Evaluated::Value(value))
    }

    fn arguments(
        &mut self,
        arguments: &[Expression],
        constructor: Option<iris_runtime::ClassId>,
    ) -> Result<Vec<RuntimeValue>, EvaluationError> {
        let float64 = self
            .kernel
            .class(BuiltinClass::Float64)
            .map_err(EvaluationError::Runtime)?;
        arguments
            .iter()
            .map(|argument| {
                if constructor == Some(float64)
                    && matches!(argument, Expression::Unary { operator: UnaryOperator::Negate, operand } if matches!(operand.as_ref(), Expression::Name(name) if name == "Infinity"))
                {
                    Ok(RuntimeValue::Float64(f64::NEG_INFINITY))
                } else {
                    self.expression(argument).and_then(|value| self.value(value))
                }
            })
            .collect()
    }

    fn literal(&self, source: &str) -> Result<RuntimeValue, EvaluationError> {
        match source {
            "nil" => return Ok(RuntimeValue::Nil),
            "true" => return Ok(RuntimeValue::Bool(true)),
            "false" => return Ok(RuntimeValue::Bool(false)),
            _ => {}
        }
        match evaluate_literals(source)? {
            Value::Integer(value) => value
                .parse()
                .map(RuntimeValue::Integer)
                .map_err(|_| EvaluationError::UnsupportedConstruct),
            Value::Float32Bits(bits) => Ok(RuntimeValue::Float32(f32::from_bits(bits))),
            Value::Float64Bits(bits) => Ok(RuntimeValue::Float64(f64::from_bits(bits))),
            // IRIS-V1-COLLECTIONS-C041 makes a String an immutable sequence of
            // Unicode scalar values, already validated and unescaped here.
            Value::String(value) => Ok(RuntimeValue::Text(value)),
            Value::Array(_) => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn send(
        &self,
        receiver: RuntimeValue,
        selector: &str,
        arguments: &[RuntimeValue],
    ) -> Result<Evaluated, EvaluationError> {
        let selector = NativeSelector::from_source(selector).ok_or_else(|| {
            EvaluationError::MessageNotFound {
                receiver_class: receiver_class_name(&receiver).into(),
                selector: selector.into(),
            }
        })?;
        self.kernel
            .send(&self.registry, receiver, selector, arguments)
            .map(Evaluated::Value)
            .map_err(EvaluationError::Runtime)
    }

    fn binary(
        &self,
        left: RuntimeValue,
        operator: &BinaryOperator,
        right: RuntimeValue,
    ) -> Result<Evaluated, EvaluationError> {
        let selector = match operator {
            BinaryOperator::Add => NativeSelector::Add,
            BinaryOperator::Subtract => NativeSelector::Subtract,
            BinaryOperator::Multiply => NativeSelector::Multiply,
            BinaryOperator::Divide => NativeSelector::Divide,
            BinaryOperator::Power => NativeSelector::Power,
            BinaryOperator::ShiftLeft => NativeSelector::ShiftLeft,
            BinaryOperator::ShiftRight => NativeSelector::ShiftRight,
            BinaryOperator::Equal => NativeSelector::Equal,
            BinaryOperator::NotEqual => NativeSelector::NotEqual,
            BinaryOperator::Less => NativeSelector::Less,
            BinaryOperator::LessEqual => NativeSelector::LessEqual,
            BinaryOperator::Greater => NativeSelector::Greater,
            BinaryOperator::GreaterEqual => NativeSelector::GreaterEqual,
            BinaryOperator::Compare => NativeSelector::Compare,
            BinaryOperator::NamedInfix { selector } => NativeSelector::from_source(selector)
                .ok_or(EvaluationError::UnsupportedConstruct)?,
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        self.kernel
            .send(&self.registry, left, selector, &[right])
            .map(Evaluated::Value)
            .map_err(EvaluationError::Runtime)
    }

    fn truthy(&self, value: RuntimeValue) -> Result<bool, EvaluationError> {
        match self
            .kernel
            .send(&self.registry, value, NativeSelector::ToBool, &[])
            .map_err(EvaluationError::Runtime)?
        {
            RuntimeValue::Bool(value) => Ok(value),
            RuntimeValue::Nil
            | RuntimeValue::Integer(_)
            | RuntimeValue::Float32(_)
            | RuntimeValue::Float64(_)
            | RuntimeValue::Array(_)
            | RuntimeValue::Hash(_)
            | RuntimeValue::Text(_)
            | RuntimeValue::Symbol(_)
            | RuntimeValue::Class(_)
            | RuntimeValue::Type(..)
            | RuntimeValue::ComposedType(_)
            | RuntimeValue::Contract(_)
            | RuntimeValue::Closure(_)
            | RuntimeValue::KeywordArgument(_, _)
            | RuntimeValue::IterationYield(_)
            | RuntimeValue::ReadonlyArray(_)
            | RuntimeValue::SourceLocation(..)
            | RuntimeValue::StackFrame(..)
            | RuntimeValue::RaiseSite(_)
            | RuntimeValue::ArrayIterator(_)
            | RuntimeValue::IterationDone
            | RuntimeValue::Transformation { .. }
            | RuntimeValue::ExceptionContext(..)
            | RuntimeValue::ContractView(_, _)
            | RuntimeValue::Object(_)
            | RuntimeValue::BoundMethod(_)
            | RuntimeValue::Method(_) => Err(EvaluationError::TypeContractError),
        }
    }

    fn value(&self, value: Evaluated) -> Result<RuntimeValue, EvaluationError> {
        match value {
            Evaluated::Value(value) => Ok(value),
            Evaluated::Member(receiver, selector) => self
                .send(receiver, &selector, &[])
                .and_then(|value| self.value(value)),
            // IRIS-V1-CONTROL-C011 makes an unresolved bare name a `NameError`
            // regardless of spelling. Reporting `UnsupportedConstruct` for an
            // UPPERCASE name conflated "this implementation cannot do it" with
            // "the program named a Class that does not exist".
            Evaluated::UnresolvedClass(_) => Err(EvaluationError::NameError),
            Evaluated::UnresolvedClassMember { .. } => Err(EvaluationError::UnsupportedConstruct),
        }
    }
}

fn receiver_class_name(value: &RuntimeValue) -> &'static str {
    match value {
        RuntimeValue::Nil => "Nil",
        RuntimeValue::Bool(_) => "Bool",
        RuntimeValue::Integer(_) => "Integer",
        RuntimeValue::Float32(_) => "Float32",
        RuntimeValue::Float64(_) => "Float64",
        RuntimeValue::Array(_) => "Array",
        RuntimeValue::Hash(_) => "Hash",
        RuntimeValue::ReadonlyArray(_) => "ReadonlyArray",
        RuntimeValue::SourceLocation(..) => "SourceLocation",
        RuntimeValue::StackFrame(..) => "StackFrame",
        RuntimeValue::RaiseSite(_) => "RaiseSite",
        RuntimeValue::Text(_) => "String",
        RuntimeValue::Symbol(_) => "Symbol",
        RuntimeValue::Class(_) => "Class",
        RuntimeValue::Type(..) | RuntimeValue::ComposedType(_) => "Type",
        RuntimeValue::Contract(_) => "Contract",
        RuntimeValue::Closure(_) => "Closure",
        RuntimeValue::KeywordArgument(_, _) | RuntimeValue::IterationYield(_) => "Iteration",
        RuntimeValue::ArrayIterator(..) | RuntimeValue::IterationDone => "Iteration",
        RuntimeValue::ExceptionContext(..) => "ExceptionContext",
        RuntimeValue::ContractView(_, _) => "ContractView",
        RuntimeValue::Object(_) => "Object",
        RuntimeValue::BoundMethod(_) => "BoundMethod",
        RuntimeValue::Method(_) => "Method",
        RuntimeValue::Transformation { .. } => "Transformation",
    }
}

fn source_runtime_statement(statement: &Statement) -> bool {
    match statement {
        Statement::GlobalBinding { .. }
        | Statement::SharedBinding { .. }
        | Statement::Binding { .. }
        | Statement::DeferredBinding { .. } => true,
        Statement::Expression(expression) => source_runtime_expression(expression),
        Statement::If { .. } => true,
        // A loop needs the source runtime: the literal evaluator has no heap and
        // no statement sequencing. Its BODY is checked too, since a `break`
        // there is what carries the loop result.
        Statement::While { .. }
        | Statement::For { .. }
        | Statement::Break { .. }
        | Statement::Continue(_)
        | Statement::Match { .. } => true,
        Statement::StoredProperty { .. } | Statement::Method(_) | Statement::Return(_) => false,
        Statement::Raise(_) | Statement::Try { .. } => true,
    }
}

/// Reports whether a callee is `Object.new`, which allocates an ordinary instance.
///
/// The literal evaluator has no heap, so construction of the
/// `IRIS-V1-RUNTIME-C005` root Class must route to the source runtime even when
/// the program is a single expression and would otherwise stay on the literal path.
fn constructs_root_object(callee: &Expression) -> bool {
    matches!(
        callee,
        Expression::Member { receiver, selector }
            if selector == "new"
                && matches!(receiver.as_ref(), Expression::Name(name) if name == "Object")
    )
}

/// Reports whether an expression names the `Iteration` results of C013.
///
/// They are value constructors rather than Class sends, and the literal
/// evaluator cannot build them, so they route to the source runtime.
fn builds_iteration(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Member { receiver, .. }
            if matches!(receiver.as_ref(), Expression::Name(name) if name == "Iteration")
    )
}

fn source_runtime_expression(expression: &Expression) -> bool {
    if builds_iteration(expression) {
        return true;
    }
    match expression {
        // A Closure needs the heap the literal evaluator does not have.
        // A keyword argument binds by name, which only the source runtime does.
        Expression::Closure { .. }
        | Expression::Hash(_)
        | Expression::If { .. }
        | Expression::KeywordArgument { .. }
        | Expression::Index { .. }
        | Expression::Try { .. }
        | Expression::ClosedGeneric { .. }
        | Expression::ReifiedType(_)
        // A declared global lives in the source runtime's cell table.
        | Expression::GlobalVar(_)
        | Expression::While { .. } => true,
        Expression::Array(values) => values.iter().any(source_runtime_expression),
        Expression::Member { receiver, .. }
        | Expression::ContractView { receiver, .. }
        | Expression::Grouped(receiver)
        | Expression::Unary {
            operand: receiver, ..
        } => source_runtime_expression(receiver),
        Expression::Call {
            callee,
            arguments,
            ..
        } => {
            builds_iteration(callee)
                || constructs_root_object(callee)
                || source_runtime_expression(callee)
                || arguments.iter().any(source_runtime_expression)
        }
        // An assignment needs the binding environment to decide whether the
        // target exists at all, which C009 makes a NameError when it does not.
        Expression::Assignment { .. } => true,
        Expression::Binary { left, right, .. } => {
            source_runtime_expression(left) || source_runtime_expression(right)
        }
        Expression::Name(_)
        | Expression::Literal(_)
        | Expression::Symbol(_)
        | Expression::RawIvar(_)
        | Expression::ClassVar(_) => false,
    }
}

/// Evaluates source containing only lexer-converted literal values.
///
/// # Errors
/// Returns the lexer diagnostic when conversion rejects a literal, or
/// [`EvaluationError::UnsupportedConstruct`] when no literal value is present.
pub fn evaluate_literals(source: &str) -> Result<Value, EvaluationError> {
    let conversion = convert_literals(source);
    if let Some(&diagnostic) = conversion.diagnostics().first() {
        return Err(EvaluationError::LexicalDiagnostic(diagnostic));
    }

    let values = conversion
        .values()
        .iter()
        .cloned()
        .map(Value::from)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [] => Err(EvaluationError::UnsupportedConstruct),
        [value] => Ok(value.clone()),
        _ => Ok(Value::Array(values)),
    }
}

#[cfg(test)]
mod evaluator_bridge_tests {
    use iris_runtime::{MethodBody, NativeSelector, Value as RuntimeValue, Visibility};

    use super::evaluate;

    #[test]
    fn evaluates_v039_floor_division_from_source() {
        // Given
        let source = "[-5 div 2, 5 div -2, -5 div -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer((-3_i8).into()),
                RuntimeValue::Integer((-3_i8).into()),
                RuntimeValue::Integer(2_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v040_modulo_from_source() {
        // Given
        let source = "[-5 mod 2, 5 mod -2, -5 mod -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-1_i8).into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v019_singleton_identity_from_source() {
        // Given
        let source = "[nil same? nil, true same? true, false same? false]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![RuntimeValue::Bool(true); 3]))
        );
    }

    #[test]
    fn evaluates_v046_integer_division_from_source() {
        // Given
        let source = "Integer(5) / Integer(2)";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Float64(2.5)));
    }

    #[test]
    fn evaluates_v036_signed_zero_equality_while_preserving_bits() {
        // Given
        let source =
            "Float64.from_bits(0x0000000000000000) == Float64.from_bits(0x8000000000000000)";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn evaluates_v036_signed_zero_hash_equality_through_ordinary_member_sends() {
        // Given
        let source = "Float64.from_bits(0x0000000000000000).hash == Float64.from_bits(0x8000000000000000).hash";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn equal_numbers_have_equal_public_hashes_through_ordinary_member_sends() {
        // Given
        let source = "Float64(1).hash == Float64.from_bits(0x3ff0000000000000).hash";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn named_infix_and_member_send_have_the_same_result() {
        // Given
        let infix = "5 div 2";
        let member = "Integer(5).div(2)";

        // When
        let infix_result = evaluate(infix);
        let member_result = evaluate(member);

        // Then
        assert_eq!(infix_result, member_result);
        assert_eq!(infix_result, Ok(RuntimeValue::Integer(2_u8.into())));
    }

    #[test]
    fn executes_declared_method_through_member_and_named_infix_sends() {
        // Given
        let source = "class A { public fun scale(value: Integer) -> Integer { value * 2 } }; let a = A.new(); a.scale(3); a scale 3";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(6_u8.into()),
                RuntimeValue::Integer(6_u8.into()),
            ]))
        );
    }

    #[test]
    fn integer_division_by_zero_is_a_typed_runtime_error() {
        // Given
        let source = "1 div 0";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(result, Err(super::EvaluationError::Runtime(_))));
    }

    #[test]
    fn identity_primitive_bypasses_replaced_comparison_slots()
    -> Result<(), iris_runtime::KernelError> {
        // Given
        let mut registry = iris_runtime::ClassRegistry::new();
        let kernel = iris_runtime::Kernel::new(&mut registry)?;
        let bool_class = kernel.class(iris_runtime::BuiltinClass::Bool)?;
        registry.publish_method(
            bool_class,
            NativeSelector::Equal.id(),
            MethodBody::new(1),
            Visibility::Public,
        )?;

        // When
        let identity = iris_runtime::Kernel::same_identity(
            &RuntimeValue::Bool(true),
            &RuntimeValue::Bool(true),
        )?;

        // Then
        assert!(identity);
        Ok(())
    }

    #[test]
    fn evaluates_addition_at_the_float32_receiver_width() {
        // Given
        let source = "16777217 + 0.0f32";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Float32(16_777_216.0)));
    }

    #[test]
    fn evaluates_v042_negative_float_power_as_nan() {
        // Given
        let source = "(-2.0f64) ** 0.5f64";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(result, Ok(RuntimeValue::Float64(value)) if value.is_nan()));
    }

    #[test]
    fn evaluates_integer_bitwise_not_and_right_shift_from_source() {
        // Given
        let source = "[~0, -3 >> 1, 8 << -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-2_i8).into()),
                RuntimeValue::Integer(2_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v047_infinity_member_chain_as_width_preserving_nan() {
        // Given
        let source = "Float64.infinity.mul_add(0, 1); Float32.infinity.mul_add(0, 1)";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float64(value), RuntimeValue::Float32(other)] if value.is_nan() && other.is_nan())
        ));
    }

    #[test]
    fn evaluates_v059_nan_equality_and_less_than() {
        // Given
        let equality = "Float64.nan == Float64.nan";
        let less_than = "Float64.nan < 1.0";
        let comparison = "Float64.nan <=> 1.0";

        // When
        let equality_result = evaluate(equality);
        let less_than_result = evaluate(less_than);
        let comparison_result = evaluate(comparison);

        // Then
        assert_eq!(equality_result, Ok(RuntimeValue::Bool(false)));
        assert_eq!(less_than_result, Ok(RuntimeValue::Bool(false)));
        assert_eq!(comparison_result, Ok(RuntimeValue::Nil));
    }

    #[test]
    fn evaluates_v063_large_and_negative_shift_counts() {
        // Given
        let source = "[1 >> 1000000, -1 >> 1000000, -3 >> 1000000, 1 << -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(0_u8.into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer(0_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v065_class_getters_at_their_declared_widths() {
        // Given
        let source = "Float32.nan; Float64.infinity; -Float64.infinity";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float32(nan), RuntimeValue::Float64(infinity), RuntimeValue::Float64(negative_infinity)] if nan.is_nan() && infinity.is_infinite() && infinity.is_sign_positive() && negative_infinity.is_infinite() && negative_infinity.is_sign_negative())
        ));
    }

    #[test]
    fn rejects_v065_bare_special_value_names() {
        // Given
        let names = ["nan", "inf"];

        // When
        let results = names.map(evaluate);

        // Then `IRIS-V1-RUNTIME-V065` calls these ABSENT names, and
        // `IRIS-V1-CONTROL-C011` makes an unresolved bare name a `NameError`.
        assert!(
            results
                .into_iter()
                .all(|result| matches!(result, Err(super::EvaluationError::NameError)))
        );
    }

    #[test]
    fn evaluates_v104_special_value_arithmetic_as_width_preserving_nan() {
        // Given
        let source = "Float32.nan + 1.0f32; Float64.infinity - Float64.infinity";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float32(value), RuntimeValue::Float64(other)] if value.is_nan() && other.is_nan())
        ));
    }

    #[test]
    fn evaluates_v069_out_of_range_float_bit_patterns_as_typed_errors() {
        // Given
        let sources = [
            "Float64.from_bits(-1)",
            "Float32.from_bits(2 ** 32)",
            "Float64.from_bits(2 ** 64)",
        ];

        // When
        let results = sources.map(evaluate);

        // Then
        assert!(results.into_iter().all(|result| matches!(
            result,
            Err(super::EvaluationError::Runtime(
                iris_runtime::KernelError::Numeric(iris_runtime::NumericError::Range)
            ))
        )));
    }
}

#[cfg(test)]
mod runner_gap_tests;

#[cfg(test)]
mod operator_super_tests;

#[cfg(test)]
mod module_composition_tests;

#[cfg(test)]
mod builtin_protocol_tests;

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Integer(value) => Self::Integer(value),
            Literal::Float32(value) => Self::Float32Bits(value.to_bits()),
            Literal::Float64(value) => Self::Float64Bits(value.to_bits()),
            Literal::String(value) => Self::String(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EvaluationError, Value, evaluate_literals};

    #[test]
    fn evaluates_v152_hexadecimal_float_to_exact_float64_bits() -> Result<(), EvaluationError> {
        // Given
        let source = "0x1.fp3";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Float64Bits(0x402f_0000_0000_0000));
        Ok(())
    }

    #[test]
    fn evaluates_v151_float32_to_exact_bits() -> Result<(), EvaluationError> {
        // Given
        let source = "1e+3f32";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Float32Bits(0x447a_0000));
        Ok(())
    }

    #[test]
    fn preserves_integers_beyond_u64_through_evaluation() -> Result<(), EvaluationError> {
        // Given
        let source = "18446744073709551616";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Integer("18446744073709551616".into()));
        Ok(())
    }

    #[test]
    fn evaluates_v152_array_with_exact_value_types() -> Result<(), EvaluationError> {
        // Given
        let source = "[0x1.fp3, 0x1p0, 0x1.p0, 0x.8p0, 0x1e3]";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(
            value,
            Value::Array(vec![
                Value::Float64Bits(0x402f_0000_0000_0000),
                Value::Float64Bits(0x3ff0_0000_0000_0000),
                Value::Float64Bits(0x3ff0_0000_0000_0000),
                Value::Float64Bits(0x3fe0_0000_0000_0000),
                Value::Integer("483".into()),
            ])
        );
        Ok(())
    }

    #[test]
    fn evaluates_v179_adjacent_strings_as_one_string() -> Result<(), EvaluationError> {
        // Given
        let source = "\"a\" 'b' r\"c\" \"\"\"d\"\"\"";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::String("abcd".into()));
        Ok(())
    }

    #[test]
    fn evaluates_v173_interpolated_and_non_interpolated_string_forms() -> Result<(), EvaluationError>
    {
        // Given
        let source = "[\"a\\n${1 + 1}\", 'a\\n${x}', r#\"a\\n${x}\"#, \"\"\"\n  a\n  \"\"\"]";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(
            value,
            Value::Array(vec![
                Value::String("a\n2".into()),
                Value::String("a\n${x}".into()),
                Value::String(r"a\n${x}".into()),
                Value::String("a".into()),
            ])
        );
        Ok(())
    }

    #[test]
    fn returns_the_lexer_diagnostic_for_malformed_literals() {
        // Given
        let source = "1__0";

        // When
        let result = evaluate_literals(source);

        // Then
        assert_eq!(
            result,
            Err(EvaluationError::LexicalDiagnostic(
                "LEX_BAD_NUMERIC_SEPARATOR"
            ))
        );
    }

    #[test]
    fn rejects_a_source_without_a_literal_value() {
        // Given
        let source = "identifier";

        // When
        let result = evaluate_literals(source);

        // Then
        assert_eq!(result, Err(EvaluationError::UnsupportedConstruct));
    }
}
