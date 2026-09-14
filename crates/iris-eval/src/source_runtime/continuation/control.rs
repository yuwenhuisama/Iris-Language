use super::*;

impl Continuation {
    pub(super) fn catch(
        &mut self,
        evaluator: &mut SourceEvaluator,
        catches: Vec<CatchClause>,
        finally: Option<Vec<Statement>>,
    ) -> Result<(), EvaluationError> {
        self.work.push(Step::Finally(finally));
        let raised = match &self.outcome {
            Err(EvaluationError::Raised(value)) => Some(value.clone()),
            Err(error) => super::super::catchable_name(error).map(Value::Symbol),
            Ok(_) => None,
        };
        let Some(value) = raised else {
            return Ok(());
        };
        for catch in catches {
            if !catch
                .filter
                .as_ref()
                .is_none_or(|filter| evaluator.catch_matches(&value, filter))
            {
                continue;
            }
            let mut locals = self.locals();
            if let Some(iris_syntax::CatchBinding::Name(name)) = catch.binding {
                locals.insert(name, value.clone());
            }
            if let Some(name) = catch.context {
                let context = match evaluator.active_context.clone() {
                    Some(context) => context,
                    None => Value::ExceptionContext(
                        evaluator.next_context_identity(),
                        Box::new(value.clone()),
                        Box::new(Value::Nil),
                        Vec::new(),
                        Vec::new(),
                        Box::new(evaluator.source_location(0).into()),
                    ),
                };
                locals.insert(name, context);
            }
            let previous = evaluator.active_exception.replace(value);
            self.work.push(Step::RestoreException(previous));
            self.enter(evaluator, catch.body, locals);
            return Ok(());
        }
        Ok(())
    }

    pub(super) fn finally(
        &mut self,
        evaluator: &mut SourceEvaluator,
        finally: Option<Vec<Statement>>,
    ) {
        if let Some(body) = finally {
            let pending = self.value();
            let raised = match &pending {
                Err(EvaluationError::Raised(value)) => Some(value.clone()),
                _ => None,
            };
            let context = raised.as_ref().and(evaluator.active_context.clone());
            let previous = std::mem::replace(&mut evaluator.active_exception, raised);
            self.work
                .push(Step::FinishFinally(pending, previous, context));
            self.enter(evaluator, body, self.locals());
        }
    }

    pub(super) fn finish_finally(
        &mut self,
        evaluator: &mut SourceEvaluator,
        pending: Outcome,
        previous: Option<Value>,
        context: Option<Value>,
    ) {
        evaluator.active_exception = previous;
        match &self.outcome {
            Ok(_) => {
                self.outcome = pending;
                evaluator.active_context = context;
            }
            Err(EvaluationError::Raised(raised)) => {
                if let Some(cause) = context {
                    evaluator.active_context = Some(Value::ExceptionContext(
                        evaluator.next_context_identity(),
                        Box::new(raised.clone()),
                        Box::new(cause),
                        Vec::new(),
                        Vec::new(),
                        Box::new(evaluator.source_location(0).into()),
                    ));
                }
            }
            Err(
                EvaluationError::Return(_)
                | EvaluationError::LoopBreak(..)
                | EvaluationError::LoopContinue(_),
            ) => {
                if let Err(EvaluationError::Raised(value)) = pending {
                    evaluator.discarded_contexts.push(value);
                }
            }
            Err(_) => {}
        }
    }
}
