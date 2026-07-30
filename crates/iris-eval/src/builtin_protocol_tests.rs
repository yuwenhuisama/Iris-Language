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

#[test]
fn c088_an_ordinary_object_has_a_stable_identity_hash() {
    // Given
    let stable = "class A { } let a = A.new(); let before = a.hash(); \
                  let others = [A.new(), A.new(), A.new()]; let after = a.hash(); before == after";
    let distinct = "class A { } let a = A.new(); let b = A.new(); a.hash() == b.hash()";
    // C088 must not disturb the specification-stable numeric hash.
    let numeric = "Integer(1).hash()";

    // When
    let stable = rendered(stable);
    let distinct = rendered(distinct);
    let numeric = rendered(numeric);

    // Then
    assert_eq!(stable, "Bool(true)");
    assert_eq!(distinct, "Bool(false)");
    assert_eq!(numeric, "Integer(IntegerValue(17824117788395916856))");
}

#[test]
fn c043_while_tests_before_each_iteration_and_break_carries_the_loop_result() {
    // Given
    let zero = "while false { 1 }";
    let counted = "let mut n = 0; while n < 3 { n = n + 1 }; n";
    let broke = "while true { break 7 }";
    let bare = "while true { break }";

    // When
    let zero = rendered(zero);
    let counted = rendered(counted);
    let broke = rendered(broke);
    let bare = rendered(bare);

    // Then
    assert_eq!(zero, "Nil");
    // `while` is a STATEMENT, not an expression, so a multi-statement program
    // returns every statement value and the count arrives last.
    assert!(counted.ends_with("Integer(IntegerValue(3))])"));
    assert_eq!(broke, "Integer(IntegerValue(7))");
    assert_eq!(bare, "Nil");
}

#[test]
fn c044_for_iterates_until_done_and_c046_closes_the_iterator() {
    // Given
    let source = "let mut log = []; let mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n == 1 { Iteration.yield(nil) } else { Iteration.done } } \
                  public fun close() { log.append(:closed); nil } } \
                  class Src { public fun iterator() { It.new() } } \
                  for x in Src.new() { log.append(:body) }; log";

    // When
    let result = rendered(source);

    // Then a yielded nil still runs the body once, and close fires on exit.
    assert!(result.ends_with("Array([Symbol(\"body\"), Symbol(\"closed\")])])"));
}

#[test]
fn c044_each_iteration_binds_in_a_fresh_scope() {
    // Given
    let source = "let mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n < 3 { Iteration.yield(n) } else { Iteration.done } } \
                  public fun close() { nil } } \
                  class Src { public fun iterator() { It.new() } } \
                  let mut first = nil; let mut second = nil; \
                  for x in Src.new() { if first == nil { first = { x } } else { second = { x } } }; \
                  [first.call(), second.call()]";

    // When
    let result = rendered(source);

    // Then escaped Closures return distinct per-iteration values, not a shared cell.
    assert!(result.ends_with("Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))])])"));
}

#[test]
fn c048_labels_target_the_named_loop_and_bare_control_targets_the_nearest() {
    // Given
    let labelled = "outer: while true { while true { break outer: 7 } }";
    let nearest = "let mut n = 0; outer: while n < 2 { n = n + 1; while true { break } }; n";
    let skipped = "let mut n = 0; let mut log = []; \
                   while n < 3 { n = n + 1; continue; log.append(:unreachable) }; log";

    // When
    let labelled = rendered(labelled);
    let nearest = rendered(nearest);
    let skipped = rendered(skipped);

    // Then
    assert_eq!(labelled, "Integer(IntegerValue(7))");
    // A bare break leaves only the inner loop, so the outer one still completes.
    assert!(nearest.ends_with("Integer(IntegerValue(2))])"));
    // continue starts the next iteration, so the rest of the body never runs.
    assert!(skipped.ends_with("Array([])])"));
}

