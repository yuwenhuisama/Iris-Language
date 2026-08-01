//! Static analysis over an accepted program.
//!
//! Lexing and parsing reject MALFORMED source. This pass reports source that
//! parses cleanly but is still invalid, which is what the `IRIS-V1-CONTROL`
//! diagnostic rows assert: a binding declared without a required initializer, a
//! write to an immutable binding, or a control transfer with no valid target.

use iris_syntax::{Expression, Program, ProgramEntry, Statement};

use crate::Diagnostic;

/// Reports the static diagnostics for an accepted program.
pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut analyzer = Analyzer {
        diagnostics: Vec::new(),
        scopes: vec![Vec::new()],
        declared_class_variables: Vec::new(),
        qualified_namespace: Vec::new(),
        generic_classes: Vec::new(),
        generic_constraints: Vec::new(),
        declared_conformance: Vec::new(),
        contract_requirements: Vec::new(),
    };
    analyzer.program(program);
    analyzer.diagnostics
}

/// One binding visible to later statements in the same scope.
struct Local {
    name: String,
    mutable: bool,
    /// The binding's fixed local Type, when it can be determined statically.
    ///
    /// `IRIS-V1-CONTROL-C005` makes an untyped initializer's precise static
    /// Type the binding's FIXED local Type, and `C004` requires later
    /// assignments to satisfy it without widening. `None` means the Type is not
    /// statically known, so no assignment can be rejected against it.
    fixed_type: Option<StaticType>,
}

/// One nominal member of a static Type.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TypeMember {
    Bool,
    Integer,
    Float,
    String,
    Symbol,
    Nil,
    Array,
    Hash,
}

/// The statically known Type of an expression, to the depth this pass tracks.
///
/// Members are kept SORTED and deduplicated, which is what `D-359` means by a
/// normalized union: `String | Nil` and `Nil | String` are one Type. Only the
/// forms below are inferred; anything else is `None` at the call site, which
/// makes the fixed-type check SILENT rather than guessing, since a wrong
/// rejection is far worse than a missed one.
#[derive(Clone, Debug, Eq, PartialEq)]
struct StaticType(Vec<TypeMember>);

impl StaticType {
    fn single(member: TypeMember) -> Self {
        Self(vec![member])
    }

    /// Builds the normalized union of two Types.
    fn union(left: &Self, right: &Self) -> Self {
        let mut members = left.0.clone();
        members.extend(right.0.iter().copied());
        members.sort_unstable();
        members.dedup();
        Self(members)
    }

    /// Reports whether a value of `other` satisfies this Type.
    ///
    /// A wider declared Type ACCEPTS a narrower value, which is what makes
    /// `mut value: String | Integer = 1` legal under `C005` while
    /// `mut value: Integer = 1; value = "text"` is not.
    fn accepts(&self, other: &Self) -> bool {
        other.0.iter().all(|member| self.0.contains(member))
    }
}

impl StaticType {
    /// Maps a written nominal annotation to a tracked Type.
    ///
    /// A union, intersection, or generic annotation deliberately yields
    /// `None`: those widen the cell in ways this pass does not model, and
    /// `C005` explicitly points at `String | Integer` and `Dynamic<Object>` as
    /// the way to ask for a wider cell.
    fn from_nominal(name: &str) -> Option<Self> {
        let member = match name {
            "Bool" => TypeMember::Bool,
            "Integer" => TypeMember::Integer,
            "Float32" | "Float64" => TypeMember::Float,
            "String" => TypeMember::String,
            "Symbol" => TypeMember::Symbol,
            "Nil" => TypeMember::Nil,
            _ => return None,
        };
        Some(Self::single(member))
    }

    /// Infers the Type of an expression, or `None` when it is not known here.
    fn of(expression: &Expression) -> Option<Self> {
        match expression {
            Expression::Symbol(_) => Some(Self::single(TypeMember::Symbol)),
            Expression::Array(_) => Some(Self::single(TypeMember::Array)),
            Expression::Hash(_) => Some(Self::single(TypeMember::Hash)),
            Expression::Grouped(inner) => Self::of(inner),
            Expression::Literal(text) => Self::of_literal(text),
            // `nil`, `true`, and `false` are keywords the expression grammar
            // reaches through the NAME path, so a bare `nil` arrives here as a
            // Name rather than a Literal. Typing only the Literal spelling left
            // `fun m() -> Integer { nil }` unprovable and therefore unreported.
            Expression::Name(text) => Self::of_literal(text),
            _ => None,
        }
    }

    fn of_literal(text: &str) -> Option<Self> {
        match text {
            "nil" => return Some(Self::single(TypeMember::Nil)),
            "true" | "false" => return Some(Self::single(TypeMember::Bool)),
            _ => {}
        }
        let first = text.chars().next()?;
        if first == '"' || first == '\'' {
            return Some(Self::single(TypeMember::String));
        }
        if !first.is_ascii_digit() {
            return None;
        }
        // A float literal is distinguished by its point or exponent, which the
        // lexer has already validated by this point.
        if text.contains('.') || text.contains('e') || text.contains('E') {
            return Some(Self::single(TypeMember::Float));
        }
        Some(Self::single(TypeMember::Integer))
    }
}

/// The control context a statement appears in.
///
/// `IRIS-V1-CONTROL-D-421` makes `break`/`continue` unable to target a loop
/// outside a Closure call boundary and puts `return` inside a Closure at that
/// Closure's own boundary, so a Closure RESETS the loop depth rather than
/// inheriting it.
#[derive(Clone, Copy)]
struct Control {
    loop_depth: usize,
    in_callable: bool,
    /// Whether a loop exists OUTSIDE the nearest Closure boundary.
    ///
    /// `IRIS-V1-CONTROL-C077` keeps two diagnostics distinct: a transfer with no
    /// target loop anywhere reports `CONTROL_TRANSFER_WITHOUT_TARGET`, while one
    /// whose target exists but lies across a Closure call boundary reports
    /// `CONTROL_TARGET_CROSSES_CLOSURE`. Distinguishing them needs the enclosing
    /// loop state the Closure boundary discarded.
    enclosing_loop_across_closure: bool,
}

impl Control {
    const fn top_level() -> Self {
        Self {
            loop_depth: 0,
            in_callable: false,
            enclosing_loop_across_closure: false,
        }
    }

    const fn callable() -> Self {
        Self {
            loop_depth: 0,
            in_callable: true,
            enclosing_loop_across_closure: false,
        }
    }

    /// Enters a Closure body, which resets the loop depth but REMEMBERS whether
    /// a loop was in scope outside it.
    const fn entering_closure(self) -> Self {
        Self {
            loop_depth: 0,
            in_callable: true,
            enclosing_loop_across_closure: self.loop_depth > 0
                || self.enclosing_loop_across_closure,
        }
    }

    const fn entering_loop(self) -> Self {
        Self {
            loop_depth: self.loop_depth + 1,
            in_callable: self.in_callable,
            enclosing_loop_across_closure: self.enclosing_loop_across_closure,
        }
    }

    /// The diagnostic for a `break`/`continue` with no reachable target.
    const fn transfer_diagnostic(self) -> Option<&'static str> {
        if self.loop_depth > 0 {
            None
        } else if self.enclosing_loop_across_closure {
            Some("CONTROL_TARGET_CROSSES_CLOSURE")
        } else {
            Some("CONTROL_TRANSFER_WITHOUT_TARGET")
        }
    }
}

struct Analyzer {
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<Vec<Local>>,
    /// Class-variable names a `shared` declaration published.
    ///
    /// `IRIS-V1-CONTROL-C009` makes assignment to absent `@@name` fail as
    /// missing declared storage rather than creating it, so an assignment is
    /// checked against what was actually declared.
    declared_class_variables: Vec<String>,
    /// Each declared Contract paired with the selectors it REQUIRES.
    ///
    /// `IRIS-V1-TYPES-C046` makes a member satisfying a declared requirement
    /// write `impl`, so a Class listing `for C` is checked against what `C`
    /// actually requires rather than against every member it happens to hold.
    contract_requirements: Vec<(String, Vec<String>)>,
    /// Each generic Class paired with its declared `where` constraints.
    ///
    /// `IRIS-V1-TYPES-C058` lets a concrete argument satisfy an F-bounded
    /// constraint ONLY through explicit nominal conformance, so a closed
    /// construction is checked against what the declaration actually requires.
    generic_constraints: Vec<(String, Vec<String>, Vec<iris_syntax::Constraint>)>,
    /// Each Class paired with the Contract names it declares `for`.
    declared_conformance: Vec<(String, Vec<String>)>,
    /// Names published into the ONE qualified namespace.
    ///
    /// `IRIS-V1-CONTROL-D-432` puts Class, Module, Contract, Type aliases and
    /// constants in a single namespace, so a kind or name collision between any
    /// two of them is an error rather than a shadowing.
    qualified_namespace: Vec<String>,
    /// Generic Class names with their declared parameter arity.
    ///
    /// `IRIS-V1-TYPES-C061` makes a bare generic Class name definition
    /// METADATA rather than an instance Type, so construction and instance
    /// annotations need a closed `Box<Type>`.
    generic_classes: Vec<(String, usize)>,
}

