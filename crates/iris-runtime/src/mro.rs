use std::collections::HashSet;

use crate::module_registry::ModuleError;
use crate::{ClassError, ClassId, ModuleId, MroEntry};

impl crate::ClassRegistry {
    pub(crate) fn origin_mro(
        &self,
        class: ClassId,
        superclass: Option<ClassId>,
    ) -> Result<Vec<MroEntry>, ClassError> {
        let mut mro = vec![MroEntry::Class(class)];
        if let Some(superclass) = superclass {
            mro.extend_from_slice(self.active(superclass)?.mro());
        }
        Ok(mro)
    }

    pub(crate) fn compute_mro(
        &self,
        candidate: &crate::CandidateRevision,
    ) -> Result<Vec<MroEntry>, ClassError> {
        let mut mro = vec![MroEntry::Class(candidate.owner)];
        let mut seen = HashSet::new();
        let mut visiting = HashSet::new();
        for module in candidate.modules.iter().rev() {
            self.append_module(*module, &mut mro, &mut seen, &mut visiting)?;
        }
        if let Some(superclass) = candidate.runtime_superclass {
            mro.extend_from_slice(self.active(superclass)?.mro());
        }
        Ok(mro)
    }

    fn append_module(
        &self,
        module: ModuleId,
        mro: &mut Vec<MroEntry>,
        seen: &mut HashSet<ModuleId>,
        visiting: &mut HashSet<ModuleId>,
    ) -> Result<(), ClassError> {
        if seen.contains(&module) {
            return Ok(());
        }
        if !visiting.insert(module) {
            return Err(ClassError::ModuleCompositionCycle(module));
        }
        mro.push(MroEntry::Module(module));
        seen.insert(module);
        if !self.modules.contains(module) {
            visiting.remove(&module);
            return Ok(());
        }
        let components = self
            .modules
            .components(module)
            .map_err(Self::module_error)?;
        for component in components.iter().rev() {
            self.append_module(component.module(), mro, seen, visiting)?;
        }
        visiting.remove(&module);
        Ok(())
    }

    pub(crate) fn module_error(error: ModuleError) -> ClassError {
        match error {
            ModuleError::UnknownModuleId(module) => ClassError::UnknownModuleId(module),
            ModuleError::ModuleIdentityExhausted => ClassError::ClassIdentityExhausted,
        }
    }
}
