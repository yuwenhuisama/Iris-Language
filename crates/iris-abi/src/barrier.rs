//! `IRIS-V1-FFI-C019` unwinding barrier and `C017` result records.

use crate::{IrisHandle, IrisStatus};

/// The result of one call that may raise.
///
/// `IRIS-V1-FFI-C017` makes a status alone insufficient when the operation
/// raised, so an ordinary result and a captured `ExceptionContext` travel in
/// EXPLICIT out handles rather than being encoded in the status. A string-only
/// error channel is not conforming, which is why neither field is a message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct IrisCallResult {
    /// Byte size of the record the producer filled in.
    pub size: u32,
    /// The ordinary answer, or `NULL` when the call raised.
    pub value: IrisHandle,
    /// The captured `ExceptionContext`, or `NULL` when it did not raise.
    ///
    /// `IRIS-V1-FFI-C018` keeps the raised value an ordinary Iris value while
    /// stack, cause and native bridge records belong to this context.
    pub context: IrisHandle,
}

impl IrisCallResult {
    /// An empty result record.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            size: u32::try_from(core::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            value: IrisHandle::NULL,
            context: IrisHandle::NULL,
        }
    }
}

/// Runs `body` so no Rust unwinding reaches C.
///
/// `IRIS-V1-FFI-C019` forbids a Rust panic, C++ exception, SEH exception, host
/// unwinding or long jump from crossing the C ABI, and requires the wrapper to
/// catch it BEFORE it reaches C. A caught panic becomes a status, because
/// returning normally is the only way a C caller can observe the failure.
///
/// The clause also permits aborting when a condition prevents safe translation.
/// A caught panic IS safely translatable, so this converts rather than aborts;
/// an abort would discard a failure the caller could have handled.
pub fn guard<T>(fallback: T, body: impl FnOnce() -> T + std::panic::UnwindSafe) -> (IrisStatus, T) {
    match std::panic::catch_unwind(body) {
        Ok(value) => (IrisStatus::Success, value),
        Err(_) => (IrisStatus::InvalidBoundary, fallback),
    }
}

#[cfg(test)]
mod tests {
    use super::{IrisCallResult, guard};
    use crate::{IrisHandle, IrisStatus};

    #[test]
    fn c019_a_panic_becomes_a_status_instead_of_crossing_the_boundary() {
        // Given a body that unwinds
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        // When a body unwinds. An out-of-bounds index is an ordinary panic
        // and avoids the explicit `panic!` the workspace lint forbids.
        let (status, value) = guard(-1_i64, || {
            let empty: [i64; 0] = [];
            let index = std::hint::black_box(0_usize);
            empty[index]
        });

        // Then
        std::panic::set_hook(previous);
        assert_eq!(status, IrisStatus::InvalidBoundary);
        assert_eq!(value, -1);
    }

    #[test]
    fn c019_an_ordinary_body_passes_its_value_through() {
        // When
        let (status, value) = guard(-1_i64, || 41);

        // Then
        assert_eq!(status, IrisStatus::Success);
        assert_eq!(value, 41);
    }

    #[test]
    fn c017_an_empty_result_names_neither_a_value_nor_a_context() {
        // When
        let result = IrisCallResult::empty();

        // Then
        assert_eq!(result.value, IrisHandle::NULL);
        assert_eq!(result.context, IrisHandle::NULL);
        assert!(result.size >= 8);
    }
}
