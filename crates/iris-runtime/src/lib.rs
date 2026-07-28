//! Iris runtime value foundation.

mod class_registry;
mod class_revision;
mod dispatch;
mod heap;
mod identity;
mod kernel;
mod meta;
mod method;
mod module_registry;
mod mro;
mod numeric;
mod protocol;
mod publication;
mod runtime;
mod stable_hash;
mod value;

pub use class_registry::{ClassError, ClassRegistry};
pub use class_revision::{
    CandidateRevision, ClassRevision, LogicalClass, MroEntry, StaticSpine, StoredProperty,
};
pub use dispatch::{DispatchContext, DispatchError, DispatchOutcome};
pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use kernel::{Kernel, KernelError, NativeSelector};
pub use meta::{BuiltinClass, Capability, MetaCapabilities};
pub use method::{BoundMethod, Method, MethodBody, MethodOwner, Visibility};
pub use numeric::{Numeric, NumericError, NumericValue};
pub use protocol::{
    ComparisonError, ComparisonProtocol, ComparisonSlot, Truthiness, TruthinessError,
    TruthinessMethod,
};
pub use runtime::{ConstructionError, ExecutionError, Runtime};
pub use stable_hash::{StableHashError, numeric_hash, numeric_public_hash, public_hash};
pub use value::{IntegerValue, Value};
