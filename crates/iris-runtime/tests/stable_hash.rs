use iris_runtime::{IntegerValue, Numeric, NumericValue, Value, public_hash};

fn integer(source: &str) -> IntegerValue {
    match source.parse() {
        Ok(value) => value,
        Err(_) => unreachable!(),
    }
}

#[test]
fn frozen_numeric_and_singleton_hashes_match_c146() -> Result<(), iris_runtime::StableHashError> {
    // Given
    let cases = [
        (
            NumericValue::Integer(integer("0")),
            4_379_003_086_384_345_280,
        ),
        (
            NumericValue::Integer(integer("1")),
            17_824_117_788_395_916_856,
        ),
        (
            NumericValue::Integer(integer("-1")),
            4_134_578_751_433_783_052,
        ),
        (
            NumericValue::Integer(integer("2")),
            14_159_628_755_083_520_277,
        ),
        (NumericValue::Float64(1.5), 14_152_187_996_935_738_735),
        (
            NumericValue::Float64(f64::INFINITY),
            13_642_208_101_069_356_640,
        ),
        (
            NumericValue::Float64(f64::NEG_INFINITY),
            6_949_751_103_572_778_473,
        ),
    ];

    // When
    let actual = cases
        .iter()
        .map(|(value, _)| Numeric::public_hash(value))
        .collect::<Result<Vec<_>, _>>()?;
    let singleton_actual = [
        public_hash(&Value::Nil)?,
        public_hash(&Value::Bool(false))?,
        public_hash(&Value::Bool(true))?,
    ];

    // Then
    assert_eq!(
        actual,
        cases
            .iter()
            .map(|(_, expected)| *expected)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        singleton_actual.map(|value| value.decimal_text()),
        [
            String::from("11850167709044604115"),
            String::from("17921396551637717540"),
            String::from("14186115676603356736"),
        ]
    );
    Ok(())
}

#[test]
fn equal_numbers_and_signed_zeroes_share_public_hashes() -> Result<(), iris_runtime::StableHashError>
{
    // Given
    let zeroes = [
        NumericValue::Integer(integer("0")),
        NumericValue::Float32(0.0),
        NumericValue::Float64(-0.0),
    ];
    let ones = [
        NumericValue::Integer(integer("1")),
        NumericValue::Float64(1.0),
    ];

    // When
    let zero_hashes = zeroes
        .iter()
        .map(Numeric::public_hash)
        .collect::<Result<Vec<_>, _>>()?;
    let one_hashes = ones
        .iter()
        .map(Numeric::public_hash)
        .collect::<Result<Vec<_>, _>>()?;

    // Then
    assert!(zero_hashes.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(one_hashes[0], one_hashes[1]);
    Ok(())
}

#[test]
fn kernel_exposes_only_the_public_hash_through_ordinary_dispatch()
-> Result<(), iris_runtime::KernelError> {
    // Given
    let mut registry = iris_runtime::ClassRegistry::new();
    let kernel = iris_runtime::Kernel::new(&mut registry)?;

    // When
    let hash = kernel.send(
        &registry,
        Value::Float64(-0.0),
        iris_runtime::NativeSelector::Hash,
        &[],
    )?;

    // Then
    assert_eq!(
        hash,
        Value::Integer(IntegerValue::from(4_379_003_086_384_345_280_u64))
    );
    Ok(())
}

#[test]
fn canonical_numeric_bytes_are_not_a_public_selector() {
    // Given
    let selector = "canonical_numeric_bytes";

    // When
    let native_selector = iris_runtime::NativeSelector::from_source(selector);

    // Then
    assert_eq!(native_selector, None);
}