impl Analyzer {
    fn report(&mut self, code: &'static str) {
        if !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code)
        {
            self.diagnostics.push(Diagnostic { code });
        }
    }

    fn declare(&mut self, name: &str, mutable: bool) {
        self.declare_typed(name, mutable, None);
    }

    fn declare_typed(&mut self, name: &str, mutable: bool, fixed_type: Option<StaticType>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Local {
                name: name.to_owned(),
                mutable,
                fixed_type,
            });
        }
    }

    /// Resolves a written annotation to a tracked Type.
    ///
    /// `IRIS-V1-TYPES-C093` makes `typeof(expression)` denote the NORMALIZED
    /// STATIC Type of its operand at that program point, without evaluating it,
    /// so it resolves to whatever Type this pass already tracks there.
    /// Rejects a bare generic Class name used as an instance annotation.
    ///
    /// `IRIS-V1-TYPES-C061` makes bare `Box` definition metadata, not an
    /// instance Type nor shorthand for `Box<Object>`, so an instance annotation
    /// requires a closed `Box<Type>`. `IRIS-V1-TYPES-V209` names the code.
    fn check_raw_generic(&mut self, annotation: &iris_syntax::TypeExpression) {
        if let iris_syntax::TypeExpression::Name(name) = annotation
            && self
                .generic_classes
                .iter()
                .any(|(declared, _)| declared == name)
        {
            self.report("RAW_GENERIC_TYPE_FORBIDDEN");
        }
        self.check_generic_arity(annotation);
    }

    /// Rejects a construction whose closed generic Type is not the annotated one.
    ///
    /// `IRIS-V1-TYPES-V226` makes generic arguments INVARIANT: `Box<String>` is
    /// not assignable to `Box<Object>` even though `String` is an `Object`. The
    /// declared and constructed argument lists must therefore match exactly
    /// rather than by subtyping.
    fn check_generic_invariance(
        &mut self,
        annotation: &iris_syntax::TypeExpression,
        value: &Expression,
    ) {
        let iris_syntax::TypeExpression::Generic {
            name: declared,
            arguments: declared_arguments,
        } = annotation
        else {
            return;
        };
        let Expression::Call { callee, .. } = value else {
            return;
        };
        let Expression::Member { receiver, selector } = callee.as_ref() else {
            return;
        };
        if selector != "new" {
            return;
        }
        let Expression::ClosedGeneric {
            name: constructed,
            arguments: constructed_arguments,
        } = receiver.as_ref()
        else {
            return;
        };
        // A mismatched ARITY is already reported as GENERIC_ARGUMENT_ARITY, and
        // an argument list of the wrong length says nothing about variance, so
        // only a same-length mismatch is an invariance violation.
        if constructed == declared
            && constructed_arguments.len() == declared_arguments.len()
            && constructed_arguments != declared_arguments
        {
            self.report("GENERIC_ARGUMENT_INVARIANCE");
        }
    }

    /// Rejects a closed generic Type whose argument count does not match the
    /// declared parameter list.
    ///
    /// `IRIS-V1-TYPES-V245` observes `GENERIC_ARGUMENT_ARITY` for `Pair<String>`
    /// against `class Pair<T,U>`: no default `U` is supplied. A Class this pass
    /// has not seen declared is left alone, since its arity is unknown rather
    /// than wrong.
    fn check_generic_arity(&mut self, annotation: &iris_syntax::TypeExpression) {
        match annotation {
            iris_syntax::TypeExpression::Generic { name, arguments } => {
                if self
                    .generic_classes
                    .iter()
                    .any(|(declared, arity)| declared == name && *arity != arguments.len())
                {
                    self.report("GENERIC_ARGUMENT_ARITY");
                }
                for argument in arguments {
                    self.check_generic_arity(argument);
                }
            }
            iris_syntax::TypeExpression::Union(members)
            | iris_syntax::TypeExpression::Intersection(members) => {
                for member in members {
                    self.check_generic_arity(member);
                }
            }
            iris_syntax::TypeExpression::Name(_)
            | iris_syntax::TypeExpression::Typeof(_)
            | iris_syntax::TypeExpression::Function { .. } => {}
        }
    }

    fn annotation_type(&self, annotation: &iris_syntax::TypeExpression) -> Option<StaticType> {
        match annotation {
            iris_syntax::TypeExpression::Name(name) => StaticType::from_nominal(name),
            iris_syntax::TypeExpression::Typeof(operand) => self.expression_type(operand),
            // `C005` names `String | Integer` as the way to declare a wider
            // cell, so a written union is the union of its members. One
            // unknown member makes the whole annotation unknown rather than
            // narrower than written, which would cause a WRONG rejection.
            iris_syntax::TypeExpression::Union(members) => members
                .iter()
                .try_fold(None, |accumulated, member| {
                    let member = self.annotation_type(member)?;
                    Some(Some(match accumulated {
                        Some(accumulated) => StaticType::union(&accumulated, &member),
                        None => member,
                    }))
                })
                .flatten(),
            _ => None,
        }
    }

    /// Infers an expression's static Type, or `None` when it is not known.
    fn expression_type(&self, expression: &Expression) -> Option<StaticType> {
        match expression {
            // `nil`, `true` and `false` reach the parser as NAMES rather than
            // literals, so they are resolved here before an ordinary binding
            // lookup, which would otherwise find nothing and report no Type.
            Expression::Name(name) if name == "nil" => Some(StaticType::single(TypeMember::Nil)),
            Expression::Name(name) if name == "true" || name == "false" => {
                Some(StaticType::single(TypeMember::Bool))
            }
            // Any other name carries the fixed Type its binding was declared
            // with.
            Expression::Name(name) => self.lookup(name).and_then(|local| local.fixed_type.clone()),
            // `D-361`: `!x` and `!!x` ALWAYS have static Type Bool, whatever the
            // operand's Type is.
            Expression::Unary {
                operator: iris_syntax::UnaryOperator::Not,
                ..
            } => Some(StaticType::single(TypeMember::Bool)),
            // `D-359`: `to_bool` decides only WHICH operand value is returned,
            // so the static result Type is the normalized union of the
            // reachable operand Types rather than `Bool`.
            Expression::Binary {
                left,
                operator:
                    iris_syntax::BinaryOperator::LogicalAnd | iris_syntax::BinaryOperator::LogicalOr,
                right,
            } => {
                let left = self.expression_type(left)?;
                let right = self.expression_type(right)?;
                Some(StaticType::union(&left, &right))
            }
            _ => StaticType::of(expression),
        }
    }

    /// Reports a parameter default that references a later parameter.
    fn check_default_forward_reference(
        &mut self,
        default: &Expression,
        parameters: &[iris_syntax::Parameter],
    ) {
        let Expression::Name(name) = default else {
            return;
        };
        // Already-declared parameters are in scope, so a name that matches a
        // parameter yet resolves to nothing must be one declared later.
        if self.lookup(name).is_none() && parameters.iter().any(|parameter| parameter.name == *name)
        {
            self.report("PARAMETER_DEFAULT_FORWARD_REFERENCE");
        }
    }

    /// Publishes one name into the shared qualified namespace.
    ///
    /// `V351A` names the collision `QUALIFIED_NAMESPACE_COLLISION`, and D-432
    /// requires the SECOND declaration not to publish, so the namespace keeps
    /// only the first.
    fn publish_qualified_name(&mut self, name: &str) {
        if self.qualified_namespace.iter().any(|taken| taken == name) {
            self.report("QUALIFIED_NAMESPACE_COLLISION");
            return;
        }
        self.qualified_namespace.push(name.to_owned());
    }

    fn lookup(&self, name: &str) -> Option<&Local> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|local| local.name == name))
    }

    fn program(&mut self, program: &Program) {
        for entry in &program.entries {
            match entry {
                ProgramEntry::Statement(statement) => {
                    self.statement(statement, Control::top_level());
                }
                ProgramEntry::Declaration(declaration) => self.declaration(declaration),
            }
        }
    }

    fn declaration(&mut self, declaration: &iris_syntax::Declaration) {
        if let iris_syntax::Declaration::Class(value) = declaration
            && !value.parameters.is_empty()
        {
            self.generic_classes
                .push((value.name.clone(), value.parameters.len()));
        }
        let (name, body) = match declaration {
            iris_syntax::Declaration::Class(value) => {
                self.check_declared_conformance(value);
                self.check_generic_constraints(&value.parameters, &value.constraints);
                if !value.constraints.is_empty() {
                    self.generic_constraints.push((
                        value.name.clone(),
                        value.parameters.clone(),
                        value.constraints.clone(),
                    ));
                }
                self.declared_conformance.push((
                    value.name.clone(),
                    value
                        .implements
                        .iter()
                        .filter_map(|target| match target {
                            iris_syntax::TypeExpression::Name(name) => Some(name.clone()),
                            _ => None,
                        })
                        .collect(),
                ));
                self.check_shared_properties(&value.parameters, &value.body);
                // C063: `open class Box<String>` is an error in v1. A generic
                // parameter list declares NAMES, so an entry that names an
                // already-declared Type is a closed construction rather than a
                // parameter, and reopening one is forbidden.
                if value.reopen && self.has_closed_generic_argument(&value.parameters) {
                    self.report("CLOSED_GENERIC_OPEN_FORBIDDEN");
                }
                (&value.name, Some(&value.body))
            }
            iris_syntax::Declaration::Module(value) => {
                // D-279: `for` declares Contract conformance, which is a Class
                // fact. A Module has no instances to conform, so the clause is
                // parsed and rejected here rather than at parse time.
                if !value.contract_for.is_empty() {
                    self.report("CONTRACT_FOR_CLASS_ONLY");
                }
                (&value.name, Some(&value.body))
            }
            iris_syntax::Declaration::Contract(value) => {
                // D-177: a Contract has no revision to reopen, so `open` is
                // rejected here rather than at parse time. V204 observes a
                // STATIC diagnostic and no published Contract revision.
                if value.open {
                    self.report("OPEN_CONTRACT_FORBIDDEN");
                }
                self.check_contract_body(&value.body);
                let requirements = value
                    .body
                    .iter()
                    .filter_map(|statement| match statement {
                        Statement::Method(method) if method.body.is_none() => {
                            Some(method.selector.clone())
                        }
                        _ => None,
                    })
                    .collect();
                self.contract_requirements
                    .push((value.name.clone(), requirements));
                (&value.name, None)
            }
        };
        self.publish_qualified_name(name);
        let Some(body) = body else {
            return;
        };
        // C062 makes a bodyless Method declaration a Contract REQUIREMENT, so
        // one in a Class or Module body declares an obligation where only an
        // implementation belongs.
        for statement in body {
            if let Statement::Method(method) = statement
                && method.body.is_none()
            {
                self.report("METHOD_BODY_REQUIRED");
            }
        }
        // A Class body holds Method declarations, each of which is its own
        // callable boundary and is entered through `Statement::Method`.
        self.scoped_body(body, Control::top_level());
    }

    /// Reports a value whose static Type the written annotation cannot accept.
    ///
    /// `IRIS-V1-TYPES-C004` requires a PROVABLE violation at any annotated
    /// boundary to be diagnosed BEFORE execution. A value this pass cannot type
    /// is not proven wrong and is left to the runtime guard.
    fn check_annotated_value(
        &mut self,
        annotation: &iris_syntax::TypeExpression,
        value: &Expression,
    ) {
        let (Some(declared), Some(actual)) = (
            self.annotation_type(annotation),
            self.expression_type(value),
        ) else {
            return;
        };
        if !declared.accepts(&actual) {
            self.report("ANNOTATED_VALUE_CONTRACT_VIOLATION");
        }
    }

    /// Rejects a return whose Type is PROVABLY not the declared one.
    ///
    /// `IRIS-V1-TYPES-C004` makes a written annotation both a static Contract
    /// and a runtime guard, and requires a PROVABLE violation to be diagnosed
    /// BEFORE execution. A body whose result is a literal is exactly such a
    /// proof, so `fun m() -> Integer { nil }` is rejected here rather than
    /// waiting to raise when the Method is finally called.
    ///
    /// Only a provable case is reported. A result this pass cannot type is left
    /// to the runtime guard, which is what keeps a not-proven boundary a
    /// runtime check rather than a false rejection.
    fn check_return_annotation(&mut self, declaration: &iris_syntax::MethodDeclaration) {
        let Some(annotation) = &declaration.return_type else {
            return;
        };
        // `Never` is uninhabited, so ANY normal return violates it. It has no
        // nominal StaticType, which is why it is decided before the mapping.
        if matches!(annotation, iris_syntax::TypeExpression::Name(name) if name == "Never")
            && declaration
                .body
                .as_ref()
                .is_some_and(|body| Self::result_expression(body).is_some())
        {
            self.report("RETURN_TYPE_CONTRACT_VIOLATION");
            return;
        }
        let (Some(declared), Some(body)) = (self.annotation_type(annotation), &declaration.body)
        else {
            return;
        };
        let Some(result) = Self::result_expression(body) else {
            return;
        };
        if let Some(actual) = StaticType::of(result)
            && !declared.accepts(&actual)
        {
            self.report("RETURN_TYPE_CONTRACT_VIOLATION");
        }
    }

    /// The expression a body evaluates to, when the body ends in one.
    ///
    /// A body ending in anything else has no statically known result, so it is
    /// left to the runtime guard.
    fn result_expression(body: &[Statement]) -> Option<&Expression> {
        match body.last() {
            Some(Statement::Expression(expression)) => Some(expression),
            _ => None,
        }
    }

    /// Rejects a `shared class property` whose Type mentions a type parameter.
    ///
    /// `IRIS-V1-TYPES-C064` puts a `shared class property` on the UNAPPLIED
    /// generic definition, so it has one slot for every closed construction and
    /// MUST NOT reference the definition's type parameters directly or
    /// indirectly. `IRIS-V1-TYPES-V237` names the code. An ordinary class-level
    /// property is per closed construction and may name a parameter freely.
    fn check_shared_properties(&mut self, parameters: &[String], body: &[Statement]) {
        if parameters.is_empty() {
            return;
        }
        for statement in body {
            if let Statement::StoredProperty {
                shared, annotation, ..
            } = statement
                && *shared
                && Self::mentions_parameter(annotation, parameters)
            {
                self.report("GENERIC_SHARED_PROPERTY_REFERENCES_TYPE_PARAMETER");
            }
        }
    }

    /// Reports whether a written Type names one of the given parameters, at any
    /// depth, so an INDIRECT reference such as `Array<T>` is caught too.
    fn mentions_parameter(annotation: &iris_syntax::TypeExpression, parameters: &[String]) -> bool {
        match annotation {
            iris_syntax::TypeExpression::Name(name) => parameters.contains(name),
            iris_syntax::TypeExpression::Generic { name, arguments } => {
                parameters.contains(name)
                    || arguments
                        .iter()
                        .any(|argument| Self::mentions_parameter(argument, parameters))
            }
            iris_syntax::TypeExpression::Union(members)
            | iris_syntax::TypeExpression::Intersection(members) => members
                .iter()
                .any(|member| Self::mentions_parameter(member, parameters)),
            iris_syntax::TypeExpression::Function {
                parameters: p,
                result,
            } => {
                p.iter()
                    .any(|entry| Self::mentions_parameter(entry, parameters))
                    || Self::mentions_parameter(result, parameters)
            }
            iris_syntax::TypeExpression::Typeof(_) => false,
        }
    }

    /// Reports whether a generic parameter list names a closed construction.
    ///
    /// A generic parameter DECLARES a fresh name, so an entry naming a Type
    /// that already exists is a closed generic argument such as the `String` in
    /// `Box<String>`. `IRIS-V1-TYPES-C063` forbids reopening one.
    fn has_closed_generic_argument(&self, parameters: &[String]) -> bool {
        const BUILTIN_TYPES: [&str; 9] = [
            "Object", "Nil", "Bool", "Integer", "Float32", "Float64", "String", "Symbol", "Never",
        ];
        parameters.iter().any(|parameter| {
            BUILTIN_TYPES.contains(&parameter.as_str())
                || self
                    .qualified_namespace
                    .iter()
                    .any(|name| name == parameter)
        })
    }

    /// Rejects a `where` clause whose parameter bounds form a cycle.
    ///
    /// `IRIS-V1-TYPES-V243` observes `GENERIC_CONSTRAINT_CYCLE` for
    /// `where T: U, U: T`: neither parameter can be resolved before the other,
    /// so no argument can ever satisfy the pair. A bound naming a Type OUTSIDE
    /// the parameter list is ordinary and forms no cycle, and `C058` keeps an
    /// F-bounded constraint such as `where T: Comparable<T>` legal, so only a
    /// bound that is a BARE parameter name contributes an edge.
    fn check_generic_constraints(
        &mut self,
        parameters: &[String],
        constraints: &[iris_syntax::Constraint],
    ) {
        let edges: Vec<(&String, &String)> = constraints
            .iter()
            .filter_map(|constraint| match &constraint.bound {
                iris_syntax::TypeExpression::Name(bound) if parameters.contains(bound) => {
                    Some((&constraint.parameter, bound))
                }
                _ => None,
            })
            .collect();
        let cyclic = parameters.iter().any(|start| {
            let mut seen = vec![start];
            let mut cursor = start;
            loop {
                let Some((_, next)) = edges.iter().find(|(from, _)| *from == cursor) else {
                    break false;
                };
                if seen.contains(next) {
                    break true;
                }
                seen.push(next);
                cursor = next;
            }
        });
        if cyclic {
            self.report("GENERIC_CONSTRAINT_CYCLE");
        }
    }

    /// Checks that a Class satisfying a declared Contract requirement marks the
    /// member with `impl`.
    ///
    /// `IRIS-V1-TYPES-C046` rejects an unmarked Class-provided implementation
    /// where an explicit one is required, which is what `IRIS-V1-TYPES-V248`
    /// observes. A member whose selector no listed Contract requires is an
    /// ordinary Method and is left alone.
    fn check_declared_conformance(&mut self, declaration: &iris_syntax::ClassDeclaration) {
        let listed: Vec<&String> = declaration
            .implements
            .iter()
            .filter_map(|target| match target {
                iris_syntax::TypeExpression::Name(name) => Some(name),
                _ => None,
            })
            .collect();
        let required: Vec<String> = self
            .contract_requirements
            .iter()
            .filter(|(contract, _)| listed.contains(&contract))
            .flat_map(|(_, selectors)| selectors.iter().cloned())
            .collect();
        if required.is_empty() {
            return;
        }
        for statement in &declaration.body {
            if let Statement::Method(method) = statement
                && method.body.is_some()
                && method.impl_contract.is_none()
                && required.contains(&method.selector)
            {
                self.report("CONTRACT_IMPLEMENTATION_REQUIRES_IMPL");
            }
        }
    }

    /// Checks the members a Contract body may hold.
    ///
    /// `IRIS-V1-TYPES-C042` permits Method and property REQUIREMENTS and
    /// forbids Method bodies, stored state, initializers, and executable
    /// statements. `IRIS-V1-GRAMMAR-C062` makes the body optional so the
    /// requirement form parses at all; a body that IS written now reaches here
    /// and is reported, which is what `IRIS-V1-TYPES-V258` observes.
    fn check_contract_body(&mut self, body: &[Statement]) {
        for statement in body {
            match statement {
                Statement::Method(method) if method.body.is_some() => {
                    self.report("CONTRACT_METHOD_BODY_FORBIDDEN");
                }
                Statement::Method(_) => {}
                // Anything else in a Contract body is stored state, an
                // initializer, or an executable statement, all of which C042
                // forbids outright.
                _ => self.report("CONTRACT_BODY_MEMBER_FORBIDDEN"),
            }
        }
    }

    /// Analyzes a body in its own scope, so a binding does not leak outward.
    fn scoped_body(&mut self, body: &[Statement], control: Control) {
        self.scopes.push(Vec::new());
        for statement in body {
            self.statement(statement, control);
        }
        self.scopes.pop();
    }

    fn statement(&mut self, statement: &Statement, control: Control) {
        match statement {
            Statement::Binding {
                mutable,
                constant,
                name,
                annotation,
                value,
            } => {
                // D-432: a `const` publishes into the shared qualified
                // namespace alongside Class, Module, Contract and Type aliases.
                if *constant {
                    self.publish_qualified_name(name);
                }
                self.expression(value, control);
                // C005: an annotation is the contract when written, otherwise
                // the initializer's precise static Type becomes the fixed one.
                if let Some(annotation) = annotation {
                    self.check_raw_generic(annotation);
                }
                if let Some(annotation) = annotation {
                    self.check_generic_invariance(annotation, value);
                }
                let fixed_type = match annotation {
                    Some(annotation) => {
                        let declared = self.annotation_type(annotation);
                        let contract = declared.clone();
                        // C005 makes the annotation a CONTRACT for the cell, so
                        // the initializer must satisfy it at the declaration
                        // just as later assignments must.
                        if let (Some(contract), Some(initial)) =
                            (contract, self.expression_type(value))
                            && !contract.accepts(&initial)
                        {
                            self.report("BINDING_FIXED_LOCAL_TYPE");
                        }
                        declared
                    }
                    None => self.expression_type(value),
                };
                // C078: a declaration rebinding a name already declared in the
                // SAME scope is a diagnostic, and the original stays bound. An
                // outer scope is unaffected, since shadowing is ordinary.
                if self
                    .scopes
                    .last()
                    .is_some_and(|scope| scope.iter().any(|local| local.name == *name))
                {
                    self.report("DECLARATION_REBINDING");
                }
                self.declare_typed(name, *mutable, fixed_type);
            }
            // `IRIS-V1-CONTROL-D-427`: `let` MUST be initialized, and a deferred
            // `mut` is legal only with an explicit type.
            Statement::DeferredBinding {
                mutable,
                annotated,
                name,
            } => {
                if *mutable {
                    if !annotated {
                        self.report("BINDING_MISSING_TYPE_FOR_DEFERRED_INIT");
                    }
                } else {
                    self.report("BINDING_LET_REQUIRES_INITIALIZER");
                }
                self.declare(name, *mutable);
            }
            Statement::SharedBinding {
                name,
                mutable,
                annotation,
                value,
            } => {
                // C004 guards a binding boundary whether the cell is local or
                // class-level, so a class variable's annotation is checked with
                // the same proof an ordinary binding uses.
                if let Some(annotation) = annotation {
                    self.check_raw_generic(annotation);
                    self.check_annotated_value(annotation, value);
                }
                self.declared_class_variables.push(name.clone());
                self.declare(name, *mutable);
            }
            Statement::Expression(expression) => self.expression(expression, control),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expression(condition, control);
                self.scoped_body(then_body, control);
                if let Some(body) = else_body {
                    self.scoped_body(body, control);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                self.expression(condition, control);
                self.scoped_body(body, control.entering_loop());
            }
            Statement::For {
                binding,
                iterable,
                body,
                ..
            } => {
                self.expression(iterable, control);
                self.scopes.push(Vec::new());
                // An iteration binding is immutable under `D-426`.
                for name in pattern_names(binding) {
                    self.declare(&name, false);
                }
                for statement in body {
                    self.statement(statement, control.entering_loop());
                }
                self.scopes.pop();
            }
            // `D-438` and `D-421`: a loop transfer needs a loop in the SAME
            // callable, since a Closure boundary resets the depth.
            Statement::Break { value, .. } => {
                if let Some(code) = control.transfer_diagnostic() {
                    self.report(code);
                }
                if let Some(value) = value {
                    self.expression(value, control);
                }
            }
            Statement::Continue(_) => {
                if let Some(code) = control.transfer_diagnostic() {
                    self.report(code);
                }
            }
            // `D-421` puts `return` at a callable boundary, so one outside any
            // callable has no invocation to end.
            Statement::Return(value) => {
                if !control.in_callable {
                    self.report("CONTROL_RETURN_OUTSIDE_CALLABLE");
                }
                if let Some(value) = value {
                    self.expression(value, control);
                }
            }
            Statement::Match { subject, arms, .. } => {
                self.expression(subject, control);
                for arm in arms {
                    match &arm.body {
                        iris_syntax::MatchBody::Expression(value) => {
                            self.expression(value, control);
                        }
                        iris_syntax::MatchBody::Block(body) => self.scoped_body(body, control),
                    }
                }
            }
            Statement::Try {
                body,
                catches,
                finally,
                ..
            } => {
                self.scoped_body(body, control);
                for catch in catches {
                    self.scoped_body(&catch.body, control);
                }
                if let Some(body) = finally {
                    self.scoped_body(body, control);
                }
            }
            // A bare `raise` re-raises the active context and carries no
            // expression of its own.
            Statement::Raise(raise) => {
                if let Some(raise) = raise {
                    self.expression(&raise.value, control);
                    if let Some(cause) = &raise.cause {
                        self.expression(cause, control);
                    }
                }
            }
            Statement::Method(declaration) => {
                // IRIS-V1-CONTROL-C006 makes parameter bindings IMMUTABLE
                // unless their own declaration uses `mut`. Declaring them keeps
                // a write to one reported as an immutable-binding error rather
                // than as an unresolved target.
                self.scopes.push(Vec::new());
                for parameter in &declaration.parameters {
                    // C078: a default referencing a parameter declared LATER is
                    // a forward reference. Each parameter is declared before the
                    // next default is checked, so an earlier parameter resolves
                    // and a later one does not.
                    if let Some(default) = &parameter.default {
                        self.check_default_forward_reference(default, &declaration.parameters);
                        // C004 names a PARAMETER as an annotated boundary, and
                        // a written default is a value crossing it at the
                        // declaration, so a provable mismatch is reported here.
                        if let Some(annotation) = &parameter.annotation {
                            self.check_annotated_value(annotation, default);
                        }
                    }
                    self.declare(&parameter.name, false);
                }
                // A bodyless C062 requirement has no statements to analyse; it
                // declares an obligation rather than an implementation.
                for statement in declaration.body.iter().flatten() {
                    self.statement(statement, Control::callable());
                }
                self.check_return_annotation(declaration);
                self.scopes.pop();
            }
            Statement::StoredProperty {
                annotation,
                initializer,
                ..
            } => {
                self.expression(initializer, control);
                // C004 names a PROPERTY as one of the annotated boundaries, so
                // an initializer whose Type the annotation does not accept is a
                // provable violation exactly as a binding's is.
                self.check_raw_generic(annotation);
                self.check_annotated_value(annotation, initializer);
            }
        }
    }

    fn expression(&mut self, expression: &Expression, control: Control) {
        match expression {
            Expression::Assignment { left, right, .. } => {
                self.expression(right, control);
                // `D-426`: a bare `name = expr` only ASSIGNS an existing mutable
                // binding. It never implicitly declares, which is what prevents
                // a typo from creating a local.
                // C009 and C078: assignment to absent `@@name` storage is
                // `MISSING_DECLARED_STORAGE` and creates nothing. A `shared`
                // declaration is what publishes that storage.
                if let Expression::ClassVar(name) = left.as_ref()
                    && !self.declared_class_variables.iter().any(|declared| {
                        declared == name || declared.trim_start_matches('@') == name
                    })
                {
                    self.report("MISSING_DECLARED_STORAGE");
                }
                if let Expression::Name(name) = left.as_ref() {
                    match self.lookup(name) {
                        None => self.report("BINDING_UNRESOLVED_ASSIGNMENT_TARGET"),
                        Some(local) if !local.mutable => {
                            self.report("BINDING_ASSIGN_TO_IMMUTABLE");
                        }
                        // C004: a later assignment MUST satisfy the fixed local
                        // Type and MUST NOT widen it implicitly. Both Types must
                        // be known, so an unknown initializer or a union
                        // annotation leaves the binding unchecked rather than
                        // guessed at.
                        Some(local) => {
                            let assigned = self.expression_type(right);
                            let fixed = local.fixed_type.clone();
                            if let (Some(fixed), Some(assigned)) = (fixed, assigned)
                                && !fixed.accepts(&assigned)
                            {
                                self.report("BINDING_FIXED_LOCAL_TYPE");
                            }
                        }
                    }
                } else {
                    self.expression(left, control);
                }
            }
            // A Closure body is a callable boundary for `return`, and it resets
            // the loop depth so a `break` inside it cannot target an outer loop.
            Expression::Closure { body, .. } => {
                self.scoped_body(body, control.entering_closure());
            }
            Expression::Call { callee, arguments } => {
                // C061: ordinary construction requires a closed `Box<Type>`, so
                // `Box.new()` on a generic Class supplies no arguments for its
                // declared parameters. V232 names the code.
                if let Expression::Member { receiver, selector } = callee.as_ref()
                    && selector == "new"
                    && let Expression::Name(name) = receiver.as_ref()
                    && self
                        .generic_classes
                        .iter()
                        .any(|(declared, _)| declared == name)
                {
                    self.report("GENERIC_ARGUMENT_ARITY");
                }
                self.expression(callee, control);
                for argument in arguments {
                    self.expression(argument, control);
                }
            }
            Expression::Index { receiver, index } => {
                self.expression(receiver, control);
                self.expression(index, control);
            }
            // A reified Type expression names Types and holds no subexpression
            // to analyse, but its generic arguments carry the same arity
            // obligation a written annotation does.
            Expression::ReifiedType(annotation) => self.check_generic_arity(annotation),
            // A closed generic construction names a Type, so it carries the
            // same arity obligation a written annotation does.
            Expression::ClosedGeneric { name, arguments } => {
                self.check_generic_arity(&iris_syntax::TypeExpression::Generic {
                    name: name.clone(),
                    arguments: arguments.clone(),
                });
            }
            Expression::Binary { left, right, .. } => {
                self.expression(left, control);
                self.expression(right, control);
            }
            Expression::Unary { operand, .. } => self.expression(operand, control),
            Expression::Grouped(value) | Expression::KeywordArgument { value, .. } => {
                self.expression(value, control);
            }
            Expression::Member { receiver, .. } | Expression::ContractView { receiver, .. } => {
                self.expression(receiver, control);
            }
            Expression::Array(values) => {
                for value in values {
                    self.expression(value, control);
                }
            }
            Expression::Hash(entries) => {
                for (key, value) in entries {
                    self.expression(key, control);
                    self.expression(value, control);
                }
            }
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expression(condition, control);
                self.scoped_body(then_body, control);
                if let Some(body) = else_body {
                    self.scoped_body(body, control);
                }
            }
            Expression::While {
                condition, body, ..
            } => {
                self.expression(condition, control);
                self.scoped_body(body, control.entering_loop());
            }
            Expression::Try {
                body,
                catches,
                finally,
            } => {
                self.scoped_body(body, control);
                for catch in catches {
                    self.scoped_body(&catch.body, control);
                }
                if let Some(body) = finally {
                    self.scoped_body(body, control);
                }
            }
            // IRIS-V1-CONTROL-C006 makes `_` a discard binding that creates no
            // readable binding, so READING it as an expression is a
            // compile-time error. `IRIS-V1-CONTROL-V301` names the code.
            Expression::Name(name) if name == "_" => {
                self.report("DISCARD_BINDING_READ");
            }
            Expression::Name(_)
            | Expression::Literal(_)
            | Expression::Symbol(_)
            | Expression::RawIvar(_)
            | Expression::ClassVar(_) => {}
        }
    }
}

