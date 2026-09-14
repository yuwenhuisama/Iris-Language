use crate::{
    Capability, ClassError, ClassId, ClassRegistry, DecoratorTransform, Method, MethodBody,
    MroEntry, PolicyOrigin, Selector, Visibility,
};

impl ClassRegistry {
    /// Checks backend mutation authority against the current transaction candidate.
    ///
    /// Unpublished origins use their own immutable policy, published superclass
    /// policy and composed Module policies. Published targets additionally require
    /// their active policy to allow the operation: staging never grants authority.
    /// Without a staged candidate this is the ordinary active-only check.
    ///
    /// Success is not a grant token, does not publish the target, and must not be
    /// exposed as ordinary reflection on an unpublished Class. Checked publication
    /// rechecks authority at mutation time. No revision or commit ID is consumed.
    pub fn require_candidate_meta_capability(
        &self,
        target: ClassId,
        operation: Capability,
    ) -> Result<(), ClassError> {
        let Some(candidate) = self.staged.get(&target) else {
            return self.require_meta_capability(target, operation);
        };
        if self.classes.contains_key(&target) {
            self.require_meta_capability(target, operation)?;
        }
        let overlay = self.candidate_module_overlay();
        let (mro, policy) = self.overlay_structure(candidate, &overlay)?;
        if policy.allows(operation) {
            return Ok(());
        }
        let policy_origin = mro.iter().find_map(|entry| match entry {
            MroEntry::Class(class) if *class == target => {
                (!candidate.static_spine.meta_capabilities().allows(operation))
                    .then_some(PolicyOrigin::Class(target))
            }
            MroEntry::Class(class) => self
                .active(*class)
                .ok()
                .filter(|revision| {
                    !revision
                        .static_spine()
                        .meta_capabilities()
                        .allows(operation)
                })
                .map(|_| PolicyOrigin::Class(*class)),
            MroEntry::Module(module) => overlay
                .resolve(*module)
                .ok()
                .map(|(_, policy)| policy)
                .filter(|policy| !policy.allows(operation))
                .map(|_| PolicyOrigin::Module(*module)),
        });
        Err(ClassError::MetaCapabilityDenied {
            target,
            operation,
            policy_origin: policy_origin.unwrap_or(PolicyOrigin::Class(target)),
            reason: "the candidate effective policy denies this meta operation",
        })
    }

    /// Installs a decorator-produced Method through checked candidate mutation.
    ///
    /// A local candidate slot requires `MethodBody`; a new local slot requires
    /// `MethodSet`. Backends validate signature compatibility and decorator payloads
    /// before calling, and retain wrapper chains by the returned real Method ID.
    /// This is not the static-origin policy bypass or a grant-token API.
    ///
    /// Staged origins and opens accumulate changes without consuming revision or
    /// commit IDs. Ordinary lookup stays active-only; origins become visible only
    /// after `commit_declaration_group`. A published target without a transaction
    /// retains standalone publication semantics. On error the backend must abort
    /// its group and roll back its own body/wrapper state, as for `publish_method`.
    pub fn publish_candidate_decorated_method(
        &mut self,
        class: ClassId,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
        decorators: impl IntoIterator<Item = DecoratorTransform>,
    ) -> Result<Method, ClassError> {
        self.publish_decorated_method_as(class, selector, body, visibility, decorators, false)
    }

    pub(crate) fn require_candidate_method_mutation(
        &mut self,
        class: ClassId,
        selector: Selector,
    ) -> Result<(), ClassError> {
        let present = match self.staged.get(&class) {
            Some(candidate) => candidate.methods.contains_key(&selector),
            None => self.active(class)?.methods().contains_key(&selector),
        };
        let capability = if present {
            Capability::MethodBody
        } else {
            Capability::MethodSet
        };
        self.require_candidate_meta_capability(class, capability)?;
        if let Some(candidate) = self.staged.get_mut(&class) {
            candidate.required.push(capability);
        }
        Ok(())
    }

    /// Adds or removes one Module edge in the current transaction candidate.
    pub fn recompose_candidate(
        &mut self,
        class: ClassId,
        module: crate::ModuleId,
        include: bool,
    ) -> Result<(), ClassError> {
        self.require_candidate_meta_capability(class, Capability::Modules)?;
        self.mutate_candidate(class, |candidate| {
            candidate.required.push(Capability::Modules);
            if include {
                if !candidate.modules.contains(&module) {
                    candidate.add_module(module);
                }
            } else {
                candidate.remove_module(module);
            }
        })
    }
}
