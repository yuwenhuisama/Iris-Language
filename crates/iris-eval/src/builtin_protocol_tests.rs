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
    let source = "mut calls = []; class P { public fun <=>(o) { calls.append(:c); nil } }; \
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
    let source = "mut log = []; \
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
                  mut x = P.new(); x &&= 1";
    let skipped = "mut log = []; mut x = nil; let r = x &&= log.append(:ran); log";
    let written = "mut x = true; let r = x &&= 5; x";

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
    let written = "mut x = nil; let r = x ||= 7; x";
    let skipped = "mut log = []; mut x = true; let r = x ||= log.append(:ran); log";

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
    let counted = "mut n = 0; while n < 3 { n = n + 1 }; n";
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
    let source = "mut log = []; mut n = 0; \
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
    let source = "mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n < 3 { Iteration.yield(n) } else { Iteration.done } } \
                  public fun close() { nil } } \
                  class Src { public fun iterator() { It.new() } } \
                  mut first = nil; mut second = nil; \
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
    let nearest = "mut n = 0; outer: while n < 2 { n = n + 1; while true { break } }; n";
    let skipped = "mut n = 0; mut log = []; \
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
    let matched = "mut n = 0; \
                   class It { public fun next() { n = n + 1; \
                   if n < 2 { Iteration.yield([1, 2]) } else { Iteration.done } } \
                   public fun close() { nil } } \
                   class Src { public fun iterator() { It.new() } } \
                   mut got = nil; for [a, b] in Src.new() { got = a }; got";
    let mismatched = "mut n = 0; \
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
    let suppressed = "mut n = 0; \
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
    let add = "mut a = 1; a += 2; a";
    let subtract = "mut a = 8; a -= 3; a";
    let multiply = "mut a = 10; a *= 3; a";
    let shift = "mut a = 1; a <<= 4; a";

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
    let counted = "mut index_calls = 0; mut rhs_calls = 0; mut a = [1, 2]; \
                   class C { public fun index() { index_calls = index_calls + 1; 0 } \
                   public fun rhs() { rhs_calls = rhs_calls + 1; 5 } } \
                   let c = C.new(); a[c.index()] += c.rhs(); [a[0], index_calls, rhs_calls]";
    // A missing Hash key and an out-of-range Array index read `nil` rather than
    // raising, and a written value must survive into the next read.
    let array_round_trip = "mut a = [1, 2]; a[0] = 9; a[0]";
    let hash_round_trip = "mut h = %{ 1: 2 }; h[7] = 3; h[7]";
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
    let source = "mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                  public fun close() { raise :close } } \
                  class Src { public fun iterator() { It.new() } } \
                  try { for x in Src.new() { raise :body } } \
                  catch e, c { [e, c.value, c.suppressed] }";
    // A cleanup that SUCCEEDS must leave the primary context untouched.
    let clean = "mut n = 0; \
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
        payload.starts_with(
            "Array([Symbol(\"body\"), Symbol(\"body\"), ReadonlyArray([ExceptionContext("
        ),
        "unexpected payload: {payload}"
    );
    assert!(
        payload.contains("Symbol(\"close\")"),
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
    let counted = "mut i = 0; while i < 1000 { i = i + 1 }; i";
    // `for` needs a scripted iterator: an Array is not itself iterable here.
    let iterated = "mut n = 0; mut total = 0; \
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
    let summed = "mut total = 0; for x in [1, 2, 3] { total = total + x }; total";
    let empty = "mut count = 0; for x in [] { count = count + 1 }; count";
    // Each iterator() call must allocate an INDEPENDENT cursor, or a nested
    // traversal of the same Array would share one position and stop early.
    let nested = "let a = [1, 2]; mut count = 0; \
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
    let shared_cell = "mut v = 1; let c = { v = v + 1; v }; let a = c.call(); [a, v + 3]";
    // Each loop iteration owns a fresh cell, so two escaping Closures differ.
    let per_iteration = "mut fs = []; for x in [1, 2] { fs.append({ x }) }; \
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
    let from_loop = "class C { public fun m() { mut i = 0; while i < 5 { i = i + 1; return i } } } \
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
    let assigned = "class C { public fun m() { mut i = 0; i = 1; i } } C.new().m()";
    let counted = "class C { public fun m() { mut i = 0; while i < 3 { i = i + 1 }; i } } \
                   C.new().m()";
    // The binding is block-local, so it must not remain visible afterwards.
    let escaped = "class C { public fun m() { mut i = 0; i } } let r = C.new().m(); i";

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
    let ordered = "mut log = []; \
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
    let as_statement = "mut i = 0; while i < 3 { i = i + 1 }; i";

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

#[test]
fn c035_yields_the_setter_result_for_a_property_write() {
    // Given a property write and a binding write. C035 makes them yield
    // DIFFERENT things: a binding yields the stored value, a property yields
    // whatever its setter Method returned.
    let both_writes = "class Box { public property fun name=(value) -> Symbol { :written } } \
                       mut local = 0; let local_result = (local = 1); \
                       let property_result = (Box.new().name = 2); [local_result, property_result]";
    // Assignment is right-associative, so the inner setter runs first and the
    // outer setter receives its RESULT rather than the original operand.
    let nested = "mut log = []; \
                  class Box { public property fun name=(value) -> Symbol { log.append(:set); value } } \
                  let outer = Box.new(); let inner = Box.new(); \
                  let r = (outer.name = (inner.name = :inner)); [r, log]";

    // When / Then
    assert_eq!(
        rendered(both_writes),
        "Array([Integer(IntegerValue(1)), Symbol(\"written\")])"
    );
    assert_eq!(
        rendered(nested),
        "Array([Symbol(\"inner\"), Array([Symbol(\"set\"), Symbol(\"set\")])])"
    );
}

#[test]
fn c011_does_not_invoke_a_bare_callable_value() {
    // Given a bare `c.f` bound to a name. A call COUNTER is required here:
    // comparing two results cannot distinguish "the bare name was not invoked"
    // from "it was invoked and happened to return the same value".
    let counted = "mut calls = 0; class C { public fun f() { calls = calls + 1; 1 } } \
                   let c = C.new(); let bare = c.f; let after_bind = calls; \
                   let invoked = bare.call(); [after_bind, invoked, calls]";
    // A compound index assignment evaluates receiver, index and RHS exactly
    // once each, in that order.
    let ordered = "mut events = []; mut store = [1, 2]; \
                   class P { public fun factory() { events.append(:factory); store } \
                   public fun idx() { events.append(:index); 0 } \
                   public fun rhs() { events.append(:rhs); 5 } } \
                   let p = P.new(); p.factory()[p.idx()] += p.rhs(); events";

    // When / Then binding the callable performs no call.
    assert_eq!(
        rendered(counted),
        "Array([Integer(IntegerValue(0)), Integer(IntegerValue(1)), Integer(IntegerValue(1))])"
    );
    assert!(
        rendered(ordered)
            .ends_with("Array([Symbol(\"factory\"), Symbol(\"index\"), Symbol(\"rhs\")])])")
    );
}

#[test]
fn c047_distinguishes_a_cleanup_failure_with_and_without_a_pending_exception() {
    // Given a `close` that raises while nothing is pending. With no primary to
    // attach to, the cleanup failure IS the primary and its suppressed list is
    // empty, which is the opposite arrangement from the body-failure case.
    let cleanup_only = "mut n = 0; \
                        class It { public fun next() { n = n + 1; \
                        if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                        public fun close() { raise :close } } \
                        class Src { public fun iterator() { It.new() } } \
                        try { for x in Src.new() { nil } } catch _, c { [c.value, c.suppressed] }";
    // A `break` still runs cleanup exactly once on its way out.
    let break_closes = "mut log = []; \
                        class It { public fun next() { Iteration.yield(1) } \
                        public fun close() { log.append(:close) } } \
                        class Src { public fun iterator() { It.new() } } \
                        for x in Src.new() { break }; log";

    // When / Then
    assert_eq!(
        rendered(cleanup_only),
        "Array([Symbol(\"close\"), ReadonlyArray([])])"
    );
    assert!(rendered(break_closes).ends_with("Array([Symbol(\"close\")])])"));
}

#[test]
fn c042_calls_to_bool_once_per_tested_operand_and_short_circuits() {
    // Given one probe counting `to_bool` across negation, conjunction and
    // disjunction. The disjunction's right side ASSIGNS, so a
    // non-short-circuiting implementation is visible as a changed counter
    // rather than only as a different result.
    let counted = "mut calls = 0; mut rhs_ran = 0; \
                   class P { public fun to_bool() { calls = calls + 1; true } } \
                   let negated = !P.new(); \
                   let conjoined = P.new() && :yes; \
                   let disjoined = P.new() || (rhs_ran = 1); \
                   [negated, conjoined, calls, rhs_ran]";

    // When / Then three operands are tested, and the right side never runs.
    assert_eq!(
        rendered(counted),
        "Array([Bool(false), Symbol(\"yes\"), Integer(IntegerValue(3)), Integer(IntegerValue(0))])"
    );
}

#[test]
fn c094_makes_every_value_answer_to_bool_through_root_object() {
    // Given values with no dedicated builtin Class. C005 makes `Object` the
    // single root and C094 gives it a `to_bool` returning `true`, so these are
    // ordinary Objects rather than values without a Class. Resolving them to an
    // error made `if :sym`, `if [1]` and `if %{}` fail outright.
    let symbol = "if :sym { :yes } else { :no }";
    let array = "if [1] { :yes } else { :no }";
    let hash = "if %{} { :yes } else { :no }";
    // Only `Nil` is false and `Bool` returns itself.
    let nil = "if nil { :yes } else { :no }";
    let falsehood = "if false { :yes } else { :no }";
    // Logical assignment reads the target through the same protocol, so a
    // truthy target must SHORT-CIRCUIT rather than fail.
    let short_circuit = "mut x = :kept; x ||= :other; x";

    // When / Then
    assert_eq!(rendered(symbol), "Symbol(\"yes\")");
    assert_eq!(rendered(array), "Symbol(\"yes\")");
    assert_eq!(rendered(hash), "Symbol(\"yes\")");
    assert_eq!(rendered(nil), "Symbol(\"no\")");
    assert_eq!(rendered(falsehood), "Symbol(\"no\")");
    assert!(rendered(short_circuit).ends_with("Symbol(\"kept\")])"));
}

#[test]
fn c027_reads_a_bare_identifier_hash_key_as_a_binding() {
    // Given a bare identifier as a Hash key. C027 makes every key position an
    // ordinary expression, so `a` READS that binding and must not become an
    // implicit Symbol. The parser consumed a lone `:` while probing for a `::`
    // qualified separator, so this form failed to parse at all.
    let identifier_key = "let a = 1; let h = %{ a: 10 }; h[1]";
    // A qualified name must still parse, since the fix narrows that lookahead.
    let qualified = "class C { public fun m() { 1 } } C.new().m()";

    // When / Then the key is the BOUND value, not the symbol `:a`: reading the
    // Hash at `1` finds the entry, which an implicit `:a` key could not.
    assert_eq!(rendered(identifier_key), "Integer(IntegerValue(10))");
    assert_eq!(rendered(qualified), "Integer(IntegerValue(1))");
}

#[test]
fn c088_lets_an_exception_context_serve_as_a_hash_key() {
    // Given two contexts from separate raises of the SAME value. C056 makes
    // them distinct events, so default equality separates them and each is its
    // own Hash key. Reading back through both keys proves two entries exist: a
    // single merged entry could not return two different values.
    let source = "let first = try { raise :same } catch _, c { c }; \
                  let second = try { raise :same } catch _, c { c }; \
                  let h = %{ first: 1, second: 2 }; \
                  [first == second, first same? second, h[first], h[second]]";

    // When / Then
    assert_eq!(
        rendered(source),
        "Array([Bool(false), Bool(false), Integer(IntegerValue(1)), Integer(IntegerValue(2))])"
    );
}

#[test]
fn d155_appends_one_re_raise_site_per_bare_raise_in_occurrence_order() {
    // D-155 makes each bare `raise` APPEND one site to the context it
    // continues, without replacing the root stack or creating a fresh context.
    let one_site = "mut captured = nil; \
                    try { try { raise :x } catch _, c { captured = c; raise } } \
                    catch _, o { [o same? captured, o.re_raise_sites] }";
    let two_sites = "try { try { try { raise :x } catch _, c { raise } } \
                     catch _, m { raise } } catch _, o { o.re_raise_sites }";
    let no_site = "try { raise :x } catch _, c { c.re_raise_sites }";

    // When / Then the continued context is the SAME one, which is why identity
    // rather than payload decides `same?` under C067: appending a site would
    // otherwise make a retained context compare as a different one.
    let one = rendered(one_site);
    assert!(
        one.starts_with("Array([Bool(true), ReadonlyArray([RaiseSite("),
        "{one}"
    );
    assert_eq!(one.matches("RaiseSite(").count(), 1, "{one}");
    let two = rendered(two_sites);
    assert_eq!(two.matches("RaiseSite(").count(), 2, "{two}");
    assert_eq!(rendered(no_site), "ReadonlyArray([])");
}

#[test]
fn c067_keeps_distinct_events_unequal_under_identity_comparison() {
    // Identity comparison must not collapse into "always equal": two events
    // carrying the same raised object stay distinct, and a context is still
    // `same?` itself.
    let reraised_value = "try { raise :same } catch value, first { \
                          try { raise value } catch _, second { second same? first } }";
    let reflexive = "try { raise :x } catch _, c { c same? c }";

    // When / Then
    assert_eq!(rendered(reraised_value), "Bool(false)");
    assert_eq!(rendered(reflexive), "Bool(true)");
}

#[test]
fn c012_makes_a_top_level_call_a_privileged_implicit_send_to_main() {
    // Given a Module body declaring a top-level helper and calling it. C012
    // installs the helper on the Module's `main`, private by default, and makes
    // the bare call a PRIVILEGED send that reaches it.
    //
    // A Module body yields no value of its own, so the call's EFFECT is what is
    // observed: a helper that never ran would leave the counter at 0.
    let called = "mut log = 0; \
                  module M { fun helper() -> Integer { log = 1; 1 } helper() } log";
    let not_called = "mut log = 0; module M { fun helper() -> Integer { log = 1; 1 } } log";
    // D-433: a bare unresolved name is still a NameError rather than an
    // implicit send, so the privilege does not make every name resolvable.
    let unresolved = "module M { fun helper() -> Integer { 1 } helper_missing }";
    // A helper declared LATER in the body is still callable, since declarations
    // are published before any executable statement runs.
    // The call and the declaration need a separator: C078 makes a bare
    // `f 1` the parenthesis-less call form, so two statements must be split.
    let forward = "mut log = 0; \
                   module M { later(); fun later() -> Integer { log = 2; 1 } } log";

    // When / Then
    assert_eq!(rendered(called), "Integer(IntegerValue(1))");
    assert_eq!(rendered(not_called), "Integer(IntegerValue(0))");
    assert_eq!(rendered(unresolved), "NameError");
    assert_eq!(rendered(forward), "Integer(IntegerValue(2))");
}

#[test]
fn c077_denies_an_external_send_to_a_private_module_method() {
    // C077 defaults a Module Method to PRIVATE. The `M.name()` path read the
    // method map directly and never checked visibility, so an importer could
    // call a private top-level helper against C012.
    let private_helper = "module M { fun hidden() -> Integer { 1 } } M.hidden()";
    let explicitly_private = "module M { private module fun h() -> Integer { 1 } } M.h()";
    let public_helper = "module M { public module fun shown() -> Integer { 1 } } M.shown()";

    // When / Then. `rendered` shows the raw error; the conformance runner maps
    // this to `MethodVisibilityError`, which is what the vector asserts.
    let denied = "Construction(Dispatch(VisibilityDenied";
    assert!(rendered(private_helper).starts_with(denied));
    assert!(rendered(explicitly_private).starts_with(denied));
    assert_eq!(rendered(public_helper), "Integer(IntegerValue(1))");
}

#[test]
fn c079_exposes_source_locations_with_one_based_line_and_column() {
    // C079 makes `line` and `column` ONE-BASED, so the first character of a
    // program is line 1, column 1, and `raise` at column 7 reports 7 rather
    // than a byte offset.
    let single_line = "try { raise :x } catch _, c { [c.raise_location.line, \
                       c.raise_location.column] }";
    // The location must track real position, so a raise on the third line
    // reports 3 rather than always reporting 1.
    let third_line = "let a = 1\nlet b = 2\ntry { raise :x } catch _, c { c.raise_location.line }";
    // C066 forbids fabricated propagation metadata, and this evaluator keeps no
    // call stack, so `original_stack` is EMPTY rather than invented.
    let stack = "try { raise :x } catch _, c { c.original_stack }";

    // When / Then
    assert_eq!(
        rendered(single_line),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(7))])"
    );
    assert_eq!(rendered(third_line), "Integer(IntegerValue(3))");
    assert_eq!(rendered(stack), "ReadonlyArray([])");
}

