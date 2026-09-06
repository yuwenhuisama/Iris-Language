//! Reachability tracing over the whole Iris object graph.
//!
//! Iris values live in two places at once. Ordinary instances live in the
//! runtime heap and are named by [`ObjectId`], while an Array, Hash, ByteArray
//! or MutableString body lives behind an `Arc<Mutex<..>>` outside it. Because
//! [`Value`] is a SINGLE enum, those two regions point at each other freely: an
//! instance field can hold an Array, and that Array's elements can hold further
//! instances.
//!
//! A collector that walked only the heap would therefore miss half the graph
//! and free objects that are still reachable through an Array. This module
//! walks BOTH regions in one pass, which is what makes the object graph
//! unified in the only sense a collector cares about: reachability.

use std::collections::{HashMap, HashSet};

use crate::{BoundReceiver, HeapPayload, ObjectId, RuntimeHeap, Selector, Value};

/// An object's raw receiver ivars, which the runtime stores OUTSIDE the heap.
pub type RawIvars = HashMap<ObjectId, HashMap<Selector, Value>>;

/// The set of identities reachable from a set of roots.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Reachable {
    objects: HashSet<ObjectId>,
}

impl Reachable {
    /// Whether `object` was reached.
    #[must_use]
    pub fn contains(&self, object: ObjectId) -> bool {
        self.objects.contains(&object)
    }

    /// How many identities were reached.
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Whether nothing was reached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// The reached identities, in a deterministic order.
    #[must_use]
    pub fn ids(&self) -> Vec<ObjectId> {
        let mut ids: Vec<ObjectId> = self.objects.iter().copied().collect();
        // IRIS-V1-IDENTITY-C021 forbids an observable outcome from depending on
        // runtime allocation or hash order.
        ids.sort_unstable();
        ids
    }
}

/// Walks `roots` and returns every identity reachable from them.
///
/// The walk crosses freely between the heap and the shared value cells, so an
/// instance held ONLY inside an Array is still reached. Visited cells are
/// recorded by [`crate::ArrayRef::cell_id`], so a container that contains
/// itself terminates instead of recursing forever.
///
/// `ivars` is required because an object's raw receiver ivars live in a
/// runtime-side table rather than in its heap payload. Walking the payload
/// alone missed every edge written through `assign_raw_ivar`, which made a
/// REACHABLE cycle look like garbage and freed live objects.
pub fn reachable_from<'a>(
    heap: &RuntimeHeap,
    ivars: &RawIvars,
    roots: impl IntoIterator<Item = &'a Value>,
) -> Reachable {
    let mut walker = Walker {
        heap,
        ivars,
        objects: HashSet::new(),
        cells: HashSet::new(),
    };
    for root in roots {
        walker.walk(root);
    }
    Reachable {
        objects: walker.objects,
    }
}

struct Walker<'a> {
    heap: &'a RuntimeHeap,
    ivars: &'a RawIvars,
    objects: HashSet<ObjectId>,
    cells: HashSet<usize>,
}

