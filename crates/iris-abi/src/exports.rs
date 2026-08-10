//! The `extern "C"` surface a native extension links against.
//!
//! `IRIS-V1-FFI-C003` makes this the binary compatibility identity, so every
//! function here is `extern "C"`, takes and returns only `#[repr(C)]` types,
//! and answers a status per `IRIS-V1-FFI-C017`. No Rust type, trait, vtable,
//! allocator or panic may appear in a signature.

use crate::{HandleTable, IrisAbiTable, IrisHandle, IrisHandleKind, IrisStatus};
use std::cell::RefCell;

thread_local! {
    /// The runtime this thread owns, for the fixtures to drive.
    ///
    /// A production embedding passes an `IrisRuntime*`; the conformance
    /// fixtures drive one implicit runtime, which keeps the C signatures the
    /// same shape while avoiding a second lifetime concern in the harness.
    static RUNTIME: RefCell<HandleTable<i64>> = const { RefCell::new(HandleTable::new(1)) };
}

/// Negotiates an extension attachment.
///
/// `IRIS-V1-FFI-C038` requires every participant to declare its required major
/// and minimum minor BEFORE receiving authority to create or observe Iris
/// values, so this is the first call an extension makes.
///
/// # Safety
/// `out_table` must be a valid, writable `IrisAbiTable` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_extension_attach(
    requested_major: u32,
    minimum_minor: u32,
    out_table: *mut IrisAbiTable,
) -> IrisStatus {
    if out_table.is_null() {
        return IrisStatus::InvalidArgument;
    }
    match crate::attach(requested_major, minimum_minor) {
        Ok(table) => {
            // SAFETY: checked non-null above; the caller owns writable storage.
            unsafe { out_table.write(table) };
            IrisStatus::Success
        }
        Err(status) => status,
    }
}

/// Creates an Iris Integer and roots it as a handle.
///
/// `IRIS-V1-FFI-C007` makes the answer an opaque handle rather than an address.
///
/// # Safety
/// `out_handle` must be a valid, writable `IrisHandle` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_int_create(value: i64, out_handle: *mut IrisHandle) -> IrisStatus {
    if out_handle.is_null() {
        return IrisStatus::InvalidArgument;
    }
    let handle =
        RUNTIME.with_borrow_mut(|table| table.retain(value, IrisHandleKind::ExplicitRelease));
    // SAFETY: checked non-null above.
    unsafe { out_handle.write(handle) };
    IrisStatus::Success
}

/// Reads an Integer through a handle.
///
/// `IRIS-V1-FFI-C017` places the result in an out parameter and answers a
/// status, so a released handle reports `invalid-handle` rather than reading
/// whatever now occupies the slot.
///
/// # Safety
/// `out_value` must be a valid, writable `i64` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_handle_get_int(
    handle: IrisHandle,
    out_value: *mut i64,
) -> IrisStatus {
    if out_value.is_null() {
        return IrisStatus::InvalidArgument;
    }
    RUNTIME.with_borrow(|table| match table.get(handle) {
        Ok(value) => {
            // SAFETY: checked non-null above.
            unsafe { out_value.write(*value) };
            IrisStatus::Success
        }
        Err(status) => status,
    })
}

/// Releases a handle's root.
///
/// `IRIS-V1-FFI-C008` ends the strong root here, and `C009` makes every handle
/// already issued for the slot detectably stale afterwards.
#[unsafe(no_mangle)]
pub extern "C" fn iris_handle_release(handle: IrisHandle) -> IrisStatus {
    RUNTIME.with_borrow_mut(|table| table.release(handle))
}

/// Resets the fixture runtime between scenarios.
#[unsafe(no_mangle)]
pub extern "C" fn iris_runtime_reset() {
    RUNTIME.with_borrow_mut(|table| *table = HandleTable::new(1));
}
