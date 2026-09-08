use super::{ClassId, EvaluationError, SourceEvaluator, Value};

impl SourceEvaluator {
    pub(super) fn check_method_replacement(
        &self,
        class: ClassId,
        replacement: &iris_syntax::MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        let registry = self.runtime.registry();
        let active = registry.active(class).map_err(EvaluationError::Class)?;
        let Some(selector) = self.selectors.get(&replacement.selector) else {
            return Ok(());
        };
        let table = match replacement.kind {
            iris_syntax::MethodKind::Class => active.singleton_methods(),
            iris_syntax::MethodKind::Property
                if self.is_builtin_class(class)
                    && self.builtin_class_property(&replacement.selector) =>
            {
                active.singleton_methods()
            }
            iris_syntax::MethodKind::Instance
            | iris_syntax::MethodKind::Property
            | iris_syntax::MethodKind::Module => active.methods(),
        };
        if let Some(promise) = table
            .get(selector)
            .and_then(|identity| registry.method_by_id(*identity))
            .and_then(|method| self.bodies.get(&method.body().raw()))
            && !iris_syntax::method_signature_compatible(replacement, promise, |source, target| {
                self.nominal_subtype(source, target)
            })
        {
            return Err(EvaluationError::TypeContractError);
        }
        Ok(())
    }
    pub(super) fn assign_local(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<Value, EvaluationError> {
        let binding = self.names.get(name).ok_or(EvaluationError::NameError)?;
        if !binding.mutable {
            return Err(EvaluationError::ImmutableBinding);
        }
        let contract = binding.contract.clone();
        if let Some(contract) = contract {
            self.check_binding_annotation(&value, &contract)?;
        }
        self.names
            .get_mut(name)
            .ok_or(EvaluationError::NameError)?
            .assign(value)
            .map_err(|()| EvaluationError::ImmutableBinding)
    }

    pub(super) fn validate_signature_promises(
        &self,
        class: ClassId,
    ) -> Result<(), EvaluationError> {
        let registry = self.runtime.registry();
        let active = registry.active(class).map_err(EvaluationError::Class)?;
        for (selector, identity) in active.methods() {
            let Some(promise) = registry
                .method_by_id(*identity)
                .and_then(|method| self.bodies.get(&method.body().raw()))
            else {
                continue;
            };
            let Some(replacement) = self.candidate_method_declaration(class, *selector) else {
                continue;
            };
            if !iris_syntax::method_signature_compatible(replacement, promise, |source, target| {
                self.nominal_subtype(source, target)
            }) {
                return Err(EvaluationError::TypeContractError);
            }
        }
        for (selector, identity) in active.singleton_methods() {
            let Some(promise) = registry
                .method_by_id(*identity)
                .and_then(|method| self.bodies.get(&method.body().raw()))
            else {
                continue;
            };
            let Some(replacement) = self.singleton_declarations.get(&(class, *selector)) else {
                continue;
            };
            if !iris_syntax::method_signature_compatible(replacement, promise, |source, target| {
                self.nominal_subtype(source, target)
            }) {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }

    fn nominal_subtype(&self, source: &str, target: &str) -> bool {
        if let Some(target) = self.contract_names.get(target) {
            if let Some(source) = self.contract_names.get(source) {
                return self.contract_subtype(*source, *target);
            }
            return self.class_name(source).is_ok_and(|class| {
                class.is_some_and(|class| {
                    self.runtime.registry().active(class).is_ok_and(|revision| {
                        revision.mro().iter().any(|entry| match entry {
                            iris_runtime::MroEntry::Class(owner) => self
                                .class_contracts
                                .get(owner)
                                .into_iter()
                                .flatten()
                                .any(|source| self.contract_subtype(*source, *target)),
                            iris_runtime::MroEntry::Module(_) => false,
                        })
                    })
                })
            });
        }
        match (self.class_name(source), self.class_name(target)) {
            (Ok(Some(source)), Ok(Some(target))) => self
                .runtime
                .registry()
                .active(source)
                .is_ok_and(|revision| {
                    revision.mro().iter().any(|entry| {
                        matches!(entry, iris_runtime::MroEntry::Class(class) if *class == target)
                    })
                }),
            _ => false,
        }
    }

    fn contract_subtype(
        &self,
        source: iris_runtime::ContractId,
        target: iris_runtime::ContractId,
    ) -> bool {
        source == target
            || self
                .contract_parents
                .get(&source)
                .into_iter()
                .flatten()
                .any(|parent| self.contract_subtype(*parent, target))
    }

    pub(super) fn value_conforms_to_contract(
        &mut self,
        value: &Value,
        target: iris_runtime::ContractId,
    ) -> Result<bool, EvaluationError> {
        let value = match value {
            Value::ContractView(receiver, _) => receiver.as_ref(),
            value => value,
        };
        let class = match self.class_of_value(value) {
            Ok(class) => class,
            Err(EvaluationError::UnsupportedConstruct) => return Ok(false),
            Err(error) => return Err(error),
        };
        let revision = self
            .runtime
            .registry()
            .active(class)
            .map_err(EvaluationError::Class)?;
        Ok(revision.mro().iter().any(|entry| match entry {
            iris_runtime::MroEntry::Class(owner) => self
                .class_contracts
                .get(owner)
                .into_iter()
                .flatten()
                .any(|source| self.contract_subtype(*source, target)),
            iris_runtime::MroEntry::Module(_) => false,
        }))
    }

    pub(super) fn meta_alias_method(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let outcome = self.alias_method_boundary(class, arguments);
        self.record_meta_outcome(outcome)
    }

    fn alias_method_boundary(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [Value::Symbol(alias), Value::Symbol(original)] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let alias_slot = self.selector(alias);
        let original = self.selector(original);
        if !self.runtime.registry().is_staging(class)
            && let Some(mut declaration) =
                self.candidate_method_declaration(class, original).cloned()
        {
            declaration.selector = alias.clone();
            self.check_method_replacement(class, &declaration)?;
        }
        self.runtime
            .registry_mut()
            .alias_method(class, alias_slot, original)
            .map(|()| Value::Nil)
            .map_err(EvaluationError::Class)
    }

    pub(super) fn record_meta_outcome(
        &mut self,
        outcome: Result<Value, EvaluationError>,
    ) -> Result<Value, EvaluationError> {
        if outcome.is_err() && self.open_target.is_some() {
            self.open_group_state = Some(super::OpenGroupState::Aborted);
        }
        outcome
    }
}
