use std::sync::Arc;

use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ImmutableArray(Arc<ArrayStorage>);

#[derive(Debug, PartialEq)]
struct ArrayStorage {
    elements: Vec<Value>,
    element_type: Value,
}

impl ImmutableArray {
    pub fn new(elements: Vec<Value>, element_type: Value) -> Self {
        Self(Arc::new(ArrayStorage {
            elements,
            element_type,
        }))
    }
    pub fn elements(&self) -> &[Value] {
        &self.0.elements
    }
    pub fn element_type(&self) -> &Value {
        &self.0.element_type
    }
    pub fn len(&self) -> usize {
        self.0.elements.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.elements.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.elements.get(index)
    }
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn cell_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        visitor(self.element_type());
        for element in self.elements() {
            visitor(element);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImmutableHash(Arc<HashStorage>);

#[derive(Debug, PartialEq)]
struct HashStorage {
    entries: Vec<(Value, Value)>,
    key_type: Value,
    value_type: Value,
}

impl ImmutableHash {
    pub fn new(entries: Vec<(Value, Value)>, key_type: Value, value_type: Value) -> Self {
        Self(Arc::new(HashStorage {
            entries,
            key_type,
            value_type,
        }))
    }
    pub fn entries(&self) -> &[(Value, Value)] {
        &self.0.entries
    }
    pub fn key_type(&self) -> &Value {
        &self.0.key_type
    }
    pub fn value_type(&self) -> &Value {
        &self.0.value_type
    }
    pub fn len(&self) -> usize {
        self.0.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.entries.is_empty()
    }
    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.entries()
            .iter()
            .find(|(held, _)| held == key)
            .map(|(_, value)| value)
    }
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn cell_id(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        visitor(self.key_type());
        visitor(self.value_type());
        for (key, value) in self.entries() {
            visitor(key);
            visitor(value);
        }
    }
}
