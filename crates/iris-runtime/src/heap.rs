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
    /// Storage slots. A compaction MOVES objects between slots, so this is a
    /// slab rather than a map: with objects living directly in a
    /// `HashMap<ObjectId, HeapObject>` there is no placement to move, and a
    /// "compaction" would be a no-op that could not exercise `D-111` at all.
    slots: Vec<Option<HeapObject>>,
    /// Maps each identity to its CURRENT slot. Compaction rewrites this rather
    /// than the identities, which is what keeps `ObjectId` opaque and stable
    /// while placement changes underneath it.
    slot_of: HashMap<ObjectId, usize>,
    next_object_id: u64,
    next_identity_hash: u64,
}

impl RuntimeHeap {
    /// Creates an empty runtime heap with a process-local identity-hash sequence.
    pub fn new() -> Self {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(0);
        Self {
            slots: Vec::new(),
            slot_of: HashMap::new(),
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
        let slot = self.slots.len();
        self.slots.push(Some(object));
        self.slot_of.insert(object_id, slot);
        self.next_object_id = next_object_id;
        self.next_identity_hash = next_identity_hash;
        Ok(object_id)
    }

    /// Looks up an object without exposing the table's internal storage.
    pub fn lookup(&self, object_id: ObjectId) -> Result<HeapObject, RuntimeError> {
        let slot = *self
            .slot_of
            .get(&object_id)
            .ok_or(RuntimeError::UnknownObjectId(object_id))?;
        self.slots
            .get(slot)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or(RuntimeError::UnknownObjectId(object_id))
    }

    /// Releases an object's storage slot.
    ///
    /// This is the vacancy primitive a compaction needs: without it the slab
    /// is always dense and `compact` could never relocate anything, so
    /// `D-111`'s "across movement by GC" would stay unobservable. It is
    /// deliberately NOT a collector - it decides nothing about reachability,
    /// it only releases a slot the caller has already determined is dead.
    pub fn free(&mut self, object_id: ObjectId) -> Result<(), RuntimeError> {
        let slot = self
            .slot_of
            .remove(&object_id)
            .ok_or(RuntimeError::UnknownObjectId(object_id))?;
        if let Some(occupant) = self.slots.get_mut(slot) {
            *occupant = None;
        }
        Ok(())
    }

    /// Relocates every live object, returning how many MOVED slot.
    ///
    /// `D-111` requires an object to keep its runtime-local identity hash
    /// "across movement by GC", which is only meaningful if movement can
    /// actually happen. Objects are therefore repacked into a dense prefix in
    /// identity order and the slot index is rewritten, while `ObjectId` and
    /// `identity_hash` are carried through untouched.
    ///
    /// This relocates rather than collects: without a root set it must not
    /// claim to decide reachability, so nothing is discarded here.
    pub fn compact(&mut self) -> usize {
        let mut live: Vec<(ObjectId, HeapObject)> = self
            .slot_of
            .iter()
            .filter_map(|(id, slot)| {
                self.slots
                    .get(*slot)
                    .and_then(Option::as_ref)
                    .map(|object| (*id, object.clone()))
            })
            .collect();
        // A deterministic order, since IRIS-V1-IDENTITY-C021 forbids an
        // observable outcome from depending on hash iteration order.
        live.sort_by_key(|(id, _)| *id);

        let mut moved = 0;
        let mut slots = Vec::with_capacity(live.len());
        let mut slot_of = HashMap::with_capacity(live.len());
        for (destination, (id, object)) in live.into_iter().enumerate() {
            if self.slot_of.get(&id) != Some(&destination) {
                moved += 1;
            }
            slots.push(Some(object));
            slot_of.insert(id, destination);
        }
        self.slots = slots;
        self.slot_of = slot_of;
        moved
    }

    /// Returns an object's current storage slot.
    ///
    /// This is deliberately NOT reachable from Iris source: `IRIS-V1-RUNTIME-V079`
    /// requires the fixture to expose no address, so placement stays a Rust-side
    /// detail used only to show that a compaction really moved something.
    #[cfg(test)]
    pub(crate) fn slot_of(&self, object_id: ObjectId) -> Option<usize> {
        self.slot_of.get(&object_id).copied()
    }
}

impl Default for RuntimeHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod compaction_tests {
    use super::*;

    fn instance(heap: &mut RuntimeHeap) -> ObjectId {
        let Ok(id) = heap.alloc(ClassId::new(1), HeapPayload::InstanceFields(Vec::new())) else {
            unreachable!("a fresh heap allocates")
        };
        id
    }

    fn identity_hash_of(heap: &RuntimeHeap, object: ObjectId) -> u64 {
        let Ok(found) = heap.lookup(object) else {
            unreachable!("the object is live")
        };
        found.identity_hash()
    }

    /// `D-111` keeps an object's runtime-local identity hash stable "across
    /// movement by GC". The compaction must therefore MOVE something, or the
    /// stability it demonstrates would be vacuous.
    #[test]
    fn identity_hash_survives_relocation() {
        let mut heap = RuntimeHeap::new();
        let first = instance(&mut heap);
        let second = instance(&mut heap);
        let third = instance(&mut heap);

        let before = identity_hash_of(&heap, third);
        let slot_before = heap.slot_of(third);

        // Freeing EARLIER slots is what gives the later object somewhere to
        // move to; compacting a dense heap would relocate nothing.
        assert!(heap.free(first).is_ok());
        assert!(heap.free(second).is_ok());
        let moved = heap.compact();

        // Then the object really changed placement
        assert_eq!(moved, 1);
        assert_ne!(heap.slot_of(third), slot_before);
        assert_eq!(heap.slot_of(third), Some(0));

        // ...and its identity and hash came through untouched.
        assert_eq!(identity_hash_of(&heap, third), before);
    }

    /// A compaction relocates rather than collects, so it must not invent or
    /// discard identities.
    #[test]
    fn relocation_preserves_every_live_object() {
        let mut heap = RuntimeHeap::new();
        let ids: Vec<_> = (0..4).map(|_| instance(&mut heap)).collect();
        let hashes: Vec<_> = ids.iter().map(|id| identity_hash_of(&heap, *id)).collect();

        assert_eq!(heap.compact(), 0, "a dense heap relocates nothing");

        for (id, hash) in ids.iter().zip(hashes) {
            assert_eq!(identity_hash_of(&heap, *id), hash);
        }
    }
}
