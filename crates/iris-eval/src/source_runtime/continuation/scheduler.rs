use super::*;

impl SourceEvaluator {
    pub(in crate::source_runtime) fn start_async(
        &mut self,
        body: Vec<Statement>,
        locals: HashMap<String, Value>,
        receiver: Option<Value>,
        return_type: Option<&TypeExpression>,
    ) -> Outcome {
        let result = self.async_result_type(return_type)?;
        let identity = self.allocate_task(result);
        let mut continuation = Continuation::new(self, body, locals);
        continuation.receiver = receiver;
        self.run_continuation(identity, continuation);
        Ok(Value::Task(identity))
    }

    pub(super) fn run_continuation(&mut self, identity: ObjectId, mut continuation: Continuation) {
        let previous_task = self.current_task.replace(identity);
        let _previous_task = self.rooted(previous_task.map(Value::Task));
        let previous_source = std::mem::replace(&mut self.source, continuation.source.clone());
        let previous_names = std::mem::replace(&mut self.names, continuation.names.clone());
        let previous_names = self.rooted(previous_names);
        let previous_context = self.wrapper_context();
        self.restore_wrapper_context(continuation.context.clone());
        let previous_exception =
            std::mem::replace(&mut self.active_exception, continuation.exception.take());
        let previous_exception_context = std::mem::replace(
            &mut self.active_context,
            continuation.exception_context.take(),
        );
        let previous_exception = self.rooted(previous_exception);
        let previous_exception_context = self.rooted(previous_exception_context);
        let _previous_context = self.rooted(previous_context.clone());
        let _context = self.rooted(continuation.context.clone());
        self.async_depth += 1;
        let _work = self.rooted(continuation.work.clone());
        let _scopes = self.rooted(continuation.scopes.clone());
        let waiting = loop {
            let _receiver = self.rooted(match &continuation.receiver {
                Some(Value::Object(identity)) => Some(Value::Object(*identity)),
                _ => None,
            });
            let Some(step) = continuation.work.pop() else {
                break None;
            };
            if let Err(error) = continuation.step(self, step) {
                continuation.outcome = Err(error);
            }
            self.retain_task_failure_context(&mut continuation.outcome);
            if let Err(EvaluationError::AwaitSuspended(waiting)) = &continuation.outcome {
                break Some(*waiting);
            }
        };
        self.retain_task_failure_context(&mut continuation.outcome);
        self.async_depth -= 1;
        continuation.names =
            std::mem::replace(&mut self.names, Rc::unwrap_or_clone(previous_names));
        continuation.context = self.wrapper_context();
        self.restore_wrapper_context(previous_context);
        continuation.exception = std::mem::replace(
            &mut self.active_exception,
            Rc::unwrap_or_clone(previous_exception),
        );
        continuation.exception_context = std::mem::replace(
            &mut self.active_context,
            Rc::unwrap_or_clone(previous_exception_context),
        );
        self.current_task = previous_task;
        self.source = previous_source;
        match waiting {
            Some(waiting) => {
                self.suspension_order += 1;
                self.suspended.insert(
                    identity,
                    SuspendedTask {
                        continuation,
                        waiting,
                        order: self.suspension_order,
                    },
                );
            }
            None => {
                let outcome = match continuation.outcome {
                    Err(EvaluationError::Return(value)) => Ok(value),
                    outcome => outcome,
                };
                if let Err(error) = &outcome {
                    if let Some(context) = continuation.exception_context {
                        self.task_contexts.insert(identity, context);
                    }
                    let captured = match error {
                        EvaluationError::Raised(value) => value.clone(),
                        other => super::super::catchable_name(other)
                            .map_or(Value::Symbol("AsyncFailure".into()), Value::Symbol),
                    };
                    self.unobserved_failures.push((identity, captured));
                }
                self.tasks.insert(identity, outcome.map_err(Box::new));
                self.finish_next_scopes(identity);
                self.enqueue_waiters(identity);
            }
        }
    }
}

