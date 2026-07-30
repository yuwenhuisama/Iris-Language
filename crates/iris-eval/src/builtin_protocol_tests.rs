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
fn d093_and_d094_separate_a_protocol_violation_from_a_return_type_violation() {
    // Given
    let integer = "class B { public fun <=>(o) { 7 } }; let x = B.new(); let y = B.new(); x < y";
    let symbol = "class C { public fun <=>(o) { :sym } }; let x = C.new(); let y = C.new(); x == y";
    let boolean =
        "class D { public fun <=>(o) { true } }; let x = D.new(); let y = D.new(); x == y";

    // When
    let integer = rendered(integer);
    let symbol = rendered(symbol);
    let boolean = rendered(boolean);

    // Then
    assert_eq!(integer, "ComparisonContractError");
    assert!(symbol.contains("Type"));
    assert!(boolean.contains("Type"));
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

#[test]
fn c049_qualified_and_ordinary_selector_namespaces_stay_separate() {
    // Given
    let source = "contract C { } class A for C { impl fun C::m() { :qualified } \
                  public fun m() { :ordinary } } let a = A.new(); [(a as C)..m(), a.m()]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Symbol(\"qualified\"), Symbol(\"ordinary\")])"
    );
}

#[test]
fn c049_rejects_a_view_the_receiver_never_declared_with_for() {
    // Given
    let source = "contract C { } class B { public fun m() { :ordinary } } \
                  let b = B.new(); (b as C)..m()";

    // When
    let result = rendered(source);

    // Then
    assert!(result.contains("Type"));
}

#[test]
fn c048_a_qualified_impl_is_unreachable_through_ordinary_dispatch() {
    // Given
    let source = "contract C { } class A for C { impl fun C::only() { :qualified } } \
                  let a = A.new(); a.only()";

    // When
    let result = rendered(source);

    // Then
    assert!(result.contains("MessageNotFound"));
}

#[test]
fn c042_each_closure_evaluation_creates_a_distinct_identity() {
    // Given
    let source = "let mk = { { :v } }; let a = mk.call(); let b = mk.call(); \
                  [a same? a, a same? b, a == a, a == b]";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(
        result,
        "Array([Bool(true), Bool(false), Bool(true), Bool(false)])"
    );
}

#[test]
fn c072_an_escaped_closure_keeps_writing_its_captured_receiver() {
    // Given
    let source = "class A { public property fun x() { @x } \
                  public property fun x=(v) { @x = v } \
                  public fun mk() { { @x = 5 } } } \
                  let a = A.new(); let escaped = a.mk(); let ran = escaped.call(); a.x";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Integer(IntegerValue(5))");
}

#[test]
fn c099_passes_a_trailing_block_as_the_separate_block_parameter() {
    // Given
    let with_block = "class A { public fun method_missing(s, a, b) { [s, a, b.call()] } } \
                      let o = A.new(); o.missing(1) { :block }";
    let without = "class A { public fun method_missing(s, a, b) { [s, a] } } \
                   let o = A.new(); o.missing(1)";

    // When
    let with_block = rendered(with_block);
    let without = rendered(without);

    // Then
    assert_eq!(
        with_block,
        "Array([Symbol(\"missing\"), Array([Integer(IntegerValue(1))]), Symbol(\"block\")])"
    );
    assert_eq!(
        without,
        "Array([Symbol(\"missing\"), Array([Integer(IntegerValue(1))])])"
    );
}

#[test]
fn a_local_callable_shadows_a_self_send_without_recursing() {
    // Given
    let called = "class A { public fun m(&b) { b.call() } } let o = A.new(); o.m({ :v })";
    // A nil block must fail cleanly rather than recurse through method_missing.
    let nil_block = "class A { public fun method_missing(s, a, b) { b.call() } } \
                     let o = A.new(); o.missing(1)";

    // When
    let called = rendered(called);
    let nil_block = rendered(nil_block);

    // Then
    assert_eq!(called, "Symbol(\"v\")");
    // C025 binds an omitted block to nil, and nil answers no `call` selector.
    assert!(nil_block.contains("MessageNotFound"));
}

#[test]
fn c076_call_is_the_sole_invocation_spelling() {
    // Given
    let closure = "let c = { :v }; c.call()";
    let bound = "class A { public fun m() { :x } } let o = A.new(); o.m.call()";
    let block = "class A { public fun m(&b) { b.call() } } let o = A.new(); o.m({ :blk })";
    let direct = "let c = { :v }; c()";

    // When
    let closure = rendered(closure);
    let bound = rendered(bound);
    let block = rendered(block);
    let direct = rendered(direct);

    // Then
    assert_eq!(closure, "Symbol(\"v\")");
    assert_eq!(bound, "Symbol(\"x\")");
    assert_eq!(block, "Symbol(\"blk\")");
    assert_eq!(direct, "UnsupportedConstruct");
}

