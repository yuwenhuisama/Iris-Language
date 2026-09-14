use super::{Machine, MachineError};
use crate::compile::Program;
use iris_runtime::{CallableKind, CallableSignature, ClassId, NominalType, SignatureType, Value};
use iris_syntax::{MethodDeclaration, TypeExpression};

impl Machine {
    pub(super) fn register_callable_types(
        &mut self,
        _program: &Program,
    ) -> Result<(), MachineError> {
        self.kernel
            .register_decorator_classes(self.runtime.registry_mut())
            .map_err(MachineError::Kernel)?;
        Ok(())
    }

    pub(super) fn reify_callable(
        &mut self,
        expression: &TypeExpression,
        context: (&Program, &[ClassId]),
    ) -> Result<Value, MachineError> {
        let TypeExpression::Generic { name, arguments } = expression else {
            return Err(MachineError::UnsupportedConstruct);
        };
        let [TypeExpression::Function { parameters, result }] = arguments.as_slice() else {
            return Err(MachineError::UnsupportedConstruct);
        };
        let parameters = parameters
            .iter()
            .map(|parameter| self.signature_type(parameter, context))
            .collect::<Result<Vec<_>, _>>()?;
        let result = self.signature_type(result, context)?;
        let signature = CallableSignature::new(parameters, result);
        let registry = self.runtime.registry_mut();
        match name.as_str() {
            "Block" => registry.intern_block_alias(signature),
            "Closure" => registry.intern_callable_type(CallableKind::Closure, signature),
            "BoundMethod" => registry.intern_callable_type(CallableKind::BoundMethod, signature),
            _ => return Err(MachineError::UnsupportedConstruct),
        }
        .map_err(|_| MachineError::UnsupportedConstruct)
    }

    fn signature_type(
        &mut self,
        expression: &TypeExpression,
        (program, classes): (&Program, &[ClassId]),
    ) -> Result<SignatureType, MachineError> {
        match self.reify_type(expression, program, classes)? {
            Value::Type(class, arguments) => {
                Ok(SignatureType::Nominal(NominalType::new(class, arguments)))
            }
            Value::ComposedType(form) => Ok(SignatureType::Composed(form)),
            _ => Err(MachineError::UnsupportedConstruct),
        }
    }

