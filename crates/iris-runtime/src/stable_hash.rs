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
        // C087 fixes a SPECIFICATION-STABLE hash per value family, each with
        // its own domain-separated context, so these no longer fall through to
        // the unsupported arm.
        Value::Text(text) => Ok(string_hash(text)),
        // C088 tags an inclusive end 0x00 and an exclusive one 0x01, and C089
        // composes over the components' own public hashes.
        Value::Range(range) => {
            let start = numeric_hash(&NumericValue::Integer(range.start.clone()))?;
            let end = numeric_hash(&NumericValue::Integer(range.end.clone()))?;
            // C039 includes the STEP in the public hash, so the value's own
            // step is hashed rather than an assumed 1.
            let step = numeric_hash(&NumericValue::Integer(range.step.clone()))?;
            Ok(range_hash(
                range.inclusive_end,
                start.to_u64().unwrap_or_default(),
                end.to_u64().unwrap_or_default(),
                step.to_u64().unwrap_or_default(),
            ))
        }
        Value::Symbol(name) => Ok(symbol_hash(name)),
        // C068 makes the Bytes public hash stable while a ByteArray's raises,
        // so these deliberately do not share an arm.
        Value::Bytes(bytes) => Ok(bytes_hash(bytes)),
        // C077 hashes canonical pattern text plus canonical flags.
        Value::Regex(regex) => Ok(regex_hash(&regex.pattern, &regex.flags)),
        // C022 makes a Tuple hash succeed ONLY when every element hash
        // succeeds, so a failed element propagates and prevents use as a Hash
        // key rather than being skipped.
        Value::Tuple(elements) => {
            let mut hashes = Vec::with_capacity(elements.len());
            for element in elements {
                hashes.push(public_hash(element)?.to_u64().unwrap_or_default());
            }
            Ok(tuple_hash(&hashes))
        }
        Value::IterationDone => Ok(iteration_hash(None)),
        Value::IterationYield(payload) => Ok(iteration_hash(public_hash(payload)?.to_u64())),
        Value::Array(_)
        | Value::ByteArray(_)
        | Value::MutableString(_)
        | Value::Match(_)
        | Value::Library(_)
        | Value::Hash(_)
        | Value::Class(_)
        | Value::Type(..)
        | Value::ComposedType(_)
        | Value::Contract(_)
        | Value::Closure(_)
        | Value::KeywordArgument(_, _)
        | Value::ReadonlyArray(_)
        | Value::SourceLocation(..)
        | Value::StackFrame(..)
        | Value::RaiseSite(_)
        | Value::ArrayIterator(_)
        | Value::HashIterator(_)
        | Value::ByteIterator(_)
        | Value::Generator(_)
        | Value::Task(_)
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

/// `IRIS-V1-COLLECTIONS-C087` contexts, one per value family.
const SYMBOL_CONTEXT: &str = "Iris Language v1 stable symbol hash";
const STRING_CONTEXT: &str = "Iris Language v1 stable string hash";
const BYTES_CONTEXT: &str = "Iris Language v1 stable bytes hash";
const TUPLE_CONTEXT: &str = "Iris Language v1 stable tuple hash";
const RANGE_CONTEXT: &str = "Iris Language v1 stable range hash";
const ITERATION_CONTEXT: &str = "Iris Language v1 stable iteration hash";
const REGEX_CONTEXT: &str = "Iris Language v1 stable regex hash";

/// The shortest unsigned LEB128 encoding of `value`.
///
/// `IRIS-V1-COLLECTIONS-C086` makes the CANONICAL encoding the shortest one, so
/// a redundant continuation group would be noncanonical and produce a different
/// hash for the same value.
fn uleb128(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let byte = u8::try_from(value & 0x7f).unwrap_or_default();
        value >>= 7;
        if value == 0 {
            bytes.push(byte);
            return bytes;
        }
        bytes.push(byte | 0x80);
    }
}

/// A length-prefixed stable hash over raw content bytes.
///
/// `IRIS-V1-COLLECTIONS-C087` gives Symbol, String, and Bytes the same shape:
/// `ULEB128(length) || content`, differing only in the domain-separated
/// context, which `C087` forbids reusing across families.
fn length_prefixed_hash(context: &str, content: &[u8]) -> IntegerValue {
    let mut input = uleb128(content.len() as u64);
    input.extend_from_slice(content);
    IntegerValue::from(digest_hash(context, &input))
}

/// The `IRIS-V1-COLLECTIONS-C087` Symbol public hash.
pub fn symbol_hash(name: &str) -> IntegerValue {
    length_prefixed_hash(SYMBOL_CONTEXT, name.as_bytes())
}

