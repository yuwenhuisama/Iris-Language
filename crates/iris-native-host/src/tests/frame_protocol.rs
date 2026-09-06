use super::*;

#[test]
fn raised_callback_preserves_code_message_and_os_code() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    let mut result = CallResultV1::default();
    // SAFETY: static byte slices and a disjoint output remain valid for the live frame.
    let status = unsafe {
        (crate::callbacks::HOST.error_raise)(
            frame.context(),
            SliceV1::from_bytes(b"SocketError"),
            SliceV1::from_bytes(b"refused"),
            61,
            &mut result.context,
        )
    };
    assert!(matches!(frame.value(result.context).unwrap(),
        Value::ExceptionContext(_, ref value, ..) if **value == Value::Symbol("SocketError".into())));
    let error = frame.finish(status, result).unwrap_err();
    assert!(matches!(&error, NativeError::Context { error, .. }
        if matches!(error.as_ref(), NativeError::Raised { code, message, os_code: 61 }
            if code == "SocketError" && message == "refused")));
    let (_, context) = error.into_propagation();
    assert!(
        matches!(context, Value::ExceptionContext(_, _, _, _, _, origin)
        if origin.native_bridge.as_ref().is_some_and(|bridge|
            bridge.code == "SocketError" && bridge.message == "refused" && bridge.os_code == 61))
    );
}

#[test]
fn finish_accepts_only_the_valid_success_and_raised_combinations() {
    for status in [SUCCESS, RAISED, INVALID_ARGUMENT, 12345] {
        for value_kind in 0..3 {
            for context_kind in 0..5 {
                for full_size in [true, false] {
                    let registry = NativeRegistry::new();
                    let frame = Frame::new(&registry, loaded());
                    let value = frame.insert(Value::Nil).unwrap();
                    let other = Frame::new(&registry, loaded());
                    let stale = other.raise(NativeError::Type).unwrap();
                    drop(other);
                    let context = frame
                        .raise(NativeError::Raised {
                            code: "SocketError".into(),
                            message: "refused".into(),
                            os_code: 61,
                        })
                        .unwrap();
                    let result = CallResultV1 {
                        size: if full_size {
                            size_of::<CallResultV1>()
                        } else {
                            0
                        },
                        value: [0, value, u64::MAX][value_kind],
                        context: [0, context, value, stale, u64::MAX][context_kind],
                    };

                    let outcome = frame.finish(status, result);

                    match (status, value_kind, context_kind, full_size) {
                        (SUCCESS, 1, 0, true) => assert_eq!(outcome.unwrap(), Value::Nil),
                        (RAISED, 0, 1, true) => {
                            assert!(matches!(outcome, Err(NativeError::Context { .. })))
                        }
                        _ => assert!(matches!(outcome, Err(NativeError::Protocol(_)))),
                    }
                }
            }
        }
    }
}

#[test]
fn finish_preserves_the_callback_context_identity_when_raised() {
    let registry = NativeRegistry::new();
    let frame = Frame::new(&registry, loaded());
    let handle = frame
        .raise(NativeError::Raised {
            code: "SocketError".into(),
            message: "refused".into(),
            os_code: 61,
        })
        .unwrap();
    let expected = frame.value(handle).unwrap();

    let error = frame
        .finish(
            RAISED,
            CallResultV1 {
                context: handle,
                ..CallResultV1::default()
            },
        )
        .unwrap_err();
    let (value, context) = error.into_propagation();

    assert_eq!(context, expected);
    assert_eq!(value, Value::Symbol("SocketError".into()));
}
