//! Iris runtime value foundation.

mod class_registry;
mod class_revision;
mod heap;
mod identity;
mod numeric;
mod value;

pub use class_registry::{ClassError, ClassRegistry};
pub use class_revision::{CandidateRevision, ClassRevision, LogicalClass, StaticSpine};
pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use numeric::{Numeric, NumericError, NumericValue};
pub use value::{IntegerValue, Value};
