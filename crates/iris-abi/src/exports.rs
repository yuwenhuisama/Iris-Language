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

/// Drains one posted completion, reporting its token.
///
/// `IRIS-V1-FFI-C037` makes a token authorize exactly ONE completion, so the
/// runtime thread has to learn WHICH token a drained value belongs to. Without
/// the token a caller can only see that some value arrived, which cannot
/// distinguish a completion matched to its own request from an unrelated one.
///
/// # Safety
/// Every out pointer must be valid and writable, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_drain_first_completion(
    out_token: *mut u64,
    out_value: *mut i64,
    out_count: *mut u32,
) -> IrisStatus {
    if out_token.is_null() || out_value.is_null() || out_count.is_null() {
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
            let first = posts.first();
            let token = first.map_or(0, |post| post.token);
            let value = first
                .and_then(|post| post.payload.get(..8))
                .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
                .map_or(0, i64::from_le_bytes);
            // SAFETY: all three checked non-null above.
            unsafe {
                out_token.write(token);
                out_value.write(value);
                out_count.write(u32::try_from(posts.len()).unwrap_or(u32::MAX));
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

/// Loads an extension, verifying its artifact before negotiating.
///
/// `IRIS-V1-FFI-C023` makes runtime native load verify the artifact against its
/// metadata, and `V009` aborts an EXTENSION load on a digest mismatch just as a
/// package load aborts. `iris_extension_attach` performs `C038` version
/// negotiation only, so this is the entry an extension load actually uses: a
/// mismatched artifact never reaches the table.
///
/// # Safety
/// `declared_digest` and `actual_digest` must be valid NUL-terminated C strings,
/// and `out_table` a valid writable record or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_extension_load(
    declared_digest: *const core::ffi::c_char,
    actual_digest: *const core::ffi::c_char,
    requested_major: u32,
    minimum_minor: u32,
    out_table: *mut IrisAbiTable,
) -> IrisStatus {
    if out_table.is_null() || declared_digest.is_null() || actual_digest.is_null() {
        return IrisStatus::InvalidArgument;
    }
    // SAFETY: both checked non-null; the caller supplies NUL-terminated bytes.
    let (declared, actual) = unsafe {
        (
            core::ffi::CStr::from_ptr(declared_digest),
            core::ffi::CStr::from_ptr(actual_digest),
        )
    };
    let (Ok(declared), Ok(actual)) = (declared.to_str(), actual.to_str()) else {
        return IrisStatus::InvalidArgument;
    };
    let manifest = crate::NativeManifest {
        package: Some("fixture.extension".into()),
        reflection_policy: Some("default".into()),
        digest: Some(declared.to_owned()),
        abi_major: crate::ABI_MAJOR,
        abi_minor: crate::ABI_MINOR,
    };
    match crate::load_extension(&manifest, actual, requested_major, minimum_minor) {
        Ok(table) => {
            // SAFETY: checked non-null above.
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

/// The registered payload for the fixture runtime.
///
/// `IRIS-V1-FFI-C027` gives the RUNTIME control of payload storage attached to
/// Iris objects, so the payload lives here rather than in extension memory. A
/// native extension only names it.
static PAYLOAD: std::sync::Mutex<Option<crate::NativePayload>> = std::sync::Mutex::new(None);

/// Registers a native payload descriptor.
///
/// `IRIS-V1-FFI-C027` requires validation BEFORE the runtime owns any payload
/// storage, and `C029` refuses a descriptor whose cleanup claims it may raise.
///
/// # Safety
/// `out_diagnostic` must be a valid writable pointer or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_payload_register(
    size: u32,
    alignment: u32,
    cleanup_may_raise: i32,
    external_resource: i32,
    out_diagnostic: *mut u32,
) -> IrisStatus {
    let descriptor = crate::PayloadDescriptor {
        size,
        alignment,
        trace: crate::TraceReport::default(),
        cleanup: if cleanup_may_raise == 0 {
            crate::CleanupPolicy::NoRaise
        } else {
            crate::CleanupPolicy::MayRaise
        },
        external_resource: external_resource != 0,
    };
    match crate::NativePayload::register(descriptor) {
        Ok(payload) => {
            let Ok(mut slot) = PAYLOAD.lock() else {
                return IrisStatus::InvalidRuntime;
            };
            *slot = Some(payload);
            if !out_diagnostic.is_null() {
                // SAFETY: checked non-null.
                unsafe { out_diagnostic.write(0) };
            }
            IrisStatus::Success
        }
        Err(rejection) => {
            if !out_diagnostic.is_null() {
                let code = match rejection {
                    crate::DescriptorRejection::AlignmentNotPowerOfTwo => 1,
                    crate::DescriptorRejection::SizeNotMultipleOfAlignment => 2,
                    crate::DescriptorRejection::CleanupMayRaise => 3,
                };
                // SAFETY: checked non-null.
                unsafe { out_diagnostic.write(code) };
            }
            IrisStatus::InvalidArgument
        }
    }
}

/// Declares one managed handle as a payload trace root.
///
/// `IRIS-V1-FFI-C028` limits payload trace logic to REPORTING managed handles
/// or roots, and forbids it from creating Iris values, calling Iris Methods,
/// raising, or dereferencing moved objects. So a payload DECLARES its roots and
/// the runtime reads them; there is no extension callback to run at trace time,
/// which is what makes those prohibitions unreachable rather than merely
/// forbidden.
#[unsafe(no_mangle)]
pub extern "C" fn iris_payload_add_root(handle: IrisHandle) -> IrisStatus {
    // C028 forbids depending on worker-thread heap access, and a root names a
    // managed value, so declaring one is an owning-thread operation.
    let affinity = owns_runtime();
    if affinity != IrisStatus::Success {
        return affinity;
    }
    // A root must name a live managed value; a stale or foreign handle is
    // refused here rather than being reported to the collector.
    let valid = RUNTIME.with_borrow(|table| table.get(handle).map(|_| ()));
    if let Err(status) = valid {
        return status;
    }
    let Ok(mut slot) = PAYLOAD.lock() else {
        return IrisStatus::InvalidRuntime;
    };
    slot.as_mut().map_or(IrisStatus::InvalidHandle, |payload| {
        payload.add_root(handle);
        IrisStatus::Success
    })
}

/// How many roots the payload reports, and whether they are still live.
///
/// `IRIS-V1-FFI-C028` makes trace keep reachable targets alive, so the count of
/// roots that still resolve is what shows the referenced managed values
/// survived rather than merely having been listed.
///
/// # Safety
/// Both out pointers must be valid and writable, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_payload_trace(
    out_reported: *mut u32,
    out_live: *mut u32,
) -> IrisStatus {
    if out_reported.is_null() || out_live.is_null() {
        return IrisStatus::InvalidArgument;
    }
    let affinity = owns_runtime();
    if affinity != IrisStatus::Success {
        return affinity;
    }
    let Ok(slot) = PAYLOAD.lock() else {
        return IrisStatus::InvalidRuntime;
    };
    let Some(payload) = slot.as_ref() else {
        return IrisStatus::InvalidHandle;
    };
    let roots = payload.trace();
    let live = RUNTIME.with_borrow(|table| {
        roots
            .iter()
            .filter(|handle| table.get(**handle).is_ok())
            .count()
    });
    // SAFETY: both checked non-null above.
    unsafe {
        out_reported.write(u32::try_from(roots.len()).unwrap_or(u32::MAX));
        out_live.write(u32::try_from(live).unwrap_or(u32::MAX));
    }
    IrisStatus::Success
}

/// Closes the registered payload's external resource.
///
/// `IRIS-V1-FFI-C030` makes deterministic release explicit and IDEMPOTENT, so
/// a second call succeeds without releasing again.
#[unsafe(no_mangle)]
pub extern "C" fn iris_payload_close() -> IrisStatus {
    let Ok(mut slot) = PAYLOAD.lock() else {
        return IrisStatus::InvalidRuntime;
    };
    slot.as_mut()
        .map_or(IrisStatus::InvalidHandle, crate::NativePayload::close)
}

/// Runs final cleanup on the registered payload.
///
/// `IRIS-V1-FFI-C029` runs this in a GC-safe context where raising into Iris is
/// prohibited, so it answers a status only.
#[unsafe(no_mangle)]
pub extern "C" fn iris_payload_final_cleanup() -> IrisStatus {
    let Ok(mut slot) = PAYLOAD.lock() else {
        return IrisStatus::InvalidRuntime;
    };
    slot.as_mut().map_or(
        IrisStatus::InvalidHandle,
        crate::NativePayload::final_cleanup,
    )
}

/// How many times the payload's external resource was actually released.
#[unsafe(no_mangle)]
pub extern "C" fn iris_payload_release_count() -> u32 {
    PAYLOAD
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(crate::NativePayload::releases))
        .unwrap_or(u32::MAX)
}

/// Resets the fixture runtime between scenarios.
#[unsafe(no_mangle)]
pub extern "C" fn iris_runtime_reset() {
    RUNTIME.with_borrow_mut(|table| *table = HandleTable::new(1));
    if let Ok(mut slot) = PAYLOAD.lock() {
        *slot = None;
    }
    iris_bridge_reset();
}
