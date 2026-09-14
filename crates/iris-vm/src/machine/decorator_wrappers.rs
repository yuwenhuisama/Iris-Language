use super::{Machine, MachineError};
use crate::compile::Program;
use iris_runtime::decorator_protocol::{ActivationId, ArgumentChanges, Invocation, NextScope};
use iris_runtime::{ClassId, DecoratorValue, ObjectId, Value};
use iris_syntax::TypeExpression;

#[derive(Clone)]
pub(super) struct WrapperChain {
    pub program: std::rc::Rc<Program>,
    pub function: usize,
    pub wrappers: Vec<ObjectId>,
    pub body_closure: Option<ObjectId>,
}

pub(super) struct NextContinuation {
    pub(super) scope: NextScope,
    pub(super) chain: WrapperChain,
    pub(super) layer: usize,
    pub(super) invocation: Invocation,
}

impl NextContinuation {
    pub(super) fn visit_values(&self, visitor: &mut impl FnMut(&Value)) {
        self.invocation.visit_values(visitor);
        self.scope.visit_values(visitor);
    }
}

impl Machine {
    pub(super) fn decorator_type_error(&mut self) -> MachineError {
        self.raise_core_value(Value::Symbol("TypeError".into()))
    }

    pub(super) fn admit_wrapper(
        &mut self,
        value: &Value,
        _program: &Program,
        is_async: bool,
    ) -> Result<ObjectId, MachineError> {
        let Value::Closure(identity) = value else {
            return Err(self.decorator_type_error());
        };
        let closure = self
            .closures
            .get(identity)
            .ok_or(MachineError::UnsupportedConstruct)?;
        let Some(signature) = &closure.signature else {
            return Err(self.decorator_type_error());
        };
        let next_type = TypeExpression::Generic {
            name: "Closure".into(),
            arguments: vec![TypeExpression::Function {
                parameters: vec![TypeExpression::Name("ArgumentChanges".into())],
                result: Box::new(if is_async {
                    TypeExpression::Generic {
                        name: "Task".into(),
                        arguments: vec![TypeExpression::Name("Object".into())],
                    }
                } else {
                    TypeExpression::Name("Object".into())
                }),
            }],
        };
        let valid = signature.is_async == is_async
            && signature.return_type == Some(TypeExpression::Name("Object".into()))
            && signature.parameters.len() == 2
            && signature.parameters.iter().all(|parameter| {
                parameter.category == iris_syntax::ParameterCategory::Positional
                    && parameter.default.is_none()
            })
            && signature.parameters[0].annotation
                == Some(TypeExpression::Name("Invocation".into()))
            && signature.parameters[1].annotation == Some(next_type);
        if !valid {
            return Err(self.decorator_type_error());
        }
        Ok(*identity)
    }

    pub(super) fn invoke_layer(
        &mut self,
        chain: WrapperChain,
        layer: usize,
        invocation: Invocation,
        _program: &Program,
        _classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        if invocation.selected().signature().is_async() {
            let owner = std::rc::Rc::clone(&chain.program);
            let owned_classes = owner
                .link
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?
                .classes
                .borrow()
                .clone();
            return self.invoke_async_layer(chain, layer, invocation, &owner, &owned_classes);
        }
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
        let result = if let Some(callback) = chain.wrappers.get(layer).copied() {
            let identity = ObjectId::new(self.next_closure);
            self.next_closure += 1;
            self.next_continuations.insert(
                identity,
                NextContinuation {
                    scope: NextScope::synchronous(
                        identity,
                        ActivationId::new(identity),
                        self.current_task,
                    ),
                    chain: chain.clone(),
                    layer: layer + 1,
                    invocation: invocation.clone(),
                },
            );
            let previous = std::mem::take(&mut self.method_types);
            self.active_values.push(Value::Closure(identity));
            let result = self.invoke_closure_value(
                callback,
                &[
                    DecoratorValue::Invocation(Box::new(invocation.clone())).into(),
                    Value::Closure(identity),
                ],
                program,
                classes,
            );
            self.method_types = previous;
            self.active_values.pop();
            if let Some(next) = self.next_continuations.get_mut(&identity) {
                next.scope.finish_owner();
            }
            result?
        } else {
            self.run_selected_body(chain.function, &invocation, program, classes)?
        };
        if !self.decorator_accepts(invocation.selected().signature().result(), &result) {
            return Err(self.decorator_type_error());
        }
        Ok(result)
    }

