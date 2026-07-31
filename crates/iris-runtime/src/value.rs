use core::str::FromStr;

use num_bigint::{BigInt, ParseBigIntError, Sign};

use crate::{BoundMethod, ClassId, ContractId, Method, ObjectId};

/// An arbitrary-precision Iris integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerValue(BigInt);

impl FromStr for IntegerValue {
    type Err = ParseBigIntError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(source).map(Self)
    }
}

impl IntegerValue {
    /// Returns the canonical base-10 text used for external observations.
    pub fn decimal_text(&self) -> String {
        self.0.to_string()
    }

    pub(crate) fn from_bigint(value: BigInt) -> Self {
        Self(value)
    }

    pub(crate) const fn as_bigint(&self) -> &BigInt {
        &self.0
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.sign() == Sign::NoSign
    }

    pub(crate) fn is_negative(&self) -> bool {
        self.0.sign() == Sign::Minus
    }

    pub(crate) fn abs(&self) -> Self {
        Self(if self.is_negative() {
            -&self.0
        } else {
            self.0.clone()
        })
    }

    pub(crate) fn to_u32(&self) -> Option<u32> {
        let (sign, digits) = self.0.to_u32_digits();
        match (sign, digits.as_slice()) {
            (Sign::NoSign, []) | (Sign::Plus, []) => Some(0),
            (Sign::Plus, [value]) => Some(*value),
            (Sign::Minus, _) | (Sign::Plus, [_, ..]) | (Sign::NoSign, [..]) => None,
        }
    }

    pub(crate) fn to_u64(&self) -> Option<u64> {
        let (sign, digits) = self.0.to_u64_digits();
        match (sign, digits.as_slice()) {
            (Sign::NoSign, []) | (Sign::Plus, []) => Some(0),
            (Sign::Plus, [value]) => Some(*value),
            (Sign::Minus, _) | (Sign::Plus, [_, ..]) | (Sign::NoSign, [..]) => None,
        }
    }

    /// Returns the value as a container index, or `None` when out of range.
    pub fn to_usize(&self) -> Option<usize> {
        self.to_u64().and_then(|value| value.try_into().ok())
    }

    pub(crate) fn to_f32(&self) -> f32 {
        self.0.to_string().parse::<f32>().unwrap_or_else(|_| {
            if self.is_negative() {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            }
        })
    }

    pub(crate) fn to_f64(&self) -> f64 {
        self.0.to_string().parse::<f64>().unwrap_or_else(|_| {
            if self.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        })
    }
}

impl From<u8> for IntegerValue {
    fn from(value: u8) -> Self {
        Self(BigInt::from(value))
    }
}

impl From<i8> for IntegerValue {
    fn from(value: i8) -> Self {
        Self(BigInt::from(value))
    }
}

impl From<u64> for IntegerValue {
    fn from(value: u64) -> Self {
        Self(BigInt::from(value))
    }
}

