use core::fmt;
use std::{collections::HashMap, error::Error};

use crate::{
    ClassError, ClassId, ClassRegistry, DispatchError, DispatchOutcome, HeapPayload, Method,
    MethodId, MethodOwner, ObjectId, RuntimeHeap, Selector, Value, Visibility,
};

/// An evaluator-raised Iris value or runtime storage failure.
#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionError {
    Raised(Value),
    Class(ClassError),
    Heap(crate::RuntimeError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raised(_) => formatter.write_str("Iris execution raised a value"),
            Self::Class(error) => error.fmt(formatter),
            Self::Heap(error) => error.fmt(formatter),
        }
    }
}

impl Error for ExecutionError {}

impl From<crate::RuntimeError> for ExecutionError {
    fn from(error: crate::RuntimeError) -> Self {
        Self::Heap(error)
    }
}

impl From<ClassError> for ExecutionError {
    fn from(error: ClassError) -> Self {
        Self::Class(error)
    }
}

/// A typed failure from construction or object storage.
#[derive(Clone, Debug, PartialEq)]
pub enum ConstructionError {
    Class(ClassError),
    Dispatch(DispatchError),
    Runtime(ExecutionError),
    EscapedObjectMissing,
    MissingDeclaredClassVariable { class: ClassId, name: Selector },
    ImmutableClassVariable { class: ClassId, name: Selector },
}

impl fmt::Display for ConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(error) => error.fmt(formatter),
            Self::Dispatch(error) => write!(formatter, "dispatch error: {error:?}"),
            Self::Runtime(error) => error.fmt(formatter),
            Self::EscapedObjectMissing => formatter.write_str("escaped object was not retained"),
            Self::MissingDeclaredClassVariable { .. } => {
                formatter.write_str("declared Iris Class variable storage is missing")
            }
            Self::ImmutableClassVariable { .. } => {
                formatter.write_str("declared Iris Class variable storage is immutable")
            }
        }
    }
}

impl Error for ConstructionError {}

impl From<ClassError> for ConstructionError {
    fn from(error: ClassError) -> Self {
        Self::Class(error)
    }
}

impl From<DispatchError> for ConstructionError {
    fn from(error: DispatchError) -> Self {
        Self::Dispatch(error)
    }
}

impl From<ExecutionError> for ConstructionError {
    fn from(error: ExecutionError) -> Self {
        Self::Runtime(error)
    }
}

/// Runtime-owned ordinary-object state addressed only by opaque identities.
#[derive(Debug, Default)]
pub struct Runtime {
    registry: ClassRegistry,
    heap: RuntimeHeap,
    raw_ivars: HashMap<ObjectId, HashMap<Selector, Value>>,
    class_raw_ivars: HashMap<ClassId, HashMap<Selector, Value>>,
    class_vars: HashMap<(ClassId, Selector), Value>,
}

impl Runtime {
    /// Creates an empty runtime.
    pub fn new() -> Self {
        Self::default()
    }

    /// Exposes Class declarations and publications to the embedding evaluator.
    pub fn registry_mut(&mut self) -> &mut ClassRegistry {
        &mut self.registry
    }

    /// Exposes Class dispatch state to the embedding evaluator.
    pub fn registry(&self) -> &ClassRegistry {
        &self.registry
    }

    /// Allocates a complete ordinary instance.
    pub fn allocate(&mut self, class: ClassId) -> Result<ObjectId, ConstructionError> {
        self.registry.class(class)?;
        self.heap
            .alloc(class, HeapPayload::InstanceFields(Vec::new()))
            .map_err(ExecutionError::from)
            .map_err(ConstructionError::from)
    }

