use iris_runtime::{IntegerValue, Numeric, NumericError, NumericValue};

fn integer(source: &str) -> IntegerValue {
    match source.parse() {
        Ok(value) => value,
        Err(_) => unreachable!(),
    }
}

#[test]
fn v035_nan_compares_unequal_to_itself() {
    // Given
    let nan = NumericValue::Float64(f64::NAN);

    // When
    let equal = Numeric::equal(&nan, &nan);

    // Then
    assert!(!equal);
    assert!(Numeric::not_equal(&nan, &nan));
}

#[test]
fn v036_signed_zeroes_are_equal_but_keep_their_interchange_bits() -> Result<(), NumericError> {
    // Given
    let positive = Numeric::float64_from_bits(&integer("0"))?;
    let negative = Numeric::float64_from_bits(&integer("9223372036854775808"))?;

    // When
    let equal = Numeric::equal(&positive, &negative);

    // Then
    assert!(equal);
    assert_eq!(Numeric::float64_to_bits(&positive)?, integer("0"));
    assert_eq!(
        Numeric::float64_to_bits(&negative)?,
        integer("9223372036854775808")
    );
    Ok(())
}

#[test]
fn v039_and_v040_integer_floor_division_and_modulo_match_the_spec() -> Result<(), NumericError> {
    // Given
    let positive_five = NumericValue::Integer(integer("5"));
    let negative_five = NumericValue::Integer(integer("-5"));
    let positive_two = NumericValue::Integer(integer("2"));
    let negative_two = NumericValue::Integer(integer("-2"));

    // When
    let divisions = [
        Numeric::integer_div(&negative_five, &positive_two)?,
        Numeric::integer_div(&positive_five, &negative_two)?,
        Numeric::integer_div(&negative_five, &negative_two)?,
    ];
    let remainders = [
        Numeric::integer_mod(&negative_five, &positive_two)?,
        Numeric::integer_mod(&positive_five, &negative_two)?,
        Numeric::integer_mod(&negative_five, &negative_two)?,
    ];

    // Then
    assert_eq!(divisions, [integer("-3"), integer("-3"), integer("2")]);
    assert_eq!(remainders, [integer("1"), integer("-1"), integer("-1")]);
    Ok(())
}

#[test]
fn v041_and_v112_integer_zero_powers_are_domain_or_positive_infinity() {
    // Given
    let zero = NumericValue::Integer(integer("0"));
    let float_zero = NumericValue::Float32(0.0);
    let negative_float_zero = NumericValue::Float64(-0.0);

    // When
    let zero_to_zero = Numeric::pow(&zero, &zero);
    let zero_to_negative_one = Numeric::pow(&zero, &NumericValue::Integer(integer("-1")));
    let float_zero_to_zero = Numeric::pow(&float_zero, &float_zero);
    let negative_float_zero_to_negative_zero =
        Numeric::pow(&negative_float_zero, &NumericValue::Float64(-0.0));

    // Then
    assert_eq!(zero_to_zero, Err(NumericError::Domain));
    assert_eq!(float_zero_to_zero, Err(NumericError::Domain));
    assert_eq!(
        negative_float_zero_to_negative_zero,
        Err(NumericError::Domain)
    );
    assert_eq!(
        zero_to_negative_one,
        Ok(NumericValue::Float64(f64::INFINITY))
    );
}

#[test]
fn v042_negative_float_to_fractional_power_returns_nan() -> Result<(), NumericError> {
    // Given
    let base = NumericValue::Float64(-2.0);
    let exponent = NumericValue::Float64(0.5);

    // When
    let result = Numeric::pow(&base, &exponent)?;

    // Then
    assert!(matches!(result, NumericValue::Float64(value) if value.is_nan()));
    Ok(())
}