fn pattern_names(pattern: &iris_syntax::Pattern) -> Vec<String> {
    match pattern {
        iris_syntax::Pattern::Name(name) => vec![name.clone()],
        iris_syntax::Pattern::Array(patterns) | iris_syntax::Pattern::Alternatives(patterns) => {
            patterns.iter().flat_map(pattern_names).collect()
        }
        iris_syntax::Pattern::Literal(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn d427_requires_an_initializer_for_let_and_a_type_for_a_deferred_mut() {
        assert_eq!(
            codes("let x: Integer"),
            ["BINDING_LET_REQUIRES_INITIALIZER"]
        );
        assert_eq!(codes("mut x"), ["BINDING_MISSING_TYPE_FOR_DEFERRED_INIT"]);

        // A typed deferred `mut` is exactly the form D-427 permits, and an
        // initialized binding of either kind is unremarkable.
        assert!(codes("mut x: Integer").is_empty());
        assert!(codes("let x = 1").is_empty());
        assert!(codes("let mut x = 1").is_empty());
    }

    #[test]
    fn d426_rejects_writing_an_absent_or_immutable_binding() {
        assert_eq!(
            codes("missing = 1"),
            ["BINDING_UNRESOLVED_ASSIGNMENT_TARGET"]
        );
        assert_eq!(
            codes("let x: Integer = 1; x = 2"),
            ["BINDING_ASSIGN_TO_IMMUTABLE"]
        );

        // Writing a declared mutable binding is the legal case, including from
        // inside a Closure that captures it.
        assert!(codes("let mut x = 1; x = 2").is_empty());
        assert!(codes("let mut x = 1; let c = { x = 2 }").is_empty());
    }

    #[test]
    fn d421_binds_a_loop_transfer_and_a_return_to_their_own_callable() {
        // C077 keeps the two control-target diagnostics DISTINCT. A loop exists
        // here, but it lies across a Closure call boundary.
        assert_eq!(
            codes("let mut i = 0; while i < 3 { let c = { break }; i = i + 1 }"),
            ["CONTROL_TARGET_CROSSES_CLOSURE"]
        );
        // With no loop anywhere, the target does not exist at all.
        assert_eq!(
            codes("let c = { break }"),
            ["CONTROL_TRANSFER_WITHOUT_TARGET"]
        );
        assert_eq!(codes("break"), ["CONTROL_TRANSFER_WITHOUT_TARGET"]);
        assert_eq!(codes("continue"), ["CONTROL_TRANSFER_WITHOUT_TARGET"]);
        assert_eq!(codes("return 1"), ["CONTROL_RETURN_OUTSIDE_CALLABLE"]);

        // `return` inside a Closure ends that Closure, and a loop transfer
        // inside its own loop is ordinary.
        assert!(codes("let c = { return 1 }").is_empty());
        assert!(codes("let mut i = 0; while i < 3 { i = i + 1; break }").is_empty());
        assert!(codes("for x in [1, 2] { continue }").is_empty());
        assert!(codes("class C { public fun m() { return 1 } }").is_empty());
    }

    #[test]
    fn c062_admits_a_bodyless_contract_requirement_and_rejects_a_body() {
        // C062 makes `block_body` optional so a Contract can state the Method
        // requirements C042 already presupposes. Before this, `fun m() -> Nil`
        // could not parse at all, so a Contract could not be written.
        let requirement = "contract C { fun m() -> Nil }";
        // C042 forbids a Method body in a Contract. The body now PARSES and is
        // reported as a static diagnostic rather than failing as a parse error,
        // which is what V258 observes.
        let body_in_contract = "contract C { fun m() -> Nil { nil } }";
        // A bodyless declaration is a REQUIREMENT, so one in a Class body
        // declares an obligation where only an implementation belongs.
        let requirement_in_class = "class A { fun m() -> Nil }";
        // C042 also forbids stored state and executable statements.
        let statement_in_contract = "contract C { let x = 1 }";
        // An ordinary Method that writes a body is unaffected.
        let ordinary = "class A { public fun m() { 7 } }";

        // When / Then
        assert_eq!(codes(requirement), Vec::<&str>::new());
        assert_eq!(
            codes(body_in_contract),
            vec!["CONTRACT_METHOD_BODY_FORBIDDEN"]
        );
        assert_eq!(codes(requirement_in_class), vec!["METHOD_BODY_REQUIRED"]);
        assert_eq!(
            codes(statement_in_contract),
            vec!["CONTRACT_BODY_MEMBER_FORBIDDEN"]
        );
        assert_eq!(codes(ordinary), Vec::<&str>::new());
    }

    #[test]
    fn d177_rejects_an_open_contract_statically() {
        // D-177: a Contract has no revision to reopen. V204 expects a STATIC
        // `OPEN_CONTRACT_FORBIDDEN` and no published Contract revision, so
        // `open` is CONSUMED by the parser and rejected here. Failing at parse
        // time instead would report a parse error the vector does not name.
        let open_contract = "open contract C { fun m() -> Nil }";
        let plain_contract = "contract C { fun m() -> Nil }";
        // `open` keeps its ordinary meaning on a Class, which is a reopen.
        let open_class = "open class A {}";

        // When / Then
        assert_eq!(codes(open_contract), vec!["OPEN_CONTRACT_FORBIDDEN"]);
        assert_eq!(codes(plain_contract), Vec::<&str>::new());
        assert_eq!(codes(open_class), Vec::<&str>::new());
    }

    #[test]
    fn d279_keeps_contract_conformance_a_class_fact() {
        // D-279: `for` declares Contract conformance, which is a Class fact. A
        // Module has no instances to conform. V261 expects a STATIC
        // `CONTRACT_FOR_CLASS_ONLY`, so the clause `module_decl` does not admit
        // is parsed and rejected here rather than failing as a header-order
        // parse error.
        let module_for = "contract C { fun m() -> Nil } module M for C {}";
        let plain_module = "module M {}";
        // The same clause on a Class is ordinary conformance and is accepted.
        let class_for =
            "contract C { fun m() -> Nil } class A for C { impl fun m() -> Nil { nil } }";

        // When / Then
        assert_eq!(codes(module_for), vec!["CONTRACT_FOR_CLASS_ONLY"]);
        assert_eq!(codes(plain_module), Vec::<&str>::new());
        assert_eq!(codes(class_for), Vec::<&str>::new());
    }

    #[test]
    fn d233_requires_impl_on_a_member_satisfying_a_requirement() {
        // C046 rejects an unmarked Class-provided implementation where an
        // explicit one is required, which is what V248 observes.
        let unmarked = "contract C { fun m() -> Nil } class A for C { fun m() -> Nil {} }";
        let marked = "contract C { fun m() -> Nil } class A for C { impl fun m() -> Nil {} }";
        // A member whose selector NO listed Contract requires is an ordinary
        // Method, so it is left alone rather than swept up by the check.
        let unrelated =
            "contract C { fun m() -> Nil } class A for C { impl fun m() -> Nil {} fun other() {} }";
        // Without `for C` the Class declares no conformance at all.
        let no_conformance = "contract C { fun m() -> Nil } class A { fun m() -> Nil {} }";

        // When / Then
        assert_eq!(
            codes(unmarked),
            vec!["CONTRACT_IMPLEMENTATION_REQUIRES_IMPL"]
        );
        assert_eq!(codes(marked), Vec::<&str>::new());
        assert_eq!(codes(unrelated), Vec::<&str>::new());
        assert_eq!(codes(no_conformance), Vec::<&str>::new());
    }

    #[test]
    fn c063_and_d216_reject_reopened_and_cyclic_generics() {
        // C063 makes `open class Box<String>` an error in v1: a generic
        // parameter DECLARES a fresh name, so an entry naming an existing Type
        // is a closed construction rather than a parameter.
        let reopen_closed = "open class Box<String> { fun m() -> Nil {} }";
        let reopen_parameter = "open class Box<T> { fun m() -> Nil {} }";
        let closed_without_open = "class Box<String> {}";
        // D-216: `where T: U, U: T` resolves neither parameter before the
        // other, so no argument can ever satisfy the pair.
        let cycle = "class Bad<T,U> where T: U, U: T {}";
        let self_cycle = "class Bad<T> where T: T {}";
        let acyclic = "class Pair<T,U> where U: T {}";
        // C058 keeps a restricted F-bounded constraint legal, so a bound that
        // MENTIONS its parameter inside a generic argument is not an edge.
        let f_bounded = "contract Comparable<T> {} class Box<T> where T: Comparable<T> {}";

        // When / Then
        assert_eq!(codes(reopen_closed), vec!["CLOSED_GENERIC_OPEN_FORBIDDEN"]);
        assert_eq!(codes(reopen_parameter), Vec::<&str>::new());
        assert_eq!(codes(closed_without_open), Vec::<&str>::new());
        assert_eq!(codes(cycle), vec!["GENERIC_CONSTRAINT_CYCLE"]);
        assert_eq!(codes(self_cycle), vec!["GENERIC_CONSTRAINT_CYCLE"]);
        assert_eq!(codes(acyclic), Vec::<&str>::new());
        assert_eq!(codes(f_bounded), Vec::<&str>::new());
    }

    #[test]
    fn c063_admits_a_closed_generic_in_expression_position() {
        // C063 adds `closed_generic_name` to `primary_expr`, so a CLOSED
        // generic construction may be used as a value.
        let construction = "class Box<T> {} let item: Box<Nil> = Box<Nil>.new()";
        // C020 is NOT weakened: where both readings are well formed the
        // OPERATOR reading wins, so a comparison between two Class names still
        // parses as a comparison rather than as a failed generic.
        let comparison = "class Box<T> {} class C {} Box < C";
        let plain_comparison = "class A {} class B {} A < B";
        // D-218: `Pair<String>` against `class Pair<T,U>` supplies no default
        // `U`, and the same obligation applies in expression position.
        let short_arity = "class Pair<T,U> {} let p: Pair<String> = Pair<String, Integer>.new()";
        let exact_arity =
            "class Pair<T,U> {} let p: Pair<String,Integer> = Pair<String,Integer>.new()";
        // A Class this pass never saw declared has unknown arity, not wrong
        // arity, so it is left alone.
        let undeclared = "let x: Unknown<String> = 1";

        // When / Then
        assert_eq!(codes(construction), Vec::<&str>::new());
        assert_eq!(codes(comparison), Vec::<&str>::new());
        assert_eq!(codes(plain_comparison), Vec::<&str>::new());
        assert_eq!(codes(short_arity), vec!["GENERIC_ARGUMENT_ARITY"]);
        assert_eq!(codes(exact_arity), Vec::<&str>::new());
        assert_eq!(codes(undeclared), Vec::<&str>::new());
    }

    #[test]
    fn v226_keeps_generic_arguments_invariant() {
        // V226 makes generic arguments INVARIANT: `Box<String>` is not
        // assignable to `Box<Object>` even though `String` IS an `Object`, so
        // the argument lists must match exactly rather than by subtyping.
        let widened = "class Box<T> {} let target: Box<Object> = Box<String>.new()";
        let exact = "class Box<T> {} let target: Box<String> = Box<String>.new()";
        let non_generic = "class A {} let a: A = A.new()";
        // A mismatched ARITY says nothing about variance and is already its own
        // diagnostic, so only a same-length mismatch is an invariance failure.
        let wrong_arity = "class Pair<T,U> {} let p: Pair<String> = Pair<String, Integer>.new()";

        // When / Then
        assert_eq!(codes(widened), vec!["GENERIC_ARGUMENT_INVARIANCE"]);
        assert_eq!(codes(exact), Vec::<&str>::new());
        assert_eq!(codes(non_generic), Vec::<&str>::new());
        assert_eq!(codes(wrong_arity), vec!["GENERIC_ARGUMENT_ARITY"]);
    }

    #[test]
    fn c064_keeps_a_shared_property_off_the_type_parameters() {
        // C064 puts a `shared class property` on the UNAPPLIED generic
        // definition, so it has ONE slot across every closed construction and
        // cannot reference a parameter that differs per construction.
        let direct = "class Cache<T> { shared class property bad: T }";
        // The reference may be INDIRECT, so nesting must be searched too.
        let indirect = "class Cache<T> { shared class property bad: Array<T> }";
        let concrete = "class Cache<T> { shared class property n: Integer = 0 }";
        // Ordinary class-level storage IS per closed construction, so naming a
        // parameter there is exactly what C064 permits.
        let per_construction = "class Cache<T> { class property ok: T }";
        // A non-generic Class has no parameters to reference.
        let non_generic = "class A { property n: Integer = 0 }";

        // When / Then
        assert_eq!(
            codes(direct),
            vec!["GENERIC_SHARED_PROPERTY_REFERENCES_TYPE_PARAMETER"]
        );
        assert_eq!(
            codes(indirect),
            vec!["GENERIC_SHARED_PROPERTY_REFERENCES_TYPE_PARAMETER"]
        );
        assert_eq!(codes(concrete), Vec::<&str>::new());
        assert_eq!(codes(per_construction), Vec::<&str>::new());
        assert_eq!(codes(non_generic), Vec::<&str>::new());
    }

    #[test]
    fn c004_diagnoses_a_provable_return_violation_before_execution() {
        // C004 requires a PROVABLE violation to be diagnosed BEFORE execution,
        // not merely guarded at runtime. A body whose result is a literal is
        // exactly such a proof.
        let provable = "class A { public fun m() -> Integer { nil } }";
        let satisfied = "class A { public fun m() -> Integer { 1 } }";
        // A `Nil` return ADMITS nil, so the proof must not fire on every nil.
        let nil_return = "class A { public fun m() -> Nil { nil } }";
        // D-458: `Never` is uninhabited, so ANY normal return violates it.
        let never = "fun fail() -> Never { nil }";
        // A result this pass cannot type is NOT proven wrong, so it is left to
        // the runtime guard rather than rejected here.
        let unprovable = "class A { public fun m(x) -> Integer { x } }";
        let unannotated = "class A { public fun m() { nil } }";

        // When / Then
        assert_eq!(codes(provable), vec!["RETURN_TYPE_CONTRACT_VIOLATION"]);
        assert_eq!(codes(satisfied), Vec::<&str>::new());
        assert_eq!(codes(nil_return), Vec::<&str>::new());
        assert_eq!(codes(never), vec!["RETURN_TYPE_CONTRACT_VIOLATION"]);
        assert_eq!(codes(unprovable), Vec::<&str>::new());
        assert_eq!(codes(unannotated), Vec::<&str>::new());
    }

    #[test]
    fn c004_diagnoses_provable_violations_at_every_annotated_boundary() {
        // C004 names binding, property, and parameter among the annotated
        // boundaries. Only the binding one proved its violations; a property
        // initializer, a parameter default, and a class-variable annotation
        // were parsed and then ignored.
        let property = "class A { property n: Integer = nil }";
        let parameter_default = "class A { public fun m(x: Integer = nil) { x } }";
        let class_variable = "class A { shared let @@n: Integer = nil }";
        // The satisfying spelling of each must stay silent, so the checks prove
        // a mismatch rather than firing on the presence of an annotation.
        let property_ok = "class A { property n: Integer = 1 }";
        let parameter_ok = "class A { public fun m(x: Integer = 1) { x } }";
        let class_variable_ok = "class A { shared let @@n: Integer = 1 }";
        // An unannotated boundary has nothing to prove against.
        let unannotated_parameter = "class A { public fun m(x = nil) { x } }";

        // When / Then
        assert_eq!(codes(property), vec!["ANNOTATED_VALUE_CONTRACT_VIOLATION"]);
        assert_eq!(
            codes(parameter_default),
            vec!["ANNOTATED_VALUE_CONTRACT_VIOLATION"]
        );
        assert_eq!(
            codes(class_variable),
            vec!["ANNOTATED_VALUE_CONTRACT_VIOLATION"]
        );
        assert_eq!(codes(property_ok), Vec::<&str>::new());
        assert_eq!(codes(parameter_ok), Vec::<&str>::new());
        assert_eq!(codes(class_variable_ok), Vec::<&str>::new());
        assert_eq!(codes(unannotated_parameter), Vec::<&str>::new());
    }
}

#[cfg(test)]
mod reserved_form_tests {
    use crate::parse;

    fn codes(source: &str) -> Vec<&'static str> {
        parse(source)
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn reserved_forms_report_the_codes_their_frozen_rows_name() {
        // `IRIS-V1-CONTROL-V359` and `V324` NAME these codes, so the parser uses
        // them rather than a locally invented spelling.
        assert_eq!(codes("defer { cleanup() }"), ["PARSE_UNSUPPORTED_DEFER"]);
        assert_eq!(
            codes("let mut x = 1; x %= 2"),
            ["PARSE_UNSUPPORTED_COMPOUND_ASSIGNMENT"]
        );

        // The ten compound assignments C036 lists must all still parse, so the
        // rejection is specific to `%=` rather than to compound assignment.
        assert!(
            codes(
                "let mut a = 1; a += 1; a -= 1; a *= 1; a /= 1; a **= 1; \
                 a &= 1; a |= 1; a ^= 1; a <<= 1; a >>= 1"
            )
            .is_empty()
        );
    }
}

#[cfg(test)]
mod discard_binding_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c006_rejects_reading_a_discard_binding_but_allows_binding_to_it() {
        // C006 makes `_` accept a value WITHOUT creating a readable binding, so
        // reading it as an expression is a compile-time error.
        assert_eq!(
            codes("try { raise :x } catch _, context { _ }"),
            ["DISCARD_BINDING_READ"]
        );

        // Binding to `_` remains legal wherever binding patterns allow it, and
        // a sibling binding in the same clause is still readable.
        assert!(codes("try { raise :x } catch _, context { context.value }").is_empty());
        assert!(codes("let mut n = 0; for _ in [1, 2] { n = n + 1 }; n").is_empty());
    }
}

#[cfg(test)]
mod immutable_binding_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c006_makes_parameter_catch_and_iteration_bindings_immutable() {
        // C006 makes all three immutable unless their own declaration uses
        // `mut`. A parameter write previously reported an UNRESOLVED target,
        // because parameters were never declared in the analysis scope, so the
        // diagnostic named the wrong defect.
        assert_eq!(
            codes("class C { public fun m(p) { p = 9 } }"),
            ["BINDING_ASSIGN_TO_IMMUTABLE"]
        );
        assert_eq!(
            codes("let error = :outer; try { raise :x } catch error: Symbol, c { error = :other }"),
            ["BINDING_ASSIGN_TO_IMMUTABLE"]
        );
        assert_eq!(
            codes("for x in [1, 2] { x = 9 }"),
            ["BINDING_ASSIGN_TO_IMMUTABLE"]
        );

        // Reading any of them stays legal.
        assert!(codes("class C { public fun m(p) { p } }").is_empty());
        assert!(codes("try { raise :x } catch error: Symbol, c { error }").is_empty());
    }
}