#[test]
fn c050_match_tests_arms_in_source_order_with_no_fallthrough() {
    // Given
    let first = "let x = 1; match x { 1 => :one, 2 => :two, else => :other }";
    let second = "let x = 2; match x { 1 => :one, 2 => :two, else => :other }";
    let fallback = "let x = 9; match x { 1 => :one, 2 => :two, else => :other }";
    // C053: `_` discards and creates no binding, and a binding pattern binds.
    let binding = "let x = 5; match x { found => found }";

    // When
    let first = rendered(first);
    let second = rendered(second);
    let fallback = rendered(fallback);
    let binding = rendered(binding);

    // Then
    assert_eq!(first, "Symbol(\"one\")");
    assert_eq!(second, "Symbol(\"two\")");
    assert_eq!(fallback, "Symbol(\"other\")");
    assert_eq!(binding, "Integer(IntegerValue(5))");
}

#[test]
fn c045_for_destructuring_binds_or_raises_pattern_match_error() {
    // Given
    let matched = "let mut n = 0; \
                   class It { public fun next() { n = n + 1; \
                   if n < 2 { Iteration.yield([1, 2]) } else { Iteration.done } } \
                   public fun close() { nil } } \
                   class Src { public fun iterator() { It.new() } } \
                   let mut got = nil; for [a, b] in Src.new() { got = a }; got";
    let mismatched = "let mut n = 0; \
                      class It { public fun next() { n = n + 1; \
                      if n < 2 { Iteration.yield([1, 2, 3]) } else { Iteration.done } } \
                      public fun close() { nil } } \
                      class Src { public fun iterator() { It.new() } } \
                      for [a, b] in Src.new() { a }";

    // When
    let matched = rendered(matched);
    let mismatched = rendered(mismatched);

    // Then
    assert!(matched.ends_with("Integer(IntegerValue(1))])"));
    assert_eq!(mismatched, "PatternMatchError");
}

#[test]
fn c056_and_c057_give_each_raise_a_context_with_an_optional_cause() {
    // Given
    let value = "try { raise :x } catch e: Symbol, c { c.value }";
    let unchained = "try { raise :x from nil } catch e: Symbol, c { c.cause }";
    let bad_cause = "try { raise :x from :plain } catch e: Symbol, c { c }";
    // C058: a bare raise continues the current propagation, and outside a catch
    // extent there is nothing to continue.
    let no_active = "raise";

    // When
    let value = rendered(value);
    let unchained = rendered(unchained);
    let bad_cause = rendered(bad_cause);
    let no_active = rendered(no_active);

    // Then
    assert_eq!(value, "Symbol(\"x\")");
    assert_eq!(unchained, "Nil");
    assert!(bad_cause.contains("Type"));
    assert_eq!(no_active, "NoActiveExceptionError");
}

#[test]
fn c064_and_c047_chain_a_finally_raise_and_append_a_cleanup_failure() {
    // Given
    let chained = "try { try { raise :old } finally { raise :new } } \
                   catch e: Symbol, c { e }";
    let suppressed = "let mut n = 0; \
                      class It { public fun next() { n = n + 1; \
                      if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                      public fun close() { raise :cleanup } } \
                      class Src { public fun iterator() { It.new() } } \
                      try { for x in Src.new() { raise :primary } } catch e: Symbol, c { e }";
    // D-161: a cause edge may not close a cycle.
    let cycle = "try { raise :x } catch e: Symbol, c { raise :x from c }";

    // When
    let chained = rendered(chained);
    let suppressed = rendered(suppressed);
    let cycle = rendered(cycle);

    // Then the finally raise is primary, and a cleanup failure does not displace
    // the primary propagation.
    assert_eq!(chained, "Symbol(\"new\")");
    assert_eq!(suppressed, "Symbol(\"primary\")");
    assert_eq!(cycle, "ExceptionChainError");
}