#[test]
fn v043_integer_bitwise_operations_use_unbounded_twos_complement() -> Result<(), NumericError> {
    // Given
    let zero = NumericValue::Integer(integer("0"));
    let negative_three = NumericValue::Integer(integer("-3"));
    let eight = NumericValue::Integer(integer("8"));

    // When
    let inverted = Numeric::integer_not(&zero)?;
    let shifted = Numeric::integer_shift_right(&negative_three, &integer("1"))?;
    let reversed = Numeric::integer_shift_left(&eight, &integer("-2"))?;

    // Then
    assert_eq!(inverted, integer("-1"));
    assert_eq!(shifted, integer("-2"));
    assert_eq!(reversed, integer("2"));
    Ok(())
}

#[test]
fn v046_integer_division_returns_float64_even_when_exact() -> Result<(), NumericError> {
    // Given
    let five = NumericValue::Integer(integer("5"));
    let two = NumericValue::Integer(integer("2"));
    let four = NumericValue::Integer(integer("4"));

    // When
    let fractional = Numeric::div(&five, &two)?;
    let exact = Numeric::div(&four, &two)?;

    // Then
    assert_eq!(fractional, NumericValue::Float64(2.5));
    assert_eq!(exact, NumericValue::Float64(2.0));
    Ok(())
}

#[test]
fn v047_and_v048_fused_arithmetic_preserves_result_width() -> Result<(), NumericError> {
    // Given
    let infinity = NumericValue::Float64(f64::INFINITY);
    let negative_infinity = NumericValue::Float64(f64::NEG_INFINITY);

    // When
    let invalid = Numeric::mul_add(
        &infinity,
        &NumericValue::Integer(integer("1")),
        &negative_infinity,
    )?;

    // Then
    assert!(matches!(invalid, NumericValue::Float64(value) if value.is_nan()));
    assert!(
        matches!(Numeric::mul_add(&NumericValue::Float32(f32::INFINITY), &NumericValue::Integer(integer("0")), &NumericValue::Integer(integer("1")))?, NumericValue::Float32(value) if value.is_nan())
    );
    assert!(
        matches!(Numeric::mul_add(&NumericValue::Float64(f64::INFINITY), &NumericValue::Integer(integer("1")), &negative_infinity)?, NumericValue::Float64(value) if value.is_nan())
    );
    Ok(())
}

#[test]
fn v056_float_widths_remain_distinct_across_arithmetic() -> Result<(), NumericError> {
    // Given
    let large_integer = NumericValue::Integer(integer("16777217"));

    // When
    let float32_sum = Numeric::add(&large_integer, &NumericValue::Float32(0.0))?;
    let float64_sum = Numeric::add(&NumericValue::Float32(1.5), &NumericValue::Float64(2.25))?;

    // Then
    assert_eq!(float32_sum, NumericValue::Float32(16_777_216.0));
    assert_eq!(float64_sum, NumericValue::Float64(3.75));
    Ok(())
}

#[test]
fn v057_v058_v059_v060_and_v104_observe_ieee_and_typed_integer_errors() -> Result<(), NumericError>
{
    // Given
    let one = NumericValue::Float64(1.0);
    let positive_zero = NumericValue::Float64(0.0);
    let negative_zero = NumericValue::Float64(-0.0);
    let max_float32 = Numeric::float32_from_bits(&integer("2139095039"))?;

    // When
    let positive_infinity = Numeric::div(&one, &positive_zero)?;
    let negative_infinity = Numeric::div(&one, &negative_zero)?;
    let nan = Numeric::div(&positive_zero, &positive_zero)?;
    let overflow = Numeric::mul(&max_float32, &NumericValue::Float32(2.0))?;
    let integer_zero_division = Numeric::div(
        &NumericValue::Integer(integer("1")),
        &NumericValue::Integer(integer("0")),
    );
    let nan_addition = Numeric::add(
        &NumericValue::Float32(f32::NAN),
        &NumericValue::Float32(1.0),
    )?;
    let infinity_subtraction = Numeric::sub(
        &NumericValue::Float64(f64::INFINITY),
        &NumericValue::Float64(f64::INFINITY),
    )?;

    // Then
    assert_eq!(positive_infinity, NumericValue::Float64(f64::INFINITY));
    assert_eq!(negative_infinity, NumericValue::Float64(f64::NEG_INFINITY));
    assert!(matches!(nan, NumericValue::Float64(value) if value.is_nan()));
    assert_eq!(overflow, NumericValue::Float32(f32::INFINITY));
    assert_eq!(integer_zero_division, Err(NumericError::DivisionByZero));
    assert!(matches!(nan_addition, NumericValue::Float32(value) if value.is_nan()));
    assert!(matches!(infinity_subtraction, NumericValue::Float64(value) if value.is_nan()));
    Ok(())
}

