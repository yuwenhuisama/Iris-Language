use core::{cmp::Ordering, fmt};

use num_bigint::BigInt;

use crate::IntegerValue;
use crate::StableHashError;

/// A primitive Iris number without heap identity.
#[derive(Clone, Debug, PartialEq)]
pub enum NumericValue {
    /// An arbitrary-precision integer.
    Integer(IntegerValue),
    /// An IEEE-754 binary32 value.
    Float32(f32),
    /// An IEEE-754 binary64 value.
    Float64(f64),
}

/// A recoverable failure produced by primitive numeric operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericError {
    /// Integer division or remainder received a zero divisor.
    DivisionByZero,
    /// An exponentiation input has no defined Iris result.
    Domain,
    /// A bit reinterpretation input is outside its unsigned interchange range.
    Range,
    /// The requested exact integer operation exceeds the runtime's host resource limit.
    Resource,
    /// An operation requires integer operands.
    IntegerOperandsRequired,
}

impl fmt::Display for NumericError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivisionByZero => formatter.write_str("Iris integer division by zero"),
            Self::Domain => formatter.write_str("Iris numeric domain error"),
            Self::Range => formatter.write_str("Iris numeric range error"),
            Self::Resource => formatter.write_str("Iris numeric resource limit exceeded"),
            Self::IntegerOperandsRequired => {
                formatter.write_str("Iris operation requires Integer operands")
            }
        }
    }
}

impl std::error::Error for NumericError {}

/// Primitive Iris numeric operations independent of message dispatch.
pub struct Numeric;

impl Numeric {
    /// Returns the stable public hash for a numeric value.
    pub fn public_hash(value: &NumericValue) -> Result<u64, StableHashError> {
        crate::numeric_public_hash(value)
    }
    /// Adds two Iris numeric values using the receiver-width rules.
    pub fn add(left: &NumericValue, right: &NumericValue) -> Result<NumericValue, NumericError> {
        Self::binary(
            left,
            right,
            |left, right| left + right,
            |left, right| left + right,
            |left, right| left + right,
        )
    }

    /// Subtracts two Iris numeric values using the receiver-width rules.
    pub fn sub(left: &NumericValue, right: &NumericValue) -> Result<NumericValue, NumericError> {
        Self::binary(
            left,
            right,
            |left, right| left - right,
            |left, right| left - right,
            |left, right| left - right,
        )
    }

    /// Multiplies two Iris numeric values using the receiver-width rules.
    pub fn mul(left: &NumericValue, right: &NumericValue) -> Result<NumericValue, NumericError> {
        Self::binary(
            left,
            right,
            |left, right| left * right,
            |left, right| left * right,
            |left, right| left * right,
        )
    }

    /// Divides two Iris numeric values using Iris integer and IEEE float semantics.
    pub fn div(left: &NumericValue, right: &NumericValue) -> Result<NumericValue, NumericError> {
        match (left, right) {
            (NumericValue::Integer(left), NumericValue::Integer(right)) => {
                if right.is_zero() {
                    return Err(NumericError::DivisionByZero);
                }
                Ok(NumericValue::Float64(left.to_f64() / right.to_f64()))
            }
            _ => Self::binary(
                left,
                right,
                |left, right| left / right,
                |left, right| left / right,
                |_, _| BigInt::from(0_u8),
            ),
        }
    }

    /// Computes the fused multiply-add result at the common float width.
    pub fn mul_add(
        receiver: &NumericValue,
        multiplier: &NumericValue,
        addend: &NumericValue,
    ) -> Result<NumericValue, NumericError> {
        match Self::common_width(receiver, multiplier, Some(addend)) {
            FloatWidth::Float32 => Ok(NumericValue::Float32(
                Self::as_f32(receiver).mul_add(Self::as_f32(multiplier), Self::as_f32(addend)),
            )),
            FloatWidth::Float64 => Ok(NumericValue::Float64(
                Self::as_f64(receiver).mul_add(Self::as_f64(multiplier), Self::as_f64(addend)),
            )),
        }
    }

