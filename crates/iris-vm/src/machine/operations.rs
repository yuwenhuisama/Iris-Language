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
        match (&receiver, selector, arguments) {
            (Value::Bytes(bytes), "length", []) => {
                return Ok(Value::Integer((bytes.len() as u64).into()));
            }
            (Value::ByteArray(bytes), "length", []) => {
                return Ok(Value::Integer((bytes.len() as u64).into()));
            }
            (Value::MutableString(text), "length", []) => {
                return Ok(Value::Integer((text.text().chars().count() as u64).into()));
            }
            _ => {}
        }
        // A Regex or Match has no dedicated builtin Class, so it dispatches as
        // an `Object` whose registry entry has no `hash`. `C087` still fixes a
        // public hash over the CANONICAL pattern and flags, which is what
        // makes `/a+/im` and `/a+/mi` hash alike.
        // `C043` makes each open an IDENTITY-BEARING Library, so equality
        // compares identity rather than the path two opens happen to share.
        // An OBJECT compares by identity unless its class defines `==`, and
        // `!=` is that negated. Neither reaches the kernel, because an Object
        // dispatches through its own class and the registry has no entry.
        if matches!(selector, "==" | "!=")
            && matches!(
                receiver,
                Value::Object(_) | Value::Class(_) | Value::Method(_)
            )
            && let [other] = arguments
        {
            let same = self.identity(&receiver, other)?;
            let Value::Bool(same) = same else {
                return Err(MachineError::Kernel(KernelError::Type));
            };
            return Ok(Value::Bool(if selector == "==" { same } else { !same }));
        }
        if selector == "=="
            && let (Value::Library(left), [Value::Library(right)]) = (&receiver, arguments)
        {
            return Ok(Value::Bool(left.identity == right.identity));
        }
        // `C087` fixes a SPECIFICATION-STABLE hash per value family, and
        // `public_hash` decides every family that has one - a Symbol, a Range,
        // a Tuple, an iteration signal and the rest. Installing the selector
        // only on the kernel's own classes left each of those answering
        // MessageNotFound for a hash the language plainly defines. A family it
        // does NOT decide falls through, so an Object still hashes by identity
        // and an Array still refuses.
        // `C091` gives each value family its own EQUALITY, and the kernel
        // installs `==` only on String, Nil and Bool - so a Symbol, a Range, a
        // Tuple, a byte string, a mutable string and an iteration signal all
        // answered MessageNotFound for a comparison the language plainly
        // defines. A family this cannot decide falls through to the kernel.
        if matches!(selector, "==" | "!=")
            && let [other] = arguments
            && let Some(equal) = structural_equality(&receiver, other)
        {
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        if selector == "hash" && arguments.is_empty() {
            // An IDENTITY-bearing value hashes by its identity, which is
            // stable across the value's life rather than derived from content.
            if let Some(identity) = identity_hash(&receiver) {
                return Ok(Value::Integer(identity.into()));
            }
            match iris_runtime::public_hash(&receiver) {
                Ok(hash) => return Ok(Value::Integer(hash)),
                // A NaN has no valid hash for a reason of its OWN, which the
                // kernel names - so that failure is reported as the kernel
                // spells it rather than flattened into a key failure.
                Err(iris_runtime::StableHashError::InvalidNumericKey) => {
                    return Err(MachineError::Kernel(KernelError::StableHash(
                        iris_runtime::StableHashError::InvalidNumericKey,
                    )));
                }
                // A family with no stable hash still HAS the selector, so
                // asking for one is a key failure rather than an absent
                // method. An Object never reaches here: it hashes by identity
                // through authored dispatch, which runs first.
                Err(_) => return Err(MachineError::InvalidKeyError),
            }
        }
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
            | (Value::Bool(true), Value::Bool(true))
            | (Value::IterationDone, Value::IterationDone) => true,
            (Value::Nil, _) | (Value::Bool(_), _) | (_, Value::Nil) | (_, Value::Bool(_)) => false,
            (Value::IterationDone, _) | (_, Value::IterationDone) => false,
            (Value::IterationYield(_), Value::IterationYield(_)) => false,
            (Value::IterationYield(_), _) | (_, Value::IterationYield(_)) => false,
            (Value::Object(left), Value::Object(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,
            (Value::Type(left, left_arguments), Value::Type(right, right_arguments)) => {
                left == right && left_arguments == right_arguments
            }
            (Value::ComposedType(left), Value::ComposedType(right)) => left == right,
            (Value::Type(..), Value::ComposedType(_))
            | (Value::ComposedType(_), Value::Type(..)) => false,
            (Value::Array(left), Value::Array(right)) => left.same(right),
            (Value::Hash(left), Value::Hash(right)) => left.same(right),
            (Value::ExceptionContext(left, ..), Value::ExceptionContext(right, ..)) => {
                left == right
            }
            // `same?` asks whether two references name ONE value, so only a
            // family with a REFERENCE answers it. A Symbol, a Text or an
            // Integer is a value with no identity of its own, and answering
            // by content made `:t same? :t` true where the language refuses
            // the question entirely.
            (Value::MutableString(left), Value::MutableString(right)) => left.same(right),
            (Value::Closure(left), Value::Closure(right)) => left == right,
            (Value::ArrayIterator(left), Value::ArrayIterator(right)) => left == right,
            (Value::HashIterator(left), Value::HashIterator(right)) => left == right,
            (Value::ByteIterator(left), Value::ByteIterator(right)) => left == right,
            (Value::Generator(left), Value::Generator(right)) => left == right,
            (Value::Task(left), Value::Task(right)) => left == right,
            (Value::Gate(left), Value::Gate(right)) => left == right,
            // A BOUND method is a fresh value per binding, so `obj.method`
            // twice names two of them - its own runtime identity decides.
            (Value::BoundMethod(left), Value::BoundMethod(right)) => left.id() == right.id(),
            // A METHOD is the definition itself, which is interned once per
            // declaration, so two reads of one selector name the same value.
            (Value::Method(left), Value::Method(right)) => left == right,
            // `C029` accepts only identity-BEARING operands, so an
            // identity-less value raises rather than being compared by
            // content. A contract VIEW is identity-less too: `C050` makes it a
            // capability rather than a value with its own identity.
            _ => return Err(MachineError::IdentityError),
        };
        Ok(Value::Bool(same))
    }

    pub(super) fn index(&self, receiver: Value, index: Value) -> Result<Value, MachineError> {
        match receiver {
            Value::Array(values) => {
                let elements = values.elements();
                // A RANGE index answers a SLICE, and the slice is its own
                // array rather than a view: writing through either one leaves
                // the other unchanged, which is what `a[0 ..< 2]` then
                // `a[0] = 9` observes.
                if let Value::Range(range) = &index {
                    return Ok(Value::Array(iris_runtime::ArrayRef::new(
                        slice_bounds(range, elements.len())
                            .map(|(from, to)| elements[from..to].to_vec())
                            .unwrap_or_default(),
                    )));
                }
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
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
            Value::ReadonlyArray(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                Ok(resolve_index(&index, values.len())
                    .and_then(|index| values.get(index).cloned())
                    .unwrap_or(Value::Nil))
            }
            Value::Bytes(bytes) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                Ok(resolve_index(&index, bytes.len())
                    .and_then(|index| bytes.get(index).copied())
                    .map(|byte| Value::Integer(byte.into()))
                    .unwrap_or(Value::Nil))
            }
            Value::ByteArray(bytes) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                let bytes = bytes.bytes();
                Ok(resolve_index(&index, bytes.len())
                    .and_then(|index| bytes.get(index).copied())
                    .map(|byte| Value::Integer(byte.into()))
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
                // The key was validated by the caller, which can reach an
                // object's own `hash`; this path only stores.
                entries.insert(index, value.clone());
                Ok(value)
            }
            _ => Err(MachineError::UnknownSelector("[]=".to_owned())),
        }
    }
}

