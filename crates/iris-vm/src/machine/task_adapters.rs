use super::{Machine, MachineError};
use iris_runtime::{DecoratorValue, ObjectId, Value};

pub(super) enum AdapterKind {
    Admission,
    Bridge(ObjectId),
}

pub(super) struct TaskAdapter {
    pub identity: ObjectId,
    pub source: ObjectId,
    pub kind: AdapterKind,
}

impl Machine {
    pub(super) fn allocate_task(&mut self, result: Value) -> ObjectId {
        let identity = ObjectId::new(self.next_context);
        self.next_context = self.next_context.saturating_add(1);
        self.task_types.insert(identity, result);
        identity
    }

    pub(super) fn mark_task_observed(&mut self, identity: ObjectId) {
        self.observed_tasks.insert(identity);
        self.unobserved_failures.retain(|held| *held != identity);
    }

    pub(super) fn complete_task(
        &mut self,
        identity: ObjectId,
        mut outcome: Result<Value, MachineError>,
    ) {
        if let Some(next_id) = self.task_owners.remove(&identity) {
            let next = self.next_continuations.get_mut(&next_id);
            if let Some(next) = next {
                let bridge = next.scope.bridge();
                if let Some(error) = next.scope.finish_owner() {
                    match &outcome {
                        Ok(_) => {
                            outcome =
                                Err(self
                                    .raise_core_value(DecoratorValue::ProtocolError(error).into()))
                        }
                        Err(primary) => {
                            let context = match primary {
                                MachineError::Raised(raised) => raised.1.clone(),
                                _ => Value::Nil,
                            };
                            self.decorator_lifetime_diagnostics.push(Value::Tuple(vec![
                                Value::Symbol("IRIS-DECORATOR-PROTOCOL".into()),
                                Value::Symbol("error".into()),
                                Value::Symbol("runtime".into()),
                                Value::Symbol("unfinished_inner".into()),
                                Value::Task(identity),
                                Value::Closure(next_id),
                                bridge.map_or(Value::Nil, |task| Value::Task(task.object_id())),
                                context,
                            ]));
                        }
                    }
                }
            }
        }
        if let (Some(result), Ok(value)) = (self.task_types.get(&identity), &outcome)
            && !self.decorator_accepts(result, value)
        {
            outcome = Err(self.decorator_type_error());
        }
        if matches!(outcome, Err(MachineError::TypeContractError))
            && self.task_types.contains_key(&identity)
        {
            outcome = Err(self.decorator_type_error());
        }
        if outcome.is_err() && !self.observed_tasks.contains(&identity) {
            self.unobserved_failures.push(identity);
        }
        self.tasks.insert(identity, outcome.map_err(Box::new));
    }

    pub(super) fn link_task(&mut self, adapter: TaskAdapter) {
        self.mark_task_observed(adapter.source);
        self.task_adapters.push(adapter);
        self.settle_adapters();
    }

    pub(super) fn settle_adapters(&mut self) {
        while let Some(index) = self
            .task_adapters
            .iter()
            .position(|adapter| self.tasks.contains_key(&adapter.source))
        {
            let adapter = self.task_adapters.remove(index);
            let outcome = self.tasks[&adapter.source].clone().map_err(|error| *error);
            self.complete_task(adapter.identity, outcome);
            match adapter.kind {
                AdapterKind::Admission => {}
                AdapterKind::Bridge(next) => {
                    if let Some(next) = self.next_continuations.get_mut(&next) {
                        next.scope.finish_attempt();
                    }
                }
            }
        }
    }

    pub fn decorator_lifetime_diagnostics(&self) -> &[Value] {
        &self.decorator_lifetime_diagnostics
    }
}