    /// Raises a numeric value to a numeric power.
    pub fn pow(base: &NumericValue, exponent: &NumericValue) -> Result<NumericValue, NumericError> {
        match (base, exponent) {
            (NumericValue::Integer(base), NumericValue::Integer(exponent)) => {
                Self::integer_pow(base, exponent)
            }
            _ => Self::float_pow(base, exponent),
        }
    }

    /// Computes floor division for two Integers.
    pub fn integer_div(
        left: &NumericValue,
        right: &NumericValue,
    ) -> Result<IntegerValue, NumericError> {
        let (left, right) = Self::integer_pair(left, right)?;
        if right.is_zero() {
            return Err(NumericError::DivisionByZero);
        }
        Ok(IntegerValue::from_bigint(Self::floor_div(
            left.as_bigint(),
            right.as_bigint(),
        )))
    }

    /// Computes modulo using `a - (a div b) * b`.
    pub fn integer_mod(
        left: &NumericValue,
        right: &NumericValue,
    ) -> Result<IntegerValue, NumericError> {
        let (left, right) = Self::integer_pair(left, right)?;
        if right.is_zero() {
            return Err(NumericError::DivisionByZero);
        }
        let quotient = Self::floor_div(left.as_bigint(), right.as_bigint());
        Ok(IntegerValue::from_bigint(
            left.as_bigint() - quotient * right.as_bigint(),
        ))
    }

    /// Bitwise-inverts an Integer under infinite two's-complement semantics.
    pub fn integer_not(value: &NumericValue) -> Result<IntegerValue, NumericError> {
        let value = Self::integer(value)?;
        Ok(IntegerValue::from_bigint(!value.as_bigint()))
    }

    /// Bitwise-ands two Integers under infinite two's-complement semantics.
    ///
    /// `IRIS-V1-RUNTIME-C127` lists `&`, `|`, `^`, `~`, `<<` and `>>` together
    /// under one abstract infinite sign-extended model, so these three are not
    /// a host-width operation on a machine word.
    pub fn integer_and(
        left: &NumericValue,
        right: &NumericValue,
    ) -> Result<IntegerValue, NumericError> {
        let (left, right) = Self::integer_pair(left, right)?;
        Ok(IntegerValue::from_bigint(
            left.as_bigint() & right.as_bigint(),
        ))
    }

    /// Bitwise-ors two Integers under infinite two's-complement semantics.
    pub fn integer_or(
        left: &NumericValue,
        right: &NumericValue,
    ) -> Result<IntegerValue, NumericError> {
        let (left, right) = Self::integer_pair(left, right)?;
        Ok(IntegerValue::from_bigint(
            left.as_bigint() | right.as_bigint(),
        ))
    }

    /// Bitwise-xors two Integers under infinite two's-complement semantics.
    pub fn integer_xor(
        left: &NumericValue,
        right: &NumericValue,
    ) -> Result<IntegerValue, NumericError> {
        let (left, right) = Self::integer_pair(left, right)?;
        Ok(IntegerValue::from_bigint(
            left.as_bigint() ^ right.as_bigint(),
        ))
    }

    /// Shifts an Integer left, reversing direction for negative counts.
    pub fn integer_shift_left(
        value: &NumericValue,
        count: &IntegerValue,
    ) -> Result<IntegerValue, NumericError> {
        Self::integer_shift(value, count, true)
    }

    /// Shifts an Integer right, reversing direction for negative counts.
    pub fn integer_shift_right(
        value: &NumericValue,
        count: &IntegerValue,
    ) -> Result<IntegerValue, NumericError> {
        Self::integer_shift(value, count, false)
    }

