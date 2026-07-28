use core::str::FromStr;

use num_bigint::{BigInt, ParseBigIntError};

use crate::ObjectId;

/// An arbitrary-precision Iris integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerValue(BigInt);

impl FromStr for IntegerValue {
    type Err = ParseBigIntError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        BigInt::from_str(source).map(Self)
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
    /// An object owned by the runtime heap.
    Object(ObjectId),
}
