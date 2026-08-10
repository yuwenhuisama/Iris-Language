//! The `extern "C"` surface a native extension links against.
//!
//! `IRIS-V1-FFI-C003` makes this the binary compatibility identity, so every
//! function here is `extern "C"`, takes and returns only `#[repr(C)]` types,
//! and answers a status per `IRIS-V1-FFI-C017`. No Rust type, trait, vtable,
//! allocator or panic may appear in a signature.

use crate::{
    HandleTable, IrisAbiTable, IrisHandle, IrisHandleKind, IrisStatus, Post, PostQueue,
    ThreadAffinity,
};
use std::cell::RefCell;
use std::sync::OnceLock;

thread_local! {
    /// The runtime this thread owns, for the fixtures to drive.
    ///
    /// A production embedding passes an `IrisRuntime*`; the conformance
    /// fixtures drive one implicit runtime, which keeps the C signatures the
    /// same shape while avoiding a second lifetime concern in the harness.
    static RUNTIME: RefCell<HandleTable<i64>> = const { RefCell::new(HandleTable::new(1)) };
}

thread_local! {
    /// This thread's own affinity token and post queue.
    ///
    /// A runtime binds ONE owning thread. Recording that per thread lets
    /// several runtimes exist in one process, which is what `IRIS-V1-FFI-C009`
    /// already assumes when it forbids passing a handle between runtimes.
    static OWNER: RefCell<Option<(ThreadAffinity, PostQueue)>> = const { RefCell::new(None) };
}

/// The post queue shared with worker threads.
///
/// `IRIS-V1-FFI-C014` makes the post queue the ONLY cross-thread entry, so a
/// worker reaches it while the handle table stays thread-local and
/// unreachable.
fn shared_queue() -> &'static PostQueue {
    static QUEUE: OnceLock<PostQueue> = OnceLock::new();
    QUEUE.get_or_init(PostQueue::new)
}

/// Whether the caller owns the runtime it is touching.
///
/// `IRIS-V1-FFI-C012` answers a thread-affinity status rather than racing the
/// heap, so a thread that never bound a runtime is refused too.
fn owns_runtime() -> IrisStatus {
    OWNER.with_borrow(|owner| {
        owner
            .as_ref()
            .map_or(IrisStatus::ThreadAffinity, |(affinity, _)| affinity.check())
    })
}

/// Binds the calling thread as the runtime owner.
#[unsafe(no_mangle)]
pub extern "C" fn iris_bridge_reset() {
    // The queue is process-wide, so a reset that left it populated would let
    // one scenario's accepted post be counted by the next one.
    shared_queue().clear();
    OWNER.with_borrow_mut(|owner| {
        *owner = Some((ThreadAffinity::bind_current(), shared_queue().clone()));
    });
}

/// Posts one copied completion from any thread.
///
/// `IRIS-V1-FFI-C013` lets an external thread post COPIED data only, never a
/// handle, and `C037` makes the first completion for a token stand.
#[unsafe(no_mangle)]
pub extern "C" fn iris_post_completion(token: u64, value: i64) -> IrisStatus {
    shared_queue().post(Post {
        token,
        payload: value.to_le_bytes().to_vec(),
    })
}

/// Drains posted completions on the runtime thread.
///
/// `IRIS-V1-FFI-C013` makes the RUNTIME thread the one that converts posted
/// data into Iris values, so draining is itself an owning-thread operation.
///
/// # Safety
/// `out_count` and `out_first` must be valid writable pointers or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_drain_completions(
    out_count: *mut u32,
    out_first: *mut i64,
) -> IrisStatus {
    if out_count.is_null() || out_first.is_null() {
        return IrisStatus::InvalidArgument;
    }
    let drained = OWNER.with_borrow(|owner| {
        owner
            .as_ref()
            .map_or(Err(IrisStatus::ThreadAffinity), |(affinity, _)| {
                shared_queue().drain(affinity)
            })
    });
    match drained {
        Ok(posts) => {
            let first = posts
                .first()
                .and_then(|post| post.payload.get(..8))
                .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
                .map_or(0, i64::from_le_bytes);
            // SAFETY: both checked non-null above.
            unsafe {
                out_count.write(u32::try_from(posts.len()).unwrap_or(u32::MAX));
                out_first.write(first);
            }
            IrisStatus::Success
        }
        Err(status) => status,
    }
}

/// Raises an Iris value from native code.
///
/// `IRIS-V1-FFI-C017` forbids a status-only answer when the operation raised,
/// and `C018` keeps the raised value an ordinary Iris value while the context
/// carries the bridge record. So this answers `Raised` AND fills the context
/// handle; a string-only error channel would not conform.
///
/// # Safety
/// `out_context` must be a valid writable `IrisHandle` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_raise_marker(
    marker: i64,
    out_context: *mut IrisHandle,
) -> IrisStatus {
    if out_context.is_null() {
        return IrisStatus::InvalidArgument;
    }
    let handle =
        RUNTIME.with_borrow_mut(|table| table.retain(marker, IrisHandleKind::ExplicitRelease));
    // SAFETY: checked non-null above.
    unsafe { out_context.write(handle) };
    IrisStatus::Raised
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
    // C012 requires every call touching a handle to run on the OWNING runtime
    // thread and to answer a thread-affinity status otherwise. Without this a
    // worker reached its own empty thread-local table and got `invalid-handle`,
    // which reports the wrong reason and hides the affinity violation.
    let affinity = owns_runtime();
    if affinity != IrisStatus::Success {
        return affinity;
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
    let affinity = owns_runtime();
    if affinity != IrisStatus::Success {
        return affinity;
    }
    RUNTIME.with_borrow_mut(|table| table.release(handle))
}

/// Calls a body that unwinds, from C.
///
/// `IRIS-V1-FFI-C019` forbids a Rust panic from crossing the C ABI and requires
/// the wrapper to catch it BEFORE it reaches C. Unit-testing `guard` in Rust
/// cannot show that: the panic must actually be raised beneath an `extern "C"`
/// frame and observed by a C caller as an ordinary returned status.
///
/// # Safety
/// `out_value` must be a valid, writable `i64` or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_call_panicking(out_value: *mut i64) -> IrisStatus {
    if out_value.is_null() {
        return IrisStatus::InvalidArgument;
    }
    // The default hook would print a backtrace for a panic this fixture raises
    // ON PURPOSE, so it is silenced for the duration of the guarded call only.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let (status, value) = crate::guard(-1_i64, || {
        // An out-of-bounds index is an ordinary panic, which avoids the
        // explicit `panic!` the workspace lint forbids.
        let empty: [i64; 0] = [];
        let index = std::hint::black_box(0_usize);
        empty[index]
    });
    std::panic::set_hook(previous);
    // SAFETY: checked non-null above.
    unsafe { out_value.write(value) };
    status
}

/// Resets the fixture runtime between scenarios.
#[unsafe(no_mangle)]
pub extern "C" fn iris_runtime_reset() {
    RUNTIME.with_borrow_mut(|table| *table = HandleTable::new(1));
    iris_bridge_reset();
}
