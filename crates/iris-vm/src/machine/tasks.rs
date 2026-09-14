use super::{Machine, MachineError, SuspendedTask};
use crate::compile::{Instruction, Program};
use iris_runtime::decorator_protocol::TaskId;
use iris_runtime::{ClassId, KernelError, ObjectId, Value};

#[cfg(test)]
mod tests;

impl Machine {
    pub(super) fn with_task<T>(
        &mut self,
        identity: ObjectId,
        run: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let parent = self.current_task.replace(TaskId::new(identity));
        let root_base = self.active_values.len();
        self.active_values
            .extend(parent.map(|task| Value::Task(task.object_id())));
        self.async_depth += 1;
        let result = run(self);
        self.async_depth -= 1;
        self.current_task = parent;
        self.active_values.truncate(root_base);
        result
    }

    pub(super) fn spawn_task(
        &mut self,
        function: usize,
        registers: usize,
        instructions: &[Instruction],
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        if self.decorator_planning {
            return Err(MachineError::LexicalDiagnostic(
                "IRIS-DECORATOR-NONDETERMINISTIC",
            ));
        }
        let identity = ObjectId::new(self.next_context);
        self.next_context = self.next_context.saturating_add(1);
        if let Some(annotation) = program.functions[function]
            .signature
            .as_ref()
            .and_then(|signature| signature.return_type.as_ref())
        {
            let result = self.reify_type(annotation, program, classes)?;
            self.task_types.insert(identity, result);
        }
        let outcome = self.with_task(identity, |machine| {
            let returned =
                machine.run_body(instructions, registers, arguments, program, classes)?;
            Ok(returned.into_iter().next().unwrap_or(Value::Nil))
        });
        self.settle_task(identity, function, outcome);
        Ok(Value::Task(identity))
    }

    pub(super) fn settle_task(
        &mut self,
        identity: ObjectId,
        function: usize,
        outcome: Result<Value, MachineError>,
    ) {
        if matches!(outcome, Err(MachineError::Suspended(_))) {
            if let Some(mut frame) = self.pending_frame.take() {
                frame.function.get_or_insert(function);
                self.suspended.push(SuspendedTask {
                    identity,
                    frame,
                    function,
                });
            }
        } else {
            self.complete_task(identity, outcome);
        }
    }

    pub(super) fn drive_ready(
        &mut self,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        loop {
            self.settle_adapters();
            let ready: Vec<ObjectId> = self
                .suspended
                .iter()
                .filter(|task| {
                    matches!(self.gates.get(&task.frame.gate), Some(Some(_)))
                        || self.tasks.contains_key(&task.frame.gate)
                })
                .map(|task| task.frame.gate)
                .collect();
            if ready.is_empty() {
                break;
            }
            for gate in ready {
                self.resume_gate(gate, program, classes)?;
            }
        }
        Ok(())
    }

    pub(super) fn resume_gate(
        &mut self,
        gate: ObjectId,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let posted = self
            .gates
            .get(&gate)
            .cloned()
            .flatten()
            .unwrap_or(Value::Nil);
        let mut ready = Vec::new();
        let mut held = Vec::new();
        for task in std::mem::take(&mut self.suspended) {
            if task.frame.gate == gate {
                ready.push(task);
            } else {
                held.push(task);
            }
        }
        self.suspended = held;
        self.active_values.push(if self.gates.contains_key(&gate) {
            Value::Gate(gate)
        } else {
            Value::Task(gate)
        });
        let queue = self.ready_tasks.len();
        self.ready_tasks.push(ready.into());
        while let Some(task) = self.ready_tasks[queue].pop_front() {
            let SuspendedTask {
                identity,
                mut frame,
                function,
            } = task;
            let continuations = std::mem::take(&mut frame.continuations);
            let frames = std::iter::once(frame).chain(continuations);
            self.pending_frame = None;
            let outcome = self.with_task(identity, |machine| {
                machine.resume_task_frames(frames, posted.clone(), function, program, classes)
            });
            self.settle_task(identity, function, outcome);
            self.settle_adapters();
        }
        self.ready_tasks.pop();
        self.active_values.pop();
        Ok(())
    }

    pub(super) fn observe_task(
        &mut self,
        task: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Task(identity) = task else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        self.drive_ready(program, classes)?;
        let outcome = self
            .tasks
            .get(&identity)
            .cloned()
            .ok_or(MachineError::UnsupportedConstruct)?;
        self.mark_task_observed(identity);
        outcome.map_err(|error| *error)
    }
}
