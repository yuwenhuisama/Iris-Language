//! Iris runtime value foundation.

mod class_registry;
mod class_revision;
mod dispatch;
mod heap;
mod identity;
mod method;
mod module_registry;
mod mro;
mod numeric;
mod publication;
mod runtime;
mod value;

pub use class_registry::{ClassError, ClassRegistry};
pub use class_revision::{
    CandidateRevision, ClassRevision, LogicalClass, MroEntry, StaticSpine, StoredProperty,
};
pub use dispatch::{DispatchError, DispatchOutcome};
pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use method::{BoundMethod, Method, MethodBody, MethodOwner, Visibility};
pub use numeric::{Numeric, NumericError, NumericValue};
pub use runtime::{ConstructionError, ExecutionError, Runtime};
pub use value::{IntegerValue, Value};
