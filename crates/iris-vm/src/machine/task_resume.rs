use super::{Machine, MachineError, PendingFrame, VerifyError};
use crate::compile::Program;
use iris_runtime::{ClassId, Value};

impl Machine {
    pub(super) fn resume_task_frames(
        &mut self,
        frames: impl Iterator<Item = PendingFrame>,
        mut value: Value,
        owner_function: usize,
        _program: &Program,
        _classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let queue = self.resuming_frames.len();
        self.resuming_frames.push(frames.collect());
        let result = (|| {
            while let Some(mut frame) = self.resuming_frames[queue].pop_front() {
                let owner = std::rc::Rc::clone(&frame.program);
                let program = owner.as_ref();
                let owned_classes = program
                    .link
                    .as_ref()
                    .ok_or(MachineError::UnsupportedConstruct)?
                    .classes
                    .borrow()
                    .clone();
                let classes = owned_classes.as_slice();
                let function = frame.function.unwrap_or(owner_function);
                let cleanup = frame.cleanup.take();
                let callee = program
                    .functions
                    .get(function)
                    .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                        function,
                    }))?;
                let bindings = std::mem::take(&mut frame.method_types);
                let (cleanup, outcome) = self.with_local_roots(cleanup, |machine, _| {
                    machine.with_method_types(bindings, |machine| {
                        machine
                            .run_frame_from(
                                &callee.instructions,
                                callee.registers,
                                Vec::new(),
                                program,
                                classes,
                                Some((frame, value)),
                            )
                            .map(|returned| returned.into_iter().next().unwrap_or(Value::Nil))
                    })
                });
                match outcome {
                    Err(MachineError::Suspended(waiting)) => {
                        if let Some(frame) = self.pending_frame.as_mut() {
                            frame.function.get_or_insert(function);
                            frame.cleanup = cleanup;
                            frame
                                .continuations
                                .extend(self.resuming_frames[queue].drain(..));
                        }
                        return Err(MachineError::Suspended(waiting));
                    }
                    outcome => {
                        value = match cleanup {
                            Some(resource) => {
                                self.close_after(resource, outcome, program, classes)?
                            }
                            None => outcome?,
                        };
                    }
                }
            }
            Ok(value)
        })();
        self.resuming_frames.pop();
        result
    }
}
