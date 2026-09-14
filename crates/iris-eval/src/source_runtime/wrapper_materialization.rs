use super::WrapperChain;
use crate::source_runtime::decorator_phases::PhaseTarget;
use crate::source_runtime::wrapper_execution::LexicalContext;
use crate::source_runtime::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{DecoratorKind, DecoratorReason, DecoratorValue, Operation};
use iris_runtime::{Capability, Method, MethodId, MethodOwner, Value};
use iris_syntax::TypeExpression;
use std::cell::RefCell;
use std::rc::Rc;

pub(in crate::source_runtime) struct CanonicalMethod {
    method: Method,
    pub(in crate::source_runtime) context: LexicalContext,
    closed: RefCell<Vec<ClosedMethod>>,
}

struct ClosedMethod {
    canonical: MethodId,
    owner_types: Vec<Value>,
    method_types: Vec<Value>,
    chain: Rc<WrapperChain>,
}

impl CanonicalMethod {
    pub(super) fn new(method: Method, context: LexicalContext) -> Self {
        Self {
            method,
            context,
            closed: RefCell::new(Vec::new()),
        }
    }
}

impl crate::source_runtime::gc_roots::TraceRoots for CanonicalMethod {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        self.context.trace_roots(roots);
        for closed in self.closed.borrow().iter() {
            roots.extend(
                closed
                    .owner_types
                    .iter()
                    .chain(&closed.method_types)
                    .cloned(),
            );
            closed.chain.trace_roots(roots);
        }
    }
}

impl SourceEvaluator {
    pub(in crate::source_runtime) fn materialize_wrapper_chain(
        &mut self,
        chain: Rc<WrapperChain>,
        owner_types: Vec<Value>,
    ) -> Result<Rc<WrapperChain>, EvaluationError> {
        let Some(canonical) = &chain.canonical else {
            return Ok(chain);
        };
        let method_types = chain
            .declaration
            .type_parameters
            .iter()
            .map(|name| self.wrapper_type_value(&TypeExpression::Name(name.clone())))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(closed) = canonical.closed.borrow().iter().find(|closed| {
            closed.canonical == canonical.method.id()
                && closed.owner_types == owner_types
                && closed.method_types == method_types
        }) {
            return Ok(Rc::clone(&closed.chain));
        }
        let previous = self.wrapper_context();
        self.restore_wrapper_context(canonical.context.clone());
        self.method_type_bindings.clear();
        let result = self.build_closed_wrapper_chain(&chain);
        self.restore_wrapper_context(previous);
        let closed = result?;
        canonical.closed.borrow_mut().push(ClosedMethod {
            canonical: canonical.method.id(),
            owner_types,
            method_types,
            chain: Rc::clone(&closed),
        });
        Ok(closed)
    }

    fn build_closed_wrapper_chain(
        &mut self,
        chain: &WrapperChain,
    ) -> Result<Rc<WrapperChain>, EvaluationError> {
        let MethodOwner::Class(class) = chain.original.owner() else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        self.runtime
            .registry()
            .require_candidate_meta_capability(class, Capability::MethodBody)
            .map_err(EvaluationError::Class)?;
        let wrappers = self.rooted(RefCell::new(Vec::new()));
        for application in &chain.declaration.decorators {
            let metadata = self.method_decorator_metadata(class, &chain.declaration)?;
            let result = self.execute_decorator_phase(
                application,
                PhaseTarget {
                    kind: DecoratorKind::Method,
                    reason: DecoratorReason::ClosedMaterialization,
                    metadata,
                    candidate: None,
                },
            )?;
            let Value::Decorator(record) = result else {
                return Err(self.wrapper_type_error());
            };
            let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                return Err(self.wrapper_type_error());
            };
            for operation in transformation.operations() {
                match operation {
                    Operation::WrapMethod(wrapper) => {
                        let identity = self.admit_wrapper(wrapper, chain.declaration.is_async)?;
                        wrappers.borrow_mut().push(Value::Closure(identity));
                    }
                    Operation::AddMethod { .. }
                    | Operation::WrapGetter(_)
                    | Operation::WrapSetter(_) => {
                        return Err(crate::source_runtime::decorator_phases::kind_error(false));
                    }
                }
            }
        }
        let identity = MethodId::new(self.next_body);
        self.next_body = self.next_body.checked_add(1).ok_or(EvaluationError::Class(
            iris_runtime::ClassError::MethodIdentityExhausted,
        ))?;
        let original = Method::new(
            identity,
            chain.original.owner(),
            chain.original.selector(),
            chain.original.body(),
            chain.original.visibility(),
        );
        let wrappers = wrappers
            .borrow()
            .iter()
            .map(|wrapper| self.admit_wrapper(wrapper, chain.declaration.is_async))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Rc::new(WrapperChain {
            original,
            declaration: chain.declaration.clone(),
            wrappers,
            qualifier: chain.qualifier,
            canonical: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_methods_have_distinct_identities_when_exact_types_differ()
    -> Result<(), EvaluationError> {
        let mut given = crate::Session::new()?;
        given.evaluate(r#"
            class Wrap {}
            impl Wrap for MethodDecorator {
                public fun plan(d, a) -> Plan { Plan.empty }
                public fun transform(d, a, c) -> Transformation {
                    Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
                }
            }
            class Target { @Wrap() public class fun echo<Element>(value: Element) -> Element { value } }
            %[Target.echo<Integer>(1), Target.echo<String>("s"), Target.echo<Integer>(2)]
        "#)?;
        let owner = given
            .evaluator
            .class_name("Target")?
            .ok_or(EvaluationError::NameError)?;
        let selector = given.evaluator.selector("echo");
        let iris_runtime::DispatchOutcome::Invoke(method) = given
            .evaluator
            .runtime
            .registry()
            .dispatch_class_object(owner, selector)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)?
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };

        let when = given.evaluator.wrapper_chains[&method.id()]
            .canonical
            .as_ref()
            .ok_or(EvaluationError::UnsupportedConstruct)?
            .closed
            .borrow();

        assert_eq!(when.len(), 2);
        assert_ne!(when[0].chain.original.id(), when[1].chain.original.id());
        for closed in when.iter() {
            assert_eq!(closed.canonical, method.id());
            assert_ne!(closed.chain.original.id(), method.id());
            assert_ne!(
                closed.chain.original.id(),
                given.evaluator.wrapper_chains[&method.id()].original.id()
            );
            assert_eq!(closed.owner_types, Vec::<Value>::new());
        }
        assert_ne!(when[0].method_types, when[1].method_types);
        assert_ne!(when[0].chain.wrappers, when[1].chain.wrappers);
        Ok(())
    }
}
