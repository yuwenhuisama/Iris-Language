//! Calls into the conformance C fixtures.
//!
//! These declarations name functions COMPILED BY A C COMPILER from
//! `conformance/iris-v1/fixtures/ffi`. Calling them exercises the real C ABI
//! rather than a Rust-shaped stand-in, which is what `IRIS-V1-FFI-V060`
//! requires and what would expose a mismatch in the handle or status layout.

unsafe extern "C" {
    /// `IRIS-V1-FFI-V060` fixture.
    pub safe fn fixture_host_abi_v1(out_major: *mut u32, out_minor: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V908` fixture: a C++ wrapper reports its C ABI version.
    ///
    /// Compiled as C++ rather than C, so `IRIS-V1-FFI-C053` is exercised
    /// rather than asserted: the wrapper's vtable, destructor and templates
    /// stay inside the translation unit and only these C entries cross.
    pub safe fn fixture_cpp_wrapper_reports_abi(out_major: *mut u32, out_minor: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V908` fixture: the wrapper fails closed on an
    /// unsupported major and retains nothing from the rejected negotiation.
    pub safe fn fixture_cpp_wrapper_fails_closed(
        out_requested_major: *mut u32,
        out_retained: *mut i32,
    ) -> i32;
    /// `IRIS-V1-FFI-V911` fixture: only the negotiated C ABI is claimed stable.
    pub safe fn fixture_cpp_wrapper_claims_only_c(
        out_major: *mut u32,
        out_crosses_cpp: *mut i32,
    ) -> i32;
    /// `IRIS-V1-FFI-V061` fixture.
    pub safe fn fixture_rooted_handle(out_before: *mut i64, out_after: *mut i32) -> i32;
    /// `IRIS-V1-FFI-V062` fixture: a worker reads through a handle.
    pub safe fn fixture_worker_reads_handle(handle: crate::IrisHandle, out_value: *mut i64) -> i32;
    /// `IRIS-V1-FFI-V062` fixture: a worker posts copied data.
    pub safe fn fixture_worker_posts(token: u64, value: i64) -> i32;
    /// `IRIS-V1-FFI-V063` fixture.
    pub safe fn fixture_raise_marker(
        out_context: *mut crate::IrisHandle,
        out_marker: *mut i64,
    ) -> i32;
    /// `IRIS-V1-FFI-V066` fixture.
    pub safe fn fixture_post_twice(
        token: u64,
        out_second: *mut i32,
        out_count: *mut u32,
        out_value: *mut i64,
    ) -> i32;
    /// `IRIS-V1-FFI-V067` fixture.
    pub safe fn fixture_negotiate_v2(out_major: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V008` fixture: a panic beneath the boundary.
    pub safe fn fixture_panic_does_not_cross(out_value: *mut i64, out_resumed: *mut i32) -> i32;
    /// `IRIS-V1-FFI-V012`/`V065` fixture: a Closeable payload closed twice.
    pub safe fn fixture_payload_close_twice(out_second: *mut i32, out_releases: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V009` fixture: an extension artifact digest mismatch.
    pub safe fn fixture_extension_digest_mismatch(out_major: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V009` fixture: the same load with a matching digest.
    pub safe fn fixture_extension_digest_matches(out_major: *mut u32) -> i32;
    /// `IRIS-V1-FFI-V011` fixture: a payload declares a managed trace root.
    pub safe fn fixture_payload_traces_root(
        out_value: *mut i64,
        out_reported: *mut u32,
        out_live: *mut u32,
    ) -> i32;
    /// `IRIS-V1-FFI-V011` fixture: a stale root is refused at declaration.
    pub safe fn fixture_payload_rejects_stale_root() -> i32;
    /// `IRIS-V1-FFI-V013` fixture: a descriptor whose cleanup may raise.
    pub safe fn fixture_payload_cleanup_may_raise(
        out_diagnostic: *mut u32,
        out_releases: *mut u32,
    ) -> i32;
    /// `IRIS-V1-FFI-V005` fixture: a worker posts a copied completion.
    pub safe fn fixture_worker_completes(token: u64, value: i64) -> i32;
    /// `IRIS-V1-FFI-V005` fixture: the runtime thread takes the completion.
    pub safe fn fixture_runtime_takes_completion(
        out_token: *mut u64,
        out_value: *mut i64,
        out_count: *mut u32,
    ) -> i32;
    /// `IRIS-V1-FFI-V064` fixture: the entry a refused load must never call.
    pub safe fn fixture_static_api_call_count() -> i32;
    /// `IRIS-V1-FFI-V059` fixture: the entry a manifestless load must not bind.
    pub safe fn fixture_manifestless_call_count() -> i32;
    /// `IRIS-V1-FFI-V001` fixture: an address forged into a handle.
    pub safe fn fixture_raw_pointer_handle(
        out_value: *mut i64,
        out_touched: *mut i32,
        out_tagged_status: *mut i32,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::{
        fixture_extension_digest_matches, fixture_extension_digest_mismatch, fixture_host_abi_v1,
        fixture_negotiate_v2, fixture_panic_does_not_cross, fixture_payload_cleanup_may_raise,
        fixture_payload_close_twice, fixture_payload_rejects_stale_root,
        fixture_payload_traces_root, fixture_post_twice, fixture_raise_marker,
        fixture_raw_pointer_handle, fixture_rooted_handle, fixture_runtime_takes_completion,
        fixture_worker_completes, fixture_worker_posts, fixture_worker_reads_handle,
    };
    use crate::{IrisStatus, iris_runtime_reset};

    /// Serializes every scenario that resets or uses the shared runtime.
    ///
    /// `IRIS-V1-FFI-C014` makes ONE post queue the only cross-thread entry, and
    /// resetting a runtime clears it. So a reset in ANY parallel test discards
    /// another scenario's pending post, not just a competing queue test. Test
    /// threads run in parallel, which is why this is a lock rather than more
    /// resetting: clearing cannot stop a post landing between clear and drain.
    static QUEUE_SCENARIO: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Takes the scenario lock, ignoring poisoning from an unrelated failure.
    fn scenario() -> std::sync::MutexGuard<'static, ()> {
        QUEUE_SCENARIO
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn v060_c_code_attaches_and_reads_the_negotiated_versions() {
        // Given a fixture compiled as C
        let mut major = 0_u32;
        let mut minor = 0_u32;

        // When it attaches through the Host table
        let status = fixture_host_abi_v1(&raw mut major, &raw mut minor);

        // Then the negotiated versions reach C unchanged, which only holds if
        // the record layout really is the C one.
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(major, crate::ABI_MAJOR);
        assert_eq!(minor, crate::ABI_MINOR);
    }

    #[test]
    fn v062_a_worker_read_is_refused_while_its_copied_post_is_accepted() {
        let _scenario = scenario();
        // Given a handle created on the runtime thread
        crate::iris_runtime_reset();
        crate::iris_bridge_reset();
        let mut handle = crate::IrisHandle::NULL;
        // SAFETY:  is a live local, so the out pointer is valid.
        unsafe { crate::iris_int_create(41, &raw mut handle) };

        // When a worker reads through it and then posts copied data
        let observed = std::thread::scope(|scope| {
            scope
                .spawn(move || {
                    let mut value = 0_i64;
                    let read = fixture_worker_reads_handle(handle, &raw mut value);
                    (read, value, fixture_worker_posts(7, 7))
                })
                .join()
        });
        let Ok((read, value, posted)) = observed else {
            unreachable!("the worker returns its statuses")
        };

        // Then C012 refuses the read with no value written, while C013 accepts
        // the copied post.
        assert_eq!(read, IrisStatus::ThreadAffinity as i32);
        assert_eq!(value, 0);
        assert_eq!(posted, IrisStatus::Success as i32);
    }

    #[test]
    fn v063_a_native_raise_answers_a_status_and_a_context() {
        let _scenario = scenario();
        // Given
        crate::iris_runtime_reset();
        let mut context = crate::IrisHandle::NULL;
        let mut marker = 0_i64;

        // When C raises through the Host ABI
        let status = fixture_raise_marker(&raw mut context, &raw mut marker);

        // Then C017 answers a status AND fills the context, so the raised value
        // stays an ordinary Iris value rather than a string in the status.
        assert_eq!(status, IrisStatus::Raised as i32);
        assert!(!context.is_null());
        assert_eq!(marker, 41);
    }

    #[test]
    fn v066_one_token_authorizes_exactly_one_completion() {
        let _scenario = scenario();
        // Given
        crate::iris_bridge_reset();
        let mut second = 0_i32;
        let mut count = 0_u32;
        let mut value = 0_i64;

        // When C posts the same token twice
        let status = fixture_post_twice(99, &raw mut second, &raw mut count, &raw mut value);

        // Then C037 makes the first stand and refuses the second.
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(second, IrisStatus::DuplicateCompletion as i32);
        assert_eq!(count, 1);
        assert_eq!(value, 9);
    }

    #[test]
    fn v067_an_abi_major_mismatch_publishes_no_table() {
        // Given
        let mut major = 0_u32;

        // When C requests major 2 from a major 1 runtime
        let status = fixture_negotiate_v2(&raw mut major);

        // Then C039 rejects outright and the record stays untouched.
        assert_eq!(status, IrisStatus::IncompatibleAbi as i32);
        assert_eq!(major, 0);
    }

    #[test]
    fn v061_a_released_handle_reports_invalid_handle_to_c() {
        let _scenario = scenario();
        // Given
        iris_runtime_reset();
        let mut before = 0_i64;
        let mut after = 0_i32;

        // When C creates, reads, releases, then reads again
        let status = fixture_rooted_handle(&raw mut before, &raw mut after);

        // Then the first read answers 41 and the second is refused, so the
        // generation check survives the crossing rather than living only in
        // Rust.
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(before, 41);
        assert_eq!(after, IrisStatus::InvalidHandle as i32);
    }

    #[test]
    fn v008_a_panic_beneath_the_boundary_does_not_reach_c() {
        let _scenario = scenario();
        // Given a C caller of a body that unwinds
        iris_runtime_reset();
        let mut value = 0_i64;
        let mut resumed = 0_i32;

        // When the panic happens beneath the C ABI frame
        let status = fixture_panic_does_not_cross(&raw mut value, &raw mut resumed);

        // Then C observes an ordinary status and keeps running. `resumed` is
        // the real evidence: had the unwind crossed, the C frame would never
        // have reached the line that sets it.
        assert_eq!(status, IrisStatus::InvalidBoundary as i32);
        assert_eq!(resumed, 1);
        assert_eq!(value, -1);
    }

    #[test]
    fn v001_a_forged_pointer_handle_is_refused_without_dereference() {
        let _scenario = scenario();
        // Given a fixture that reinterprets a real address as a handle
        iris_runtime_reset();
        let mut value = 0_i64;
        let mut touched = 0_i32;
        let mut tagged = 0_i32;

        // When both forgeries cross the boundary
        let status = fixture_raw_pointer_handle(&raw mut value, &raw mut touched, &raw mut tagged);

        // Then both are refused. `touched` is the load-bearing assertion: the
        // pointee is 41, so observing 41 through either path would mean the
        // runtime followed the address instead of treating the handle as an
        // opaque table name.
        assert_eq!(status, IrisStatus::InvalidRuntime as i32);
        assert_ne!(tagged, IrisStatus::Success as i32);
        assert_eq!(touched, 0);
        assert_eq!(value, 0);
    }

    #[test]
    fn v005_a_worker_posts_a_completion_the_runtime_thread_takes() {
        let _scenario = scenario();
        // Given a runtime thread that owns the queue
        crate::iris_runtime_reset();
        let token = 5150_u64;

        // When a WORKER thread posts copied data under that token
        let posted = std::thread::scope(|scope| {
            scope
                .spawn(|| fixture_worker_completes(token, 23))
                .join()
                .unwrap_or(-1)
        });

        // Then the RUNTIME thread is what converts it into a completion, and
        // the token comes back so the value is matched to its own request
        // rather than merely observed to have arrived.
        let mut seen_token = 0_u64;
        let mut value = 0_i64;
        let mut count = 0_u32;
        let status =
            fixture_runtime_takes_completion(&raw mut seen_token, &raw mut value, &raw mut count);

        assert_eq!(posted, IrisStatus::Success as i32);
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(seen_token, token);
        assert_eq!(value, 23);
        assert_eq!(count, 1);
    }

    #[test]
    fn v012_a_closeable_native_resource_closes_twice() {
        let _scenario = scenario();
        // Given a runtime-owned payload registered from C
        crate::iris_runtime_reset();
        let mut second = 0_i32;
        let mut releases = 0_u32;

        // When C closes it twice and final cleanup then runs
        let first = fixture_payload_close_twice(&raw mut second, &raw mut releases);

        // Then C030 makes both calls succeed while the resource is released
        // EXACTLY once. Without the count, a double release would look
        // identical to an idempotent close.
        assert_eq!(first, IrisStatus::Success as i32);
        assert_eq!(second, IrisStatus::Success as i32);
        assert_eq!(releases, 1);
    }

    #[test]
    fn v013_a_descriptor_whose_cleanup_may_raise_never_registers() {
        let _scenario = scenario();
        // Given a descriptor claiming final cleanup may raise into Iris
        crate::iris_runtime_reset();
        let mut diagnostic = 0_u32;
        let mut releases = 0_u32;

        // When C tries to register it
        let status = fixture_payload_cleanup_may_raise(&raw mut diagnostic, &raw mut releases);

        // Then C029 refuses it at REGISTRATION rather than accepting it and
        // containing a raise at drop time, so no storage is ever owned.
        assert_eq!(status, IrisStatus::InvalidArgument as i32);
        assert_eq!(diagnostic, 3);
        assert_eq!(releases, u32::MAX);
    }

    #[test]
    fn v011_a_payload_traces_managed_roots_and_keeps_them_alive() {
        let _scenario = scenario();
        // Given a payload declaring one managed handle as a trace root
        crate::iris_runtime_reset();
        let mut value = 0_i64;
        let mut reported = 0_u32;
        let mut live = 0_u32;

        // When the runtime reads the roots back
        let status = fixture_payload_traces_root(&raw mut value, &raw mut reported, &raw mut live);

        // Then the root is reported AND still resolves. The live count is the
        // load-bearing part: listing a root that had died would satisfy
        // `reported` alone.
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(reported, 1);
        assert_eq!(live, 1);
        assert_eq!(value, 41);
    }

    #[test]
    fn c028_a_stale_handle_is_refused_as_a_trace_root() {
        let _scenario = scenario();
        // Given a handle released after the payload registered
        crate::iris_runtime_reset();

        // When it is declared as a root
        let status = fixture_payload_rejects_stale_root();

        // Then it is refused at DECLARATION rather than being handed to the
        // collector, which is what keeps trace from reporting a dead target.
        assert_eq!(status, IrisStatus::InvalidHandle as i32);
    }

    #[test]
    fn v009_an_extension_artifact_digest_mismatch_aborts_the_load() {
        // Given an extension whose artifact does not hash to its metadata
        let mut major = 0_u32;

        // When it loads
        let status = fixture_extension_digest_mismatch(&raw mut major);

        // Then C023 aborts before binding and no table is published. The
        // sentinel surviving is the evidence: a negotiated table would mean an
        // unverified extension had already received authority.
        assert_eq!(status, IrisStatus::IncompatibleAbi as i32);
        assert_eq!(major, 99);
    }

    #[test]
    fn c023_a_matching_extension_artifact_attaches() {
        // The same path with a matching digest must attach, which is what
        // shows the refusal comes from verification rather than from the
        // extension path being broken outright.
        let mut major = 0_u32;
        let status = fixture_extension_digest_matches(&raw mut major);
        assert_eq!(status, IrisStatus::Success as i32);
        assert_eq!(major, crate::ABI_MAJOR);
    }
}