/// A runtime value independent of the heap's storage representation.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// The singleton nil value.
    Nil,
    /// A Boolean singleton value.
    Bool(bool),
    /// An arbitrary-precision integer.
    Integer(IntegerValue),
    /// An IEEE-754 binary32 value.
    Float32(f32),
    /// An IEEE-754 binary64 value.
    Float64(f64),
    /// A literal Iris Array.
    Array(Vec<Value>),
    /// A `SourceLocation` record: path, one-based line, one-based column.
    ///
    /// `IRIS-V1-CONTROL-C079` makes it an immutable identity-less value that
    /// compares and hashes STRUCTURALLY, unlike `ExceptionContext`, so two
    /// locations naming the same position are equal.
    SourceLocation(String, u32, u32),
    /// A `StackFrame` record: the callable's name and its location.
    StackFrame(String, Box<Value>),
    /// A `RaiseSite` record: the location a bare `raise` continued from.
    RaiseSite(Box<Value>),
    /// A cursor over an Array, produced by `Array#iterator`.
    ///
    /// `IRIS-V1-COLLECTIONS-C011` makes Array iterable and `C012` drives `for`
    /// through `iterator()`/`next()`. The cursor carries its own position, so
    /// nested traversals of one Array use distinct Iterator objects as `C037`
    /// requires of the collection iterators generally.
    ArrayIterator(ObjectId),
    /// A literal Iris `Hash<K,V>`.
    ///
    /// `IRIS-V1-COLLECTIONS-C033` leaves iteration order UNSPECIFIED, and
    /// `C028` dispatches each key's current `==` rather than a container-owned
    /// relation. Entries are therefore kept as an association list keyed by
    /// `Value` equality instead of a host `HashMap`, which would impose both a
    /// host hash and a host equality the clauses do not permit.
    Hash(Vec<(Value, Value)>),
    /// An interned Iris Symbol spelling.
    Symbol(String),
    /// A logical built-in Class object.
    Class(ClassId),
    /// An object owned by the runtime heap.
    Object(ObjectId),
    /// A Method bound to one receiver at member-read time.
    BoundMethod(BoundMethod),
    /// An unbound reflective Method object.
    Method(Method),
    /// An immutable Contract view over a receiver.
    ///
    /// `IRIS-V1-TYPES-C050` makes views identity-less capability values, so this
    /// carries the receiver and Contract identity rather than an allocation.
    ContractView(Box<Value>, ContractId),
    /// A declared Contract object.
    ///
    /// `IRIS-V1-TYPES-C041` makes Contract the obligation surface, and
    /// `IRIS-V1-TYPES-C076` requires Class, Module, Contract and Type objects to
    /// stay mutually distinct, so a Contract carries its own identity rather
    /// than reusing `ClassId` or `ModuleId`.
    Contract(ContractId),
    /// An `Iteration.yield(value)` result carrying one yielded value.
    ///
    /// `IRIS-V1-COLLECTIONS-C013` makes it an immutable identity-less value that
    /// MAY carry any Iris value including `nil`, which is why a yielded `nil`
    /// must stay distinguishable from exhaustion.
    IterationYield(Box<Value>),
    /// The unique `Iteration.done` singleton.
    IterationDone,
    /// One evaluated `name: value` argument in flight to a call.
    ///
    /// `IRIS-V1-CONTROL-C026` fixes evaluation order across the positional and
    /// keyword channels, so a keyword argument is evaluated in place alongside
    /// the positionals and carries its name to the binding step rather than
    /// being split into a separate pre-evaluated list.
    KeywordArgument(String, Box<Value>),
    /// An identity-bearing `ExceptionContext` for one propagation event.
    ///
    /// `IRIS-V1-CONTROL-C056` gives every `raise` a fresh runtime-owned context
    /// carrying the raised value, and `IRIS-V1-CONTROL-C057` lets `raise value
    /// from cause` chain an explicit one, where `nil` suppresses chaining.
    ///
    /// The leading `ObjectId` is that identity. `C056` makes each propagation
    /// event DISTINCT, so re-raising the same value must produce a context that
    /// `same?` separates from the one being handled; comparing the payload
    /// structurally would wrongly make those two equal.
    /// The trailing `Vec` is `re_raise_sites`, which `IRIS-V1-CONTROL-D-155`
    /// makes an ORDERED sequence appended to by each bare `raise` without
    /// replacing the root stack, so multiple sites retain occurrence order.
    /// The trailing `Box<Value>` is `raise_location`, the `SourceLocation` of
    /// the INITIAL raise, which `IRIS-V1-CONTROL-C065` exposes get-only.
    ExceptionContext(
        ObjectId,
        Box<Value>,
        Box<Value>,
        Vec<Value>,
        Vec<Value>,
        Box<Value>,
    ),
    /// An identity-bearing Closure object.
    ///
    /// `IRIS-V1-RUNTIME-C042` requires each evaluation of a Closure expression to
    /// create a NEW identity-bearing object with its own captured environment,
    /// and makes default equality identity-only, so this carries an allocation
    /// identity rather than the code it runs.
    Closure(ObjectId),
    /// An interned Type object, distinct from the Class it reifies.
    ///
    /// `IRIS-V1-TYPES-C016` requires Type objects to be interned and
    /// identity-bearing, and `IRIS-V1-TYPES-C076` requires them to be distinct
    /// from the Class object, so a nominal Type carries the ClassId rather than
    /// being that ClassId.
    Type(ClassId),
}