#[cfg(test)]
mod fixed_local_type_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c004_fixes_a_local_type_and_rejects_implicit_widening() {
        // C005 makes an untyped initializer's precise static Type the fixed
        // local Type, and C004 forbids a later assignment from widening it.
        assert_eq!(
            codes("let mut value = 1; value = \"text\""),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
        assert_eq!(
            codes("let mut value: Integer = 1; value = :sym"),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );

        // Assigning the SAME Type is the ordinary case.
        assert!(codes("let mut value = 1; value = 2").is_empty());
        assert!(codes("let mut value: Integer = 1; value = 2").is_empty());

        // C005 names an explicit union as the way to ask for a wider cell, so
        // the check must not fire there. This is the escape hatch: without it
        // the diagnostic would be inescapable rather than a contract.
        assert!(codes("let mut value: String | Integer = 1; value = \"text\"").is_empty());
        assert!(codes("let mut value: Dynamic<Object> = 1; value = \"text\"").is_empty());

        // An initializer whose Type this pass cannot infer leaves the binding
        // UNCHECKED rather than guessed at, since a wrong rejection is worse
        // than a missed one.
        assert!(codes("class C {} let mut value = C.new(); value = 1").is_empty());
    }
}

#[cfg(test)]
mod static_type_observation_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c093_typeof_copies_the_static_type_at_that_program_point() {
        // IRIS-V1-TYPES-C093 makes `typeof(expression)` the operand's static
        // Type without evaluating it, which is what makes a Type OBSERVABLE
        // from source at all: annotate with it and see whether the initializer
        // is accepted.
        assert!(codes("let a = 1; let b: typeof(a) = 2").is_empty());
        assert_eq!(
            codes("let a = 1; let b: typeof(a) = :sym"),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
    }