#[test]
fn c023_binds_each_declared_parameter_category_by_its_own_rule() {
    // Given
    let default_applies = "class A { public fun m(a, b = 2) { b } } A.new().m(1)";
    let default_overridden = "class A { public fun m(a, b = 2) { b } } A.new().m(1, 9)";
    let rest_collects = "class A { public fun m(a, *r) { r } } A.new().m(1, 2, 3)";
    let rest_may_be_empty = "class A { public fun m(a, *r) { r } } A.new().m(1)";
    // A keyword parameter binds by NAME, so a positional argument must not fill
    // it and a duplicate is an ArgumentError under D-357.
    let keyword_binds_by_name = "class A { public fun m(key n) { n } } A.new().m(n: 5)";
    let keyword_not_positional = "class A { public fun m(key n) { n } } A.new().m(5)";
    let duplicate_keyword = "class A { public fun m(key n) { n } } A.new().m(n: 1, n: 2)";
    let required_missing = "class A { public fun m(a) { a } } A.new().m()";

    // When / Then
    assert_eq!(rendered(default_applies), "Integer(IntegerValue(2))");
    assert_eq!(rendered(default_overridden), "Integer(IntegerValue(9))");
    assert_eq!(
        rendered(rest_collects),
        "Array([Integer(IntegerValue(2)), Integer(IntegerValue(3))])"
    );
    assert_eq!(rendered(rest_may_be_empty), "Array([])");
    assert_eq!(rendered(keyword_binds_by_name), "Integer(IntegerValue(5))");
    assert_eq!(rendered(keyword_not_positional), "ArgumentError");
    assert_eq!(rendered(duplicate_keyword), "ArgumentError");
    assert_eq!(rendered(required_missing), "ArgumentError");
}

#[test]
fn initialize_does_not_collide_with_the_add_native_selector() {
    // Given a Class declaring `+`, whose NativeSelector id was also the id
    // construction dispatches for `initialize`. The collision made `V.new()`
    // invoke `+` with no arguments.
    let construct = "class V { public fun +(o) { 1 } } V.new()";
    let operator = "class V { public fun +(o) -> Symbol { :plus } } V.new() + V.new()";

    // When / Then
    assert_eq!(rendered(construct), "Object(ObjectId(0))");
    assert_eq!(rendered(operator), "Symbol(\"plus\")");
}

#[test]
fn c027_hash_literals_build_a_hash_and_c134_still_rejects_a_nan_key() {
    // Given
    let empty = "%{}";
    let entries = "%{ 1: 2, 3: 4 }";
    // C028 dispatches the key's current `==`, so a repeated key UPDATES its
    // entry rather than adding a second one.
    let repeated_key = "%{ 1: 2, 1: 9 }";
    let nan_key = "%{ Float64.nan: 1 }";

    // When / Then
    assert_eq!(rendered(empty), "Hash([])");
    assert_eq!(
        rendered(entries),
        "Hash([(Integer(IntegerValue(1)), Integer(IntegerValue(2))), \
         (Integer(IntegerValue(3)), Integer(IntegerValue(4)))])"
    );
    assert_eq!(
        rendered(repeated_key),
        "Hash([(Integer(IntegerValue(1)), Integer(IntegerValue(9)))])"
    );
    assert_eq!(rendered(nan_key), "Runtime(StableHash(InvalidNumericKey))");
}

#[test]
fn c023_keyword_rest_collects_the_unmatched_keywords_into_a_hash() {
    // Given a call supplying one keyword a parameter names and one it does not.
    let source = "class A { public fun m(key k, **kw) { [k, kw] } } A.new().m(k: 5, z: 6)";
    let none_left_over = "class A { public fun m(key k, **kw) { kw } } A.new().m(k: 5)";

    // When / Then
    assert_eq!(
        rendered(source),
        "Array([Integer(IntegerValue(5)), Hash([(Symbol(\"z\"), Integer(IntegerValue(6)))])])"
    );
    assert_eq!(rendered(none_left_over), "Hash([])");
}

