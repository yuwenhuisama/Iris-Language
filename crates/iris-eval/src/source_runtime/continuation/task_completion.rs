use super::*;

impl SourceEvaluator {
    pub(in crate::source_runtime) fn enqueue_waiters(&mut self, waiting: ObjectId) {
        let mut ready: Vec<_> = self
            .suspended
            .iter()
            .filter(|(_, task)| task.waiting == waiting)
            .map(|(identity, task)| (task.order, *identity))
            .collect();
        ready.sort_unstable();
        self.ready
            .extend(ready.into_iter().map(|(_, identity)| identity));
    }

    pub(in crate::source_runtime) fn drive_ready_continuations(
        &mut self,
    ) -> Result<(), EvaluationError> {
        while !self.ready.is_empty() {
            let identity = self.ready.remove(0);
            let Some(mut task) = self.suspended.remove(&identity) else {
                continue;
            };
            let previous_context = self.active_context.clone();
            task.continuation.outcome = self.await_value(task.waiting);
            if task.continuation.outcome.is_err() {
                task.continuation.exception_context =
                    self.task_contexts.get(&task.waiting).cloned();
            }
            self.active_context = previous_context;
            self.run_continuation(identity, task.continuation);
        }
        Ok(())
    }

    pub(super) fn await_value(&mut self, identity: ObjectId) -> Outcome {
        if let Some(value) = self.gates.get(&identity) {
            return value
                .clone()
                .ok_or(EvaluationError::AwaitSuspended(identity));
        }
        match self.tasks.get(&identity).cloned() {
            Some(Ok(value)) => Ok(value),
            Some(Err(error)) => {
                self.active_context = self.task_contexts.get(&identity).cloned();
                self.unobserved_failures
                    .retain(|(task, _)| *task != identity);
                Err(*error)
            }
            None => Err(EvaluationError::AwaitSuspended(identity)),
        }
    }
}