    #[test]
    fn d360_and_d361_fix_the_types_of_a_tested_binding_and_a_negation() {
        // D-360: a condition does NOT narrow the tested binding, so `x` is
        // still Integer afterwards and a Bool initializer is rejected.
        assert_eq!(
            codes("let x = 1; if x { nil } else { nil }; let probe: typeof(x) = true"),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
        assert!(codes("let x = 1; if x { nil } else { nil }; let probe: typeof(x) = 2").is_empty());

        // D-361: `!x` ALWAYS has static Type Bool, whatever the operand is.
        assert_eq!(
            codes("let x = 1; let negated = !x; let probe: typeof(negated) = 1"),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
        assert!(codes("let x = 1; let negated = !x; let probe: typeof(negated) = true").is_empty());
    }
}

#[cfg(test)]
mod operand_union_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn d359_types_a_logical_operator_as_the_normalized_operand_union() {
        // D-359: `to_bool` decides only WHICH operand value is returned, so the
        // static result Type is the union of the reachable operands rather than
        // Bool. Both members must be accepted and Bool must not be.
        let prefix = "let left: String | Nil = nil; let r = left || \"fallback\"; ";
        assert!(codes(&format!("{prefix}let probe: typeof(r) = \"s\"")).is_empty());
        assert!(codes(&format!("{prefix}let probe: typeof(r) = nil")).is_empty());
        assert_eq!(
            codes(&format!("{prefix}let probe: typeof(r) = true")),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
        // A Type outside the union is rejected too, so the union is not simply
        // accepting everything.
        assert_eq!(
            codes(&format!("{prefix}let probe: typeof(r) = 1")),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
    }

    #[test]
    fn c005_lets_a_written_union_widen_the_declared_cell() {
        // A wider declared Type accepts a narrower value, in either order, and
        // normalization makes the member order irrelevant.
        assert!(codes("let mut v: String | Integer = 1; v = \"text\"").is_empty());
        assert!(codes("let mut v: Integer | String = \"text\"; v = 1").is_empty());
        // A Type outside the written union is still rejected.
        assert_eq!(
            codes("let mut v: String | Integer = 1; v = :sym"),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
    }
}

#[cfg(test)]
mod closure_header_tests {
    use crate::parse;

    #[test]
    fn a_closure_parameter_may_carry_a_type_annotation() {
        // `|` both separates union members and CLOSES a closure parameter list.
        // Adding the union level made `{ |x: Integer| 2 }` read the closing bar
        // as a union operator and consume it, so the header never terminated.
        assert!(parse("let cl = { |x: Integer| 2 }").program_accepted);
        assert!(parse("let cl = { |x| 2 }").program_accepted);
        assert!(parse("let cl = { |x: Integer, y: Bool| 2 }").program_accepted);

        // The RETURN annotation sits outside the parameter list, so a union is
        // legitimate there and must still parse.
        assert!(parse("let cl = { |x: Integer| -> Integer | Nil 2 }").program_accepted);

        // A binding annotation is unaffected by the suppression.
        assert!(parse("let mut v: String | Integer = 1").program_accepted);
    }
}

#[cfg(test)]
mod errata_named_code_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        parsed
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .chain(if parsed.program_accepted {
                analyze(&parsed.program)
                    .into_iter()
                    .map(|diagnostic| diagnostic.code)
                    .collect()
            } else {
                Vec::new()
            })
            .collect()
    }

    #[test]
    fn c078_rejects_a_legacy_form_where_a_v1_production_is_required() {
        for source in [
            "groan :x",
            "throw :x",
            "try { nil } rescue { nil }",
            "try { nil } ensure { nil }",
            "repeat { nil }",
            "switch value { when 1 { :one } }",
        ] {
            assert!(
                codes(source).contains(&"PARSE_LEGACY_FORM"),
                "not rejected: {source}"
            );
        }

        // D-509 leaves these spellings ordinary IDENTIFIERS, so the rejection is
        // context-specific: the v1 forms they were replaced by must still parse.
        assert!(codes("raise :x").is_empty());
        assert!(codes("try { nil } catch _ { nil }").is_empty());
        assert!(codes("while false { nil }").is_empty());
        assert!(codes("match 1 { 1 => :one, else => :other }").is_empty());
    }

    #[test]
    fn c078_rejects_a_declaration_rebinding_the_same_scope() {
        // A repeated `const` violates BOTH rules: it rebinds the name in scope
        // under C078, and it collides in the qualified namespace under D-432,
        // which puts constants alongside Class, Module and Contract names.
        assert_eq!(
            codes("const N = 1; const N = 2"),
            ["QUALIFIED_NAMESPACE_COLLISION", "DECLARATION_REBINDING"]
        );
        assert_eq!(codes("let a = 1; let a = 2"), ["DECLARATION_REBINDING"]);

        // Shadowing in an INNER scope stays legal, so the check is scoped rather
        // than a blanket ban on repeating a name.
        assert!(codes("let a = 1; if true { let a = 2; a } else { nil }").is_empty());
        assert!(codes("const N = 1; const M = 2").is_empty());
    }
}

#[cfg(test)]
mod errata_storage_and_default_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c078_rejects_a_parameter_default_referencing_a_later_parameter() {
        assert_eq!(
            codes("class C { public fun m(a = later, later = 1) { a } }"),
            ["PARAMETER_DEFAULT_FORWARD_REFERENCE"]
        );

        // A default referencing an EARLIER parameter is the legal form V337
        // already covers, so the check is directional rather than a ban on
        // parameter references in defaults.
        assert!(codes("class C { public fun m(a, b = a) { b } }").is_empty());
        assert!(codes("class C { public fun m(a, b = 2) { b } }").is_empty());
    }

    #[test]
    fn c078_rejects_assignment_to_absent_class_variable_storage() {
        assert_eq!(
            codes("class C { public fun m() { @@missing = 1 } }"),
            ["MISSING_DECLARED_STORAGE"]
        );

        // A `shared` declaration publishes the storage, and reading is never a
        // missing-storage error.
        assert!(codes("class C { shared mut @@x = 0 public fun m() { @@x = 1 } }").is_empty());
        assert!(codes("class C { shared mut @@x = 0 public fun m() { @@x } }").is_empty());
    }
}

#[cfg(test)]
mod call_parenthesis_tests {
    use crate::parse;