#[test]
fn c023_block_parameter_carries_a_function_type_annotation() {
    // Given
    let annotated = "class A { public fun m(&b: Block<(Integer) -> Symbol>) { b.call(1) } } \
                     let o = A.new(); o.m({ |x| :got })";
    let unannotated = "class A { public fun m(&b) { b.call() } } let o = A.new(); o.m({ :v })";

    // When
    let annotated = rendered(annotated);
    let unannotated = rendered(unannotated);

    // Then
    assert_eq!(annotated, "Symbol(\"got\")");
    assert_eq!(unannotated, "Symbol(\"v\")");
}

#[test]
fn c025_an_omitted_optional_block_binds_nil() {
    // Given
    let source = "class A { public fun m(&b: Block<(Integer) -> Symbol> = nil) { b } } \
                  let o = A.new(); o.m()";

    // When
    let result = rendered(source);

    // Then
    assert_eq!(result, "Nil");
}

#[test]
fn c094_and_c095_require_a_callable_annotation_to_name_its_kind() {
    // Given
    let block = "class A { public fun m(&b: Block<(Integer) -> Symbol>) { b.call(1) } } \
                 let o = A.new(); o.m({ |x| :got })";
    let closure = "class A { public fun m(&b: Closure<(Integer) -> Symbol>) { b.call(1) } } \
                   let o = A.new(); o.m({ |x| :c })";
    // C094 makes the bare signature not a Type on its own.
    let bare = "class A { public fun m(&b: (Integer) -> Symbol) { b.call(1) } } \
                let o = A.new(); o.m({ |x| :x })";

    // When
    let block = rendered(block);
    let closure = rendered(closure);
    let bare = rendered(bare);

    // Then
    assert_eq!(block, "Symbol(\"got\")");
    assert_eq!(closure, "Symbol(\"c\")");
    assert_eq!(bare, "ParseDiagnostic");
}

#[test]
fn c037_logical_assignment_truth_tests_before_evaluating_the_right_side() {
    // Given
    let raises = "class P { public fun to_bool() -> Bool { raise :sentinel } } \
                  let mut x = P.new(); x &&= 1";
    let skipped = "let mut log = []; let mut x = nil; let r = x &&= log.append(:ran); log";
    let written = "let mut x = true; let r = x &&= 5; x";

    // When
    let raises = rendered(raises);
    let skipped = rendered(skipped);
    let written = rendered(written);

    // Then
    assert_eq!(raises, "Raised(Symbol(\"sentinel\"))");
    // C037: a no-write path must not evaluate the right side at all.
    assert_eq!(skipped, "Array([])");
    assert_eq!(written, "Integer(IntegerValue(5))");
}

#[test]
fn c037_or_assignment_writes_only_on_the_falsy_path() {
    // Given
    let written = "let mut x = nil; let r = x ||= 7; x";
    let skipped = "let mut log = []; let mut x = true; let r = x ||= log.append(:ran); log";

    // When
    let written = rendered(written);
    let skipped = rendered(skipped);

    // Then
    assert_eq!(written, "Integer(IntegerValue(7))");
    assert_eq!(skipped, "Array([])");
}

#[test]
fn c134_rejects_a_nan_key_at_hash_construction() {
    // Given
    let float64 = "%{ Float64.nan: 1 }";
    let float32 = "%{ Float32.nan: 1 }";
    // A non-NaN key gets past the C134 check and fails later for another reason.
    let finite = "%{ 1: 2 }";

    // When
    let float64 = rendered(float64);
    let float32 = rendered(float32);
    let finite = rendered(finite);

    // Then
    assert!(float64.contains("InvalidNumericKey"));
    assert!(float32.contains("InvalidNumericKey"));
    assert!(!finite.contains("InvalidNumericKey"));
}

#[test]
fn c014_dynamic_entry_yields_the_value_without_widening_visibility() {
    // Given
    let public_send = "class A { public fun m() { :m } } let a = A.new(); (a as Dynamic<A>).m()";
    // D-313: Dynamic must NOT reach a private Method merely because the
    // selector exists, so the denial survives the Dynamic boundary.
    let private_send = "class A { private fun p() { :p } } let a = A.new(); (a as Dynamic<A>).p()";

    // When
    let public_send = rendered(public_send);
    let private_send = rendered(private_send);

    // Then
    assert_eq!(public_send, "Symbol(\"m\")");
    assert!(private_send.contains("VisibilityDenied"));
}
