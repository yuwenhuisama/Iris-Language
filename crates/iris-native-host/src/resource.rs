use crate::{NativeError, NativeRegistry, NativeType, frame::Frame, registry::Loaded};
use iris_native_sdk::{CallResultV1, ResourceV1};
use iris_runtime::{ExternalResource, Value};
use std::rc::Rc;

pub(crate) struct ResourceRecord {
    pub value: ExternalResource,
    pub loaded: Rc<Loaded>,
    pub index: usize,
    pub cookie: u64,
    pub callbacks: ResourceV1,
}
impl Drop for ResourceRecord {
    fn drop(&mut self) {
        self.value.take_open();
        if let Some(destroy) = self.callbacks.destroy {
            // SAFETY: this unique host record owns the cookie; loaded retains the
            // library until destroy returns on the registry's owning thread.
            unsafe { destroy(self.cookie) };
        }
    }
}
impl NativeRegistry {
    pub fn close(&self, value: &ExternalResource) -> Result<Value, NativeError> {
        let (loaded, cookie, callback) = {
            let records = self.resources.borrow();
            let record = records
                .iter()
                .find(|record| record.value == *value)
                .ok_or(NativeError::Closed)?;
            (
                Rc::clone(&record.loaded),
                record.cookie,
                record.callbacks.close.ok_or(NativeError::Metadata)?,
            )
        };
        if !value.take_open() {
            return Ok(Value::Nil);
        }
        let frame = Frame::new(self, loaded);
        let mut result = CallResultV1::default();
        // SAFETY: callback belongs to the retained library; frame/result are live
        // and disjoint, and no resource-table borrow survives into native code.
        let status = unsafe {
            callback(
                &crate::callbacks::HOST,
                frame.context(),
                cookie,
                &mut result,
            )
        };
        frame.finish(status, result)?;
        Ok(Value::Nil)
    }
    pub(crate) fn check_type(
        &self,
        value: &Value,
        name: &str,
        module: &str,
        loaded: &Rc<Loaded>,
    ) -> Result<(), NativeError> {
        let valid = match (NativeType::parse(name)?, value) {
            (NativeType::Nil, Value::Nil)
            | (NativeType::Bool, Value::Bool(_))
            | (NativeType::String, Value::Text(_))
            | (NativeType::Bytes, Value::Bytes(_)) => true,
            (NativeType::Integer, Value::Integer(integer)) => integer
                .to_i128()
                .is_some_and(|value| i64::try_from(value).is_ok()),
            (NativeType::Resource(name), Value::ExternalResource(resource)) => {
                resource.type_name() == format!("{module}::{name}")
                    && self.resources.borrow().iter().any(|record| {
                        record.value == *resource && Rc::ptr_eq(&record.loaded, loaded)
                    })
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(NativeError::Type)
        }
    }
}
impl Frame<'_> {
    pub(crate) fn create_resource(&self, index: usize, cookie: u64) -> Result<Value, NativeError> {
        let (name, callbacks) = self.loaded.resources.get(index).ok_or(NativeError::Type)?;
        let mut records = self.registry.resources.borrow_mut();
        if records
            .iter()
            .filter(|record| Rc::ptr_eq(&record.loaded, &self.loaded))
            .count()
            >= self.loaded.quota
        {
            return Err(NativeError::Quota);
        }
        if records
            .iter()
            .any(|record| Rc::ptr_eq(&record.loaded, &self.loaded) && record.cookie == cookie)
        {
            return Err(NativeError::Type);
        }
        let value = ExternalResource::new(name.clone());
        records.push(ResourceRecord {
            value: value.clone(),
            loaded: Rc::clone(&self.loaded),
            index,
            cookie,
            callbacks: *callbacks,
        });
        Ok(Value::ExternalResource(value))
    }
    pub(crate) fn read_resource(&self, value: &Value, index: usize) -> Result<u64, NativeError> {
        let Value::ExternalResource(value) = value else {
            return Err(NativeError::Type);
        };
        if !value.is_open() {
            return Err(NativeError::Closed);
        }
        self.registry
            .resources
            .borrow()
            .iter()
            .find(|record| {
                record.value == *value
                    && record.index == index
                    && Rc::ptr_eq(&record.loaded, &self.loaded)
            })
            .map(|record| record.cookie)
            .ok_or(NativeError::Type)
    }
}
