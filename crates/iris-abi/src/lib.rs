//! The stable Iris v1 C ABI.
//!
//! `IRIS-V1-FFI-C003` makes the C ABI the binary compatibility identity. Every
//! item that crosses this boundary is `extern "C"` with a `#[repr(C)]` layout,
//! and no Rust type, trait, vtable or allocator is part of the contract.
//! `IRIS-V1-FFI-C041` lets a wrapper add RAII, ownership checks and typed
//! helpers on top; those belong to the wrapper, never to this surface.

mod barrier;
mod exports;
pub mod fixtures;
mod handle;
mod manifest;
mod negotiate;
mod status;
mod table;
mod thread;

pub use barrier::{IrisCallResult, guard};
pub use exports::{
    iris_bridge_reset, iris_call_panicking, iris_drain_completions, iris_extension_attach,
    iris_handle_get_int, iris_handle_release, iris_int_create, iris_post_completion,
    iris_raise_marker, iris_runtime_reset,
};
pub use fixtures::{
    fixture_host_abi_v1, fixture_negotiate_v2, fixture_panic_does_not_cross, fixture_post_twice,
    fixture_raise_marker, fixture_raw_pointer_handle, fixture_rooted_handle, fixture_worker_posts,
    fixture_worker_reads_handle,
};
pub use handle::{IrisFrame, IrisHandle, IrisHandleKind};
pub use manifest::{ManifestRejection, NativeManifest, verify};
pub use negotiate::{ABI_MAJOR, ABI_MINOR, IrisAbiTable, attach};
pub use status::IrisStatus;
pub use table::HandleTable;
pub use thread::{Post, PostQueue, ThreadAffinity};