impl Continuation {
    fn step(&mut self, evaluator: &mut SourceEvaluator, step: Step) -> Result<(), EvaluationError> {
        match step {
            Step::Adapter(adapter) => self.finish_adapter(evaluator, adapter),
            Step::ExitScope(names) => {
                self.scopes.pop();
                evaluator.names = names;
            }
            Step::RestoreException(previous) => evaluator.active_exception = previous,
            Step::RestoreCall(receiver, context, names) => {
                self.receiver = receiver;
                evaluator.restore_wrapper_context(context);
                evaluator.names = names;
                if let Err(EvaluationError::Return(value)) = &self.outcome {
                    self.outcome = Ok(value.clone());
                }
            }
            Step::Close(resource) => {
                let outcome = self.value();
                self.outcome = evaluator.close_after(resource, outcome);
            }
            Step::Try(catches, finally) => self.catch(evaluator, catches, finally)?,
            Step::Finally(finally) => self.finally(evaluator, finally),
            Step::FinishFinally(pending, previous, context) => {
                self.finish_finally(evaluator, pending, previous, context)
            }
            Step::LoopBody(loop_) => self.loop_body(loop_),
            Step::ForBody(loop_, iterator) => self.for_body(loop_, iterator),
            step if self.outcome.is_err() => {
                drop(step);
            }
            Step::Block(body, position) => {
                if let Some(statement) = body.get(position).cloned() {
                    self.work.push(Step::Block(body, position + 1));
                    self.work.push(Step::Statement(statement));
                }
            }
            Step::Statement(statement) => self.statement(evaluator, statement)?,
            Step::Expression(expression) => self.expression(evaluator, expression)?,
            Step::Bind(name, mutable, annotation, previous) => {
                let value = self.value()?;
                if let Some(annotation) = &annotation {
                    evaluator.check_binding_annotation(&value, annotation)?;
                }
                if mutable {
                    let mut binding = Binding::new(value, true);
                    binding.contract = annotation;
                    evaluator.names.insert(name.clone(), binding);
                    if let Some(mut locals) = self.scopes.last_mut() {
                        locals.remove(&name);
                    }
                } else if let Some(mut locals) = self.scopes.last_mut() {
                    locals.insert(name, value);
                }
                self.outcome = Ok(previous);
            }
            Step::Operands(operands) => self.operands(evaluator, operands)?,
            Step::Operand(mut operands, name) => {
                operands.values.insert(name, self.value()?);
                self.work.push(Step::Operands(operands));
            }
            Step::Assign(operands, current) => self.assign(evaluator, operands, current)?,
            Step::Await => {
                if evaluator.open_target.is_some() {
                    return Err(EvaluationError::MetaTransactionSuspension);
                }
                self.outcome = match self.value()? {
                    Value::Gate(identity) | Value::Task(identity) => {
                        evaluator.await_value(identity)
                    }
                    _ => Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
            }
            Step::NonNull => {
                let value = self.value()?;
                self.outcome = if value == Value::Nil {
                    Err(evaluator.core_boundary_error(EvaluationError::Runtime(
                        iris_runtime::KernelError::Type,
                    )))
                } else {
                    Ok(value)
                };
            }
            Step::Branch(then_body, else_body) => {
                let test = self.value()?;
                let body = if evaluator.truthy(test)? {
                    then_body
                } else {
                    else_body.unwrap_or_default()
                };
                self.enter(evaluator, body, self.locals());
            }
            Step::Logical(operator, right) => {
                let value = self.value()?;
                let truthy = evaluator.truthy(value.clone())?;
                let evaluate_right = match operator {
                    iris_syntax::BinaryOperator::LogicalAnd => truthy,
                    iris_syntax::BinaryOperator::LogicalOr => !truthy,
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                if evaluate_right {
                    self.work.push(Step::Expression(right));
                } else {
                    self.outcome = Ok(value);
                }
            }
            Step::Transfer(statement) => self.transfer(evaluator, statement)?,
            Step::While(loop_) => {
                evaluator.charge_step()?;
                self.work.push(Step::WhileTest(loop_.clone()));
                self.work.push(Step::Expression(loop_.condition));
            }
            Step::WhileTest(loop_) => {
                let test = self.value()?;
                if evaluator.truthy(test)? {
                    self.work.push(Step::LoopBody(loop_.clone()));
                    self.enter(evaluator, loop_.body, self.locals());
                }
            }
            Step::ForStart(loop_) => {
                let source = self.value()?;
                let iterator = evaluator.send(source, "iterator", &[])?;
                self.work.push(Step::Close(iterator.clone()));
                self.work.push(Step::ForNext(loop_, iterator));
            }
            Step::ForNext(loop_, iterator) => self.for_next(evaluator, loop_, iterator)?,
            Step::Match(arms, fallback) => {
                let value = self.value()?;
                self.match_arms(evaluator, value, arms, fallback)?;
            }
            Step::MatchGuard(value, arms, fallback, body) => {
                let value = evaluator.rooted(value);
                let test = self.value()?;
                if evaluator.truthy(test)? {
                    self.match_body(evaluator, body);
                } else {
                    self.scopes.pop();
                    self.work.pop();
                    self.match_arms(evaluator, Rc::unwrap_or_clone(value), arms, fallback)?;
                }
            }
        }
        Ok(())
    }
}