#[test]
fn d142_rejects_every_mutation_of_a_runtime_owned_collection() {
    // D-142 lets user code ITERATE and COPY a suppressed collection but never
    // insert, delete, replace, or reorder it.
    let prefix = "mut n = 0; \
                  class It { public fun next() { n = n + 1; \
                  if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
                  public fun close() { raise :close } } \
                  class Src { public fun iterator() { It.new() } } \
                  try { for x in Src.new() { raise :body } } catch _, c { ";
    for mutation in [
        "c.suppressed.append(1)",
        "c.suppressed.delete(0)",
        "c.suppressed[0] = 1",
        "c.suppressed.reverse!()",
    ] {
        assert_eq!(
            rendered(&format!("{prefix}{mutation} }}")),
            "ReadonlyMutation",
            "not rejected: {mutation}"
        );
    }

    // Reading and indexing stay legal, so the rejection is specific to mutation
    // rather than making the collection unusable.
    assert_eq!(
        rendered(&format!("{prefix}c.suppressed[0].value }}")),
        "Symbol(\"close\")"
    );
}

#[test]
fn d415_names_three_callable_kinds_and_no_function() {
    // D-415: Iris has NO separate Function runtime kind. An unbound Method is
    // reflective, a BoundMethod captures a receiver plus Method, and a Closure
    // is anonymous lexical code.
    let kinds = "class C { public fun m(x: Integer) -> Integer { x } } \
                 let method = Reflection::Class.method(C, :m); \
                 let bound = C.new().m; let closure = { |x: Integer| x }; \
                 [method.class_name, bound.class_name, closure.class_name]";
    // The absence is part of the requirement, so it is asserted rather than
    // left unchecked.
    let absent = "Function";

    // When / Then
    assert_eq!(
        rendered(kinds),
        "Array([Symbol(\"Method\"), Symbol(\"BoundMethod\"), Symbol(\"Closure\")])"
    );
    assert_eq!(rendered(absent), "NameError");
}

#[test]
fn c096_dispatches_an_absent_to_bool_through_method_missing() {
    // C096: when `to_bool` is ABSENT after a permitted removal, truth testing
    // invokes `method_missing(:to_bool, [], nil)` ONCE and uses its Bool result
    // directly. A call counter is required here: probing this row previously
    // produced a FALSE POSITIVE, because the expected `:then` also arrives from
    // the C094 default while `method_missing` runs zero times.
    let dispatched = "mut calls = 0; mut seen = nil; \
                      class C { public fun method_missing(selector, args, block) { \
                      calls = calls + 1; seen = [selector, args, block]; true } } \
                      C.undef_method(:to_bool); \
                      let result = if C.new() { :then } else { :else }; [result, calls, seen]";
    // The fallback's Bool result is used directly, so returning false selects
    // the else branch rather than being truth-tested again.
    let falsehood = "class C { public fun method_missing(s, a, b) { false } } \
                     C.undef_method(:to_bool); if C.new() { :then } else { :else }";
    // With `to_bool` still present, the ordinary path applies and the fallback
    // is never consulted.
    let present = "mut calls = 0; \
                   class C { public fun method_missing(s, a, b) { calls = calls + 1; true } } \
                   let result = if C.new() { :then } else { :else }; [result, calls]";

    // When / Then
    assert!(rendered(dispatched).ends_with(
        "Array([Symbol(\"then\"), Integer(IntegerValue(1)), \
         Array([Symbol(\"to_bool\"), Array([]), Nil])])])"
    ));
    assert!(rendered(falsehood).ends_with("Symbol(\"else\")])"));
    assert_eq!(
        rendered(present),
        "Array([Symbol(\"then\"), Integer(IntegerValue(0))])"
    );
}

#[test]
fn d149_matches_a_typed_catch_against_the_current_class_hierarchy() {
    // D-149: exception dispatch uses the CURRENT Class hierarchy when selection
    // begins. A committed `set_superclass` must therefore be visible to a later
    // typed catch, which walked a declaration-time map that the commit never
    // updated: `is Parent` already saw the change while the catch did not.
    let after_commit = "class Parent {} class Child {} \
                        Reflection::Class.set_superclass(Child, Parent); \
                        try { raise Child.new() } catch _: Parent { :matched } \
                        catch _ { :unmatched }";
    let before_commit = "class Parent {} class Child {} \
                         try { raise Child.new() } catch _: Parent { :matched } \
                         catch _ { :unmatched }";
    // A statically declared superclass must keep working, since both now read
    // the same runtime hierarchy.
    let declared = "class Parent {} class Child extends Parent {} \
                    try { raise Child.new() } catch _: Parent { :matched } catch _ { :unmatched }";

    // When / Then
    assert!(rendered(after_commit).ends_with("Symbol(\"matched\")])"));
    assert_eq!(rendered(before_commit), "Symbol(\"unmatched\")");
    assert_eq!(rendered(declared), "Symbol(\"matched\")");
}

#[test]
fn raw_ivar_slots_are_dynamic_and_per_instance() {
    // A raw `@x` slot is `Dynamic<Object>`, so one instance may store an
    // Integer and then a Symbol in the same slot, and a SECOND instance is
    // unaffected: the slot belongs to the receiver rather than the Class.
    let per_instance = "class A { public fun w(v) { @x = v } public fun r() { @x } } \
                        let a = A.new(); let b = A.new(); a.w(1); a.w(:s); [a.r(), b.r()]";
    // A Closure created in an instance Method captures its CURRENT receiver and
    // keeps mutating that receiver's raw ivars after the Method returns.
    let escaped = "class A { public fun m() { @x = 0; { @x = @x + 1; @x } } } \
                   let c = A.new().m(); [c.call(), c.call()]";

    // When / Then
    assert!(rendered(per_instance).ends_with("Array([Symbol(\"s\"), Nil])])"));
    assert_eq!(
        rendered(escaped),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))])"
    );
}

#[test]
fn c041_and_c043_make_string_a_value_with_exact_scalar_equality() {
    // C041 makes a String an identity-less immutable sequence of Unicode scalar
    // values, and C043 compares the EXACT sequence and case with no
    // normalization, case folding, or locale mapping.
    let literal = "\"iris\"";
    let equal = "\"a\" == \"a\"";
    let unequal = "\"a\" == \"b\"";
    let case_sensitive = "\"A\" == \"a\"";
    // A String is never equal to a non-String, so it never reaches the numeric
    // comparison path.
    let cross_type = "\"1\" == 1";
    // C041 gives String its own builtin Class, so `is String` resolves it like
    // the numeric value Classes rather than as an ordinary Object.
    let typed = "let v = \"iris\"; [v is String, v is Object, 1 is String]";

    // When / Then
    assert_eq!(rendered(literal), "Text(\"iris\")");
    assert_eq!(rendered(equal), "Bool(true)");
    assert_eq!(rendered(unequal), "Bool(false)");
    assert_eq!(rendered(case_sensitive), "Bool(false)");
    assert_eq!(rendered(cross_type), "Bool(false)");
    assert_eq!(
        rendered(typed),
        "Array([Bool(true), Bool(true), Bool(false)])"
    );
}

