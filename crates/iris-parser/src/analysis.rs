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
    };
    analyzer.program(program);
    analyzer.diagnostics
}

/// One binding visible to later statements in the same scope.
struct Local {
    name: String,
    mutable: bool,
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
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Local {
                name: name.to_owned(),
                mutable,
            });
        }
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
        let body = match declaration {
            iris_syntax::Declaration::Class(value) => &value.body,
            iris_syntax::Declaration::Module(value) => &value.body,
            iris_syntax::Declaration::Contract(_) => return,
        };
        // A Class body holds Method declarations, each of which is its own
        // callable boundary and is entered through `Statement::Method`.
        self.scoped_body(body, Control::top_level());
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
                name,
                value,
            } => {
                self.expression(value, control);
                self.declare(name, *mutable);
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
            Statement::SharedBinding { name, mutable, .. } => self.declare(name, *mutable),
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
                    self.declare(&parameter.name, false);
                }
                for statement in &declaration.body {
                    self.statement(statement, Control::callable());
                }
                self.scopes.pop();
            }
            Statement::StoredProperty { initializer, .. } => {
                self.expression(initializer, control);
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
                if let Expression::Name(name) = left.as_ref() {
                    match self.lookup(name) {
                        None => self.report("BINDING_UNRESOLVED_ASSIGNMENT_TARGET"),
                        Some(local) if !local.mutable => {
                            self.report("BINDING_ASSIGN_TO_IMMUTABLE");
                        }
                        Some(_) => {}
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
                self.expression(callee, control);
                for argument in arguments {
                    self.expression(argument, control);
                }
            }
            Expression::Index { receiver, index } => {
                self.expression(receiver, control);
                self.expression(index, control);
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