/// Reports whether two values are EQUAL, when this decides their family.
///
/// `IRIS-V1-RUNTIME-C091` gives each family its own equality: a Symbol
/// compares by name, a byte string by its bytes whatever its mutability, a
/// mutable string by its current text, and an iteration signal by its payload.
/// A pair this cannot decide answers `None` so the kernel still rules.
fn structural_equality(left: &Value, right: &Value) -> Option<bool> {
    match (left, right) {
        // The PRIMITIVES are decided here too, because a composite's elements
        // are compared through this same function: a Tuple of Texts answered
        // nothing at all while its own arm existed but its elements' did not.
        (Value::Text(left), Value::Text(right)) => Some(left == right),
        (Value::Integer(left), Value::Integer(right)) => Some(left == right),
        (Value::Float32(left), Value::Float32(right)) => Some(left == right),
        (Value::Float64(left), Value::Float64(right)) => Some(left == right),
        (Value::Nil, Value::Nil) => Some(true),
        (Value::Bool(left), Value::Bool(right)) => Some(left == right),
        (Value::Symbol(left), Value::Symbol(right)) => Some(left == right),
        // A Bytes and a ByteArray hold the same content differently, so the
        // comparison is over BYTES rather than over the representation.
        (Value::Bytes(left), Value::Bytes(right)) => Some(left == right),
        (Value::Bytes(left), Value::ByteArray(right)) => Some(*left == right.bytes()),
        (Value::ByteArray(left), Value::Bytes(right)) => Some(left.bytes() == *right),
        (Value::ByteArray(left), Value::ByteArray(right)) => Some(left.bytes() == right.bytes()),
        // A MutableString compares by its CURRENT text, so a write before the
        // comparison is seen.
        (Value::MutableString(left), Value::MutableString(right)) => {
            Some(left.text() == right.text())
        }
        (Value::MutableString(left), Value::Text(right)) => Some(left.text() == *right),
        (Value::Text(left), Value::MutableString(right)) => Some(*left == right.text()),
        (Value::Range(left), Value::Range(right)) => Some(left == right),
        (Value::Tuple(left), Value::Tuple(right)) => {
            if left.len() != right.len() {
                return Some(false);
            }
            let mut equal = true;
            for (left, right) in left.iter().zip(right) {
                equal = equal && structural_equality(left, right)?;
            }
            Some(equal)
        }
        (Value::IterationDone, Value::IterationDone) => Some(true),
        (Value::IterationYield(left), Value::IterationYield(right)) => {
            structural_equality(left, right)
        }
        (Value::IterationDone, Value::IterationYield(_))
        | (Value::IterationYield(_), Value::IterationDone) => Some(false),
        // `C087` makes `/a+/im` and `/a+/mi` the same Regex, so the CANONICAL
        // pattern and flags decide rather than the written order.
        (Value::Regex(left), Value::Regex(right)) => {
            Some(left.pattern == right.pattern && left.flags == right.flags)
        }
        (Value::Contract(left), Value::Contract(right)) => Some(left == right),
        _ => None,
    }
}

