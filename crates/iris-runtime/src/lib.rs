//! Iris runtime value foundation.

mod heap;
mod identity;
mod value;

pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use value::{IntegerValue, Value};
