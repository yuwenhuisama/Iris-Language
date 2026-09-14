use std::collections::{HashMap, HashSet};

use crate::{
    CandidateModule, ClassRegistry, CompositionEdge, MetaCapabilities, ModuleCandidateError,
    ModuleId, ModuleRevision, ModuleRevisionId,
};

impl ClassRegistry {
    pub fn roll_back_group(&mut self) {
        for method in self.provisional_structural_methods() {
            self.methods.remove(&method);
        }
        self.staged.clear();
        self.modules.staged.clear();
    }

    pub fn active_module(&self, module: ModuleId) -> Result<&ModuleRevision, ModuleCandidateError> {
        self.modules
            .modules
            .get(&module)
            .ok_or(ModuleCandidateError::UnknownModule(module))
    }

    pub fn module_revision(
        &self,
        revision: ModuleRevisionId,
    ) -> Result<&ModuleRevision, ModuleCandidateError> {
        self.modules
            .revisions
            .get(&revision)
            .ok_or(ModuleCandidateError::UnknownRevision(revision))
    }

    pub fn staged_module(
        &self,
        module: ModuleId,
    ) -> Result<&CandidateModule, ModuleCandidateError> {
        self.modules
            .staged
            .get(&module)
            .ok_or(ModuleCandidateError::NotStaged(module))
    }

    /// Reserves only a Module identity. All edges are validated at mixed-group commit.
    pub fn stage_module_origin(
        &mut self,
        components: &[CompositionEdge],
        policy: MetaCapabilities,
    ) -> Result<ModuleId, ModuleCandidateError> {
        let module = ModuleId::new(self.modules.next_module_id);
        let next = self
            .modules
            .next_module_id
            .checked_add(1)
            .ok_or(ModuleCandidateError::IdentityExhausted)?;
        self.modules.staged.insert(
            module,
            CandidateModule {
                owner: module,
                base: None,
                number: 1,
                components: components.to_vec(),
                methods: HashMap::new(),
                meta_capabilities: policy,
                required: Vec::new(),
                additions: HashSet::new(),
                provisional_methods: HashSet::new(),
            },
        );
        self.modules.next_module_id = next;
        Ok(module)
    }

    pub fn begin_module_transaction(
        &mut self,
        module: ModuleId,
    ) -> Result<(), ModuleCandidateError> {
        if self.modules.staged.contains_key(&module) {
            return Ok(());
        }
        let active = self.active_module(module)?;
        let candidate = CandidateModule {
            owner: module,
            base: Some(active.id()),
            number: active
                .number()
                .checked_add(1)
                .ok_or(ModuleCandidateError::RevisionNumberExhausted(module))?,
            components: active.components.clone(),
            methods: active.methods.clone(),
            meta_capabilities: active.meta_capabilities(),
            required: Vec::new(),
            additions: HashSet::new(),
            provisional_methods: HashSet::new(),
        };
        self.modules.staged.insert(module, candidate);
        Ok(())
    }

    /// Existing Module edges are frozen until host-MRO revalidation is implemented.
    pub fn stage_module_composition(
        &mut self,
        module: ModuleId,
        components: &[CompositionEdge],
    ) -> Result<(), ModuleCandidateError> {
        let candidate = self
            .modules
            .staged
            .get_mut(&module)
            .ok_or(ModuleCandidateError::NotStaged(module))?;
        if candidate.base.is_some() {
            return Err(ModuleCandidateError::CompositionFrozen(module));
        }
        candidate.components = components.to_vec();
        Ok(())
    }

    pub fn stage_module_policy(
        &mut self,
        module: ModuleId,
        _policy: MetaCapabilities,
    ) -> Result<(), ModuleCandidateError> {
        self.staged_module(module)?;
        Err(ModuleCandidateError::PolicyFrozen(module))
    }

    pub fn roll_back_module_transaction(&mut self, module: ModuleId) {
        self.modules.staged.remove(&module);
    }
}

impl crate::module_registry::ModuleRegistry {
    pub(crate) fn candidate_policy(
        &self,
        module: ModuleId,
        staged: &HashMap<ModuleId, CandidateModule>,
    ) -> Result<MetaCapabilities, ModuleCandidateError> {
        crate::module_overlay::ModuleOverlay {
            active: &self.modules,
            staged,
        }
        .linearize(&[module])
        .map(|(_, policy)| policy)
        .map_err(ModuleCandidateError::from)
    }
}
