//! Statement and control-flow lowering.

use iris_syntax::{Expression, MatchBody, Pattern, Statement, TypeExpression};

use super::lowering::{Binding, LoopContext, Lowering};
use super::{CompileError, Instruction, Register};

impl<'a, 'b> Lowering<'a, 'b> {
    pub(super) fn statement(&mut self, statement: &Statement) -> Result<Register, CompileError> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            // A plain immutable binding is covered. `mut`, `const`, globals and
            // deferred bindings carry rules - reassignment, definite
            // assignment, package-qualified identity - this subset lacks.
            Statement::Binding {
                mutable,
                name,
                annotation,
                value,
                ..
            } => {
                // `C004` makes an annotated binding a GUARDED boundary, so
                // `let s: String = 1` raises when the binding runs. Treating
                // the annotation as static metadata answered the value instead
                // of the failure the language states.
                let value = self.expression(value)?;
                if let Some(annotation) = annotation {
                    self.instructions.push(Instruction::CheckAnnotation {
                        value,
                        annotation: annotation.clone(),
                    });
                }
                // A rebinding SHADOWS rather than overwrites: the earlier
                // register may still be read by a closure or an earlier
                // instruction, so reusing it would corrupt that read.
                //
                // A `mut` binding still gets ONE register, which assignment
                // then updates in place. That is what lets a loop carry a
                // value across iterations: a fresh register per assignment
                // would leave the loop reading its pre-loop value forever.
                let destination = self.allocate()?;
                if *mutable {
                    self.instructions.push(Instruction::MakeCell {
                        destination,
                        source: value,
                    });
                } else {
                    self.instructions.push(Instruction::Move {
                        destination,
                        source: value,
                    });
                }
                if self.method_values.contains(&value) {
                    self.method_values.push(destination);
                }
                self.names.push(if *mutable {
                    Binding::shared(name.clone(), destination)
                } else {
                    Binding::value(name.clone(), destination)
                });
                if self.top_level {
                    let published = self.allocate()?;
                    self.instructions.push(Instruction::PublishBinding {
                        destination: published,
                        name: name.clone(),
                        source: destination,
                    });
                }
                Ok(destination)
            }
            Statement::GlobalBinding {
                name,
                annotation: None,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::StoreGlobal {
                    destination,
                    name: name.clone(),
                    value,
                });
                Ok(destination)
            }
            // A deferred `let`, or an unannotated one, declares a name that is
            // never assignable, so every read of it fails definite assignment.
            // The reference still ACCEPTS the declaration - `let x: Integer`
            // on its own answers nil - so refusing it declined a program that
            // runs. Only `mut` with an annotation can later be written, which
            // is why that form alone carries an assigned flag.
            Statement::DeferredBinding { mutable, name, .. } => {
                let register = self.allocate()?;
                let assigned = self.allocate()?;
                self.instructions
                    .push(Instruction::DeclareDeferred { register, assigned });
                if *mutable {
                    self.names
                        .push(Binding::deferred(name.clone(), register, assigned));
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            // A loop is a BACKWARD jump, which is why the verifier had to
            // become a dataflow fixpoint: a body is entered before its own
            // writes have happened, so a linear scan cannot decide definite
            // assignment across the back edge.
            Statement::While {
                label,
                condition,
                body,
            } => self.while_value(label.as_deref(), condition, body),
            Statement::For {
                label,
                binding: iris_syntax::Pattern::Name(name),
                iterable,
                body,
            } => self.for_iterable(label.as_deref(), std::slice::from_ref(name), iterable, body),
            // `C051` lets the loop binding DESTRUCTURE each item, so
            // `for [a, b] in source` binds two names per iteration. Only a
            // flat array of names is lowered: a nested or literal sub-pattern
            // decides more than an index can express.
            Statement::For {
                label,
                binding: iris_syntax::Pattern::Array(elements),
                iterable,
                body,
            } => {
                let mut names = Vec::with_capacity(elements.len());
                for element in elements {
                    let iris_syntax::Pattern::Name(name) = element else {
                        return Err(CompileError::new("statement for"));
                    };
                    names.push(name.clone());
                }
                self.for_iterable(label.as_deref(), &names, iterable, body)
            }
            // An `if` yields a value, so both arms write the SAME destination
            // register. That is what lets the value be read afterwards without
            // knowing which arm ran.
            Statement::If {
                condition,
                then_body,
                else_body,
            } => self.if_value(condition, then_body, else_body.as_deref()),
            Statement::Return(value) => {
                // A RETURN needs a call to return FROM. At the top level there
                // is no frame to leave, so the reference refuses the program
                // rather than treating it as the script's value - answering
                // the operand made `return 1` a legal way to end a script.
                if self.top_level {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseUnsupported { destination });
                    return Ok(destination);
                }
                let value = match value {
                    Some(value) => self.expression(value)?,
                    None => {
                        let destination = self.allocate()?;
                        self.instructions.push(Instruction::LoadNil { destination });
                        destination
                    }
                };
                // A return bypasses the exception handlers that protect loop
                // bodies, so active iterators must be closed explicitly here.
                for iterator in self.loops.iter().rev().filter_map(|loop_| loop_.iterator) {
                    self.instructions.push(Instruction::IteratorClose {
                        iterator,
                        context: None,
                    });
                }
                // `C004` guards the return boundary on the EXPLICIT path too,
                // not only where the body falls off its end.
                if let Some(annotation) = self.return_annotation.clone() {
                    self.instructions
                        .push(Instruction::CheckReturn { value, annotation });
                }
                self.instructions.push(Instruction::Return { value });
                Ok(value)
            }
            // A LABELLED break unwinds to the loop that name belongs to rather
            // than the innermost one, which is the only way an inner loop can
            // stop an outer one and hand it a value.
            Statement::Break {
                label: Some(label),
                value,
            } => {
                let carried = match value {
                    Some(value) => Some(self.expression(value)?),
                    None => None,
                };
                let Some(index) = self
                    .loops
                    .iter()
                    .rposition(|context| context.label.as_deref() == Some(label.as_str()))
                else {
                    // A name no enclosing loop carries is a NameError the
                    // reference raises when the transfer runs.
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseNameError { destination });
                    return Ok(destination);
                };
                if let (Some(carried), Some(target)) = (carried, self.loops[index].value) {
                    self.instructions.push(Instruction::Move {
                        destination: target,
                        source: carried,
                    });
                }
                let jump = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });
                self.loops[index].breaks.push(jump);
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            // A `break` may carry an OPERAND, which becomes the loop's value:
            // `while true { break 7 }` answers 7 where a normal completion
            // answers nil.
            Statement::Break {
                label: None,
                value: Some(value),
            } => {
                let value = self.expression(value)?;
                let Some(target) = self.loops.last().and_then(|context| context.value) else {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseLoopTransfer { destination });
                    return Ok(destination);
                };
                self.instructions.push(Instruction::Move {
                    destination: target,
                    source: value,
                });
                let jump = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });
                let Some(loop_context) = self.loops.last_mut() else {
                    return Err(CompileError::new("loop context"));
                };
                loop_context.breaks.push(jump);
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Break {
                label: None,
                value: None,
            } => {
                let jump = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });
                let Some(loop_context) = self.loops.last_mut() else {
                    // A transfer with no target is a program error the
                    // reference reports when it RUNS, so the jump just emitted
                    // is replaced by the failure rather than declined.
                    self.instructions.pop();
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseLoopTransfer { destination });
                    return Ok(destination);
                };
                loop_context.breaks.push(jump);
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Continue(None) => {
                let Some(target) = self.loops.last().map(|context| context.continue_target) else {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseLoopTransfer { destination });
                    return Ok(destination);
                };
                self.instructions.push(Instruction::Jump { target });
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Raise(Some(raise)) => {
                let value = self.expression(&raise.value)?;
                let cause = match &raise.cause {
                    Some(cause) => Some(self.expression(cause)?),
                    None => self.exception_contexts.last().map(|(_, context)| *context),
                };
                self.instructions.push(Instruction::Raise {
                    value,
                    cause,
                    offset: raise.offset,
                });
                Ok(value)
            }
            Statement::Raise(None) => {
                if let Some((value, context)) = self.exception_contexts.last().copied() {
                    self.instructions.push(Instruction::ReRaise {
                        value,
                        context,
                        offset: 0,
                    });
                } else {
                    self.instructions.push(Instruction::RaiseNoActiveException);
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_body(body, catches, finally),
            Statement::Match {
                subject,
                arms,
                fallback,
            } => self.match_statement(subject, arms, fallback.as_ref()),
            // A top-level `fun`, and a `shared let` outside a class, are forms
            // the reference REFUSES when the program runs, as
            // UnsupportedConstruct - so refusing them here described the same
            // refusal differently and held the row.
            Statement::Method(_) | Statement::SharedBinding { .. } => {
                let destination = self.allocate()?;
                self.instructions
                    .push(Instruction::RaiseUnsupported { destination });
                Ok(destination)
            }
            other => Err(CompileError::new(format!(
                "statement {}",
                match other {
                    Statement::Binding { .. } => "binding",
                    Statement::Method(_) => "method",
                    Statement::GlobalBinding { .. } => "global",
                    Statement::SharedBinding { .. } => "shared",
                    Statement::DeferredBinding { .. } => "deferred",
                    Statement::StoredProperty { .. } => "stored property",
                    Statement::Match { .. } => "match",
                    Statement::For { .. } => "for",
                    Statement::Try { .. } => "try",
                    Statement::Raise(_) => "raise",
                    Statement::Break { .. } => "break",
                    Statement::Continue(_) => "continue",
                    _ => "other",
                }
            ))),
        }
    }

    /// Lowers a block, answering the register holding its last value.
    pub(super) fn body(&mut self, statements: &[Statement]) -> Result<Register, CompileError> {
        let Some((last, leading)) = statements.split_last() else {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadNil { destination });
            return Ok(destination);
        };
        // A block scopes its bindings: a name bound inside must not leak out.
        let outer = self.names.len();
        for statement in leading {
            self.statement(statement)?;
        }
        let value = self.statement(last)?;
        self.names.truncate(outer);
        Ok(value)
    }

    pub(super) fn for_iterable(
        &mut self,
        label: Option<&str>,
        names: &[String],
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadNil { destination });
        let iterable = self.expression(iterable)?;
        let iterator = self.allocate()?;
        self.instructions.push(Instruction::IteratorOpen {
            destination: iterator,
            iterable,
        });
        let item = self.allocate()?;
        let exception = self.allocate()?;
        let context = self.allocate()?;
        let protected = self.instructions.len();
        self.instructions.push(Instruction::EnterTry {
            handler: 0,
            cleanup: 0,
            exception,
            context,
        });
        let top = self.instructions.len();
        let next = self.instructions.len();
        self.instructions.push(Instruction::IteratorNext {
            destination: item,
            iterator,
            exhausted: 0,
        });
        let outer = self.names.len();
        // A single name takes the item ITSELF; a destructuring binding takes
        // its elements by index, which is what makes `for [a, b] in source`
        // bind two names from one yielded array.
        match names {
            [name] => self.names.push(Binding::value(name.clone(), item)),
            names => {
                for (position, name) in names.iter().enumerate() {
                    let element = self.allocate()?;
                    self.instructions.push(Instruction::DestructureElement {
                        destination: element,
                        item,
                        position,
                        arity: names.len(),
                    });
                    self.names.push(Binding::value(name.clone(), element));
                }
            }
        }
        self.loops.push(LoopContext {
            label: label.map(str::to_owned),
            continue_target: top,
            breaks: Vec::new(),
            iterator: Some(iterator),
            value: Some(destination),
        });
        self.body(body)?;
        let Some(loop_context) = self.loops.pop() else {
            return Err(CompileError::new("loop context"));
        };
        self.names.truncate(outer);
        self.instructions.push(Instruction::Jump { target: top });
        let close = self.instructions.len();
        match self.instructions.get_mut(next) {
            Some(Instruction::IteratorNext { exhausted, .. }) => *exhausted = close,
            _ => return Err(CompileError::new("branch patch")),
        }
        for jump in loop_context.breaks {
            self.patch(jump, close)?;
        }
        self.instructions.push(Instruction::LeaveTry);
        self.instructions.push(Instruction::IteratorClose {
            iterator,
            context: None,
        });
        let skip_handler = self.instructions.len();
        self.instructions.push(Instruction::Jump { target: 0 });
        let exceptional = self.instructions.len();
        if let Some(Instruction::EnterTry {
            handler, cleanup, ..
        }) = self.instructions.get_mut(protected)
        {
            *handler = exceptional;
            *cleanup = close;
        }
        self.instructions.push(Instruction::IteratorClose {
            iterator,
            context: Some(context),
        });
        self.instructions.push(Instruction::Propagate {
            value: exception,
            context,
        });
        let after = self.instructions.len();
        self.patch(skip_handler, after)?;
        Ok(destination)
    }

    pub(super) fn try_body(
        &mut self,
        body: &[Statement],
        catches: &[iris_syntax::CatchClause],
        finally: &Option<Vec<Statement>>,
    ) -> Result<Register, CompileError> {
        if catches.iter().any(|catch| {
            catch
                .filter
                .as_ref()
                .is_some_and(|filter| !matches!(filter, TypeExpression::Name(_)))
        }) {
            return Err(CompileError::new("try catch filter"));
        }
        let destination = self.allocate()?;
        let exception = self.allocate()?;
        let context = self.allocate()?;
        let enter = self.instructions.len();
        self.instructions.push(Instruction::EnterTry {
            handler: 0,
            cleanup: 0,
            exception,
            context,
        });
        let value = self.body(body)?;
        self.instructions.push(Instruction::LeaveTry);
        self.instructions.push(Instruction::Move {
            destination,
            source: value,
        });
        let normal_skip = self.instructions.len();
        self.instructions.push(Instruction::Jump { target: 0 });

        let handler = self.instructions.len();
        self.patch(enter, handler)?;
        let mut caught_skips = Vec::with_capacity(catches.len());
        for catch in catches {
            let mismatch = if let Some(TypeExpression::Name(class)) = &catch.filter {
                let matches = self.allocate()?;
                self.instructions.push(Instruction::CatchMatch {
                    destination: matches,
                    exception,
                    class: class.clone(),
                });
                let mismatch = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition: matches,
                    target: 0,
                });
                Some(mismatch)
            } else {
                None
            };
            let catch_enter = self.instructions.len();
            self.instructions.push(Instruction::EnterTry {
                handler: 0,
                cleanup: 0,
                exception,
                context,
            });
            let outer = self.names.len();
            if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
                self.names.push(Binding::value(name.clone(), exception));
            }
            if let Some(name) = &catch.context {
                self.names.push(Binding::value(name.clone(), context));
            }
            self.exception_contexts.push((exception, context));
            let caught = self.body(&catch.body)?;
            self.exception_contexts.pop();
            self.names.truncate(outer);
            self.instructions.push(Instruction::LeaveTry);
            self.instructions.push(Instruction::Move {
                destination,
                source: caught,
            });
            let caught_skip = self.instructions.len();
            self.instructions.push(Instruction::Jump { target: 0 });
            caught_skips.push(caught_skip);
            let exceptional_cleanup = self.instructions.len();
            self.patch(catch_enter, exceptional_cleanup)?;
            if let Some(finally) = finally {
                // `C067` makes a cleanup that RAISES chain the exception it
                // interrupted as the new one's cause, so the propagating
                // context travels with the cleanup body.
                self.instructions.push(Instruction::EnterCleanup {
                    context: Some(context),
                });
                self.body(finally)?;
                self.instructions
                    .push(Instruction::EnterCleanup { context: None });
            }
            self.instructions.push(Instruction::Propagate {
                value: exception,
                context,
            });
            if let Some(mismatch) = mismatch {
                let next = self.instructions.len();
                self.patch(mismatch, next)?;
            } else {
                break;
            }
        }
        if catches.last().is_none_or(|catch| catch.filter.is_some()) {
            if let Some(finally) = finally {
                self.instructions.push(Instruction::EnterCleanup {
                    context: Some(context),
                });
                self.body(finally)?;
                self.instructions
                    .push(Instruction::EnterCleanup { context: None });
            }
            self.instructions.push(Instruction::Propagate {
                value: exception,
                context,
            });
        }

        let cleanup = self.instructions.len();
        for caught_skip in caught_skips {
            self.patch(caught_skip, cleanup)?;
        }
        self.patch(normal_skip, cleanup)?;
        if let Some(Instruction::EnterTry { cleanup: slot, .. }) = self.instructions.get_mut(enter)
        {
            *slot = cleanup;
        }
        if let Some(finally) = finally {
            self.body(finally)?;
        }
        Ok(destination)
    }

    pub(super) fn match_statement(
        &mut self,
        subject: &Expression,
        arms: &[iris_syntax::MatchArm],
        fallback: Option<&MatchBody>,
    ) -> Result<Register, CompileError> {
        let subject = self.expression(subject)?;
        let destination = self.allocate()?;
        let mut exits = Vec::new();
        let mut previous_miss = None;
        for arm in arms {
            if let Some(miss) = previous_miss.take() {
                self.patch(miss, self.instructions.len())?;
            }
            let is_fallback = matches!(&arm.pattern, Pattern::Name(name) if name == "_");
            // A NAME pattern binds the subject for the arm, which is what lets
            // a guard test it: `match 5 { x if x > 3 => .. }`. It matches
            // unconditionally, so only the guard can turn the arm down.
            let bound = matches!(&arm.pattern, Pattern::Name(name) if name != "_");
            let outer = self.names.len();
            if bound {
                let Pattern::Name(name) = &arm.pattern else {
                    return Err(CompileError::new("match pattern"));
                };
                self.names.push(Binding::value(name.clone(), subject));
            }
            if !is_fallback && !bound {
                let Pattern::Literal(literal) = &arm.pattern else {
                    return Err(CompileError::new("match pattern"));
                };
                let expected = self.literal(literal)?;
                let matches = self.allocate()?;
                self.instructions.push(Instruction::Binary {
                    destination: matches,
                    selector: "==",
                    left: subject,
                    right: expected,
                });
                let miss = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition: matches,
                    target: 0,
                });
                previous_miss = Some(miss);
            }
            // A GUARD is tested after the pattern matched, and a false one
            // falls through to the next arm rather than failing the match.
            if let Some(guard) = &arm.guard {
                if let Some(miss) = previous_miss.take() {
                    self.patch(miss, self.instructions.len())?;
                }
                let condition = self.expression(guard)?;
                let condition = self.truth_test(condition)?;
                let miss = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition,
                    target: 0,
                });
                previous_miss = Some(miss);
            }
            let value = self.match_body(&arm.body)?;
            self.names.truncate(outer);
            self.instructions.push(Instruction::Move {
                destination,
                source: value,
            });
            let exit = self.instructions.len();
            self.instructions.push(Instruction::Jump { target: 0 });
            exits.push(exit);
            // An arm that matched UNCONDITIONALLY ends the match, but one
            // whose guard may turn it down does not: the arms after it are
            // still reachable.
            if is_fallback && arm.guard.is_none() {
                previous_miss = None;
                break;
            }
        }
        if let Some(miss) = previous_miss {
            self.patch(miss, self.instructions.len())?;
        }
        if let Some(fallback) = fallback {
            let value = self.match_body(fallback)?;
            self.instructions.push(Instruction::Move {
                destination,
                source: value,
            });
        } else {
            // Falling off the last arm means NO arm matched. Without a
            // fallback there is no value to answer, so the reference refuses
            // the match as UnsupportedConstruct - leaving the destination
            // unwritten instead made the machine read an undefined register
            // and report a defect where the language states a refusal.
            let refusal = self.allocate()?;
            self.instructions.push(Instruction::RaiseUnsupported {
                destination: refusal,
            });
            self.instructions.push(Instruction::Move {
                destination,
                source: refusal,
            });
        }
        let after = self.instructions.len();
        for exit in exits {
            self.patch(exit, after)?;
        }
        Ok(destination)
    }

    pub(super) fn match_body(&mut self, body: &MatchBody) -> Result<Register, CompileError> {
        match body {
            MatchBody::Expression(expression) => self.expression(expression),
            MatchBody::Block(statements) => self.body(statements),
        }
    }

    /// Lowers a `while`, in STATEMENT or expression position alike.
    ///
    /// `IRIS-V1-CONTROL-C023` gives a normal loop completion no value of its
    /// own, so the destination starts at nil and only a `break` carrying an
    /// operand writes it.
    pub(super) fn while_value(
        &mut self,
        label: Option<&str>,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<Register, CompileError> {
        // `IRIS-V1-CONTROL-C023` gives a normal loop completion no
        // value of its own, so the destination starts at nil and only
        // a `break` with an operand writes it.
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadNil { destination });
        let top = self.instructions.len();
        let condition = self.expression(condition)?;
        let condition = self.truth_test(condition)?;
        let exit = self.instructions.len();
        self.instructions.push(Instruction::JumpUnless {
            condition,
            target: 0,
        });
        self.loops.push(LoopContext {
            label: label.map(str::to_owned),
            continue_target: top,
            breaks: Vec::new(),
            iterator: None,
            value: Some(destination),
        });
        self.body(body)?;
        let Some(loop_context) = self.loops.pop() else {
            return Err(CompileError::new("loop context"));
        };
        self.instructions.push(Instruction::Jump { target: top });
        let after = self.instructions.len();
        self.patch(exit, after)?;
        for jump in loop_context.breaks {
            self.patch(jump, after)?;
        }
        Ok(destination)
    }

    /// Lowers an `if`, in STATEMENT or expression position alike.
    ///
    /// Both arms write ONE destination register, and a missing `else` writes
    /// nil, so the destination is written on every path - which is what keeps
    /// the verifier's written-before-read rule satisfied however the branch
    /// goes.
    pub(super) fn if_value(
        &mut self,
        condition: &Expression,
        then_body: &[Statement],
        else_body: Option<&[Statement]>,
    ) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        let condition = self.expression(condition)?;
        let condition = self.truth_test(condition)?;
        let branch = self.instructions.len();
        self.instructions.push(Instruction::JumpUnless {
            condition,
            target: 0,
        });
        let taken = self.body(then_body)?;
        self.instructions.push(Instruction::Move {
            destination,
            source: taken,
        });
        let skip = self.instructions.len();
        self.instructions.push(Instruction::Jump { target: 0 });

        let otherwise = self.instructions.len();
        match else_body {
            Some(body) => {
                let value = self.body(body)?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: value,
                });
            }
            None => self.instructions.push(Instruction::LoadNil { destination }),
        }
        let after = self.instructions.len();
        self.patch(branch, otherwise)?;
        self.patch(skip, after)?;
        Ok(destination)
    }

    /// Fills in a forward jump once its target is known.
    pub(super) fn patch(&mut self, at: usize, target: usize) -> Result<(), CompileError> {
        match self.instructions.get_mut(at) {
            Some(
                Instruction::JumpUnless { target: slot, .. }
                | Instruction::Jump { target: slot }
                | Instruction::EnterTry { handler: slot, .. },
            ) => {
                *slot = target;
                Ok(())
            }
            _ => Err(CompileError::new("branch patch")),
        }
    }
}
