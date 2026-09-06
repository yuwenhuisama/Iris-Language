use super::*;
pub(super) unsafe extern "C" fn info(
    context: *mut c_void,
    handle: Handle,
    result: *mut BlobInfoV1,
) -> i32 {
    // SAFETY: forwards the ABI context and writable output contract.
    unsafe {
        boundary(context, |frame| {
            let info = match frame.value(handle)? {
                Value::Text(text) => BlobInfoV1 {
                    kind: STRING,
                    length: text.len(),
                },
                Value::Bytes(bytes) => BlobInfoV1 {
                    kind: BYTES,
                    length: bytes.len(),
                },
                _ => return Err(NativeError::Type),
            };
            output(result, info)
        })
    }
}
pub(super) unsafe extern "C" fn read(
    context: *mut c_void,
    handle: Handle,
    buffer: *mut u8,
    capacity: usize,
) -> i32 {
    // SAFETY: forwards the ABI context and disjoint writable buffer contract.
    unsafe {
        boundary(context, |frame| {
            let value = frame.value(handle)?;
            let bytes = match &value {
                Value::Text(text) => text.as_bytes(),
                Value::Bytes(bytes) => bytes,
                _ => return Err(NativeError::Type),
            };
            if bytes.len() > capacity || (!bytes.is_empty() && buffer.is_null()) {
                return Err(NativeError::Type);
            }
            if !bytes.is_empty() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
            }
            Ok(())
        })
    }
}
pub(super) unsafe extern "C" fn create(
    context: *mut c_void,
    kind: u32,
    input: SliceV1,
    result: *mut Handle,
) -> i32 {
    // SAFETY: forwards the ABI context, readable input and disjoint output contract.
    unsafe {
        boundary(context, |frame| {
            let bytes = bytes(input)?;
            let value = match kind {
                STRING => Value::Text(
                    std::str::from_utf8(bytes)
                        .map_err(|_| NativeError::Type)?
                        .to_owned(),
                ),
                BYTES => Value::Bytes(bytes.to_vec()),
                _ => return Err(NativeError::Type),
            };
            output(result, frame.insert(value)?)
        })
    }
}