    pub(super) fn callable_signature(
        &mut self,
        signature: &MethodDeclaration,
        context: (&Program, &[ClassId]),
    ) -> Result<CallableSignature, MachineError> {
        let object = TypeExpression::Name("Object".into());
        let parameters = signature
            .parameters
            .iter()
            .map(|parameter| {
                self.signature_type(parameter.annotation.as_ref().unwrap_or(&object), context)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = signature.return_type.as_ref().unwrap_or(&object);
        let result = if signature.is_async {
            TypeExpression::Generic {
                name: "Task".into(),
                arguments: vec![result.clone()],
            }
        } else {
            result.clone()
        };
        Ok(CallableSignature::new(
            parameters,
            self.signature_type(&result, context)?,
        ))
    }

    pub(super) fn remember_bound_signature(
        &mut self,
        bound: iris_runtime::BoundMethod,
        context: (&Program, &[ClassId]),
    ) {
        let method = bound.method();
        let Ok(owner) = self.resolve_method_body(method.body(), context.0) else {
            return;
        };
        let context = (owner.program.as_ref(), owner.classes.as_slice());
        let Some(signature) = self.method_signatures.get(&method.id()).cloned() else {
            return;
        };
        let receiver = match bound.receiver() {
            iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
            iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
            iris_runtime::BoundReceiver::Module(_) => Value::Nil,
        };
        let Ok(mut bindings) = self.receiver_type_bindings(&receiver, context) else {
            return;
        };
        bindings.extend(self.closed_method_bindings(method).unwrap_or_default());
        let previous = std::mem::replace(&mut self.method_types, bindings);
        let resolved = self.callable_signature(&signature, context);
        self.method_types = previous;
        if let Ok(signature) = resolved {
            self.bound_signatures.insert(bound.id(), signature);
        }
    }

    pub(super) fn callable_admits(
        &self,
        target: &iris_runtime::CallableType,
        value: &Value,
    ) -> bool {
        let actual = match value {
            Value::BoundMethod(bound) if target.kind() == CallableKind::BoundMethod => {
                self.bound_signatures.get(&bound.id())
            }
            Value::Closure(identity) if target.kind() == CallableKind::Closure => {
                if let Some(next) = self.next_continuations.get(identity) {
                    let Ok(changes) = self.builtin_class("ArgumentChanges") else {
                        return false;
                    };
                    let Ok(object) = self.builtin_class("Object") else {
                        return false;
                    };
                    let result = if next.invocation.selected().signature().is_async() {
                        let Ok(task) = self.builtin_class("Task") else {
                            return false;
                        };
                        NominalType::new(task, vec![NominalType::new(object, Vec::new())])
                    } else {
                        NominalType::new(object, Vec::new())
                    };
                    return target.signature()
                        == &CallableSignature::new(
                            vec![NominalType::new(changes, Vec::new()).into()],
                            result.into(),
                        );
                }
                self.closures
                    .get(identity)
                    .and_then(|closure| closure.callable_signature.as_ref())
            }
            _ => None,
        };
        actual == Some(target.signature())
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "tests require compiled bytecode")]
mod tests {
    use super::*;

    #[test]
    fn reification_preserves_publication_ids_when_callable_types_are_interned() {
        let program = crate::compile("nil").expect("source compiles");
        let mut given = Machine::new().expect("machine initializes");
        given
            .register_callable_types(&program)
            .expect("core registers");
        let mut control = Machine::new().expect("machine initializes");
        control
            .register_callable_types(&program)
            .expect("core registers");
        let expression = TypeExpression::Generic {
            name: "Block".into(),
            arguments: vec![TypeExpression::Function {
                parameters: Vec::new(),
                result: Box::new(TypeExpression::Name("Integer".into())),
            }],
        };
        let when = given
            .reify_type(&expression, &program, &[])
            .expect("type reifies");
        let Value::ComposedType(iris_runtime::ComposedType::Union(members)) = when else {
            unreachable!("transparent union")
        };
        assert_eq!(members.len(), 2);
        for member in members {
            let iris_runtime::TypeAtom::Nominal(identity, _) = member else {
                unreachable!("callable leaf")
            };
            assert!(given.runtime.registry().callable_type(identity).is_some());
            assert!(given.runtime.registry().class(identity).is_err());
        }
        let held = given
            .runtime
            .registry_mut()
            .define_class(iris_runtime::StaticSpine::new(9), None)
            .expect("class registers");
        let expected = control
            .runtime
            .registry_mut()
            .define_class(iris_runtime::StaticSpine::new(9), None)
            .expect("class registers");
        assert_eq!(held, expected);
        assert_eq!(
            given
                .runtime
                .registry()
                .active(held)
                .expect("revision")
                .id(),
            control
                .runtime
                .registry()
                .active(expected)
                .expect("revision")
                .id()
        );
        assert_eq!(
            given
                .runtime
                .registry()
                .active(held)
                .expect("revision")
                .commit_id(),
            control
                .runtime
                .registry()
                .active(expected)
                .expect("revision")
                .commit_id()
        );
    }

    #[test]
    fn task_identity_is_canonical_when_bootstrap_order_changes() {
        let program = crate::compile("nil").expect("source compiles");
        for core_first in [false, true] {
            let mut machine = Machine::new().expect("machine initializes");
            if core_first {
                machine.register_core_records().expect("core registers");
            }
            machine
                .register_callable_types(&program)
                .expect("callables register");
            let before = machine.builtin_class("Task").expect("task registers");

            machine.register_core_records().expect("core registers");
            machine
                .register_callable_types(&program)
                .expect("callables register again");

            assert_eq!(Some(before), machine.kernel.core_class("Task"));
            assert_eq!(machine.builtin_class("Kernel::Task"), Ok(before));
            assert!(!machine.core_support.contains_key("Task"));
        }
    }

    #[test]
    fn qualified_task_checks_preserve_invariance_when_task_resumes() {
        let source = r#"
module Scenario {
 public fun run() {
  let gate = Gate.new()
  let callback = { async || -> Integer; await gate; 7 }
  let task: Kernel::Task<Integer> = callback.call()
  let before = %[task is? Task<Integer>, task is? Kernel::Task<Integer>, task is? Kernel::Task<Object>, (task as Kernel::Task<Integer>) same? task]
  let rejected = try { let wrong: Kernel::Task<Object> = task; false } catch error { true }
  Gate.complete(gate, 1)
  let result = Host.run(task)
  %[before, rejected, result, task is? Kernel::Task<Integer>, task is? Task<Object>, (task as Task<Integer>) same? task]
 }
}
Scenario.run()
"#;
        let program = crate::compile(source).expect("source compiles");

        let result = crate::run(&program);

        assert_eq!(
            result,
            Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
                Value::Array(iris_runtime::ArrayRef::new(vec![
                    Value::Bool(true),
                    Value::Bool(true),
                    Value::Bool(false),
                    Value::Bool(true),
                ])),
                Value::Bool(true),
                Value::Integer(7_u64.into()),
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ])))
        );
    }
}
