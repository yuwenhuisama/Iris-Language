use crate::{NativeError, frame::Frame};
use iris_native_sdk::*;
use iris_runtime::Value;
use std::ffi::c_void;
mod blob;
pub(crate) static HOST: HostV1 = HostV1 {
    header: HeaderV1::new::<HostV1>(),
    scalar_read,
    scalar_create,
    blob_info: blob::info,
    blob_read: blob::read,
    blob_create: blob::create,
    resource_create,
    resource_read,
    error_raise,
};

unsafe fn boundary(
    context: *mut c_void,
    action: impl FnOnce(&Frame<'_>) -> Result<(), NativeError>,
) -> i32 {
    if context.is_null() || !context.cast::<Frame<'_>>().is_aligned() {
        return INVALID_ARGUMENT;
    }
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: the ABI call-scoped context is a live Frame, only shared references
        // are created; RefCell owns mutation and no borrow spans another callback.
        let frame = unsafe { &*context.cast::<Frame<'_>>() };
        match action(frame) {
            Ok(()) => SUCCESS,
            Err(NativeError::Protocol(status)) => status,
            Err(NativeError::Closed) => INVALID_HANDLE,
            Err(NativeError::Quota) => INVALID_BOUNDARY,
            Err(_) => INVALID_ARGUMENT,
        }
    }))
    .unwrap_or(INVALID_BOUNDARY)
}
unsafe fn output<T>(pointer: *mut T, value: T) -> Result<(), NativeError> {
    if pointer.is_null() || !pointer.is_aligned() {
        return Err(NativeError::Protocol(INVALID_ARGUMENT));
    }
    // SAFETY: callback caller supplies writable, aligned, disjoint output storage.
    unsafe { pointer.write(value) };
    Ok(())
}
unsafe fn bytes<'a>(slice: SliceV1) -> Result<&'a [u8], NativeError> {
    if slice.length == 0 {
        return Ok(&[]);
    }
    if slice.data.is_null() || slice.length > isize::MAX as usize {
        return Err(NativeError::Type);
    }
    // SAFETY: ABI caller guarantees this nonempty buffer is readable throughout
    // the callback; it is copied/consumed before the callback returns.
    Ok(unsafe { std::slice::from_raw_parts(slice.data, slice.length) })
}
unsafe extern "C" fn scalar_read(
    context: *mut c_void,
    handle: Handle,
    result: *mut ScalarV1,
) -> i32 {
    // SAFETY: this callback forwards the ABI context and output obligations.
    unsafe {
        boundary(context, |frame| {
            let value = match frame.value(handle)? {
                Value::Nil => ScalarV1 {
                    kind: NIL,
                    integer: 0,
                },
                Value::Bool(value) => ScalarV1 {
                    kind: BOOL,
                    integer: i64::from(value),
                },
                Value::Integer(value) => ScalarV1 {
                    kind: INTEGER,
                    integer: value
                        .to_i128()
                        .and_then(|value| i64::try_from(value).ok())
                        .ok_or(NativeError::Type)?,
                },
                _ => return Err(NativeError::Type),
            };
            output(result, value)
        })
    }
}
unsafe extern "C" fn scalar_create(
    context: *mut c_void,
    scalar: ScalarV1,
    result: *mut Handle,
) -> i32 {
    // SAFETY: this callback forwards the ABI context and output obligations.
    unsafe {
        boundary(context, |frame| {
            let value = match (scalar.kind, scalar.integer) {
                (NIL, 0) => Value::Nil,
                (BOOL, 0) => Value::Bool(false),
                (BOOL, 1) => Value::Bool(true),
                (INTEGER, integer) => {
                    Value::Integer(integer.to_string().parse().map_err(|_| NativeError::Type)?)
                }
                _ => return Err(NativeError::Type),
            };
            output(result, frame.insert(value)?)
        })
    }
}
unsafe extern "C" fn resource_create(
    context: *mut c_void,
    index: usize,
    cookie: u64,
    result: *mut Handle,
) -> i32 {
    // SAFETY: this callback forwards the ABI context and output obligations.
    unsafe {
        boundary(context, |frame| {
            output(result, frame.insert(frame.create_resource(index, cookie)?)?)
        })
    }
}
unsafe extern "C" fn resource_read(
    context: *mut c_void,
    handle: Handle,
    index: usize,
    result: *mut u64,
) -> i32 {
    // SAFETY: this callback forwards the ABI context and output obligations.
    unsafe {
        boundary(context, |frame| {
            output(result, frame.read_resource(&frame.value(handle)?, index)?)
        })
    }
}
unsafe extern "C" fn error_raise(
    context: *mut c_void,
    code: SliceV1,
    message: SliceV1,
    os_code: i64,
    result: *mut Handle,
) -> i32 {
    // SAFETY: this callback forwards the ABI context, readable slice and disjoint output obligations.
    let status = unsafe {
        boundary(context, |frame| {
            let code = std::str::from_utf8(bytes(code)?)
                .map_err(|_| NativeError::Type)?
                .to_owned();
            let message = std::str::from_utf8(bytes(message)?)
                .map_err(|_| NativeError::Type)?
                .to_owned();
            output(
                result,
                frame.raise(NativeError::Raised {
                    code,
                    message,
                    os_code,
                })?,
            )
        })
    };
    if status == SUCCESS { RAISED } else { status }
}