#[test]
fn c030_makes_a_safe_cast_yield_the_same_value_or_nil() {
    // C030: `value as? T` evaluates `value` once and returns the SAME
    // underlying value on success, or `nil` on a failed runtime check. It never
    // converts, which is what separates it from a coercion.
    let failed = "let v = \"iris\"; v as? Integer";
    let succeeded = "let v = 1; v as? Integer";
    let combined = "let value: Object = \"iris\"; [value is String, value as? Integer]";

    // When / Then
    assert_eq!(rendered(failed), "Nil");
    assert_eq!(rendered(succeeded), "Integer(IntegerValue(1))");
    assert_eq!(rendered(combined), "Array([Bool(true), Nil])");
}

#[test]
fn c004_enforces_a_written_binding_annotation_as_a_runtime_guard() {
    // C004 makes a written annotation BOTH a static contract and a runtime
    // boundary guard: a not-proven boundary MUST check before the value is
    // published. Nothing checked before, so a violating value was stored
    // silently.
    let violated = "let value: Integer = nil; value";
    let satisfied = "let value: Integer = 1; value";
    // C011: `NonNil` admits every value except nil, and is a Type rather than a
    // declared Class, so it never resolves through the Class registry.
    let non_nil_violated = "let value: NonNil = nil; value";
    let non_nil_satisfied = "let value: NonNil = 1; value";
    // C023: `Never` is uninhabited, so NO value satisfies it.
    let never = "let value: Never = 1; value";
    // C020: a union admits a value satisfying ANY constituent.
    let union_satisfied = "let value: String | Integer = \"iris\"; value";
    let union_violated = "let value: String | Integer = nil; value";
    // C009 keeps `Object` the top, so it admits everything.
    let root = "let value: Object = \"iris\"; value";

    // When / Then
    assert_eq!(rendered(violated), "TypeContractError");
    assert_eq!(rendered(satisfied), "Integer(IntegerValue(1))");
    assert_eq!(rendered(non_nil_violated), "TypeContractError");
    assert_eq!(rendered(non_nil_satisfied), "Integer(IntegerValue(1))");
    assert_eq!(rendered(never), "TypeContractError");
    assert_eq!(rendered(union_satisfied), "Text(\"iris\")");
    assert_eq!(rendered(union_violated), "TypeContractError");
    assert_eq!(rendered(root), "Text(\"iris\")");
}

#[test]
fn c004_guards_the_parameter_and_return_boundaries() {
    // C004 makes a written annotation a runtime boundary guard on a PARAMETER
    // and a RETURN as well as on a binding. Both were parsed and discarded, so
    // a violating value crossed either boundary silently.
    let return_violated = "class A { public fun m() -> Integer { nil } } A.new().m()";
    let return_satisfied = "class A { public fun m() -> Integer { 1 } } A.new().m()";
    let parameter_violated = "class A { public fun m(x: Integer) { x } } A.new().m(nil)";
    let parameter_satisfied = "class A { public fun m(x: Integer) { x } } A.new().m(7)";
    // An unannotated boundary has no guard to apply, so it still accepts
    // anything. This is what keeps gradual typing gradual.
    let unannotated = "class A { public fun m(x) { x } } A.new().m(nil)";
    // A `Nil` return annotation ADMITS nil, so the guard must not treat every
    // nil as a violation.
    let nil_return = "class A { public fun m() -> Nil { nil } } A.new().m()";

    // When / Then
    assert_eq!(rendered(return_violated), "TypeContractError");
    assert_eq!(rendered(return_satisfied), "Integer(IntegerValue(1))");
    assert_eq!(rendered(parameter_violated), "TypeContractError");
    assert_eq!(rendered(parameter_satisfied), "Integer(IntegerValue(7))");
    assert_eq!(rendered(unannotated), "Nil");
    assert_eq!(rendered(nil_return), "Nil");
}

#[test]
fn d206_interns_a_closed_type_by_definition_and_arguments() {
    // D-206 interns a CLOSED Type identity by definition AND normalized
    // arguments, so two constructions of one definition are the same Type only
    // when their arguments match. The arguments were dropped, which made every
    // construction of a definition one interned Type.
    let same_arguments = "class Box<T> {} Box<String>.type same? Box<String>.type";
    let different_arguments = "class Box<T> {} Box<String>.type same? Box<Integer>.type";
    // A bare Class name carries no arguments, so its Type is the unapplied
    // definition's and still interns to one identity.
    let unapplied = "class A {} A.type same? A.type";
    // C076 keeps the Type object DISTINCT from the Class object it reifies.
    let type_is_not_class = "class A {} A.type same? A";

    // When / Then
    assert_eq!(rendered(same_arguments), "Bool(true)");
    assert_eq!(rendered(different_arguments), "Bool(false)");
    assert_eq!(rendered(unapplied), "Bool(true)");
    assert_eq!(rendered(type_is_not_class), "Bool(false)");
}

#[test]
fn c065_guards_a_stored_property_slot_against_a_raw_write() {
    // C065 makes stored-property storage TYPED and C161 makes `@name` that
    // exact slot, so a raw write meets the SAME C004 contract the generated
    // setter enforces. A Method body writing `@n` bypassed the guard entirely.
    let raw_violation = "class A { property n: Integer = 0
  public fun bad(x) { @n = x } }
let a = A.new(); a.bad(nil)";
    let raw_satisfied = "class A { property n: Integer = 0
  public fun ok(x) { @n = x } }
let a = A.new(); a.ok(7); a.n";
    // The generated setter guards the same slot from outside.
    let setter_violation = "class A { property n: Integer = 0 }
let a = A.new(); a.n = nil";
    // C066 makes slot identity `(receiver, name)` rather than the declaring
    // Class, so a subclass Method writing an inherited slot meets the SAME
    // contract.
    let inherited_violation = "class A { property n: Integer = 0 }
class B extends A { public fun bad(x) { @n = x } }
B.new().bad(nil)";
    let inherited_satisfied = "class A { property n: Integer = 0 }
class B extends A { public fun ok(x) { @n = x } }
B.new().ok(3)";
    // C068 gives an UNDECLARED raw ivar static type `Dynamic<Object>`, so it
    // carries no contract and stays unguarded.
    let undeclared = "class A { public fun m(x) { @free = x } }
let a = A.new(); a.m(nil)";

    // When / Then
    assert_eq!(rendered(raw_violation), "TypeContractError");
    assert!(rendered(raw_satisfied).contains("Integer(IntegerValue(7))"));
    assert_eq!(rendered(setter_violation), "TypeContractError");
    assert_eq!(rendered(inherited_violation), "TypeContractError");
    assert_eq!(rendered(inherited_satisfied), "Integer(IntegerValue(3))");
    assert_eq!(rendered(undeclared), "Nil");
}

#[test]
fn c016_interns_a_composed_type_in_its_normal_form() {
    // C016 interns Type objects by identity, so two spellings of ONE Type must
    // reify to equal values. Members are sorted and deduplicated when the form
    // is built, which makes commutativity and idempotence hold by construction
    // rather than by a separate comparison rule.
    let commutative = "(String | Integer).type same? (Integer | String).type";
    let idempotent_union = "(String | String).type same? String.type";
    let idempotent_intersection = "(String & String).type same? String.type";
    // C023 makes `Never` the union identity and an intersection annihilator.
    let union_identity = "(String | Never).type same? String.type";
    let intersection_annihilator = "(String & Never).type same? (Never).type";
    // C009 keeps `Object` the top, so it is the intersection identity and the
    // union absorber.
    let intersection_identity = "(String & Object).type same? String.type";
    let union_absorber = "(String | Object).type same? Object.type";
    // C011 makes `T?` sugar for `T | Nil`, so the sugar normalizes alike.
    let nilable = "(String?).type same? (String | Nil).type";
    let object_optional = "(Object?).type same? Object.type";
    let never_optional = "(Never?).type same? Nil.type";
    // `NonNil` removes `Nil`, and removing the only other member leaves an
    // uninhabited Type.
    let non_nil_removal = "((String | Nil) & NonNil).type same? String.type";
    let non_nil_impossible = "(Nil & NonNil).type same? (Never).type";
    // C065 confines the Type reading to a `.type` lookahead, so a parenthesized
    // expression NOT followed by `.type` keeps the OPERATOR reading. Bitwise or
    // on Integer is not implemented, so reaching its dispatch at all proves the
    // Type reading was not taken.
    let operator_reading = "let a = 6; let b = 3; (a | b)";

    // When / Then
    for source in [
        commutative,
        idempotent_union,
        idempotent_intersection,
        union_identity,
        intersection_annihilator,
        intersection_identity,
        union_absorber,
        nilable,
        object_optional,
        never_optional,
        non_nil_removal,
        non_nil_impossible,
    ] {
        assert_eq!(rendered(source), "Bool(true)", "law failed: {source}");
    }
    assert!(rendered(operator_reading).contains("selector: \"|\""));
}

#[test]
fn c016_absorbs_a_declared_subtype_without_distributing() {
    // V005 and V006 state absorption over a DECLARED subtype pair, so the laws
    // must consult the same nominal ancestry `is` does rather than a fixed
    // builtin table.
    let declared = "class Animal {} class Dog extends Animal {} ";
    let union_keeps_wider = format!("{declared}(Dog | Animal).type same? Animal.type");
    let intersection_keeps_narrower = format!("{declared}(Dog & Animal).type same? Dog.type");
    // V016 states that an intersection over a union is deliberately NOT
    // distributed: the compact form is kept, so it does NOT equal the expanded
    // one. This is the one law whose expected answer is false.
    let no_distribution = "class A {} class B {} class C {} \
(A & (B | C)).type same? ((A & B) | (A & C)).type";
    // Two unrelated Classes absorb nothing, so both survive and only order is
    // normalized.
    let unrelated_commutes = "class A {} class B {} (A & B).type same? (B & A).type";

    // When / Then
    assert_eq!(rendered(&union_keeps_wider), "Bool(true)");
    assert_eq!(rendered(&intersection_keeps_narrower), "Bool(true)");
    assert_eq!(rendered(no_distribution), "Bool(false)");
    assert_eq!(rendered(unrelated_commutes), "Bool(true)");
}

#[test]
fn c064_keeps_generic_class_storage_per_closed_construction() {
    // C064 gives ordinary generic class-level storage INDEPENDENT storage per
    // closed construction. v1 interns one Class per generic definition, so
    // `Cache<String>` and `Cache<Integer>` reach the same ClassId and shared
    // one bucket: a read answered the other construction's last write.
    let independent = "class Cache<T> { class property value: T } \
Cache<String>.value = \"s\"; Cache<Integer>.value = 1; \
[Cache<String>.value, Cache<Integer>.value]";
    // The SAME construction keeps one bucket, so a second write is visible to
    // its own read rather than creating a third slot.
    let same_construction = "class Cache<T> { class property value: T } \
Cache<String>.value = \"s\"; Cache<String>.value = \"t\"; Cache<String>.value";
    // A non-generic Class is unaffected: it has no arguments to qualify with.
    let non_generic = "class Cache { class property value: Object } \
Cache.value = \"s\"; Cache.value";

    // When / Then
    assert!(rendered(independent).ends_with("Array([Text(\"s\"), Integer(IntegerValue(1))])])"));
    assert!(rendered(same_construction).ends_with("Text(\"t\")])"));
    assert!(rendered(non_generic).ends_with("Text(\"s\")])"));
}

#[test]
fn c064_puts_a_shared_class_property_on_the_unapplied_definition() {
    // C064 puts class-level storage on the CLASS OBJECT. It was installed as an
    // instance property, so `A.n` was unresolvable and reading it answered nil.
    let class_level_read = "class A { class property n: Integer = 5 } A.n";
    let class_level_write = "class A { class property n: Integer = 5 } A.n = 9; A.n";
    // A `shared class property` belongs to the UNAPPLIED definition, so the
    // bare Class reads its declared initializer. The V238 vector asserts the
    // closed-access error; this covers the read the vector cannot also carry.
    let shared_read = "class Cache<T> { shared class property count: Integer = 0 } Cache.count";
    // An ordinary class-level property stays per closed construction.
    let per_construction =
        "class Cache<T> { class property v: Integer = 7 } Cache<String>.v = 3; Cache<String>.v";
    // An instance property is unaffected by the class-level path.
    let instance = "class A { property n: Integer = 5 } A.new().n";

    // When / Then
    assert_eq!(rendered(class_level_read), "Integer(IntegerValue(5))");
    assert!(rendered(class_level_write).ends_with("Integer(IntegerValue(9))])"));
    assert_eq!(rendered(shared_read), "Integer(IntegerValue(0))");
    assert!(rendered(per_construction).ends_with("Integer(IntegerValue(3))])"));
    assert_eq!(rendered(instance), "Integer(IntegerValue(5))");
}

