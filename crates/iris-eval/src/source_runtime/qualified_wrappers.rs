use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{Capability, ClassId, ContractId, Method, MethodId, MethodOwner, Selector};
use iris_syntax::MethodDeclaration;

impl SourceEvaluator {
    pub(super) fn inherit_planning_contracts(&mut self, source: &Self) {
        self.contract_names = source.contract_names.clone();
        self.contract_requirements = source.contract_requirements.clone();
        self.contract_requirement_arities = source.contract_requirement_arities.clone();
        self.contract_requirement_parameters = source.contract_requirement_parameters.clone();
        self.contract_requirement_returns = source.contract_requirement_returns.clone();
        self.contract_capabilities = source.contract_capabilities.clone();
        self.contract_parents = source.contract_parents.clone();
        self.next_contract = source.next_contract;
        self.qualified_contracts.parameters = source.qualified_contracts.parameters.clone();
        self.qualified_contracts.parents = source.qualified_contracts.parents.clone();
        self.qualified_contracts.requirements = source.qualified_contracts.requirements.clone();
        for (name, contract) in &source.contract_names {
            self.names.insert(
                name.clone(),
                super::Binding::immutable(iris_runtime::Value::Contract(*contract, Vec::new())),
            );
        }
    }

    pub(super) fn qualified_declaration(
        &self,
        class: ClassId,
        contract: ContractId,
        selector: Selector,
    ) -> Option<&MethodDeclaration> {
        self.qualified_methods
            .get(&(class, contract, selector))
            .and_then(|method| self.bodies.get(&method.body().raw()))
    }

    pub(super) fn qualified_wrapper_target(
        &mut self,
        class: ClassId,
        declaration: &MethodDeclaration,
    ) -> Result<Option<(ContractId, Method, MethodDeclaration)>, EvaluationError> {
        let Some(Some(name)) = &declaration.impl_contract else {
            return Ok(None);
        };
        let contract = *self
            .contract_names
            .get(name)
            .ok_or(EvaluationError::NameError)?;
        let selector = self.selector(&declaration.selector);
        let original = *self
            .qualified_methods
            .get(&(class, contract, selector))
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let mut selected = self
            .bodies
            .get(&original.body().raw())
            .cloned()
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let key = (contract, declaration.selector.clone());
        let bindings = self.qualified_requirement_bindings(class, contract);
        if let Some(parameters) = self.contract_requirement_parameters.get(&key) {
            for (parameter, required) in selected.parameters.iter_mut().zip(parameters) {
                if parameter.annotation.is_none() {
                    parameter.annotation = required.as_ref().map(|annotation| {
                        super::wrapper_generics::substitute(annotation, &bindings)
                    });
                }
            }
        }
        if selected.return_type.is_none() {
            selected.return_type = self
                .contract_requirement_returns
                .get(&key)
                .map(|annotation| super::wrapper_generics::substitute(annotation, &bindings));
        }
        Ok(Some((contract, original, selected)))
    }

