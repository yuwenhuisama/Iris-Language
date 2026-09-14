use core::fmt;
use std::{collections::HashMap, error::Error};

use crate::{
    CompositionEdge, MetaCapabilities, Method, MethodBody, MethodId, MethodOwner, ModuleId,
    Selector, Visibility,
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
    pub(crate) modules: HashMap<ModuleId, crate::ModuleRevision>,
    pub(crate) revisions: HashMap<crate::ModuleRevisionId, crate::ModuleRevision>,
    pub(crate) staged: HashMap<ModuleId, crate::CandidateModule>,
    pub(crate) next_module_id: u64,
    pub(crate) next_revision_id: u64,
}

impl ModuleRegistry {
    pub(crate) fn define(
        &mut self,
        components: &[CompositionEdge],
        meta_capabilities: MetaCapabilities,
    ) -> Result<ModuleId, ModuleError> {
        let id = ModuleId::new(self.next_module_id);
        let next_module_id = self
            .next_module_id
            .checked_add(1)
            .ok_or(ModuleError::ModuleIdentityExhausted)?;
        let next_revision_id = self
            .next_revision_id
            .checked_add(1)
            .ok_or(ModuleError::ModuleIdentityExhausted)?;
        let revision = crate::ModuleRevision {
            id: crate::ModuleRevisionId(self.next_revision_id),
            owner: id,
            number: 1,
            commit_id: 0,
            components: components.to_vec(),
            methods: HashMap::new(),
            meta_capabilities,
        };
        self.revisions.insert(revision.id, revision.clone());
        self.modules.insert(id, revision);
        self.next_revision_id = next_revision_id;
        self.next_module_id = next_module_id;
        Ok(id)
    }

    pub(crate) fn components(&self, module: ModuleId) -> Result<&[CompositionEdge], ModuleError> {
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
        let mut revision = self
            .modules
            .get(&module)
            .ok_or(ModuleError::UnknownModuleId(module))?
            .clone();
        let next_revision_id = self
            .next_revision_id
            .checked_add(1)
            .ok_or(ModuleError::ModuleIdentityExhausted)?;
        revision.number = revision
            .number
            .checked_add(1)
            .ok_or(ModuleError::ModuleIdentityExhausted)?;
        revision.id = crate::ModuleRevisionId(self.next_revision_id);
        revision.commit_id = 0;
        revision.methods.insert(selector, method);
        self.revisions.insert(revision.id, revision.clone());
        self.modules.insert(module, revision);
        self.next_revision_id = next_revision_id;
        Ok(method)
    }

    pub(crate) fn method(&self, module: ModuleId, selector: Selector) -> Option<Method> {
        self.modules
            .get(&module)
            .and_then(|definition| definition.methods.get(&selector))
            .copied()
    }
}
