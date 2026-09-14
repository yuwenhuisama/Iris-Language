use std::collections::HashSet;

use crate::{
    Capability, ClassRegistry, Method, MethodBody, MethodId, MethodOwner, ModuleCandidateError,
    ModuleId, ModuleMethodDefinition, Selector, Visibility,
};

impl ClassRegistry {
    pub fn require_module_candidate_capability(
        &self,
        module: ModuleId,
        operation: Capability,
    ) -> Result<(), ModuleCandidateError> {
        self.staged_module(module)?;
        let policy = self
            .modules
            .candidate_policy(module, &self.modules.staged)?;
        if !policy.allows(operation)
            || self
                .modules
                .modules
                .get(&module)
                .is_some_and(|active| !active.meta_capabilities().allows(operation))
        {
            return Err(ModuleCandidateError::CapabilityDenied { module, operation });
        }
        Ok(())
    }

    /// Static declaration installation only; checked transformations use the other staging APIs.
    pub fn stage_module_origin_method(
        &mut self,
        module: ModuleId,
        definition: ModuleMethodDefinition,
    ) -> Result<Method, ModuleCandidateError> {
        if self.staged_module(module)?.base.is_some() {
            return Err(ModuleCandidateError::OriginRequired(module));
        }
        if self
            .staged_module(module)?
            .method(definition.selector)
            .is_some()
        {
            return Err(ModuleCandidateError::MethodCollision {
                module,
                selector: definition.selector,
            });
        }
        self.install_module_candidate_method(module, definition, None)
    }

    /// Adds only private, non-colliding slots, with no override or host-access grant.
    /// Backends must validate callable signatures and Contract obligations before commit.
    pub fn stage_module_method(
        &mut self,
        module: ModuleId,
        definition: ModuleMethodDefinition,
    ) -> Result<Method, ModuleCandidateError> {
        self.require_module_candidate_capability(module, Capability::MethodSet)?;
        if definition.visibility != Visibility::Private {
            return Err(ModuleCandidateError::PublicAdditionDenied {
                module,
                selector: definition.selector,
            });
        }
        self.stage_module_declaration_method(module, definition)
    }

    /// Adds a source declaration with its declared visibility, subject to MethodSet
    /// authority and composition collisions. Decorator additions remain private-only.
    pub fn stage_module_declaration_method(
        &mut self,
        module: ModuleId,
        definition: ModuleMethodDefinition,
    ) -> Result<Method, ModuleCandidateError> {
        self.require_module_candidate_capability(module, Capability::MethodSet)?;
        let mut pending = vec![module];
        let mut seen = HashSet::new();
        while let Some(owner) = pending.pop() {
            if !seen.insert(owner) {
                continue;
            }
            let (method, edges) = match self.modules.staged.get(&owner) {
                Some(candidate) => (
                    candidate.method(definition.selector),
                    candidate.composition_edges(),
                ),
                None => {
                    let active = self.active_module(owner)?;
                    (
                        active.method(definition.selector),
                        active.composition_edges(),
                    )
                }
            };
            if method.is_some() {
                return Err(ModuleCandidateError::MethodCollision {
                    module,
                    selector: definition.selector,
                });
            }
            pending.extend(edges.iter().map(|edge| edge.module()));
        }
        self.install_module_candidate_method(module, definition, Some(Capability::MethodSet))
    }

    /// Replaces a local body with a fresh Method ID, retaining visibility and lexical owner.
    /// Signature/native compatibility and wrapper state admission remain backend responsibilities.
    pub fn stage_module_method_body(
        &mut self,
        module: ModuleId,
        selector: Selector,
        body: MethodBody,
    ) -> Result<Method, ModuleCandidateError> {
        self.require_module_candidate_capability(module, Capability::MethodBody)?;
        let original = self
            .staged_module(module)?
            .method(selector)
            .ok_or(ModuleCandidateError::MethodNotFound { module, selector })?;
        self.install_module_candidate_method(
            module,
            ModuleMethodDefinition::new(selector, body, original.visibility()),
            Some(Capability::MethodBody),
        )
    }

    fn install_module_candidate_method(
        &mut self,
        module: ModuleId,
        definition: ModuleMethodDefinition,
        requirement: Option<Capability>,
    ) -> Result<Method, ModuleCandidateError> {
        let candidate = self
            .modules
            .staged
            .get_mut(&module)
            .ok_or(ModuleCandidateError::NotStaged(module))?;
        let next = self
            .next_method_id
            .checked_add(1)
            .ok_or(ModuleCandidateError::MethodIdentityExhausted)?;
        let method = Method::new(
            MethodId::new(self.next_method_id),
            MethodOwner::Module(module),
            definition.selector,
            definition.body,
            definition.visibility,
        );
        candidate.methods.insert(definition.selector, method);
        candidate.provisional_methods.insert(method.id());
        if let Some(operation) = requirement {
            candidate.required.push(operation);
            if operation == Capability::MethodSet {
                candidate.additions.insert(definition.selector);
            }
        }
        self.methods.insert(method.id(), method);
        self.next_method_id = next;
        Ok(method)
    }
}