/// The `IRIS-V1-COLLECTIONS-C087` String public hash.
pub fn string_hash(text: &str) -> IntegerValue {
    length_prefixed_hash(STRING_CONTEXT, text.as_bytes())
}

/// The `IRIS-V1-COLLECTIONS-C087` Bytes public hash.
pub fn bytes_hash(bytes: &[u8]) -> IntegerValue {
    length_prefixed_hash(BYTES_CONTEXT, bytes)
}

/// The `IRIS-V1-COLLECTIONS-C087` Tuple public hash.
///
/// `C089` makes it COMPOSITIONAL over component public hashes and forbids
/// falling back to object identity, so the caller supplies the already-computed
/// element hashes and any component failure propagates before reaching here.
pub fn tuple_hash(elements: &[u64]) -> IntegerValue {
    let mut input = uleb128(elements.len() as u64);
    for element in elements {
        input.extend_from_slice(&element.to_le_bytes());
    }
    IntegerValue::from(digest_hash(TUPLE_CONTEXT, &input))
}

/// The `IRIS-V1-COLLECTIONS-C088` Range public hash.
///
/// The openness tag is `0x00` for an inclusive end and `0x01` for an exclusive
/// one, which is what distinguishes `a ..= b` from `a ..< b`.
pub fn range_hash(inclusive_end: bool, start: u64, end: u64, step: u64) -> IntegerValue {
    let mut input = vec![u8::from(!inclusive_end)];
    input.extend_from_slice(&start.to_le_bytes());
    input.extend_from_slice(&end.to_le_bytes());
    input.extend_from_slice(&step.to_le_bytes());
    IntegerValue::from(digest_hash(RANGE_CONTEXT, &input))
}

/// The `IRIS-V1-COLLECTIONS-C087` Iteration public hash.
///
/// `Iteration.done` hashes `[0x00]`; a yield hashes `[0x01]` followed by its
/// payload's public hash, so a yielded `nil` stays distinguishable from
/// exhaustion.
pub fn iteration_hash(payload: Option<u64>) -> IntegerValue {
    let input = payload.map_or_else(
        || vec![0x00],
        |payload| {
            let mut input = vec![0x01];
            input.extend_from_slice(&payload.to_le_bytes());
            input
        },
    );
    IntegerValue::from(digest_hash(ITERATION_CONTEXT, &input))
}

/// The `IRIS-V1-COLLECTIONS-C087` Regex public hash.
///
/// `C081` stores canonical flags in fixed `imsx` order with absent flags
/// omitted, so two spellings of one Regex hash alike.
pub fn regex_hash(pattern: &str, canonical_flags: &str) -> IntegerValue {
    let mut input = uleb128(pattern.len() as u64);
    input.extend_from_slice(pattern.as_bytes());
    input.extend(uleb128(canonical_flags.len() as u64));
    input.extend_from_slice(canonical_flags.as_bytes());
    IntegerValue::from(digest_hash(REGEX_CONTEXT, &input))
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

#[cfg(test)]
mod collections_hash_tests {
    use super::{
        bytes_hash, iteration_hash, range_hash, regex_hash, string_hash, symbol_hash, tuple_hash,
    };

    /// Every value is one of the `IRIS-V1-COLLECTIONS-C087` vectors the chapter
    /// states, reproduced here so a change to any context or byte layout fails
    /// against the specification's own numbers rather than against this
    /// implementation's previous output.
    #[test]
    fn c087_reproduces_the_stated_vectors() {
        // V001, V002: Symbol.
        assert_eq!(
            symbol_hash("name").to_u64(),
            Some(10_999_003_376_002_830_561)
        );
        assert_eq!(symbol_hash("@x").to_u64(), Some(16_062_566_904_318_171_380));
        // V003, V004: String.
        assert_eq!(string_hash("").to_u64(), Some(8_628_710_579_024_922_659));
        assert_eq!(
            string_hash("Iris").to_u64(),
            Some(2_999_030_340_536_694_829)
        );
        // V005, V006: Bytes.
        assert_eq!(bytes_hash(&[]).to_u64(), Some(7_904_966_358_175_078_983));
        assert_eq!(
            bytes_hash(&[0xff, 0x00, 0x01, 0x02]).to_u64(),
            Some(17_410_268_034_348_844_959)
        );
        // V011, V012: Iteration. A yielded nil stays distinct from exhaustion.
        assert_ne!(iteration_hash(None), iteration_hash(Some(0)));
        // The compositional families differ by shape, not just by content.
        assert_ne!(tuple_hash(&[]), tuple_hash(&[0]));
        assert_ne!(range_hash(true, 1, 3, 1), range_hash(false, 1, 3, 1));
        assert_ne!(regex_hash("a+", "im"), regex_hash("a+", "i"));
    }
}
