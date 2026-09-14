use std::collections::HashSet;

use super::{RawIvars, Reachable};
use crate::{BoundReceiver, HeapPayload, ObjectId, RuntimeHeap, Value};

pub(crate) fn trace<'a, 'b>(
    storage: (&RuntimeHeap, &RawIvars),
    roots: (
        impl IntoIterator<Item = &'a Value>,
        impl IntoIterator<Item = &'b Value>,
    ),
    edges: &mut dyn FnMut(&Value, &mut Vec<Value>),
) -> Reachable {
    let mut walker = Walker {
        heap: storage.0,
        ivars: storage.1,
        objects: HashSet::new(),
        cells: HashSet::new(),
        edges,
        pending: Vec::new(),
    };
    for root in roots.0 {
        walker.walk(root);
    }
    for root in roots.1 {
        walker.walk(root);
    }
    while let Some(value) = walker.pending.pop() {
        walker.walk(&value);
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
    edges: &'a mut dyn FnMut(&Value, &mut Vec<Value>),
    pending: Vec<Value>,
}

impl Walker<'_> {
    fn reach(&mut self, object: ObjectId) {
        if !self.objects.insert(object) {
            return;
        }
        if let Some(slots) = self.ivars.get(&object) {
            for slot in slots.values() {
                self.walk(slot);
            }
        }
        let Ok(found) = self.heap.lookup(object) else {
            return;
        };
        let HeapPayload::InstanceFields(fields) = found.payload();
        for field in fields {
            self.walk(field);
        }
    }

    fn walk(&mut self, value: &Value) {
        (self.edges)(value, &mut self.pending);
        match value {
            Value::Object(id)
            | Value::Task(id)
            | Value::Generator(id)
            | Value::ArrayIterator(id)
            | Value::HashIterator(id)
            | Value::ByteIterator(id)
            | Value::Gate(id)
            | Value::Closure(id) => self.reach(*id),
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
            Value::ReadonlyArray(values) | Value::Tuple(values) => {
                for element in values {
                    self.walk(element);
                }
            }
            Value::StackFrame(_, inner)
            | Value::RaiseSite(inner)
            | Value::IterationYield(inner)
            | Value::KeywordArgument(_, inner)
            | Value::BlockArgument(inner)
            | Value::ContractView(inner, _) => self.walk(inner),
            Value::ExceptionContext(id, value, cause, suppressed, sites, location) => {
                self.reach(*id);
                self.walk(value);
                self.walk(cause);
                for entry in suppressed.iter().chain(sites) {
                    self.walk(entry);
                }
                self.walk(&location.location);
                for frame in &location.original_stack {
                    self.walk(frame);
                }
            }
            Value::Decorator(record) => record.visit_values(&mut |value| self.walk(value)),
            Value::ImmutableArray(array) => {
                if self.cells.insert(array.cell_id()) {
                    array.visit_values(&mut |value| self.walk(value));
                }
            }
            Value::ImmutableHash(hash) => {
                if self.cells.insert(hash.cell_id()) {
                    hash.visit_values(&mut |value| self.walk(value));
                }
            }
            Value::BoundMethod(bound) => {
                self.reach(bound.id());
                self.walk(&Value::Method(bound.method()));
                match bound.receiver() {
                    BoundReceiver::Object(receiver) => self.walk(&Value::Object(receiver)),
                    BoundReceiver::Class(_) | BoundReceiver::Module(_) => {}
                }
            }
            Value::Library(library) => {
                for (_, signature) in &library.signatures {
                    self.walk(signature);
                }
            }
            Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::SourceLocation(..)
            | Value::Range(_)
            | Value::NativeResource(_)
            | Value::ExternalResource(_)
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
