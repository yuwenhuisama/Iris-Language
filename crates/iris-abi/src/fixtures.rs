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
}

#[cfg(test)]
mod tests {
    use super::{fixture_host_abi_v1, fixture_rooted_handle};
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
}
