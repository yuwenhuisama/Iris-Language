//! Iris runtime value foundation.

mod class_registry;
mod class_revision;
mod decorator;
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

pub use class_registry::{ClassError, ClassRegistry, PolicyOrigin};
pub use class_revision::{
    CandidateRevision, ClassRevision, CompositionEdge, LogicalClass, MroEntry, StaticSpine,
    StoredProperty,
};
pub use decorator::{AppliedDecorator, DecoratorTransform, DecoratorViolation};
pub use dispatch::{DispatchContext, DispatchError, DispatchOutcome};
pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, ContractId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use kernel::{Kernel, KernelError, NativeSelector};
pub use meta::{BuiltinClass, Capability, MetaCapabilities};
pub use method::{BoundMethod, BoundReceiver, Method, MethodBody, MethodOwner, Visibility};
pub use numeric::{Numeric, NumericError, NumericValue};
pub use protocol::{
    ComparisonError, ComparisonProtocol, ComparisonSlot, Truthiness, TruthinessError,
    TruthinessMethod,
};
pub use runtime::{ConstructionError, ExecutionError, Runtime};
pub use stable_hash::{
    StableHashError, artifact_digest, bytes_hash, contract_type_hash, contract_view_hash,
    iteration_hash, numeric_hash, numeric_public_hash, public_hash, range_hash, regex_hash,
    string_hash, symbol_hash, tuple_hash,
};
pub use value::{
    ArrayBody, ArrayRef, ByteArrayBody, ByteArrayRef, ComposedType, HashBody, HashRef,
    IntegerValue, MutableStringRef, TypeAtom, Value,
};
