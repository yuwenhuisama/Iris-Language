use super::gc_roots::TraceRoots;
use super::{Binding, EvaluationError, SourceEvaluator, Value};
use std::collections::HashSet;

impl SourceEvaluator {
    pub(super) fn collect_garbage(&mut self) -> Result<usize, EvaluationError> {
        let mut roots = Vec::new();
        roots.extend(self.upgrade_state.slots.values().cloned());
        for (_, context) in self.lazy_class_properties.values() {
            context.trace_roots(&mut roots);
        }
        self.active_roots.trace(&mut roots);
        self.frames.trace_roots(&mut roots);
        roots.extend(
            self.names
                .values()
                .chain(self.globals.values())
                .map(Binding::value),
        );
        roots.extend(
            self.module_constants
                .values()
                .chain(self.imported_names.values())
                .cloned(),
        );
        roots.extend(
            self.event_errors
                .iter()
                .chain(&self.discarded_contexts)
                .cloned(),
        );
        roots.extend(self.committed_targets.values().flatten().cloned());
        roots.extend(self.decorator_lifetime_diagnostics.iter().cloned());
        roots.extend(self.contract_metadata.values().cloned());
        roots.extend(
            self.active_exception
                .iter()
                .chain(&self.active_context)
                .cloned(),
        );
        roots.extend(
            self.suspended
                .keys()
                .chain(&self.ready)
                .copied()
                .map(Value::Task),
        );
        roots.extend(self.current_task.map(Value::Task));
        roots.extend(
            self.unobserved_failures
                .iter()
                .map(|(task, _)| Value::Task(*task)),
        );
        roots.extend(
            self.unobserved_failures
                .iter()
                .map(|(_, value)| value.clone()),
        );
        roots.extend(
            self.revision_subscribers
                .iter()
                .map(|subscriber| Value::Closure(subscriber.callback)),
        );
        roots.extend(self.current_method.map(Value::Method));
        if let Some(context) = &self.rollback_state.context {
            roots.extend(context.methods.values().copied().map(Value::Method));
        }
        roots.extend(
            self.module_methods
                .values()
                .chain(self.qualified_methods.values())
                .copied()
                .map(Value::Method),
        );
        for class in self
            .static_superclasses
            .keys()
            .chain(self.module_classes.values())
            .copied()
        {
            if let Ok(active) = self.runtime.registry().active(class) {
                roots.extend(
                    active
                        .methods()
                        .values()
                        .chain(active.singleton_methods().values())
                        .filter_map(|identity| self.runtime.registry().method_by_id(*identity))
                        .map(Value::Method),
                );
            }
            for selector in self.selectors.values() {
                if let Some(staged) = self.runtime.registry().staged_method(class, *selector) {
                    roots.extend(
                        self.runtime
                            .registry()
                            .method_by_id(staged)
                            .map(Value::Method),
                    );
                }
            }
        }
        let mut closures = HashSet::new();
        let mut tasks = HashSet::new();
        let mut gates = HashSet::new();
        let mut generators = HashSet::new();
        let mut arrays = HashSet::new();
        let mut hashes = HashSet::new();
        let mut bytes = HashSet::new();
        let mut methods = HashSet::new();
        let (freed, _) = self
            .runtime
            .collect_with_edges(&roots, |value, edges| match value {
                Value::Closure(identity) if closures.insert(*identity) => {
                    if let Some(record) = self.closures.get(identity) {
                        record.captured.trace_roots(edges);
                        record.cells.trace_roots(edges);
                        record.receiver.trace_roots(edges);
                        record.lexical_context.trace_roots(edges);
                    }
                    if let Some(call) = self
                        .wrapper_next
                        .get(identity)
                        .and_then(|entry| entry.call.as_ref())
                    {
                        call.invocation
                            .visit_values(&mut |value| edges.push(value.clone()));
                        call.chain.trace_roots(edges);
                        call.context.trace_roots(edges);
                        if let Some(entry) = self.wrapper_next.get(identity) {
                            entry
                                .scope
                                .borrow()
                                .visit_values(&mut |value| edges.push(value.clone()));
                        }
                    }
                }
                Value::Method(method) if methods.insert(method.id()) => {
                    if let Some(chain) = self.wrapper_chains.get(&method.id()) {
                        chain.trace_roots(edges);
                    }
                    if let Some(closure) = self.captured_methods.get(&method.body().raw()) {
                        edges.push(Value::Closure(*closure));
                    }
                    if let Some(context) = self.rollback_state.bodies.get(&method.body().raw()) {
                        edges.extend(context.methods.values().copied().map(Value::Method));
                    }
                }
                Value::Task(identity) if tasks.insert(*identity) => {
                    edges.extend(self.task_types.get(identity).cloned());
                    edges.extend(self.task_contexts.get(identity).cloned());
                    if let Some(outcome) = self.tasks.get(identity) {
                        match outcome {
                            Ok(value) => edges.push(value.clone()),
                            Err(error) => Err::<Value, _>(*error.clone()).trace_roots(edges),
                        }
                    }
                    if let Some(task) = self.suspended.get(identity) {
                        edges.extend(task.continuation.roots());
                        if self.gates.contains_key(&task.waiting) {
                            edges.push(Value::Gate(task.waiting));
                        } else {
                            edges.push(Value::Task(task.waiting));
                        }
                    }
                }
                Value::Gate(identity) if gates.insert(*identity) => {
                    edges.extend(self.gates.get(identity).into_iter().flatten().cloned())
                }
                Value::Generator(identity) if generators.insert(*identity) => {
                    if let Some(body) = self.generators.get(identity) {
                        body.locals.trace_roots(edges);
                        body.receiver.trace_roots(edges);
                    }
                }
                Value::ArrayIterator(identity) if arrays.insert(*identity) => {
                    if let Some(cursor) = self.array_iterators.get(identity) {
                        edges.extend(cursor.values.clone().map(Value::Array));
                    }
                }
                Value::HashIterator(identity) if hashes.insert(*identity) => {
                    if let Some(cursor) = self.hash_iterators.get(identity) {
                        edges.push(Value::Hash(cursor.entries.clone()));
                        cursor.keys.trace_roots(edges);
                        cursor.yielded.trace_roots(edges);
                    }
                }
                Value::ByteIterator(identity) if bytes.insert(*identity) => {
                    if let Some(cursor) = self.byte_iterators.get(identity) {
                        cursor.values.trace_roots(edges);
                    }
                }
                _ => {}
            });
        self.closures
            .retain(|identity, _| closures.contains(identity));
        self.wrapper_next
            .retain(|identity, _| closures.contains(identity));
        self.tasks.retain(|identity, _| tasks.contains(identity));
        self.wrapper_chains
            .retain(|identity, _| methods.contains(identity));
        self.task_contexts
            .retain(|identity, _| tasks.contains(identity));
        self.task_types
            .retain(|identity, _| tasks.contains(identity));
        self.gates.retain(|identity, _| gates.contains(identity));
        self.generators
            .retain(|identity, _| generators.contains(identity));
        self.array_iterators
            .retain(|identity, _| arrays.contains(identity));
        self.hash_iterators
            .retain(|identity, _| hashes.contains(identity));
        self.byte_iterators
            .retain(|identity, _| bytes.contains(identity));
        Ok(freed)
    }
}
