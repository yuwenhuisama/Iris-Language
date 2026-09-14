use std::collections::{HashMap, HashSet};

use crate::{
    CandidateModule, ClassError, CompositionEdge, MetaCapabilities, ModuleCandidateError, ModuleId,
    ModuleRevision,
};

pub(crate) enum ModuleGraphError {
    Unknown(ModuleId),
    Cycle(ModuleId),
}

impl From<ModuleGraphError> for ClassError {
    fn from(error: ModuleGraphError) -> Self {
        match error {
            ModuleGraphError::Unknown(module) => Self::UnknownModuleId(module),
            ModuleGraphError::Cycle(module) => Self::ModuleCompositionCycle(module),
        }
    }
}

impl From<ModuleGraphError> for ModuleCandidateError {
    fn from(error: ModuleGraphError) -> Self {
        match error {
            ModuleGraphError::Unknown(module) => Self::UnknownModule(module),
            ModuleGraphError::Cycle(module) => Self::CompositionCycle(module),
        }
    }
}

pub(crate) struct ModuleOverlay<'registry> {
    pub(crate) active: &'registry HashMap<ModuleId, ModuleRevision>,
    pub(crate) staged: &'registry HashMap<ModuleId, CandidateModule>,
}

impl ModuleOverlay<'_> {
    pub(crate) fn resolve(
        &self,
        module: ModuleId,
    ) -> Result<(&[CompositionEdge], MetaCapabilities), ModuleGraphError> {
        match self.staged.get(&module) {
            Some(candidate) => Ok((&candidate.components, candidate.meta_capabilities)),
            None => self
                .active
                .get(&module)
                .map(|revision| (revision.composition_edges(), revision.meta_capabilities()))
                .ok_or(ModuleGraphError::Unknown(module)),
        }
    }

    pub(crate) fn linearize(
        &self,
        roots: &[ModuleId],
    ) -> Result<(Vec<ModuleId>, MetaCapabilities), ModuleGraphError> {
        let mut pending: Vec<_> = roots.iter().map(|module| (*module, false)).collect();
        let mut visiting = HashSet::new();
        let mut seen = HashSet::new();
        let mut order = Vec::new();
        let mut policy = MetaCapabilities::all();
        while let Some((current, exit)) = pending.pop() {
            if exit {
                visiting.remove(&current);
                seen.insert(current);
                continue;
            }
            if visiting.contains(&current) {
                return Err(ModuleGraphError::Cycle(current));
            }
            if seen.contains(&current) {
                continue;
            }
            let (edges, local) = self.resolve(current)?;
            visiting.insert(current);
            order.push(current);
            policy = policy.narrowed_by(local);
            pending.push((current, true));
            pending.extend(edges.iter().map(|edge| (edge.module(), false)));
        }
        Ok((order, policy))
    }
}
