use iris_native_sdk::*;
use std::{
    ffi::c_void,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
static CLOSES: AtomicU64 = AtomicU64::new(0);
static DESTROYS: AtomicU64 = AtomicU64::new(0);
include!(concat!(env!("OUT_DIR"), "/metadata_digest.rs"));

unsafe extern "C" fn echo(
    host: *const HostV1,
    call: *mut c_void,
    args: *const Handle,
    count: usize,
    result: *mut CallResultV1,
) -> i32 {
    if count != 1 {
        return INVALID_ARGUMENT;
    }
    // SAFETY: host provides one argument, a live table and disjoint writable outputs.
    unsafe {
        let mut scalar = ScalarV1::default();
        if ((*host).scalar_read)(call, *args, &mut scalar) == SUCCESS {
            return ((*host).scalar_create)(call, scalar, &mut (*result).value);
        }
        let mut info = BlobInfoV1::default();
        let status = ((*host).blob_info)(call, *args, &mut info);
        if status != SUCCESS {
            return status;
        }
        let mut bytes = vec![0; info.length];
        let status = ((*host).blob_read)(call, *args, bytes.as_mut_ptr(), bytes.len());
        if status != SUCCESS {
            return status;
        }
        ((*host).blob_create)(
            call,
            info.kind,
            SliceV1::from_bytes(&bytes),
            &mut (*result).value,
        )
    }
}
unsafe extern "C" fn fail(
    host: *const HostV1,
    call: *mut c_void,
    _args: *const Handle,
    _count: usize,
    result: *mut CallResultV1,
) -> i32 {
    // SAFETY: host table and result stay valid through the callback; static slices are readable.
    unsafe {
        ((*host).error_raise)(
            call,
            SliceV1::from_bytes(b"NativeTestError"),
            SliceV1::from_bytes(b"test failure"),
            42,
            &mut (*result).context,
        )
    }
}
unsafe extern "C" fn create(
    host: *const HostV1,
    call: *mut c_void,
    _args: *const Handle,
    _count: usize,
    result: *mut CallResultV1,
) -> i32 {
    let cookie = NEXT.fetch_add(1, Ordering::Relaxed);
    // SAFETY: resource zero is declared, result is valid and cookie is unique.
    unsafe { ((*host).resource_create)(call, 0, cookie, &mut (*result).value) }
}
unsafe extern "C" fn close(
    host: *const HostV1,
    call: *mut c_void,
    _cookie: u64,
    result: *mut CallResultV1,
) -> i32 {
    CLOSES.fetch_add(1, Ordering::Relaxed);
    // SAFETY: call-scoped table and result obey the host callback contract.
    unsafe {
        ((*host).scalar_create)(
            call,
            ScalarV1 {
                kind: NIL,
                integer: 0,
            },
            &mut (*result).value,
        )
    }
}
unsafe extern "C" fn destroy(_cookie: u64) {
    DESTROYS.fetch_add(1, Ordering::Relaxed);
}
#[unsafe(no_mangle)]
pub extern "C" fn iris_native_test_destroy_count() -> u64 {
    DESTROYS.load(Ordering::Relaxed)
}
unsafe extern "C" fn closes(
    host: *const HostV1,
    call: *mut c_void,
    _args: *const Handle,
    _count: usize,
    result: *mut CallResultV1,
) -> i32 {
    let integer = i64::try_from(CLOSES.load(Ordering::Relaxed)).unwrap_or(i64::MAX);
    // SAFETY: call-scoped table and result obey the host callback contract.
    unsafe {
        ((*host).scalar_create)(
            call,
            ScalarV1 {
                kind: INTEGER,
                integer,
            },
            &mut (*result).value,
        )
    }
}
unsafe extern "C" fn available(
    host: *const HostV1,
    call: *mut c_void,
    args: *const Handle,
    count: usize,
    result: *mut CallResultV1,
) -> i32 {
    if count != 1 {
        return INVALID_ARGUMENT;
    }
    let mut cookie = 0;
    // SAFETY: the host supplies one live argument, the output is disjoint, and
    // resource_read validates module ownership without exposing the cookie to Iris.
    unsafe {
        let status = ((*host).resource_read)(call, *args, 0, &mut cookie);
        if status != SUCCESS {
            return ((*host).error_raise)(
                call,
                SliceV1::from_bytes(b"ClosedResourceError"),
                SliceV1::from_bytes(b"closed"),
                0,
                &mut (*result).context,
            );
        }
        ((*host).scalar_create)(
            call,
            ScalarV1 {
                kind: BOOL,
                integer: 1,
            },
            &mut (*result).value,
        )
    }
}
unsafe extern "C" fn function_at(index: usize, output: *mut Option<FunctionV1>) -> i32 {
    let function: FunctionV1 = match index {
        0..=4 => echo,
        5 => fail,
        6 => create,
        7 => closes,
        8 => available,
        _ => return INVALID_ARGUMENT,
    };
    // SAFETY: host provides aligned writable function pointer storage.
    unsafe {
        output.write(Some(function));
    }
    SUCCESS
}
unsafe extern "C" fn resource_at(index: usize, output: *mut ResourceV1) -> i32 {
    if index != 0 {
        return INVALID_ARGUMENT;
    }
    // SAFETY: host provides aligned writable resource descriptor storage.
    unsafe {
        output.write(ResourceV1 {
            size: size_of::<ResourceV1>(),
            close: Some(close),
            destroy: Some(destroy),
        });
    }
    SUCCESS
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iris_native_module_v1(output: *mut ModuleV1) -> i32 {
    // SAFETY: host provides initialized descriptor capacity and writable output.
    unsafe {
        if output.is_null() || (*output).header.size < size_of::<ModuleV1>() {
            return INCOMPATIBLE_ABI;
        }
        output.write(ModuleV1 {
            header: HeaderV1::new::<ModuleV1>(),
            function_count: 9,
            resource_count: 1,
            function_at: Some(function_at),
            resource_at: Some(resource_at),
            metadata_sha256: METADATA_SHA256,
        });
    }
    SUCCESS
}