    /// Returns whether two Iris numeric values have equal mathematical values.
    pub fn equal(left: &NumericValue, right: &NumericValue) -> bool {
        match (left, right) {
            (NumericValue::Integer(left), NumericValue::Integer(right)) => left == right,
            (NumericValue::Float32(left), NumericValue::Float32(right)) => left == right,
            (NumericValue::Float64(left), NumericValue::Float64(right)) => left == right,
            (NumericValue::Float32(left), NumericValue::Float64(right)) => {
                f64::from(*left) == *right
            }
            (NumericValue::Float64(left), NumericValue::Float32(right)) => {
                *left == f64::from(*right)
            }
            (NumericValue::Integer(integer), NumericValue::Float32(float)) => {
                Self::integer_equals_float(integer, f64::from(*float))
            }
            (NumericValue::Float32(float), NumericValue::Integer(integer)) => {
                Self::integer_equals_float(integer, f64::from(*float))
            }
            (NumericValue::Integer(integer), NumericValue::Float64(float)) => {
                Self::integer_equals_float(integer, *float)
            }
            (NumericValue::Float64(float), NumericValue::Integer(integer)) => {
                Self::integer_equals_float(integer, *float)
            }
        }
    }

    /// Returns whether two Iris numeric values differ.
    pub fn not_equal(left: &NumericValue, right: &NumericValue) -> bool {
        !Self::equal(left, right)
    }

    /// Returns a three-way numeric comparison, or nil-equivalent `None` for NaN.
    pub fn compare(left: &NumericValue, right: &NumericValue) -> Option<Ordering> {
        match (left, right) {
            (NumericValue::Integer(left), NumericValue::Integer(right)) => {
                Some(left.as_bigint().cmp(right.as_bigint()))
            }
            (NumericValue::Integer(integer), NumericValue::Float32(float)) => {
                Self::compare_integer_to_float(integer, f64::from(*float))
            }
            (NumericValue::Integer(integer), NumericValue::Float64(float)) => {
                Self::compare_integer_to_float(integer, *float)
            }
            (NumericValue::Float32(float), NumericValue::Integer(integer)) => {
                Self::compare_integer_to_float(integer, f64::from(*float)).map(Ordering::reverse)
            }
            (NumericValue::Float64(float), NumericValue::Integer(integer)) => {
                Self::compare_integer_to_float(integer, *float).map(Ordering::reverse)
            }
            _ => Self::as_f64(left).partial_cmp(&Self::as_f64(right)),
        }
    }