#[test]
fn c014_checks_the_value_when_entering_a_bounded_dynamic() {
    // C014 makes `Dynamic<T>` BOUNDED dynamic sending: entering it checks that
    // the value satisfies the reified `T`. The boundary only lifts static
    // member validation INSIDE it, so the entry itself is guarded like any
    // other annotation. Nothing checked, so a violating value entered silently.
    let violated = "let value: Dynamic<String> = 1; value";
    let satisfied = "let value: Dynamic<String> = \"s\"; value";
    // Bare `Dynamic` normalizes to `Dynamic<Object>`, which admits everything,
    // so the check must not fire on an unbounded Dynamic.
    let unbounded = "let value: Dynamic = 1; value";
    let explicit_object = "let value: Dynamic<Object> = 1; value";

    // When / Then
    assert_eq!(rendered(violated), "TypeContractError");
    assert_eq!(rendered(satisfied), "Text(\"s\")");
    assert_eq!(rendered(unbounded), "Integer(IntegerValue(1))");
    assert_eq!(rendered(explicit_object), "Integer(IntegerValue(1))");
}

#[test]
fn c047_and_c050_govern_contract_view_dispatch_and_identity() {
    // C047 lets ONE unqualified `impl` member satisfy every declared same-name
    // Contract requirement. Only a QUALIFIED slot was consulted, so a Class
    // whose `impl fun m` is unqualified could never be reached through its own
    // Contract view.
    let declared = "contract C { fun m() -> String } \
class X for C { public impl fun m() -> String { \"c\" } } ";
    let qualified_send = format!("{declared}let view = X.new() as C; view..m()");
    // C032 keeps an ordinary `view.member()` an unqualified message forwarded
    // to the receiver, so both spellings reach the one implementation.
    let ordinary_send = format!("{declared}let view = X.new() as C; view.m()");
    // The Class must actually DECLARE the Contract, which keeps an unrelated
    // same-name Method from becoming an accidental implementation.
    let undeclared = "contract C { fun m() -> String } \
class Y { public fun m() -> String { \"y\" } } let v = Y.new() as C; v..m()";
    // C050 makes a Contract view an identity-LESS capability value, so `same?`
    // raises rather than comparing the underlying receiver.
    let view_identity = format!("{declared}let a = X.new() as C; a same? a");
    let object_identity = "class A {} let a = A.new(); a same? a";

    // When / Then
    assert_eq!(rendered(&qualified_send), "Text(\"c\")");
    assert_eq!(rendered(&ordinary_send), "Text(\"c\")");
    assert_ne!(rendered(undeclared), "Text(\"y\")");
    assert_eq!(rendered(&view_identity), "IdentityError");
    assert_eq!(rendered(object_identity), "Bool(true)");
}

#[test]
fn c050_compares_contract_views_by_receiver_and_contract() {
    // C050 defines built-in view equality as the SAME Contract identity plus
    // receiver identity for an identity-bearing receiver, or receiver equality
    // under current equality for an identity-LESS one. Forwarding `==` to the
    // receiver would have ignored the Contract identity entirely.
    let declared = "contract N { fun m() -> Integer } \
open class Integer for N { public impl fun m() -> Integer { 1 } } ";
    let same_receiver = format!("{declared}(1 as N) == (1 as N)");
    let different_receiver = format!("{declared}(1 as N) == (2 as N)");
    let negated = format!("{declared}(1 as N) != (2 as N)");
    // A view is never equal to a NON-view, so the wrapper is not transparent.
    let view_versus_value = format!("{declared}(1 as N) == 1");
    // C050 also defines equality over identity-LESS receivers, which is why a
    // value Class may be viewed at all.
    let value_view = format!("{declared}1 as N");

    // When / Then
    assert_eq!(rendered(&same_receiver), "Bool(true)");
    assert_eq!(rendered(&different_receiver), "Bool(false)");
    assert_eq!(rendered(&negated), "Bool(true)");
    assert_eq!(rendered(&view_versus_value), "Bool(false)");
    assert!(rendered(&value_view).starts_with("ContractView(Integer"));
}

#[test]
fn c058_needs_explicit_conformance_for_an_f_bounded_constraint() {
    // C058 permits restricted nominal F-bounded constraints such as
    // `where T: Comparable<T>`, and a concrete argument satisfies one ONLY
    // through explicit nominal conformance. Matching member SHAPE is
    // insufficient, so the declared `for` list is consulted, not the members.
    // C067 validates this at closed generic materialization and RAISES.
    let unsatisfied =
        "contract Comparable<T> {} class Box<T> where T: Comparable<T> {} Box<String>.new()";
    let satisfied = "contract Comparable<T> {} class Ok for Comparable {} \
class Box<T> where T: Comparable<T> {} Box<Ok>.new()";
    // A Class with no `where` clause has no bound to violate.
    let unconstrained = "class Box<T> {} Box<String>.new()";

    // When / Then
    assert_eq!(rendered(unsatisfied), "TypeContractError");
    assert!(rendered(satisfied).starts_with("Object("));
    assert!(rendered(unconstrained).starts_with("Object("));
}

#[test]
fn d219_composes_a_closed_generic_module_as_a_mixin() {
    // D-219 reifies and interns a CLOSED Module Type such as `Helpers<String>`.
    // A mixin target accepted only a bare name, so a closed construction was
    // rejected outright. v1 interns one Module per generic definition, so the
    // closed form composes that definition rather than a second Module.
    let closed = "module Helpers<T> { public fun h() -> Integer { 7 } } \
class Host mixin Helpers<String> {} Host.new().h()";
    let plain = "module Helpers { public fun h() -> Integer { 7 } } \
class Host mixin Helpers {} Host.new().h()";

    // When / Then
    assert_eq!(rendered(closed), "Integer(IntegerValue(7))");
    assert_eq!(rendered(plain), "Integer(IntegerValue(7))");
}

#[test]
fn c061_makes_a_type_alias_a_name_for_its_target() {
    // `type_alias_decl` is in the frozen grammar but was never implemented, so
    // `type Name = Integer` did not parse at all. C061 makes an alias a NAME
    // for its target rather than a new nominal Type, so the two share one
    // interned Type identity. The V256 vector asserts the recursive-alias
    // diagnostic; this covers the identity half, which the frozen row writes
    // with `Array` and no built-in Class provides yet.
    let shares_identity = "class Box<T> {} type Name<T> = Box<T>; \
Name<String>.type same? Box<String>.type";
    // `type` stays an ordinary selector when it does not begin a declaration.
    let selector = "class A {} A.type same? A.type";

    // When / Then
    assert_eq!(rendered(shares_identity), "Bool(true)");
    assert_eq!(rendered(selector), "Bool(true)");
}

#[test]
fn c047_lets_one_impl_satisfy_two_compatible_contracts() {
    // C047 merges compatible same-name requirements into ONE obligation, so a
    // single `impl` member is reached through either Contract's view without
    // listing targets.
    let source = "contract A { fun m(x: Object) -> String } \
contract B { fun m(x: Object) -> String } \
class X for A, B { public impl fun m(x: Object) -> String { \"x\" } } \
let x = X.new(); [(x as A)..m(1), (x as B)..m(1)]";
    // A Contract the Class does NOT declare cannot be viewed, which keeps the
    // merge from reaching an undeclared conformance.
    let undeclared = "contract A { fun m(x: Object) -> String } \
contract B { fun m(x: Object) -> String } \
class X for A { public impl fun m(x: Object) -> String { \"x\" } } \
(X.new() as B)..m(1)";

    // When / Then
    assert_eq!(rendered(source), "Array([Text(\"x\"), Text(\"x\")])");
    assert_ne!(rendered(undeclared), "Text(\"x\")");
}

#[test]
fn c013_reads_a_top_level_helper_as_a_bound_method() {
    // C012 puts top-level executable code inside a Module body, and C013
    // resolves a bare name against visible DECLARATIONS as well as lexical
    // bindings. C014 makes reading a Method create a BoundMethod rather than
    // exposing a Function runtime kind, so a helper is readable as a value and
    // not only callable.
    let read = "mut r = 0; module M { fun f() -> Integer { 1 } r = f } r";
    // A Module body is where top-level executable code lives, which includes
    // BINDINGS; only expressions ran, so every `let` there was skipped.
    let binding = "mut r = 0; module M { fun f() -> Integer { 1 } let g = 5; r = g } r";
    // C037: a parameter is CONTRAVARIANT, so a wider declared parameter accepts
    // a narrower argument, through a bound helper as through a direct call.
    let contravariant = "mut r = 0; \
module M { fun accept(x: Object) -> String { \"ok\" } let f = accept; r = f(\"x\") } r";
    // Narrowing the parameter reverses the relation, which proves the
    // acceptance is variance and not an absence of checking.
    let narrowed = "mut r = 0; \
module M { fun accept(x: String) -> String { \"ok\" } let f = accept; r = f(1) } r";
    // C011 keeps an unresolved bare name a NameError, so the declaration lookup
    // does not make every name resolvable.
    let unresolved = "module M { fun f() -> Integer { 1 } missing }";

    // When / Then
    assert!(rendered(read).starts_with("BoundMethod("));
    assert_eq!(rendered(binding), "Integer(IntegerValue(5))");
    assert_eq!(rendered(contravariant), "Text(\"ok\")");
    assert_eq!(rendered(narrowed), "TypeContractError");
    assert_eq!(rendered(unresolved), "NameError");
}

#[test]
fn c059_infers_a_method_type_argument_from_the_call_site() {
    // `method_decl` spells `"fun" selector generic_params? parameter_list`, so
    // a Method may declare its OWN type parameters. They were never read, which
    // made `fun id<T>(x: T) -> T` a parse error rather than a generic Method.
    let inferred = "mut result: String = \"z\"; \
module M { fun id<T>(x: T) -> T { x } let bound: String = id(\"iris\"); result = bound } result";
    // The inferred argument must be OBSERVED, not merely returned. A `let` with
    // a written Type is the boundary C004 guards AT RUNTIME, so a mismatched
    // call raises there. A `mut` reassignment reports only statically, which
    // would have let this pass for the wrong reason.
    let mismatched = "mut result: String = \"z\"; \
module M { fun id<T>(x: T) -> T { x } let bound: String = id(1); result = bound } result";
    // A non-generic Method with the same shape is unaffected.
    let plain = "mut result = 0; module M { fun id(x) { x } result = id(\"iris\") } result";

    // When / Then
    assert_eq!(rendered(inferred), "Text(\"iris\")");
    assert_eq!(rendered(mismatched), "TypeContractError");
    assert_eq!(rendered(plain), "Text(\"iris\")");
}

#[test]
fn c013_makes_a_global_a_declared_cell() {
    // C013 makes `$name` reachable ONLY through a `global let` or `global mut`
    // declaration: a missing one is a declaration error rather than a fresh
    // cell created by use.
    let read = "global let $g = 1; $g";
    let assigned = "global mut $g = 1; $g = 2; $g";
    // `global let` is immutable exactly as an ordinary `let` is.
    let immutable = "global let $g = 1; $g = 2; $g";
    let undeclared = "$missing";

    // When / Then
    assert!(rendered(read).ends_with("Integer(IntegerValue(1))])"));
    assert!(rendered(assigned).ends_with("Integer(IntegerValue(2))])"));
    assert_eq!(rendered(immutable), "ImmutableBinding");
    assert_eq!(rendered(undeclared), "NameError");
}

