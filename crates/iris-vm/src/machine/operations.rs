//! Native sends, identity, and indexed collection operations.

use iris_runtime::{BuiltinClass, ClassId, KernelError, NativeSelector, Value};

use super::{Machine, MachineError, resolve_index, value_class_name};

impl Machine {
    pub(super) fn builtin_class(&self, name: &str) -> Result<ClassId, MachineError> {
        let kind = match name {
            "Object" => BuiltinClass::Object,
            "Nil" => BuiltinClass::Nil,
            "Bool" => BuiltinClass::Bool,
            "Integer" => BuiltinClass::Integer,
            "Float32" => BuiltinClass::Float32,
            "Float64" => BuiltinClass::Float64,
            "String" => BuiltinClass::String,
            _ => return Err(MachineError::NameError),
        };
        self.kernel.class(kind).map_err(MachineError::Kernel)
    }

    pub(super) fn type_test(&self, value: &Value, target: &Value) -> Result<Value, MachineError> {
        let Value::Class(target) = target else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        if *target
            == self
                .kernel
                .class(BuiltinClass::Object)
                .map_err(MachineError::Kernel)?
        {
            return Ok(Value::Bool(true));
        }
        let class = match value {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction)?,
            Value::Nil => self
                .kernel
                .class(BuiltinClass::Nil)
                .map_err(MachineError::Kernel)?,
            Value::Bool(_) => self
                .kernel
                .class(BuiltinClass::Bool)
                .map_err(MachineError::Kernel)?,
            Value::Integer(_) => self
                .kernel
                .class(BuiltinClass::Integer)
                .map_err(MachineError::Kernel)?,
            Value::Float32(_) => self
                .kernel
                .class(BuiltinClass::Float32)
                .map_err(MachineError::Kernel)?,
            Value::Float64(_) => self
                .kernel
                .class(BuiltinClass::Float64)
                .map_err(MachineError::Kernel)?,
            Value::Text(_) => self
                .kernel
                .class(BuiltinClass::String)
                .map_err(MachineError::Kernel)?,
            Value::Class(class) => *class,
            _ => return Ok(Value::Bool(false)),
        };
        self.is_subtype(class, *target).map(Value::Bool)
    }

    pub(super) fn is_subtype(&self, class: ClassId, target: ClassId) -> Result<bool, MachineError> {
        if target
            == self
                .kernel
                .class(BuiltinClass::Object)
                .map_err(MachineError::Kernel)?
        {
            return Ok(true);
        }
        Ok(self
            .runtime
            .registry()
            .active(class)
            .map_err(MachineError::Class)?
            .mro()
            .iter()
            .any(|entry| matches!(entry, iris_runtime::MroEntry::Class(held) if *held == target)))
    }

    pub(super) fn send(
        &self,
        selector: &str,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        let Some(native) = NativeSelector::from_source(selector) else {
            return Err(MachineError::MessageNotFound {
                receiver_class: value_class_name(&receiver).to_owned(),
                selector: selector.to_owned(),
            });
        };
        self.kernel
            .send(self.runtime.registry(), receiver, native, arguments)
            .map_err(MachineError::Kernel)
    }

    pub(super) fn identity(&self, left: &Value, right: &Value) -> Result<Value, MachineError> {
        let same = match (left, right) {
            (Value::Nil, Value::Nil)
            | (Value::Bool(false), Value::Bool(false))
            | (Value::Bool(true), Value::Bool(true)) => true,
            (Value::Nil, _) | (Value::Bool(_), _) | (_, Value::Nil) | (_, Value::Bool(_)) => false,
            (Value::Object(left), Value::Object(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,
            (Value::Array(left), Value::Array(right)) => left.same(right),
            (Value::Hash(left), Value::Hash(right)) => left.same(right),
            _ => return Err(MachineError::Kernel(KernelError::Identity)),
        };
        Ok(Value::Bool(same))
    }

    pub(super) fn index(&self, receiver: Value, index: Value) -> Result<Value, MachineError> {
        match receiver {
            Value::Array(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                let elements = values.elements();
                Ok(resolve_index(&index, elements.len())
                    .and_then(|index| elements.get(index).cloned())
                    .unwrap_or(Value::Nil))
            }
            Value::Hash(entries) => Ok(entries.get(&index).unwrap_or(Value::Nil)),
            Value::Tuple(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                Ok(resolve_index(&index, values.len())
                    .and_then(|index| values.get(index).cloned())
                    .unwrap_or(Value::Nil))
            }
            _ => Err(MachineError::UnknownSelector("[]".to_owned())),
        }
    }

    pub(super) fn set_index(
        &self,
        receiver: Value,
        index: Value,
        value: Value,
    ) -> Result<Value, MachineError> {
        match receiver {
            Value::Array(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                // The reference answers nil for an out-of-range READ and
                // raises for an out-of-range WRITE, because a write has no
                // position to store into. Discarding it silently would leave
                // the program believing the element was stored.
                let Some(index) = resolve_index(&index, values.len()) else {
                    return Err(MachineError::IndexError);
                };
                let mut stored = false;
                values.mutate(|elements| {
                    if let Some(slot) = elements.get_mut(index) {
                        *slot = value.clone();
                        stored = true;
                    }
                });
                if stored {
                    Ok(value)
                } else {
                    Err(MachineError::IndexError)
                }
            }
            Value::Hash(entries) => {
                iris_runtime::public_hash(&index)
                    .map_err(KernelError::StableHash)
                    .map_err(MachineError::Kernel)?;
                entries.insert(index, value.clone());
                Ok(value)
            }
            _ => Err(MachineError::UnknownSelector("[]=".to_owned())),
        }
    }
}