impl Walker<'_> {
    /// Records an identity and follows its heap payload.
    ///
    /// Re-entry is what terminates a cycle: an object already recorded is not
    /// expanded a second time.
    fn reach(&mut self, object: ObjectId) {
        if !self.objects.insert(object) {
            return;
        }
        // Raw ivars live OUTSIDE the heap payload, so they are followed even
        // for an identity the heap does not own.
        if let Some(slots) = self.ivars.get(&object) {
            for slot in slots.values().cloned().collect::<Vec<_>>() {
                self.walk(&slot);
            }
        }
        let Ok(found) = self.heap.lookup(object) else {
            // An identity with no heap entry is still REACHED - a Closure or a
            // Gate is named by an ObjectId whose body lives in the evaluator,
            // not here. Dropping it would let a sweep free a live identity.
            return;
        };
        let HeapPayload::InstanceFields(fields) = found.payload();
        for field in fields.clone() {
            self.walk(&field);
        }
    }

    /// One arm per `Value` variant, with NO catch-all: an omitted variant
    /// would silently under-approximate reachability and let a sweep free a
    /// live object, so a new variant must fail to compile here instead.
    fn walk(&mut self, value: &Value) {
        match value {
            // Identities whose payload the heap owns.
            Value::Object(id)
            | Value::Task(id)
            | Value::Generator(id)
            | Value::ArrayIterator(id)
            | Value::HashIterator(id)
            | Value::ByteIterator(id)
            | Value::Gate(id)
            | Value::Closure(id) => self.reach(*id),

            // Shared cells: the crossing point between the two regions.
            Value::Array(values) => {
                if self.cells.insert(values.cell_id()) {
                    for element in values.elements() {
                        self.walk(&element);
                    }
                }
            }
            Value::Hash(entries) => {
                if self.cells.insert(entries.cell_id()) {
                    for (key, entry) in entries.entries() {
                        self.walk(&key);
                        self.walk(&entry);
                    }
                }
            }

            // Inline sequences.
            Value::ReadonlyArray(values) | Value::Tuple(values) => {
                for element in values {
                    self.walk(element);
                }
            }

            // Single nested values.
            Value::StackFrame(_, inner)
            | Value::RaiseSite(inner)
            | Value::IterationYield(inner)
            | Value::KeywordArgument(_, inner)
            | Value::ContractView(inner, _) => self.walk(inner),

            Value::ExceptionContext(id, value, cause, suppressed, sites, location) => {
                self.reach(*id);
                self.walk(value);
                self.walk(cause);
                for entry in suppressed.iter().chain(sites) {
                    self.walk(entry);
                }
                self.walk(location);
            }

            Value::Transformation { staged, .. } => {
                for (_, id) in staged {
                    self.reach(*id);
                }
            }

            Value::BoundMethod(bound) => {
                self.reach(bound.id());
                if let BoundReceiver::Object(receiver) = bound.receiver() {
                    self.reach(receiver);
                }
            }

            Value::Library(library) => {
                for (_, signature) in &library.signatures {
                    self.walk(signature);
                }
            }

            // Leaves: these hold no Iris value that could keep an identity
            // alive. A ByteArray and a MutableString are shared cells too, but
            // their bodies are bytes and text rather than `Value`s, so there is
            // nothing further to reach through them.
            Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::SourceLocation(..)
            | Value::Range(_)
            | Value::NativeResource(_)
            | Value::Regex(_)
            | Value::Match(_)
            | Value::MutableString(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Class(_)
            | Value::ClosedClass(..)
            | Value::Method(_)
            | Value::Contract(..)
            | Value::IterationDone
            | Value::Type(..)
            | Value::ComposedType(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArrayRef, ClassId, HashRef};

    /// These tests build heap payloads directly, so no raw ivars exist.
    fn no_ivars() -> RawIvars {
        RawIvars::new()
    }

    fn instance(heap: &mut RuntimeHeap, fields: Vec<Value>) -> ObjectId {
        let Ok(id) = heap.alloc(ClassId::new(1), HeapPayload::InstanceFields(fields)) else {
            unreachable!("a fresh heap allocates")
        };
        id
    }

    /// The whole point of the unified walk: an instance held ONLY inside an
    /// Array is still reachable. A heap-only walk would miss it and a collector
    /// would free a live object.
    #[test]
    fn an_object_held_only_inside_an_array_is_reached() {
        let mut heap = RuntimeHeap::new();
        let hidden = instance(&mut heap, Vec::new());
        let holder = Value::Array(ArrayRef::new(vec![Value::Object(hidden)]));

        let reached = reachable_from(&heap, &no_ivars(), [&holder]);

        assert!(reached.contains(hidden));

        // Control: with the Array NOT among the roots, the same object is
        // unreachable. Without this the assertion above could hold simply
        // because everything is always reported reachable.
        let nothing = reachable_from(&heap, &no_ivars(), []);
        assert!(!nothing.contains(hidden));
        assert!(nothing.is_empty());
    }

    /// The walk crosses BACK: heap field -> Array -> Hash -> heap field.
    #[test]
    fn the_walk_crosses_between_the_heap_and_shared_cells_repeatedly() {
        let mut heap = RuntimeHeap::new();
        let deep = instance(&mut heap, Vec::new());
        let inner = HashRef::new(vec![(Value::Symbol("k".to_owned()), Value::Object(deep))]);
        let middle = ArrayRef::new(vec![Value::Hash(inner)]);
        let root = instance(&mut heap, vec![Value::Array(middle)]);

        let reached = reachable_from(&heap, &no_ivars(), [&Value::Object(root)]);

        assert!(reached.contains(root));
        assert!(reached.contains(deep), "the walk must re-enter the heap");
        assert_eq!(reached.len(), 2);
    }

    /// A container that contains ITSELF must terminate rather than recurse
    /// forever. `Arc` cannot reclaim such a cycle, which is precisely why a
    /// tracing collector is needed.
    #[test]
    fn a_self_referential_array_terminates() {
        let mut heap = RuntimeHeap::new();
        let held = instance(&mut heap, Vec::new());
        let cycle = ArrayRef::new(vec![Value::Object(held)]);
        cycle.mutate(|elements| elements.push(Value::Array(cycle.clone())));

        let reached = reachable_from(&heap, &no_ivars(), [&Value::Array(cycle)]);

        assert!(reached.contains(held));
        assert_eq!(reached.len(), 1);
    }

    /// An object cycle through heap fields terminates too.
    #[test]
    fn mutually_referring_objects_terminate() {
        let mut heap = RuntimeHeap::new();
        let first = instance(&mut heap, Vec::new());
        let second = instance(&mut heap, vec![Value::Object(first)]);
        let Ok(()) = heap.set_fields(first, vec![Value::Object(second)]) else {
            unreachable!("both identities are live")
        };

        let reached = reachable_from(&heap, &no_ivars(), [&Value::Object(first)]);

        assert_eq!(reached.ids(), vec![first, second]);
    }

    /// A dead object is exactly what `owned - reachable` reports, which is the
    /// set a sweep may free.
    #[test]
    fn unreached_owned_identities_are_the_dead_set() {
        let mut heap = RuntimeHeap::new();
        let live = instance(&mut heap, Vec::new());
        let dead = instance(&mut heap, Vec::new());

        let reached = reachable_from(&heap, &no_ivars(), [&Value::Object(live)]);
        let garbage: Vec<ObjectId> = heap
            .live_ids()
            .into_iter()
            .filter(|id| !reached.contains(*id))
            .collect();

        assert_eq!(garbage, vec![dead]);
    }
}

#[cfg(test)]
mod collect_tests {
    use crate::{ArrayRef, ClassId, ObjectId, Runtime, Selector, StaticSpine, Value};

    const FIELD: Selector = Selector::new(9);

    fn class_of(runtime: &mut Runtime) -> ClassId {
        let Ok(class) = runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)
        else {
            unreachable!("a fresh registry defines a class")
        };
        class
    }

    fn instance(runtime: &mut Runtime, class: ClassId) -> ObjectId {
        let Ok(id) = runtime.allocate(class) else {
            unreachable!("a defined class allocates")
        };
        id
    }

    fn point_at(runtime: &mut Runtime, holder: ObjectId, target: Value) {
        let Ok(_) = runtime.assign_raw_ivar(holder, FIELD, target) else {
            unreachable!("the holder is live")
        };
    }

    /// A collector decides what died; the caller supplies roots only.
    #[test]
    fn unreachable_objects_are_freed_and_reachable_ones_survive() {
        let mut runtime = Runtime::new();
        let class = class_of(&mut runtime);
        let live = instance(&mut runtime, class);
        let dead = instance(&mut runtime, class);

        let (freed, _) = runtime.collect_garbage([&Value::Object(live)]);

        assert_eq!(freed, 1);
        assert!(runtime.identity_hash(live).is_ok(), "the root survives");
        assert!(runtime.identity_hash(dead).is_err(), "the garbage is gone");
    }

    /// The crossing a heap-only collector would get WRONG: an object held only
    /// inside an Array body is still live.
    #[test]
    fn an_object_held_only_inside_an_array_is_not_freed() {
        let mut runtime = Runtime::new();
        let class = class_of(&mut runtime);
        let hidden = instance(&mut runtime, class);
        let holder = Value::Array(ArrayRef::new(vec![Value::Object(hidden)]));

        let (freed, _) = runtime.collect_garbage([&holder]);

        assert_eq!(freed, 0);
        assert!(runtime.identity_hash(hidden).is_ok());

        // Control: with the Array no longer a root, the same object IS
        // collected. Without this, survival above could mean the collector
        // simply never frees anything.
        let (freed, _) = runtime.collect_garbage([]);
        assert_eq!(freed, 1);
        assert!(runtime.identity_hash(hidden).is_err());
    }

    /// A cycle is exactly what reference counting cannot reclaim, and the
    /// reason a tracing collector is needed at all.
    #[test]
    fn an_unreachable_cycle_is_reclaimed() {
        let mut runtime = Runtime::new();
        let class = class_of(&mut runtime);
        let first = instance(&mut runtime, class);
        let second = instance(&mut runtime, class);
        point_at(&mut runtime, first, Value::Object(second));
        point_at(&mut runtime, second, Value::Object(first));

        // Nothing outside the cycle refers to it.
        let (freed, _) = runtime.collect_garbage([]);

        assert_eq!(freed, 2);
        assert!(runtime.identity_hash(first).is_err());
        assert!(runtime.identity_hash(second).is_err());
    }

    /// A live cycle is NOT reclaimed: reachability decides, not shape.
    #[test]
    fn a_reachable_cycle_survives() {
        let mut runtime = Runtime::new();
        let class = class_of(&mut runtime);
        let first = instance(&mut runtime, class);
        let second = instance(&mut runtime, class);
        point_at(&mut runtime, first, Value::Object(second));
        point_at(&mut runtime, second, Value::Object(first));

        let (freed, _) = runtime.collect_garbage([&Value::Object(first)]);

        assert_eq!(freed, 0);
        assert!(runtime.identity_hash(first).is_ok());
        assert!(runtime.identity_hash(second).is_ok());
    }

    /// `D-111` keeps an identity hash stable across movement BY GC, which is
    /// finally observable: freeing garbage leaves a gap the survivor moves into.
    #[test]
    fn a_surviving_object_keeps_its_identity_hash_across_collection() {
        let mut runtime = Runtime::new();
        let class = class_of(&mut runtime);
        let dead = instance(&mut runtime, class);
        let live = instance(&mut runtime, class);
        let Ok(before) = runtime.identity_hash(live) else {
            unreachable!("the object is live")
        };

        let (freed, moved) = runtime.collect_garbage([&Value::Object(live)]);

        assert_eq!(freed, 1);
        assert_eq!(moved, 1, "the survivor moves into the freed slot");
        assert!(runtime.identity_hash(dead).is_err());
        assert_eq!(runtime.identity_hash(live), Ok(before));
    }
}
