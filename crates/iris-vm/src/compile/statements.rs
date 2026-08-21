//! Statement and control-flow lowering.

use iris_syntax::{Expression, MatchBody, Pattern, Statement, TypeExpression};

use super::lowering::{LoopContext, Lowering};
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
                annotation: None,
                value,
                ..
            } => {
                let _ = mutable;
                let value = self.expression(value)?;
                // A rebinding SHADOWS rather than overwrites: the earlier
                // register may still be read by a closure or an earlier
                // instruction, so reusing it would corrupt that read.
                //
                // A `mut` binding still gets ONE register, which assignment
                // then updates in place. That is what lets a loop carry a
                // value across iterations: a fresh register per assignment
                // would leave the loop reading its pre-loop value forever.
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: value,
                });
                self.names.push((name.clone(), destination));
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
            Statement::DeferredBinding {
                mutable: true,
                annotated: true,
                name,
            } => {
                let register = self.allocate()?;
                self.instructions
                    .push(Instruction::DeclareDeferred { register });
                self.names.push((name.clone(), register));
                self.deferred.push(name.clone());
                Ok(register)
            }
            // A loop is a BACKWARD jump, which is why the verifier had to
            // become a dataflow fixpoint: a body is entered before its own
            // writes have happened, so a linear scan cannot decide definite
            // assignment across the back edge.
            Statement::While {
                label: None,
                condition,
                body,
            } => {
                // The loop answers nil: `IRIS-V1-CONTROL-C023` gives a normal
                // loop completion no value of its own, and only a `break` with
                // an operand carries one - which this subset declines.
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                let top = self.instructions.len();
                let condition = self.expression(condition)?;
                let exit = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition,
                    target: 0,
                });
                self.body(body)?;
                self.instructions.push(Instruction::Jump { target: top });
                let after = self.instructions.len();
                self.patch(exit, after)?;
                Ok(destination)
            }
            Statement::For {
                label: None,
                binding: iris_syntax::Pattern::Name(name),
                iterable,
                body,
            } => self.for_array(name, iterable, body),
            // An `if` yields a value, so both arms write the SAME destination
            // register. That is what lets the value be read afterwards without
            // knowing which arm ran.
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let destination = self.allocate()?;
                let condition = self.expression(condition)?;
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
                    // A missing else answers nil, so the destination is
                    // written on EVERY path and the verifier's
                    // written-before-read rule holds however the branch goes.
                    None => self.instructions.push(Instruction::LoadNil { destination }),
                }
                let after = self.instructions.len();
                self.patch(branch, otherwise)?;
                self.patch(skip, after)?;
                Ok(destination)
            }
            Statement::Return(value) => {
                let value = match value {
                    Some(value) => self.expression(value)?,
                    None => {
                        let destination = self.allocate()?;
                        self.instructions.push(Instruction::LoadNil { destination });
                        destination
                    }
                };
                self.instructions.push(Instruction::Return { value });
                Ok(value)
            }
            Statement::Break {
                label: None,
                value: None,
            } => {
                let jump = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });
                let Some(loop_context) = self.loops.last_mut() else {
                    return Err(CompileError::new("break outside loop"));
                };
                loop_context.breaks.push(jump);
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Continue(None) => {
                let Some(target) = self.loops.last().map(|context| context.continue_target) else {
                    return Err(CompileError::new("continue outside loop"));
                };
                self.instructions.push(Instruction::Jump { target });
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                Ok(destination)
            }
            Statement::Raise(Some(raise)) if raise.cause.is_none() => {
                let value = self.expression(&raise.value)?;
                self.instructions.push(Instruction::Raise { value });
                Ok(value)
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
        // A deferred binding is discharged by an assignment that happens on
        // EVERY path, and a nested block is not every path: a branch may not
        // run at all. Restoring what was outstanding on entry means an
        // assignment inside a branch leaves the binding deferred, so the read
        // after it declines rather than lowering to a register the verifier
        // then proves unwritten - which is a compiler defect reported as a
        // machine failure, not a program error.
        let held = self.deferred.clone();
        for statement in leading {
            self.statement(statement)?;
        }
        let value = self.statement(last)?;
        self.deferred = held;
        self.names.truncate(outer);
        Ok(value)
    }

    pub(super) fn for_array(
        &mut self,
        name: &str,
        iterable: &Expression,
        body: &[Statement],
    ) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadNil { destination });
        let array = self.expression(iterable)?;
        let index = self.literal("0")?;
        let item = self.allocate()?;
        let top = self.instructions.len();
        let next = self.instructions.len();
        self.instructions.push(Instruction::ArrayNext {
            destination: item,
            array,
            index,
            exhausted: 0,
        });
        let one = self.literal("1")?;
        let advanced = self.allocate()?;
        self.instructions.push(Instruction::Binary {
            destination: advanced,
            selector: "+",
            left: index,
            right: one,
        });
        self.instructions.push(Instruction::Move {
            destination: index,
            source: advanced,
        });
        let outer = self.names.len();
        self.names.push((name.to_owned(), item));
        self.loops.push(LoopContext {
            continue_target: top,
            breaks: Vec::new(),
        });
        self.body(body)?;
        let Some(loop_context) = self.loops.pop() else {
            return Err(CompileError::new("loop context"));
        };
        self.names.truncate(outer);
        self.instructions.push(Instruction::Jump { target: top });
        let after = self.instructions.len();
        if let Some(Instruction::ArrayNext { exhausted, .. }) = self.instructions.get_mut(next) {
            *exhausted = after;
        } else {
            return Err(CompileError::new("branch patch"));
        }
        for jump in loop_context.breaks {
            self.patch(jump, after)?;
        }
        Ok(destination)
    }

    pub(super) fn try_body(
        &mut self,
        body: &[Statement],
        catches: &[iris_syntax::CatchClause],
        finally: &Option<Vec<Statement>>,
    ) -> Result<Register, CompileError> {
        if catches.iter().any(|catch| catch.context.is_some()) {
            return Err(CompileError::new("try exception context"));
        }
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
        let enter = self.instructions.len();
        self.instructions.push(Instruction::EnterTry {
            handler: 0,
            cleanup: 0,
            exception,
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
            });
            let outer = self.names.len();
            if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
                self.names.push((name.clone(), exception));
            }
            let caught = self.body(&catch.body)?;
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
                self.body(finally)?;
            }
            self.instructions
                .push(Instruction::Raise { value: exception });
            if let Some(mismatch) = mismatch {
                let next = self.instructions.len();
                self.patch(mismatch, next)?;
            } else {
                break;
            }
        }
        if catches.last().is_none_or(|catch| catch.filter.is_some()) {
            if let Some(finally) = finally {
                self.body(finally)?;
            }
            self.instructions
                .push(Instruction::Raise { value: exception });
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
            if arm.guard.is_some() {
                return Err(CompileError::new("match guard"));
            }
            if let Some(miss) = previous_miss.take() {
                self.patch(miss, self.instructions.len())?;
            }
            let is_fallback = matches!(&arm.pattern, Pattern::Name(name) if name == "_");
            if !is_fallback {
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
            let value = self.match_body(&arm.body)?;
            self.instructions.push(Instruction::Move {
                destination,
                source: value,
            });
            let exit = self.instructions.len();
            self.instructions.push(Instruction::Jump { target: 0 });
            exits.push(exit);
            if is_fallback {
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
        } else if exits.is_empty() {
            return Err(CompileError::new("match fallback"));
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