    fn codes(source: &str) -> Vec<&'static str> {
        parse(source)
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c078_requires_parentheses_for_an_ordinary_call() {
        assert_eq!(codes("f 1"), ["PARSE_CALL_REQUIRES_PARENTHESES"]);
        assert_eq!(
            codes("let o = 1; o.m 1"),
            ["PARSE_CALL_REQUIRES_PARENTHESES"]
        );

        // A NAMED INFIX has the same `name token token` shape but consumes the
        // middle token as its operator, so it must remain legal. Without this
        // the rejection would swallow `n mod 2`.
        assert!(codes("let n = 5; n mod 2").is_empty());
        assert!(codes("let o = 1; o.to_bool()").is_empty());
        assert!(codes("let a = 1; let b = 2").is_empty());
        assert!(codes("class C { public fun m() { 1 } }").is_empty());
    }
}

#[cfg(test)]
mod qualified_namespace_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "source must parse: {source}");
        analyze(&parsed.program)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn d432_puts_constants_and_declarations_in_one_qualified_namespace() {
        // D-432 puts Class, Module, Contract, Type aliases and constants in ONE
        // namespace, so a collision between any two of them is an error.
        assert_eq!(
            codes("const Name = 1; class Name {}"),
            ["QUALIFIED_NAMESPACE_COLLISION"]
        );
        assert_eq!(
            codes("class Name {} module Name { }"),
            ["QUALIFIED_NAMESPACE_COLLISION"]
        );

        // An ordinary `let` is a lexical binding rather than a namespace entry,
        // so it does not collide with a Class of the same name. Without this the
        // check would reject ordinary shadowing.
        assert!(codes("let Name = 1; class Name {}").is_empty());
        assert!(codes("const Name = 1; class Other {}").is_empty());
    }
}