#[test]
fn v002_makes_a_contract_a_type_constituent() {
    // V002 states intersection commutativity over two CONTRACTS, so a Contract
    // is an irreducible Type constituent alongside a nominal Class. Only
    // Classes were normalized, so a Contract intersection was unsupported.
    let commutative = "contract Readable {} contract Closeable {} \
(Readable & Closeable).type same? (Closeable & Readable).type";
    let distinct = "contract Readable {} contract Closeable {} contract Other {} \
(Readable & Closeable).type same? (Readable & Other).type";
    // A Class intersection is unaffected.
    let classes = "class A {} class B {} (A & B).type same? (B & A).type";

    // When / Then
    assert_eq!(rendered(commutative), "Bool(true)");
    assert_eq!(rendered(distinct), "Bool(false)");
    assert_eq!(rendered(classes), "Bool(true)");
}

#[test]
fn c003_reflects_only_the_written_signature_annotations() {
    // C003 gives an OMITTED Method parameter or return annotation the Contract
    // `Dynamic<Object>`, and D-452 keeps body-local inference OUT of signature
    // metadata. Nothing reflected a signature at all.
    let omitted = "class A { public fun f(value) { value } } \
[A.method(:f).parameters, A.method(:f).return_type]";
    // A written annotation reflects as itself.
    let written = "class A { public fun f(value: Integer) -> String { \"x\" } } \
[A.method(:f).parameters, A.method(:f).return_type]";
    // The BODY must not contribute: returning an Integer from an unannotated
    // Method leaves the reflected return `Dynamic<Object>`, which is what
    // "body-local inference is absent" means.
    let body_typed = "class A { public fun f(value) { 1 } } \
[A.method(:f).parameters, A.method(:f).return_type]";

    // When / Then
    assert_eq!(
        rendered(omitted),
        "Array([Array([Symbol(\"Dynamic<Object>\")]), Symbol(\"Dynamic<Object>\")])"
    );
    assert_eq!(
        rendered(written),
        "Array([Array([Symbol(\"Integer\")]), Symbol(\"String\")])"
    );
    assert_eq!(rendered(body_typed), rendered(omitted));
}

#[test]
fn d178_resolves_the_origin_before_the_open_transaction() {
    // D-178: declaration collection resolves the ORIGIN before the open
    // transaction, so an `open class A` may PRECEDE the `class A` it reopens.
    // A single ordered pass ran the transaction against a Class that did not
    // exist yet.
    let open_first = "open class A { public override fun marker() -> String { \"open\" } } \
class A { public fun marker() -> String { \"origin\" } } A.new().marker()";
    // The ordinary order still applies the transaction AFTER the origin's own
    // members exist, so hoisting the origin must not reorder the reopen.
    let origin_first = "class A { public fun marker() -> String { \"origin\" } } \
open class A { public override fun marker() -> String { \"open\" } } A.new().marker()";
    // A Class with no reopen is unaffected by the hoisting pass.
    let plain = "class A { public fun m() -> Integer { 1 } } A.new().m()";

    // When / Then
    assert_eq!(rendered(open_first), "Text(\"open\")");
    assert_eq!(rendered(origin_first), "Text(\"open\")");
    assert_eq!(rendered(plain), "Integer(IntegerValue(1))");
}

#[test]
fn c067_interns_no_identity_for_a_constraint_violation() {
    // C067 validates every normalized `where` constraint BEFORE interning or
    // publishing, so a violating construction interns no closed Type identity.
    // Only Contract bounds were checked, so a Type bound such as `NonNil` was
    // ignored entirely.
    let violated = "class Box<T> where T: NonNil {} Box<Nil>.type";
    let satisfied = "class Box<T> where T: NonNil {} Box<Integer>.type";
    // C023 makes `Never` uninhabited, so no argument satisfies it.
    let never = "class Box<T> where T: Never {} Box<Integer>.type";
    // A Class with no `where` clause has no bound to violate.
    let unconstrained = "class Box<T> {} Box<Nil>.type";
    // The construction path checks the same bound the interning path does.
    let constructed = "class Box<T> where T: NonNil {} Box<Nil>.new()";

    // When / Then
    assert_eq!(rendered(violated), "TypeContractError");
    assert!(rendered(satisfied).starts_with("Type("));
    assert_eq!(rendered(never), "TypeContractError");
    assert!(rendered(unconstrained).starts_with("Type("));
    assert_eq!(rendered(constructed), "TypeContractError");
}

#[test]
fn c012_runs_a_raise_and_a_class_property_in_a_module_body() {
    // C012 makes a Module body the home of top-level EXECUTABLE code, so a
    // `raise` there is ordinary control flow. The declaration pass rejected it
    // outright, and a class-level property in a Module body was unsupported
    // even though C064 puts that storage on the Module's own object.
    let raised = "module M { raise :stop } 1";
    let declared = "module M { shared class property first: Integer = 1 } 1";
    // A raise in a Module body propagates, so statements after it do not run.
    let halted =
        "mut r = 0; module M { shared class property first: Integer = 1; r = 5; raise :stop } r";
    let completed = "mut r = 0; module M { shared class property first: Integer = 1; r = 5 } r";

    // When / Then
    assert_eq!(rendered(raised), "Raised(Symbol(\"stop\"))");
    assert_eq!(rendered(declared), "Integer(IntegerValue(1))");
    assert_eq!(rendered(halted), "Raised(Symbol(\"stop\"))");
    assert_eq!(rendered(completed), "Integer(IntegerValue(5))");
}

#[test]
fn v214_keeps_a_union_member_inside_an_intersection() {
    // V016 keeps `A & (B | C)` a COMPACT intersection CONTAINING the union
    // member rather than distributing it, and V214 reflects exactly those two
    // members. Normalization flattened the union into three peers, so the
    // structure both rows describe was not represented at all.
    let declared = "class A {} class B {} class C {} ";
    let reflected =
        format!("{declared}let t = (A & (B | C)).type; [t.kind, (B | C).type same? t.members[1]]");
    // A flat intersection has no nested member to report.
    let flat = format!("{declared}(A & B).type.kind");
    // V014 and V221 still remove `Nil` from a NESTED union, so the constraint
    // reaches inside the constituent rather than stopping at its boundary.
    let non_nil = "((String | Nil) & NonNil).type same? String.type";
    // A union reduced to one member collapses back to that member.
    let collapsed = "(Nil & NonNil).type same? (Never).type";

    // When / Then
    assert_eq!(
        rendered(&reflected),
        "Array([Symbol(\"intersection\"), Bool(true)])"
    );
    assert_eq!(rendered(&flat), "Symbol(\"intersection\")");
    assert_eq!(rendered(non_nil), "Bool(true)");
    assert_eq!(rendered(collapsed), "Bool(true)");
}

#[test]
fn v240_runs_a_per_closed_initializer_once_per_construction() {
    // C066 runs a per-closed class property initializer ONCE when the closed
    // Class is first materialized, not once at the generic declaration. The
    // initializer never ran at all, so every closed construction answered nil.
    let declared = "mut n = 0; mut log = []; class Box<T> { class property tag: Integer = \
                    { n = n + 1; log.append(n); n }.call() } ";
    // Two requests for the SAME construction reuse the first materialization.
    let repeated = format!(
        "{declared}let a = Box<String>.tag; let b = Box<String>.tag; \
         let c = Box<Integer>.tag; [log, a, b, c]"
    );
    // C064 keeps ordinary generic class-level storage independent per closed
    // construction.
    let independent = "class Box<T> { class property tag: Integer = 7 } \
                       [Box<String>.tag, Box<Integer>.tag]";

    // When / Then
    assert_eq!(
        rendered(&repeated),
        "Array([Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))]), \
         Integer(IntegerValue(1)), Integer(IntegerValue(1)), Integer(IntegerValue(2))])"
    );
    assert_eq!(
        rendered(independent),
        "Array([Integer(IntegerValue(7)), Integer(IntegerValue(7))])"
    );
}

#[test]
fn a_failed_materialization_reports_a_type_contract_error() {
    // C097 reports an exception escaping a per-closed initializer as
    // `TypeContractError`, matching how C067 reports a constraint failure on
    // the SAME materialization path. The initializer's own `:boom` propagated
    // instead, so the failure was indistinguishable from an ordinary raise.
    //
    // C066 does NOT undo external side effects and lets a later request retry,
    // so both attempts survive in the external log. IRIS-V1-TYPES-V241
    // observes exactly that.
    let retried = "mut log = []; class Box<T> { class property tag: Integer = \
                   { log.append(:attempt); raise :boom; 1 }.call() } \
                   let a = try { Box<String>.tag } catch e { e }; \
                   let b = try { Box<String>.tag } catch e { e }; [a, b, log]";
    // C066 publishes nothing for the failed construction, so an UNRELATED
    // construction still materializes normally afterwards.
    let unaffected = "mut n = 0; class Box<T> { class property tag: Integer = \
                      { n = n + 1; n }.call() } Box<String>.tag";

    // When / Then
    assert_eq!(
        rendered(retried),
        "Array([Symbol(\"TypeContractError\"), Symbol(\"TypeContractError\"), \
         Array([Symbol(\"attempt\"), Symbol(\"attempt\")])])"
    );
    assert_eq!(rendered(unaffected), "Integer(IntegerValue(1))");
}

#[test]
fn a_module_class_level_property_is_readable_through_the_module_name() {
    // C064 puts a class-level property on the MODULE's own object, and D-212
    // runs each shared initializer once in source declaration order. The
    // storage was installed correctly but nothing could READ it: a Module name
    // evaluates to a Symbol, so `M.first` reported a missing message on Symbol
    // whatever the Module actually held.
    let single = "module M { shared class property first: Integer = 1 } M.first";
    // D-212 runs the initializers in SOURCE DECLARATION ORDER.
    let ordered = "module M { shared class property a: Integer = 1 \
                   shared class property b: Integer = 2 } [M.a, M.b]";
    // C064 also gives a Module an ordinary, non-shared class-level property.
    let ordinary = "module M { class property c: Integer = 3 } M.c";
    // A Module Method send still resolves, rather than being captured by the
    // property path.
    let method = "module M { public module fun f() { 7 } } M.f()";
    // A lexical binding SHADOWS the Module name, so the property path must not
    // hijack a name the program rebound.
    let shadowed = "module M { shared class property x: Integer = 1 } let M = 9; M";

    // When / Then
    assert_eq!(rendered(single), "Integer(IntegerValue(1))");
    assert_eq!(
        rendered(ordered),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))])"
    );
    assert_eq!(rendered(ordinary), "Integer(IntegerValue(3))");
    assert_eq!(rendered(method), "Integer(IntegerValue(7))");
    assert_eq!(rendered(shadowed), "Integer(IntegerValue(9))");
}

#[test]
fn c067_admits_a_bare_closed_generic_name_as_a_value() {
    // C067 admits a CLOSED generic name as a complete expression, which D-456
    // needs in order to distinguish an interned Type object from the Class
    // object. Requiring a `postfix_part` left `Box<String>` unparseable as a
    // value, so V257's comparison could not be written at all.
    let distinct = "class Box<T> {} let t = Box<String>.type; \
                    [t same? Box<String>.type, t same? Box<String>]";
    // C020 is NOT weakened: anything that could continue an expression keeps
    // the operator reading.
    let comparison = "let a = 1; let b = 2; let c = 3; let d = 4; [a < b, c > d]";
    let shift = "let a = 8; let b = 1; a >> b";
    // A closed generic name is complete at the end of the input and as the
    // RIGHT operand of an infix send. As a LEFT operand it is followed by a
    // name, which could continue an expression, so C067 leaves that to the
    // operator reading.
    let at_end = "class Box<T> {} Box<String>";
    let right_operand = "class Box<T> {} let t = Box<String>.type; t same? Box<String>";

    // When / Then
    assert_eq!(rendered(distinct), "Array([Bool(true), Bool(false)])");
    assert_eq!(rendered(comparison), "Array([Bool(true), Bool(false)])");
    assert_eq!(rendered(shift), "Integer(IntegerValue(4))");
    assert_eq!(rendered(at_end), "Class(ClassId(7))");
    assert_eq!(rendered(right_operand), "Bool(false)");
}

