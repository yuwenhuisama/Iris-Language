use super::{ConstructionError, Runtime};
use crate::{ClassError, ClassId, ClassRevision, Selector, Value};

impl Runtime {
    /// Publishes mixed Class/Module metadata with staged Class storage in one group.
    pub fn commit_structural_group(
        &mut self,
    ) -> Result<crate::StructuralCommit, crate::RuntimeStructuralError> {
        if let Err(error) = self.validate_staged_storage() {
            self.roll_back_group();
            return Err(error.into());
        }
        let storage = std::mem::take(&mut self.staged_class_raw_ivars);
        let class_vars = std::mem::take(&mut self.staged_class_vars);
        self.staged_storage_bases.clear();
        let receipt = self.registry.commit_structural_group()?;
        for (class, slots) in storage {
            self.class_raw_ivars.entry(class).or_default().extend(slots);
        }
        self.class_vars.extend(class_vars);
        Ok(receipt)
    }

    /// Stores a backend-only raw ivar on an unpublished Class origin.
    ///
    /// Published Classes, including open transactions, and unknown identities
    /// are rejected. Use the Runtime group APIs to commit or discard these slots.
    pub fn assign_staged_class_raw_ivar(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        self.registry.staged_origin(class)?;
        self.assign_candidate_class_raw_ivar(class, name, value)
    }

    pub fn assign_candidate_class_raw_ivar(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        if !self.registry.is_staging(class) {
            return Err(ClassError::UnknownClassId(class).into());
        }
        self.record_staged_storage_base(class);
        self.staged_class_raw_ivars
            .entry(class)
            .or_default()
            .insert(name, value.clone());
        Ok(value)
    }

    /// Reads a backend-only origin slot, distinguishing absence from stored nil.
    ///
    /// Published Classes and discarded origins are rejected, even if a caller
    /// bypassed the Runtime group APIs through `registry_mut`.
    pub fn staged_class_raw_ivar(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<Option<Value>, ConstructionError> {
        self.registry.staged_origin(class)?;
        Ok(self
            .staged_class_raw_ivars
            .get(&class)
            .and_then(|slots| slots.get(&name))
            .cloned())
    }

    pub fn candidate_class_raw_ivar(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<Value, ConstructionError> {
        if !self.registry.is_staging(class) {
            return Err(ClassError::UnknownClassId(class).into());
        }
        Ok(self
            .staged_class_raw_ivars
            .get(&class)
            .and_then(|slots| slots.get(&name))
            .or_else(|| {
                self.class_raw_ivars
                    .get(&class)
                    .and_then(|slots| slots.get(&name))
            })
            .cloned()
            .unwrap_or(Value::Nil))
    }

    /// Publishes the group's metadata, private origin raw ivars and initial Class cells.
    ///
    /// Failure discards all candidates and private slots without undoing writes
    /// to published state. Stale storage left by direct registry commit or
    /// rollback rejects the group before any metadata can be published.
    pub fn commit_declaration_group(&mut self) -> Result<Vec<ClassRevision>, ClassError> {
        if let Err(error) = self.validate_staged_storage() {
            self.roll_back_group();
            return Err(error);
        }
        let storage = std::mem::take(&mut self.staged_class_raw_ivars);
        let class_vars = std::mem::take(&mut self.staged_class_vars);
        self.staged_storage_bases.clear();
        let revisions = self.registry.commit_declaration_group()?;
        self.class_raw_ivars.extend(storage);
        self.class_vars.extend(class_vars);
        Ok(revisions)
    }

    /// Discards the entire candidate group and its private storage roots.
    ///
    /// Published Class storage and external side effects are unchanged.
    pub fn roll_back_group(&mut self) {
        self.staged_class_raw_ivars.clear();
        self.staged_class_vars.clear();
        self.staged_storage_bases.clear();
        self.registry.roll_back_group();
    }
}
