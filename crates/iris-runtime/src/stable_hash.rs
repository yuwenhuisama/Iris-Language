use core::fmt;

use num_bigint::{BigInt, Sign};

use crate::{IntegerValue, NumericValue, Value};

const NUMERIC_CONTEXT: &str = "Iris Language v1 stable numeric hash";
const SINGLETON_CONTEXT: &str = "Iris Language v1 stable singleton hash";
/// `D-241` fixes this exact ASCII context for the Contract-view public hash.
const CONTRACT_VIEW_CONTEXT: &str = "Iris Language v1 contract view hash";
/// `D-242` derives a statically named Contract Type's hash from nominal
/// identity rather than structural member shape.
const CONTRACT_TYPE_CONTEXT: &str = "Iris Language v1 contract type hash";

/// A failure from the stable public hash contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StableHashError {
    /// NaN has no valid public hash.
    InvalidNumericKey,
    /// The value category has no stable hash in this runtime slice.
    UnsupportedValue,
}

impl fmt::Display for StableHashError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumericKey => formatter.write_str("NaN has no valid Iris hash"),
            Self::UnsupportedValue => formatter.write_str("Iris value has no stable hash"),
        }
    }
}

impl std::error::Error for StableHashError {}

/// Returns the specification-stable public hash for a supported built-in value.
pub fn public_hash(value: &Value) -> Result<IntegerValue, StableHashError> {
    match value {
        Value::Nil => Ok(IntegerValue::from(singleton_hash(0))),
        Value::Bool(false) => Ok(IntegerValue::from(singleton_hash(1))),
        Value::Bool(true) => Ok(IntegerValue::from(singleton_hash(2))),
        Value::Integer(value) => numeric_hash(&NumericValue::Integer(value.clone())),
        Value::Float32(value) => numeric_hash(&NumericValue::Float32(*value)),
        Value::Float64(value) => numeric_hash(&NumericValue::Float64(*value)),
        Value::Array(_)
        | Value::Hash(_)
        | Value::Text(_)
        | Value::Symbol(_)
        | Value::Class(_)
        | Value::Type(..)
        | Value::ComposedType(_)
        | Value::Contract(_)
        | Value::Closure(_)
        | Value::KeywordArgument(_, _)
        | Value::IterationYield(_)
        | Value::ReadonlyArray(_)
        | Value::SourceLocation(..)
        | Value::StackFrame(..)
        | Value::RaiseSite(_)
        | Value::ArrayIterator(_)
        | Value::Generator(_)
        | Value::IterationDone
        | Value::Transformation { .. }
        | Value::ExceptionContext(..)
        | Value::ContractView(_, _)
        | Value::Object(_)
        | Value::BoundMethod(_)
        | Value::Method(_) => Err(StableHashError::UnsupportedValue),
    }
}

/// Returns the specification-stable public hash for a numeric value.
pub fn numeric_hash(value: &NumericValue) -> Result<IntegerValue, StableHashError> {
    numeric_public_hash(value).map(IntegerValue::from)
}

/// Returns the stable public hash bits for a numeric value.
pub fn numeric_public_hash(value: &NumericValue) -> Result<u64, StableHashError> {
    canonical_numeric_encoding(value).map(|encoding| digest_hash(NUMERIC_CONTEXT, &encoding))
}

fn singleton_hash(tag: u8) -> u64 {
    digest_hash(SINGLETON_CONTEXT, &[tag])
}

/// The public hash of a statically named Contract Type.
///
/// `D-242` derives this from canonical package identity, the fully qualified
/// Contract name, and major-version contract identity, NOT from structural
/// member shape or runtime allocation. Two Contracts with identical
/// declarations therefore stay distinct, and moving a Contract between
/// packages changes its Type identity and hash.
pub fn contract_type_hash(package: &str, qualified_name: &str, api_major: u64) -> IntegerValue {
    let mut input = Vec::new();
    input.extend_from_slice(&(package.len() as u64).to_le_bytes());
    input.extend_from_slice(package.as_bytes());
    input.extend_from_slice(&(qualified_name.len() as u64).to_le_bytes());
    input.extend_from_slice(qualified_name.as_bytes());
    input.extend_from_slice(&api_major.to_le_bytes());
    IntegerValue::from(digest_hash(CONTRACT_TYPE_CONTEXT, &input))
}

/// The hex BLAKE3-256 digest of an audit artifact's source bytes.
///
/// `IRIS-V1-META-C126` scopes a lightweight audit record's digest to the
/// referenced artifact's SOURCE bytes and not its locator, which is what lets a
/// locator-only change preserve the digest while a source change alters all 32
/// bytes. This is the plain hash, not the derive-key mode the public value
/// hashes use, since it identifies an artifact rather than an Iris value.
pub fn artifact_digest(source: &[u8]) -> String {
    blake3::hash(source).to_hex().to_string()
}

/// The public hash of a Contract view.
///
/// `D-241` uses BLAKE3 derive-key mode with the exact ASCII context
/// `Iris Language v1 contract view hash` and input
/// `receiver_public_hash_u64_le || contract_type_hash_u64_le`, reduced by the
/// same first-eight-bytes little-endian rule every other public hash uses.
/// Stability inherits its components rather than being asserted here.
pub fn contract_view_hash(receiver: u64, contract_type: u64) -> IntegerValue {
    let mut input = [0_u8; 16];
    input[..8].copy_from_slice(&receiver.to_le_bytes());
    input[8..].copy_from_slice(&contract_type.to_le_bytes());
    IntegerValue::from(digest_hash(CONTRACT_VIEW_CONTEXT, &input))
}

