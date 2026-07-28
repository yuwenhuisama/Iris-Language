use core::str::FromStr;

use num_bigint::{BigInt, ParseBigIntError, Sign};

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

impl IntegerValue {
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

    pub(crate) fn to_usize(&self) -> Option<usize> {
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
    /// An object owned by the runtime heap.
    Object(ObjectId),
}
