use core::fmt;
use std::{collections::HashMap, error::Error};

use crate::{
    MetaCapabilities, Method, MethodBody, MethodId, MethodOwner, ModuleId, Selector, Visibility,
};

/// Recoverable failure from Module registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleError {
    /// The requested Module identity is absent from this runtime.
    UnknownModuleId(ModuleId),
    /// The runtime cannot issue another Module identity.
    ModuleIdentityExhausted,
}

impl fmt::Display for ModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownModuleId(_) => formatter.write_str("unknown Iris Module identity"),
            Self::ModuleIdentityExhausted => {
                formatter.write_str("Iris Module identity space exhausted")
            }
        }
    }
}

impl Error for ModuleError {}

/// Immutable Module composition and Method tables owned by the runtime.
#[derive(Debug, Default)]
pub(crate) struct ModuleRegistry {
    modules: HashMap<ModuleId, Module>,
    next_module_id: u64,
}

#[derive(Debug)]
struct Module {
    components: Vec<ModuleId>,
    methods: HashMap<Selector, Method>,
    meta_capabilities: MetaCapabilities,
}

impl ModuleRegistry {
    pub(crate) fn define(
        &mut self,
        components: &[ModuleId],
        meta_capabilities: MetaCapabilities,
    ) -> Result<ModuleId, ModuleError> {
        let id = ModuleId::new(self.next_module_id);
        let next_module_id = self
            .next_module_id
            .checked_add(1)
            .ok_or(ModuleError::ModuleIdentityExhausted)?;
        self.modules.insert(
            id,
            Module {
                components: components.to_vec(),
                methods: HashMap::new(),
                meta_capabilities,
            },
        );
        self.next_module_id = next_module_id;
        Ok(id)
    }

    pub(crate) fn components(&self, module: ModuleId) -> Result<&[ModuleId], ModuleError> {
        Ok(&self
            .modules
            .get(&module)
            .ok_or(ModuleError::UnknownModuleId(module))?
            .components)
    }

    pub(crate) fn contains(&self, module: ModuleId) -> bool {
        self.modules.contains_key(&module)
    }

    pub(crate) fn meta_capabilities(&self, module: ModuleId) -> Option<MetaCapabilities> {
        self.modules
            .get(&module)
            .map(|definition| definition.meta_capabilities)
    }

    pub(crate) fn define_method(
        &mut self,
        module: ModuleId,
        method: MethodId,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
    ) -> Result<Method, ModuleError> {
        let method = Method::new(
            method,
            MethodOwner::Module(module),
            selector,
            body,
            visibility,
        );
        self.modules
            .get_mut(&module)
            .ok_or(ModuleError::UnknownModuleId(module))?
            .methods
            .insert(selector, method);
        Ok(method)
    }

    pub(crate) fn method(&self, module: ModuleId, selector: Selector) -> Option<Method> {
        self.modules
            .get(&module)
            .and_then(|definition| definition.methods.get(&selector))
            .copied()
    }
}