    /// Compares an arbitrary-precision Integer against a float by exact
    /// mathematical value, placing infinities by extended-real position.
    ///
    /// `IRIS-V1-RUNTIME-C131` requires exact comparison, so the Integer is never
    /// rounded to a float first: a value beyond finite float range would round to
    /// an infinity and compare equal to a genuine infinity.
    fn compare_integer_to_float(integer: &IntegerValue, float: f64) -> Option<Ordering> {
        if float.is_nan() {
            return None;
        }
        if float.is_infinite() {
            // Every Integer is finite, so it sits below +inf and above -inf.
            return Some(if float.is_sign_positive() {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }
        let truncated = Self::float_to_integer(float.trunc())?;
        let ordering = integer.as_bigint().cmp(&truncated);
        if ordering != Ordering::Equal {
            return Some(ordering);
        }
        let fraction = float.fract();
        Some(if fraction > 0.0 {
            Ordering::Less
        } else if fraction < 0.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }

    /// Negates a primitive numeric value.
    pub fn negate(value: &NumericValue) -> Result<NumericValue, NumericError> {
        Ok(match value {
            NumericValue::Integer(value) => {
                NumericValue::Integer(IntegerValue::from_bigint(-value.as_bigint()))
            }
            NumericValue::Float32(value) => NumericValue::Float32(-value),
            NumericValue::Float64(value) => NumericValue::Float64(-value),
        })
    }

    /// Returns canonical Float32 NaN.
    pub const fn float32_nan() -> NumericValue {
        NumericValue::Float32(f32::NAN)
    }

    /// Returns canonical Float64 infinity.
    pub const fn float64_infinity() -> NumericValue {
        NumericValue::Float64(f64::INFINITY)
    }

    /// Reinterprets an unsigned 32-bit Integer as Float32 bits.
    pub fn float32_from_bits(bits: &IntegerValue) -> Result<NumericValue, NumericError> {
        let bits = bits.to_u32().ok_or(NumericError::Range)?;
        Ok(NumericValue::Float32(f32::from_bits(bits)))
    }

    /// Reinterprets an unsigned 64-bit Integer as Float64 bits.
    pub fn float64_from_bits(bits: &IntegerValue) -> Result<NumericValue, NumericError> {
        let bits = bits.to_u64().ok_or(NumericError::Range)?;
        Ok(NumericValue::Float64(f64::from_bits(bits)))
    }

    /// Returns the exact Float32 interchange bits as a nonnegative Integer.
    pub fn float32_to_bits(value: &NumericValue) -> Result<IntegerValue, NumericError> {
        match value {
            NumericValue::Float32(value) => Ok(IntegerValue::from(u64::from(value.to_bits()))),
            NumericValue::Integer(_) | NumericValue::Float64(_) => Err(NumericError::Range),
        }
    }

    /// Returns the exact Float64 interchange bits as a nonnegative Integer.
    pub fn float64_to_bits(value: &NumericValue) -> Result<IntegerValue, NumericError> {
        match value {
            NumericValue::Float64(value) => Ok(IntegerValue::from(value.to_bits())),
            NumericValue::Integer(_) | NumericValue::Float32(_) => Err(NumericError::Range),
        }
    }

    fn binary(
        left: &NumericValue,
        right: &NumericValue,
        float32: impl FnOnce(f32, f32) -> f32,
        float64: impl FnOnce(f64, f64) -> f64,
        integer: impl FnOnce(&BigInt, &BigInt) -> BigInt,
    ) -> Result<NumericValue, NumericError> {
        match (left, right) {
            (NumericValue::Integer(left), NumericValue::Integer(right)) => {
                Ok(NumericValue::Integer(IntegerValue::from_bigint(integer(
                    left.as_bigint(),
                    right.as_bigint(),
                ))))
            }
            _ => match Self::common_width(left, right, None) {
                FloatWidth::Float32 => Ok(NumericValue::Float32(float32(
                    Self::as_f32(left),
                    Self::as_f32(right),
                ))),
                FloatWidth::Float64 => Ok(NumericValue::Float64(float64(
                    Self::as_f64(left),
                    Self::as_f64(right),
                ))),
            },
        }
    }

    fn integer_pow(
        base: &IntegerValue,
        exponent: &IntegerValue,
    ) -> Result<NumericValue, NumericError> {
        if exponent.is_zero() {
            return if base.is_zero() {
                Err(NumericError::Domain)
            } else {
                Ok(NumericValue::Integer(IntegerValue::from(1_u8)))
            };
        }
        if exponent.is_negative() {
            return Ok(NumericValue::Float64(base.to_f64().powf(exponent.to_f64())));
        }
        let exponent = exponent.to_u32().ok_or(NumericError::Resource)?;
        Ok(NumericValue::Integer(IntegerValue::from_bigint(
            base.as_bigint().pow(exponent),
        )))
    }

    fn float_pow(
        base: &NumericValue,
        exponent: &NumericValue,
    ) -> Result<NumericValue, NumericError> {
        if Self::is_zero(base) && Self::is_zero(exponent) {
            return Err(NumericError::Domain);
        }
        match Self::common_width(base, exponent, None) {
            FloatWidth::Float32 => Ok(NumericValue::Float32(
                Self::as_f32(base).powf(Self::as_f32(exponent)),
            )),
            FloatWidth::Float64 => Ok(NumericValue::Float64(
                Self::as_f64(base).powf(Self::as_f64(exponent)),
            )),
        }
    }

    fn integer_shift(
        value: &NumericValue,
        count: &IntegerValue,
        leftward: bool,
    ) -> Result<IntegerValue, NumericError> {
        let value = Self::integer(value)?;
        let count_is_negative = count.is_negative();
        let count = count.abs();
        let leftward = if count_is_negative {
            !leftward
        } else {
            leftward
        };
        if !leftward && count.as_bigint().bits() >= value.as_bigint().bits() {
            return Ok(if value.is_negative() {
                IntegerValue::from(-1_i8)
            } else {
                IntegerValue::from(0_u8)
            });
        }
        let count = count.to_usize().ok_or(NumericError::Resource)?;
        let result = if leftward {
            value.as_bigint() << count
        } else {
            value.as_bigint() >> count
        };
        Ok(IntegerValue::from_bigint(result))
    }

    fn integer_pair<'a>(
        left: &'a NumericValue,
        right: &'a NumericValue,
    ) -> Result<(&'a IntegerValue, &'a IntegerValue), NumericError> {
        Ok((Self::integer(left)?, Self::integer(right)?))
    }

    fn integer(value: &NumericValue) -> Result<&IntegerValue, NumericError> {
        match value {
            NumericValue::Integer(value) => Ok(value),
            NumericValue::Float32(_) | NumericValue::Float64(_) => {
                Err(NumericError::IntegerOperandsRequired)
            }
        }
    }

    fn common_width(
        first: &NumericValue,
        second: &NumericValue,
        third: Option<&NumericValue>,
    ) -> FloatWidth {
        if matches!(first, NumericValue::Float64(_))
            || matches!(second, NumericValue::Float64(_))
            || matches!(third, Some(NumericValue::Float64(_)))
        {
            FloatWidth::Float64
        } else {
            FloatWidth::Float32
        }
    }

    fn as_f32(value: &NumericValue) -> f32 {
        match value {
            NumericValue::Integer(value) => value.to_f32(),
            NumericValue::Float32(value) => *value,
            NumericValue::Float64(value) => value.to_string().parse::<f32>().unwrap_or_else(|_| {
                if value.is_sign_negative() {
                    f32::NEG_INFINITY
                } else {
                    f32::INFINITY
                }
            }),
        }
    }

    fn as_f64(value: &NumericValue) -> f64 {
        match value {
            NumericValue::Integer(value) => value.to_f64(),
            NumericValue::Float32(value) => f64::from(*value),
            NumericValue::Float64(value) => *value,
        }
    }

    fn is_zero(value: &NumericValue) -> bool {
        match value {
            NumericValue::Integer(value) => value.is_zero(),
            NumericValue::Float32(value) => *value == 0.0,
            NumericValue::Float64(value) => *value == 0.0,
        }
    }

    fn integer_equals_float(integer: &IntegerValue, float: f64) -> bool {
        if !float.is_finite() || float.fract() != 0.0 {
            return false;
        }
        Self::float_to_integer(float).is_some_and(|value| integer.as_bigint() == &value)
    }

    fn float_to_integer(float: f64) -> Option<BigInt> {
        if !float.is_finite() {
            return None;
        }
        let bits = float.to_bits();
        let exponent_bits = u16::try_from((bits >> 52) & 0x07ff).ok()?;
        let exponent = i32::from(exponent_bits) - 1023;
        let fraction = bits & ((1_u64 << 52) - 1);
        if exponent_bits == 0 {
            return (fraction == 0).then_some(BigInt::from(0_u8));
        }
        if exponent < 0 {
            return None;
        }
        let significand = fraction | (1_u64 << 52);
        let value = if exponent >= 52 {
            BigInt::from(significand) << usize::try_from(exponent - 52).ok()?
        } else {
            let shift = u32::try_from(52 - exponent).ok()?;
            let mask = (1_u64 << shift) - 1;
            if significand & mask != 0 {
                return None;
            }
            BigInt::from(significand >> shift)
        };
        Some(if float.is_sign_negative() {
            -value
        } else {
            value
        })
    }

    fn floor_div(left: &BigInt, right: &BigInt) -> BigInt {
        let quotient = left / right;
        let remainder = left % right;
        if remainder != BigInt::from(0_u8) && left.sign() != right.sign() {
            quotient - 1
        } else {
            quotient
        }
    }
}

#[derive(Clone, Copy)]
enum FloatWidth {
    Float32,
    Float64,
}