    pub(super) fn invoke_next(
        &mut self,
        identity: ObjectId,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        if self
            .next_continuations
            .get(&identity)
            .is_some_and(|next| next.invocation.selected().signature().is_async())
        {
            return self.invoke_async_next(identity, arguments, program, classes);
        }
        let next = self
            .next_continuations
            .get_mut(&identity)
            .ok_or(MachineError::UnsupportedConstruct)?;
        next.scope
            .begin_attempt(self.current_task, None)
            .map_err(|error| self.raise_core_value(DecoratorValue::ProtocolError(error).into()))?;
        let result = self.run_next_attempt(identity, arguments, program, classes);
        if let Some(next) = self.next_continuations.get_mut(&identity) {
            next.scope.finish_attempt();
        }
        result
    }

    pub(super) fn run_next_attempt(
        &mut self,
        identity: ObjectId,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        if arguments.iter().any(|argument| {
            matches!(
                argument,
                Value::KeywordArgument(..) | Value::BlockArgument(..)
            )
        }) {
            return Err(MachineError::ArgumentError);
        }
        let next = self
            .next_continuations
            .get(&identity)
            .ok_or(MachineError::UnsupportedConstruct)?;
        let changes = match arguments {
            [] => ArgumentChanges::empty(),
            [Value::Decorator(record)] => match record.as_ref() {
                DecoratorValue::ArgumentChanges(changes) => changes.clone(),
                _ => return Err(self.decorator_type_error()),
            },
            [_] => return Err(self.decorator_type_error()),
            _ => return Err(MachineError::ArgumentError),
        };
        let chain = next.chain.clone();
        let layer = next.layer;
        let invocation = next
            .invocation
            .patch(&changes, |target, value| {
                self.decorator_accepts(target, value)
            })
            .map_err(|error| self.payload_error(error))?;
        self.invoke_layer(chain, layer, invocation, program, classes)
    }

    pub(super) fn payload_error(
        &mut self,
        error: iris_runtime::decorator_protocol::ArgumentError,
    ) -> MachineError {
        if error.is_type_error() {
            self.decorator_type_error()
        } else {
            MachineError::ArgumentError
        }
    }

    pub(super) fn decorator_accepts(&self, target: &Value, value: &Value) -> bool {
        match target {
            Value::ComposedType(iris_runtime::ComposedType::Never) => false,
            Value::ComposedType(iris_runtime::ComposedType::Union(members)) => members
                .iter()
                .any(|member| self.decorator_atom_accepts(member, value)),
            Value::ComposedType(iris_runtime::ComposedType::Intersection(members)) => members
                .iter()
                .all(|member| self.decorator_atom_accepts(member, value)),
            _ => self.type_test(value, target) == Ok(Value::Bool(true)),
        }
    }

    fn decorator_atom_accepts(&self, atom: &iris_runtime::TypeAtom, value: &Value) -> bool {
        match atom {
            iris_runtime::TypeAtom::Nominal(class, arguments) => {
                self.decorator_accepts(&Value::Type(*class, arguments.clone()), value)
            }
            iris_runtime::TypeAtom::NonNil => !matches!(value, Value::Nil),
            iris_runtime::TypeAtom::Union(members) => members
                .iter()
                .any(|member| self.decorator_atom_accepts(member, value)),
            iris_runtime::TypeAtom::Intersection(members) => members
                .iter()
                .all(|member| self.decorator_atom_accepts(member, value)),
            iris_runtime::TypeAtom::Contract(_, _) | iris_runtime::TypeAtom::Iteration(_) => false,
        }
    }
}