#[test]
fn c036_compound_assignment_applies_its_operator_to_the_read_value() {
    // Given each supported symbolic compound assignment. D-348 stresses that
    // none is an independent selector, so each must send its ORDINARY operator
    // to the value read from the target.
    let add = "let mut a = 1; a += 2; a";
    let subtract = "let mut a = 8; a -= 3; a";
    let multiply = "let mut a = 10; a *= 3; a";
    let shift = "let mut a = 1; a <<= 4; a";

    // When / Then the result is the operation applied to the old value, not the
    // right-hand side written over the top of it.
    assert_eq!(
        rendered(add),
        "Array([Integer(IntegerValue(3)), Integer(IntegerValue(3))])"
    );
    assert_eq!(
        rendered(subtract),
        "Array([Integer(IntegerValue(5)), Integer(IntegerValue(5))])"
    );
    assert_eq!(
        rendered(multiply),
        "Array([Integer(IntegerValue(30)), Integer(IntegerValue(30))])"
    );
    assert_eq!(
        rendered(shift),
        "Array([Integer(IntegerValue(16)), Integer(IntegerValue(16))])"
    );
}

#[test]
fn c036_index_assignment_evaluates_receiver_index_and_rhs_exactly_once() {
    // Given a side-effectful index and right-hand side, each counting its own
    // evaluations. D-347 requires exactly one evaluation of each.
    let counted = "let mut index_calls = 0; let mut rhs_calls = 0; let mut a = [1, 2]; \
                   class C { public fun index() { index_calls = index_calls + 1; 0 } \
                   public fun rhs() { rhs_calls = rhs_calls + 1; 5 } } \
                   let c = C.new(); a[c.index()] += c.rhs(); [a[0], index_calls, rhs_calls]";
    // A missing Hash key and an out-of-range Array index read `nil` rather than
    // raising, and a written value must survive into the next read.
    let array_round_trip = "let mut a = [1, 2]; a[0] = 9; a[0]";
    let hash_round_trip = "let mut h = %{ 1: 2 }; h[7] = 3; h[7]";
    let absent = "let h = %{ 1: 2 }; [h[9], [1, 2][9]]";

    // When / Then
    assert!(rendered(counted).ends_with(
        "Array([Integer(IntegerValue(6)), Integer(IntegerValue(1)), Integer(IntegerValue(1))])])"
    ));
    assert_eq!(
        rendered(array_round_trip),
        "Array([Integer(IntegerValue(9)), Integer(IntegerValue(9))])"
    );
    assert_eq!(
        rendered(hash_round_trip),
        "Array([Integer(IntegerValue(3)), Integer(IntegerValue(3))])"
    );
    assert_eq!(rendered(absent), "Array([Nil, Nil])");
}

#[test]
fn c047_binds_a_context_reporting_the_primary_not_the_cleanup_failure() {
    // Given a body failure and a cleanup failure. The earlier C047 test read
    // only the caught VALUE, which cannot tell the two apart: a `close` that
    // raises installs its own context, and reading the value alone would still
    // report `:body` while the bound context reported `:close`.
    let source = "let mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                  public fun close() { raise :close } } \
                  class Src { public fun iterator() { It.new() } } \
                  try { for x in Src.new() { raise :body } } \
                  catch e, c { [e, c.value, c.suppressed] }";
    // A cleanup that SUCCEEDS must leave the primary context untouched.
    let clean = "let mut n = 0; \
                 class It { public fun next() { n = n + 1; \
                 if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                 public fun close() { nil } } \
                 class Src { public fun iterator() { It.new() } } \
                 try { for x in Src.new() { raise :body } } catch e, c { [e, c.value] }";

    // When / Then the caught value and its bound context AGREE on the primary,
    // and the cleanup failure appears only in `suppressed`.
    // The identity is elided here: C056 makes it distinct per event, so
    // asserting a specific ObjectId would pin an allocation order the clause
    // does not promise.
    let payload = rendered(source);
    assert!(
        payload.starts_with("Array([Symbol(\"body\"), Symbol(\"body\"), Array([ExceptionContext("),
        "unexpected payload: {payload}"
    );
    assert!(
        payload.ends_with("Symbol(\"close\"), Nil, [])])])"),
        "unexpected payload: {payload}"
    );
    assert_eq!(
        rendered(clean),
        "Array([Symbol(\"body\"), Symbol(\"body\")])"
    );
}

