use core::fmt;
use std::{
    collections::{HashMap, hash_map::RandomState},
    error::Error,
    hash::{BuildHasher, Hasher},
};

use crate::{ClassId, ObjectId, Value};

/// Object data currently supported by the runtime heap.
#[derive(Clone, Debug, PartialEq)]
pub enum HeapPayload {
    /// Slots owned by an ordinary instance.
    InstanceFields(Vec<Value>),
}

/// A heap-owned Iris object returned as an owned snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct HeapObject {
    class_id: ClassId,
    identity_hash: u64,
    payload: HeapPayload,
}

impl HeapObject {
    /// Returns the object's logical class identity.
    pub const fn class_id(&self) -> ClassId {
        self.class_id
    }

    /// Returns the object's runtime-local, stable identity hash.
    pub const fn identity_hash(&self) -> u64 {
        self.identity_hash
    }

    /// Returns the object's payload snapshot.
    pub const fn payload(&self) -> &HeapPayload {
        &self.payload
    }
}

/// A failure produced by the runtime heap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    /// The requested object is not owned by this heap.
    UnknownObjectId(ObjectId),
    /// The heap cannot issue another object identity.
    ObjectIdentityExhausted,
    /// The heap cannot issue another identity hash.
    IdentityHashExhausted,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownObjectId(_) => formatter.write_str("unknown Iris object identity"),
            Self::ObjectIdentityExhausted => {
                formatter.write_str("Iris object identity space exhausted")
            }
            Self::IdentityHashExhausted => {
                formatter.write_str("Iris identity hash space exhausted")
            }
        }
    }
}

impl Error for RuntimeError {}

/// The runtime-owned table through which all ordinary objects are allocated and found.
#[derive(Debug)]
pub struct RuntimeHeap {
    objects: HashMap<ObjectId, HeapObject>,
    next_object_id: u64,
    next_identity_hash: u64,
}

impl RuntimeHeap {
    /// Creates an empty runtime heap with a process-local identity-hash sequence.
    pub fn new() -> Self {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(0);
        Self {
            objects: HashMap::new(),
            next_object_id: 0,
            next_identity_hash: hasher.finish(),
        }
    }

    /// Allocates an object and returns its opaque identity.
    pub fn alloc(
        &mut self,
        class_id: ClassId,
        payload: HeapPayload,
    ) -> Result<ObjectId, RuntimeError> {
        let object_id = ObjectId::new(self.next_object_id);
        let next_object_id = self
            .next_object_id
            .checked_add(1)
            .ok_or(RuntimeError::ObjectIdentityExhausted)?;
        let identity_hash = self.next_identity_hash;
        let next_identity_hash = self
            .next_identity_hash
            .checked_add(1)
            .ok_or(RuntimeError::IdentityHashExhausted)?;
        let object = HeapObject {
            class_id,
            identity_hash,
            payload,
        };
        self.objects.insert(object_id, object);
        self.next_object_id = next_object_id;
        self.next_identity_hash = next_identity_hash;
        Ok(object_id)
    }

    /// Looks up an object without exposing the table's internal storage.
    pub fn lookup(&self, object_id: ObjectId) -> Result<HeapObject, RuntimeError> {
        self.objects
            .get(&object_id)
            .cloned()
            .ok_or(RuntimeError::UnknownObjectId(object_id))
    }
}

impl Default for RuntimeHeap {
    fn default() -> Self {
        Self::new()
    }
}