/// The identity a value hashes by, when its family bears one.
///
/// An iterator, a closure, a task and the other identity-bearing families have
/// no content hash - they hash by WHICH value they are, which stays stable as
/// the value advances. `C087` fixes a hash per family, and for these that is
/// the identity rather than a derivation.
fn identity_hash(value: &Value) -> Option<u64> {
    match value {
        Value::ArrayIterator(identity)
        | Value::HashIterator(identity)
        | Value::ByteIterator(identity)
        | Value::Generator(identity)
        | Value::Task(identity)
        | Value::Gate(identity)
        | Value::Closure(identity) => Some(identity.raw()),
        _ => None,
    }
}

/// The half-open bounds a RANGE names within a sequence of `length`.
///
/// An endpoint past the end is clamped rather than refused, and a start past
/// the end names an empty slice - `C088` tags an inclusive end separately, so
/// `0 ..= 1` and `0 ..< 2` name the same two elements.
fn slice_bounds(range: &iris_runtime::RangeValue, length: usize) -> Option<(usize, usize)> {
    let from = range
        .start
        .to_u64()
        .and_then(|start| usize::try_from(start).ok())?;
    let end = range
        .end
        .to_u64()
        .and_then(|end| usize::try_from(end).ok())?;
    let to = if range.inclusive_end { end + 1 } else { end };
    let from = from.min(length);
    Some((from, to.clamp(from, length)))
}
