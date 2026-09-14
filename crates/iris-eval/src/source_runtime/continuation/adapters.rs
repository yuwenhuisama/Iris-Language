use super::*;
use iris_runtime::decorator_protocol::{DecoratorValue, ScopeOwner};

pub(super) enum Adapter {
    Admission {
        next: Option<ObjectId>,
        result: Value,
    },
    Bridge,
}

impl Adapter {
    pub(super) fn roots(&self, roots: &mut Vec<Value>) {
        match self {
            Self::Admission { next, result } => {
                roots.extend(next.iter().map(|identity| Value::Closure(*identity)));
                roots.push(result.clone());
            }
            Self::Bridge => {}
        }
    }
}

impl Continuation {
    pub(super) fn finish_adapter(&mut self, evaluator: &mut SourceEvaluator, adapter: Adapter) {
        if let Err(EvaluationError::Return(value)) = &self.outcome {
            self.outcome = Ok(value.clone());
        }
        match adapter {
            Adapter::Admission { next, result } => {
                let unfinished = next
                    .and_then(|next| evaluator.wrapper_next.get(&next))
                    .and_then(|entry| entry.unfinished.clone());
                if let (Some(error), Ok(_)) = (unfinished, &self.outcome) {
                    self.outcome = Err(EvaluationError::Raised(
                        DecoratorValue::ProtocolError(error).into(),
                    ));
                    evaluator.active_context = None;
                }
                if let Ok(value) = &self.outcome
                    && !evaluator.wrapper_accepts(&result, value)
                {
                    self.outcome = Err(evaluator.wrapper_type_error());
                    evaluator.active_context = None;
                }
            }
            Adapter::Bridge => {}
        }
    }
}

impl SourceEvaluator {
    pub(in crate::source_runtime) fn fail_adapter_task(
        &mut self,
        identity: ObjectId,
        error: EvaluationError,
    ) {
        let mut continuation = Continuation::new(self, Vec::new(), HashMap::new());
        continuation.outcome = Err(error);
        self.run_continuation(identity, continuation);
    }

    pub(super) fn link_adapter(&mut self, identity: ObjectId, source: ObjectId, adapter: Adapter) {
        let mut continuation = Continuation::new(self, Vec::new(), HashMap::new());
        continuation.work = Stack::new(vec![Step::Adapter(adapter), Step::Await]);
        continuation.outcome = Ok(Value::Task(source));
        self.run_continuation(identity, continuation);
    }

    pub(super) fn finish_next_scopes(&mut self, identity: ObjectId) {
        for entry in self.wrapper_next.values_mut() {
            let mut scope = entry.scope.borrow_mut();
            if scope
                .bridge()
                .is_some_and(|bridge| bridge.object_id() == identity)
            {
                scope.finish_attempt();
            }
            if matches!(scope.owner(), ScopeOwner::Asynchronous(owner) if owner.object_id() == identity)
            {
                let bridge = scope.bridge();
                entry.unfinished = scope.finish_owner();
                entry.call = None;
                if entry.unfinished.is_some()
                    && self.tasks.get(&identity).is_some_and(Result::is_err)
                {
                    self.decorator_lifetime_diagnostics.push(Value::Tuple(vec![
                        Value::Symbol("IRIS-DECORATOR-PROTOCOL".into()),
                        Value::Symbol("error".into()),
                        Value::Symbol("runtime".into()),
                        Value::Symbol("unfinished_inner".into()),
                        Value::Task(identity),
                        Value::Closure(scope.next()),
                        bridge.map_or(Value::Nil, |task| Value::Task(task.object_id())),
                        self.task_contexts
                            .get(&identity)
                            .cloned()
                            .unwrap_or(Value::Nil),
                    ]));
                }
            }
        }
    }
}
