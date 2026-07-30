use super::evaluate;

fn rendered(source: &str) -> String {
    match evaluate(source) {
        Ok(value) => format!("{value:?}"),
        Err(error) => format!("{error:?}"),
    }
}

#[test]
fn c091_bool_defines_a_total_order_within_bool() {
    // Given
    let source = "[false <=> false, false <=> true, true <=> false, true <=> true]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Integer(IntegerValue(0)), Integer(IntegerValue(-1)), \
         Integer(IntegerValue(1)), Integer(IntegerValue(0))])"
    );
}

#[test]
fn c091_bool_derived_relations_place_false_below_true() {
    // Given
    let source = "[false < true, true > false, false <= false, true >= true, true < false]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(true), Bool(true), Bool(true), Bool(false)])"
    );
}

#[test]
fn c091_bool_is_never_equal_or_ordered_with_a_numeric() {
    // Given
    let source = "[true == 1, false == 0, 1 == true, true < 1, 1 < true, true <=> 1, 1 <=> true]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(false), Bool(false), Bool(false), Bool(false), Bool(false), Nil, Nil])"
    );
}

#[test]
fn c092_nil_is_order_equivalent_only_to_itself() {
    // Given
    let source = "[nil <=> nil, nil == nil, nil != nil, nil <= nil, nil >= nil]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Integer(IntegerValue(0)), Bool(true), Bool(false), Bool(true), Bool(true)])"
    );
}

#[test]
fn c092_nil_is_neither_a_global_minimum_nor_a_global_maximum() {
    // Given
    let source = "class A { }; [nil < 1, nil > 1, nil < A.new(), nil > A.new(), nil <=> false]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(false), Bool(false), Bool(false), Bool(false), Nil])"
    );
}

#[test]
fn c091_and_c092_cross_category_comparison_is_symmetric() {
    // Given
    let source = "[1 <=> nil, nil <=> 1, true <=> nil, nil <=> true]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Array([Nil, Nil, Nil, Nil])");
}

#[test]
fn c115_signaling_nan_is_classified_without_being_quieted() {
    // Given
    let source = "let x = Float64.from_bits(0x7ff0000000000001); \
                  [x.is_nan(), x.is_signaling_nan(), x.is_infinite(), x.to_bits()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(true), Bool(false), \
         Integer(IntegerValue(9218868437227405313))])"
    );
}

#[test]
fn c115_canonical_nan_is_quiet_and_infinity_is_not_nan() {
    // Given
    let source = "[Float64.nan.is_nan(), Float64.nan.is_signaling_nan(), \
                  Float64.infinity.is_infinite(), Float64.infinity.is_nan()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(true), Bool(false)])"
    );
}

#[test]
fn c115_classifies_finite_subnormal_zero_and_sign() {
    // Given
    let source = "let s = Float64.from_bits(0x0000000000000001); \
                  let n = Float64.from_bits(0x8000000000000000); \
                  [s.is_subnormal(), s.is_normal(), s.is_finite(), n.is_zero(), n.sign_bit()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(true), Bool(true), Bool(true)])"
    );
}

#[test]
fn c114_float32_bit_classification_round_trips_through_to_bits() {
    // Given
    let source = "let f = Float32.from_bits(0x7f800001); \
                  [f.is_nan(), f.is_signaling_nan(), f.is_finite(), f.to_bits()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(true), Bool(false), Integer(IntegerValue(2139095041))])"
    );
}

#[test]
fn c086_equality_tests_identity_before_consulting_spaceship() {
    // Given
    let source = "let mut calls = []; class P { public fun <=>(o) { calls.append(:c); nil } }; \
                  let a = P.new(); let same = a; let b = P.new(); \
                  let r = [a == same, a <=> b, a == b, a != b, a < b]; [r, calls]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Array([Bool(true), Nil, Bool(false), Bool(true), Bool(false)]), \
         Array([Symbol(\"c\"), Symbol(\"c\"), Symbol(\"c\"), Symbol(\"c\")])])"
    );
}

#[test]
fn c083_root_spaceship_answers_nil_for_every_operand() {
    // Given
    let source = "class Q { }; let a = Q.new(); let b = Q.new(); [a <=> b, a <=> 1, a <=> nil]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Array([Nil, Nil, Nil])");
}

#[test]
fn c084_derives_all_six_relations_from_the_visible_spaceship() {
    // Given
    let zero = "class D { public fun <=>(o) { 0 } }; let x = D.new(); let y = D.new(); \
                [x == y, x < y, x <= y, x > y, x >= y, x != y]";
    let less = "class E { public fun <=>(o) { 0 - 1 } }; let x = E.new(); let y = E.new(); \
                [x == y, x < y, x <= y, x > y, x >= y, x != y]";

    // When
    let zero = rendered(zero);
    let less = rendered(less);

    // Then
    assert_eq!(
        zero,
        "Array([Bool(true), Bool(false), Bool(true), Bool(false), Bool(true), Bool(false)])"
    );
    assert_eq!(
        less,
        "Array([Bool(false), Bool(true), Bool(true), Bool(false), Bool(false), Bool(true)])"
    );
}

#[test]
fn c084_nil_spaceship_makes_every_ordered_relation_false() {
    // Given
    let source = "class P { public fun <=>(o) { nil } }; let x = P.new(); let y = P.new(); \
                  [x == y, x != y, x < y, x <= y, x > y, x >= y]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(false), Bool(true), Bool(false), Bool(false), Bool(false), Bool(false)])"
    );
}

#[test]
fn c085_rejects_a_spaceship_result_outside_the_contract() {
    // Given
    let integer = "class B { public fun <=>(o) { 7 } }; let x = B.new(); let y = B.new(); x < y";
    let symbol = "class C { public fun <=>(o) { :sym } }; let x = C.new(); let y = C.new(); x == y";

    // When
    let integer = rendered(integer);
    let symbol = rendered(symbol);

    // Then
    assert_eq!(integer, "ComparisonContractError");
    assert_eq!(symbol, "ComparisonContractError");
}