#[test]
fn d431_scopes_a_global_to_its_declaring_package() {
    // D-431 makes a global's TRUE identity `(package_id, $name)`, unique
    // within a package and separately instantiated per runtime, with NO flat
    // cross-package namespace and no auto-merge. Keying by name alone let two
    // packages declaring the same `$name` share one cell, which is exactly the
    // process-global storage D-431 says does not exist.
    let packages = |entries: &[(&str, &str)]| {
        let owned: Vec<(String, String)> = entries
            .iter()
            .map(|(package, source)| ((*package).to_owned(), (*source).to_owned()))
            .collect();
        format!("{:?}", crate::evaluate_packages(&owned))
    };
    let mutated = "global mut $count: Integer = 1; $count = 7";

    // When / Then: `b` declares the same name and still reads its OWN cell.
    assert_eq!(
        packages(&[
            ("a", mutated),
            ("b", "global mut $count: Integer = 1; $count")
        ]),
        "Ok(Array([Integer(IntegerValue(1)), Integer(IntegerValue(1))]))"
    );
    // The declaring package still sees its own mutation.
    assert_eq!(
        packages(&[("a", mutated), ("a", "$count")]),
        "Ok(Integer(IntegerValue(7)))"
    );
    // C013 keeps an undeclared `$name` an error that creates NO storage, and
    // another package's global does not satisfy the read.
    assert_eq!(
        packages(&[("a", mutated), ("b", "$count")]),
        "Err(NameError)"
    );
    assert_eq!(
        packages(&[("a", mutated), ("a", "$missing")]),
        "Err(NameError)"
    );
}

#[test]
fn d432_orders_unqualified_resolution_across_three_tiers() {
    // D-432 resolves an unqualified name as LEXICAL scope, then the CURRENT
    // module's declarations, then explicit imports. A Module body `const` was
    // an ordinary lexical binding, so it LEAKED: code outside any Module read
    // it, and a second Module declaring the same name overwrote the first.
    let tiers = "module S { const K = 5 } \
                 module M { const K = 1 public module fun lexical() { let K = 9; K } \
                 public module fun declared() { K } } \
                 from S import K; [M.lexical(), M.declared(), K]";
    // A Module's constant is invisible OUTSIDE the Module that declared it.
    let scoped = "module M { const K = 5 } K";
    // Two Modules may declare the same constant name without collision.
    let independent = "module A { const K = 1 public module fun f() { K } } \
                       module B { const K = 2 public module fun g() { K } } [A.f(), B.g()]";
    // An import binds under its alias when the source writes one.
    let aliased = "module M { const K = 5 } from M import K as J; J";

    // When / Then: 9 is lexical, 1 is the Module's own declaration, 5 is the
    // import, which is the exact precedence D-432 states.
    assert_eq!(
        rendered(tiers),
        "Array([Integer(IntegerValue(9)), Integer(IntegerValue(1)), Integer(IntegerValue(5))])"
    );
    assert_eq!(rendered(scoped), "NameError");
    assert_eq!(
        rendered(independent),
        "Array([Integer(IntegerValue(1)), Integer(IntegerValue(2))])"
    );
    assert_eq!(rendered(aliased), "Integer(IntegerValue(5))");
}

#[test]
fn c022_publishes_nothing_from_a_failed_class_body_transaction() {
    // IRIS-V1-META-C022 makes a Class or open body an executable construction
    // transaction over a CANDIDATE: success validates the complete candidate
    // and publishes atomically, failure publishes NOTHING from it. Every
    // structural operation used to open and publish its own revision, so a
    // body failing halfway had already published its earlier members.
    let staged = |source: &str, selector: &str| {
        let (outcome, published) = crate::evaluate_with_member_probe(source, "A", selector);
        (outcome.is_err(), published)
    };
    // An open body whose LATER member fails must not leave `ok` published.
    let open_fails = "class A { public fun base() { 0 } } \
                      open class A { public fun ok() { 9 } public override fun nope() { 1 } }";
    // An origin body failing halfway must not leave its earlier member either.
    let origin_fails = "class A { public fun ok() { 9 } public fun bad() { 1 } \
                        public fun bad() { 2 } }";
    // A succeeding open publishes what it staged.
    let open_succeeds = "class A { public fun base() { 0 } } \
                         open class A { public fun ok() { 9 } }";

    // When / Then
    assert_eq!(staged(open_fails, "ok"), (true, false));
    assert_eq!(staged(origin_fails, "ok"), (true, false));
    assert_eq!(staged(open_succeeds, "ok"), (true, true));
}

#[test]
fn v206_validates_a_candidate_against_its_declared_contracts() {
    // C022 validates the COMPLETE candidate before publishing, and
    // IRIS-V1-TYPES-C006 reports a Contract failure as `TypeContractError`. An
    // open that breaks a declared Contract used to fail as an unsupported
    // construct AFTER publishing the member staged before it.
    let incompatible = "contract C { fun draw() } class A for C { public fun draw() { 1 } } \
                        open class A { public fun m() { 9 } \
                        public override fun draw(a, b) { 2 } } A.new().m()";
    // The same body with a COMPATIBLE replacement commits, and `m` is callable.
    let compatible = "contract C { fun draw() } class A for C { public fun draw() { 1 } } \
                      open class A { public fun m() { 9 } \
                      public override fun draw() { 2 } } A.new().m()";

    // When / Then
    assert_eq!(rendered(incompatible), "TypeContractError");
    assert_eq!(rendered(compatible), "Integer(IntegerValue(9))");
}

#[test]
fn v203_rejects_a_candidate_that_replaces_a_contract_visible_return_type() {
    // IRIS-V1-TYPES-C045 makes declared Contract conformance immutable for a
    // revision's static spine and forbids metaprogramming from incompatibly
    // replacing it. Candidate validation compared ARITY alone, so an open
    // rewriting `draw() -> String` as `draw() -> Integer` committed silently.
    let replaced = "contract C { fun draw() -> String } \
                    class A for C { public fun draw() -> String { \"a\" } } \
                    open class A { public fun extra() { 9 } \
                    public override fun draw() -> Integer { 1 } } A.new().draw()";
    // A replacement keeping the declared return Type still commits.
    let compatible = "contract C { fun draw() -> String } \
                      class A for C { public fun draw() -> String { \"a\" } } \
                      open class A { public override fun draw() -> String { \"b\" } } \
                      A.new().draw()";
    // C022 publishes NOTHING from the failed candidate, so the member staged
    // beside the offending one is absent from the published revision too.
    let staged = "contract C { fun draw() -> String } \
                  class A for C { public fun draw() -> String { \"a\" } } \
                  open class A { public fun extra() { 9 } \
                  public override fun draw() -> Integer { 1 } }";

    // When
    let (outcome, published) = crate::evaluate_with_member_probe(staged, "A", "extra");

    // Then
    assert_eq!(rendered(replaced), "TypeContractError");
    assert_eq!(rendered(compatible), "Text(\"b\")");
    assert!(outcome.is_err());
    assert!(!published);
}

#[test]
fn c022_runs_executable_statements_in_a_class_body() {
    // IRIS-V1-META-C022 makes a Class body an EXECUTABLE construction
    // transaction that may run ordinary control flow and use lexical locals,
    // and C027 keeps those locals ordinary: they do NOT become class state
    // merely because the transaction commits. A Class body rejected every
    // executable statement as an unsupported construct.
    let binding = "class A { let v = 7 } 1";
    let expression = "class A { 1 + 1 } 1";
    // C027: the local is not published as class state.
    let not_state = "class A { let v = 7 } A.v";

    // When / Then
    assert_eq!(rendered(binding), "Integer(IntegerValue(1))");
    assert_eq!(rendered(expression), "Integer(IntegerValue(1))");
    assert_eq!(
        rendered(not_state),
        "MessageNotFound { receiver_class: \"Class\", selector: \"v\" }"
    );
}

#[test]
fn c023_defines_a_method_on_the_current_candidate() {
    // IRIS-V1-META-C023 makes a structural meta message sent to `self`, such as
    // `define_method`, target the current transaction CANDIDATE. C026 keeps the
    // defined Method from closing over the body's transaction-temporary
    // locals: its lexical environment is definition scope and its own
    // parameters, not the body execution's locals.
    let defined = "class A { self.define_method(:m) { 1 } } A.new().m()";
    let parameters = "class A { self.define_method(:add) { |x| x + 1 } } A.new().add(1)";
    // C026: the body local `value` is NOT captured, so the call resolves to the
    // declared Method and answers 8 rather than the transaction local's 7.
    let uncaptured = "class A { let value = 7 self.define_method(:answer) { value() } \
                      public fun value() -> Integer { 8 } } A.new().answer()";
    // Without a declaration to resolve to, the local is simply not in scope.
    let absent = "class A { let value = 7 self.define_method(:answer) { value } } A.new().answer()";

    // When / Then
    assert_eq!(rendered(defined), "Integer(IntegerValue(1))");
    assert_eq!(rendered(parameters), "Integer(IntegerValue(2))");
    assert_eq!(rendered(uncaptured), "Integer(IntegerValue(8))");
    assert_eq!(rendered(absent), "NameError");
}

#[test]
fn c014_reads_a_bare_name_as_a_bound_method_of_its_own_class() {
    // IRIS-V1-CONTROL-C014 makes reading an instance Method create a
    // BoundMethod, and IRIS-V1-META-C024 makes an unqualified name that matched
    // no binding or declaration a PRIVILEGED send to the current `self`. A bare
    // name naming a Method of the receiver's own Class reported NameError.
    let read = "class A { public fun v() { 8 } public fun m() { v } } A.new().m()";
    let called = "class A { public fun v() { 8 } public fun m() { v() } } A.new().m()";
    // A name matching nothing is still unresolved.
    let unresolved = "class A { public fun m() { nothing_here } } A.new().m()";

    // When / Then
    assert!(rendered(read).starts_with("BoundMethod"));
    assert_eq!(rendered(called), "Integer(IntegerValue(8))");
    assert_eq!(rendered(unresolved), "NameError");
}

#[test]
fn c023_defines_a_module_method_on_its_main_receiver() {
    // IRIS-V1-META-C023 makes `self.define_method` inside a Module body target
    // the current transaction candidate, and IRIS-V1-CONTROL-C012 gives that
    // body its Module's `main` receiver. IRIS-V1-TYPES-C074 keeps the Method a
    // Module member as well, which is why a declared `fun` publishes both
    // copies; the defined Method needs the same second copy or `M.m()` reports
    // a missing message on Module.
    let defined = "module M { self.define_method(:m) { 1 } } M.m()";
    // C026 keeps the defined Method from closing over the body's
    // transaction-temporary locals, so `value` resolves to the DECLARED Method
    // and answers 8 rather than the local's 7. This is IRIS-V1-META-V342.
    let uncaptured = "module M { let value = 7 self.define_method(:answer) { value() } \
                      public fun value() -> Integer { 8 } } M.answer()";

    // When / Then
    assert_eq!(rendered(defined), "Integer(IntegerValue(1))");
    assert_eq!(rendered(uncaptured), "Integer(IntegerValue(8))");
}

#[test]
fn c027_discards_module_body_locals_after_the_body_runs() {
    // IRIS-V1-META-C027 makes Class and Module body locals ORDINARY LEXICAL
    // LOCALS that do NOT become Module state merely because the body
    // transaction commits. A Module body `let` stayed in the shared lexical
    // scope, so it outlived its body and was readable from outside the Module.
    let escaped = "module M { let value = 7 } value";
    // A `const` is a DECLARATION of the current module under D-432 and is
    // still reachable from the Module's own Methods.
    let constant = "module M { const K = 5 public fun read() -> Integer { K } } M.read()";

    // When / Then
    assert_eq!(rendered(escaped), "NameError");
    assert_eq!(rendered(constant), "Integer(IntegerValue(5))");
}