#[test]
fn c056_gives_every_propagation_event_a_distinct_identity() {
    // Given a re-raise of the CAUGHT value. The payload is identical, so a
    // structurally compared context would wrongly report the same event.
    let reraised = "try { raise :same } catch value, first { \
                    try { raise value } catch _, second { \
                    [second same? first, second.value same? first.value] } }";
    // A context is still `same?` itself, so the identity is stable rather than
    // merely always-unequal.
    let reflexive = "try { raise :x } catch _, c { c same? c }";
    // An explicit cause links to the captured context without becoming it.
    let chained = "try { try { raise :first } catch _, f { raise :second from f } } \
                   catch _, s { [s.value, s.cause.value, s.cause same? s] }";

    // When / Then
    assert_eq!(rendered(reraised), "Array([Bool(false), Bool(true)])");
    assert_eq!(rendered(reflexive), "Bool(true)");
    assert_eq!(
        rendered(chained),
        "Array([Symbol(\"second\"), Symbol(\"first\"), Bool(false)])"
    );
}

#[test]
fn a_non_terminating_program_is_reported_rather_than_hanging_the_suite() {
    // Given the two ways a run fails to terminate. The first is the exact
    // source that hung the conformance suite: the inner `break` leaves only the
    // inner loop, so the outer `while true` reruns forever.
    let unbounded_loop = "outer: while true { while true { break 9 } }";
    let bare_loop = "while true { 1 }";
    // Unbounded recursion needs its own bound: each frame is a host stack
    // frame, so a step budget large enough for ordinary loops would abort the
    // process on stack overflow before ever being exhausted.
    let unbounded_recursion = "class C { public fun f() { self.f() } } C.new().f()";

    // When / Then each is reported as evidence instead of stalling.
    assert_eq!(rendered(unbounded_loop), "StepBudgetExhausted");
    assert_eq!(rendered(bare_loop), "StepBudgetExhausted");
    assert_eq!(rendered(unbounded_recursion), "StepBudgetExhausted");
}

#[test]
fn the_execution_bounds_admit_ordinary_loops_and_recursion() {
    // Given work far larger than any committed vector performs.
    let counted = "let mut i = 0; while i < 1000 { i = i + 1 }; i";
    // `for` needs a scripted iterator: an Array is not itself iterable here.
    let iterated = "let mut n = 0; let mut total = 0; \
                    class It { public fun next() { n = n + 1; \
                    if n <= 3 { Iteration.yield(n) } else { Iteration.done } } \
                    public fun close() { nil } } \
                    class Src { public fun iterator() { It.new() } } \
                    for x in Src.new() { total = total + x }; total";
    let recursive = "class C { public fun f(n) { if n <= 0 { 0 } else { self.f(n - 1) } } } \
                     C.new().f(12)";
    // The labeled form the hang was mistranscribed FROM still terminates.
    let labeled = "outer: while true { while true { break outer: 7 } }";

    // When / Then the bounds never fire.
    assert!(rendered(counted).ends_with("Integer(IntegerValue(1000))])"));
    assert!(rendered(iterated).ends_with("Integer(IntegerValue(6))])"));
    assert_eq!(rendered(recursive), "Integer(IntegerValue(0))");
    assert_eq!(rendered(labeled), "Integer(IntegerValue(7))");
}

#[test]
fn c011_makes_an_array_iterable_through_the_ordinary_iterator_protocol() {
    // Given `for` over an Array, which C012 drives through iterator()/next().
    let summed = "let mut total = 0; for x in [1, 2, 3] { total = total + x }; total";
    let empty = "let mut count = 0; for x in [] { count = count + 1 }; count";
    // Each iterator() call must allocate an INDEPENDENT cursor, or a nested
    // traversal of the same Array would share one position and stop early.
    let nested = "let a = [1, 2]; let mut count = 0; \
                  for x in a { for y in a { count = count + 1 } }; count";
    // C013 returns the same done singleton on every call after exhaustion.
    let exhausted = "let i = [1].iterator(); [i.next(), i.next(), i.next()]";

    // When / Then
    assert!(rendered(summed).ends_with("Integer(IntegerValue(6))])"));
    assert!(rendered(empty).ends_with("Integer(IntegerValue(0))])"));
    assert!(rendered(nested).ends_with("Integer(IntegerValue(4))])"));
    assert_eq!(
        rendered(exhausted),
        "Array([IterationYield(Integer(IntegerValue(1))), IterationDone, IterationDone])"
    );
}

