use super::*;

impl Continuation {
    pub(super) fn statement(
        &mut self,
        evaluator: &mut SourceEvaluator,
        statement: Statement,
    ) -> Result<(), EvaluationError> {
        match statement {
            Statement::Binding {
                name,
                mutable,
                annotation,
                value,
                ..
            } => {
                let previous = self.value()?;
                self.work
                    .push(Step::Bind(name, mutable, annotation, previous));
                self.work.push(Step::Expression(value));
            }
            Statement::Expression(expression) => self.work.push(Step::Expression(expression)),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.work.push(Step::Branch(then_body, else_body));
                self.work.push(Step::Expression(condition));
            }
            Statement::While {
                label,
                condition,
                body,
            } => self.work.push(Step::While(Loop {
                label,
                condition,
                body,
            })),
            Statement::For {
                label,
                binding,
                iterable,
                body,
            } => {
                self.work.push(Step::ForStart(ForLoop {
                    label,
                    binding,
                    body,
                }));
                self.work.push(Step::Expression(iterable));
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => {
                self.work.push(Step::Try(catches, finally));
                self.enter(evaluator, body, self.locals());
            }
            Statement::Return(value) => {
                self.work.push(Step::Transfer(Statement::Return(None)));
                self.work.push(Step::Expression(
                    value.unwrap_or(Expression::Literal("nil".into())),
                ));
            }
            Statement::Break { label, value } => {
                self.work
                    .push(Step::Transfer(Statement::Break { label, value: None }));
                self.work.push(Step::Expression(
                    value.unwrap_or(Expression::Literal("nil".into())),
                ));
            }
            Statement::Raise(Some(raise)) => {
                let values = vec![
                    raise.value.clone(),
                    raise
                        .cause
                        .clone()
                        .unwrap_or(Expression::Literal("nil".into())),
                ];
                self.work
                    .push(Step::Transfer(Statement::Raise(Some(raise))));
                self.work.push(Step::Expression(Expression::Tuple(values)));
            }
            Statement::Match {
                subject,
                arms,
                fallback,
            } => {
                self.work.push(Step::Match(arms, fallback));
                self.work.push(Step::Expression(subject));
            }
            statement @ (Statement::DeferredBinding { .. }
            | Statement::Continue(_)
            | Statement::Raise(None)) => {
                self.outcome =
                    evaluator.statement(&statement, &self.locals(), self.receiver.clone())
            }
            Statement::InstanceField { .. }
            | Statement::GlobalBinding { .. }
            | Statement::SharedBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_) => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    pub(super) fn transfer(
        &mut self,
        evaluator: &mut SourceEvaluator,
        statement: Statement,
    ) -> Result<(), EvaluationError> {
        let value = self.value()?;
        self.outcome = match statement {
            Statement::Return(_) => Err(EvaluationError::Return(value)),
            Statement::Break { label, .. } => Err(EvaluationError::LoopBreak(label, value)),
            Statement::Raise(Some(mut raise)) => {
                let Value::Tuple(values) = value else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let mut locals = self.locals();
                locals.insert("\0raised".into(), values[0].clone());
                locals.insert("\0cause".into(), values[1].clone());
                raise.value = Expression::Name("\0raised".into());
                if raise.cause.is_some() {
                    raise.cause = Some(Expression::Name("\0cause".into()));
                }
                evaluator.statement(
                    &Statement::Raise(Some(raise)),
                    &locals,
                    self.receiver.clone(),
                )
            }
            _ => Err(EvaluationError::UnsupportedConstruct),
        };
        Ok(())
    }

    pub(super) fn loop_body(&mut self, loop_: Loop) {
        match &self.outcome {
            Err(EvaluationError::LoopBreak(target, value))
                if target.is_none() || *target == loop_.label =>
            {
                self.outcome = Ok(value.clone())
            }
            Ok(_) => {
                self.outcome = Ok(Value::Nil);
                self.work.push(Step::While(loop_));
            }
            Err(EvaluationError::LoopContinue(target))
                if target.is_none() || *target == loop_.label =>
            {
                self.outcome = Ok(Value::Nil);
                self.work.push(Step::While(loop_));
            }
            Err(_) => {}
        }
    }

    pub(super) fn for_body(&mut self, loop_: ForLoop, iterator: Value) {
        match &self.outcome {
            Err(EvaluationError::LoopBreak(target, value))
                if target.is_none() || *target == loop_.label =>
            {
                self.outcome = Ok(value.clone())
            }
            Ok(_) => {
                self.outcome = Ok(Value::Nil);
                self.work.push(Step::ForNext(loop_, iterator));
            }
            Err(EvaluationError::LoopContinue(target))
                if target.is_none() || *target == loop_.label =>
            {
                self.outcome = Ok(Value::Nil);
                self.work.push(Step::ForNext(loop_, iterator));
            }
            Err(_) => {}
        }
    }

    pub(super) fn for_next(
        &mut self,
        evaluator: &mut SourceEvaluator,
        loop_: ForLoop,
        iterator: Value,
    ) -> Result<(), EvaluationError> {
        evaluator.charge_step()?;
        match evaluator.send(iterator.clone(), "next", &[])? {
            Value::IterationDone => self.outcome = Ok(Value::Nil),
            Value::IterationYield(value) => {
                let mut locals = self.locals();
                if !evaluator.pattern_matches(&loop_.binding, &value, &mut locals)? {
                    return Err(EvaluationError::PatternMatchError);
                }
                self.work.push(Step::ForBody(loop_.clone(), iterator));
                self.enter(evaluator, loop_.body, locals);
            }
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    pub(super) fn match_arms(
        &mut self,
        evaluator: &mut SourceEvaluator,
        value: Value,
        mut arms: Vec<MatchArm>,
        fallback: Option<MatchBody>,
    ) -> Result<(), EvaluationError> {
        while !arms.is_empty() {
            let arm = arms.remove(0);
            let mut locals = self.locals();
            if !evaluator.pattern_matches(&arm.pattern, &value, &mut locals)? {
                continue;
            }
            self.scopes.push(locals);
            self.work.push(Step::ExitScope(evaluator.names.clone()));
            if let Some(guard) = arm.guard {
                self.work
                    .push(Step::MatchGuard(value, arms, fallback, arm.body));
                self.work.push(Step::Expression(guard));
            } else {
                self.match_body(evaluator, arm.body);
            }
            return Ok(());
        }
        match fallback {
            Some(body) => self.match_body(evaluator, body),
            None => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    pub(super) fn match_body(&mut self, evaluator: &SourceEvaluator, body: MatchBody) {
        match body {
            MatchBody::Expression(expression) => self.work.push(Step::Expression(expression)),
            MatchBody::Block(body) => self.enter(evaluator, body, self.locals()),
        }
    }
}
