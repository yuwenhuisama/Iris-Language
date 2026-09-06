use crate::{frame::Frame, registry::Loaded, *};
use iris_native_sdk::*;
use iris_runtime::Value;
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

mod frame_protocol;

fn loaded() -> Rc<Loaded> {
    Rc::new(Loaded {
        metadata: serde_json::from_str(include_str!("../tests/fixture/module.json")).unwrap(),
        functions: Vec::new(),
        resources: vec![(
            "NativeTest::Socket".into(),
            ResourceV1 {
                size: size_of::<ResourceV1>(),
                close: Some(close),
                destroy: Some(destroy),
            },
        )],
        quota: 1,
        _library: None,
    })
}
static DESTROYS: AtomicUsize = AtomicUsize::new(0);
static CLOSES: AtomicUsize = AtomicUsize::new(0);
unsafe extern "C" fn close(
    host: *const HostV1,
    context: *mut std::ffi::c_void,
    _cookie: u64,
    result: *mut CallResultV1,
) -> i32 {
    CLOSES.fetch_add(1, Ordering::SeqCst);
    // SAFETY: invoked by the host with a live frame and writable disjoint result.
    unsafe { ((*host).scalar_create)(context, ScalarV1::default(), &mut (*result).value) }
}
unsafe extern "C" fn destroy(cookie: u64) {
    if cookie == 7 {
        DESTROYS.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn copying_callbacks_roundtrip_scalars_and_blobs() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    for value in [
        Value::Nil,
        Value::Bool(true),
        Value::Integer("-9223372036854775808".parse().unwrap()),
        Value::Text("a\0é".into()),
        Value::Bytes(vec![0, 255]),
    ] {
        let handle = frame.insert(value.clone()).unwrap();
        let mut result = 0;
        // SAFETY: frame is live, output pointers are disjoint initialized stack
        // allocations, and copied buffers remain live throughout each callback.
        unsafe {
            let host = &crate::callbacks::HOST;
            match &value {
                Value::Text(_) | Value::Bytes(_) => {
                    let mut info = BlobInfoV1::default();
                    assert_eq!(
                        (host.blob_info)(frame.context(), handle, &mut info),
                        SUCCESS
                    );
                    let mut buffer = vec![0; info.length];
                    assert_eq!(
                        (host.blob_read)(
                            frame.context(),
                            handle,
                            buffer.as_mut_ptr(),
                            buffer.len()
                        ),
                        SUCCESS
                    );
                    assert_eq!(
                        (host.blob_create)(
                            frame.context(),
                            info.kind,
                            SliceV1::from_bytes(&buffer),
                            &mut result
                        ),
                        SUCCESS
                    );
                    buffer.fill(42);
                }
                _ => {
                    let mut scalar = ScalarV1::default();
                    assert_eq!(
                        (host.scalar_read)(frame.context(), handle, &mut scalar),
                        SUCCESS
                    );
                    assert_eq!(
                        (host.scalar_create)(frame.context(), scalar, &mut result),
                        SUCCESS
                    );
                }
            }
        }
        assert_eq!(frame.value(result).unwrap(), value);
    }
}

#[test]
fn handles_from_an_ended_call_are_never_accepted_in_a_new_call() {
    let registry = NativeRegistry::new();
    let first = Frame::new(&registry, loaded());
    let stale = first.insert(Value::Nil).unwrap();
    drop(first);
    let second = Frame::new(&registry, loaded());
    let live = second.insert(Value::Bool(true)).unwrap();
    assert_ne!(stale, live);
    assert!(matches!(
        second.value(stale),
        Err(NativeError::Protocol(INVALID_HANDLE))
    ));
}

#[test]
fn resource_aliases_close_once_and_destroy_on_registry_teardown() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    let Value::ExternalResource(resource) = frame.create_resource(0, 7).unwrap() else {
        unreachable!()
    };
    let alias = resource.clone();
    assert_eq!(
        frame
            .read_resource(&Value::ExternalResource(alias.clone()), 0)
            .unwrap(),
        7
    );
    assert!(matches!(
        frame.create_resource(0, 8),
        Err(NativeError::Quota)
    ));
    assert_eq!(registry.close(&resource).unwrap(), Value::Nil);
    assert_eq!(registry.close(&alias).unwrap(), Value::Nil);
    assert!(matches!(
        frame.read_resource(&Value::ExternalResource(alias), 0),
        Err(NativeError::Closed)
    ));
    assert_eq!(CLOSES.load(Ordering::SeqCst), 1);
    drop(frame);
    drop(registry);
    assert_eq!(DESTROYS.load(Ordering::SeqCst), 1);
    assert!(!resource.is_open());
}

#[test]
fn resource_callbacks_enforce_cookie_type_ownership_and_quota() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    let mut handle = 0;
    let mut cookie = 0;
    // SAFETY: the frame and disjoint initialized outputs live across all callbacks.
    unsafe {
        let host = &crate::callbacks::HOST;
        assert_eq!(
            (host.resource_create)(frame.context(), 0, 17, &mut handle),
            SUCCESS
        );
        assert_eq!(
            (host.resource_read)(frame.context(), handle, 0, &mut cookie),
            SUCCESS
        );
        assert_eq!(cookie, 17);
        assert_eq!(
            (host.resource_read)(frame.context(), handle, 1, &mut cookie),
            INVALID_ARGUMENT
        );
        let other = Frame::new(&registry, loaded());
        let foreign = other.insert(frame.value(handle).unwrap()).unwrap();
        assert_eq!(
            (host.resource_read)(other.context(), foreign, 0, &mut cookie),
            INVALID_ARGUMENT
        );
        assert_eq!(
            (host.resource_create)(frame.context(), 0, 18, &mut handle),
            INVALID_BOUNDARY
        );
    }
}

#[test]
fn invalid_native_status_and_malformed_outputs_are_protocol_errors() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    assert!(matches!(
        frame.finish(12345, CallResultV1::default()),
        Err(NativeError::Protocol(12345))
    ));
    assert!(matches!(
        frame.finish(SUCCESS, CallResultV1::default()),
        Err(NativeError::Protocol(INVALID_HANDLE))
    ));
    assert!(matches!(
        frame.finish(RAISED, CallResultV1::default()),
        Err(NativeError::Protocol(RAISED))
    ));
    let mut handle = 0;
    // SAFETY: live frame and disjoint writable output; the invalid payloads are
    // ordinary integers and bytes, not invalid Rust enum or bool representations.
    unsafe {
        let host = &crate::callbacks::HOST;
        assert_eq!(
            (host.scalar_create)(
                frame.context(),
                ScalarV1 {
                    kind: BOOL,
                    integer: 2
                },
                &mut handle
            ),
            INVALID_ARGUMENT
        );
        assert_eq!(
            (host.blob_create)(
                frame.context(),
                STRING,
                SliceV1::from_bytes(&[255]),
                &mut handle
            ),
            INVALID_ARGUMENT
        );
        assert_eq!(
            (host.blob_create)(
                frame.context(),
                BYTES,
                SliceV1 {
                    data: std::ptr::null(),
                    length: 0
                },
                &mut handle
            ),
            SUCCESS
        );
    }
    assert_eq!(frame.value(handle).unwrap(), Value::Bytes(Vec::new()));
}
