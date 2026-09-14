use std::collections::HashMap;

use super::Runtime;
use crate::{Reachable, Value};

impl Runtime {
    /// Traces runtime storage and caller-supplied engine edges without sweeping.
    ///
    /// Explicit roots are borrowed. Committed Class raw ivars and variable cells
    /// are implicit roots, as are staged cells whose Class still has an active
    /// unpublished origin. Stale staged storage is ignored, not removed.
    ///
    /// `edges` observes every encountered typed value, including `Method` and
    /// the exact `Method` inside a `BoundMethod`. Append engine-owned captures,
    /// receivers, continuation values, etc. to the supplied worklist. The walk
    /// drains that worklist to a fixed point before returning. Do not remove
    /// previously queued edges. Visit order and repeated observations are not
    /// guaranteed; the callback must expand each engine handle only once using
    /// separate visited sets per kind (and Method identity), not guessed ID ranges.
    /// Runtime heap identities and shared container bodies are deduplicated here.
    ///
    /// Run at a safepoint: neither the callback nor another thread may mutate
    /// the graph, execute Iris code, or sweep engine records during tracing.
    /// Only the callback supplies engine edges; engine record tables are NOT
    /// implicit roots. The caller must root all live frames, bindings, active
    /// calls, scheduler obligations, and other externally retained values.
    /// `Reachable` records ObjectIds, not MethodIds or typed engine liveness;
    /// retain callback visited sets for engine sweeps after the fixed point.
    ///
    /// Runtime does not clone persistent roots. Shared mutable container snapshots
    /// and callback-supplied owned edges may temporarily increase share counts;
    /// all such trace temporaries are dropped before return, including unwinding.
    pub fn trace_with_edges<'a>(
        &self,
        roots: impl IntoIterator<Item = &'a Value>,
        mut edges: impl FnMut(&Value, &mut Vec<Value>),
    ) -> Reachable {
        let committed = self
            .class_raw_ivars
            .values()
            .flat_map(HashMap::values)
            .chain(self.class_vars.values());
        let staged_ivars = self
            .staged_class_raw_ivars
            .iter()
            .filter(|(class, _)| self.staged_storage_is_current(**class))
            .flat_map(|(_, slots)| slots.values());
        let staged_vars = self
            .staged_class_vars
            .iter()
            .filter(|((class, _), _)| self.staged_storage_is_current(*class))
            .map(|(_, value)| value);
        crate::trace::trace(
            (&self.heap, &self.raw_ivars),
            (roots, committed.chain(staged_ivars).chain(staged_vars)),
            &mut edges,
        )
    }

    /// Traces to the full engine/runtime fixed point, then frees unreachable heap
    /// identities and compacts survivors, returning `(freed, moved)`.
    ///
    /// The root and callback contracts are those of [`Self::trace_with_edges`].
    /// No heap sweep or relocation occurs before tracing finishes. Invalid staged
    /// cells are pruned as in [`Self::collect_garbage`]. Engine-owned records are
    /// not swept; the engine may sweep them using its visited sets after return.
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    /// use iris_runtime::{ObjectId, Runtime, StaticSpine, Value};
    /// struct Engine {
    ///     runtime: Runtime,
    ///     captures: HashMap<ObjectId, Vec<Value>>,
    ///     roots: Vec<Value>,
    /// }
    /// let mut engine = Engine {
    ///     runtime: Runtime::new(), captures: HashMap::new(), roots: Vec::new(),
    /// };
    /// let class = engine.runtime.registry_mut().define_class(StaticSpine::new(1), None)?;
    /// let captured = engine.runtime.allocate(class)?;
    /// let closure = ObjectId::new(100);
    /// engine.captures.insert(closure, vec![Value::Object(captured)]);
    /// engine.roots.push(Value::Closure(closure));
    /// let mut live_closures = HashSet::new();
    /// let captures = &engine.captures;
    /// let (freed, _) = engine.runtime.collect_with_edges(&engine.roots, |value, edges| {
    ///     if let Value::Closure(id) = value {
    ///         if live_closures.insert(*id) {
    ///             if let Some(values) = captures.get(id) {
    ///                 edges.extend(values.iter().cloned());
    ///             }
    ///         }
    ///     }
    /// });
    /// engine.captures.retain(|id, _| live_closures.contains(id));
    /// assert_eq!(freed, 0);
    /// assert_eq!(engine.runtime.class_of(captured)?, class);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn collect_with_edges<'a>(
        &mut self,
        roots: impl IntoIterator<Item = &'a Value>,
        edges: impl FnMut(&Value, &mut Vec<Value>),
    ) -> (usize, usize) {
        let reachable = self.trace_with_edges(roots, edges);
        self.staged_storage_bases.retain(|class, base| {
            self.registry.is_staging(*class) && self.registry.active_revision(*class).ok() == *base
        });
        self.staged_class_raw_ivars
            .retain(|class, _| self.staged_storage_bases.contains_key(class));
        self.staged_class_vars
            .retain(|(class, _), _| self.staged_storage_bases.contains_key(class));
        let garbage: Vec<_> = self
            .heap
            .live_ids()
            .into_iter()
            .filter(|id| !reachable.contains(*id))
            .collect();
        let freed = garbage.len();
        for dead in garbage {
            drop(self.heap.free(dead));
            self.instance_type_arguments.remove(&dead);
        }
        (freed, self.heap.compact())
    }
}