#[test]
fn c012_sends_a_bare_call_to_the_executing_module() {
    // IRIS-V1-CONTROL-C012 makes a bare `f(...)` inside a Module a PRIVILEGED
    // implicit send, and IRIS-V1-META-C024 falls back to the current Module
    // `main` receiver. A Module name evaluates to a Symbol, so the evaluated
    // receiver could not serve the send and one Module Method calling another
    // reported a missing message on Symbol.
    let called = "module M { public fun a() -> Integer { b() } \
                  public fun b() -> Integer { 8 } } M.a()";
    // IRIS-V1-CONTROL-C014 makes READING an instance Method create a
    // BoundMethod rather than invoking it.
    let read = "module M { public fun a() { b } public fun b() -> Integer { 8 } } M.a()";

    // When / Then
    assert_eq!(rendered(called), "Integer(IntegerValue(8))");
    assert!(rendered(read).starts_with("BoundMethod"));
}

#[test]
fn c033_and_c034_run_a_programmatic_open_transaction() {
    // IRIS-V1-META-C033 makes programmatic `Class#open` the same transaction
    // model the declarative `open class` uses, and C034 commits on normal
    // completion and rolls the candidate back on failure. Neither `Class#open`
    // nor `define_method` existed, so seven TYPES rows and most of chapter 08's
    // open-transaction rows had no way to be driven at all.
    let defined = "class A {} A.open() { |t| t.define_method(:extra) { 9 } }; A.new().extra()";
    let parameters = "class A {} A.open() { |t| t.define_method(:add) { |x| x + 1 } }; \
                      A.new().add(1)";
    // C033 forbids targeting a closed generic Class, and v1 interns one Class
    // per generic definition, so the definition itself is refused.
    let generic = "class Box<T> {} Box.open() { |t| 1 }";
    let closed = "class Box<T> {} Box<String>.open() { |t| 1 }";

    // When / Then: the block's own value is reported, after the `define_method`
    // statement's nil.
    assert_eq!(rendered(defined), "Array([Nil, Integer(IntegerValue(9))])");
    assert_eq!(
        rendered(parameters),
        "Array([Nil, Integer(IntegerValue(2))])"
    );
    assert_eq!(rendered(generic), "UnsupportedConstruct");
    assert_eq!(rendered(closed), "UnsupportedConstruct");
}

#[test]
fn c034_rolls_back_a_failed_programmatic_open() {
    // C034 rolls back every candidate on exception, and C022 publishes NOTHING
    // from a failed candidate, so a Method staged before the failure must be
    // absent from the published revision.
    let fails = "class A { public fun base() { 0 } } \
                 A.open() { |t| t.define_method(:extra) { 9 } raise :boom }";
    let succeeds = "class A { public fun base() { 0 } } \
                    A.open() { |t| t.define_method(:extra) { 9 } }";

    // When
    let (failed, leaked) = crate::evaluate_with_member_probe(fails, "A", "extra");
    let (_, published) = crate::evaluate_with_member_probe(succeeds, "A", "extra");

    // Then
    assert!(failed.is_err());
    assert!(!leaked);
    assert!(published);
}

#[test]
fn c038_joins_nested_opens_into_one_transaction_group() {
    // IRIS-V1-META-C038 makes same-thread nested opens join the OUTERMOST
    // transaction group, forbids an inner open from committing independently,
    // and requires all candidates in the group to publish together or all roll
    // back. An inner open committed on its own, so its target stayed published
    // even when the outer transaction later failed.
    let published = |source: &str, class: &str, selector: &str| {
        crate::evaluate_with_member_probe(source, class, selector).1
    };
    // The group commits together when the outermost completes.
    let commits = "class A {} class B {} \
                   A.open() { |x| x.define_method(:m) { 1 }; \
                   B.open() { |y| y.define_method(:n) { 2 } } }";
    // The group rolls back together when the outermost fails.
    let rolls_back = "class A {} class B {} \
                      A.open() { |x| x.define_method(:m) { 1 }; \
                      B.open() { |y| y.define_method(:n) { 2 } }; raise :boom }";
    // Reopening a target already in the group REUSES its candidate, so both
    // Methods survive rather than the second candidate replacing the first.
    let reused = "class A {} A.open() { |x| x.define_method(:one) { 1 }; \
                  A.open() { |y| y.define_method(:two) { 2 } } }";

    // When / Then
    assert!(published(commits, "A", "m"));
    assert!(published(commits, "B", "n"));
    assert!(!published(rolls_back, "A", "m"));
    assert!(!published(rolls_back, "B", "n"));
    assert!(published(reused, "A", "one"));
    assert!(published(reused, "A", "two"));
}

#[test]
fn c039_raises_a_conflict_when_a_target_moved_past_its_base() {
    // IRIS-V1-META-C039 records every candidate's base active revision and, at
    // commit, raises `MetaTransactionConflictError` and rolls the whole group
    // back when an overlapping target changed since that base. Nothing recorded
    // or compared a base revision, so a concurrent structural change was
    // silently overwritten by the committing candidate.
    //
    // A reflective mutation inside the block publishes its own revision, which
    // is what moves the target past the recorded base.
    let conflicting = "class A { public fun base() { 0 } } \
                       A.open() { |x| x.define_method(:m) { 1 }; \
                       Reflection::Class.set_superclass(A, Object) }";
    // A group with no outside change still commits.
    let clean = "class A {} A.open() { |x| x.define_method(:m) { 1 } }; A.new().m()";

    // When
    let (conflicted, published) = crate::evaluate_with_member_probe(conflicting, "A", "m");

    // Then: the conflict rolls the group back, so the staged Method is absent.
    assert!(rendered(conflicting).starts_with("Class(MetaTransactionConflict"));
    assert!(conflicted.is_err());
    assert!(!published);
    assert_eq!(rendered(clean), "Array([Nil, Integer(IntegerValue(1))])");
}

#[test]
fn c035_reads_the_candidate_inside_a_transaction_and_the_revision_outside() {
    // IRIS-V1-META-C035 lets the open block read its OWN candidate structural
    // metadata after writes, while code outside that transaction keeps
    // observing the published active revision until the commit. C036 makes
    // candidate properties visible ONLY through such a read. No property
    // reflection existed at all, so neither half was observable.
    let staged = "class A { public property x: Integer = 1 } \
                  A.open() { |t| t.define_property(:y) { 2 }; t.properties }";
    // A rollback publishes none of it, which is IRIS-V1-TYPES-V207.
    let rolled_back = "class A { public property x: Integer = 1 } \
                       try { A.open() { |t| t.define_property(:y) { 2 }; raise :boom } } \
                       catch e { 0 }; A.properties";
    // A committing open publishes it.
    let committed = "class A { public property x: Integer = 1 } \
                     A.open() { |t| t.define_property(:y) { 2 } }; A.properties";
    // C119 implements the operation once and exposes it under both spellings.
    let both = "class A { public property x: Integer = 1 } \
                [A.properties, Reflection::Class.properties(A)]";

    // When / Then
    assert_eq!(rendered(staged), "Array([Symbol(\"@x\"), Symbol(\"@y\")])");
    assert_eq!(
        rendered(rolled_back),
        "Array([Integer(IntegerValue(0)), Array([Symbol(\"@x\")])])"
    );
    assert_eq!(
        rendered(committed),
        "Array([Nil, Array([Symbol(\"@x\"), Symbol(\"@y\")])])"
    );
    assert_eq!(
        rendered(both),
        "Array([Array([Symbol(\"@x\")]), Array([Symbol(\"@x\")])])"
    );
}

#[test]
fn c056_makes_a_specification_named_error_catchable() {
    // IRIS-V1-CONTROL-C056 makes `raise value` accept ANY Iris object or value
    // and hands the ORIGINAL value to the catch. Every runtime failure the
    // specification NAMES travelled past every handler as an evaluator-internal
    // error instead, so no `catch` of any form intercepted one. That blocked
    // C040's explicit retry and V241's second attempt among others.
    let name_error = "try { undefined_name } catch e { e }";
    let contract = "contract C { fun d() } class Box<T> where T: C {} class X {} \
                    try { Box<X>.tag } catch e { e }";
    // IRIS-V1-META-C040 lets user code catch a conflict and retry explicitly.
    let conflict = "class A {} try { A.open() { |x| x.define_method(:m) { 1 }; \
                    Reflection::Class.set_superclass(A, Object) } } catch e { e }";
    // A control-flow unwind is NOT an exception and still reaches its own
    // boundary rather than being intercepted by an unrelated handler.
    let returned = "class A { public fun m() { try { return 5 } catch e { 9 } } } A.new().m()";

    // When / Then
    assert_eq!(rendered(name_error), "Symbol(\"NameError\")");
    assert_eq!(rendered(contract), "Symbol(\"TypeContractError\")");
    assert_eq!(
        rendered(conflict),
        "Symbol(\"MetaTransactionConflictError\")"
    );
    assert_eq!(rendered(returned), "Integer(IntegerValue(5))");
}

#[test]
fn c022_rolls_back_an_origin_body_that_raises() {
    // IRIS-V1-META-C022 lets a Class body run ordinary synchronous control
    // flow, which includes `raise`, and publishes NOTHING from a failed
    // candidate. A `raise` in a Class body was rejected as an unsupported
    // construct instead, so IRIS-V1-META-V341's origin-transaction failure
    // could not be expressed at all.
    let raises = "class Box { self.define_method(:ok) { 1 }; raise :stop }";
    let commits = "class Box { self.define_method(:ok) { 1 } } Box.new().ok()";

    // When
    let (outcome, class_published) = crate::evaluate_with_class_publication(raises, "Box");
    let (_, member_published) = crate::evaluate_with_member_probe(raises, "Box", "ok");

    // Then: IRIS-V1-RUNTIME-C024 leaves the Class unpublished and C022 commits
    // no staged Method with it.
    assert!(outcome.is_err());
    assert!(!class_published);
    assert!(!member_published);
    assert_eq!(rendered(commits), "Integer(IntegerValue(1))");
}

#[test]
fn c081_denies_a_meta_operation_missing_its_capability() {
    // IRIS-V1-META-C081's capability matrix is normative and complete for v1
    // core meta operations, and IRIS-V1-META-V361 requires every denied lane to
    // raise `MetaCapabilityError` while the allowed lane commits its diff. The
    // denial was raised but was NOT catchable, so a row could observe neither
    // lane from Iris source.
    let allowed = "class B { } B.open() { |t| t.define_method(:m) { 7 } }; B.new().m()";
    let method_set = "class B meta deny method_set { } \
                      try { B.open() { |t| t.define_method(:m) { 1 } } } catch e { e }";
    let method_body = "class B meta deny method_body { public fun m() -> Integer { 1 } } \
                       try { B.open() { |t| t.define_method(:m) { 2 } } } catch e { e }";
    let property_set = "class B meta deny property_set { } \
                        try { B.open() { |t| t.define_property(:x) { 1 } } } catch e { e }";

    // When / Then
    assert_eq!(rendered(allowed), "Array([Nil, Integer(IntegerValue(7))])");
    assert_eq!(rendered(method_set), "Symbol(\"MetaCapabilityError\")");
    assert_eq!(rendered(method_body), "Symbol(\"MetaCapabilityError\")");
    assert_eq!(rendered(property_set), "Symbol(\"MetaCapabilityError\")");
}

#[test]
fn c076_combines_deny_origins_from_every_source() {
    // IRIS-V1-META-C076 makes a Class's effective capabilities the defaults
    // minus every denial from the declared superclass chain, local origin
    // denies, the runtime chain, Module-sourced denies, AND Contract-required
    // denies. A Contract's `meta deny` was parsed and then DISCARDED, so a
    // Contract-required denial never reached an implementing Class.
    let combined = "class Base meta deny method_set { } module NoShape meta deny shape { } \
                    contract Fixed meta deny class_state_set { } \
                    class Target extends Base for Fixed mixin NoShape { } \
                    Target.denied_capabilities";
    // C077 forbids a subclass or an open from re-enabling an ancestor's denial.
    let cannot_restore = "class Base meta deny method_set { } class T extends Base { } \
                          try { T.open() { |t| t.define_method(:m) { 1 } } } catch e { e }";
    // A Class denying nothing reports an empty set.
    let none = "class B { } B.denied_capabilities";

    // When / Then: the denials are reported in the fixed C081 vocabulary order.
    assert_eq!(
        rendered(combined),
        "Array([Symbol(\"method_set\"), Symbol(\"shape\"), Symbol(\"class_state_set\")])"
    );
    assert_eq!(rendered(cannot_restore), "Symbol(\"MetaCapabilityError\")");
    assert_eq!(rendered(none), "Array([])");
}

