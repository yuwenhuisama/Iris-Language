use std::collections::HashSet;

use crate::{
    ClassError, ClassRegistry, ClassRevision, ModuleCandidateError, ModuleRevision,
    ModuleRevisionId, RuntimeStructuralError,
};

#[derive(Debug, Eq, PartialEq)]
pub struct StructuralCommit {
    pub commit_id: Option<u64>,
    pub classes: Vec<ClassRevision>,
    pub modules: Vec<ModuleRevision>,
}

impl ClassRegistry {
    /// Consumes both staged sets. Validation failure discards both without consuming
    /// revision/commit IDs. Only candidate-owned Method artifacts are discarded;
    /// backend bodies and external effects remain backend responsibilities.
    pub fn commit_structural_group(&mut self) -> Result<StructuralCommit, RuntimeStructuralError> {
        let artifacts = self.provisional_structural_methods();
        let result = self.publish_structural_group();
        if result.is_err() {
            for method in artifacts {
                self.methods.remove(&method);
            }
        }
        result
    }

    pub(crate) fn provisional_structural_methods(&self) -> Vec<crate::MethodId> {
        let retained: HashSet<_> = self
            .revisions
            .values()
            .flat_map(|revision| {
                revision
                    .methods()
                    .values()
                    .chain(revision.singleton_methods().values())
                    .copied()
            })
            .chain(
                self.modules
                    .revisions
                    .values()
                    .flat_map(|revision| revision.methods.values().map(|method| method.id())),
            )
            .collect();
        self.staged
            .values()
            .flat_map(|candidate| candidate.provisional_methods.iter())
            .chain(
                self.modules
                    .staged
                    .values()
                    .flat_map(|candidate| candidate.provisional_methods.iter()),
            )
            .filter(|method| !retained.contains(method))
            .copied()
            .collect()
    }

    fn publish_structural_group(&mut self) -> Result<StructuralCommit, RuntimeStructuralError> {
        let classes = std::mem::take(&mut self.staged);
        let modules = std::mem::take(&mut self.modules.staged);
        if classes.is_empty() && modules.is_empty() {
            return Ok(StructuralCommit {
                commit_id: None,
                classes: Vec::new(),
                modules: Vec::new(),
            });
        }
        let mut policies = std::collections::HashMap::new();
        for (module, candidate) in &modules {
            if let Some(base) = candidate.base {
                let active = self.active_module(*module)?;
                if active.id() != base {
                    return Err(ModuleCandidateError::Conflict(*module).into());
                }
                if active.composition_edges() != candidate.components {
                    return Err(ModuleCandidateError::CompositionFrozen(*module).into());
                }
                if active.meta_capabilities() != candidate.meta_capabilities {
                    return Err(ModuleCandidateError::PolicyFrozen(*module).into());
                }
            }
            let policy = self.modules.candidate_policy(*module, &modules)?;
            for operation in &candidate.required {
                if !policy.allows(*operation) {
                    return Err(ModuleCandidateError::CapabilityDenied {
                        module: *module,
                        operation: *operation,
                    }
                    .into());
                }
            }
            for selector in &candidate.additions {
                let mut pending: Vec<_> = candidate
                    .components
                    .iter()
                    .map(|edge| edge.module())
                    .collect();
                let mut seen = HashSet::new();
                while let Some(component) = pending.pop() {
                    if !seen.insert(component) {
                        continue;
                    }
                    let (method, edges) = match modules.get(&component) {
                        Some(candidate) => {
                            (candidate.method(*selector), candidate.composition_edges())
                        }
                        None => {
                            let active = self.active_module(component)?;
                            (active.method(*selector), active.composition_edges())
                        }
                    };
                    if method.is_some() {
                        return Err(ModuleCandidateError::MethodCollision {
                            module: *module,
                            selector: *selector,
                        }
                        .into());
                    }
                    pending.extend(edges.iter().map(|edge| edge.module()));
                }
            }
            let published_policy = match candidate.base {
                Some(_) => candidate.meta_capabilities,
                None => policy,
            };
            policies.insert(*module, published_policy);
        }
        let mut origins = HashSet::new();
        for (class, candidate) in &classes {
            match self.classes.get(class) {
                Some(active) if active.active_revision() != candidate.base_revision() => {
                    return Err(ClassError::MetaTransactionConflict { class: *class }.into());
                }
                Some(_) => {}
                None => {
                    origins.insert(*class);
                }
            }
        }
        let mut classes: Vec<_> = classes.into_values().collect();
        classes.sort_by_key(|candidate| candidate.owner);
        let overlay = crate::module_overlay::ModuleOverlay {
            active: &self.modules.modules,
            staged: &modules,
        };
        let classes = self.prepare_candidates(classes, &origins, Some(&overlay))?;
        self.check_revision_capacity(classes.len())?;
        let count = u64::try_from(modules.len())
            .map_err(|_| ModuleCandidateError::RevisionIdentityExhausted)?;
        let final_module_revision = self
            .modules
            .next_revision_id
            .checked_add(count)
            .ok_or(ModuleCandidateError::RevisionIdentityExhausted)?;
        let commit_id = self
            .next_commit_id
            .checked_add(1)
            .ok_or(ClassError::CommitIdentityExhausted)?;
        let mut modules: Vec<_> = modules.into_values().collect();
        modules.sort_by_key(|candidate| candidate.owner);
        let mut revision_id = self.modules.next_revision_id;
        let modules: Vec<_> = modules
            .into_iter()
            .map(|candidate| {
                let revision = ModuleRevision {
                    id: ModuleRevisionId(revision_id),
                    owner: candidate.owner,
                    number: candidate.number,
                    commit_id,
                    components: candidate.components,
                    methods: candidate.methods,
                    meta_capabilities: policies[&candidate.owner],
                };
                revision_id += 1;
                revision
            })
            .collect();
        let classes = self.install_candidates(classes, commit_id);
        for revision in &modules {
            self.modules
                .revisions
                .insert(revision.id(), revision.clone());
            self.modules
                .modules
                .insert(revision.owner(), revision.clone());
        }
        self.modules.next_revision_id = final_module_revision;
        Ok(StructuralCommit {
            commit_id: Some(commit_id),
            classes,
            modules,
        })
    }
}
