//! Standalone C-layout native module ABI. See `include/iris_native_v1.h`.
use core::ffi::c_void;

pub type Handle = u64;
pub const SUCCESS: i32 = 0;
pub const INVALID_HANDLE: i32 = 1;
pub const INVALID_RUNTIME: i32 = 2;
pub const THREAD_AFFINITY: i32 = 3;
pub const RAISED: i32 = 4;
pub const DUPLICATE_COMPLETION: i32 = 5;
pub const INCOMPATIBLE_ABI: i32 = 6;
pub const INVALID_ARGUMENT: i32 = 7;
pub const INVALID_BOUNDARY: i32 = 8;
pub const NIL: u32 = 0;
pub const BOOL: u32 = 1;
pub const INTEGER: u32 = 2;
pub const STRING: u32 = 3;
pub const BYTES: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HeaderV1 {
    pub major: u32,
    pub minor: u32,
    pub size: usize,
    pub features: u64,
}
impl HeaderV1 {
    pub const fn new<T>() -> Self {
        Self {
            major: 1,
            minor: 0,
            size: size_of::<T>(),
            features: 0,
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SliceV1 {
    pub data: *const u8,
    pub length: usize,
}
impl SliceV1 {
    pub const fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            length: bytes.len(),
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ScalarV1 {
    pub kind: u32,
    pub integer: i64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BlobInfoV1 {
    pub kind: u32,
    pub length: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CallResultV1 {
    pub size: usize,
    pub value: Handle,
    pub context: Handle,
}
impl Default for CallResultV1 {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            value: 0,
            context: 0,
        }
    }
}
pub type FunctionV1 = unsafe extern "C" fn(
    *const HostV1,
    *mut c_void,
    *const Handle,
    usize,
    *mut CallResultV1,
) -> i32;
pub type CloseV1 = unsafe extern "C" fn(*const HostV1, *mut c_void, u64, *mut CallResultV1) -> i32;
pub type DestroyV1 = unsafe extern "C" fn(u64);
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ResourceV1 {
    pub size: usize,
    pub close: Option<CloseV1>,
    pub destroy: Option<DestroyV1>,
}
impl Default for ResourceV1 {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            close: None,
            destroy: None,
        }
    }
}
#[repr(C)]
pub struct HostV1 {
    pub header: HeaderV1,
    pub scalar_read: unsafe extern "C" fn(*mut c_void, Handle, *mut ScalarV1) -> i32,
    pub scalar_create: unsafe extern "C" fn(*mut c_void, ScalarV1, *mut Handle) -> i32,
    pub blob_info: unsafe extern "C" fn(*mut c_void, Handle, *mut BlobInfoV1) -> i32,
    pub blob_read: unsafe extern "C" fn(*mut c_void, Handle, *mut u8, usize) -> i32,
    pub blob_create: unsafe extern "C" fn(*mut c_void, u32, SliceV1, *mut Handle) -> i32,
    pub resource_create: unsafe extern "C" fn(*mut c_void, usize, u64, *mut Handle) -> i32,
    pub resource_read: unsafe extern "C" fn(*mut c_void, Handle, usize, *mut u64) -> i32,
    pub error_raise: unsafe extern "C" fn(*mut c_void, SliceV1, SliceV1, i64, *mut Handle) -> i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModuleV1 {
    pub header: HeaderV1,
    pub function_count: usize,
    pub resource_count: usize,
    pub function_at: Option<unsafe extern "C" fn(usize, *mut Option<FunctionV1>) -> i32>,
    pub resource_at: Option<unsafe extern "C" fn(usize, *mut ResourceV1) -> i32>,
    pub metadata_sha256: [u8; 32],
}
impl Default for ModuleV1 {
    fn default() -> Self {
        Self {
            header: HeaderV1::new::<Self>(),
            function_count: 0,
            resource_count: 0,
            function_at: None,
            resource_at: None,
            metadata_sha256: [0; 32],
        }
    }
}
pub type EntryV1 = unsafe extern "C" fn(*mut ModuleV1) -> i32;
