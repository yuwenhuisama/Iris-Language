use super::*;

impl crate::Session {
    /// Returns runtime decorator lifetime diagnostics without observing Task failures.
    pub fn decorator_lifetime_diagnostics(&self) -> &[Value] {
        &self.evaluator.decorator_lifetime_diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_runtime::wrapper_execution::NextEntry;
    use iris_runtime::decorator_protocol::{NextScope, TaskId};
    use std::cell::RefCell;

    #[test]
    fn diagnostic_names_exact_callback_when_owner_finishes_with_pending_attempt() {
        let mut evaluator = SourceEvaluator::new_in_package("test").unwrap();
        let owner = evaluator.next_context_identity();
        let next = evaluator.next_context_identity();
        let bridge = evaluator.next_context_identity();
        let mut scope = NextScope::asynchronous(next, TaskId::new(owner));
        scope
            .begin_attempt(Some(TaskId::new(owner)), Some(TaskId::new(bridge)))
            .unwrap();
        evaluator.wrapper_next.insert(
            next,
            NextEntry {
                scope: Rc::new(RefCell::new(scope)),
                call: None,
                unfinished: None,
            },
        );

        evaluator.fail_adapter_task(
            owner,
            EvaluationError::Raised(Value::Symbol("primary".into())),
        );

        let context = evaluator.task_contexts[&owner].clone();
        assert_eq!(
            evaluator.decorator_lifetime_diagnostics,
            vec![Value::Tuple(vec![
                Value::Symbol("IRIS-DECORATOR-PROTOCOL".into()),
                Value::Symbol("error".into()),
                Value::Symbol("runtime".into()),
                Value::Symbol("unfinished_inner".into()),
                Value::Task(owner),
                Value::Closure(next),
                Value::Task(bridge),
                context,
            ])]
        );
        assert!(evaluator.wrapper_next[&next].scope.borrow().in_flight());
        assert_eq!(
            evaluator.unobserved_failures,
            vec![(owner, Value::Symbol("primary".into()))]
        );
    }
}

impl SourceEvaluator {
    pub(super) fn retain_task_failure_context(&mut self, outcome: &mut Outcome) {
        if outcome.is_err() {
            *outcome = std::mem::replace(outcome, Ok(Value::Nil))
                .map_err(|error| self.core_boundary_error(error));
        }
        let value = match outcome {
            Err(EvaluationError::Raised(value)) => value.clone(),
            Err(error) => match super::super::catchable_name(error) {
                Some(name) => Value::Symbol(name),
                None => return,
            },
            Ok(_) => return,
        };
        if matches!(&self.active_context, Some(Value::ExceptionContext(_, held, ..)) if held.as_ref() == &value)
        {
            return;
        }
        self.active_context = Some(Value::ExceptionContext(
            self.next_context_identity(),
            Box::new(value),
            Box::new(Value::Nil),
            Vec::new(),
            Vec::new(),
            Box::new(self.source_location(0).into()),
        ));
    }
}
