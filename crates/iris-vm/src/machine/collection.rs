use super::{IteratorSource, Machine, MachineError, PendingFrame};
use iris_runtime::{MethodId, ObjectId, Value};
use std::collections::HashSet;

#[derive(Default)]
struct Visited {
    closures: HashSet<ObjectId>,
    tasks: HashSet<ObjectId>,
    methods: HashSet<MethodId>,
    gates: HashSet<ObjectId>,
    iterators: HashSet<ObjectId>,
    bound_methods: HashSet<ObjectId>,
}

fn frame_values(frame: &PendingFrame, roots: &mut Vec<Value>) {
    roots.extend(frame.registers.iter().cloned());
    roots.extend(frame.method_types.iter().map(|(_, value)| value.clone()));
    roots.extend(frame.cleanup.iter().cloned());
    for continuation in &frame.continuations {
        frame_values(continuation, roots);
    }
}

impl Machine {
    pub(super) fn collect_engine(&mut self) -> usize {
        let mut roots = self.active_values.clone();
        for local in &self.local_roots {
            local.append_roots(&mut roots);
        }
        for frame in &self.frame_roots {
            let registers = frame.registers.borrow();
            match &frame.selected {
                Some(selected) => roots.extend(
                    selected
                        .iter()
                        .map(|index| registers[usize::from(*index)].clone()),
                ),
                None => roots.extend(registers.iter().cloned()),
            }
        }
        roots.extend(self.globals.values().chain(self.bindings.values()).cloned());
        roots.extend(self.method_types.iter().map(|(_, value)| value.clone()));
        roots.extend(self.explicit_method_types.iter().cloned());
        roots.extend(self.pending_cleanup_cause.iter().cloned());
        roots.extend(
            self.discarded_contexts
                .iter()
                .chain(&self.revision_event_errors)
                .chain(&self.decorator_lifetime_diagnostics)
                .cloned(),
        );
        roots.extend(
            self.decorator_metadata
                .iter()
                .cloned()
                .map(Value::ImmutableHash),
        );
        roots.extend(self.contracts.values().cloned().map(Value::ImmutableHash));
        roots.extend(
            self.revision_subscribers
                .iter()
                .map(|subscriber| Value::Closure(subscriber.callback)),
        );
        roots.extend(self.unobserved_failures.iter().copied().map(Value::Task));
        roots.extend(self.current_task.map(|task| Value::Task(task.object_id())));
        roots.extend(
            self.task_types
                .keys()
                .filter(|id| !self.tasks.contains_key(id))
                .copied()
                .map(Value::Task),
        );
        for task in self
            .suspended
            .iter()
            .chain(self.ready_tasks.iter().flatten())
        {
            roots.push(Value::Task(task.identity));
            roots.push(if self.gates.contains_key(&task.frame.gate) {
                Value::Gate(task.frame.gate)
            } else {
                Value::Task(task.frame.gate)
            });
            frame_values(&task.frame, &mut roots);
        }
        for frame in self
            .pending_frame
            .iter()
            .chain(self.resuming_frames.iter().flatten())
        {
            frame_values(frame, &mut roots);
        }
        for adapter in &self.task_adapters {
            roots.extend([Value::Task(adapter.identity), Value::Task(adapter.source)]);
            match adapter.kind {
                super::task_adapters::AdapterKind::Admission => {}
                super::task_adapters::AdapterKind::Bridge(next) => roots.push(Value::Closure(next)),
            }
        }
        for identity in self
            .method_signatures
            .keys()
            .chain(self.wrapper_chains.keys())
        {
            if let Some(method) = self.runtime.registry().method_by_id(*identity) {
                let active = match method.owner() {
                    iris_runtime::MethodOwner::Class(class) => {
                        self.runtime.registry().active(class).is_ok_and(|revision| {
                            revision
                                .methods()
                                .values()
                                .chain(revision.singleton_methods().values())
                                .any(|id| id == identity)
                        }) || self
                            .runtime
                            .registry()
                            .staged_method(class, method.selector())
                            == Some(*identity)
                    }
                    iris_runtime::MethodOwner::Module(module) => {
                        self.runtime
                            .registry()
                            .module_method(module, method.selector())
                            .is_some_and(|active| active.id() == *identity)
                            || self.runtime.registry().staged_module(module).is_ok_and(
                                |candidate| {
                                    candidate
                                        .method(method.selector())
                                        .is_some_and(|active| active.id() == *identity)
                                },
                            )
                    }
                };
                if active {
                    roots.push(Value::Method(method));
                }
            }
        }
        roots.extend(
            self.qualified_methods
                .slots
                .values()
                .copied()
                .map(Value::Method),
        );
        let mut live = Visited::default();
        let (freed, _) = self
            .runtime
            .collect_with_edges(&roots, |value, edges| match value {
                Value::Closure(id) if live.closures.insert(*id) => {
                    if let Some(closure) = self.closures.get(id) {
                        edges.extend(closure.captures.iter().cloned());
                        edges.extend(closure.method_types.iter().map(|(_, value)| value.clone()));
                    }
                    if let Some(next) = self.next_continuations.get(id) {
                        next.visit_values(&mut |value| edges.push(value.clone()));
                        edges.extend(
                            next.chain
                                .wrappers
                                .iter()
                                .chain(next.chain.body_closure.iter())
                                .copied()
                                .map(Value::Closure),
                        );
                    }
                }
                Value::Method(method) if live.methods.insert(method.id()) => {
                    for closed in &self.closed_methods {
                        if closed.canonical == method.id() {
                            edges.push(Value::Method(closed.method));
                        }
                        if closed.method.id() == method.id() {
                            edges.extend(closed.types.iter().cloned());
                            edges.extend(closed.qualifier.iter().cloned());
                            edges.extend(
                                closed.owner_bindings.iter().map(|(_, value)| value.clone()),
                            );
                        }
                    }
                    if let Some(chain) = self.wrapper_chains.get(&method.id()) {
                        edges.extend(
                            chain
                                .wrappers
                                .iter()
                                .chain(chain.body_closure.iter())
                                .copied()
                                .map(Value::Closure),
                        );
                    }
                }
                Value::BoundMethod(method) => {
                    live.bound_methods.insert(method.id());
                }
                Value::Task(id) if live.tasks.insert(*id) => {
                    edges.extend(self.task_types.get(id).cloned());
                    if let Some(outcome) = self.tasks.get(id) {
                        match outcome {
                            Ok(value) => edges.push(value.clone()),
                            Err(error) => {
                                if let MachineError::Raised(raised) = error.as_ref() {
                                    edges.extend([raised.0.clone(), raised.1.clone()]);
                                }
                            }
                        }
                    }
                    edges.extend(self.task_owners.get(id).copied().map(Value::Closure));
                }
                Value::Gate(id) if live.gates.insert(*id) => {
                    edges.extend(self.gates.get(id).into_iter().flatten().cloned())
                }
                Value::ArrayIterator(id) | Value::HashIterator(id) | Value::ByteIterator(id)
                    if live.iterators.insert(*id) =>
                {
                    if let Some(iterator) = self.iterators.get(id) {
                        match &iterator.source {
                            Some(IteratorSource::Array { source, .. }) => {
                                edges.push(Value::Array(source.clone()))
                            }
                            Some(IteratorSource::Hash { source, keys, .. }) => {
                                edges.push(Value::Hash(source.clone()));
                                edges.extend(keys.iter().cloned());
                            }
                            Some(
                                IteratorSource::Values(values)
                                | IteratorSource::Text { values, .. },
                            ) => edges.extend(values.iter().cloned()),
                            None => {}
                        }
                    }
                }
                _ => {}
            });
        self.closures.retain(|id, _| live.closures.contains(id));
        self.next_continuations
            .retain(|id, _| live.closures.contains(id));
        self.wrapper_chains
            .retain(|id, _| live.methods.contains(id));
        self.closed_methods
            .retain(|closed| live.methods.contains(&closed.method.id()));
        self.bound_signatures
            .retain(|id, _| live.bound_methods.contains(id));
        self.method_signatures
            .retain(|id, _| live.methods.contains(id));
        self.dynamic_methods.retain(|id| live.methods.contains(id));
        self.qualified_methods
            .records
            .retain(|id, _| live.methods.contains(id));
        self.qualified_methods
            .definitions
            .retain(|id, _| live.methods.contains(id));
        self.tasks.retain(|id, _| live.tasks.contains(id));
        self.task_types.retain(|id, _| live.tasks.contains(id));
        self.task_owners.retain(|id, _| live.tasks.contains(id));
        self.observed_tasks.retain(|id| live.tasks.contains(id));
        self.gates.retain(|id, _| live.gates.contains(id));
        self.iterators.retain(|id, _| live.iterators.contains(id));
        freed
    }
}