    /// Constructs against the revision active when construction began.
    pub fn construct<F>(
        &mut self,
        class: ClassId,
        arguments: &[Value],
        mut invoke: F,
    ) -> Result<ObjectId, ConstructionError>
    where
        F: FnMut(&mut Self, Method, ObjectId, &[Value]) -> Result<Value, ExecutionError>,
    {
        let snapshot = self.registry.active(class)?.clone();
        let revisions = snapshot
            .mro()
            .iter()
            .filter_map(|entry| match entry {
                crate::MroEntry::Class(owner) => Some(*owner),
                crate::MroEntry::Module(_) => None,
            })
            .map(|owner| {
                self.registry
                    .active(owner)
                    .map(|revision| (owner, revision.clone()))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let instance = self.allocate(class)?;
        for entry in snapshot.mro().iter().rev() {
            if let crate::MroEntry::Class(owner) = entry {
                let revision = revisions
                    .get(owner)
                    .ok_or(ClassError::UnknownClassId(*owner))?;
                for property in revision.properties() {
                    let body = property.initializer();
                    let initializer = Method::new(
                        MethodId::new(body.raw()),
                        MethodOwner::Class(*owner),
                        property.selector(),
                        body,
                        Visibility::Public,
                    );
                    invoke(self, initializer, instance, &[])?;
                }
            }
        }
        match self.dispatch_snapshot(&snapshot, &revisions, Selector::new(1))? {
            DispatchOutcome::Invoke(method) => {
                invoke(self, method, instance, arguments)?;
            }
            DispatchOutcome::WouldInvokeMethodMissing { .. } => {}
        }
        Ok(instance)
    }

    /// Resolves a later ordinary send through the current active revision.
    pub fn dispatch_instance(
        &self,
        instance: ObjectId,
        selector: Selector,
    ) -> Result<Method, ConstructionError> {
        match self.registry.dispatch(self.class_of(instance)?, selector)? {
            DispatchOutcome::Invoke(method) => Ok(method),
            DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                Err(DispatchError::MissingMethod { selector }.into())
            }
        }
    }

    /// Sends a setter and returns its actual Method result.
    pub fn assign_property<F>(
        &mut self,
        instance: ObjectId,
        setter: Selector,
        value: Value,
        mut invoke: F,
    ) -> Result<Value, ConstructionError>
    where
        F: FnMut(&mut Self, Method, ObjectId, &[Value]) -> Result<Value, ExecutionError>,
    {
        let method = self.dispatch_instance(instance, setter)?;
        invoke(self, method, instance, &[value]).map_err(ConstructionError::from)
    }

    /// Reads an absent raw ivar as nil without materializing it.
    pub fn raw_ivar(&self, instance: ObjectId, name: Selector) -> Result<Value, ConstructionError> {
        self.class_of(instance)?;
        Ok(self
            .raw_ivars
            .get(&instance)
            .and_then(|slots| slots.get(&name))
            .cloned()
            .unwrap_or(Value::Nil))
    }

    /// Stores a raw receiver ivar and returns the stored value.
    pub fn assign_raw_ivar(
        &mut self,
        instance: ObjectId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        self.class_of(instance)?;
        self.raw_ivars
            .entry(instance)
            .or_default()
            .insert(name, value.clone());
        Ok(value)
    }

    /// Counts materialized raw ivars for a receiver.
    pub fn raw_ivar_count(&self, instance: ObjectId) -> Result<usize, ConstructionError> {
        self.class_of(instance)?;
        Ok(self.raw_ivars.get(&instance).map_or(0, HashMap::len))
    }

    /// Returns the materialized raw ivar names for a receiver.
    pub fn raw_ivar_names(&self, instance: ObjectId) -> Result<Vec<Selector>, ConstructionError> {
        self.class_of(instance)?;
        Ok(self
            .raw_ivars
            .get(&instance)
            .map(|slots| slots.keys().copied().collect())
            .unwrap_or_default())
    }

    /// Removes and returns a materialized raw ivar when it exists.
    pub fn remove_raw_ivar(
        &mut self,
        instance: ObjectId,
        name: Selector,
    ) -> Result<Option<Value>, ConstructionError> {
        self.class_of(instance)?;
        Ok(self
            .raw_ivars
            .get_mut(&instance)
            .and_then(|slots| slots.remove(&name)))
    }

    /// Stores a Class variable cell and returns its stored value.
    pub fn declare_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
        mutable: bool,
    ) -> Result<Value, ConstructionError> {
        self.registry.declare_class_var(class, name, mutable)?;
        self.class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    /// Reads an absent Class-object raw ivar as nil without materializing it.
    pub fn class_raw_ivar(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<Value, ConstructionError> {
        self.registry.class(class)?;
        Ok(self
            .class_raw_ivars
            .get(&class)
            .and_then(|slots| slots.get(&name))
            .cloned()
            .unwrap_or(Value::Nil))
    }

    /// Stores a Class-object raw ivar and returns the stored value.
    pub fn assign_class_raw_ivar(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        self.registry.class(class)?;
        self.class_raw_ivars
            .entry(class)
            .or_default()
            .insert(name, value.clone());
        Ok(value)
    }

    /// Returns materialized raw ivar names on a Class object.
    pub fn class_raw_ivar_names(&self, class: ClassId) -> Result<Vec<Selector>, ConstructionError> {
        self.registry.class(class)?;
        Ok(self
            .class_raw_ivars
            .get(&class)
            .map(|slots| slots.keys().copied().collect())
            .unwrap_or_default())
    }

    /// Removes and returns a materialized raw ivar from a Class object.
    pub fn remove_class_raw_ivar(
        &mut self,
        class: ClassId,
        name: Selector,
    ) -> Result<Option<Value>, ConstructionError> {
        self.registry.class(class)?;
        Ok(self
            .class_raw_ivars
            .get_mut(&class)
            .and_then(|slots| slots.remove(&name)))
    }

    /// Stores an existing Class variable cell and returns its stored value.
    pub fn assign_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        value: Value,
    ) -> Result<Value, ConstructionError> {
        let declared = self.registry.active(class)?.class_vars().contains(&name);
        if !declared {
            return Err(ConstructionError::MissingDeclaredClassVariable { class, name });
        }
        if self
            .registry
            .active(class)?
            .immutable_class_vars()
            .contains(&name)
        {
            return Err(ConstructionError::ImmutableClassVariable { class, name });
        }
        self.class_vars.insert((class, name), value.clone());
        Ok(value)
    }

    /// Reads a Class variable cell.
    pub fn class_var(
        &self,
        class: ClassId,
        name: Selector,
    ) -> Result<Option<Value>, ConstructionError> {
        let declared = self.registry.active(class)?.class_vars().contains(&name);
        if !declared {
            return Err(ConstructionError::MissingDeclaredClassVariable { class, name });
        }
        Ok(self.class_vars.get(&(class, name)).cloned())
    }

    /// Returns an instance's stable logical Class.
    pub fn class_of(&self, instance: ObjectId) -> Result<ClassId, ConstructionError> {
        self.heap
            .lookup(instance)
            .map(|object| object.class_id())
            .map_err(ExecutionError::from)
            .map_err(ConstructionError::from)
    }

    fn dispatch_snapshot(
        &self,
        snapshot: &crate::ClassRevision,
        revisions: &HashMap<ClassId, crate::ClassRevision>,
        selector: Selector,
    ) -> Result<DispatchOutcome, ConstructionError> {
        for entry in snapshot.mro() {
            match entry {
                crate::MroEntry::Class(owner) => {
                    let revision = revisions
                        .get(owner)
                        .ok_or(ClassError::UnknownClassId(*owner))?;
                    if let Some(method) = revision
                        .methods()
                        .get(&selector)
                        .and_then(|id| self.registry.method(*id))
                    {
                        return Ok(DispatchOutcome::Invoke(method));
                    }
                }
                crate::MroEntry::Module(module) => {
                    if let Some(method) = self.registry.module_method(*module, selector) {
                        return Ok(DispatchOutcome::Invoke(method));
                    }
                }
            }
        }
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    }
}