    pub(super) fn publish_qualified_wrapper(
        &mut self,
        contract: ContractId,
        original: Method,
    ) -> Result<Method, EvaluationError> {
        let MethodOwner::Class(class) = original.owner() else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        self.runtime
            .registry()
            .require_candidate_meta_capability(class, Capability::MethodBody)
            .map_err(EvaluationError::Class)?;
        let identity = MethodId::new(self.next_body);
        self.next_body = self.next_body.checked_add(1).ok_or(EvaluationError::Class(
            iris_runtime::ClassError::MethodIdentityExhausted,
        ))?;
        let published = Method::new(
            identity,
            original.owner(),
            original.selector(),
            original.body(),
            original.visibility(),
        );
        self.qualified_methods
            .insert((class, contract, original.selector()), published);
        Ok(published)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_runtime::Value;

    fn run(evaluator: &mut SourceEvaluator, source: &str) -> Result<Value, EvaluationError> {
        let parsed = iris_parser::parse(source);
        assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
        evaluator.set_source(source);
        evaluator.program(&parsed.program)
    }

    #[test]
    fn retained_closed_qualified_method_keeps_owner_and_cache_when_reopened() {
        let mut given = SourceEvaluator::new_in_package("qualified.tests").unwrap();
        run(&mut given, r#"
            class Wrap {}
            impl Wrap for MethodDecorator {
                public fun plan(d, a) -> Plan { Plan.empty }
                public fun transform(d, a, c) -> Transformation {
                    mut count = 0
                    Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                        count = count + 1
                        next.call() + count
                    })
                }
            }
            contract Named<T> { fun value(value: T) -> T }
            class Box<T> {
                @Wrap() public fun value(value: T) -> T { value }
            }
            impl Box<T> for Named<T> {}
            let box = Box<Integer>.new()
            (box as Named<Integer>)..value(7)
        "#).unwrap();
        let owner = given.class_name("Box").unwrap().unwrap();
        let selector = given.selector("value");
        let iris_runtime::DispatchOutcome::Invoke(retained) =
            given.runtime.registry().dispatch(owner, selector).unwrap()
        else {
            panic!("missing retained method")
        };
        let receiver = run(&mut given, "box").unwrap();
        run(
            &mut given,
            r#"
            open class Box<T> {
                @Wrap() public override fun value(value: T) -> T { value + 10 }
            }; 0
        "#,
        )
        .unwrap();
        let iris_runtime::DispatchOutcome::Invoke(published) =
            given.runtime.registry().dispatch(owner, selector).unwrap()
        else {
            panic!("missing published method")
        };

        let when =
            given.reflective_invoke(retained, receiver.clone(), &[Value::Integer(7u64.into())]);

        assert_ne!(retained.id(), published.id());
        assert_eq!(when, Ok(Value::Integer(9u64.into())));
        assert_eq!(
            given.reflective_invoke(published, receiver, &[Value::Integer(7u64.into())]),
            Ok(Value::Integer(18u64.into()))
        );
    }

    #[test]
    fn retained_qualified_identity_keeps_old_chain_and_rejects_absent_owner() {
        let mut given = SourceEvaluator::new_in_package("qualified.tests").unwrap();
        run(
            &mut given,
            r#"
            class Effects { public class property wrappers: Integer = 0 }
            class Wrap {}
            impl Wrap for MethodDecorator {
                public fun plan(declaration, arguments) -> Plan { Plan.empty }
                public fun transform(declaration, arguments, context) -> Transformation {
                    Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                        Effects.wrappers = Effects.wrappers + 1
                        next.call()
                    })
                }
            }
            contract Named { fun name() -> Symbol }
            class Target { @Wrap() public fun name() -> Symbol { :qualified } }
            impl Target for Named {}
            let target = Target.new()
            0
            "#,
        )
        .unwrap();
        run(
            &mut given,
            "class Child extends Target {}; class Other {}; 0",
        )
        .unwrap();
        let owner = given.class_name("Target").unwrap().unwrap();
        let child = given.class_name("Child").unwrap().unwrap();
        let selector = given.selector("name");
        let iris_runtime::DispatchOutcome::Invoke(retained) =
            given.runtime.registry().dispatch(owner, selector).unwrap()
        else {
            panic!("missing retained method")
        };
        let receiver = Value::Object(given.construct(child, &[]).unwrap());
        let original = given.wrapper_chains[&retained.id()].original;
        assert_ne!(original.id(), retained.id());
        run(&mut given, "open class Target { @Wrap() public override fun name() -> Symbol { :replacement } }; 0").unwrap();
        let iris_runtime::DispatchOutcome::Invoke(published) =
            given.runtime.registry().dispatch(owner, selector).unwrap()
        else {
            panic!("missing published method")
        };
        assert_ne!(retained.id(), published.id());

        let when = given.reflective_invoke(retained, receiver.clone(), &[]);
        assert_eq!(when, Ok(Value::Symbol("qualified".into())));
        assert_eq!(
            given.reflective_invoke(published, receiver.clone(), &[]),
            Ok(Value::Symbol("replacement".into()))
        );

        let other = given.class_name("Other").unwrap().unwrap();
        let receiver = Value::Object(given.construct(other, &[]).unwrap());
        let before = run(&mut given, "Effects.wrappers").unwrap();
        let when = given.reflective_invoke(retained, receiver, &[]);
        assert!(matches!(
            when,
            Err(EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::MethodBinding { .. }
                )
            ))
        ));
        assert_eq!(run(&mut given, "Effects.wrappers").unwrap(), before);
    }
}
