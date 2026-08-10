//! Calls into the conformance C fixtures.
//!
//! These declarations name functions COMPILED BY A C COMPILER from
//! `conformance/iris-v1/fixtures/ffi`. Calling them exercises the real C ABI
//! rather than a Rust-shaped stand-in, which is what `IRIS-V1-FFI-V060`
//! requires and what would expose a mismatch in the handle or status layout.

unsafe extern "C" {
    /// `IRIS-V1-FFI-V060` fixture.
    pub safe fn fixture_host_abi_v1(out_major: *mut u32, out_minor: *mut u32) -> i32;
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
        fixture_host_abi_v1, fixture_negotiate_v2, fixture_panic_does_not_cross,
        fixture_post_twice, fixture_raise_marker, fixture_raw_pointer_handle,
        fixture_rooted_handle, fixture_worker_posts, fixture_worker_reads_handle,
    };
    use crate::{IrisStatus, iris_runtime_reset};

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
}