#[cfg(test)]
mod generic_type_tests {
    use crate::{analyze, parse};

    fn codes(source: &str) -> Vec<&'static str> {
        let parsed = parse(source);
        parsed
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .chain(if parsed.program_accepted {
                analyze(&parsed.program)
                    .into_iter()
                    .map(|diagnostic| diagnostic.code)
                    .collect()
            } else {
                Vec::new()
            })
            .collect()
    }

    #[test]
    fn c061_requires_a_closed_generic_type_for_annotation_and_construction() {
        // C061 makes a bare generic Class name definition METADATA: it is not
        // an instance Type and not shorthand for `Box<Object>`.
        assert_eq!(
            codes("class Box<T> {} let raw: Box = Box.new()"),
            ["GENERIC_ARGUMENT_ARITY", "RAW_GENERIC_TYPE_FORBIDDEN"]
        );
        assert!(codes("class Box<T> {} let ok: Box<String> = 1").is_empty());

        // A NON-generic Class is unaffected, so the checks are specific to a
        // declared parameter list rather than to construction generally.
        assert!(codes("class Plain {} let p: Plain = Plain.new()").is_empty());
    }

    #[test]
    fn c020_rejects_square_brackets_for_generic_arguments() {
        // GRAMMAR-C020 makes angle brackets the accepted spelling, and V210
        // names the code for the square-bracket form.
        assert_eq!(
            codes("class Box<T> {} let item: Box[String] = 1"),
            ["GENERIC_BRACKET_SYNTAX_FORBIDDEN"]
        );

        // An ordinary index expression still uses square brackets, so the
        // rejection is confined to Type position.
        assert!(codes("let a = [1, 2]; a[0]").is_empty());
    }
}