#[test]
fn c148_lets_a_stable_builtin_gain_behaviour_but_keep_its_identity() {
    // IRIS-V1-RUNTIME-C148 lets a stable built-in Class compose Modules and
    // gain compatible Methods on open, while C150 protects its superclass, and
    // IRIS-V1-META-C081 requires the refusal to name the missing capability.
    // The refusal was not catchable, so IRIS-V1-META-V359 could observe the
    // behaviour half but not the protection half.
    let behaviour = "open class Bool { public fun tag() -> Symbol { :ok } } \
                     open class Integer { public fun tag() -> Symbol { :ok } } \
                     [true.tag(), Integer(1).tag()]";
    // The primitives keep their identity across the open.
    let identity = "module Marker { } open class Nil mixin Marker { } \
                    [nil same? nil, true same? true]";
    // C150 refuses a superclass change on a protected built-in.
    let protected = "class Other { } \
                     [try { Reflection::Class.set_superclass(Nil, Other) } catch e { e }, \
                     try { Reflection::Class.set_superclass(Bool, Other) } catch e { e }]";
    // An ordinary Class is unaffected by that protection.
    let ordinary = "class B { } class O { } \
                    try { Reflection::Class.set_superclass(B, O) } catch e { e }";

    // When / Then
    assert_eq!(
        rendered(behaviour),
        "Array([Symbol(\"ok\"), Symbol(\"ok\")])"
    );
    assert_eq!(rendered(identity), "Array([Bool(true), Bool(true)])");
    assert_eq!(
        rendered(protected),
        "Array([Symbol(\"MetaCapabilityError\"), Symbol(\"MetaCapabilityError\")])"
    );
    assert_eq!(rendered(ordinary), "Nil");
}

#[test]
fn c017_rejects_a_package_whose_sources_import_each_other() {
    // IRIS-V1-META-C017 makes Module initialization an ACYCLIC deterministic
    // DAG and requires a dependency or initialization cycle to be a compile or
    // link error. A package whose two files imported each other LOADED
    // SUCCESSFULLY and reported both Modules as initialized.
    let cyclic = [
        ("src/a.ir".to_owned(), "import B\nmodule A { }\n".to_owned()),
        ("src/b.ir".to_owned(), "import A\nmodule B { }\n".to_owned()),
    ];
    // Breaking either edge makes the graph acyclic again.
    let acyclic = [
        ("src/a.ir".to_owned(), "module A { }\n".to_owned()),
        ("src/b.ir".to_owned(), "import A\nmodule B { }\n".to_owned()),
    ];
    // A file importing a name no other file declares is not a cycle.
    let unresolved = [(
        "src/a.ir".to_owned(),
        "import Absent\nmodule A { }\n".to_owned(),
    )];

    // When / Then: the cycle is rejected BEFORE any Module body runs, so the
    // package publishes nothing.
    assert!(matches!(
        crate::load_package("org.test", &cyclic),
        Err(crate::EvaluationError::ModuleInitializationCycleError)
    ));
    assert_eq!(
        crate::load_package("org.test", &acyclic),
        Ok(vec!["A".to_owned(), "B".to_owned()])
    );
    assert_eq!(
        crate::load_package("org.test", &unresolved),
        Ok(vec!["A".to_owned()])
    );
}

#[test]
fn c077_refuses_a_private_method_from_every_path_but_its_class() {
    // IRIS-V1-RUNTIME-C077 makes a Method private by default outside an
    // explicit declaration, and IRIS-V1-META-V434 requires only the declaring
    // Class's lexical call to succeed. The refusal was raised correctly but was
    // NOT catchable, so the row could observe the success and not the refusals.
    let own = "class B { private fun secret() -> Symbol { :secret } \
               public fun own() -> Symbol { secret() } } B.new().own()";
    let external = "class B { private fun secret() -> Symbol { :secret } } \
                    try { B.new().secret() } catch e { e }";
    let subclass = "class B { private fun secret() -> Symbol { :secret } } \
                    class C extends B { public fun t() -> Symbol { secret() } } \
                    try { C.new().t() } catch e { e }";

    // When / Then
    assert_eq!(rendered(own), "Symbol(\"secret\")");
    assert_eq!(rendered(external), "Symbol(\"MethodVisibilityError\")");
    assert_eq!(rendered(subclass), "Symbol(\"MethodVisibilityError\")");
}

#[test]
fn c072_keeps_a_raw_ivar_per_receiver_and_binds_an_escaping_closure() {
    // IRIS-V1-RUNTIME-C161 makes `@name` per-receiver storage and C072 keeps a
    // Closure created in an instance Method bound to the receiver that created
    // it, which is what IRIS-V1-META-V434 observes across two receivers.
    let distinct = "class B { public fun w(v) { @x = v } public fun r() { @x } } \
                    let a = B.new(); let b = B.new(); a.w(:first); b.w(:second); [a.r(), b.r()]";
    let escaped = "class B { public fun w(v) { @x = v } public fun grab() { { @x } } } \
                   let a = B.new(); let f = a.grab(); a.w(:first); \
                   let b = B.new(); b.w(:second); f.call()";

    // When / Then: the escaped Closure still reads the FIRST receiver.
    assert_eq!(
        rendered(distinct),
        "Array([Symbol(\"first\"), Symbol(\"second\"), Array([Symbol(\"first\"), Symbol(\"second\")])])"
    );
    assert_eq!(
        rendered(escaped),
        "Array([Symbol(\"first\"), Symbol(\"second\"), Symbol(\"first\")])"
    );
}

#[test]
fn c100_gives_raw_ivar_reflection_one_slot_vocabulary() {
    // IRIS-V1-RUNTIME-C161 makes a stored property named `name` create the slot
    // `@name`, and IRIS-V1-META-C100 gives `list_ivars`, `get_ivar`, `set_ivar`
    // and `remove_ivar` one name vocabulary over those slots. Source `@x = 1`
    // parsed to the slot `x` while a declared property and `set_ivar` published
    // `@x`, so ONE ivar had TWO slots and reflection could not see what source
    // had written.
    let written = "class B { public fun w(v) { @x = v } } let o = B.new(); o.w(3); \
                   [Reflection::Object.list_ivars(o), Reflection::Object.get_ivar(o, :@x)]";
    // A name `list_ivars` reports feeds straight back in, with or without the
    // sigil.
    let round_trip = "class B { public fun w(v) { @x = v } } let o = B.new(); o.w(3); \
                      Reflection::Object.get_ivar(o, :x)";
    // C100 removes an EXISTING slot and returns its old value.
    let removed = "class B { public fun w(v) { @x = v } } let o = B.new(); o.w(3); \
                   Reflection::Object.remove_ivar(o, :@x)";
    // V362 names the absent case.
    let absent = "class B { } \
                  try { Reflection::Object.remove_ivar(B.new(), :@gone) } catch e { e }";
    // An absent GET is nil rather than an error.
    let missing = "class B { } Reflection::Object.get_ivar(B.new(), :@gone)";

    // When / Then
    assert_eq!(
        rendered(written),
        "Array([Integer(IntegerValue(3)), Array([Array([Symbol(\"@x\")]), Integer(IntegerValue(3))])])"
    );
    assert_eq!(
        rendered(round_trip),
        "Array([Integer(IntegerValue(3)), Integer(IntegerValue(3))])"
    );
    assert_eq!(
        rendered(removed),
        "Array([Integer(IntegerValue(3)), Integer(IntegerValue(3))])"
    );
    assert_eq!(
        rendered(absent),
        "Symbol(\"InstanceVariableNotFoundError\")"
    );
    assert_eq!(rendered(missing), "Nil");
}

#[test]
fn v358_reports_no_implicit_contract_parent_or_module_edge() {
    // IRIS-V1-TYPES-C043 forms Contract inheritance as a plain relation and
    // IRIS-V1-META-C078 adds a Module edge only through `mixin`, so a Contract
    // written without `extends` and a Module written without `mixin` have EMPTY
    // views. Neither view existed, so no row could observe the absence.
    let empty = "contract Plain { } module Empty { } [Plain.parents, Empty.modules]";
    // A declared parent and a declared edge DO appear.
    let parent = "contract A { } contract B extends A { } B.parents";
    let edge = "module A { } module B mixin A { } B.modules";

    // When / Then
    assert_eq!(rendered(empty), "Array([Array([]), Array([])])");
    assert_eq!(rendered(parent), "Array([Contract(ContractId(0))])");
    assert_eq!(rendered(edge), "Array([Symbol(\"A\")])");
}

#[test]
fn c006_loads_packages_in_dependency_order_on_one_runtime() {
    // IRIS-V1-META-C006 selects dependencies before initialization and
    // IRIS-V1-META-C017 initializes a dependency before its dependent, so
    // ordered packages share ONE runtime and a consumer reaches what its
    // dependency exported. No entry loaded more than one package at a time, so
    // no cross-package row could be driven at all.
    let dependency = (
        "org.dep".to_owned(),
        vec![(
            "src/main.ir".to_owned(),
            "export class Base { public fun tag() -> Symbol { :tag } }".to_owned(),
        )],
    );
    let consumer = (
        "org.app".to_owned(),
        vec![("src/main.ir".to_owned(), "import org.dep::Base".to_owned())],
    );

    // When
    let loaded = crate::load_package_tree(&[dependency, consumer], Some("Base.new().tag()"));

    // Then: the consumer's probe runs under its OWN package identity and still
    // reaches the dependency's exported Class.
    assert_eq!(
        loaded.map(|(_, observed)| format!("{observed:?}")),
        Ok("Some(Symbol(\"tag\"))".to_owned())
    );
}

#[test]
fn c064_appends_to_a_class_level_storage_slot() {
    // IRIS-V1-TYPES-C064 puts class-level storage on the Class or Module
    // OBJECT, so `S.e` names a slot read through a singleton accessor rather
    // than a lexical binding. `append` is routed by SYNTAX before its receiver
    // is evaluated, and that routing reached only the binding form, so a
    // class-level Array could be READ and never appended to.
    let module_slot = "module S { shared class property e: Array = [] } S.e.append(1); S.e";
    let class_slot = "class C { shared class property e: Array = [] } C.e.append(1); C.e";
    // A slot that is not an Array is a type failure, not a silent no-op.
    let wrong_type = "module S { shared class property e: Integer = 1 } S.e.append(1)";
    // An ordinary lexical binding is unaffected.
    let binding = "let a = []; a.append(1); a";

    // When / Then
    assert_eq!(
        rendered(module_slot),
        "Array([Nil, Array([Integer(IntegerValue(1))])])"
    );
    assert_eq!(
        rendered(class_slot),
        "Array([Nil, Array([Integer(IntegerValue(1))])])"
    );
    assert_eq!(rendered(wrong_type), "Runtime(Type)");
    assert_eq!(
        rendered(binding),
        "Array([Nil, Array([Integer(IntegerValue(1))])])"
    );
}

#[test]
fn c027_lets_an_escaping_closure_keep_a_body_local() {
    // IRIS-V1-META-C027 makes Class body locals ordinary lexical locals that
    // nested Closures may capture and escaping Closures may outlive, while
    // C025 keeps them from becoming properties or storage. This is
    // IRIS-V1-META-V344, and it needed the class-level append above to be
    // expressible in a package fixture at all.
    let escaped = "mut kept = []; class Box { let local = 7; let shadow = 9; \
                   kept.append({ local }); kept.append({ shadow }) } \
                   [kept[0].call(), kept[1].call(), Box.properties]";

    // When / Then: each Closure keeps its OWN local, and neither became a slot.
    assert_eq!(
        rendered(escaped),
        "Array([Integer(IntegerValue(7)), Integer(IntegerValue(9)), Array([])])"
    );
}
