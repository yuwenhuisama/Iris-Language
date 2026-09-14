use super::{ConstructionError, Runtime};
use crate::{ClassError, ClassId, Selector, Value};

impl Runtime {
    pub fn declare_candidate_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
        mutable: bool,
    ) -> Result<Value, ConstructionError> {
        if !self.registry.is_staging(class) {
            return Err(ClassError::UnknownClassId(class).into());
        }
        self.registry.declare_class_var(class, name, mutable)?;
        self.record_staged_storage_base(class);
        self.staged_class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    pub fn candidate_class_var(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<Option<Value>, ConstructionError> {
        if !self.registry.is_staging(class) {
            return Err(ClassError::UnknownClassId(class).into());
        }
        Ok(self
            .staged_class_vars
            .get(&(class, name))
            .or_else(|| self.class_vars.get(&(class, name)))
            .cloned())
    }

    pub fn assign_candidate_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        let candidate = self
            .registry
            .staged
            .get(&class)
            .ok_or(ClassError::UnknownClassId(class))?;
        if !candidate.class_vars.contains(&name) {
            return Err(ConstructionError::MissingDeclaredClassVariable { class, name });
        }
        if candidate.immutable_class_vars.contains(&name) {
            return Err(ConstructionError::ImmutableClassVariable { class, name });
        }
        self.record_staged_storage_base(class);
        self.staged_class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    /// Declares and initializes a Class variable, staging provisional origin payloads.
    pub fn declare_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
        mutable: bool,
    ) -> Result<Value, ConstructionError> {
        self.registry.declare_class_var(class, name, mutable)?;
        if self.registry.staged_origin(class).is_ok() {
            return self.initialize_staged_class_var(class, name, value);
        }
        self.class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    /// Initializes a locally declared Class-variable cell on an unpublished origin.
    ///
    /// Call `registry_mut().declare_class_var` first with the declared mutability.
    /// This is one-shot declaration initialization, not assignment authority.
    /// Published Classes (including open transactions), inherited or undeclared
    /// slots, and already initialized cells are rejected. Stored nil is a value.
    /// Use the Runtime group commit/rollback APIs to publish/discard the payload.
    pub fn initialize_staged_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        self.registry.staged_origin(class)?;
        if !self
            .registry
            .staged
            .get(&class)
            .is_some_and(|candidate| candidate.class_vars.contains(&name))
        {
            return Err(ConstructionError::MissingDeclaredClassVariable { class, name });
        }
        if self.staged_class_vars.contains_key(&(class, name))
            || self.class_vars.contains_key(&(class, name))
        {
            return Err(ClassError::DuplicateClassVariable { class, name }.into());
        }
        self.record_staged_storage_base(class);
        self.staged_class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    pub(super) fn validate_staged_storage(&self) -> Result<(), ClassError> {
        for (&class, base) in &self.staged_storage_bases {
            if !self.registry.is_staging(class)
                || self.registry.active_revision(class).ok() != *base
            {
                return Err(ClassError::UnknownClassId(class));
            }
        }
        if let Some(class) = self
            .staged_class_raw_ivars
            .keys()
            .copied()
            .filter(|class| !self.registry.is_staging(*class))
            .min()
        {
            return Err(ClassError::UnknownClassId(class));
        }
        for &(class, name) in self.staged_class_vars.keys() {
            if !self
                .registry
                .staged
                .get(&class)
                .is_some_and(|candidate| candidate.class_vars.contains(&name))
            {
                return Err(ClassError::StaleCandidate { class });
            }
        }
        Ok(())
    }

    pub(super) fn record_staged_storage_base(&mut self, class: ClassId) {
        let base = self.registry.active_revision(class).ok();
        self.staged_storage_bases.entry(class).or_insert(base);
    }

    pub(super) fn staged_storage_is_current(&self, class: ClassId) -> bool {
        self.registry.is_staging(class)
            && self.staged_storage_bases.get(&class)
                == Some(&self.registry.active_revision(class).ok())
    }
}