#[test]
fn c028_lets_a_nested_block_shadow_a_captured_binding_without_replacing_it() {
    // Given a Closure capturing `value`, then a SEPARATE Closure that declares
    // its own `value`. The inner declaration must shadow, not overwrite: the
    // Closure body previously ran its statements directly against the enclosing
    // binding map, so the inner `let` replaced the outer name and the captured
    // Closure observed 2 instead of 1.
    let shadowed = "let value = 1; let c = { value }; \
                    let inner = { let value = 2; value }; [inner.call(), c.call()]";
    // Capture by REFERENCE is unaffected: a Closure that assigns the captured
    // binding is still seen by the enclosing scope.
    let shared_cell = "let mut v = 1; let c = { v = v + 1; v }; let a = c.call(); [a, v + 3]";
    // Each loop iteration owns a fresh cell, so two escaping Closures differ.
    let per_iteration = "let mut fs = []; for x in [1, 2] { fs.append({ x }) }; \
                         [fs[0].call(), fs[1].call()]";

    // When / Then
    assert_eq!(
        rendered(shadowed),
        "Array([Integer(IntegerValue(2)), Integer(IntegerValue(1))])"
    );
    assert_eq!(
        rendered(shared_cell),
        "Array([Integer(IntegerValue(2)), Integer(IntegerValue(5))])"
    );
    assert!(
        rendered(per_iteration)
            .ends_with("Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))])])")
    );
}

#[test]
fn d421_returns_to_the_nearest_callable_boundary_only() {
    // Given a `return` inside a Closure inside a Method. D-421 ends only that
    // CLOSURE invocation: v1 has no nonlocal-return Closure, so the Method
    // continues and returns its own final expression.
    let closure_boundary =
        "class C { public fun m() { let c = { return 4 }; let a = c.call(); [a, 5] } } C.new().m()";
    let method_return = "class C { public fun m() { return 1; 2 } } C.new().m()";
    let bare_return = "class C { public fun m() { return } } C.new().m()";
    // A `return` inside a loop still leaves the Method, not just the loop.
    let from_loop = "class C { public fun m() { let mut i = 0; while i < 5 { i = i + 1; return i } } } \
         C.new().m()";

    // When / Then
    assert_eq!(
        rendered(closure_boundary),
        "Array([Integer(IntegerValue(4)), Integer(IntegerValue(5))])"
    );
    assert_eq!(rendered(method_return), "Integer(IntegerValue(1))");
    assert_eq!(rendered(bare_return), "Nil");
    assert_eq!(rendered(from_loop), "Integer(IntegerValue(1))");
}

#[test]
fn a_block_local_mut_binding_is_assignable_and_does_not_escape() {
    // Given `let mut` inside a Method body. Block locals were stored as plain
    // values with no mutability, so ANY assignment to one was rejected as an
    // immutable-binding write and no Method could use a mutable local.
    let assigned = "class C { public fun m() { let mut i = 0; i = 1; i } } C.new().m()";
    let counted = "class C { public fun m() { let mut i = 0; while i < 3 { i = i + 1 }; i } } \
                   C.new().m()";
    // The binding is block-local, so it must not remain visible afterwards.
    let escaped = "class C { public fun m() { let mut i = 0; i } } let r = C.new().m(); i";

    // When / Then
    assert_eq!(rendered(assigned), "Integer(IntegerValue(1))");
    assert_eq!(rendered(counted), "Integer(IntegerValue(3))");
    // C011 makes an unresolved bare name a NameError.
    assert_eq!(rendered(escaped), "NameError");
}

