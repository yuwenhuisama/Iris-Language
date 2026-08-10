//! The stable Iris v1 C ABI.
//!
//! `IRIS-V1-FFI-C003` makes the C ABI the binary compatibility identity. Every
//! item that crosses this boundary is `extern "C"` with a `#[repr(C)]` layout,
//! and no Rust type, trait, vtable or allocator is part of the contract.
//! `IRIS-V1-FFI-C041` lets a wrapper add RAII, ownership checks and typed
//! helpers on top; those belong to the wrapper, never to this surface.

mod barrier;
mod handle;
mod manifest;
mod negotiate;
mod status;
mod table;
mod thread;

pub use barrier::{IrisCallResult, guard};
pub use handle::{IrisFrame, IrisHandle, IrisHandleKind};
pub use manifest::{ManifestRejection, NativeManifest, verify};
pub use negotiate::{ABI_MAJOR, ABI_MINOR, IrisAbiTable, attach};
pub use status::IrisStatus;
pub use table::HandleTable;
pub use thread::{Post, PostQueue, ThreadAffinity};
