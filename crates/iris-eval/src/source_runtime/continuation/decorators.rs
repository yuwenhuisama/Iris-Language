use super::adapters::Adapter;
use super::*;
use crate::source_runtime::wrapper_execution::{NextCall, NextEntry};
use iris_runtime::decorator_protocol::{ArgumentChanges, DecoratorValue, NextScope, TaskId};
use std::cell::RefCell;

impl SourceEvaluator {
    pub(in crate::source_runtime) fn start_async_wrapper(
        &mut self,
        identity: ObjectId,
        call: NextCall,
    ) -> Outcome {
        let _task = self.rooted(Value::Task(identity));
        let call = self.rooted(call);
        self.charge_step()?;
        let result = call.invocation.selected().signature().result().clone();
        match call.chain.wrappers.get(call.index).copied() {
            Some(wrapper) => {
                let callback_result = self.async_result_type(None)?;
                let owner = self.allocate_task(callback_result);
                let next = self.next_context_identity();
                let record = self
                    .closures
                    .get(&wrapper)
                    .cloned()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                let invocation =
                    DecoratorValue::Invocation(Box::new(call.invocation.clone())).into();
                self.wrapper_next.insert(
                    next,
                    NextEntry {
                        scope: Rc::new(RefCell::new(NextScope::asynchronous(
                            next,
                            TaskId::new(owner),
                        ))),
                        call: Some(Rc::new(NextCall {
                            index: call.index + 1,
                            ..Rc::try_unwrap(call).unwrap_or_else(|_| unreachable!())
                        })),
                        unfinished: None,
                    },
                );
                let mut locals = record.captured;
                for (parameter, value) in record
                    .parameters
                    .iter()
                    .zip([invocation, Value::Closure(next)])
                {
                    locals.insert(parameter.clone(), value);
                }
                let mut continuation = Continuation::new(self, record.body, locals);
                continuation.names.extend(record.cells);
                continuation.receiver = record.receiver;
                continuation.context = record.lexical_context;
                self.run_continuation(owner, continuation);
                self.link_adapter(
                    identity,
                    owner,
                    Adapter::Admission {
                        next: Some(next),
                        result,
                    },
                );
            }
            None => {
                let signature = call.invocation.selected().signature();
                let bindings = call
                    .invocation
                    .payload()
                    .bindings(signature)
                    .map_err(crate::source_runtime::decorator_errors::argument_error)?;
                let locals = signature
                    .parameters()
                    .iter()
                    .zip(bindings)
                    .map(|(parameter, value)| (parameter.name().to_owned(), value))
                    .collect();
                let body = call
                    .chain
                    .declaration
                    .body
                    .clone()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                let mut continuation = Continuation::new(self, body, locals);
                continuation
                    .work
                    .insert(0, Step::Adapter(Adapter::Admission { next: None, result }));
                continuation.receiver = Some(call.invocation.selected().receiver().clone());
                continuation.context = call.context.clone();
                self.run_continuation(identity, continuation);
            }
        }
        Ok(Value::Task(identity))
    }

    pub(in crate::source_runtime) fn invoke_async_wrapper_next(
        &mut self,
        identity: ObjectId,
        arguments: &[Value],
    ) -> Outcome {
        let bridge_result = self.async_result_type(None)?;
        let bridge = self.allocate_task(bridge_result);
        let entry = self
            .wrapper_next
            .get(&identity)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let scope = Rc::clone(&entry.scope);
        let call = entry.call.clone();
        let permission = scope.borrow_mut().begin_attempt(
            self.current_task.map(TaskId::new),
            Some(TaskId::new(bridge)),
        );
        if let Err(error) = permission {
            self.fail_adapter_task(
                bridge,
                EvaluationError::Raised(DecoratorValue::ProtocolError(error).into()),
            );
            return Ok(Value::Task(bridge));
        }
        let previous = self.current_task.replace(bridge);
        let outcome = (|| {
            let changes = match arguments {
                [] => ArgumentChanges::empty(),
                [Value::Decorator(record)] => match record.as_ref() {
                    DecoratorValue::ArgumentChanges(changes) => changes.clone(),
                    _ => return Err(self.wrapper_type_error()),
                },
                [Value::KeywordArgument(..) | Value::BlockArgument(_)] => {
                    return Err(EvaluationError::ArgumentError);
                }
                [_] => return Err(self.wrapper_type_error()),
                _ => return Err(EvaluationError::ArgumentError),
            };
            let call = call.ok_or(EvaluationError::UnsupportedConstruct)?;
            let invocation = call
                .invocation
                .patch(&changes, |annotation, value| {
                    self.wrapper_accepts(annotation, value)
                })
                .map_err(crate::source_runtime::decorator_errors::argument_error)?;
            let inner = self.allocate_task(invocation.selected().signature().result().clone());
            self.start_async_wrapper(
                inner,
                NextCall {
                    invocation,
                    chain: Rc::clone(&call.chain),
                    index: call.index,
                    context: call.context.clone(),
                },
            )?;
            Ok(inner)
        })();
        match outcome {
            Ok(inner) => self.link_adapter(bridge, inner, Adapter::Bridge),
            Err(error) => {
                let error = self.core_boundary_error(error);
                self.fail_adapter_task(bridge, error);
            }
        }
        self.current_task = previous;
        Ok(Value::Task(bridge))
    }
}
