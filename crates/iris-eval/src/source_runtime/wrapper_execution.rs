use super::wrapper_chain::WrapperChain;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{
    ActivationId, ArgumentChanges, DecoratorValue, Invocation, NextScope, TaskId,
};
use iris_runtime::{MethodOwner, ObjectId, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[path = "wrapper_context.rs"]
mod context;
pub(super) use context::LexicalContext;

pub(super) struct NextEntry {
    pub scope: Rc<RefCell<NextScope>>,
    pub call: Option<Rc<NextCall>>,
    pub unfinished: Option<iris_runtime::decorator_protocol::DecoratorProtocolError>,
}

impl NextEntry {
    pub fn is_async(&self) -> bool {
        matches!(
            self.scope.borrow().owner(),
            iris_runtime::decorator_protocol::ScopeOwner::Asynchronous(_)
        )
    }
}

pub(super) struct NextCall {
    pub invocation: Invocation,
    pub chain: Rc<WrapperChain>,
    pub index: usize,
    pub context: LexicalContext,
}

impl super::gc_roots::TraceRoots for NextCall {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.invocation
            .visit_values(&mut |value| roots.push(value.clone()));
        self.chain.trace_roots(roots);
        self.context.trace_roots(roots);
    }
}

impl SourceEvaluator {
    pub(super) fn invoke_wrapper_chain(
        &mut self,
        chain: Rc<WrapperChain>,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let previous = self.wrapper_context();
        let chain = self.rooted(chain);
        let _previous_root = self.rooted(previous.clone());
        if let Some(canonical) = &chain.canonical {
            self.restore_wrapper_context(canonical.context.clone());
        }
        self.current_method = Some(chain.original);
        if let MethodOwner::Class(class) = chain.original.owner() {
            self.lexical_class = Some(class);
            let (package, major) = self.class_identity(class);
            let package = package.to_owned();
            self.api_major = major;
            self.package = package;
            self.select_package_metadata();
        }
        let outer = chain
            .declaration
            .is_async
            .then(|| self.next_context_identity());
        let result = (|| {
            let owner_arguments = self.wrapper_owner_arguments(&chain, &receiver)?;
            self.select_wrapper_types(&chain.declaration, arguments)?;
            self.bind_wrapper_owner(&chain, &owner_arguments)?;
            if let Some(identity) = outer {
                let result = self.async_result_type(chain.declaration.return_type.as_ref())?;
                self.task_types.insert(identity, result);
            }
            let invocation = self.prepare_wrapper_invocation(&chain, receiver, arguments)?;
            let owner_types = invocation.selected().owner_type_arguments().to_vec();
            let chain = self.materialize_wrapper_chain(Rc::unwrap_or_clone(chain), owner_types)?;
            self.current_method = Some(chain.original);
            Ok((chain, invocation))
        })()
        .and_then(|(chain, invocation)| {
            let call = NextCall {
                invocation,
                chain,
                index: 0,
                context: self.wrapper_context(),
            };
            match outer {
                Some(identity) => self.start_async_wrapper(identity, call),
                None => self.run_wrapper_layer(call),
            }
        });
        let result = match (outer, result) {
            (Some(identity), Err(error)) => {
                let error = self.core_boundary_error(error);
                self.fail_adapter_task(identity, error);
                Ok(Value::Task(identity))
            }
            (_, result) => result,
        };
        self.restore_wrapper_context(previous);
        result.map_err(|error| self.core_boundary_error(error))
    }

    fn run_wrapper_layer(&mut self, call: NextCall) -> Result<Value, EvaluationError> {
        self.charge_step()?;
        self.invocation_depth += 1;
        let result = if self.invocation_depth > super::DEPTH_BUDGET {
            Err(EvaluationError::StepBudgetExhausted)
        } else {
            self.execute_wrapper_layer(call)
        };
        self.invocation_depth -= 1;
        result
    }

    fn execute_wrapper_layer(&mut self, call: NextCall) -> Result<Value, EvaluationError> {
        let call = self.rooted(call);
        let result_type = call.invocation.selected().signature().result().clone();
        let result = if let Some(wrapper) = call.chain.wrappers.get(call.index).copied() {
            let identity = self.next_context_identity();
            let scope = Rc::new(RefCell::new(NextScope::synchronous(
                identity,
                ActivationId::new(identity),
                self.current_task.map(TaskId::new),
            )));
            let invocation = DecoratorValue::Invocation(Box::new(call.invocation.clone())).into();
            self.wrapper_next.insert(
                identity,
                NextEntry {
                    scope: Rc::clone(&scope),
                    unfinished: None,
                    call: Some(Rc::new(NextCall {
                        index: call.index + 1,
                        ..Rc::try_unwrap(call).unwrap_or_else(|_| unreachable!())
                    })),
                },
            );
            let result = self.invoke_closure(wrapper, &[invocation, Value::Closure(identity)]);
            scope.borrow_mut().finish_owner();
            if let Some(entry) = self.wrapper_next.get_mut(&identity) {
                entry.call = None;
            }
            result
        } else {
            self.invoke_wrapper_original(&call)
        }?;
        if !self.wrapper_accepts(&result_type, &result) {
            return Err(self.wrapper_type_error());
        }
        Ok(result)
    }

    fn invoke_wrapper_original(&mut self, call: &NextCall) -> Result<Value, EvaluationError> {
        let signature = call.invocation.selected().signature();
        let bindings = call
            .invocation
            .payload()
            .bindings(signature)
            .map_err(super::decorator_errors::argument_error)?;
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
            .as_ref()
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let previous = self.wrapper_context();
        self.restore_wrapper_context(call.context.clone());
        let result = match self.block(
            body,
            &locals,
            Some(call.invocation.selected().receiver().clone()),
        ) {
            Err(EvaluationError::Return(value)) => Ok(value),
            result => result,
        };
        self.restore_wrapper_context(previous);
        result
    }

    pub(super) fn invoke_wrapper_next(
        &mut self,
        identity: ObjectId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        if self
            .wrapper_next
            .get(&identity)
            .is_some_and(NextEntry::is_async)
        {
            return self.invoke_async_wrapper_next(identity, arguments);
        }
        let entry = self
            .wrapper_next
            .get(&identity)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let scope = Rc::clone(&entry.scope);
        let call = entry.call.clone();
        scope
            .borrow_mut()
            .begin_attempt(self.current_task.map(TaskId::new), None)
            .map_err(|error| {
                EvaluationError::Raised(DecoratorValue::ProtocolError(error).into())
            })?;
        let result = (|| {
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
                .map_err(super::decorator_errors::argument_error)?;
            self.run_wrapper_layer(NextCall {
                invocation,
                chain: Rc::clone(&call.chain),
                index: call.index,
                context: call.context.clone(),
            })
        })();
        scope.borrow_mut().finish_attempt();
        result.map_err(|error| self.core_boundary_error(error))
    }
}
