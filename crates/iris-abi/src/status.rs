//! `IRIS-V1-FFI-C017` status codes.

/// The result of one C ABI operation.
///
/// `IRIS-V1-FFI-C017` requires every fallible operation to answer a status and
/// place ordinary results, raised values and `ExceptionContext` objects in
/// explicit out parameters. A status alone is not enough when the operation
/// raised, which is why the call surface carries a separate context out handle
/// rather than encoding failure detail in this enum.
///
/// `IRIS-V1-FFI-C042` makes changing the meaning of an existing status
/// MAJOR-breaking, so these discriminants are fixed for ABI major 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum IrisStatus {
    /// The operation completed and any out parameters are populated.
    Success = 0,
    /// The handle was released, never issued, or belongs to another runtime.
    ///
    /// `IRIS-V1-FFI-C009` makes handle reuse after release possible, so a stale
    /// handle MUST be detected rather than silently denoting a new target.
    InvalidHandle = 1,
    /// The runtime was destroyed or the handle names a different runtime.
    InvalidRuntime = 2,
    /// The call touched Iris state from a thread that does not own the runtime.
    ///
    /// `IRIS-V1-FFI-C012` requires this rather than racing the managed heap.
    ThreadAffinity = 3,
    /// The operation raised an Iris value; the context out handle carries it.
    ///
    /// `IRIS-V1-FFI-C018` keeps the raised value an ordinary Iris value while
    /// the stack, cause and bridge records live on the `ExceptionContext`.
    Raised = 4,
    /// A completion token was used more than once.
    ///
    /// `IRIS-V1-FFI-C037` makes the first completion stand.
    DuplicateCompletion = 5,
    /// ABI negotiation failed on major version, feature bits or record size.
    ///
    /// `IRIS-V1-FFI-C039` makes a major mismatch reject attachment outright.
    IncompatibleAbi = 6,
    /// An argument was null, mis-sized, or outside its documented domain.
    InvalidArgument = 7,
    /// The operation is not permitted at this boundary.
    ///
    /// `IRIS-V1-FFI-C007` forbids a raw managed pointer from crossing, and
    /// `IRIS-V1-FFI-C005` forbids a script reaching Host ABI tables directly.
    InvalidBoundary = 8,
}
