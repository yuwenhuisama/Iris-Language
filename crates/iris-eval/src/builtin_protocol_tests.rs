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

#[test]
fn c075_shares_a_hierarchy_cell_while_class_object_ivars_stay_distinct() {
    // Given
    let source = "class A { shared mut @@x = 0; \
                  class fun set_shared(v) { @@x = v } class fun shared() { @@x } \
                  class fun set_own(v) { @x = v } class fun own() { @x } } \
                  class B extends A { }; \
                  let s1 = A.set_shared(1); let s2 = B.set_shared(2); \
                  let o1 = A.set_own(:a); let o2 = B.set_own(:b); \
                  [A.shared(), B.shared(), A.own(), B.own()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Integer(IntegerValue(2)), Integer(IntegerValue(2)), \
         Symbol(\"a\"), Symbol(\"b\")])"
    );
}

#[test]
fn c162_rejects_a_subclass_redeclaring_an_anchored_cell() {
    // Given
    let source = "class A { shared mut @@x = 0; }; class B extends A { shared mut @@x = 1; }";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(crate::EvaluationError::Class(
            iris_runtime::ClassError::DuplicateClassVariable { .. }
        ))
    ));
}

#[test]
fn d446_runs_property_initializers_superclass_first_then_initialize() {
    // Given
    let source = "let mut log = []; \
                  class Base { property b: Nil = log.append(:base); } \
                  class Child extends Base { property c: Nil = log.append(:child); \
                  fun initialize() { log.append(:initialize) } } \
                  let x = Child.new(); log";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Symbol(\"base\"), Symbol(\"child\"), Symbol(\"initialize\")])"
    );
}

#[test]
fn c061_quoted_symbol_names_a_setter_selector_for_reflection() {
    // Given
    let source = "class P { property v: Nil = nil; }; \
                  let getter = Reflection::Class.method(P, :v); \
                  let setter = Reflection::Class.method(P, :\"v=\"); \
                  [getter same? getter, setter same? setter, getter same? setter]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Array([Bool(true), Bool(true), Bool(false)])");
}

#[test]
fn c148_composes_a_module_into_a_built_in_class_without_weakening_c150() {
    // Given
    let composed = "module Marker { public fun marker() { :nil } }; \
                    open class Nil mixin Marker { }; \
                    open class Bool { public fun mark() { :bool } } \
                    open class Integer { public fun mark() { :integer } } \
                    let n = 1; [nil.marker(), true.mark(), n.mark(), nil same? nil]";
    let state = "open class Integer { \
                 public property fun px=(value: Integer) -> Integer { @x = value } }; \
                 let n = 1; n.px = 2";

    // When
    let composed = rendered(composed);
    let state = rendered(state);

    // Then
    assert_eq!(
        composed,
        "Array([Symbol(\"nil\"), Symbol(\"bool\"), Symbol(\"integer\"), Bool(true)])"
    );
    assert!(state.contains("InstanceState"));
}

#[test]
fn c028_is_tests_current_runtime_ancestry_not_static_declaration() {
    // Given
    let before = "class A { } class N { } let a = A.new(); a is N";
    let after = "class A { } class N { } let a = A.new(); \
                 let committed = Reflection::Class.set_superclass(A, N); a is N";

    // When
    let before = rendered(before);
    let after = rendered(after);

    // Then
    assert_eq!(before, "Bool(false)");
    assert_eq!(after, "Bool(true)");
}

#[test]
fn c009_every_value_is_an_object_and_unrelated_classes_are_not() {
    // Given
    let source = "class A { } class B { } let a = A.new(); \
                  [a is A, a is B, a is Object, 1 is Integer, 1 is Object, nil is Nil]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(true), Bool(true), Bool(true), Bool(true)])"
    );
}

#[test]
fn c076_a_type_object_is_interned_and_distinct_from_its_class() {
    // Given
    let source = "class A { }; [A.type same? A.type, A.type same? A]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Array([Bool(true), Bool(false)])");
}

#[test]
fn c079_subtype_uses_the_same_ancestry_that_is_consults() {
    // Given
    let source = "class A { } class B extends A { } class C { } \
                  [B.type.subtype?(A.type), A.type.subtype?(B.type), \
                   C.type.subtype?(A.type), A.type.subtype?(Object.type)]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(false), Bool(true)])"
    );
}

#[test]
fn c076_contract_objects_carry_identity_distinct_from_classes_and_modules() {
    // Given
    let source = "contract C { } contract D { } class A { } module M { } \
                  [C same? C, C same? D, A same? A, M same? M]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(true), Bool(true)])"
    );
}

#[test]
fn c043_contract_inheritance_records_parents_without_an_implementation_mro() {
    // Given
    let source = "contract Parent { } contract Child extends Parent { } [Child same? Child, Child same? Parent]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Array([Bool(true), Bool(false)])");
}

#[test]
fn c024_rejects_a_duplicate_selector_inside_one_class_body() {
    // Given
    let duplicate = "class A { public fun f(v) { :i } public fun f(v) { :j } }";
    let distinct = "class A { public fun f(v) { :i } public fun g(v) { :j } }; :ok";

    // When
    let duplicate = evaluate(duplicate);
    let distinct = rendered(distinct);

    // Then
    assert!(matches!(
        duplicate,
        Err(crate::EvaluationError::Class(
            iris_runtime::ClassError::OverrideRequired { .. }
        ))
    ));
    assert_eq!(distinct, "Symbol(\"ok\")");
}

#[test]
fn c024_publishes_no_class_when_a_declaration_is_rejected() {
    // Given
    let source = "class A { public fun f(v) { :i } public fun f(v) { :j } }";

    // When
    let (outcome, published) = crate::evaluate_with_class_publication(source, "A");

    // Then
    assert!(outcome.is_err());
    assert!(!published);
}