fn digest_hash(context: &str, input: &[u8]) -> u64 {
    let digest = blake3::derive_key(context, input);
    let mut reduced = [0_u8; 8];
    reduced.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(reduced)
}

pub(crate) fn canonical_numeric_encoding(value: &NumericValue) -> Result<Vec<u8>, StableHashError> {
    match value {
        NumericValue::Integer(value) => Ok(finite_encoding(
            value.as_bigint().clone(),
            BigInt::from(0_u8),
        )),
        NumericValue::Float32(value) => float32_encoding(*value),
        NumericValue::Float64(value) => float64_encoding(*value),
    }
}

fn float32_encoding(value: f32) -> Result<Vec<u8>, StableHashError> {
    let bits = value.to_bits();
    float_encoding(
        bits & (1_u32 << 31) != 0,
        u64::from((bits >> 23) & 0xff),
        u64::from(bits & ((1_u32 << 23) - 1)),
        127,
        23,
        0xff,
    )
}

fn float64_encoding(value: f64) -> Result<Vec<u8>, StableHashError> {
    let bits = value.to_bits();
    float_encoding(
        bits & (1_u64 << 63) != 0,
        (bits >> 52) & 0x07ff,
        bits & ((1_u64 << 52) - 1),
        1023,
        52,
        0x07ff,
    )
}

fn float_encoding(
    negative: bool,
    exponent_bits: u64,
    fraction: u64,
    bias: i32,
    fraction_bits: u32,
    maximum_exponent: u64,
) -> Result<Vec<u8>, StableHashError> {
    if exponent_bits == maximum_exponent {
        return if fraction == 0 {
            Ok(vec![if negative { 2 } else { 1 }])
        } else {
            Err(StableHashError::InvalidNumericKey)
        };
    }
    let (significand, exponent) = if exponent_bits == 0 {
        (fraction, 1 - bias - fraction_bits as i32)
    } else {
        (
            fraction | (1_u64 << fraction_bits),
            exponent_bits as i32 - bias - fraction_bits as i32,
        )
    };
    let value = if negative {
        -BigInt::from(significand)
    } else {
        BigInt::from(significand)
    };
    Ok(finite_encoding(value, BigInt::from(exponent)))
}

fn finite_encoding(value: BigInt, exponent: BigInt) -> Vec<u8> {
    if value.sign() == Sign::NoSign {
        return vec![0, 0];
    }
    let negative = value.sign() == Sign::Minus;
    let mut odd = if negative { -value } else { value };
    let mut trailing = 0_usize;
    while (&odd % 2_u8).sign() == Sign::NoSign {
        odd >>= 1_usize;
        trailing += 1;
    }
    let (_, magnitude) = odd.to_bytes_be();
    let normalized_exponent = exponent + BigInt::from(trailing);
    let zigzag = if normalized_exponent.sign() == Sign::Minus {
        -normalized_exponent * 2_u8 - 1_u8
    } else {
        normalized_exponent * 2_u8
    };
    let mut encoding = vec![0, if negative { 2 } else { 1 }];
    append_uleb128(&mut encoding, magnitude.len());
    encoding.extend(magnitude);
    append_uleb_bigint(&mut encoding, zigzag);
    encoding
}

fn append_uleb128(output: &mut Vec<u8>, value: usize) {
    let mut value = value;
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            return;
        }
    }
}

fn append_uleb_bigint(output: &mut Vec<u8>, mut value: BigInt) {
    loop {
        let (_, digits) = value.to_u32_digits();
        let low = match digits.first() {
            Some(value) => *value & 0x7f,
            None => 0,
        };
        value >>= 7_usize;
        output.push(
            low as u8
                | if value.sign() == Sign::NoSign {
                    0
                } else {
                    0x80
                },
        );
        if value.sign() == Sign::NoSign {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_numeric_encoding;
    use crate::{IntegerValue, NumericValue};

    fn integer(source: &str) -> IntegerValue {
        match source.parse() {
            Ok(value) => value,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn canonical_numeric_encodings_match_c146_prefixes() -> Result<(), super::StableHashError> {
        // Given
        let cases = [
            (NumericValue::Integer(integer("0")), vec![0, 0]),
            (NumericValue::Integer(integer("1")), vec![0, 1, 1, 1, 0]),
            (NumericValue::Integer(integer("-1")), vec![0, 2, 1, 1, 0]),
            (NumericValue::Integer(integer("2")), vec![0, 1, 1, 1, 2]),
            (NumericValue::Float64(1.5), vec![0, 1, 1, 3, 1]),
        ];

        // When
        let encodings = cases
            .iter()
            .map(|(value, _)| canonical_numeric_encoding(value))
            .collect::<Result<Vec<_>, _>>()?;

        // Then
        assert_eq!(
            encodings,
            cases
                .iter()
                .map(|(_, expected)| expected.clone())
                .collect::<Vec<_>>()
        );
        Ok(())
    }
}
