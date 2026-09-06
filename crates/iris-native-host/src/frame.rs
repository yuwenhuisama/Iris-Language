use crate::{NativeError, NativeRegistry, registry::Loaded};
use iris_native_sdk::{CallResultV1, Handle, INVALID_HANDLE, RAISED, SUCCESS};
use iris_runtime::{ExceptionOrigin, NativeBridge, Value};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ffi::c_void,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
pub(crate) struct Frame<'a> {
    pub registry: &'a NativeRegistry,
    pub loaded: Rc<Loaded>,
    values: RefCell<BTreeMap<Handle, Value>>,
    errors: RefCell<BTreeMap<Handle, NativeError>>,
}
impl<'a> Frame<'a> {
    pub fn new(registry: &'a NativeRegistry, loaded: Rc<Loaded>) -> Self {
        Self {
            registry,
            loaded,
            values: RefCell::new(BTreeMap::new()),
            errors: RefCell::new(BTreeMap::new()),
        }
    }
    pub fn context(&self) -> *mut c_void {
        std::ptr::from_ref(self).cast_mut().cast()
    }
    pub fn insert(&self, value: Value) -> Result<Handle, NativeError> {
        let handle = NEXT_HANDLE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| NativeError::Quota)?;
        self.values.borrow_mut().insert(handle, value);
        Ok(handle)
    }
    pub fn value(&self, handle: Handle) -> Result<Value, NativeError> {
        self.values
            .borrow()
            .get(&handle)
            .cloned()
            .ok_or(NativeError::Protocol(INVALID_HANDLE))
    }
    pub fn raise(&self, error: NativeError) -> Result<Handle, NativeError> {
        let native_bridge = match &error {
            NativeError::Raised {
                code,
                message,
                os_code,
            } => Some(NativeBridge {
                package: self.loaded.metadata.package_id.clone(),
                code: code.clone(),
                message: message.clone(),
                os_code: *os_code,
            }),
            _ => None,
        };
        let context = crate::error::context(
            error.raised_value(),
            ExceptionOrigin {
                location: Value::Nil,
                original_stack: vec![Value::StackFrame(
                    format!("native::{}", self.loaded.metadata.package_id),
                    Box::new(Value::Nil),
                )],
                native_bridge,
            },
        );
        let handle = self.insert(context.clone())?;
        self.errors.borrow_mut().insert(
            handle,
            NativeError::Context {
                error: Box::new(error),
                context: Box::new(context),
            },
        );
        Ok(handle)
    }
    pub fn finish(&self, status: i32, result: CallResultV1) -> Result<Value, NativeError> {
        if result.size < size_of::<CallResultV1>() {
            return Err(NativeError::Protocol(6));
        }
        match status {
            SUCCESS if result.context == 0 => self.value(result.value),
            RAISED if result.value == 0 => Err(self
                .errors
                .borrow_mut()
                .remove(&result.context)
                .unwrap_or(NativeError::Protocol(RAISED))),
            status => Err(NativeError::Protocol(status)),
        }
    }
}