#[test]
fn v062_and_v063_integer_powers_and_huge_shifts_are_exact() -> Result<(), NumericError> {
    // Given
    let two = NumericValue::Integer(integer("2"));
    let one = NumericValue::Integer(integer("1"));
    let negative_one = NumericValue::Integer(integer("-1"));
    let negative_three = NumericValue::Integer(integer("-3"));
    let huge_shift = integer("1000000");

    // When
    let positive_power = Numeric::pow(&two, &NumericValue::Integer(integer("10")))?;
    let negative_power = Numeric::pow(&two, &NumericValue::Integer(integer("-3")))?;
    let negative_zero_power = Numeric::pow(
        &Numeric::float32_from_bits(&integer("2147483648"))?,
        &NumericValue::Integer(integer("-3")),
    )?;
    let positive_shift = Numeric::integer_shift_right(&one, &huge_shift)?;
    let negative_shift = Numeric::integer_shift_right(&negative_one, &huge_shift)?;
    let negative_three_shift = Numeric::integer_shift_right(&negative_three, &huge_shift)?;

    // Then
    assert_eq!(positive_power, NumericValue::Integer(integer("1024")));
    assert_eq!(negative_power, NumericValue::Float64(0.125));
    assert_eq!(
        negative_zero_power,
        NumericValue::Float32(f32::NEG_INFINITY)
    );
    assert_eq!(positive_shift, integer("0"));
    assert_eq!(negative_shift, integer("-1"));
    assert_eq!(negative_three_shift, integer("-1"));
    Ok(())
}

#[test]
fn v065_and_v069_special_values_and_bit_ranges_are_exact() -> Result<(), NumericError> {
    // Given
    let negative = integer("-1");
    let float32_upper_bound = integer("4294967296");
    let float64_upper_bound = integer("18446744073709551616");

    // When
    let float32_nan = Numeric::float32_nan();
    let float64_infinity = Numeric::float64_infinity();

    // Then
    assert!(matches!(float32_nan, NumericValue::Float32(value) if value.is_nan()));
    assert_eq!(float64_infinity, NumericValue::Float64(f64::INFINITY));
    assert_eq!(
        Numeric::negate(&float64_infinity)?,
        NumericValue::Float64(f64::NEG_INFINITY)
    );
    assert_eq!(
        Numeric::float64_from_bits(&negative),
        Err(NumericError::Range)
    );
    assert_eq!(
        Numeric::float32_from_bits(&float32_upper_bound),
        Err(NumericError::Range)
    );
    assert_eq!(
        Numeric::float64_from_bits(&float64_upper_bound),
        Err(NumericError::Range)
    );
    Ok(())
}

#[test]
fn integer_arithmetic_exceeding_64_bits_remains_exact() -> Result<(), NumericError> {
    // Given
    let power = Numeric::pow(
        &NumericValue::Integer(integer("2")),
        &NumericValue::Integer(integer("200")),
    )?;

    // When
    let result = Numeric::add(&power, &NumericValue::Integer(integer("1")))?;

    // Then
    assert_eq!(
        result,
        NumericValue::Integer(integer(
            "1606938044258990275541962092341162602522202993782792835301377"
        ))
    );
    Ok(())
}