#[test]
fn a_try_in_expression_position_yields_the_clause_that_supplied_the_result() {
    // Given `try` read as a value. It was statement-only, so none of these
    // parsed at all.
    let body_supplies = "let result = try { :try_value } finally { :finally_value }; result";
    let catch_supplies = "let result = try { raise :x } catch _: Symbol { :caught }; result";
    // A normally completing `finally` must NOT replace the provisional value.
    let finally_does_not_replace = "let a = try { 1 } finally { 2 }; let b = try { raise :x } catch _ { 3 } finally { 4 }; \
         [a, b]";
    // Clause order is observable, and the handler result is the value.
    let ordered = "let mut log = []; \
                   let r = try { log.append(:try); raise :x } \
                   catch _ { log.append(:catch); :handled } \
                   finally { log.append(:finally) }; [log, r]";

    // When / Then
    assert_eq!(rendered(body_supplies), "Symbol(\"try_value\")");
    assert_eq!(rendered(catch_supplies), "Symbol(\"caught\")");
    assert_eq!(
        rendered(finally_does_not_replace),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(3))])"
    );
    assert_eq!(
        rendered(ordered),
        "Array([Array([Symbol(\"try\"), Symbol(\"catch\"), Symbol(\"finally\")]), \
         Symbol(\"handled\")])"
    );
}

#[test]
fn d159_makes_the_exception_context_payload_read_only() {
    // Given a captured context. The payload is readable, and every setter is
    // rejected rather than falling through to a generic missing-message error.
    let read = "try { raise :x } catch _, c { c.value }";
    let written = "try { raise :x } catch _, c { c.value = :other }";
    // The context stays usable after its catch and can chain a later raise.
    let outlives_catch = "let saved = try { raise :x } catch _, c { c }; \
                          let read = saved.value; \
                          try { raise :next from saved } catch _, n { [read, n.cause same? saved] }";

    // When / Then
    assert_eq!(rendered(read), "Symbol(\"x\")");
    assert_eq!(rendered(written), "ReadonlyProperty");
    assert_eq!(
        rendered(outlives_catch),
        "Array([Symbol(\"x\"), Bool(true)])"
    );
}

#[test]
fn c043_gives_a_loop_a_value_in_expression_position() {
    // Given a loop read as a value. C043 yields `nil` on natural completion,
    // including zero iterations, and the `break` operand otherwise.
    let natural = "let a = while false { 1 }; a";
    let broken = "let b = while true { break 7 }; b";
    let bare_break = "let b = while true { break }; b";
    // The statement spelling must keep working, since both run one evaluator.
    let as_statement = "let mut i = 0; while i < 3 { i = i + 1 }; i";

    // When / Then
    assert_eq!(rendered(natural), "Nil");
    assert_eq!(rendered(broken), "Integer(IntegerValue(7))");
    assert_eq!(rendered(bare_break), "Nil");
    assert!(rendered(as_statement).ends_with("Integer(IntegerValue(3))])"));
}

#[test]
fn c011_raises_name_error_for_an_unresolved_bare_name() {
    // Given a branch-local binding read from outside its branch. C011 requires
    // NameError and forbids falling back to a property, Method, or global.
    let escaped_branch = "let r = if true { let inside = 1; inside }; inside";
    let never_declared = "undefined_name";
    // A resolvable name is unaffected.
    let resolved = "let a = 1; a";

    // When / Then
    assert_eq!(rendered(escaped_branch), "NameError");
    assert_eq!(rendered(never_declared), "NameError");
    assert_eq!(rendered(resolved), "Integer(IntegerValue(1))");
}

#[test]
fn a_rejected_operator_does_not_degrade_into_a_bare_name() {
    // Given `%`, which IRIS-V1-RUNTIME-V102 requires the GRAMMAR to reject.
    // Any leftover token used to become a Name, so `5 % 2` parsed as three
    // separate statements and the rejection never happened; the row passed
    // only because the stray name failed later for an unrelated reason.
    let rejected = "let n = 5; n % 2";
    // The named-infix spelling is the supported one and must still work.
    let named_infix = "let n = 5; [n.mod(2), n mod 2]";

    // When / Then
    assert_eq!(rendered(rejected), "ParseDiagnostic");
    assert_eq!(
        rendered(named_infix),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(1))])"
    );
}
