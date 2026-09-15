use super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::{ClassId, ContractId, Method, MethodId, MethodOwner, Selector};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub(super) struct QualifiedMethods {
    pub(super) slots: HashMap<(ClassId, ContractId, Selector), Method>,
    pub(super) records: HashMap<MethodId, Method>,
    pub(super) definitions: HashMap<MethodId, ContractId>,
    pub(super) views: HashMap<ContractId, (ContractId, Vec<iris_runtime::NominalType>)>,
    identities: u64,
}

impl Machine {
    pub(super) fn contract_definition(&self, identity: ContractId) -> ContractId {
        self.qualified_methods
            .views
            .get(&identity)
            .map(|(definition, _)| *definition)
            .unwrap_or(identity)
    }

    pub(super) fn intern_contract_view(
        &mut self,
        definition: ContractId,
        arguments: Vec<iris_runtime::NominalType>,
    ) -> Result<ContractId, MachineError> {
        if arguments.is_empty() {
            return Ok(definition);
        }
        if let Some((identity, _)) = self
            .qualified_methods
            .views
            .iter()
            .find(|(_, known)| known.0 == definition && known.1 == arguments)
        {
            return Ok(*identity);
        }
        let identity = self.code.allocate_contract()?;
        self.qualified_methods
            .views
            .insert(identity, (definition, arguments));
        Ok(identity)
    }

    pub(super) fn with_selected_qualifier<T>(
        &mut self,
        qualifier: iris_runtime::Value,
        run: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let previous = self.selected_qualifier.replace(qualifier);
        let result = run(self);
        self.selected_qualifier = previous;
        result
    }

    pub(super) fn selected_contract_value(&self, contract: ContractId) -> iris_runtime::Value {
        match self.qualified_methods.views.get(&contract) {
            Some((definition, arguments)) => {
                iris_runtime::Value::Contract(*definition, arguments.clone())
            }
            None => iris_runtime::Value::Contract(contract, Vec::new()),
        }
    }
    pub(super) fn allocate_managed_method_identity(&mut self) -> Result<MethodId, MachineError> {
        let raw = u64::MAX
            .checked_sub(self.qualified_methods.identities)
            .ok_or(MachineError::Class(
                iris_runtime::ClassError::MethodIdentityExhausted,
            ))?;
        let identity = MethodId::new(raw);
        if self.runtime.registry().method_by_id(identity).is_some() {
            return Err(MachineError::Class(
                iris_runtime::ClassError::MethodIdentityExhausted,
            ));
        }
        self.qualified_methods.identities = self
            .qualified_methods
            .identities
            .checked_add(1)
            .ok_or(MachineError::Class(
                iris_runtime::ClassError::MethodIdentityExhausted,
            ))?;
        Ok(identity)
    }
    pub(super) fn qualified_method(&self, slot: (ClassId, ContractId, Selector)) -> Option<Method> {
        if self.static_impl_slots.contains(&(slot.0, slot.2))
            && self.qualified_methods.slots.contains_key(&slot)
        {
            return self
                .runtime
                .registry()
                .resolve_local_or_ancestor_method(slot.0, slot.2)
                .ok();
        }
        self.qualified_methods.slots.get(&slot).copied()
    }

    pub(super) fn publish_qualified_method(
        &mut self,
        slot: (ClassId, ContractId, Selector),
        function: usize,
        program: &Program,
    ) -> Result<Method, MachineError> {
        let signature = program.functions[function]
            .signature
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        let identity = self.allocate_managed_method_identity()?;
        let visibility = match signature.visibility {
            iris_syntax::Visibility::Public => iris_runtime::Visibility::Public,
            iris_syntax::Visibility::Protected => iris_runtime::Visibility::Protected,
            iris_syntax::Visibility::Private => iris_runtime::Visibility::Private,
        };
        let method = Method::new(
            identity,
            MethodOwner::Class(slot.0),
            slot.2,
            self.code.body(function, program)?,
            visibility,
        );
        self.remember_signature(method, program);
        self.qualified_methods.records.insert(identity, method);
        self.qualified_methods.definitions.insert(identity, slot.1);
        self.qualified_methods.slots.insert(slot, method);
        Ok(method)
    }

    pub(super) fn register_qualified_methods(
        &mut self,
        target: (ClassId, usize),
        program: &Program,
    ) -> Result<(), MachineError> {
        for (contract, name, function) in &program.classes[target.1].qualified_impls {
            let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
            self.publish_qualified_method(
                (target.0, program.contract_identity(*contract), selector),
                *function,
                program,
            )?;
        }
        Ok(())
    }

    pub(super) fn discard_qualified_methods(&mut self, class: ClassId) {
        self.qualified_methods
            .slots
            .retain(|(owner, _, _), _| *owner != class);
        self.qualified_methods.records.retain(|_, method| {
            if method.owner() == MethodOwner::Class(class) {
                self.method_signatures.remove(&method.id());
                self.wrapper_chains.remove(&method.id());
                false
            } else {
                true
            }
        });
        self.qualified_methods
            .definitions
            .retain(|identity, _| self.qualified_methods.records.contains_key(identity));
    }
}

#[cfg(test)]
mod tests;
