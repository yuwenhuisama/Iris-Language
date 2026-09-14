use super::decorator_wrappers::{NextContinuation, WrapperChain};
use super::task_adapters::{AdapterKind, TaskAdapter};
use super::{Machine, MachineError, PendingFrame};
use crate::compile::Program;
use iris_runtime::decorator_protocol::{Invocation, NextScope, TaskId};
use iris_runtime::{ClassId, DecoratorValue, ObjectId, Value};

impl Machine {
    pub(super) fn invoke_async_layer(
        &mut self,
        chain: WrapperChain,
        layer: usize,
        invocation: Invocation,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let admission = self.allocate_task(invocation.selected().signature().result().clone());
        self.start_async_layer(admission, chain, layer, invocation, program, classes)?;
        Ok(Value::Task(admission))
    }

    pub(super) fn start_async_layer(
        &mut self,
        admission: ObjectId,
        chain: WrapperChain,
        layer: usize,
        invocation: Invocation,
        _program: &Program,
        _classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let owner = std::rc::Rc::clone(&chain.program);
        let program = owner.as_ref();
        let owned_classes = program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .classes
            .borrow()
            .clone();
        let classes = owned_classes.as_slice();
        match chain.wrappers.get(layer).copied() {
            Some(callback) => {
                let object = self.reify_type(
                    &iris_syntax::TypeExpression::Name("Object".into()),
                    program,
                    classes,
                )?;
                let owner = self.allocate_task(object);
                let next = ObjectId::new(self.next_closure);
                self.next_closure += 1;
                self.next_continuations.insert(
                    next,
                    NextContinuation {
                        scope: NextScope::asynchronous(next, TaskId::new(owner)),
                        chain,
                        layer: layer + 1,
                        invocation: invocation.clone(),
                    },
                );
                self.task_owners.insert(owner, next);
                let closure = self
                    .closures
                    .get(&callback)
                    .cloned()
                    .ok_or(MachineError::UnsupportedConstruct)?;
                let program = closure.program.as_ref();
                let owned_classes = program
                    .link
                    .as_ref()
                    .ok_or(MachineError::UnsupportedConstruct)?
                    .classes
                    .borrow()
                    .clone();
                let classes = owned_classes.as_slice();
                let callee = &program.functions[closure.function];
                let mut arguments = closure.captures;
                arguments.push(DecoratorValue::Invocation(Box::new(invocation)).into());
                arguments.push(Value::Closure(next));
                let outcome = self.with_method_types(closure.method_types, |machine| {
                    machine.with_task(owner, |machine| {
                        machine
                            .run_body(
                                &callee.instructions,
                                callee.registers,
                                arguments,
                                program,
                                classes,
                            )
                            .map(|returned| returned.into_iter().next().unwrap_or(Value::Nil))
                    })
                });
                self.settle_task(owner, closure.function, outcome);
                self.link_task(TaskAdapter {
                    identity: admission,
                    source: owner,
                    kind: AdapterKind::Admission,
                });
            }
            None => {
                let outcome = self.with_task(admission, |machine| {
                    machine.run_selected_body(chain.function, &invocation, program, classes)
                });
                self.settle_task(admission, chain.function, outcome);
            }
        }
        Ok(())
    }

    pub(super) fn invoke_async_next(
        &mut self,
        next: ObjectId,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let object = self.reify_type(
            &iris_syntax::TypeExpression::Name("Object".into()),
            program,
            classes,
        )?;
        let bridge = self.allocate_task(object);
        let permission = self
            .next_continuations
            .get_mut(&next)
            .ok_or(MachineError::UnsupportedConstruct)?
            .scope
            .begin_attempt(self.current_task, Some(TaskId::new(bridge)));
        match permission {
            Err(error) => {
                let error = self.raise_core_value(DecoratorValue::ProtocolError(error).into());
                self.complete_task(bridge, Err(error));
            }
            Ok(()) => {
                let outcome = self.with_task(bridge, |machine| {
                    machine.run_next_attempt(next, arguments, program, classes)
                });
                match outcome {
                    Ok(Value::Task(inner)) => self.link_task(TaskAdapter {
                        identity: bridge,
                        source: inner,
                        kind: AdapterKind::Bridge(next),
                    }),
                    Ok(_) => return Err(MachineError::UnsupportedConstruct),
                    Err(error) => {
                        self.complete_task(bridge, Err(error));
                        if let Some(next) = self.next_continuations.get_mut(&next) {
                            next.scope.finish_attempt();
                        }
                    }
                }
            }
        }
        Ok(Value::Task(bridge))
    }

    pub(super) fn run_selected_body(
        &mut self,
        function: usize,
        invocation: &Invocation,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let callee = &program.functions[function];
        let mut registers = vec![invocation.selected().receiver().clone()];
        registers.extend(
            invocation
                .payload()
                .bindings(invocation.selected().signature())
                .map_err(|error| self.payload_error(error))?,
        );
        let mut bindings: Vec<_> = callee
            .signature
            .as_ref()
            .map(|signature| {
                signature
                    .type_parameters
                    .iter()
                    .cloned()
                    .zip(
                        invocation
                            .selected()
                            .method_type_arguments()
                            .iter()
                            .cloned(),
                    )
                    .collect()
            })
            .unwrap_or_default();
        bindings.extend(
            self.receiver_type_bindings(invocation.selected().receiver(), (program, classes))?,
        );
        let frame = PendingFrame {
            program: self.code.owner(program)?,
            method_types: bindings.clone(),
            gate: ObjectId::new(0),
            registers,
            counter: callee.body_entry,
            destination: None,
            handlers: Vec::new(),
            cleanup: None,
            function: Some(function),
            continuations: Vec::new(),
        };
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let result = self
            .run_frame_from(
                &callee.instructions,
                callee.registers,
                Vec::new(),
                program,
                classes,
                Some((frame, Value::Nil)),
            )
            .map(|returned| returned.into_iter().next().unwrap_or(Value::Nil))
            .map_err(|error| match error {
                MachineError::TypeContractError => self.decorator_type_error(),
                error => error,
            });
        self.method_types = previous;
        result
    }
}
