//! Engine-neutral v1.35 records and checks, not Iris source bindings or execution.
//!
//! Backends own phase extents, signature substitution, initial call preparation,
//! exact Type predicates, candidate admission, and Task/activation lifecycle.
//! Value visitors expose roots for `reachable_from`; they do not duplicate its
//! cycle-aware walk through mutable argument objects.
//!
//! Source phase execution is pending. The reference engine still assembles its
//! historical class-targeted factory results through `Transformation::from_operations`;
//! it does not yet enforce v1.35 phase construction, planning, or wrapper execution.

mod changes;
mod errors;
mod fields;
mod invocation;
mod kinds;
mod payload;
mod payload_validation;
mod phase;
mod scope;
mod signature;
mod source;
mod transformation;
mod value;

pub use changes::{ArgumentChanges, ChangeField};
pub use errors::{
    ArgumentError, ConstructionError, DecoratorProtocolError, KindError, ProtocolCategory,
};
pub use fields::SnapshotTypes;
pub use invocation::{Invocation, InvocationSlot, SelectedCall, SlotKind};
pub use kinds::{DecoratorKind, DecoratorReason, ParameterCategory};
pub use payload::InvocationPayload;
pub use phase::{DecoratorContext, DecoratorPhase, Plan};
pub use scope::{ActivationId, NextScope, ScopeOwner, TaskId};
pub use signature::{InvocationParameter, InvocationSignature};
pub use source::SourceCall;
pub use transformation::{Operation, Transformation};
pub use value::DecoratorValue;
