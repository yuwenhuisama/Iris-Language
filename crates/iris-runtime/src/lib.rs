//! Iris runtime value foundation.

mod callable_type;
mod candidate_mutation;
mod class_candidate;
mod class_registry;
mod class_revision;
mod core_registration;
mod decorator;
pub mod decorator_protocol;
mod dispatch;
mod exception;
mod external_resource;
mod heap;
mod identity;
mod immutable_collection;
mod kernel;
mod meta;
mod method;
mod module_candidate;
mod module_mutation;
mod module_overlay;
mod module_registry;
mod module_revision;
mod mro;
mod numeric;
mod origin;
mod protocol;
mod publication;
mod runtime;
mod stable_hash;
mod structural_error;
mod structural_publication;
mod trace;
mod value;

pub use callable_type::{
    CallableKind, CallableSignature, CallableType, CallableTypeError, SignatureType,
};
pub use class_candidate::CandidateRevision;
pub use class_registry::{ClassError, ClassRegistry, PolicyOrigin};
pub use class_revision::{
    ClassRevision, CompositionEdge, LogicalClass, MroEntry, StaticSpine, StoredProperty,
};
pub use core_registration::{CoreClass, core_contract_id};
pub use decorator::{AppliedDecorator, DecoratorTransform, DecoratorViolation};
pub use decorator_protocol::DecoratorValue;
pub use dispatch::{DispatchContext, DispatchError, DispatchOutcome};
pub use exception::{ExceptionOrigin, NativeBridge};
pub use external_resource::ExternalResource;
pub use heap::{HeapObject, HeapPayload, RuntimeError, RuntimeHeap};
pub use identity::{ClassId, ContractId, MethodId, ModuleId, ObjectId, RevisionId, Selector};
pub use immutable_collection::{ImmutableArray, ImmutableHash};
pub use kernel::{Kernel, KernelError, NativeSelector};
pub use meta::{BuiltinClass, Capability, MetaCapabilities};
pub use method::{BoundMethod, BoundReceiver, Method, MethodBody, MethodOwner, Visibility};
pub use module_revision::{
    CandidateModule, ModuleMethodDefinition, ModuleRevision, ModuleRevisionId,
};
pub use numeric::{Numeric, NumericError, NumericValue};
pub use origin::StagedOrigin;
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
pub use structural_error::{ModuleCandidateError, RuntimeStructuralError};
pub use structural_publication::StructuralCommit;
pub use trace::{Reachable, reachable_from};
pub use value::{
    ArrayBody, ArrayRef, ByteArrayBody, ByteArrayRef, ComposedType, HashBody, HashRef,
    IntegerValue, LibraryValue, MatchValue, MutableStringRef, NominalType, RangeValue, RegexValue,
    TypeAtom, Value,
};
