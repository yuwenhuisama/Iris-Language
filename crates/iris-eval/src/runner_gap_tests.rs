use iris_runtime::{
    ArrayRef, BuiltinClass, ClassId, Kernel, KernelError, MethodBody, NativeSelector, Runtime,
    Selector, StaticSpine, Value as RuntimeValue, Visibility,
};

use super::{EvaluationError, evaluate};

#[test]
fn authored_array_convenience_methods_answer_their_documented_results() {
    // A length-only assertion would pass even if every method answered nil,
    // so each result is pinned to its VALUE. The receiver is restored to
    // [3, 1, 2, 2] by the push/pop pair before the reads below.
    let source = "let a = [3, 1, 2, 2]; let pushed = a.push(4); let popped = a.pop(); \
                  [a.map({ |x|; x + 1 }), a.select({ |x|; x > 1 }), a.reject({ |x|; x == 2 }), \
                   a.reduce(0, { |sum, x|; sum + x }), a.find({ |x|; x == 2 }), \
                   a.count({ |x|; x > 1 }), a.sum, a.min, a.max, a.sort, a.reverse, \
                   a.first, a.last, pushed.same?(a), popped, a.join(\"-\"), a.include?(2), \
                   a.index_of(2), a.take(2), a.drop(2), [1, 1, 2].uniq, [1, [2, [3]]].flatten, \
                   a.all?({ |x|; x > 0 }), a.any?({ |x|; x == 2 }), a.at(-1), a.to_string, \
                   [].first, [].last, [].all?({ |x|; x > 0 }), [].any?({ |x|; x > 0 })]";

    let integers = |values: &[i64]| {
        RuntimeValue::Array(ArrayRef::new(
            values
                .iter()
                .map(|value| RuntimeValue::Integer((*value as u64).into()))
                .collect(),
        ))
    };
    let integer = |value: u64| RuntimeValue::Integer(value.into());

    assert_eq!(
        evaluate(source),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            integers(&[4, 2, 3, 3]),
            integers(&[3, 2, 2]),
            integers(&[3, 1]),
            integer(8),
            integer(2),
            integer(3),
            integer(8),
            integer(1),
            integer(3),
            integers(&[1, 2, 2, 3]),
            integers(&[2, 2, 1, 3]),
            integer(3),
            integer(2),
            RuntimeValue::Bool(true),
            integer(4),
            RuntimeValue::Text("3-1-2-2".into()),
            RuntimeValue::Bool(true),
            integer(2),
            integers(&[3, 1]),
            integers(&[2, 2]),
            integers(&[1, 2]),
            integers(&[1, 2, 3]),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
            integer(2),
            RuntimeValue::Text("[3, 1, 2, 2]".into()),
            // An empty receiver answers nil rather than raising.
            RuntimeValue::Nil,
            RuntimeValue::Nil,
            // `all?` is vacuously true on an empty receiver, `any?` false.
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(false),
        ])))
    );
}

#[test]
fn authored_string_convenience_methods_answer_on_a_literal_receiver() {
    // The receiver is a LITERAL on purpose. A send on a literal used to stay in
    // the literal evaluator, whose selector table is tiny, so `"a".upcase()`
    // answered MessageNotFoundError while `let s = "a"; s.upcase()` succeeded.
    let source = "[\"ABC\".downcase, \"a,b\".split(\",\"), \" a \".trim(), \
                   \"abc\".replace(\"a\", \"z\"), \"abc\".starts_with?(\"ab\"), \
                   \"abc\".ends_with?(\"bc\"), \"abc\".contains?(\"b\"), \"abc\".upcase, \
                   \"ab\".chars, \"ab\".to_symbol]";

    let text = |value: &str| RuntimeValue::Text(value.into());

    assert_eq!(
        evaluate(source),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            text("abc"),
            RuntimeValue::Array(ArrayRef::new(vec![text("a"), text("b")])),
            text("a"),
            text("zbc"),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
            text("ABC"),
            RuntimeValue::Array(ArrayRef::new(vec![text("a"), text("b")])),
            RuntimeValue::Symbol("ab".into()),
        ])))
    );
}

#[test]
fn authored_collection_blocks_propagate_errors_unchanged() {
    // Given
    let source = "try { [1].map({ |x|; raise :array_failure }) } catch e { e }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("array_failure".into())));
}

#[test]
fn missing_numeric_selector_reports_receiver_class_and_selector() {
    // Given
    let source = "Integer(1).canonical_numeric_bytes()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::MessageNotFound {
            receiver_class: "Integer".into(),
            selector: "canonical_numeric_bytes".into(),
        })
    );
}

#[test]
fn known_numeric_selector_still_returns_its_stable_hash() {
    // Given
    let source = "Integer(1).hash";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Integer(17_824_117_788_395_916_856_u64.into()))
    );
}

#[test]
fn visibility_denial_does_not_report_message_not_found() {
    // Given
    let source = "class A { private fun secret() -> Integer { 1 } }; let a = A.new(); a.secret()";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::VisibilityDenied { .. }
            )
        ))
    ));
}

#[test]
fn user_defined_method_missing_handles_an_absent_ordinary_selector() {
    // Given
    let source = "class A { public fun method_missing(selector, arguments, block) -> Symbol { :handled } }; A.new().absent()";

    // When
    let result = crate::evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("handled".into())));
}

#[test]
fn default_method_missing_reports_message_not_found() {
    // Given
    let source = "class A {}; A.new().absent()";

    // When
    let result = crate::evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::MessageNotFound { receiver_class, selector })
            if receiver_class == "A" && selector == "absent"
    ));
}

#[test]
fn initialize_unqualified_self_send_constructs_the_instance() {
    // Given
    let source = "class A { public fun initialize() { m() } public fun m() -> Symbol { :initialized } }; A.new()";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(result, Ok(RuntimeValue::Object(_))));
}

#[test]
fn initialize_explicit_self_send_constructs_the_instance() {
    // Given
    let source = "class A { public fun initialize() { self.m() } public fun m() -> Symbol { :initialized } }; A.new()";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(result, Ok(RuntimeValue::Object(_))));
}

#[test]
fn user_class_name_is_not_constructor_syntax() {
    let source = "class A { }; A()";

    assert_eq!(evaluate(source), Err(EvaluationError::UnsupportedConstruct));
}

#[test]
fn initialize_self_send_propagates_its_raised_value_without_an_instance() {
    // Given
    let source = "class A { public fun initialize() { fail() } public fun fail() { raise :sentinel } }; A.new()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::Raised(RuntimeValue::Symbol(
            "sentinel".into()
        )))
    );
}

#[test]
fn initialize_missing_self_send_uses_default_method_missing() {
    // Given
    let source = "class A { public fun initialize() { absent() } }; A.new()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::MessageNotFound {
            receiver_class: "A".into(),
            selector: "absent".into(),
        })
    );
}

#[test]
fn stored_property_initializer_can_send_an_instance_method() {
    // Given
    let source = "class A { property value: Symbol = initial_value() public fun initial_value() -> Symbol { :ready } }; A.new().value";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("ready".into())));
}

#[test]
fn class_method_slot_operations_follow_d448_alias_remove_and_undef_rules() {
    // Given
    let source = "class Base { public fun f() -> Symbol { :base_f } public fun g() -> Symbol { :base_g } }; class A extends Base { public fun f() -> Symbol { :local } public fun method_missing(selector, arguments, block) -> Symbol { :missing } }; let ignored_alias = A.alias_method(:g, :f); let a = A.new(); let alias = a.g(); let ignored_remove = A.remove_method(:g); let removed = a.g(); let ignored_undef = A.undef_method(:f); [alias, removed, a.f()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("local".into()),
            RuntimeValue::Symbol("base_g".into()),
            RuntimeValue::Symbol("missing".into()),
        ])))
    );
}

#[test]
fn class_method_slot_operations_require_method_set_capability() {
    // Given
    let source =
        "class A meta deny method_set { public fun f() -> Symbol { :f } }; A.remove_method(:f)";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied {
                operation: iris_runtime::Capability::MethodSet,
                ..
            }
        ))
    ));
}

#[test]
fn reflection_object_ivar_operations_are_layered_under_reflection_object() {
    // Given
    let source = "class A { }; let a = A.new(); let missing = Reflection::Object.get_ivar(a, :@x); let written = Reflection::Object.set_ivar(a, :@x, :value); let names = Reflection::Object.list_ivars(a); let removed = Reflection::Object.remove_ivar(a, :@x); [missing, written, names, removed, Reflection::Object.get_ivar(a, :@x)]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Symbol("@x".into())])),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Nil,
        ])))
    );
}

#[test]
fn reflection_ivar_boundaries_report_the_specified_errors() {
    let source = "class A { }; let a = A.new(); [try { Reflection::Object.get_ivar(a, \"@x\") } catch e { e }, try { Reflection::Object.get_ivar(a, :\"@1x\") } catch e { e }, try { Reflection::Object.set_ivar(1, :@x, 1) } catch e { e }]";

    assert_eq!(
        evaluate(source),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("InvalidInstanceVariableNameError".into()),
            RuntimeValue::Symbol("InvalidInstanceVariableNameError".into()),
            RuntimeValue::Symbol("InstanceStateError".into()),
        ])))
    );
}

#[test]
fn type_identity_reports_package_and_a_stable_hash() {
    // Given: IRIS-V1-TYPES-C078 derives publishable nominal identity from
    // package ID, API major and qualified name, never from display name alone.
    let source = "class A { } class B { } [A.type.package(), A.type.hash() == A.type.hash(), A.type.hash() != B.type.hash()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("runtime-local".into()),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ])))
    );
}

#[test]
fn type_reflection_exposes_the_five_c075_queries() {
    // Given: IRIS-V1-TYPES-C075 requires at least `kind`, `arguments`,
    // `members`, `subtype?` and `assignable?` on a Type object.
    let source = "class A { public fun g() -> Integer { 1 } } class B extends A { } [A.type.kind(), A.type.arguments(), A.type.members(), B.type.subtype?(A.type), A.type.assignable?(B.type)]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("nominal".into()),
            RuntimeValue::Array(ArrayRef::new(Vec::new())),
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("to_bool".into()),
                RuntimeValue::Symbol("g".into()),
            ])),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ])))
    );
}

#[test]
fn reflection_module_invoke_reaches_a_module_method() {
    // Given: IRIS-V1-META-C118 names `Reflection::Module.invoke(method, receiver, args)`
    // as the Module-side counterpart of the Class form.
    let source = "module Mo { public fun h() -> Integer { 8 } }; let m = Reflection::Module.method(Mo, :h); Reflection::Module.invoke(m, Mo, [])";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(8_u64.into())));
}

#[test]
fn reflection_class_and_class_mixin_share_method_and_module_operations() {
    // Given
    let source = "module M { public fun m() -> Symbol { :module } }; class A mixin M { }; let reflected = Reflection::Class.method(A, :m); let direct = A.method(:m); let a = A.new(); let before = Reflection::Class.invoke(reflected, a, []); let same = Reflection::Class.invoke(direct, a, []); let ignored = A.remove_module(M); [before, same, ignored]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("module".into()),
            RuntimeValue::Symbol("module".into()),
            RuntimeValue::Nil,
        ])))
    );
}

#[test]
fn reflection_class_and_class_mixin_share_runtime_superclass_operations() {
    // Given
    let source = "class A { }; class B extends A { }; class Other { }; let first = Reflection::Class.set_superclass(B, Other); let ancestors = Reflection::Class.ancestors(B); let direct = B.set_superclass(A); [first, ancestors, direct, Reflection::Class.ancestors(B)]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            // These ids are ALLOCATION ORDER, not identities the spec fixes:
            // declared Classes are numbered after the builtin ones, so adding a
            // builtin Class shifts every declared id. What the row asserts is
            // the ANCESTOR RELATIONSHIP, `B` then its current superclass then
            // `Object`, which the ids below spell out positionally.
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Class(ClassId::new(8)),
                RuntimeValue::Class(ClassId::new(9)),
                RuntimeValue::Class(ClassId::new(0))
            ])),
            RuntimeValue::Nil,
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Class(ClassId::new(8)),
                RuntimeValue::Class(ClassId::new(7)),
                RuntimeValue::Class(ClassId::new(0))
            ])),
        ])))
    );
}

#[test]
fn runtime_superclass_change_rejects_retained_method_before_body_entry() {
    // Given
    let source = "mut log = []; class A { public fun m() -> Nil { log.append(:entered); raise :body } }; class B extends A { }; class Other { }; let method = Reflection::Class.method(A, :m); Reflection::Class.set_superclass(B, Other); Reflection::Class.invoke(method, B.new(), [])";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MethodBinding { .. }
            )
        ))
    ));
}

#[test]
fn reflection_class_rejects_protected_builtin_superclass_mutation() {
    // Given
    let source = "class A { }; Reflection::Class.set_superclass(Integer, A)";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::ProtectedSuperclass { .. }
        ))
    ));
}

#[test]
fn reflection_module_and_module_mixin_share_method_and_invoke_operations() {
    // Given
    let source = "module M { public fun m() -> Symbol { :module } }; class A mixin M { }; let direct = M.method(:m); let a = A.new(); M.invoke(direct, a, [])";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("module".into())));
}

#[test]
fn nested_reflection_module_declarations_and_mixin_qualified_names_parse() {
    // Given
    let source = "module Reflection::Class { }; module Reflection::Module { }; class A mixin Reflection::Class { }";

    // When
    let result = iris_parser::parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
}

#[test]
fn qualified_expression_sends_reach_reflection_and_nested_module_members() {
    // Given
    let reflection = "class A { }; Reflection::Object.list_ivars(A.new())";
    // C077 defaults a Module Method to private, so the external send needs it
    // declared public.
    let module = "module R::S { public module fun f() -> Symbol { :f } }; R::S.f()";

    let reflection_parse = iris_parser::parse(reflection);
    let module_parse = iris_parser::parse(module);
    assert!(reflection_parse.is_clean(), "{reflection_parse:#?}");
    assert!(module_parse.is_clean(), "{module_parse:#?}");

    // When
    let reflection_result = evaluate(reflection);
    let module_result = evaluate(module);

    // Then
    assert_eq!(
        reflection_result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![])))
    );
    assert_eq!(module_result, Ok(RuntimeValue::Symbol("f".into())));
}

#[test]
fn class_invoke_validates_binding_before_the_method_body_can_raise() {
    // Given
    let source = "class A { public fun m() -> Nil { raise :body } }; class B { }; let mm = A.method(:m); B.invoke(mm, B.new(), [])";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MethodBinding { .. }
            )
        ))
    ));
}

#[test]
fn reflection_operations_are_reachable_through_every_public_entry_point() {
    // Given
    let object = "class A { }; let a = A.new(); let first = Reflection::Object.set_ivar(a, :@x, :value); [Reflection::Object.list_ivars(a), Reflection::Object.get_ivar(a, :@x), first, Reflection::Object.remove_ivar(a, :@x)]";
    let class = "module M { public fun m() -> Symbol { :m } }; class A mixin M { }; let a = A.new(); let reflected = Reflection::Class.method(A, :m); let woven = A.method(:m); [Reflection::Class.invoke(reflected, a, []), A.invoke(woven, a, []), Reflection::Class.remove_module(A, :M)]";
    let module = "module M { public fun m() -> Symbol { :m } }; class A mixin M { }; let a = A.new(); let reflected = Reflection::Module.method(M, :m); let woven = M.method(:m); [Reflection::Module.invoke(reflected, a, []), M.invoke(woven, a, [])]";

    // When
    let object_result = evaluate(object);
    let class_result = evaluate(class);
    let module_result = evaluate(module);

    // Then
    assert_eq!(
        object_result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Symbol("@x".into())])),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Symbol("value".into()),
        ])))
    );
    assert_eq!(
        class_result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Nil,
        ])))
    );
    assert_eq!(
        module_result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Symbol("m".into()),
        ])))
    );
}

#[test]
fn retained_module_method_removal_fails_at_invocation_entry() {
    // Given
    let source = "class Base { public fun m() -> Symbol { :base } }; module M { override public fun m() -> Symbol { super() } }; class A extends Base mixin M { }; let method = M.method(:m); A.remove_module(M); A.invoke(method, A.new(), [])";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MethodBinding { .. }
            )
        ))
    ));
}

#[test]
fn rejects_identity_less_operands_with_identity_error() {
    // Given
    let source = "Integer(1) same? Integer(1)";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Err(EvaluationError::Runtime(KernelError::Identity)));
}

#[test]
fn sends_not_equal_for_nan_and_ordinary_operands() {
    // Given
    let source = "[Float64.nan == Float64.nan, Float64.nan != Float64.nan, 1 != 2]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ])))
    );
}

#[test]
fn array_append_mutates_an_unannotated_binding_in_order() {
    // Given
    let source =
        "mut log = []; let ignored_a = log.append(:a); let ignored_b = log.append(:b); log";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("a".into()),
            RuntimeValue::Symbol("b".into()),
        ])))
    );
}

#[test]
fn array_literal_append_grows_in_order_and_returns_nil() {
    // Given
    let source = "mut values = [:a]; [values.append(:b), values]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("a".into()),
                RuntimeValue::Symbol("b".into()),
            ])),
        ])))
    );
}

#[test]
fn array_append_accumulates_through_try_catch_and_finally() {
    // Given
    let source = "mut log = []; try { log.append(:try); raise :x } catch _ { log.append(:catch); :handled } finally { log.append(:finally) }; log";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("handled".into()),
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("try".into()),
                RuntimeValue::Symbol("catch".into()),
                RuntimeValue::Symbol("finally".into()),
            ])),
        ])))
    );
}

#[test]
fn array_append_does_not_add_array_add_or_size() {
    // Given
    let add = "[1] + [2]";
    let size = "[1, 2, 3].size";

    // When
    let add_result = evaluate(add);
    let size_result = evaluate(size);

    // Then `+` is genuinely NOT FOUND on an Array rather than a type mismatch:
    // IRIS-V1-RUNTIME-C005 makes a value with no dedicated builtin Class an
    // ordinary `Object`, so the send resolves a receiver and then reports the
    // missing selector, matching the `size` case below.
    assert!(matches!(
        add_result,
        Err(EvaluationError::Runtime(
            KernelError::MessageNotFound { .. }
        ))
    ));
    assert_eq!(
        size_result,
        Err(EvaluationError::MessageNotFound {
            receiver_class: "Array".into(),
            selector: "size".into(),
        })
    );
}

#[test]
fn identity_primitive_bypasses_replaced_equal_and_compare_slots()
-> Result<(), iris_runtime::KernelError> {
    // Given
    let mut registry = iris_runtime::ClassRegistry::new();
    let kernel = Kernel::new(&mut registry)?;
    let bool_class = kernel.class(BuiltinClass::Bool)?;
    registry.publish_method(
        bool_class,
        NativeSelector::Equal.id(),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    registry.publish_method(
        bool_class,
        NativeSelector::Compare.id(),
        MethodBody::new(1),
        Visibility::Public,
    )?;

    // When
    let result = Kernel::same_identity(&RuntimeValue::Bool(true), &RuntimeValue::Bool(true));

    // Then
    assert_eq!(result, Ok(true));
    Ok(())
}

#[test]
fn constructs_negative_float64_infinity_for_fused_arithmetic() {
    // Given
    let construction = "Float64(-Infinity)";
    let fused_arithmetic = "Float64.infinity.mul_add(1, Float64(-Infinity))";

    // When
    let constructed = evaluate(construction);
    let fused = evaluate(fused_arithmetic);

    // Then
    assert!(matches!(
        constructed,
        Ok(RuntimeValue::Float64(value)) if value.is_infinite() && value.is_sign_negative()
    ));
    assert!(matches!(
        fused,
        Ok(RuntimeValue::Float64(value)) if value.is_nan()
    ));
}

#[test]
fn rejects_bare_infinity_outside_float64_construction() {
    // Given
    let source = "Infinity";

    // When
    let result = evaluate(source);

    // Then bare `Infinity` names nothing, and IRIS-V1-CONTROL-C011 makes an
    // unresolved bare name a `NameError` whatever its spelling. It remains
    // reachable only through `Float64.infinity`.
    assert_eq!(result, Err(EvaluationError::NameError));
}

#[test]
fn source_method_returns_symbol_literal() {
    // Given
    let source = "class A { public fun m() { :added } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(result, Ok(value) if format!("{value:?}") == "Symbol(\"added\")"));
}

#[test]
fn source_if_returns_the_selected_branch_value() {
    // Given
    let true_symbol = "if true { :yes }";
    let missing_else = "if false { :yes }";
    let true_integer = "if true { 1 }";
    let true_else = "if true { 1 } else { 2 }";
    let false_else = "if false { 1 } else { 2 }";
    let else_if = "if false { 1 } else if false { 2 } else { 3 }";

    assert!(iris_parser::parse(true_symbol).program_accepted);

    // When
    let results = [
        evaluate(true_symbol),
        evaluate(missing_else),
        evaluate(true_integer),
        evaluate(true_else),
        evaluate(false_else),
        evaluate(else_if),
    ];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Symbol("yes".into())),
            Ok(RuntimeValue::Nil),
            Ok(RuntimeValue::Integer(1_u8.into())),
            Ok(RuntimeValue::Integer(1_u8.into())),
            Ok(RuntimeValue::Integer(2_u8.into())),
            Ok(RuntimeValue::Integer(3_u8.into())),
        ]
    );
}

#[test]
fn conditional_expressions_return_values_in_bindings_and_arrays() {
    // Given
    let source = "let bound = if true { :yes } else { :no }; [bound, if false { :yes } else { :no }, if false { :missing }, if false { :first } else if true { :second } else { :third }]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("yes".into()),
            RuntimeValue::Symbol("no".into()),
            RuntimeValue::Nil,
            RuntimeValue::Symbol("second".into()),
        ])))
    );
}

#[test]
fn conditional_expressions_are_evaluated_in_standalone_array_elements() {
    // Given
    let singleton = "[if true { :y } else { :n }]";
    let mixed = "[1, if true { :y } else { :n }]";

    // When
    let results = [evaluate(singleton), evaluate(mixed)];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("y".into())
            ]))),
            Ok(RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Symbol("y".into()),
            ]))),
        ]
    );
}

#[test]
fn conditional_expressions_use_to_bool_once_and_preserve_statement_behavior() {
    // Given
    let expression = "class Probe { shared mut @@n: Integer = 0; public fun to_bool() -> Bool { @@n = @@n + 1; false } class fun count() -> Integer { @@n } }; let value = if Probe.new() { :yes } else { :no }; [value, Probe.count(), if nil { :yes } else { :no }]";
    let statement = "if true { :yes } else { :no }";

    // When
    let results = [evaluate(expression), evaluate(statement)];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("no".into()),
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Symbol("no".into()),
            ]))),
            Ok(RuntimeValue::Symbol("yes".into())),
        ]
    );
}

#[test]
fn source_if_uses_truthiness_and_keeps_branches_scoped() {
    // Given
    let custom_false =
        "class A { public fun to_bool() -> Bool { false } }; if A.new() { :then } else { :else }";
    let non_bool = "class A { public fun to_bool() -> Bool { :not_bool } }; if A.new() { :then }";
    let skipped_branch = "class A { property count: Integer = 0; property fun count=(value: Integer) -> Integer { @count = value } }; let a = A.new(); if true { :selected } else { a.count = 1 }; a.count";
    let scoped_binding = "if true { let hidden = 1; hidden }; hidden";

    // When
    let custom_false_result = evaluate(custom_false);
    let non_bool_result = evaluate(non_bool);
    let skipped_branch_result = evaluate(skipped_branch);
    let scoped_binding_result = evaluate(scoped_binding);

    // Then
    assert_eq!(custom_false_result, Ok(RuntimeValue::Symbol("else".into())));
    assert_eq!(non_bool_result, Err(EvaluationError::TypeContractError));
    assert_eq!(
        skipped_branch_result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("selected".into()),
            RuntimeValue::Integer(0_u8.into()),
        ])))
    );
    // IRIS-V1-CONTROL-C011 makes an unresolved bare name a `NameError`, which
    // is what reading a branch-local binding from outside its branch produces.
    assert_eq!(scoped_binding_result, Err(EvaluationError::NameError));
}

#[test]
fn source_open_class_preserves_identity_and_updates_existing_instances() {
    // Given
    let source = "class A { public fun old() -> Integer { 1 } }; let before = A; let a = A.new(); open class A { public fun added() -> Integer { 2 } }; [before same? A, a.added()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Bool(true),
            RuntimeValue::Integer(2_u8.into()),
        ])))
    );
}

#[test]
fn reopen_override_replaces_method_for_new_send() {
    // Given
    let source = "class A { public fun m() { :old } }; let value = A.new(); open class A { override public fun m() { :new } }; value.m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("new".into())));
}

#[test]
fn reopen_preserves_class_identity() {
    // Given
    let source = "class A { public fun m() { :old } }; let before = A; open class A { override public fun m() { :new } }; before same? A";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Bool(true)));
}

#[test]
fn bound_method_captured_before_reopen_keeps_original_method() {
    // Given
    let source = "class A { public fun method() { :old } }; let obj = A.new(); let saved = obj.method; open class A { override public fun method() { :new } }; [saved same? saved, saved(), obj.method()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Bool(true),
            RuntimeValue::Symbol("old".into()),
            RuntimeValue::Symbol("new".into()),
        ])))
    );
}

#[test]
fn reopen_adding_method_preserves_existing_methods() {
    // Given
    let source = "class A { public fun old() { :old } }; let value = A.new(); open class A { public fun added() { :added } }; [value.old(), value.added()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("old".into()),
            RuntimeValue::Symbol("added".into()),
        ])))
    );
}

#[test]
fn meta_subclass_denial_rejects_child_before_publication() {
    // Given
    let source = "class Base meta deny subclass { }; class Child extends Base { }; 1";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
}

#[test]
fn meta_method_set_denial_blocks_a_reopen_but_subclass_denial_does_not() {
    // Given
    let denied =
        "class A meta deny method_set { }; open class A { public fun added() { :added } }; 1";
    let orthogonal = "class A meta deny subclass { }; open class A { public fun added() { :added } }; A.new().added()";

    // When
    let results = [evaluate(denied), evaluate(orthogonal)];

    // Then
    assert!(matches!(
        results[0],
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
    assert_eq!(results[1], Ok(RuntimeValue::Symbol("added".into())));
}

#[test]
fn meta_denial_is_inherited_and_a_child_can_only_narrow() {
    // Given
    let inherited = "class Base meta deny method_set { }; class Child extends Base { }; open class Child { public fun added() { :added } }; 1";
    let narrowed = "class Base { }; class Child extends Base meta deny method_set { }; open class Child { public fun added() { :added } }; 1";

    // When
    let results = [evaluate(inherited), evaluate(narrowed)];

    // Then
    assert!(matches!(
        results[0],
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
    assert!(matches!(
        results[1],
        Err(EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
}

#[test]
fn class_without_meta_clause_keeps_default_capabilities() {
    // Given
    let source = "class A { }; open class A { public fun added() { :added } }; A.new().added()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("added".into())));
}

#[test]
fn meta_policy_is_immutable_across_reopen() {
    // Given
    let source = "class A meta deny method_set { }; open class A meta deny subclass { }; 1";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(result, Err(EvaluationError::Class(_))));
}

#[test]
fn reopen_replacement_without_override_is_rejected() {
    // Given
    let source = "class A { public fun m() { :old } }; open class A { public fun m() { :new } }";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(
            iris_runtime::ClassError::OverrideRequired { .. }
        ))
    ));
}

#[test]
fn source_class_fun_dispatches_on_the_class_object() {
    // Given
    let source = "class A { class fun build() -> Integer { 7 } }; A.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(7_u8.into())));
}

#[test]
fn source_class_object_dispatch_inherits_singleton_methods_from_runtime_superclasses() {
    // Given
    let source = "class A { class fun build() { :class } }; class B extends A { }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}

#[test]
fn source_class_object_dispatch_keeps_direct_singleton_method_lookup() {
    // Given
    let source = "class A { class fun build() { :class } }; A.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}

#[test]
fn source_class_object_dispatch_prefers_its_own_singleton_method_over_an_inherited_one() {
    // Given
    let source = "class A { class fun build() { :parent } }; class B extends A { class fun build() { :child } }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("child".into())));
}

#[test]
fn source_class_object_super_continues_after_the_singleton_method_owner() {
    // Given
    let source = "class A { class fun build() { :parent } }; class B extends A { class fun build() { super() } }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("parent".into())));
}

#[test]
fn source_class_object_dispatch_reports_missing_selector_after_superclass_chain() {
    // Given
    let source = "class A { class fun build() { :class } }; class B extends A { }; B.unknown()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::MessageNotFound {
            receiver_class: "Class".into(),
            selector: "unknown".into(),
        })
    );
}

#[test]
fn source_property_getter_and_explicit_setter_return_distinct_method_results() {
    // Given
    let source = "class A { property fun name() { :get } property fun name=(value) { :set } }; let a = A.new(); [a.name, a.name = 1]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("get".into()),
            RuntimeValue::Symbol("set".into()),
        ])))
    );
}

#[test]
fn builtin_class_property_getter_replacement_is_observed() {
    // Given
    let source = "open class Float64 { override public property fun infinity() -> Float64 { 2.0f64 } }; Float64.infinity";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Float64(2.0)));
}

#[test]
fn builtin_value_property_setter_is_reachable_without_mutating_the_receiver() {
    // Given
    let source = "mut log = []; open class Integer { public property fun px=(value: Integer) -> Integer { log.append(value); value } }; let n = 1; [n.px = 2, n, log]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Integer(2_u8.into())])),
        ])))
    );
}

#[test]
fn class_property_setter_records_without_creating_an_implicit_backing_slot() {
    // Given
    let source = "mut log = []; open class Float64 { override public property fun infinity() -> Float64 { 2.0f64 } public property fun infinity=(value: Float64) -> Float64 { log.append(value); value } }; let assigned = Float64.infinity = 3.0f64; [Float64.infinity, assigned, log]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Float64(2.0),
            RuntimeValue::Float64(3.0),
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Float64(3.0)])),
        ])))
    );
}

#[test]
fn raw_ivar_forms_remain_parse_diagnostics() {
    // Given
    let sources = [
        "let n = 1; n.@x = 2",
        "class A { }; let obj = A.new(); obj.@x = 2",
        "class A { }; A.@@x = 2",
    ];

    // When
    let results = sources.map(evaluate);

    // Then
    assert_eq!(
        results,
        [
            Err(EvaluationError::ParseDiagnostic),
            Err(EvaluationError::ParseDiagnostic),
            Err(EvaluationError::ParseDiagnostic),
        ]
    );
}

#[test]
fn meta_denied_instance_state_rejects_first_raw_ivar_assignment() {
    // Given
    let source = "class A meta deny instance_state { public property fun px=(value: Integer) -> Integer { @x = value } }; let a = A.new(); a.px = 2";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(result, Err(EvaluationError::Construction(_))));
}

#[test]
fn ordinary_instance_state_assignment_still_creates_a_raw_ivar() {
    // Given
    let source = "class A { public property fun px=(value: Integer) -> Integer { @x = value } public property fun px() -> Integer { @x } }; let a = A.new(); [a.px = 2, a.px]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ])))
    );
}

#[test]
fn raw_ivar_assignment_on_a_value_receiver_reports_instance_state() {
    // Given
    let integer = "open class Integer { public property fun px=(value: Integer) -> Integer { @x = value } }; let n = 1; n.px = 2";
    let float32 = "open class Float32 { public property fun px=(value: Integer) -> Integer { @x = value } }; let f = 1.0f32; f.px = 2";

    // When
    let results = [evaluate(integer), evaluate(float32)];

    // Then
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(EvaluationError::Construction(_))))
    );
}

#[test]
fn raw_ivar_read_returns_nil_without_materializing_value_receiver_state() {
    // Given
    let source =
        "open class Integer { public property fun px() -> Nil { @x } }; let n = 1; [n.px, n.px]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Nil
        ])))
    );
}

#[test]
fn source_member_read_creates_a_fresh_bound_method_while_parenthesized_send_invokes() {
    // Given
    let member_read = "class A { public fun m() -> Integer { 1 } }; let a = A.new(); a.m";
    let identity_read =
        "class A { public fun m() -> Integer { 1 } }; let a = A.new(); a.m same? a.m";
    let invocation = "class A { public fun m() -> Integer { 1 } }; let a = A.new(); a.m()";

    // When
    let read = evaluate(member_read);
    let identity = evaluate(identity_read);
    let called = evaluate(invocation);

    // Then
    assert!(matches!(read, Ok(value) if format!("{value:?}").contains("BoundMethod")));
    assert_eq!(identity, Ok(RuntimeValue::Bool(false)));
    assert_eq!(called, Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn source_single_mixin_class_body_keeps_its_own_methods() {
    // Given: the mixed-in Module is DECLARED. This test previously named an
    // undeclared `A`, which composed silently; IRIS-V1-META-C054 validates
    // composition before publication, so that name is now refused and the
    // test states the case it was actually written to cover.
    let source = "module A { } class C mixin A { public fun t() -> Integer { 1 } }; C.new().t()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn property_assignment_returns_its_setter_result_while_raw_ivar_assignment_returns_stored_value()
-> Result<(), iris_runtime::ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)?;
    let property = Selector::new(1);
    runtime.registry_mut().publish_method(
        class,
        property,
        MethodBody::new(1),
        Visibility::Public,
    )?;
    let instance = runtime.allocate(class)?;
    let stored = RuntimeValue::Integer(4_u8.into());
    let setter_result = RuntimeValue::Symbol("set".into());

    // When
    let property_result = runtime.assign_property(
        instance,
        property,
        stored.clone(),
        |_runtime, _method, _receiver, _arguments| Ok(setter_result.clone()),
    )?;
    let ivar_result = runtime.assign_raw_ivar(instance, Selector::new(2), stored.clone())?;

    // Then
    assert_eq!(property_result, setter_result);
    assert_eq!(ivar_result, stored);
    Ok(())
}

#[test]
fn source_super_selects_the_next_method_and_reports_typed_no_successor() {
    // Given
    let source = "class B { public fun m() -> Integer { 1 } }; class C extends B { public fun m() -> Integer { super() } }; C.new().m()";
    let no_successor = "class A { public fun m() -> Integer { super() } }; A.new().m()";

    // When
    let result = evaluate(source);
    let missing = evaluate(no_successor);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(1_u8.into())));
    assert!(matches!(
        missing,
        Err(EvaluationError::Runtime(KernelError::Dispatch(
            iris_runtime::DispatchError::NoSuperMethod { .. }
        )))
    ));
}

#[test]
fn source_bare_super_is_rejected() {
    // Given
    let source = "class A { public fun m() -> Integer { super } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Runtime(KernelError::Dispatch(
            iris_runtime::DispatchError::InvalidSuper { .. }
        )))
    ));
}

#[test]
fn source_mixins_resolve_in_reverse_declaration_order() {
    // Given
    let source = "class A { public fun m() -> Integer { 1 } }; class B { public fun m() -> Integer { 2 } }; class C mixin A, B { }; C.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(2_u8.into())));
}

#[test]
fn source_typeof_annotation_accepts_the_operand_static_type() {
    // Given
    let source = "let a = 1; let b: typeof(a) = 2; b";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(2_u8.into())));
}

#[test]
fn source_module_fun_dispatches_on_the_module_object() {
    // Given
    // IRIS-V1-RUNTIME-C077 defaults a Module Method to PRIVATE, so an external
    // `M.helper()` send needs the Method declared public. Before the visibility
    // check existed, this passed with the default private declaration.
    let source = "module M { public module fun helper() -> Integer { 3 } }; M.helper()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(3_u8.into())));
}

#[test]
fn source_stored_property_uses_its_raw_ivar_backing_slot() {
    // Given
    let source = "class A { property name: Integer = 5 }; A.new().name";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(5_u8.into())));
}

#[test]
fn source_omitted_return_type_remains_dynamic() {
    // Given
    let source = "class A { public fun m() { 42 } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(42_u8.into())));
}

#[test]
fn source_decorated_class_publishes_and_evaluates_its_method() {
    // Given
    let source = "@logged() class A { public fun m() -> Integer { 1 } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn raise_propagates_as_a_typed_outcome_with_its_original_symbol() {
    // Given
    let source = "raise :sentinel";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::Raised(RuntimeValue::Symbol(
            "sentinel".into()
        )))
    );
}

#[test]
fn try_catch_binds_and_returns_the_raised_value() {
    // Given
    let source = "try { raise :boom } catch error { error }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("boom".into())));
}

#[test]
fn typed_catch_filter_leaves_non_matching_raise_unhandled() {
    // Given
    let source = "try { raise :boom } catch error: Integer { error }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::Raised(RuntimeValue::Symbol("boom".into())))
    );
}

#[test]
fn first_matching_catch_runs_before_later_matches() {
    // Given
    let source = "try { raise :boom } catch first { :first } catch second { :second }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("first".into())));
}

#[test]
fn finally_preserves_normal_and_handled_results() {
    // Given
    let normal = "try { :value } finally { :ignored }";
    let raised = "try { raise :boom } catch error { error } finally { :ignored }";

    // When
    let results = [evaluate(normal), evaluate(raised)];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Symbol("value".into())),
            Ok(RuntimeValue::Symbol("boom".into())),
        ]
    );
}

#[test]
fn method_raise_propagates_through_its_call() {
    // Given
    let source = "class A { public fun boom() { raise :sentinel } }; A.new().boom()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::Raised(RuntimeValue::Symbol(
            "sentinel".into()
        )))
    );
}

#[test]
fn initialize_raise_propagates_and_returns_no_instance() {
    // Given
    let source = "class A { public fun initialize() { raise :sentinel } }; A.new()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Err(EvaluationError::Raised(RuntimeValue::Symbol(
            "sentinel".into()
        )))
    );
}

#[test]
fn catch_receives_a_value_raised_by_initialize() {
    // Given
    let source = "class A { public fun initialize() { raise :sentinel } }; try { A.new() } catch error { error }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("sentinel".into())));
}

#[test]
fn escaped_receiver_from_failed_initialize_remains_usable() {
    // Given
    let source = "mut escaped = nil; class A { public fun initialize() { escaped = self; raise :sentinel } }; try { A.new() } catch error { escaped.to_bool() }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Bool(true)));
}

#[test]
fn ordinary_object_to_bool_returns_true_and_drives_if() {
    // Given
    let direct = "class A { }; A.new().to_bool()";
    let conditional = "class A { }; if A.new() { :then } else { :else }";

    // When
    let results = [evaluate(direct), evaluate(conditional)];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Bool(true)),
            Ok(RuntimeValue::Symbol("then".into())),
        ]
    );
}

#[test]
fn builtin_to_bool_methods_dispatch_per_c094() {
    // Given
    let source = "[nil.to_bool(), false.to_bool(), true.to_bool()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
        ])))
    );
}

#[test]
fn user_defined_to_bool_overrides_the_installed_default_and_drives_if() {
    // Given
    let direct = "class A { public fun to_bool() -> Bool { false } }; A.new().to_bool()";
    let conditional =
        "class A { public fun to_bool() -> Bool { false } }; if A.new() { :yes } else { :no }";

    // When
    let results = [evaluate(direct), evaluate(conditional)];

    // Then
    assert_eq!(
        results,
        [
            Ok(RuntimeValue::Bool(false)),
            Ok(RuntimeValue::Symbol("no".into())),
        ]
    );
}

#[test]
fn method_assignment_updates_an_enclosing_mutable_binding() {
    // Given
    let source = "mut value = 1; class A { public fun update() -> Integer { value = 2 } }; A.new().update(); value";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ])))
    );
}

#[test]
fn method_assignment_rejects_an_enclosing_immutable_binding() {
    // Given
    let source =
        "let value = 1; class A { public fun update() -> Integer { value = 2 } }; A.new().update()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Err(EvaluationError::ImmutableBinding));
}

#[test]
fn catch_binding_is_immutable() {
    // Given
    let source = "try { raise :boom } catch error { error = :other }";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Err(EvaluationError::ImmutableBinding));
}

#[test]
fn c098_protects_only_a_declared_ancestor_carrying_a_static_spine_fact() {
    // IRIS-V1-TYPES-C098 fixes WHICH declared ancestors the D-174 bound
    // protects: those carrying a static spine fact in the D-173 sense. Dropping
    // one falsifies a static promise and C045 requires refusal BEFORE
    // publication, which V201 observes by requiring the subtype fact to hold.
    let protected = "contract Walks { fun walk() } \
                     class Animal for Walks { public impl fun walk() -> Nil { nil } } \
                     class Dog extends Animal { } \
                     let refused = try { Reflection::Class.set_superclass(Dog, Object) } catch e { e }; \
                     [refused, Dog.type.subtype?(Animal.type)]";
    assert_eq!(
        evaluate(protected),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("TypeContractError".into()),
            RuntimeValue::Bool(true),
        ])))
    );

    // An ancestor carrying no such fact is deliberately NOT protected: D-104
    // and IRIS-V1-RUNTIME-C015 own that case and raise MethodBindingError at
    // reflective invocation entry instead, which is what RUNTIME-V014 observes.
    let unprotected = "class A { public fun m() -> Nil { nil } } \
                       class B extends A { } class Other { } \
                       try { Reflection::Class.set_superclass(B, Other) } catch e { e }";
    assert_eq!(evaluate(unprotected), Ok(RuntimeValue::Nil));

    // Narrowing ancestry by inserting a Class that still reaches every
    // protected ancestor is permitted, since every static subtype assumption
    // survives.
    let narrowed = "contract Walks { fun walk() } \
                    class Animal for Walks { public impl fun walk() -> Nil { nil } } \
                    class Mammal extends Animal { } class Dog extends Animal { } \
                    try { Reflection::Class.set_superclass(Dog, Mammal) } catch e { e }";
    assert_eq!(evaluate(narrowed), Ok(RuntimeValue::Nil));
}

#[test]
fn d175_validates_a_contract_requirement_satisfied_by_a_mixed_in_member() {
    // D-175 recomputes MRO and verifies declared Contract requirements BEFORE
    // commit, so a requirement satisfied by a MIXED-IN Module member is checked
    // too. D-173 puts the contract-visible SIGNATURE in the static spine, so
    // `draw(String)` conflicts with a declared `draw(Integer)` even though the
    // arities agree, which V202 observes.
    let conflicting = "contract C { fun draw(n: Integer) -> Nil } \
                       module Painter { public fun draw(s: String) -> Nil { nil } } \
                       class A for C { } open class A mixin Painter { } A";
    assert_eq!(
        evaluate(conflicting),
        Err(EvaluationError::TypeContractError)
    );

    // A composed member whose signature matches publishes normally.
    let matching = "contract C { fun draw(n: Integer) -> Nil } \
                    module Painter { public fun draw(n: Integer) -> Nil { nil } } \
                    class A for C { } open class A mixin Painter { } A.new().draw(1)";
    assert_eq!(evaluate(matching), Ok(RuntimeValue::Nil));

    // An unannotated parameter position states nothing and is left alone rather
    // than treated as a mismatch.
    let unannotated = "contract C { fun draw(n: Integer) -> Nil } \
                       module Painter { public fun draw(n) -> Nil { nil } } \
                       class A for C { } open class A mixin Painter { } A.new().draw(1)";
    assert_eq!(evaluate(unannotated), Ok(RuntimeValue::Nil));
}

#[test]
fn d212_withdraws_a_module_whose_initializer_raised() {
    // D-212 fails a Module whose initializer raises and leaves its status
    // `not_published`. The name is registered BEFORE the body runs so a
    // declaration can reach the Module being defined, so a failed initializer
    // must withdraw it rather than leave a half-initialized Module observable.
    let source = "module M { shared class property first: Integer = 1 \
                  raise :stop \
                  shared class property second: Integer = 2 } M";
    let (outcome, published) = crate::evaluate_with_class_publication(source, "M");
    assert_eq!(
        outcome,
        Err(EvaluationError::Raised(RuntimeValue::Symbol("stop".into())))
    );
    assert!(!published, "a failed Module initializer publishes nothing");

    // A Module whose body completes publishes normally.
    let succeeds = "module M { shared class property first: Integer = 1 } M";
    let (_, published) = crate::evaluate_with_class_publication(succeeds, "M");
    assert!(published);
}

#[test]
fn d207_revalidates_interned_closed_constructions_when_a_definition_opens() {
    // D-207 opens a generic definition by building a candidate definition PLUS
    // substituted candidate revisions for every already-interned closed
    // construction, validating all of them as one transaction and rolling
    // everything back on any closed failure. A reopen recorded no bounds, so a
    // `where` clause added on open was never validated against the
    // constructions that already exist.
    let violated = "contract Show { fun show() -> Symbol } \
                    class Str for Show { public impl fun show() -> Symbol { :s } } \
                    class Box<T> { }; let a = Box<Str>; let b = Box<Integer>; \
                    open class Box<T> where T: Show { }; [a, b]";
    assert_eq!(evaluate(violated), Err(EvaluationError::TypeContractError));

    // Every interned construction satisfying the added bound publishes.
    let satisfied = "contract Show { fun show() -> Symbol } \
                     class Str for Show { public impl fun show() -> Symbol { :s } } \
                     class Box<T> { }; let a = Box<Str>; \
                     open class Box<T> where T: Show { }; a";
    assert!(evaluate(satisfied).is_ok());

    // With no interned construction there is nothing to revalidate, so even an
    // uninhabitable bound publishes; it fails later at materialization.
    let none = "class Box<T> { }; open class Box<T> where T: Never { }; Box";
    assert!(evaluate(none).is_ok());
}

#[test]
fn d241_composes_a_contract_view_hash_from_its_components() {
    // D-241 uses BLAKE3 derive-key mode with context
    // `Iris Language v1 contract view hash` and input
    // `receiver_public_hash_u64_le || contract_type_hash_u64_le`. The expected
    // value below was computed INDEPENDENTLY from that specification rather
    // than read back from this implementation.
    let source = "contract C { fun m() } \
                  class A for C { public impl fun m() -> Nil { nil } \
                  public fun hash() -> Integer { 1 } } \
                  (A.new() as C).hash()";
    assert_eq!(
        evaluate(source),
        Ok(RuntimeValue::Integer(3192709805854531430_u64.into()))
    );

    // C032 forwards an ordinary `view.member()` to the receiver, but D-241
    // gives the view its OWN hash, so `hash` must not forward: forwarding made
    // a view hash equal to its receiver's and dropped the Contract component.
    let distinct = "contract C { fun m() } \
                    class A for C { public impl fun m() -> Nil { nil } } \
                    let o = A.new(); (o as C).hash() != o.hash()";
    assert_eq!(evaluate(distinct), Ok(RuntimeValue::Bool(true)));
}

#[test]
fn d242_makes_a_named_contract_type_hash_nominal() {
    // D-242 derives the hash from package identity, qualified name and major
    // version, NOT from structural member shape, so two Contracts with
    // identical declarations stay distinct.
    let distinct = "contract C { fun m() } contract D { fun m() } C.hash() != D.hash()";
    assert_eq!(evaluate(distinct), Ok(RuntimeValue::Bool(true)));

    // The same Contract hashes stably within a runtime.
    let stable = "contract C { fun m() } C.hash() == C.hash()";
    assert_eq!(evaluate(stable), Ok(RuntimeValue::Bool(true)));
}

#[test]
fn open_module_adds_members_to_the_existing_module() {
    // `module_decl ::= "open"? "module" ...` admits the marker, but the parser
    // never dispatched it, so `open module` failed to parse at all. An open
    // revision adds members to the EXISTING Module rather than defining a
    // second one, which V416 observes across two files of one package.
    let source = "module M { public fun one() -> Integer { 1 } } \
                  open module M { public fun two() -> Integer { 2 } } \
                  [M.one(), M.two()]";
    assert_eq!(
        evaluate(source),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ])))
    );
}

#[test]
fn c023_composition_changes_join_the_open_transaction() {
    // C023 targets the CURRENT transaction candidate, so a composition change
    // JOINS an open transaction. Publishing directly advanced the active
    // revision past the base every staged candidate recorded, so any
    // composition change inside an open block failed the C039 base-revision
    // check at commit with MetaTransactionConflictError.
    let base = "module A { public fun tag() -> Symbol { :a } } \
                module B { public fun tag() -> Symbol { :b } } \
                class H mixin A, B { } ";

    // The declared edge list is [A, B] and the last edge wins dispatch.
    // Removing A and re-including it makes it [B, A], so A now wins.
    let reordered =
        format!("{base} H.open() {{ |t| t.remove_module(:A); t.add_module(:A) }}; H.new().tag()");
    assert_eq!(
        evaluate(&reordered),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("a".into()),
        ])))
    );

    // D-175 recomputes MRO before commit and one Module holds one position in
    // it, so including a Module already composed is a no-op. Appending a
    // duplicate edge let a redundant `add_module` silently reorder dispatch.
    let redundant = format!("{base} H.open() {{ |t| t.add_module(:A) }}; H.new().tag()");
    assert_eq!(
        evaluate(&redundant),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("b".into()),
        ])))
    );
}

#[test]
fn c046_marks_only_conditional_body_additions_dynamic_only() {
    // C022 makes a Class body an executable construction transaction, but a
    // conditional branch in one was an unsupported construct, so the whole
    // fixture shape V345 and V427 use could not run at all.
    let conditional = "class Box { if true { self.define_method(:extra) { :extra } } } \
                       Box.new().extra()";
    assert_eq!(
        evaluate(conditional),
        Ok(RuntimeValue::Symbol("extra".into()))
    );

    // C046 makes a CONDITIONAL addition dynamic-only and C047 requires that
    // status to be recorded. An unconditional addition is ordinary declarative
    // API, so the marker must come from the conditionality, not from
    // define_method itself.
    let marked = "class Box { if true { self.define_method(:extra) { :extra } } } \
                  Reflection::Class.method(Box, :extra).source[3]";
    assert_eq!(
        evaluate(marked),
        Ok(RuntimeValue::Symbol("dynamic-only".into()))
    );

    let unconditional = "class Box { self.define_method(:extra) { :extra } } \
                         Reflection::Class.method(Box, :extra).source[3]";
    assert_eq!(
        evaluate(unconditional),
        Ok(RuntimeValue::Symbol("static".into()))
    );

    // The false branch stages nothing, and an `else` branch is taken normally.
    let not_taken = "class Box { if false { self.define_method(:extra) { :extra } } } \
                     try { Box.new().extra() } catch e { e }";
    assert_eq!(
        evaluate(not_taken),
        Ok(RuntimeValue::Symbol("MessageNotFound".into()))
    );

    let otherwise = "class Box { if false { 1 } else { self.define_method(:extra) { :other } } } \
                     Box.new().extra()";
    assert_eq!(
        evaluate(otherwise),
        Ok(RuntimeValue::Symbol("other".into()))
    );
}

#[test]
fn c099_keeps_contract_slots_in_their_own_namespace() {
    // C099 gives qualified Contract slots their OWN probe and forbids merging
    // the ordinary and qualified namespaces. Its receiver is a Contract VIEW,
    // so the Contract answers `view(Class)` to produce one.
    let base = "contract Named { fun name() -> Symbol } \
                class Box for Named { public impl fun name() -> Symbol { :box } \
                public fun method_missing(s) -> Nil { raise :called } } ";

    // C098 forbids consulting method_missing as a probe, so a Class defining
    // it still answers false for a member it does not have.
    assert_eq!(
        evaluate(&format!("{base} Box.new().respond_to?(:ghost)")),
        Ok(RuntimeValue::Bool(false))
    );

    assert_eq!(
        evaluate(&format!(
            "{base} Named.view(Box).respond_to_contract?(:name)"
        )),
        Ok(RuntimeValue::Bool(true))
    );
    assert_eq!(
        evaluate(&format!(
            "{base} Named.view(Box).respond_to_contract?(:ghost)"
        )),
        Ok(RuntimeValue::Bool(false))
    );
}

#[test]
fn c097_class_view_exposes_its_required_members() {
    // C097 fixes the minimal Class reflection view. `package`, `static_spine`,
    // `runtime_superclass`, `mro` and `meta_capabilities` were all missing, so
    // V424 could reflect only part of the surface the clause requires.
    // A denied `method_set` also forbids DECLARING a Method, so the denial
    // fixture carries no members of its own.
    let base = "class Box meta deny method_set { } ";

    // C081 fixes the capability vocabulary and its reporting order, so the
    // denied set is read through the same ordered accessor V360 observes.
    assert_eq!(
        evaluate(&format!("{base} Box.meta_capabilities")),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("method_set".into())
        ])))
    );

    // A Class with no denials reports an empty set rather than the vocabulary.
    assert_eq!(
        evaluate("class Box { } Box.meta_capabilities"),
        Ok(RuntimeValue::Array(ArrayRef::new(Vec::new())))
    );

    // The Method view members C097 lists alongside `source`.
    let method = "class Box { public fun show() -> Nil { nil } } \
                  let m = Reflection::Class.method(Box, :show); [m.selector, m.visibility]";
    assert_eq!(
        evaluate(method),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("show".into()),
            RuntimeValue::Symbol("public".into()),
        ])))
    );
}

#[test]
fn c049_requires_import_authorization_for_a_second_replacement() {
    // C049 requires import-site replacement authorization before a direct
    // import may replace an already merged static member, and D-230 authorizes
    // only the replacements the source marked. One extension ESTABLISHES the
    // member; a second replacing the same one is what needs the marker.
    let base = (
        "main".to_owned(),
        "class Base { public fun tag() -> Symbol { :base } }".to_owned(),
    );
    let first = (
        "first".to_owned(),
        "open class Base { public override fun tag() -> Symbol { :first } }".to_owned(),
    );
    let unauthorized = (
        "second".to_owned(),
        "open class Base { public override fun tag() -> Symbol { :second } }".to_owned(),
    );
    let authorized = (
        "second".to_owned(),
        "override import org.x.first \
         open class Base { public override fun tag() -> Symbol { :second } }"
            .to_owned(),
    );

    let refused = crate::load_package_with_probe(
        "org.x",
        &[base.clone(), first.clone(), unauthorized],
        Some("Base.new().tag()"),
    );
    assert_eq!(
        refused.err(),
        Some(EvaluationError::ImportReplacementAuthorization)
    );

    let accepted = crate::load_package_with_probe(
        "org.x",
        &[base, first, authorized],
        Some("Base.new().tag()"),
    );
    assert_eq!(
        accepted.map(|(_, observed)| observed),
        Ok(Some(RuntimeValue::Symbol("second".into())))
    );
}

#[test]
fn c049_grants_authorization_per_import_site() {
    // C049 authorizes a replacement at the IMPORT SITE, so a marker in one
    // source cannot authorize a DIFFERENT source's replacement. A single global
    // flag let the first extension's marker cover the second's unmarked one.
    let base = (
        "base".to_owned(),
        "class Base { public fun tag() -> Symbol { :base } }".to_owned(),
    );
    let first = (
        "first".to_owned(),
        "override import org.x.base \
         open class Base { public override fun tag() -> Symbol { :first } }"
            .to_owned(),
    );
    let unmarked_second = (
        "second".to_owned(),
        "open class Base { public override fun tag() -> Symbol { :second } }".to_owned(),
    );

    let refused = crate::load_package_with_probe(
        "org.x",
        &[base.clone(), first.clone(), unmarked_second],
        Some("Base.new().tag()"),
    );
    assert_eq!(
        refused.err(),
        Some(EvaluationError::ImportReplacementAuthorization),
        "one source's marker must not authorize another source's replacement"
    );

    // The FIRST reopen of a member establishes it, so a single marked
    // extension over an origin needs no prior authorization.
    let single = crate::load_package_with_probe("org.x", &[base, first], Some("Base.new().tag()"));
    assert_eq!(
        single.map(|(_, observed)| observed),
        Ok(Some(RuntimeValue::Symbol("first".into())))
    );
}

#[test]
fn c046_accepts_a_contract_body_replacement_marked_impl_alone() {
    // C046 writes BOTH modifiers only when the declaration also replaces an
    // INHERITED or Module Method. A reopen passed requires_override
    // unconditionally, so replacing a Class's OWN Contract implementation
    // demanded `override impl` when C046 asks only for `impl`. V436's
    // compatible body-only replacement is exactly that shape.
    let compatible = "contract D { fun draw(n: Integer) -> String } \
                      class C for D { public impl fun draw(n: Integer) -> String { \"old\" } } \
                      open class C { public impl fun draw(n: Integer) -> String { \"new\" } } \
                      C.new().draw(1)";
    assert_eq!(evaluate(compatible), Ok(RuntimeValue::Text("new".into())));

    // A Contract-VISIBLE signature change is still refused, so the relaxation
    // does not weaken C045.
    let incompatible = "contract D { fun draw(n: Integer) -> String } \
                        class C for D { public impl fun draw(n: Integer) -> String { \"old\" } } \
                        open class C { public impl fun draw(s: String) -> String { \"x\" } } C";
    assert_eq!(
        evaluate(incompatible),
        Err(EvaluationError::TypeContractError)
    );

    // An ordinary member replacement still requires `override`.
    let ordinary = "class C { public fun m() -> Symbol { :old } } \
                    open class C { public fun m() -> Symbol { :new } } C";
    assert!(matches!(
        evaluate(ordinary),
        Err(EvaluationError::Class(
            iris_runtime::ClassError::OverrideRequired { .. }
        ))
    ));
}

#[test]
fn c095_returns_a_filtered_immutable_reflection_view() {
    // C095 returns PERMISSION-FILTERED IMMUTABLE metadata, but `methods`
    // returned a mutable Array listing private members too.
    let filtered = "class Box { public fun show() -> Nil { nil } \
                    private fun hide() -> Nil { nil } } Box.methods";
    assert_eq!(
        evaluate(filtered),
        Ok(RuntimeValue::ReadonlyArray(vec![
            RuntimeValue::Symbol("to_bool".into()),
            RuntimeValue::Symbol("show".into()),
        ]))
    );

    // `append` routes by SYNTAX before its receiver is evaluated, so a view
    // held in a BINDING bypassed the send guard and reported a type failure
    // rather than the mutation refusal D-142 requires.
    let bound = "class Box { public fun show() -> Nil { nil } } \
                 let view = Box.methods; try { view.append(:x) } catch e { e }";
    assert_eq!(
        evaluate(bound),
        Ok(RuntimeValue::Symbol("ReadonlyMutationError".into()))
    );

    // An ordinary Array binding still appends.
    let ordinary = "mut a = [1]; a.append(2); a";
    assert_eq!(
        evaluate(ordinary),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Integer(2_u8.into()),
            ])),
        ])))
    );
}

#[test]
fn c006_refuses_a_lock_selecting_one_package_twice_at_one_major() {
    // C006 makes `(package_id, api_major)` ONE identity, so a lock selecting
    // two implementations of that pair has no single Type identity and fails
    // the LINK before any Module body runs.
    let sources = [("main".to_owned(), "module Main { }".to_owned())];
    let clashing = vec![
        (
            "org.token".to_owned(),
            1_u64,
            "1.2.0".to_owned(),
            "b3:a".to_owned(),
        ),
        (
            "org.token".to_owned(),
            1_u64,
            "1.2.1".to_owned(),
            "b3:b".to_owned(),
        ),
    ];
    assert_eq!(
        crate::load_resolved_package("org.x", 1, None, clashing, &sources, None).err(),
        Some(EvaluationError::PackageVersionUnification)
    );

    // Two MAJORS of one package are distinct identities and unify fine.
    let majors = vec![
        (
            "org.token".to_owned(),
            1_u64,
            "1.2.0".to_owned(),
            "b3:a".to_owned(),
        ),
        (
            "org.token".to_owned(),
            2_u64,
            "2.0.0".to_owned(),
            "b3:b".to_owned(),
        ),
    ];
    assert!(crate::load_resolved_package("org.x", 1, None, majors, &sources, None).is_ok());

    // Two DIFFERENT packages at one major are unrelated identities.
    let distinct = vec![
        (
            "org.token".to_owned(),
            1_u64,
            "1.2.0".to_owned(),
            "b3:a".to_owned(),
        ),
        (
            "org.other".to_owned(),
            1_u64,
            "1.2.1".to_owned(),
            "b3:b".to_owned(),
        ),
    ];
    assert!(crate::load_resolved_package("org.x", 1, None, distinct, &sources, None).is_ok());
}

#[test]
fn c045_activates_a_static_extension_only_for_a_direct_importer() {
    // C045 requires `export open ...` PLUS a DIRECT import for cross-Module
    // static visibility, and forbids transitive imports and re-exports from
    // activating such a member. V346 is the direct importer, V418 the facade.
    let ext = (
        "org.x.ext".to_owned(),
        vec![(
            "ext".to_owned(),
            "export class Base { } \
             export open class Base { public fun tag() -> Symbol { :tag } }"
                .to_owned(),
        )],
    );

    let direct = (
        "org.x".to_owned(),
        vec![(
            "main".to_owned(),
            "import org.x.ext.Base \
             module Main { public fun run() -> Symbol { Base.new().tag() } }"
                .to_owned(),
        )],
    );
    assert!(
        crate::load_package_tree(&[ext.clone(), direct], Some("Main.run()")).is_ok(),
        "a direct importer activates the member"
    );

    let facade_only = (
        "org.x".to_owned(),
        vec![(
            "main".to_owned(),
            "module Main { public fun run() -> Symbol { Base.new().tag() } }".to_owned(),
        )],
    );
    assert_eq!(
        crate::load_package_tree(&[ext, facade_only], Some("Main.run()")).err(),
        Some(EvaluationError::StaticMemberNotFound),
        "without a direct import the member is not activated"
    );
}

#[test]
fn c080_confines_a_getter_replacement_to_ordinary_reads() {
    // C065 and D-143 both authorize replacing a public ExceptionContext getter,
    // but the Class was not nameable, so the replacement had no entry point.
    let fixture = "mut n = 0; \
        class It { public fun next() { n = n + 1; \
          if n < 2 { Iteration.yield(1) } else { Iteration.done } } \
          public fun close() { raise :close } } \
        class S { public fun iterator() { It.new() } } ";

    // The replacement changes what an ordinary read returns.
    let replaced = format!(
        "{fixture} open class ExceptionContext {{ public fun suppressed() -> Array {{ [] }} }} \
         try {{ for x in S.new() {{ raise :body }} }} catch v, c {{ [c.suppressed, c.value] }}"
    );
    assert_eq!(
        evaluate(&replaced),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Array(ArrayRef::new(Vec::new())),
            RuntimeValue::Symbol("body".into()),
        ])))
    );

    // The same fixture WITHOUT the replacement still sees the protected
    // record, so the replacement changed the ordinary read and nothing else.
    let intact = format!(
        "{fixture} try {{ for x in S.new() {{ raise :body }} }} \
         catch v, c {{ [c.suppressed[0].value, c.value] }}"
    );
    assert_eq!(
        evaluate(&intact),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("close".into()),
            RuntimeValue::Symbol("body".into()),
        ])))
    );
}

#[test]
fn c099_refuses_removing_a_declared_contract() {
    // C099 supplies the SPELLING the refusal needs to be observable; C045 makes
    // declared conformance immutable, so the attempt is refused before commit
    // and the target keeps its conformance. C017 numbers the origin 1, and the
    // refused removal publishes nothing, so the revision stays 1.
    let declared = "contract C { fun m() -> Nil } \
                    class A for C { public impl fun m() -> Nil { nil } } \
                    let refused = try { A.remove_contract(C) } catch e { e }; \
                    [refused, A.active_revision]";
    assert_eq!(
        evaluate(declared),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("TypeContractError".into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );

    // Removing a Contract the Class never declared changes no static spine
    // fact, so it is a no-op rather than a refusal.
    let undeclared = "contract C { fun m() -> Nil } class A { } A.remove_contract(C)";
    assert_eq!(evaluate(undeclared), Ok(RuntimeValue::Nil));
}

#[test]
fn c126_scopes_the_artifact_digest_to_source_bytes() {
    // C126 scopes a lightweight audit record's digest to the referenced
    // artifact's SOURCE bytes and not its locator, which is what lets V357's
    // locator-only variant preserve the digest while a source change alters it.
    let source = "class Owner {\n  public fun status() -> Symbol { :ok } \n}\n";
    let digest = iris_runtime::artifact_digest(source.as_bytes());

    // The digest is a property of the bytes alone, so hashing them twice agrees
    // and any change to them changes the result.
    assert_eq!(digest, iris_runtime::artifact_digest(source.as_bytes()));
    let changed = source.replace(":ok", ":changed");
    let other = iris_runtime::artifact_digest(changed.as_bytes());
    assert_ne!(digest, other);

    // V357 requires a source change to alter the FULL 32-byte digest, so no
    // byte may survive.
    let retained = digest
        .as_bytes()
        .chunks(2)
        .zip(other.as_bytes().chunks(2))
        .filter(|(left, right)| left == right)
        .count();
    assert_eq!(retained, 0, "a source change must alter all 32 bytes");
}

#[test]
fn c070_leaves_the_old_package_active_when_an_upgrade_hook_fails() {
    // C068 makes a same-major upgrade transactional and C069 lets an
    // `upgrade(from_version, context)` hook validate candidate state. C070
    // leaves the old package AND STATE fully active when the hook fails, while
    // C042 makes external side effects the author's responsibility.
    let source = concat!(
        "class Ledger { shared class property log: Array = [] ",
        "shared class property counter: Integer = 4 } ",
        "module Upgrade { public fun upgrade(older, newer) -> Symbol { ",
        r#"Ledger.log.append(:"migration-start"); "#,
        "Ledger.counter = 9; raise :MigrationStop } }"
    );
    let probe = concat!(
        r#"let refused = try { Reflection::Package.upgrade(:"1.0.1") } catch e { e }; "#,
        "[refused, Reflection::Package.version(), Ledger.counter, Ledger.log]"
    );
    let outcome = crate::load_resolved_package_with_artifact(
        crate::PackageResolution {
            package_id: "org.x",
            api_major: 1,
            version: Some("1.0.0".into()),
            locked: Vec::new(),
            artifact: None,
            permissions: &[],
            grants: Vec::new(),
        },
        &[("main".to_owned(), source.to_owned())],
        Some(probe),
    );
    assert_eq!(
        outcome.map(|(_, observed)| observed),
        Ok(Some(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("MigrationStop".into()),
            // The version never advances, and the counter the hook wrote is
            // restored, since both are candidate state.
            RuntimeValue::Symbol("1.0.0".into()),
            RuntimeValue::Integer(4_u8.into()),
            // The external log the hook already wrote SURVIVES, which is the
            // half C070 hands to the package author rather than undoing.
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Symbol(
                "migration-start".into()
            )])),
        ]))))
    );
}

#[test]
fn c037_refuses_suspension_inside_a_transaction_body() {
    // C037 makes an open or revision transaction body non-suspending, with
    // STATIC violations as compile errors and DYNAMIC ones raising
    // MetaTransactionError. ASYNC-C018 owns the async reason. Both halves are
    // observed: V355 the static one, V431 the dynamic one.
    let dynamic = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class Slow for ClassDecorator { ",
        "public impl fun transform(declaration, arguments, context) -> Transformation { ",
        "await Transformation.empty } } ",
        "@Slow() class Box { } Box"
    );
    assert_eq!(
        evaluate(dynamic),
        Err(EvaluationError::MetaTransactionSuspension)
    );

    // A transform that does not suspend publishes normally, so the refusal
    // comes from the suspension rather than from running a transform at all.
    let ordinary = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class Ok for ClassDecorator { ",
        "public impl fun transform(declaration, arguments, context) -> Transformation { ",
        "Transformation.empty } } ",
        "@Ok() class Box { } Box.new()"
    );
    assert!(evaluate(ordinary).is_ok());
}

#[test]
fn c072_makes_a_yielding_callable_a_generator() {
    // C072 makes a callable containing `yield` a GENERATOR: invoking it runs no
    // body and returns an Iterator whose `next()` drives it, answering
    // Iteration.yield at each suspension and Iteration.done once complete.
    let stepped = "class G { public fun each() -> Nil { yield 1; yield 2 } } \
                   let g = G.new().each(); [g.next(), g.next(), g.next()]";
    assert_eq!(
        evaluate(stepped),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::IterationYield(Box::new(RuntimeValue::Integer(1_u8.into()))),
            RuntimeValue::IterationYield(Box::new(RuntimeValue::Integer(2_u8.into()))),
            // C013 returns the same done singleton on every later call.
            RuntimeValue::IterationDone,
        ])))
    );

    // C011 and C012 drive `for` through iterator()/next(), so a generator is
    // consumed by the SAME protocol every other Iterator uses.
    let driven = "mut seen = [] \
                  class G { public fun iterator() -> Nil { yield 1; yield 2; yield 3 } } \
                  for v in G.new() { seen.append(v) } \
                  seen";
    assert_eq!(
        evaluate(driven),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Integer(2_u8.into()),
                RuntimeValue::Integer(3_u8.into()),
            ])),
        ])))
    );
}

#[test]
fn c003_and_c012_make_an_async_call_return_a_started_task() {
    // C003 makes an async Method return Task<T> rather than T, and C012 makes
    // creating the Task and starting its initial run ONE call operation, so
    // the body has already run when the caller receives the Task.
    let started = "mut ran = false \
                   class A { public async fun f() -> Integer { ran = true; 1 } } \
                   let t = A.new().f(); ran";
    assert_eq!(evaluate(started), Ok(RuntimeValue::Bool(true)));

    // C013 continues synchronously on an already-complete Awaitable, and a
    // Task may be awaited more than once with the same result.
    let awaited = "class A { public async fun f() -> Integer { 7 } } \
                   let t = A.new().f(); [await t, await t]";
    assert_eq!(
        evaluate(awaited),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(7_u8.into()),
            RuntimeValue::Integer(7_u8.into()),
        ])))
    );

    // C016 propagates a captured failure to the awaiter rather than at
    // invocation, since invocation only starts the body.
    let failed = "class A { public async fun f() -> Integer { raise :boom } } \
                  let t = A.new().f(); try { await t } catch e { e }";
    assert_eq!(evaluate(failed), Ok(RuntimeValue::Symbol("boom".into())));
}

#[test]
fn c050_drives_a_task_only_from_a_host_position() {
    // C050 fixes the Host drive surface C015 names and C049 requires, so a
    // Task's awaited result is reachable at last.
    let driven = "class A { public async fun f() -> Integer { 1 } } Host.run(A.new().f())";
    assert_eq!(evaluate(driven), Ok(RuntimeValue::Integer(1_u8.into())));

    // C015 forbids an Iris SOURCE-level blocking wait, so the surface is
    // refused everywhere source code could build one out of it. Without these
    // refusals it would BE the hidden Task join C015 forbids.
    for source in [
        "class A { public async fun f() -> Integer { 1 } \
         public async fun g() -> Integer { Host.run(A.new().f()) } } Host.run(A.new().g())",
        "class A { public async fun f() -> Integer { 1 } } \
         let c = { Host.run(A.new().f()) }; c.call()",
        "class A { public async fun f() -> Integer { 1 } } class B { } \
         B.open() { |t| Host.run(A.new().f()) }",
    ] {
        assert_eq!(
            evaluate(source),
            Err(EvaluationError::HostDriveUnavailable),
            "the drive surface must not be reachable from Iris source: {source}"
        );
    }

    // `Host` is an ordinary identifier a program may declare, and a DECLARED
    // name wins over the surface. Routing it unconditionally hijacked a user
    // Class of that name, which TYPES-V246 and META-V340 caught.
    let declared = "module Helpers<T> { public fun h() -> Integer { 7 } } \
                    class Host mixin Helpers<String> {} Host.new().h()";
    assert_eq!(evaluate(declared), Ok(RuntimeValue::Integer(7_u8.into())));
}

#[test]
fn c032_and_c033_close_a_using_resource_exactly_once() {
    // C032 makes `using(resource, &block)` an ordinary helper that invokes the
    // block then closes the resource through try/finally equivalent control.
    let normal = "mut closed = false; \
                  class R { public fun close() -> Nil { closed = true; nil } } \
                  let v = using(R.new()) { 7 }; [v, closed]";
    assert_eq!(
        evaluate(normal),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(7_u8.into()),
            RuntimeValue::Bool(true),
        ])))
    );

    // C033: a raising block stays PRIMARY and the close failure is appended to
    // its suppressed list.
    let both_raise = "class R { public fun close() -> Nil { raise :closefail } } \
                      try { using(R.new()) { raise :blockfail } } \
                      catch v, c { [v, c.suppressed[0].value] }";
    assert_eq!(
        evaluate(both_raise),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("blockfail".into()),
            RuntimeValue::Symbol("closefail".into()),
        ])))
    );

    // C033: a close failure after a NORMAL block becomes primary, and no block
    // result is returned.
    let close_raises = "class R { public fun close() -> Nil { raise :closefail } } \
                        try { using(R.new()) { 7 } } catch e { e }";
    assert_eq!(
        evaluate(close_raises),
        Ok(RuntimeValue::Symbol("closefail".into()))
    );

    // C014 keeps `using` an ordinary Method name, so a DECLARED one wins over
    // the standard helper.
    let declared = "module M { public fun using(r) -> Symbol { :mine } } \
                    class R { public fun close() -> Nil { nil } } M.using(R.new())";
    assert_eq!(evaluate(declared), Ok(RuntimeValue::Symbol("mine".into())));
}

#[test]
fn c009_resolves_negative_indexes_and_unary_operators() {
    // C016 counts unary and binary `+` and `-` as distinct forms, and the
    // runtime already installed `negate` and `~` as native selectors, but the
    // source evaluator dispatched to neither. `-1` was unevaluatable, which
    // also made a negative index unwritable.
    assert_eq!(
        evaluate("class Z { } -1"),
        Ok(RuntimeValue::Integer((-1_i8).into()))
    );
    assert_eq!(
        evaluate("class Z { } +3"),
        Ok(RuntimeValue::Integer(3_u8.into()))
    );
    assert_eq!(
        evaluate("class Z { } ~0"),
        Ok(RuntimeValue::Integer((-1_i8).into()))
    );

    // C009 resolves a negative index as `length + index`, and a read outside
    // the resolved range answers nil rather than raising.
    assert_eq!(
        evaluate("class Z { } let a = [1,2,3]; a[-1]"),
        Ok(RuntimeValue::Integer(3_u8.into()))
    );
    assert_eq!(
        evaluate("class Z { } let a = [1,2,3]; a[-3]"),
        Ok(RuntimeValue::Integer(1_u8.into()))
    );
    assert_eq!(
        evaluate("class Z { } let a = [1,2,3]; a[-4]"),
        Ok(RuntimeValue::Nil)
    );
}

#[test]
fn c024_writes_use_negative_resolution_and_raise_index_error() {
    // C024 gives an Array write the same C009 negative resolution a read uses,
    // but an out-of-range WRITE raises IndexError where a read answers nil.
    // The write path used raw to_usize and a Type error for both.
    // The write answers nil per C024 and the following read observes it, so
    // the program yields both values.
    assert_eq!(
        evaluate("class Z { } mut a = [1,2,3]; a[-1] = 9; a[2]"),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(9_u8.into()),
            RuntimeValue::Integer(9_u8.into()),
        ])))
    );
    assert_eq!(
        evaluate("class Z { } mut a = [1,2,3]; a[9] = 1"),
        Err(EvaluationError::IndexError)
    );
    assert_eq!(
        evaluate("class Z { } mut a = [1,2,3]; a[-9] = 1"),
        Err(EvaluationError::IndexError)
    );

    // The read half is unchanged: out of range still answers nil.
    assert_eq!(
        evaluate("class Z { } let a = [1,2,3]; a[9]"),
        Ok(RuntimeValue::Nil)
    );
}

#[test]
fn call_evaluates_the_receiver_expression_before_its_arguments() {
    // Given: IRIS-V1-CONTROL-C033 fixes call evaluation order as receiver
    // expression, then positional arguments left-to-right, then invocation.
    let source = "mut log = []; class A { public fun s(x, y) -> Integer { log.append(:call); 1 } } class Mk { public fun make() -> Object { log.append(:receiver); A.new() } } module M { public fun run() -> Object { Mk.new().make().s(log.append(:one), log.append(:two)); log } } M.run()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("receiver".into()),
            RuntimeValue::Symbol("one".into()),
            RuntimeValue::Symbol("two".into()),
            RuntimeValue::Symbol("call".into()),
        ])))
    );
}

#[test]
fn iteration_compares_as_unordered_against_a_non_iteration() {
    // Given: IRIS-V1-COLLECTIONS-C015 returns nil between an Iteration and a
    // non-Iteration, alongside 0 for done <=> done and a forwarded payload
    // comparison for two yields.
    let unordered = "module M { public fun run() -> Object { Iteration.yield(1) <=> 1 } } M.run()";
    let done_vs_value = "module M { public fun run() -> Object { Iteration.done <=> 1 } } M.run()";

    // When / Then
    assert_eq!(evaluate(unordered), Ok(RuntimeValue::Nil));
    assert_eq!(evaluate(done_vs_value), Ok(RuntimeValue::Nil));
}

#[test]
fn composition_rejects_a_mixin_naming_nothing() {
    // Given: IRIS-V1-META-C054 validates composition BEFORE publication, so a
    // mixin naming neither a Module nor a Class cannot reach a published Class.
    let unknown = "class A mixin Nope { } A.new()";

    // When / Then: the declaration is refused rather than silently composing.
    assert_eq!(
        evaluate(unknown),
        Err(EvaluationError::UnsupportedConstruct)
    );

    // And a mixin that DOES name a Module still composes.
    assert_eq!(
        evaluate("module Mo { public fun h() -> Integer { 8 } } class A mixin Mo { } A.new().h()"),
        Ok(RuntimeValue::Integer(8_u64.into()))
    );
}

#[test]
fn a_subclass_inherits_its_superclass_declared_contract() {
    // Given: IRIS-V1-TYPES-C019 draws nominal subtyping from immutable
    // superclass and DECLARED CONTRACT facts, so a subclass of a Class
    // declaring `for C` conforms to C as well and may be viewed as one.
    let source = "contract C { fun n() -> Symbol } class B for C { public impl fun n() -> Symbol { :base } } class A extends B { } module M { public fun run() -> Object { (A.new() as C)..n() } } M.run()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("base".into())));
}

#[test]
fn qualified_super_reaches_an_unqualified_ancestor_impl() {
    // Given: IRIS-V1-RUNTIME-C082 keeps qualified `super` inside the Contract
    // slot, and IRIS-V1-TYPES-C047 makes one unqualified `impl` satisfy a
    // declared requirement, so super must find an ancestor's plain `impl`.
    let source = "contract C { fun n() -> Symbol } class B for C { public impl fun n() -> Symbol { :base } } class A extends B { public override impl fun n() -> Symbol { super() } } module M { public fun run() -> Object { (A.new() as C)..n() } } M.run()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("base".into())));
}

#[test]
fn a_checked_cast_returns_the_value_or_raises() {
    // Given: IRIS-V1-TYPES-C029 evaluates the operand once, returns the SAME
    // value when it satisfies the reified Type, and raises TypeError when not.
    let ok = "class A { } class B extends A { } module M { public fun run() -> Object { let b = B.new(); (b as A) is A } } M.run()";
    let bad = "class A { } class Z { } module M { public fun run() -> Object { let z = Z.new(); (z as A) } } M.run()";

    // When / Then
    assert_eq!(evaluate(ok), Ok(RuntimeValue::Bool(true)));
    assert_eq!(
        evaluate(bad),
        Err(EvaluationError::Runtime(iris_runtime::KernelError::Type))
    );
}

#[test]
fn an_origin_class_declaration_publishes_revision_one() {
    // Given: IRIS-V1-RUNTIME-C017 makes the origin Class revision number 1 and
    // gives the next per-Class integer to each successful structural
    // PUBLICATION. Declaring a Class is ONE publication.
    let bare = "class A { } A.active_revision";
    let with_member = "class A { public fun g() -> Integer { 1 } } A.active_revision";

    // When / Then: neither the implicit `to_bool` nor a declared member is a
    // separate publication, so both declarations sit at the origin number.
    assert_eq!(evaluate(bare), Ok(RuntimeValue::Integer(1_u8.into())));
    assert_eq!(
        evaluate(with_member),
        Ok(RuntimeValue::Integer(1_u8.into()))
    );

    // And: a structural change AFTER the declaration is the next publication.
    let reopened = "class A { } \
                    let ignored = A.define_method(:h) { 1 }; \
                    A.active_revision";
    assert_eq!(evaluate(reopened), Ok(RuntimeValue::Integer(2_u8.into())));
}

#[test]
fn v959_revision_metadata_is_read_only_and_monotonic() {
    // C017: a failed candidate publishes nothing, so it consumes no number.
    let failed = "contract C { fun m() -> Nil } \
                  class A { } \
                  let refused = try { A.add_contract(C) } catch e { e }; \
                  A.active_revision";
    assert_eq!(evaluate(failed), Ok(RuntimeValue::Integer(1_u8.into())));

    // C018: one successful publication takes one number and one commit_id.
    let committed = "class A { } \
                     let first = Reflection::Class.revision(A).fetch(:commit_id); \
                     let done = A.define_method(:h) { 1 }; \
                     let second = Reflection::Class.revision(A).fetch(:commit_id); \
                     [A.active_revision, first == second]";
    assert_eq!(
        evaluate(committed),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Bool(false),
        ])))
    );
}

#[test]
fn an_array_pattern_destructures_and_binds() {
    // C051 admits `[a, b]` array destructuring in the pattern vocabulary, so
    // an arity-matching subject binds each element positionally.
    assert_eq!(
        evaluate("match [1, 2] { [a, b] => a + b, _ => 0 }"),
        Ok(RuntimeValue::Integer(3_u8.into()))
    );

    // A shape that does not match falls through to the next arm rather than
    // binding, because destructuring is part of the match test.
    assert_eq!(
        evaluate("match [1, 2, 3] { [a, b] => a + b, _ => 99 }"),
        Ok(RuntimeValue::Integer(99_u8.into()))
    );
    assert_eq!(
        evaluate("match 1 { [a] => a, _ => 99 }"),
        Ok(RuntimeValue::Integer(99_u8.into()))
    );

    // Nested patterns and literal elements compose.
    assert_eq!(
        evaluate("match [1, [2, 3]] { [a, [b, c]] => a + b + c, _ => 0 }"),
        Ok(RuntimeValue::Integer(6_u8.into()))
    );
    assert_eq!(
        evaluate("match [1, 2] { [1, b] => b, _ => 0 }"),
        Ok(RuntimeValue::Integer(2_u8.into()))
    );

    // The empty array pattern matches only an empty array.
    assert_eq!(
        evaluate("match [] { [] => 7, _ => 0 }"),
        Ok(RuntimeValue::Integer(7_u8.into()))
    );
}

#[test]
fn c053_binding_only_destructuring_mismatch_raises() {
    // C053: a `for` binding is a BINDING-ONLY destructuring context, so an
    // arity mismatch there raises PatternMatchError rather than skipping.
    assert_eq!(
        evaluate("mut t = 0; for [k, v] in [[1, 2], [3, 4]] { t = t + k + v }; t"),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Integer(10_u8.into()),
        ])))
    );
    assert_eq!(
        evaluate("mut t = 0; for [k, v] in [[1]] { t = t + k }; t"),
        Err(EvaluationError::PatternMatchError)
    );

    // By contrast an ordinary match arm TESTS the shape, so a mismatch simply
    // does not select that arm.
    assert_eq!(
        evaluate("match [1] { [a, b] => a + b, _ => 42 }"),
        Ok(RuntimeValue::Integer(42_u8.into()))
    );
}

#[test]
fn c087_applies_a_decorator_transform_to_the_candidate() {
    // C122/C125: the runtime phase RETURNS a Transformation, and
    // `add_method` stages one Method onto the target candidate, so the
    // generated Method is present on the published Class.
    let staged = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class Stamp for ClassDecorator { ",
        "public impl fun transform(declaration, arguments, context) -> Transformation { ",
        "Transformation.empty.add_method(:stamped) { 7 } } } ",
        "@Stamp() class Box { } Box.new().stamped()"
    );
    assert_eq!(evaluate(staged), Ok(RuntimeValue::Integer(7_u8.into())));

    // C017 still counts the decorated declaration as ONE publication, so the
    // staged Method joins the origin revision rather than taking a number.
    let revision = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class Stamp for ClassDecorator { ",
        "public impl fun transform(declaration, arguments, context) -> Transformation { ",
        "Transformation.empty.add_method(:stamped) { 7 } } } ",
        "@Stamp() class Box { } Box.active_revision"
    );
    assert_eq!(evaluate(revision), Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn c091_aborts_a_forbidden_static_spine_change_from_a_decorator() {
    // C091 forbids a decorator from changing an immutable superclass bound,
    // and C086/C091 require the target to retain no candidate: the change must
    // ABORT the declaration rather than be silently dropped.
    let source = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class P { } ",
        "class S for ClassDecorator { ",
        "public impl fun transform(d, a, c) -> Transformation { ",
        "Reflection::Class.set_superclass(d, P); Transformation.empty } } ",
        "@S() class Box { } Box.new() is P"
    );
    assert_eq!(
        evaluate(source),
        Err(EvaluationError::Class(
            iris_runtime::ClassError::DecoratorViolation {
                class: iris_runtime::ClassId::new(9),
                violation: iris_runtime::DecoratorViolation::NominalIdentity,
            }
        ))
    );

    // An ORDINARY open transaction may still change the superclass, so the
    // refusal is scoped to the decorator phase rather than to the operation.
    assert_eq!(
        evaluate(
            "class P { } class Box { } Reflection::Class.set_superclass(Box, P); Box.new() is P"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Bool(true),
        ])))
    );

    // A decorator that changes nothing still publishes normally, so the
    // refusal comes from the forbidden change rather than from decorating.
    let ordinary = concat!(
        "contract ClassDecorator { fun transform(declaration, arguments, context) } ",
        "class S for ClassDecorator { ",
        "public impl fun transform(d, a, c) -> Transformation { Transformation.empty } } ",
        "@S() class Box { } Box.active_revision"
    );
    assert_eq!(evaluate(ordinary), Ok(RuntimeValue::Integer(1_u8.into())));
}

#[test]
fn a_declared_module_wins_over_a_builtin_service_name() {
    // The builtin service routes are guarded by "a DECLARED name wins", but the
    // guard asked only whether a CLASS of that name exists. A declared Module
    // made `class_name` answer UnsupportedConstruct, which the guard read as
    // "not declared", so the builtin route hijacked the user's Module.
    assert_eq!(
        evaluate("module JSON { public fun tag() -> Symbol { :mine } } JSON.tag()"),
        Ok(RuntimeValue::Symbol("mine".into()))
    );
    assert_eq!(
        evaluate("module Package { public fun tag() -> Symbol { :mine } } Package.tag()"),
        Ok(RuntimeValue::Symbol("mine".into()))
    );
    assert_eq!(
        evaluate("module Diagnostics { public fun tag() -> Symbol { :mine } } Diagnostics.tag()"),
        Ok(RuntimeValue::Symbol("mine".into()))
    );

    // With no such declaration the builtin service still answers, so the fix is
    // scoped to a user declaration rather than disabling the route.
    assert_eq!(
        evaluate("JSON.encode(1, canonical: true)"),
        Ok(RuntimeValue::Text("1".into()))
    );
}

#[test]
fn c036_reports_safe_decoder_diagnostics() {
    // C036 requires a safe decoding diagnostic to identify the decoder, the
    // format version when known, the byte offset when available, and the
    // violated limit or expected Contract. The raised value carried the error
    // symbol alone, so a caller could not tell WHICH decode failed or where.
    assert_eq!(
        evaluate(
            "try { JSON.decode(\"[1,2\") } catch v, c { [v, c.decoder, c.offset, c.expected] }"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("JSONSyntaxError".into()),
            RuntimeValue::Symbol("JSON".into()),
            RuntimeValue::Integer(4_u8.into()),
            RuntimeValue::Symbol("value".into()),
        ])))
    );

    // C066 forbids a FABRICATED diagnostic, so a context raised by something
    // other than a decode carries none, and a later unrelated raise does not
    // inherit the last decode's record.
    assert_eq!(
        evaluate("try { raise :plain } catch v, c { [c.decoder, c.offset, c.expected] }"),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Nil,
            RuntimeValue::Nil,
        ])))
    );
    assert_eq!(
        evaluate(
            "let first = try { JSON.decode(\"[1,2\") } catch v, c { c.decoder }; \
             try { raise :later } catch v, c { c.decoder }"
        ),
        Ok(RuntimeValue::Nil)
    );

    // C036 also forbids leaking anything beyond the fragment needed to report
    // the failure, so the diagnostic carries no Host path or address.
    assert_eq!(
        evaluate("try { JSON.decode(\"{\") } catch v, c { [c.decoder, c.offset] }"),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("JSON".into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn c020_decodes_a_nominal_value_through_a_declared_factory() {
    // C020: nominal deserialization MUST call only DECLARED standard
    // deserialization factories, and C004 makes participation opt-in through
    // `for Serializable`. A conforming Class's factory rebuilds the value.
    let stream = "mut s = %{}; s[\"magic\"] = \"IRISVALUE\"; s[\"format_version\"] = 1; \
                  s[\"nominal\"] = :User; s[\"schema_version\"] = 1; s[\"payload\"] = 7; ";
    let declared = format!(
        "contract Serializable {{ fun serialize() -> Object }} \
         class User for Serializable {{ \
           public fun serialize() -> Object {{ 1 }} \
           public class fun deserialize(representation) -> Object {{ :rebuilt }} }} \
         module M {{ public fun run() -> Object {{ {stream} IrisValue.decode(s) }} }} M.run()"
    );
    assert_eq!(
        evaluate(&declared),
        Ok(RuntimeValue::Symbol("rebuilt".into()))
    );

    // C020 rejects an UNDECLARED factory without invoking it, and C004 forbids
    // duck typing from implying eligibility, so a Class with a matching method
    // but no declared conformance is refused rather than called.
    let ghost = stream.replace(":User", ":Ghost");
    let undeclared = format!(
        "contract Serializable {{ fun serialize() -> Object }} \
         class Ghost {{ \
           public class fun deserialize(representation) -> Object {{ :leaked }} }} \
         module M {{ public fun run() -> Object {{ {ghost} IrisValue.decode(s) }} }} M.run()"
    );
    assert_eq!(
        evaluate(&undeclared),
        Err(EvaluationError::SerializationError)
    );
}

#[test]
fn c006_validates_a_nominal_stream_before_publishing() {
    let base = "contract Serializable { fun serialize() -> Object } \
                class User for Serializable { \
                  public fun serialize() -> Object { 1 } \
                  public class fun deserialize(representation) -> Object { :rebuilt } } ";
    let stream = |schema: &str| {
        format!(
            "mut s = %{{}}; s[\"magic\"] = \"IRISVALUE\"; s[\"format_version\"] = 1; \
             s[\"nominal\"] = :User; {schema} s[\"payload\"] = 7; "
        )
    };
    let run = |body: String| {
        format!(
            "{base} module M {{ public fun run() -> Object {{ {body} IrisValue.decode(s) }} }} M.run()"
        )
    };

    // C006 validates the DECLARED schema version before publishing, so a
    // stream declaring an unknown one is refused rather than rebuilt.
    assert_eq!(
        evaluate(&run(stream("s[\"schema_version\"] = 2;"))),
        Err(EvaluationError::LexicalDiagnostic(
            "IRISVALUE_INCOMPATIBLE_HEADER"
        ))
    );

    // The matching schema version rebuilds, so the refusal comes from the
    // mismatch rather than from validating at all.
    assert_eq!(
        evaluate(&run(stream("s[\"schema_version\"] = 1;"))),
        Ok(RuntimeValue::Symbol("rebuilt".into()))
    );

    // C006 also refuses a nominal name that resolves to no Class, publishing
    // nothing rather than reporting a missing message from a later send.
    let unknown = run(stream("s[\"schema_version\"] = 1;").replace(":User", ":Absent"));
    assert_eq!(evaluate(&unknown), Err(EvaluationError::SerializationError));
}

#[test]
fn c094_reflects_decorator_arguments_and_phase_participation() {
    // C094 exposes ordered decorator IDENTITY, ARGUMENTS, and static/runtime
    // phase participation. Only the identities were reported, so a caller
    // could not tell `@Stamp(1)` from `@Stamp(2)`.
    let base = "contract ClassDecorator { fun transform(declaration, arguments, context) } \
                class First for ClassDecorator { \
                  public impl fun transform(d, a, c) -> Transformation { Transformation.empty } } \
                class Second for ClassDecorator { \
                  public impl fun transform(d, a, c) -> Transformation { Transformation.empty } } \
                @First(1, :two) @Second() class Box { } ";

    // Ordered identity is preserved, which already worked.
    assert_eq!(
        evaluate(&format!("{base} Box.decorators")),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("First".into()),
            RuntimeValue::Symbol("Second".into()),
        ])))
    );

    // Each applied decorator reports its own arguments IN ORDER, and one
    // applied with none reports an empty list rather than nil.
    assert_eq!(
        evaluate(&format!("{base} Box.decorator_arguments")),
        Ok(RuntimeValue::ReadonlyArray(vec![
            RuntimeValue::ReadonlyArray(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Symbol("two".into()),
            ]),
            RuntimeValue::ReadonlyArray(Vec::new()),
        ]))
    );

    // C122 makes a decorator declare both phase members, so participation is
    // read from which the Class actually declares: both declare `transform`
    // and neither declares `plan`, so each participates at runtime only.
    assert_eq!(
        evaluate(&format!("{base} Box.decorator_phases")),
        Ok(RuntimeValue::ReadonlyArray(vec![
            RuntimeValue::Symbol("runtime".into()),
            RuntimeValue::Symbol("runtime".into()),
        ]))
    );

    // C094 exposes these as IMMUTABLE views and forbids exposing mutable
    // transform internals, so a mutating selector is refused rather than
    // silently editing a copy the caller believes is the real metadata.
    assert_eq!(
        evaluate(&format!(
            "{base} let v = Box.decorator_arguments; try {{ v.append([9]) }} catch e {{ e }}"
        )),
        Ok(RuntimeValue::Symbol("ReadonlyMutationError".into()))
    );
    assert_eq!(
        evaluate(&format!(
            "{base} let v = Box.decorator_phases; try {{ v.append(:x) }} catch e {{ e }}"
        )),
        Ok(RuntimeValue::Symbol("ReadonlyMutationError".into()))
    );
}

#[test]
fn a_bare_raise_in_a_catch_appends_a_re_raise_step() {
    // A bare `raise` inside a catch continues the SAME context rather than
    // starting a new one, and the re-raise is recorded as an appended step.
    assert_eq!(
        evaluate("try { raise :a } catch e, c { c.re_raise_sites.length }"),
        Ok(RuntimeValue::Integer(0_u8.into()))
    );
    assert_eq!(
        evaluate(
            "try { try { raise :a } catch e { raise } } catch f, d { \
             [f, d.re_raise_sites.length] }"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("a".into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn c108_reports_denial_context_only_for_a_denial() {
    // C066 forbids fabricating a diagnostic, so a context raised by anything
    // other than a reflection denial carries no denial context, and a later
    // unrelated raise does not inherit one.
    assert_eq!(
        evaluate(
            "try { raise :plain } catch v, c { \
             [c.operation, c.caller_package, c.target_scope, c.denial_origin] }"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Nil,
            RuntimeValue::Nil,
            RuntimeValue::Nil,
        ])))
    );
}

#[test]
fn c160_migrates_a_revision_only_when_called_explicitly() {
    // C160 and D-264: a commit performs NO implicit enumeration or migration,
    // so a tracked instance's own `migrate_revision` runs only when
    // application code calls it, and it answers nil.
    let base = "mut calls = 0; \
                class A { public fun migrate_revision(old, new) -> Object { \
                  calls = calls + 1; nil } } \
                let a = A.new(); \
                let opened = A.open() { |t| t.define_method(:m) { 2 } }; ";
    assert_eq!(
        evaluate(&format!("{base} [calls, a.migrate_revision(1, 2), calls]")),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(0_u8.into()),
            RuntimeValue::Nil,
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );

    // C016 makes a ClassRevision read-only: user code MUST NOT reactivate one,
    // and C020 requires a rollback to publish a NEW validated revision instead
    // of reactivating a historical one in place.
    assert_eq!(
        evaluate(&format!(
            "{base} try {{ Reflection::Class.reactivate(A, 1) }} catch e {{ e }}"
        )),
        Ok(RuntimeValue::Symbol("MetaTransactionError".into()))
    );
}

#[test]
fn c046_closes_a_programmable_iterator_on_every_exit_path() {
    // C044 obtains the Iterator through the canonical Iterable protocol, so a
    // traversal source answers `iterator()`; C046 then requires close() on
    // EVERY exit path, exactly once because close is idempotent.
    let base = "mut closed = 0; mut n = 0; \
                class It { \
                  public fun next() -> Object { \
                    n = n + 1; if n < 3 { Iteration.yield(n) } else { Iteration.done } } \
                  public fun close() -> Object { closed = closed + 1; nil } } \
                class S { public fun iterator() -> Object { It.new() } } \
                module M { public fun run() -> Object { ";

    // Natural exhaustion releases through Iteration.done.
    assert_eq!(
        evaluate(&format!(
            "{base} mut t = 0; for x in S.new() {{ t = t + x }}; [t, closed] }} }} M.run()"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(3_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );

    // `break` closes before the pending transfer commits.
    assert_eq!(
        evaluate(&format!(
            "{base} for x in S.new() {{ break }}; closed }} }} M.run()"
        )),
        Ok(RuntimeValue::Integer(1_u8.into()))
    );

    // `return` out of the body closes before the transfer COMMITS, so the
    // returned expression is evaluated first and still reads 0; the close is
    // observed after the traversal has been left.
    assert_eq!(
        evaluate(&format!(
            "{base} for x in S.new() {{ return closed }}; 99 }} }} \
             let returned = M.run(); [returned, closed]"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(0_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );

    // A body exception closes while unwinding, and the raise still propagates.
    assert_eq!(
        evaluate(&format!(
            "{base} let raised = try {{ for x in S.new() {{ raise :boom }} }} catch e {{ e }}; \
             [raised, closed] }} }} M.run()"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("boom".into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );

    // `continue` does NOT close: it stays inside the same traversal, so the
    // loop runs to exhaustion and closes once at the end.
    assert_eq!(
        evaluate(&format!(
            "{base} for x in S.new() {{ continue }}; closed }} }} M.run()"
        )),
        Ok(RuntimeValue::Integer(1_u8.into()))
    );
}

#[test]
fn c030_closes_a_native_backed_resource_idempotently() {
    // C027 validates the payload before the runtime owns its storage, so the
    // resource reaches script only once registration succeeded, and C030 makes
    // release explicit and IDEMPOTENT: both calls answer nil of type Nil and
    // the native release counter is exactly 1.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               let r = NativeFixture.resource(); \
               let first = r.close(); let second = r.close(); \
               [first, second, r.releases, r.class_name] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Nil,
            RuntimeValue::Nil,
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Symbol("FFI::Resource".into()),
        ])))
    );
}

#[test]
fn c039_rejects_an_unbound_method_as_a_closure() {
    // C094 reifies callable KIND: `Closure<S>` types a Closure and
    // `BoundMethod<S>` types a BoundMethod. C039 makes an unbound Method a
    // reflective definition object rather than an ordinary callable, so
    // assigning one to a callable annotation MUST fail unless an explicit
    // binding produces a BoundMethod.
    let base = "class A { public fun f(x: Integer) -> Integer { x } } \
                module M { public fun run() -> Object { \
                  let m = Reflection::Class.method(A, :f); ";

    assert_eq!(
        evaluate(&format!(
            "{base} let bad: Closure<(Integer) -> Integer> = m; bad }} }} M.run()"
        )),
        Err(EvaluationError::TypeContractError)
    );

    // An explicit binding produces a BoundMethod, which the BoundMethod kind
    // admits and the Closure kind still does not.
    assert_eq!(
        evaluate(&format!(
            "{base} let b: BoundMethod<(Integer) -> Integer> = m.bind(A.new()); \
             b.class_name }} }} M.run()"
        )),
        Ok(RuntimeValue::Symbol("BoundMethod".into()))
    );
    assert_eq!(
        evaluate(&format!(
            "{base} let bad: Closure<(Integer) -> Integer> = m.bind(A.new()); bad }} }} M.run()"
        )),
        Err(EvaluationError::TypeContractError)
    );

    // A Closure still satisfies its own kind, so the refusal is about KIND
    // rather than about callable annotations being unusable.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               let c: Closure<(Integer) -> Integer> = { |x: Integer| -> Integer x }; \
               c.class_name } } M.run()"
        ),
        Ok(RuntimeValue::Symbol("Closure".into()))
    );
}

#[test]
fn c096_names_callable_kinds_and_checks_calls_at_the_site() {
    // C094 names the callable kinds `Closure<S>` and `BoundMethod<S>`, and
    // C096 makes the arguments INVARIANT, so a value of one kind never
    // satisfies the other.
    let base = "class A { public fun f(x: Integer) -> Integer { x } } \
                module M { public fun run() -> Object { \
                  let m = Reflection::Class.method(A, :f); ";

    assert_eq!(
        evaluate(&format!(
            "{base} let bad: BoundMethod<(Integer) -> Integer> = \
             {{ |x: Integer| -> Integer x }}; bad }} }} M.run()"
        )),
        Err(EvaluationError::TypeContractError)
    );

    // C096 checks signature compatibility AT THE CALL SITE, so both kinds
    // remain ordinary callables once their kind matches.
    assert_eq!(
        evaluate(&format!(
            "{base} let b = m.bind(A.new()); b.call(5) }} }} M.run()"
        )),
        Ok(RuntimeValue::Integer(5_u8.into()))
    );
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               let c: Closure<(Integer) -> Integer> = { |x: Integer| -> Integer x }; \
               c.call(7) } } M.run()"
        ),
        Ok(RuntimeValue::Integer(7_u8.into()))
    );
}

#[test]
fn c057_scopes_module_private_authorization_to_one_edge() {
    // C057 scopes Module private authorization to
    // `(host logical Class, closed Module identity, composition edge, edge
    // revision)`, so the grant lets the MODULE's code reach the HOST's private
    // selector and nothing wider.
    let base = "module M { public fun reach() -> Object { \
                  try { secret() } catch e { e } } } ";

    // A grant in one host MUST NOT grant access in another: the granted edge
    // reaches the private selector, the ungranted host is denied.
    assert_eq!(
        evaluate(&format!(
            "{base} class Granted mixin M private {{ private fun secret() -> Symbol {{ :g }} }} \
             class Other mixin M {{ private fun secret() -> Symbol {{ :o }} }} \
             module Q {{ public fun run() -> Object {{ \
               [Granted.new().reach(), Other.new().reach()] }} }} Q.run()"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("g".into()),
            RuntimeValue::Symbol("MethodVisibilityError".into()),
        ])))
    );

    // Removing the edge atomically REVOKES that edge's authorization, and
    // re-including without private authorization does not inherit the old
    // grant: the same call answers the visibility denial afterwards.
    assert_eq!(
        evaluate(&format!(
            "{base} class G3 mixin M private {{ private fun secret() -> Symbol {{ :g }} }} \
             module Q {{ public fun run() -> Object {{ \
               let before = G3.new().reach(); \
               let removed = Reflection::Class.remove_module(G3, :M); \
               let added = G3.add_module(:M); \
               [before, G3.new().reach()] }} }} Q.run()"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("g".into()),
            RuntimeValue::Symbol("MethodVisibilityError".into()),
        ])))
    );
}

#[test]
fn c025_preserves_the_recorded_native_signature() {
    // C025 makes native Method binding PRESERVE the callable signature
    // recorded in metadata, with arguments crossing through declared primitive
    // interop records or handles. The signature was validated and then
    // DISCARDED, so nothing could guard the boundary afterwards.
    let base = "module M { public fun run() -> Object { \
                  let lib = FFI.open(\"libfixture.so\"); \
                  mut ptr = %{}; ptr[:type] = :pointer; ptr[:nullable] = false; \
                  ptr[:ownership] = :borrowed; \
                  mut sig = %{}; sig[:convention] = :c; sig[:parameters] = [:i32, ptr]; \
                  sig[:result] = :i32; sig[:errors] = :status; \
                  let bound = lib.bind(:c_wrapper, sig); ";

    // A declared primitive and a handle parameter bind, and the recorded
    // signature is readable back through the Library.
    assert_eq!(
        evaluate(&format!(
            "{base} [bound.bound?(:c_wrapper), \
             bound.signature(:c_wrapper)[:convention], \
             bound.signature(:c_wrapper)[:result]] }} }} M.run()"
        )),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Bool(true),
            RuntimeValue::Symbol("c".into()),
            RuntimeValue::Symbol("i32".into()),
        ])))
    );

    // A symbol that was never bound records no signature, so the surface
    // reports what binding preserved rather than fabricating one.
    assert_eq!(
        evaluate(&format!(
            "{base} bound.signature(:never_bound) }} }} M.run()"
        )),
        Ok(RuntimeValue::Nil)
    );
}

#[test]
fn c025_resumes_a_suspension_inside_try_without_double_cleanup() {
    // C013 makes a suspension a REGISTERED CONTINUATION rather than an exit,
    // so the protected region has not been left and `finally` has not been
    // reached. Cleanup therefore runs ONCE, on the real exit after resumption.
    assert_eq!(
        evaluate(
            "mut order = []; \
             module M { public async fun inner(g) -> Object { \
               try { let v = await g; order.append(:body); v } \
               finally { order.append(:cleanup) } } } \
             let g = Gate.new(); let t = M.inner(g); \
             let posted = Gate.complete(g, 7); \
             let resumed = Host.run(t); [resumed, order]"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(7_u8.into()),
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("body".into()),
                RuntimeValue::Symbol("cleanup".into()),
            ])),
        ])))
    );

    // C025: resumption retains the catch context and the surrounding control
    // state, so a suspension inside try/catch resumes into the same region.
    assert_eq!(
        evaluate(
            "module M { public async fun inner(g) -> Object { \
               try { await g } catch e { e } } } \
             let g = Gate.new(); let t = M.inner(g); \
             let posted = Gate.complete(g, 7); Host.run(t)"
        ),
        Ok(RuntimeValue::Integer(7_u8.into()))
    );
}

#[test]
fn c037_keeps_cleanup_lifo_and_suppressed_across_suspension() {
    // C037 keeps cleanup LIFO across async suspension, so a resumed body
    // closes the INNER resource before the outer one.
    assert_eq!(
        evaluate(
            "mut order = []; \
             class Outer { public fun close() -> Object { order.append(:outer); nil } } \
             class Inner { public fun close() -> Object { order.append(:inner); nil } } \
             module M { public async fun inner(g) -> Object { \
               using(Outer.new()) { using(Inner.new()) { let v = await g; v } } } } \
             let g = Gate.new(); let t = M.inner(g); \
             let posted = Gate.complete(g, 7); \
             let resumed = Host.run(t); [resumed, order]"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(7_u8.into()),
            RuntimeValue::Array(ArrayRef::new(vec![
                RuntimeValue::Symbol("inner".into()),
                RuntimeValue::Symbol("outer".into()),
            ])),
        ])))
    );

    // C037 also requires a cleanup failure AFTER resumption to use the same
    // primary-plus-suppressed ordering as synchronous cleanup, and forbids
    // losing it because the Task suspended.
    assert_eq!(
        evaluate(
            "class R { public fun close() -> Object { raise :close_failed } } \
             module M { public async fun inner(g) -> Object { \
               using(R.new()) { let v = await g; raise :body_failed } } } \
             let g = Gate.new(); let t = M.inner(g); \
             let posted = Gate.complete(g, 7); \
             try { Host.run(t) } catch v, c { [v, c.suppressed.length] }"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("body_failed".into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn c160_keeps_an_entered_frame_on_its_selected_body() {
    // C160: a frame that has ENTERED keeps the body it selected, while a LATER
    // send selects the replacement. The schedule needs a real pause inside the
    // frame, which an async suspension supplies: the frame is entered, parked
    // on a Gate, and a replacement commits while it is parked.
    assert_eq!(
        evaluate(
            "class A { public async fun m(g) -> Symbol { let v = await g; :old } } \
             let a = A.new(); let g = Gate.new(); let entered_task = a.m(g); \
             let replaced = A.open() { |t| t.define_method(:m) { |arg| :new } }; \
             let posted = Gate.complete(g, 1); \
             let entered = Host.run(entered_task); \
             let later = A.new().m(g); [entered, later]"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("old".into()),
            RuntimeValue::Symbol("new".into()),
        ])))
    );
}

#[test]
fn c011_leaves_live_state_intact_after_an_invalid_candidate() {
    // C011 constrains dynamic mutation by candidate transactions validated
    // against the static spine, and forbids an INVALID candidate from
    // partially mutating live Class state. C012 makes that spine a durable
    // promise, so the refusal must leave the prior live revision untouched.
    assert_eq!(
        evaluate(
            "contract C { fun m() -> Nil } \
             class A for C { public impl fun m() -> Nil { nil } \
               public fun ok() -> Symbol { :live } } \
             module Q { public fun run() -> Object { \
               let before = A.active_revision; \
               let refused = try { A.open() { |t| t.remove_contract(C) } } catch e { e }; \
               [refused, before, A.active_revision, A.new().ok()] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("TypeContractError".into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Symbol("live".into()),
        ])))
    );
}

#[test]
fn c160_refuses_an_integer_shift_beyond_the_resource_limit() {
    // C160 expects a resource refusal rather than an unbounded allocation. A
    // left shift's result needs `bits + count` bits, so the size is known
    // BEFORE the allocation; without the bound this ran until the host died.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { try { 1 << 1000000000 } catch e { e } } } M.run()"
        ),
        Ok(RuntimeValue::Symbol("ResourceError".into()))
    );

    // Ordinary exact-integer shifts are untouched, so the bound is a RESOURCE
    // limit rather than a narrowing of Integer precision. The result is
    // compared against the same shift written as a literal, which keeps the
    // assertion in Iris rather than reconstructing the value in Rust.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               (1 << 100) == 1267650600228229401496703205376 } } M.run()"
        ),
        Ok(RuntimeValue::Bool(true))
    );
}

#[test]
fn c042_reflects_a_contract_requirement_return_type() {
    // C042 lets a Contract body declare Method REQUIREMENTS, and the declared
    // return Type is already recorded for conformance checking. Exposing it
    // makes the normalized Type observable rather than only enforced.
    assert_eq!(
        evaluate(
            "contract C { fun m() -> Integer } \
             module Q { public fun run() -> Object { \
               [Reflection::Contract.requirement(C, :m)[:return_type], Integer.type] } } Q.run()"
        ),
        // C016 interns Type objects by identity, so the reflected requirement
        // Type IS the ordinary `Integer.type` rather than a copy of it.
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Type(iris_runtime::ClassId::new(3), Vec::new()),
            RuntimeValue::Type(iris_runtime::ClassId::new(3), Vec::new()),
        ])))
    );

    // C109 answers nil for an ABSENT lookup rather than fabricating a
    // requirement the Contract never declared.
    assert_eq!(
        evaluate(
            "contract C { fun m() -> Integer } \
             module Q { public fun run() -> Object { Reflection::Contract.requirement(C, :absent) } } Q.run()"
        ),
        Ok(RuntimeValue::Nil)
    );
}

#[test]
fn c061_admits_a_closed_generic_contract_parent() {
    // C061 interns ONE Contract per generic definition, so a closed parent
    // such as `extends Base<Integer>` names the same Contract its bare form
    // does. Accepting only the bare name refused a grammatical
    // `type_expr_list` entry outright.
    assert_eq!(
        evaluate(
            "contract Base<T> { fun m() -> Object } \
             contract Sub extends Base<Integer> { fun n() -> Object } \
             module Q { public fun run() -> Object { Sub.parents.length } } Q.run()"
        ),
        Ok(RuntimeValue::Integer(1_u8.into()))
    );
}

#[test]
fn c013_reflects_traversal_contract_requirement_types() {
    // D-466 fixes the traversal Contracts as `contract Iterable<T> { fun
    // iterator() -> Iterator<T> }` and `contract Iterator<T> { fun next() ->
    // Iteration<T>; fun close() -> Nil }`, and says `for` uses these EXACT
    // Contracts, so they are built in rather than declared per program.
    assert_eq!(
        evaluate(
            "contract Numbers extends Iterable<Integer> { fun iterator() -> Iterator<Integer> } \
             module Q { public fun run() -> Object { [ \
               Reflection::Contract.requirement(Numbers, :iterator)[:return_type], \
               Reflection::Contract.requirement(Iterator<Integer>, :next)[:return_type], \
               Reflection::Contract.requirement(Iterator<Integer>, :close)[:return_type]] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            // C061 interns one Contract per generic definition, so the closed
            // and bare forms normalize to the SAME Contract.
            RuntimeValue::ComposedType(iris_runtime::ComposedType::Intersection(vec![
                iris_runtime::TypeAtom::Contract(iris_runtime::ContractId::new(1)),
            ])),
            RuntimeValue::ComposedType(iris_runtime::ComposedType::Intersection(vec![
                iris_runtime::TypeAtom::Contract(iris_runtime::ContractId::new(2)),
            ])),
            // `close` promises Nil, which is an ordinary nominal Type.
            RuntimeValue::Type(iris_runtime::ClassId::new(1), Vec::new()),
        ])))
    );
}

#[test]
fn c004_diagnoses_a_read_before_definite_assignment() {
    // C004 lets a TYPED `mut name` defer initialization and makes the FIRST
    // assignment initialize it, so a read before that point is
    // DefiniteAssignmentError rather than a nil read.
    assert_eq!(
        evaluate("module M { public fun run() -> Object { mut x: Integer; x = 5; x } } M.run()"),
        Ok(RuntimeValue::Integer(5_u8.into()))
    );
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               mut x: Integer; try { x } catch e { e } } } M.run()"
        ),
        Ok(RuntimeValue::Symbol("DefiniteAssignmentError".into()))
    );

    // Later assignments UPDATE the initialized cell rather than re-deferring.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               mut x: Integer; x = 5; let first = x; x = 7; [first, x] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(5_u8.into()),
            RuntimeValue::Integer(7_u8.into()),
        ])))
    );

    // C011 keeps an ABSENT name a NameError, which is a different failure: the
    // deferred cell exists and holds nothing, an absent name does not exist.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               try { absent_name } catch e { e } } } M.run()"
        ),
        Ok(RuntimeValue::Symbol("NameError".into()))
    );
}

#[test]
fn c081_checks_each_meta_capability_separately_and_atomically() {
    // C080 makes MetaCapabilities ORTHOGONAL: denying one must not deny
    // another. `method_set` covers adding a slot, `method_body` replacing a
    // compatible body, so denying the latter leaves `define_method` allowed.
    assert_eq!(
        evaluate(
            "class A meta deny method_body { } \
             module Q { public fun run() -> Object { \
               try { A.define_method(:x) { 1 } } catch e { e } } } Q.run()"
        ),
        Ok(RuntimeValue::Nil)
    );
    assert_eq!(
        evaluate(
            "class A meta deny method_set { } \
             module Q { public fun run() -> Object { \
               try { A.define_method(:x) { 1 } } catch e { e } } } Q.run()"
        ),
        Ok(RuntimeValue::Symbol("MetaCapabilityError".into()))
    );

    // C081 makes the method-slot operation "static only for origin", so a
    // Class DECLARING its own members is not performing a meta operation on
    // itself: the deny applies to later operations, not to the declaration.
    // The denied operation then fails ATOMICALLY, leaving the revision and the
    // declared member untouched.
    assert_eq!(
        evaluate(
            "class A meta deny method_set { public fun m() -> Symbol { :live } } \
             module Q { public fun run() -> Object { \
               let before = A.active_revision; \
               let refused = try { A.define_method(:x) { 1 } } catch e { e }; \
               [refused, before, A.active_revision, A.new().m()] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("MetaCapabilityError".into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Symbol("live".into()),
        ])))
    );

    // C152 makes an inherited or composed deny immutable for the revision, so
    // a subclass and a Module-composed host both carry it.
    assert_eq!(
        evaluate(
            "class A meta deny method_set { } class B extends A { } \
             module Q { public fun run() -> Object { \
               [B.denied_capabilities, try { B.define_method(:x) { 1 } } catch e { e }] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Symbol(
                "method_set".into()
            )])),
            RuntimeValue::Symbol("MetaCapabilityError".into()),
        ])))
    );
}

#[test]
fn c056_construction_uses_the_captured_revision() {
    // C056 snapshots A's active revision at construction START for allocation,
    // layout, stored property initialization, and the INITIAL `initialize`
    // dispatch, and continues with it when a new revision commits before
    // construction completes.
    //
    // A stored-property initializer runs BEFORE that dispatch, so committing a
    // replacement from inside one is a genuine mid-construction commit.
    let armed = "mut ran = :none; \
                 class A { property tag: Symbol = arm() \
                   public fun initialize() -> Object { ran = :original; nil } \
                   public fun arm() -> Symbol { \
                     let committed = A.open() { |t| \
                       t.define_method(:initialize) { ran = :replacement; nil } }; \
                     :armed } } \
                 module Q { public fun run() -> Object { let a = A.new(); [a.tag, ran] } } Q.run()";
    assert_eq!(
        evaluate(armed),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("armed".into()),
            // The ORIGINAL initialize ran: the commit landed after the snapshot.
            RuntimeValue::Symbol("original".into()),
        ])))
    );

    // The same replacement committed BEFORE construction starts does take
    // effect, which is what makes the result above evidence of the snapshot
    // rather than of the replacement never working.
    assert_eq!(
        evaluate(
            "mut ran = :none; \
             class A { public fun initialize() -> Object { ran = :original; nil } } \
             module Q { public fun run() -> Object { \
               let committed = A.open() { |t| \
                 t.define_method(:initialize) { ran = :replacement; nil } }; \
               let a = A.new(); ran } } Q.run()"
        ),
        Ok(RuntimeValue::Symbol("replacement".into()))
    );

    // C056's other half: a LATER ordinary send uses the then-current active
    // revision, so the member added mid-construction is reachable afterwards.
    assert_eq!(
        evaluate(
            "class A { property tag: Symbol = arm() \
               public fun initialize() -> Object { nil } \
               public fun arm() -> Symbol { \
                 let committed = A.open() { |t| t.define_method(:m) { :new } }; :armed } \
               public fun m() -> Symbol { :old } } \
             module Q { public fun run() -> Object { let a = A.new(); [a.tag, a.m()] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("armed".into()),
            RuntimeValue::Symbol("new".into()),
        ])))
    );
}

#[test]
fn c055_recovers_from_a_configured_sink_rather_than_the_queue() {
    // C055 makes zero-loss audit recovery use a SEPARATELY CONFIGURED
    // persistent sink, and forbids an unbounded in-memory subscriber queue
    // from being the semantic guarantee.
    //
    // Pruning retained history is what tells the two apart: the in-memory
    // read then fails, while the sink still answers.
    assert_eq!(
        evaluate(
            "class B {} module M { public fun run() -> Object { \
               RevisionHistory.configure_sink(); \
               B.open() { |t| 1 }; \
               RevisionHistory.prune(1); \
               [try { RevisionHistory.events(1, 1) } catch e { e }, \
                RevisionHistory.recover(1, 1)] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("AuditHistoryUnavailableError".into()),
            RuntimeValue::Array(ArrayRef::new(vec![RuntimeValue::Integer(1_u8.into())])),
        ])))
    );

    // With NO sink configured there is no zero-loss guarantee to offer, which
    // is C055's point: the in-memory queue is not it.
    assert_eq!(
        evaluate(
            "class B {} module M { public fun run() -> Object { \
               B.open() { |t| 1 }; \
               try { RevisionHistory.recover(1, 1) } catch e { e } } } M.run()"
        ),
        Ok(RuntimeValue::Symbol("AuditHistoryUnavailableError".into()))
    );

    // C053's retained-history path is untouched when nothing is pruned.
    assert_eq!(
        evaluate(
            "class B {} module M { public fun run() -> Object { \
               B.open() { |t| 1 }; RevisionHistory.events(1, 1) } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(1_u8.into())
        ])))
    );
}

#[test]
fn c161_replacement_creates_no_second_backing_slot() {
    // C148 lets a stable value Class be opened to alter property protocols,
    // and C161 makes a compatible `property fun` replacement change only the
    // accessor body: it creates NO second backing slot.
    //
    // The absence is observed through `Reflection::Class.properties`, which
    // META-C097 lists on the CLASS reflection surface. The class is not an
    // identity-less value, so C004's restriction - the one that makes
    // `Reflection::Object.list_ivars` refuse `1.0f64` - does not apply here.
    assert_eq!(
        evaluate(
            "mut recorded = :none; \
             open class Float64 { \
               public override property fun infinity() -> Symbol { :replaced } \
               public property fun infinity=(v) -> Object { recorded = v; nil } } \
             module Q { public fun run() -> Object { \
               let read = Float64.infinity; \
               let wrote = Float64.infinity = :written; \
               [read, recorded, Reflection::Class.properties(Float64)] } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("replaced".into()),
            RuntimeValue::Symbol("written".into()),
            RuntimeValue::Array(ArrayRef::new(Vec::new())),
        ])))
    );

    // The empty list is evidence of ABSENCE rather than of the surface
    // reporting nothing: a Class that DECLARES a stored property still lists
    // its slot after the same kind of getter replacement.
    assert_eq!(
        evaluate(
            "class A { property tag: Symbol = :t } \
             open class A { public override property fun tag() -> Symbol { :replaced } } \
             module Q { public fun run() -> Object { \
               Reflection::Class.properties(A) } } Q.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Symbol("@tag".into())
        ])))
    );
}

#[test]
fn c060_publishes_complete_content_to_a_synchronized_observer() {
    // C060 makes MutableString not logically thread-safe but REQUIRES it to
    // stay memory-safe, never expose invalid Unicode, and publish either old
    // complete content or new complete content to a correctly synchronized
    // observer. The write happens on a REAL second thread, which an `Rc` cell
    // could not have carried at all.
    assert_eq!(
        evaluate(
            "module M { public fun run() -> Object { \
               let text = m\"old\"; \
               NativeFixture.concurrently_replace(text, \"new\"); \
               [text.to_string(), text.length] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Text("new".into()),
            RuntimeValue::Integer(3_u8.into()),
        ])))
    );

    // Multi-byte content on both sides: a torn write would leave a partial
    // scalar, so round-tripping the exact text AND its scalar length is what
    // shows the observer never sees invalid Unicode.
    let multi = "module M { public fun run() -> Object { \
                   let text = m\"éè\"; \
                   NativeFixture.concurrently_replace(text, \"üöä\"); \
                   [text.to_string(), text.length] } } M.run()";
    assert_eq!(
        evaluate(multi),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Text("üöä".into()),
            RuntimeValue::Integer(3_u8.into()),
        ])))
    );
}

#[test]
fn a_session_keeps_state_across_chunks() -> Result<(), EvaluationError> {
    // A REPL evaluates one line at a time. Re-running earlier lines cannot
    // stand in for a session: it repeats their side effects, and a mutation
    // made by a line that is not itself a binding is lost entirely.
    let mut session = crate::Session::new()?;

    session.evaluate(
        "class Account { property balance: Integer = 0 \
           public fun deposit(n: Integer) -> Integer { @balance = @balance + n; @balance } }\nnil",
    )?;
    session.evaluate("let account = Account.new()\nnil")?;

    // A Class declared in an EARLIER chunk is still usable, and the deposit
    // below is exactly the kind of non-binding mutation replay would drop.
    assert_eq!(
        session.evaluate("account.deposit(30)"),
        Ok(RuntimeValue::Integer(30_u8.into()))
    );
    assert_eq!(
        session.evaluate("account.deposit(12)"),
        Ok(RuntimeValue::Integer(42_u8.into()))
    );
    assert_eq!(
        session.evaluate("account.balance"),
        Ok(RuntimeValue::Integer(42_u8.into()))
    );
    Ok(())
}

#[test]
fn a_failed_chunk_leaves_the_session_usable() -> Result<(), EvaluationError> {
    // A mistyped line must not end the session, or a REPL would be unusable.
    let mut session = crate::Session::new()?;
    session.evaluate("let a = 5\nnil")?;

    assert!(session.evaluate("bogus_name").is_err());

    assert_eq!(
        session.evaluate("a + 1"),
        Ok(RuntimeValue::Integer(6_u8.into()))
    );
    Ok(())
}

#[test]
fn a_bare_print_call_reaches_the_source_runtime() {
    // `print` is a bare-name helper the source runtime owns. A top-level
    // `print(...)` used to stay in the literal evaluator, which has no such
    // name, and answered NameError - so a script could produce no output.
    assert_eq!(evaluate("print(1 + 2)"), Ok(RuntimeValue::Nil));
}

#[test]
fn a_top_level_collection_frees_only_unreachable_objects() {
    // `D-111` keeps an identity hash stable across movement BY GC, which is
    // only observable once something can actually die.
    //
    // The bound object is reachable and the anonymous one is not, so exactly
    // one is freed and the survivor keeps its hash across the move.
    assert_eq!(
        evaluate(
            "class A { } let keep = A.new(); let dropped = A.new(); \
             let before = keep.hash(); \
             let released = dropped.hash(); \
             let discarded = A.new().hash(); \
             let freed = NativeFixture.compact_gc(); \
             [freed, before == keep.hash(), released == dropped.hash()]"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ])))
    );

    // Control: with BOTH objects bound, nothing is unreachable and nothing is
    // freed. Without this the count above could mean the collector frees
    // indiscriminately.
    assert_eq!(
        evaluate("class A { } let a = A.new(); let b = A.new(); NativeFixture.compact_gc()"),
        Ok(RuntimeValue::Integer(0_u8.into()))
    );
}

#[test]
fn a_collection_inside_a_closure_keeps_captured_values() {
    // A Closure body runs through the same block path, so its captured
    // environment is registered as a frame too. Previously a collection
    // refused inside ANY nested invocation, closures included.
    assert_eq!(
        evaluate(
            "class A { } \
             module M { public fun run() -> Object { \
               let kept = A.new(); \
               let before = kept.hash(); \
               let check = { |x|; let dead = A.new().hash(); NativeFixture.compact_gc() }; \
               let freed = check.call(1); \
               [freed, before == kept.hash()] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Bool(true),
        ])))
    );
}

#[test]
fn bitwise_operators_work_on_literal_receivers() {
    // The literal evaluator listed the SHIFTS but not the bitwise operators,
    // so `6 & 3` answered UnsupportedConstruct while `let a = 6; a & 3`
    // succeeded - the same expression resolving differently because a binding
    // routed the program to the other evaluator.
    //
    // The differential harness surfaced this: the bytecode backend answered 2
    // and the reference answered an error, which is a disagreement no single
    // backend could have revealed.
    assert_eq!(evaluate("6 & 3"), Ok(RuntimeValue::Integer(2_u8.into())));
    assert_eq!(evaluate("6 | 3"), Ok(RuntimeValue::Integer(7_u8.into())));
    assert_eq!(evaluate("6 ^ 3"), Ok(RuntimeValue::Integer(5_u8.into())));

    // The bound form already worked and must keep working.
    assert_eq!(
        evaluate("let a = 6; a & 3"),
        Ok(RuntimeValue::Integer(2_u8.into()))
    );
}

#[test]
fn a_collection_inside_a_method_keeps_the_callers_locals() {
    // Locals are threaded through evaluation as a parameter, so a caller's
    // bindings used to live only on the Rust stack: a collection could not see
    // them and would have freed objects the caller still held, which is why it
    // refused to run inside a Method body at all.
    //
    // Each active block now registers its locals, so the live frames are the
    // root set. A collection triggered in a CALLEE must therefore leave the
    // caller's object alive.
    assert_eq!(
        evaluate(
            "class A { } \
             module M { \
               public fun inner() -> Object { let dead = A.new().hash(); \
                 NativeFixture.compact_gc() } \
               public fun run() -> Object { \
                 let outer = A.new(); \
                 let before = outer.hash(); \
                 let freed = M.inner(); \
                 [freed, before == outer.hash()] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            // The callee's own garbage is collected...
            RuntimeValue::Integer(1_u8.into()),
            // ...and the CALLER's local survived with its hash intact.
            RuntimeValue::Bool(true),
        ])))
    );

    // Control: with nothing unreachable, a collection inside a method frees
    // NOTHING rather than freeing a live local.
    assert_eq!(
        evaluate(
            "class A { } \
             module M { \
               public fun run() -> Object { \
                 let keep = A.new(); \
                 let before = keep.hash(); \
                 let freed = NativeFixture.compact_gc(); \
                 [freed, before == keep.hash()] } } M.run()"
        ),
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(0_u8.into()),
            RuntimeValue::Bool(true),
        ])))
    );
}

/// A compound assignment reads the target once and, when logical, SHORT-CIRCUITS.
///
/// `IRIS-V1-CONTROL-C036` sends the ordinary operator to the read value, and
/// `C037` evaluates the right side only on the writing path. Lowering the
/// logical forms as `x = x || v` would satisfy the value cases while still
/// running the right side unconditionally, so the observable evidence is a
/// side effect that must NOT happen.
#[test]
fn compound_assignment_reads_once_and_short_circuits() {
    for (source, expected) in [
        ("mut x = 1; x += 2; x", "3"),
        ("mut x = 10; x -= 3; x", "7"),
        ("mut x = 2; x **= 3; x", "8"),
        ("mut x = 6; x &= 3; x", "2"),
        ("mut x = nil; let r = x ||= 7; [r, x]", "[7, 7]"),
        (
            "mut y = :kept; let s = (y ||= :other); [s, y]",
            "[:kept, :kept]",
        ),
        ("mut t = 1; let s = (t &&= 9); [s, t]", "[9, 9]"),
        // The right side must NOT run: an appended `:ran` here is the wrong
        // answer an unconditional desugaring produces.
        (
            "mut x = nil; mut log = []; let r = x &&= log.append(:ran); [r, log]",
            "[nil, []]",
        ),
        // ...and the mirrored control: `||=` skips its right side when the
        // target is already truthy.
        (
            "mut y = :kept; mut log = []; let r = y ||= log.append(:ran); [r, log]",
            "[:kept, []]",
        ),
    ] {
        let wrapped = format!("module M {{ public fun r() -> Object {{ {source} }} }} M.r()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// Truth runs the `to_bool` PROTOCOL, not a structural test of the value.
///
/// `IRIS-V1-CONTROL-C022` makes `false` and `nil` falsey by default, but a
/// class may define `to_bool` and then its result decides. Testing the value's
/// shape instead answered `:yes` for an object whose `to_bool` answers false -
/// a wrong answer rather than a hold, and it reached every `if`, `while`,
/// `&&`, `||` and `!` alike.
#[test]
fn truth_consults_an_authored_to_bool() {
    const FALSEY: &str = "class P { public fun to_bool() -> Bool { false } } ";
    for (source, expected) in [
        // The authored `to_bool` decides, against the value's own shape.
        (
            format!("{FALSEY}module M {{ public fun r() -> Object {{ if P.new() {{ :yes }} else {{ :no }} }} }} M.r()"),
            ":no",
        ),
        (
            format!("{FALSEY}module M {{ public fun r() -> Object {{ !P.new() }} }} M.r()"),
            "true",
        ),
        // `&&` and `||` short-circuit on the PROTOCOL's answer, while the
        // result keeps the original VALUE rather than the Bool it produced.
        (
            format!("{FALSEY}module M {{ public fun r() -> Object {{ P.new() && :rhs }} }} M.r()"),
            "<object>",
        ),
        (
            format!("{FALSEY}module M {{ public fun r() -> Object {{ P.new() || :rhs }} }} M.r()"),
            ":rhs",
        ),
        // ...and a `while` never runs its body.
        (
            format!("{FALSEY}module M {{ public fun r() -> Object {{ mut n = 0; while P.new() {{ n = n + 1 }}; n }} }} M.r()"),
            "0",
        ),
        // Control: with NO authored `to_bool`, the default stands, so the
        // change widened truth rather than replacing it.
        (
            "module M { public fun r() -> Object { if 0 { :yes } else { :no } } } M.r()".to_owned(),
            ":yes",
        ),
        (
            "module M { public fun r() -> Object { [!nil, !false, !1, !\"\"] } } M.r()".to_owned(),
            "[true, true, false, false]",
        ),
        // An `if` in EXPRESSION position answers a value, and a missing else
        // answers nil.
        (
            "module M { public fun r() -> Object { let a = if true { 1 }; let b = if false { 2 }; [a, b] } } M.r()".to_owned(),
            "[1, nil]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            &source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}
/// A declaration the backend cannot ELABORATE still runs to its own answer.
///
/// Three forms were declined as gaps while the reference simply runs them: an
/// empty body is a method returning nil, a deferred `let` is a declaration
/// that answers nil and fails only when READ, and a keyword argument is an
/// ordinary argument value.
#[test]
fn accepted_declaration_forms_run_rather_than_decline() {
    for (source, expected) in [
        ("class A { public fun f() { } }; A.new().f()", "nil"),
        ("let x: Integer", "nil"),
        ("mut x", "nil"),
        ("let y: Integer; 7", "[nil, 7]"),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A deferred binding that is READ still fails, so accepting the
    // declaration widened the rule rather than removing the check.
    let agreement = crate::backend::compare_backends(
        "module M { public fun run() -> Object { let a; a } } M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("NameError".to_owned())
    );

    // A keyword argument to a RESOLVED function stays declined: the backend
    // has no keyword parameters, so lowering it would bind the wrapper into a
    // positional slot and answer `a: 1` where the reference raises.
    let crate::backend::Support::Unsupported(reason) =
        <crate::backend::Bytecode as crate::backend::Backend>::execute(
            &crate::backend::Bytecode,
            "module M { public fun r() -> Object { M.f(a: 1) } \
             public fun f(a: Integer) -> Integer { a } } M.r()",
        )
    else {
        unreachable!("a keyword argument to a positional function must be declined")
    };
    assert_eq!(reason, "expression keyword argument");
}

/// Mixins compose, and a Contract conformance is not enforced at DECLARATION.
///
/// A `mixin` names a Module the runtime already composes into a class's MRO,
/// so wiring the declaration through was enough. The Contract check was the
/// opposite: the backend demanded an `impl` marker for every requirement, but
/// the reference runs `class X for C { }` with `C` unimplemented, so the rule
/// refused programs the language accepts.
#[test]
fn mixins_compose_and_conformance_is_not_enforced_at_declaration() {
    for (source, expected) in [
        // A later mixin WINS, which is the MRO order rather than a first match.
        (
            "module A { public fun w() { :a } } module B { public fun w() { :b } } \
             class C mixin A, B { } C.new().w()",
            ":b",
        ),
        (
            "module A { public fun h() -> Integer { 1 } } class C mixin A { } C.new().h()",
            "1",
        ),
        // The class's OWN method still outranks a mixed-in one.
        (
            "module A { public fun h() -> Integer { 1 } } \
             class C mixin A { public fun h() -> Integer { 2 } } C.new().h()",
            "2",
        ),
        (
            "contract C { fun m() -> String } class X for C { public fun m() -> String { \"c\" } } \
             X.new().m()",
            "\"c\"",
        ),
        // A cast through an UNIMPLEMENTED requirement is not refused...
        (
            "contract Named { fun name() -> String } class User for Named { } \
             module M { public fun r() -> Object { User.new() as Named } } M.r()",
            "<contract>",
        ),
        // ...and a send through the view dispatches the object's own method.
        (
            "contract Named { fun name() -> String } \
             class User for Named { public fun name() -> String { \"n\" } } \
             module M { public fun r() -> Object { (User.new() as Named).name() } } M.r()",
            "\"n\"",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A module `const` is LEXICALLY visible in its methods, not a member.
///
/// `IRIS-V1-CONTROL-D-432` puts a constant in the module's qualified
/// namespace rather than its selector table, so `M.K` is a MessageNotFound
/// while a method of `M` reads `K` directly. That missing selector also used
/// to surface as `UnknownSelector`, a MACHINE DEFECT rather than a program
/// error, which held the row instead of agreeing.
#[test]
fn a_module_constant_is_lexical_rather_than_a_member() {
    for (source, expected) in [
        (
            "module M { const K = 1 public fun r() -> Object { K } } M.r()",
            "1",
        ),
        // A local binding SHADOWS the constant, which is the lexical order.
        (
            "module M { const K = 1 public module fun lexical() -> Object { let K = 9; K } } \
             M.lexical()",
            "9",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: the constant is NOT reachable as a member, and the failure is
    // a program error rather than a machine defect.
    for source in [
        "module M { const K = 1 public fun r() -> Object { K } } [M.r(), M.K]",
        "module M { const K = 1 public fun r() -> Object { M.K } } M.r()",
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        let crate::backend::Observation::Error(error) = observation else {
            unreachable!("reading a constant as a member must fail: {source}")
        };
        assert!(!error.contains("machine defect"), "{source}: {error}");
        assert!(error.contains("MessageNotFound"), "{source}: {error}");
    }
}

/// A reopen may PRECEDE the class it reopens.
///
/// Declarations were collected in source order, so `open class A { }` written
/// ahead of `class A { }` found no target and was declined - an ordering the
/// language does not impose. Origins are collected first now.
#[test]
fn a_reopen_may_precede_its_target() {
    for (source, expected) in [
        (
            "open class A { public override fun marker() -> String { \"open\" } }; \
             class A { public fun marker() -> String { \"origin\" } }; A.new().marker()",
            "\"open\"",
        ),
        // Control: the ALREADY-working order still works, so reordering
        // collection did not trade one direction for the other.
        (
            "class A { public fun m() -> Integer { 1 } } \
             open class A { public fun n() -> Integer { 2 } } A.new().n()",
            "2",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A stored-property initializer is an EXPRESSION evaluated at construction.
///
/// Only a literal could be stored before, so `property tag: Symbol = arm()`
/// was declined outright. It is lowered as a frame with `self` bound now,
/// which is what lets it call the object's own methods, and a superclass
/// initializes first so a subclass sees a fully built base.
#[test]
fn stored_property_initializers_run_at_construction() {
    for (source, expected) in [
        // The initializer CALLS the object's own method through implicit self.
        (
            "class A { property tag: Symbol = arm() public fun arm() -> Symbol { :armed } } \
             A.new().tag",
            ":armed",
        ),
        ("class C { property n: Integer = 1 + 2 } C.new().n", "3"),
        // A side effect happens ONCE, when the object is built.
        (
            "mut log = []; class B { property b: Nil = log.append(:base) } let x = B.new(); log",
            "[:base]",
        ),
        // The BASE class initializes first, and `initialize` runs after both.
        (
            "mut log = []; class Base { property b: Nil = log.append(:base) } \
             class Child extends Base { property c: Nil = log.append(:child) \
             fun initialize() { log.append(:initialize) } } let x = Child.new(); log",
            "[:base, :child, :initialize]",
        ),
        // Control: a LITERAL initializer still works, so adding the frame did
        // not replace the direct path.
        ("class D { property n: Integer = 7 } D.new().n", "7"),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A module body's statements run at the declaration's SOURCE POSITION.
///
/// They are not a separate load phase: `module M { order.append(:body) }`
/// appends when `order` is already bound above it, and is a NameError when it
/// is not. Lowering them ahead of every top-level statement answered NameError
/// for the ordinary case, so the top level walks `entries` rather than
/// `statements` to keep declarations and statements interleaved.
#[test]
fn module_body_statements_run_in_source_position() {
    for (source, expected) in [
        (
            "mut order = []; order.append(:top); module M { order.append(:body) } order",
            "[nil, [:top, :body]]",
        ),
        ("mut r = 0; module M { let x = 5; r = x } r", "5"),
        // Each body runs in DECLARATION order, so the second sees the first.
        (
            "mut r = 0; module A { r = 1 } module B { r = r + 1 } r",
            "2",
        ),
        ("mut a = 1; module M { a = a + 1 } mut b = a; b", "2"),
        // A bare call in a module body names that MODULE's own function.
        (
            "mut log = 0; module M { fun helper() -> Integer { log = 1; 1 } helper() } log",
            "1",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a body running BEFORE the binding it reads still fails, so the
    // statements really are positioned rather than merely reordered.
    let agreement = crate::backend::compare_backends(
        "module M { order.append(:body) } mut order = []; order",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("NameError".to_owned())
    );
}

/// Regex literals compile, canonicalize their flags, and MATCH.
///
/// `IRIS-V1-COLLECTIONS-C081` stores flags in `imsx` order with absent flags
/// omitted, so `/a+/im` and `/a+/mi` are one value and hash alike. `C082`
/// makes `=~` and `!~` ordinary sends on the subject, and `C083` exposes the
/// match's SCALAR ranges alongside its byte ranges - they differ for any
/// non-ASCII subject - while keeping capture absence distinct from an empty
/// capture.
#[test]
fn regex_literals_compile_and_match() {
    for (source, expected) in [
        (r#"("abc" =~ /b/).text()"#, "\"b\""),
        // A failed match answers nil, and `!~` is true exactly then.
        (r#""abc" =~ /z/"#, "nil"),
        (r#""abc" !~ /z/"#, "true"),
        (r#""abc" !~ /b/"#, "false"),
        // Flags CANONICALIZE, so two spellings hash alike.
        (r#"/a+/im.hash() == /a+/mi.hash()"#, "true"),
        // A scalar offset counts CHARACTERS, not bytes.
        (r#"("a\u{00e9}b" =~ /b/).start()"#, "2"),
        (r#"("a\u{00e9}b" =~ /b/).byte_start()"#, "3"),
        // A numbered capture reads by 1-based index.
        (r#"("2026-08" =~ /(\d+)-(\d+)/).capture(2)"#, "\"08\""),
        // A case-insensitive flag actually applies, so the same subject
        // matches with `i` and does not without it.
        (r#"("ABC" =~ /b/i).text()"#, "\"B\""),
        (r#""ABC" !~ /b/"#, "true"),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a pattern the engine cannot support is still REFUSED, and by
    // the code the reference reports - each unsupported construct names
    // itself, so a backreference is distinguishable from a lookbehind.
    for (source, code) in [
        (
            r"module M { public fun run() -> Object { /(a)\1/ } } M.run()",
            r#"LexicalDiagnostic("REGEX_UNSUPPORTED_BACKREFERENCE")"#,
        ),
        (
            "module M { public fun run() -> Object { /(?<=a)b/ } } M.run()",
            r#"LexicalDiagnostic("REGEX_UNSUPPORTED_LOOKBEHIND")"#,
        ),
        (
            "module M { public fun run() -> Object { /a/ii } } M.run()",
            r#"LexicalDiagnostic("LEX_BAD_REGEX_FLAGS")"#,
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error(code.to_owned()),
            "{source}"
        );
    }

    // An INTERPOLATING pattern splices a value the compiler cannot know, and
    // the spliced text is matched LITERALLY rather than as pattern syntax.
    for (source, expected) in [
        (r#"let x = "a+b"; ("a+b" =~ /${x}/).text()"#, r#""a+b""#),
        (r#"let x = "a+b"; ("aab" =~ /${x}/) == nil"#, "true"),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A Gate SUSPENDS an async frame mid-body, and completing it resumes.
///
/// `IRIS-V1-ASYNC-C012` makes creating a Task and starting its first run one
/// operation, so a body runs EAGERLY and only pauses when it awaits a pending
/// Gate. `C014` then resumes the parked frames in the order they suspended.
/// The frame keeps its own register file across the pause, which is what lets
/// the prefix's locals survive to the other side of the `await`.
#[test]
fn a_gate_suspends_and_resumes_an_async_frame() {
    for (source, expected) in [
        // The body runs at CALL time, before anything observes the Task.
        (
            "mut log = []; module M { public async fun f() -> Symbol { log.append(:ran); :d } } \
             let t = M.f(); log",
            "[:ran]",
        ),
        // An `await` on a PENDING gate stops after the prefix.
        (
            "mut log = []; module M { public async fun f(g) -> Symbol { log.append(:before); \
             await g; log.append(:after); :d } } let g = Gate.new(); let t = M.f(g); log",
            "[:before]",
        ),
        // Completing the gate resumes it, and the POSTED value is the await's.
        (
            "module M { public async fun inner(g) -> Object { let v = await g; v } } \
             let g = Gate.new(); let t = M.inner(g); let posted = Gate.complete(g, 7); Host.run(t)",
            "7",
        ),
        (
            "class A { public async fun f(gate: Object) -> Symbol { await gate; :done } } \
             module M { public fun run() -> Symbol { let gate = Gate.new(); \
             let task = A.new().f(gate); Gate.complete(gate, 1); Host.run(task) } } M.run()",
            ":done",
        ),
        // Two frames on ONE gate resume in the order they suspended.
        (
            "mut log = []; class A { public async fun f(gate: Object, tag: Symbol) -> Symbol { \
             await gate; log.append(tag); tag } } module M { public fun run() -> Array { \
             let gate = Gate.new(); let first = A.new().f(gate, :a); \
             let second = A.new().f(gate, :b); Gate.complete(gate, 1); Host.run(first); log } } \
             M.run()",
            "[:a, :b]",
        ),
        // Control: a body with NO await never parks, so it is already done.
        (
            "module M { public async fun f() -> Symbol { :d } } let t = M.f(); Host.run(t)",
            ":d",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Completing a Gate makes a frame READY without running it: the
    // continuation's effect appears only once something observes the Task.
    // Resuming inside `Gate.complete` made `:after` visible too early.
    let agreement = crate::backend::compare_backends(
        "mut log = []; module M { public async fun f(g) -> Symbol { log.append(:before); \
         await g; log.append(:after); :d } } let g = Gate.new(); let t = M.f(g); \
         Gate.complete(g, 1); log",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[nil, [:before]]".to_owned())
    );

    // Control: observing a task whose gate was NEVER completed still fails,
    // so resuming did not quietly complete every parked frame.
    let agreement = crate::backend::compare_backends(
        "module M { public async fun f(g) -> Symbol { await g; :d } } \
         let g = Gate.new(); let t = M.f(g); Host.run(t)",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// A module constant does not displace the function's PARAMETERS.
///
/// Constants were bound before the parameters, to make a same-named parameter
/// shadow one. But a call copies arguments into the LEADING registers, so
/// allocating anything first pushed every parameter out of them and the
/// verifier proved the argument registers unwritten - a machine defect
/// reaching any module function that took both a constant and a parameter.
/// Manual use found it; no single-feature test had both.
#[test]
fn a_module_constant_does_not_displace_parameters() {
    for (source, expected) in [
        (
            "module App { const TAG = :app public fun f(x) -> Object { [TAG, x] } } App.f(9)",
            "[:app, 9]",
        ),
        // Two constants and two parameters, so a displacement of ANY leading
        // register would show.
        (
            "module App { const A = 1 const B = 2 \
             public fun f(x, y) -> Object { [A, B, x, y] } } App.f(8, 9)",
            "[1, 2, 8, 9]",
        ),
        // A local still shadows the constant...
        (
            "module M { const K = 1 public module fun lexical() -> Object { let K = 9; K } } \
             M.lexical()",
            "9",
        ),
        // ...and so does a PARAMETER of the same name, which is what binding
        // the constants last would otherwise have reversed.
        (
            "module M { const K = 1 public fun f(K) -> Object { K } } M.f(5)",
            "5",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A source the PARSER refuses fails at run time, not at compile time.
///
/// The reference parses, refuses, and reports `ParseDiagnostic` when the
/// program RUNS. Declining refused the same program while describing it
/// differently, which holds the row rather than agreeing - the same mistake
/// that hid behind `name unbound` and `call unbound receiver`. It was the
/// second-largest bucket at 25 programs.
#[test]
fn a_parse_refusal_is_reported_at_run_time() {
    for source in [
        // An operator the grammar does not have.
        "mut x = 1; x %= 2",
        // Keywords the language deliberately does not define.
        "repeat { nil }",
        "switch value { when 1 { :one } }",
        "defer { cleanup() }",
        // A `try` with neither catch nor finally.
        "module M { public fun run() -> Object { try { 1 } } } M.run()",
        // A positional parameter after a rest parameter.
        "class A { public fun f(*items, item) { item } }; A.new().f(1)",
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error("ParseDiagnostic".to_owned()),
            "{source}"
        );
    }

    // Control: a program the parser ACCEPTS still runs, so raising the
    // diagnostic did not start refusing ordinary source.
    let agreement = crate::backend::compare_backends(
        "mut x = 1; x += 2; x",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("an accepted program must run: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[3, 3]".to_owned())
    );
}

/// The Unicode surface uses the PINNED data version, not a host locale.
///
/// `IRIS-V1-COLLECTIONS-C042` fixes the Unicode data version for case folding
/// and normalization, and `C044` exposes grapheme CLUSTERS explicitly because
/// `length` counts scalars and one cluster may span several of them.
#[test]
fn the_unicode_surface_uses_the_pinned_data_version() {
    for (source, expected) in [
        // `ß` folds to `ss`, which is a length change rather than a remap.
        (r#""\u{00DF}".casefold()"#, "\"ss\""),
        // `a` + combining diaeresis is ONE cluster but TWO scalars.
        (r#""a\u{0308}".graphemes().length()"#, "1"),
        (r#""a\u{0308}".length()"#, "2"),
        // Composition and decomposition round-trip through the same tables.
        (r#""e\u{301}".nfc()"#, "\"é\""),
        (r#""é".nfd().length()"#, "2"),
        // A Unicode property class works in a pattern, on the same version.
        (
            r#"[Unicode.version(), ("A" =~ /\p{Lu}/).text()]"#,
            "[\"17.0.0\", \"A\"]",
        ),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A CLASS-level property is class state, read and written off the Class.
///
/// It is not per-instance storage: `Cache.n` reads it and `Cache.n = 9`
/// writes it through the `n=` setter selector, which is what a class variable
/// already does. An absent initializer starts at nil, because the reference
/// lets the slot be written before it is ever read.
#[test]
fn a_class_level_property_is_class_state() {
    for (source, expected) in [
        (
            "class Cache { class property value: Integer } Cache.value = 1; Cache.value",
            "[1, 1]",
        ),
        ("class Cache { class property n: Integer = 5 } Cache.n", "5"),
        (
            "class Cache { class property n: Integer = 5 } Cache.n = 9; Cache.n",
            "[9, 9]",
        ),
        // A `shared` class property lives on the UNAPPLIED definition, so the
        // bare Class name reaches it.
        (
            "class Cache<T> { shared class property count: Integer = 0 } Cache.count",
            "0",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // On a GENERIC class a plain class property belongs to each closed
    // CONSTRUCTION rather than to the definition, so each holds its own value
    // and the BARE name reaches no slot at all.
    for (source, expected) in [
        (
            "class Cache<T> { class property value: T } Cache<String>.value = \"s\"; \
             Cache<Integer>.value = 1; [Cache<String>.value, Cache<Integer>.value]",
            Some("[\"s\", 1, [\"s\", 1]]"),
        ),
        // An unwritten construction starts at nil, since it may be read before
        // it is ever written.
        (
            "class Cache<T> { class property value: T } Cache<String>.value",
            Some("nil"),
        ),
        // THREE constructions stay independent, and one never written starts
        // at the declared initializer.
        (
            "class C<T> { class property n: Integer = 0 } C<String>.n = 1; C<Integer>.n = 2; \
             [C<String>.n, C<Integer>.n, C<Bool>.n]",
            Some("[1, 2, [1, 2, 0]]"),
        ),
        // A SHARED class property is on the definition, so the bare name does
        // reach that one.
        (
            "class C<T> { shared class property n: Integer = 7 } C.n",
            Some("7"),
        ),
        // The bare name has no slot: the property is per construction.
        ("class C<T> { class property n: Integer = 0 } C.n", None),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error(
                "MessageNotFound { receiver_class: \"Class\", selector: \"n\" }".to_owned(),
            ),
        };
        assert_eq!(observation, &wanted, "{source}");
    }

    // Control: a selector the Class does NOT have still fails, so consulting
    // the class variables did not make every name answer.
    let agreement = crate::backend::compare_backends(
        "class Cache { class property n: Integer = 5 } Cache.absent",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    let crate::backend::Observation::Error(error) = observation else {
        unreachable!("an absent class property must fail")
    };
    assert!(!error.contains("machine defect"), "{error}");
}

/// A declaration ANNOTATION does not stop the program.
///
/// A decorator, a `where` constraint and a `meta deny` list annotate a
/// declaration without changing what it declares, and the reference simply
/// runs the program: `class A<T> where T: Object { } 1` answers `1`. Declining
/// refused programs that run. They are not dropped semantics - each governs a
/// surface the backend has no support for either, so a program that DEPENDS on
/// one fails on that surface rather than at the declaration.
#[test]
fn a_declaration_annotation_does_not_stop_the_program() {
    for (source, expected) in [
        ("@sealed() class A { } 1", "1"),
        ("class A<T> where T: Object { } 1", "1"),
        ("class A meta deny instance_state { } 1", "1"),
        ("contract C meta deny method_set { } 1", "1"),
        // The declaration still WORKS rather than being skipped.
        (
            "@sealed() class A { public fun f() -> Integer { 7 } } A.new().f()",
            "7",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a contract may only inherit from a parent that EXISTS, since
    // its requirements have to be there to be inherited - so an unbound parent
    // RAISES rather than silently contributing nothing.
    let agreement = crate::backend::compare_backends(
        "contract Child extends ParentA, ParentB {} 1",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// A targetless transfer, an absent assignment target, and a DEFAULT argument.
///
/// The first two were declines of program errors: `IRIS-V1-CONTROL-C069`
/// gives every transfer a target and `C009` makes a bare `name = expr` never
/// create a binding, so both fail when they RUN. The third was a silent wrong
/// answer - a default was substituted only where the call site could resolve
/// the callee, so a dynamic send answered nil for an unfilled parameter.
#[test]
fn transfers_targets_and_defaults_behave_at_run_time() {
    // A `break` with no enclosing loop escapes as the control signal itself.
    let agreement = crate::backend::compare_backends(
        "break",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("LoopBreak(None, Nil)".to_owned())
    );

    for (source, expected) in [
        // Control: a `break` that HAS a loop still breaks it rather than
        // failing, so the transfer was not broken to report the error.
        (
            "module M { public fun run() -> Object { for x in [1] { break }; :done } } M.run()",
            ":done",
        ),
        // A default fills through a dynamic SEND, which is where it silently
        // answered nil.
        (
            "class A { public fun f(a, b: Integer = 2) { [a, b] } }; A.new().f(1)",
            "[1, 2]",
        ),
        // ...and a supplied argument still WINS over the default.
        (
            "class A { public fun f(a, b: Integer = 2) { [a, b] } }; A.new().f(1, 9)",
            "[1, 9]",
        ),
        (
            "module M { public fun f(a, b: Integer = 2) -> Object { [a, b] } \
             public fun r() -> Object { M.f(1) } } M.r()",
            "[1, 2]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// `IrisValue` validates a stream's HEADER and limits before its payload.
///
/// `IRIS-V1-LIBRARY-C016` checks magic and format version before decoding
/// anything that depends on them, `C017` forbids allocating from a DECLARED
/// length before that length is validated, and `C020` routes a nominal value
/// through the class's own factory - but only for a class that declares
/// `Serializable`, since `C010` refuses to instantiate classes from type names
/// by default. `C003` keeps a live resource out of an encoded stream.
#[test]
fn irisvalue_validates_before_it_decodes() {
    const HEADER: &str = r#"mut s = %{}; s["magic"] = "IRISVALUE"; "#;
    const SERIALIZABLE: &str = "contract Serializable { fun serialize() -> Object } ";

    const CONFORMING: &str = "contract Serializable { fun serialize() -> Object } \
                              class C for Serializable { \
                              public fun serialize() -> Object { [:c, 7] } } ";

    for (source, expected) in [
        // An ordinary value encodes as itself, across the listed families.
        (
            r#"[IrisValue.encode(1), IrisValue.encode("é"), IrisValue.encode(b"\x00\xff"), IrisValue.encode((1, "x"))]"#.to_owned(),
            r#"[1, "é", bytes:00ff, [1, "x"]]"#.to_owned(),
        ),
        // An object that DECLARES `Serializable` is asked for its own
        // representation. Reaching only the built-in surface answered
        // SerializationError for a class that plainly conforms, which manual
        // use found and no single-feature test had.
        (
            "IrisValue.encode(C.new())".to_owned(),
            "[:c, 7]".to_owned(),
        ),
        // A stream naming NO nominal class stays ordinary decoded data.
        (
            format!(r#"{HEADER}s["format_version"] = 1; s["payload"] = 7; IrisValue.decode(s)"#),
            "7".to_owned(),
        ),
    ] {
        let wrapped = format!(
            "{CONFORMING}module M {{ public fun run() -> Object {{ {source} }} }} M.run()"
        );
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected),
            "{source}"
        );
    }

    for (source, expected) in [
        // A version the decoder does not define is refused BEFORE the payload.
        (
            format!(
                "module M {{ public fun run() -> Object {{ {HEADER}s[\"format_version\"] = 2; IrisValue.decode(s) }} }} M.run()"
            ),
            r#"LexicalDiagnostic("IRISVALUE_INCOMPATIBLE_HEADER")"#,
        ),
        // A declared element count over the limit is refused before reading.
        (
            format!(
                "module M {{ public fun run() -> Object {{ {HEADER}s[\"format_version\"] = 1; s[\"element_count\"] = 1025; IrisValue.decode(s) }} }} M.run()"
            ),
            r#"LexicalDiagnostic("IRISVALUE_LIMIT_OR_STRUCTURE")"#,
        ),
        // A SCHEMA version the decoder does not define is refused too, which
        // is a different failure from a limit breach.
        (
            format!(
                "{SERIALIZABLE}class User for Serializable {{ \
                 public fun serialize() -> Object {{ 1 }} \
                 public class fun deserialize(representation) -> Object {{ :rebuilt }} }} \
                 module M {{ public fun run() -> Object {{ {HEADER}s[\"format_version\"] = 1; \
                 s[\"nominal\"] = :User; s[\"schema_version\"] = 2; s[\"payload\"] = 7; \
                 IrisValue.decode(s) }} }} M.run()"
            ),
            r#"LexicalDiagnostic("IRISVALUE_INCOMPATIBLE_HEADER")"#,
        ),
        // A class with a `deserialize` factory but NO declared conformance is
        // refused: otherwise any class could be built from a crafted stream.
        (
            format!(
                "{SERIALIZABLE}class Ghost {{ \
                 public class fun deserialize(representation) -> Object {{ :leaked }} }} \
                 module M {{ public fun run() -> Object {{ {HEADER}s[\"format_version\"] = 1; \
                 s[\"nominal\"] = :Ghost; s[\"schema_version\"] = 1; s[\"payload\"] = 7; \
                 IrisValue.decode(s) }} }} M.run()"
            ),
            "SerializationError",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            &source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error(expected.to_owned()),
            "{source}"
        );
    }
}

/// The FFI boundary refuses BEFORE it crosses, per `IRIS-V1-FFI-C045..C049`.
///
/// `C045` forbids invoking an unbound symbol and `C046` denies any
/// signature-less escape hatch, so a `call` to an unbound name never reaches
/// native code. `C047` lists what a signature must declare and rejects a
/// binding that omits it, and `C049` supports the stable C ABI only. `C043`
/// makes each open an identity-bearing Library, so two opens of one path are
/// two objects.
#[test]
fn the_ffi_boundary_refuses_before_it_crosses() {
    const COMPLETE: &str = "mut sig = %{}; sig[:convention] = :c; sig[:parameters] = []; \
                            sig[:result] = :f64; sig[:errors] = :none; ";

    for (source, expected) in [
        // A complete signature binds, and the binding is observable.
        (
            format!("{COMPLETE}FFI.open(\"libm.so\").bind(:sqrt, sig).bound?(:sqrt)"),
            "true".to_owned(),
        ),
        // A symbol that was never bound records NO signature.
        (
            format!("{COMPLETE}FFI.open(\"libm.so\").bind(:sqrt, sig).signature(:absent)"),
            "nil".to_owned(),
        ),
        // Each open is a distinct identity, even for one path.
        (
            r#"let a = FFI.open("lib"); let b = FFI.open("lib"); [a == b, a.class_name()]"#
                .to_owned(),
            r#"[false, "FFI::Library"]"#.to_owned(),
        ),
        // The refusal is an ordinary CATCHABLE error, not a lost frame.
        (
            "mut rust = %{}; rust[:convention] = :rust; rust[:parameters] = []; \
             rust[:result] = :i32; rust[:errors] = :status; \
             try { FFI.open(\"lib\").bind(:only, rust) } catch e { e }"
                .to_owned(),
            ":IncompleteNativeSignatureError".to_owned(),
        ),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected),
            "{source}"
        );
    }

    for (source, expected) in [
        // Calling an UNBOUND symbol never crosses the boundary.
        (
            r#"FFI.open("libm.so").call(:sqrt)"#.to_owned(),
            "UnboundNativeSymbol",
        ),
        // A signature missing required data is rejected at BIND time.
        (
            "mut sig = %{}; sig[:convention] = :c; FFI.open(\"libm.so\").bind(:sqrt, sig)"
                .to_owned(),
            "IncompleteNativeSignature",
        ),
        // A POINTER parameter needs its nullability and ownership, which a
        // scalar does not - so the obligation is conditional, not blanket.
        (
            "mut p = %{}; p[:type] = :pointer; mut sig = %{}; sig[:convention] = :c; \
             sig[:parameters] = [p]; sig[:result] = :i32; sig[:errors] = :none; \
             FFI.open(\"lib\").bind(:copy, sig)"
                .to_owned(),
            "IncompleteNativeSignature",
        ),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error(expected.to_owned()),
            "{source}"
        );
    }
}

/// Decoding is STRICT by default, and an encoding must be named EXPLICITLY.
///
/// `IRIS-V1-LIBRARY-C022` makes strict handling the default, so a lossy result
/// appears only because the caller asked for it by name. `C025` forbids
/// selecting an OS locale, code page or Host default implicitly: choosing one
/// for decoding requires naming a real Encoding.
#[test]
fn decoding_is_strict_and_named_explicitly() {
    for (source, expected) in [
        (r#"Encoding::UTF_8.decode(b"hi")"#, r#""hi""#),
        // An invalid sequence decodes lossily ONLY when asked by name.
        (
            r#"Encoding::UTF_8.decode(b"\xc3\x28", errors: :replace)"#,
            "\"\u{fffd}(\"",
        ),
        // Latin-1 maps every byte, so it cannot fail and needs no option.
        (r#"Encoding::Latin_1.decode(b"\xff")"#, "\"ÿ\""),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    for (source, expected) in [
        // Strict is the DEFAULT: the same bytes fail without the option.
        (r#"Encoding::UTF_8.decode(b"\xc3\x28")"#, "EncodingError"),
        // Asking for "the default" names no Encoding at all.
        (
            "Encoding.default()",
            r#"LexicalDiagnostic("EncodingSelectionError")"#,
        ),
        // A host default is exactly the implicit selection C025 refuses...
        (
            r#"File.read_text("input.txt", encoding: :host_default)"#,
            r#"LexicalDiagnostic("ENCODING_EXPLICIT_REQUIRED")"#,
        ),
        // ...and so is omitting the encoding entirely.
        (
            r#"File.read_text("input.txt")"#,
            r#"LexicalDiagnostic("ENCODING_EXPLICIT_REQUIRED")"#,
        ),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error(expected.to_owned()),
            "{source}"
        );
    }
}

/// A module composes ANOTHER module, and a class reaches through it.
///
/// A module was previously discovered only from the names of its functions,
/// which misses one that declares no method of its own: `module B mixin A { }`
/// exists solely to compose. Modules are declared explicitly now and defined
/// in DEPENDENCY order, so a composing module names an identity that already
/// exists. `V358` observes that a class composes exactly the modules it named,
/// so an implicit edge would show up as an extra entry.
#[test]
fn a_module_composes_another_module() {
    for (source, expected) in [
        // `C` names only `B`, and reaches `A`'s method through it.
        (
            "module A { public fun w() { :a } } module B mixin A { } class C mixin B { } \
             C.new().w()",
            ":a",
        ),
        // A module's OWN method still outranks the one it composed.
        (
            "module A { public fun trace() { :A } } \
             module B mixin A { public fun trace() { :B } } class C mixin A, B { } \
             C.new().trace()",
            ":B",
        ),
        // The composed edges are observable, and there are exactly two.
        (
            "module A { public fun h() -> Integer { 1 } } module B mixin A { } \
             class C mixin A, B { } \
             module M { public fun run() -> Object { [C.modules.length(), C.new().h()] } } M.run()",
            "[2, 1]",
        ),
        // Control: a module written WITHOUT `mixin` composes nothing, so no
        // implicit edge appears.
        (
            "module A { public fun w() { :a } } class C mixin A { } \
             module M { public fun run() -> Object { C.modules.length() } } M.run()",
            "1",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a FORWARD reference RAISES rather than resolving. The reference
    // refuses it when the program runs, so ordering the definitions to make it
    // work would answer a value the language does not have.
    let agreement = crate::backend::compare_backends(
        "module B mixin A { } module A { public fun w() { :a } } class C mixin B { } \
         C.new().w()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// Reopening a BUILT-IN class adds methods to every value of that class.
///
/// The kernel creates the built-in classes, so a reopen of one has no entry in
/// the compiled class table to attach to: it is recorded by NAME and published
/// onto the kernel's own class at load. An added method must then be reachable
/// through the ordinary send path, which otherwise answers from the native
/// surface and never consults the class.
#[test]
fn reopening_a_builtin_class_adds_to_its_values() {
    for (source, expected) in [
        (
            r#"open class String { public fun shout() -> String { "!" } } "a".shout()"#,
            r#""!""#,
        ),
        // `<` and `>` are DERIVED from `<=>` rather than being separate
        // methods, so a redefined `<=>` has to reach them - otherwise they
        // keep answering from the native comparison the reopen replaced.
        (
            "open class Integer { override public fun <=>(o: Integer) -> Integer { 1 } }; \
             [1 < 2, 1 > 2]",
            "[false, true]",
        ),
        // Control: a reopen adding NOTHING leaves the class as it was.
        ("open class Integer { } 1", "1"),
        // Control: an unrelated built-in family is untouched by the reopen.
        (
            r#"open class String { public fun shout() -> String { "!" } } [1 + 1, "b".shout()]"#,
            r#"[2, "!"]"#,
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a name that is neither DECLARED nor built in has no target at
    // all, so the program RAISES rather than silently creating a class - the
    // reference refuses it when it runs, so raising is what agrees with it.
    let agreement = crate::backend::compare_backends(
        "open class Absent { public fun f() -> Integer { 1 } } 1",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// A `while` answers the operand a `break` carried, in either position.
///
/// `IRIS-V1-CONTROL-C023` gives a normal loop completion no value of its own,
/// so a loop that ends by exhausting its condition answers nil - only a
/// `break` with an operand carries a value out.
#[test]
fn a_loop_answers_what_break_carried() {
    for (source, expected) in [
        (
            "let a = while false { 1 }; let b = while true { break 7 }; [a, b]",
            "[nil, 7]",
        ),
        (
            "module M { public fun run() -> Array { let none = while false { 1 }; \
             let stopped = while true { break :stopped }; [none, stopped] } } M.run()",
            "[nil, :stopped]",
        ),
        // A `for` carries a value out the same way.
        (
            "module M { public fun run() -> Object { for x in [1, 2] { break :early } } } M.run()",
            ":early",
        ),
        // Control: an ordinary loop still RUNS to completion and answers nil,
        // so adding the value register did not change normal termination.
        (
            "module M { public fun run() -> Object { mut n = 0; while n < 3 { n = n + 1 }; n } } \
             M.run()",
            "3",
        ),
        (
            "module M { public fun run() -> Object { while false { 1 } } } M.run()",
            "nil",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// An INDEXED compound assignment evaluates its parts exactly once.
///
/// `IRIS-V1-CONTROL-C036` reads the target once, which for `a[i] += v` means
/// the receiver and the index are each evaluated once and reused for both the
/// read and the write. Lowering it as `a[i] = a[i] + v` would call a receiver
/// expression twice, which is observable whenever it has an effect.
#[test]
fn an_indexed_compound_assignment_evaluates_once() {
    for (source, expected) in [
        ("mut a = [1, 2]; a[0] += 5; a", "[6, [6, 2]]"),
        ("mut a = %{}; a[:k] = 1; a[:k] += 2; a[:k]", "[1, 3, 3]"),
        // The receiver, the index and the right side each run ONCE, in that
        // order - a duplicated read would show as a repeated `:factory`.
        (
            "mut evts = []; mut store = [1, 2]; \
             class P { public fun factory() { evts.append(:factory); store } \
             public fun idx() { evts.append(:index); 0 } \
             public fun rhs() { evts.append(:rhs); 5 } } \
             let p = P.new(); p.factory()[p.idx()] += p.rhs(); evts",
            "[6, [:factory, :index, :rhs]]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A LABELLED break unwinds to the loop that name belongs to.
///
/// It is the only way an inner loop can stop an outer one, and it carries a
/// value out the same way an unlabelled `break` does. A declaration ANNOTATION
/// that names no enclosing loop is a NameError rather than a construct the
/// backend lacks.
#[test]
fn a_labelled_break_unwinds_to_its_loop() {
    for (source, expected) in [
        ("outer: while true { while true { break outer: 7 } }", "7"),
        (
            "module M { public fun run() -> Object { \
             outer: for x in [1, 2] { for y in [3, 4] { break outer: :stopped } } } } M.run()",
            ":stopped",
        ),
        // Control: an UNLABELLED break still stops only the innermost loop,
        // so labelling did not change ordinary unwinding.
        (
            "module M { public fun run() -> Object { mut seen = []; \
             for x in [1, 2] { for y in [3, 4] { seen.append(y); break }; seen.append(x) }; seen } } \
             M.run()",
            "[3, 1, 3, 2]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a label NO enclosing loop carries is a program error rather
    // than a silently ignored transfer.
    let agreement = crate::backend::compare_backends(
        "outer: while true { break outer }",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("NameError".to_owned())
    );
}

/// A match GUARD is tested after the pattern, and a false one falls through.
///
/// A NAME pattern binds the subject for the arm, which is what lets a guard
/// test it - it matches unconditionally, so only the guard can turn the arm
/// down, and the arms after it stay reachable.
#[test]
fn a_match_guard_falls_through_when_false() {
    for (source, expected) in [
        ("match 1 { x if false => :bad; _ => :good }", ":good"),
        ("match 1 { x if true => :good; _ => :bad }", ":good"),
        // The bound name is READABLE in the guard.
        ("match 5 { x if x > 3 => :big; _ => :small }", ":big"),
        ("match 2 { x if x > 3 => :big; _ => :small }", ":small"),
        // Control: an unguarded match still works, so guards did not change
        // ordinary arm selection.
        ("match 2 { 1 => :one; 2 => :two; _ => :other }", ":two"),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A top-level `fun` is a form the reference REFUSES when the program runs,
    // so both backends agree on the refusal rather than merely both refusing.
    let agreement = crate::backend::compare_backends(
        "fun fail() -> Never { nil }",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// A package claim to CORE is rejected at validation, and discards are kept.
///
/// `IRIS-V1-LIBRARY-C028` fixes which surfaces are language core: a separately
/// versioned package must not claim core ABI or replace core literal
/// semantics, so the claim is rejected at VALIDATION time and core behaviour
/// is left untouched. `IRIS-V1-ASYNC-C028` forbids a propagation a `finally`
/// transfer discarded from disappearing silently.
#[test]
fn a_core_claim_is_rejected_and_discards_are_recorded() {
    for (source, expected) in [
        // A package claiming nothing about core validates.
        (
            r#"module M { public fun run() -> Symbol { Package.validate(:"std/plain@1") } } M.run()"#,
            ":validated",
        ),
        // A `finally` that returns out of a raising body still answers, and
        // the discarded contexts are observable rather than lost.
        (
            "module M { public fun run() -> Array { try { raise :pending } \
             finally { return [:override, Diagnostics.discarded_contexts()] } } } M.run()",
            "[:override, []]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: EITHER core claim is refused, and by the same code - the two
    // are not distinguished, because both assert authority over core.
    for source in [
        r#"module M { public fun run() -> Symbol { Package.validate(:"std/http@1", core_abi: true) } } M.run()"#,
        r#"module M { public fun run() -> Symbol { Package.validate(:"std/re@2", replaces_core_regex_literals: true) } } M.run()"#,
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must fail alike: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error(
                r#"LexicalDiagnostic("PACKAGE_CORE_ABI_CLAIM")"#.to_owned()
            ),
            "{source}"
        );
    }
}

/// A `*rest` collects what the caller PASSED, not the frame's spare registers.
///
/// The argument window was sized by the SIGNATURE, so a wider frame's unset
/// registers were read as arguments and `*rest` collected `[nil, nil]` where
/// it should have collected nothing. Manual use found it; the single-feature
/// tests all passed enough arguments to hide the padding.
#[test]
fn a_rest_parameter_collects_only_what_was_passed() {
    for (source, expected) in [
        ("class A { public fun m(*r) { r } } A.new().m()", "[]"),
        ("class A { public fun m(a, *r) { r } } A.new().m(1)", "[]"),
        (
            "class A { public fun m(a, b = 2, *r, key k, **kw, &blk) { r } } A.new().m(1, 9, k: 5)",
            "[]",
        ),
        // Control: a rest that DOES receive arguments still collects them, so
        // the fix did not empty the channel.
        (
            "class A { public fun m(a, *r) { r } } A.new().m(1, 2, 3)",
            "[2, 3]",
        ),
        // Control: the other channels are unaffected by the narrower window.
        (
            "class A { public fun m(a, b = 2, *r, key k, **kw, &blk) { [a, b, k, kw] } } \
             A.new().m(1, k: 5, z: 6)",
            "[1, 2, 5, {:z: 6}]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// `as?` is a CHECKED cast, answering nil when the test does not hold.
///
/// It is the type test with a selection on top rather than a failing cast,
/// which is what distinguishes it from `as`.
#[test]
fn a_checked_cast_answers_nil_on_mismatch() {
    for (source, expected) in [
        (
            r#"let value: Object = "iris"; [value is String, value as? Integer]"#,
            "[true, nil]",
        ),
        ("let v: Object = 1; v as? Integer", "1"),
        // Control: the same value casts to its OWN type, so `as?` is not
        // simply answering nil for everything.
        (r#"let v: Object = "s"; v as? String"#, "\"s\""),
    ] {
        let wrapped = format!("module M {{ public fun run() -> Object {{ {source} }} }} M.run()");
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A `shared let` outside a class is a form the reference REFUSES when the
    // program runs, so both backends agree on the refusal.
    let agreement = crate::backend::compare_backends(
        "shared let @@count: Integer = 1",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// Type PARAMETERS restate a header; a CONTRACT bound changes what runs.
///
/// A reopen carrying its declaration's own type parameters or a `where`
/// constraint restates the header rather than changing it, and a generic
/// contract declares no more requirements than a plain one - so both are
/// annotations. A contract BOUND is not: the reference checks it when the
/// class is constructed, so accepting the construction would answer an object
/// where the language answers a failure.
#[test]
fn a_restated_header_is_an_annotation() {
    for (source, expected) in [
        (
            "class Box<T> { }; open class Box<T> where T: Object { }; 1",
            "1",
        ),
        ("contract Comparable<T> {} 1", "1"),
        // A non-contract bound is satisfied by every construction.
        (
            "class Box<T> where T: Object { public fun tag() -> Symbol { :ok } } \
             Box<String>.new().tag()",
            ":ok",
        ),
        // A reopen restating the header still ADDS its methods.
        (
            "class Box<T> { }; open class Box<T> where T: Object { \
             public fun tag() -> Symbol { :b } }; Box.new().tag()",
            ":b",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a CONTRACT bound is now RAISED at the construction it governs
    // rather than declined at the declaration, so the violating program fails
    // where the reference fails instead of being refused outright.
    let agreement = crate::backend::compare_backends(
        "contract Comparable<T> {} class Box<T> where T: Comparable<T> {} Box<String>.new()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("TypeContractError".to_owned())
    );

    // A reopen's MIXIN composes the module into the class the same way a
    // declaration's does, so the program runs rather than being refused.
    let agreement = crate::backend::compare_backends(
        "module P { public fun h() -> Integer { 7 } } class A { } \
         open class A mixin P { } A.new().h()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("7".to_owned())
    );

    // Control: a reopen that changes what the class IS - a SUPERCLASS - stays
    // declined, so accepting a mixin did not open the whole header.
    let crate::backend::Support::Unsupported(reason) =
        <crate::backend::Bytecode as crate::backend::Backend>::execute(
            &crate::backend::Bytecode,
            "class B { } class A { } open class A extends B { } 1",
        )
    else {
        unreachable!("a reopen changing the superclass must be declined")
    };
    assert_eq!(reason, "class reopen header");
}

/// A class body's `let` or `mut` declares NO instance variable.
///
/// `mut done = false` in a class body is not an ivar initializer: the
/// reference answers nil for `@done` afterwards, so the binding declares
/// nothing the object carries. Declining it refused programs that run.
#[test]
fn a_class_body_binding_declares_no_ivar() {
    for (source, expected) in [
        // The ivar starts ABSENT, whatever the body's binding said.
        (
            "class R { mut done = false public fun read() -> Object { @done } } R.new().read()",
            "nil",
        ),
        (
            "class R { let tag = :t public fun read() -> Object { @tag } } R.new().read()",
            "nil",
        ),
        // Assigning the ivar still works, so only the initializer is inert.
        (
            "class R { mut done = false public fun flip() -> Object { @done = true; @done } } \
             R.new().flip()",
            "true",
        ),
        // The realistic shape: a resource guarding double close.
        (
            "mut n = 0; class R { mut done = false \
             public fun close() -> Nil { if @done { nil } else { @done = true; n = n + 1; nil } } } \
             let r = R.new(); let v = using(r) { :body }; let again = r.close(); [v, n, again]",
            "[:body, 1, nil]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// Reflection INVOKES a Method it already selected, and the meta policy rules.
///
/// `invoke` calls a Method the program obtained through reflection on a
/// receiver it names, so the dispatch that selected the Method is not
/// repeated. A superclass change is refused by the TARGET's meta policy, and a
/// built-in class protects its superclass outright.
#[test]
fn reflection_invokes_and_the_meta_policy_refuses() {
    for (source, expected) in [
        (
            "class A { public fun m() -> Symbol { :a } } \
             module M { public fun run() -> Object { let k = Reflection::Class.method(A, :m); \
             Reflection::Class.invoke(k, A.new(), []) } } M.run()",
            ":a",
        ),
        // An ordinary class ALLOWS the change.
        (
            "class A { } class B extends A { } class Other { } \
             module M { public fun run() -> Object { \
             try { Reflection::Class.set_superclass(B, Other) } catch e { e } } } M.run()",
            "nil",
        ),
        // A BUILT-IN class protects its superclass, and the refusal is the
        // meta policy's rather than a guess.
        (
            "open class Integer { } module M { public fun run() -> Object { \
             try { Reflection::Class.set_superclass(Integer, Object) } catch e { e } } } M.run()",
            ":MetaCapabilityError",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // An OBJECT compares by identity, and `!=` is that negated.
    for (source, expected) in [
        (
            "class A { } module M { public fun run() -> Object { let a = A.new(); \
             [a == a, a != a] } } M.run()",
            "[true, false]",
        ),
        (
            "class A { } module M { public fun run() -> Object { \
             [A.new() == A.new(), A.new() != A.new()] } } M.run()",
            "[false, true]",
        ),
        // `fetch` answers the value it holds...
        (
            "module M { public fun run() -> Object { let h = %{}; h[:a] = 1; \
             h.fetch(:a) } } M.run()",
            "1",
        ),
    ] {
        let wrapped = source.to_owned();
        let agreement = crate::backend::compare_backends(
            &wrapped,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // ...and REFUSES an absent key, where indexing would answer nil. That
    // difference is the whole point of `fetch`.
    let agreement = crate::backend::compare_backends(
        "module M { public fun run() -> Object { let h = %{}; h.fetch(:absent) } } M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must fail alike: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("KeyError".to_owned())
    );
}

/// A module's type parameters annotate it, and a bare `super()` fails at RUN.
///
/// `module Helpers<T> { fun h() { 7 } }` declares the same method either way,
/// and a closed generic mixin names the same module - the backend specialises
/// a module per argument no more than the reference publishes one. A `super()`
/// with no owning class has no ancestor to reach, but the body may never be
/// invoked, so it is raised rather than refused.
#[test]
fn a_generic_module_is_annotated_and_super_fails_late() {
    for (source, expected) in [
        (
            "module Helpers<T> { public fun h() -> Integer { 7 } } \
             class Host mixin Helpers<String> {} Host.new().h()",
            "7",
        ),
        (
            "module Helpers<T> { public fun h() -> Integer { 7 } } 1",
            "1",
        ),
        // A body carrying `super()` COMPILES; only calling it would fail.
        (
            "module M { override public fun m() -> Symbol { super() } } 1",
            "1",
        ),
        // Control: an ordinary mixin still composes, so accepting the closed
        // form did not change the plain one.
        (
            "module Helpers { public fun h() -> Integer { 3 } } class Host mixin Helpers {} \
             Host.new().h()",
            "3",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A `where Self: T` constraint ANNOTATES the module: the reference runs
    // the declaration, so the program fails only on the surface that
    // constraint governs - here, a program of declarations answering no value.
    let agreement = crate::backend::compare_backends(
        "module Helpers<T> where Self: T {} class Host mixin Helpers<_> {}",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// The native ABI raise is a CONVERSION, and a second close does not release.
///
/// `IRIS-V1-FFI-C018` lets native code raise only through an ABI operation
/// that creates an ExceptionContext, and `C017` makes a status alone
/// insufficient - the raised value is read back THROUGH the handle rather than
/// recomputed, so a boundary returning no usable context cannot produce a
/// correct-looking exception. `C020` makes it a conversion rather than a long
/// jump, so a `catch` binds it like any other. `C030` makes a second close a
/// no-op, and the release counter is how a caller proves that.
#[test]
fn the_native_boundary_converts_and_releases_once() {
    for (source, expected) in [
        // The raise crosses the REAL ABI and arrives as an ordinary exception,
        // carrying a context whose value is the marker read back.
        (
            "module M { public fun run() -> Object { \
             try { NativeFixture.raise(41); :unreachable } catch e: Integer, c { [e, c.value] } } } \
             M.run()",
            "[41, 41]",
        ),
        // Closing TWICE releases once, which the counter reports.
        (
            "module M { public fun run() -> Object { let r = NativeFixture.resource(); \
             let first = r.close(); let second = r.close(); [first, second, r.releases] } } M.run()",
            "[nil, nil, 1]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A REOPEN takes effect where it was WRITTEN, on either method side.
///
/// A call made before `open class P { override fun m() }` still answers the
/// original body: publishing every reopen at load made that earlier call
/// answer from the replacement instead. Which body runs is therefore the
/// registry's answer at that moment, not a static "last definition wins".
/// A class method is reached the same way, including as an OPERATOR - `P + P`
/// must find a `class fun +` that `P.+(P)` already found.
#[test]
fn a_reopen_takes_effect_where_it_is_written() {
    for (source, expected) in [
        // The INSTANCE side, before and after the reopen.
        (
            "class P { public fun m() -> Symbol { :old } } let a = P.new().m(); \
             open class P { override public fun m() -> Symbol { :new } } [a, P.new().m()]",
            "[:old, :new]",
        ),
        // The CLASS side, which had no reopen path at all before.
        (
            "class P { class fun m() -> Symbol { :old } } let a = P.m(); \
             open class P { override class fun m() -> Symbol { :new } } [a, P.m()]",
            "[:old, :new]",
        ),
        // An OPERATOR reaches the same singleton method, and respects position.
        (
            "class P { class fun +(other) -> Symbol { :old } } let a = P + P; \
             open class P { override class fun +(other) -> Symbol { :new } } [a, P + P]",
            "[:old, :new]",
        ),
        // Control: with NO reopen the original stands, so position handling
        // did not simply prefer whatever was defined last.
        (
            "class P { class fun +(other) -> Symbol { :only } } [P + P, P.+(P)]",
            "[:only, :only]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A module's own function is reachable BARE from its siblings.
///
/// Inside `module M`, `natural()` names `M.natural` - it has no receiver, so
/// it resolves by index like `M.natural()` rather than as a send to self.
#[test]
fn a_module_function_is_reachable_bare() {
    for (source, expected) in [
        (
            "module M { public fun a() -> Integer { 1 } \
             public fun run() -> Integer { a() + 1 } } M.run()",
            "2",
        ),
        // Control: a LOCAL binding of the same name still wins, so a bare call
        // did not start ignoring the enclosing scope.
        (
            "module M { public fun a() -> Integer { 1 } \
             public fun run() -> Integer { let a = { || 9 }; a.call() } } M.run()",
            "9",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A DECLARED contract cannot be dropped, through either entry point.
///
/// `C119` makes the direct send and the reflective call ONE implementation,
/// so both refuse identically. A contract the class declared is part of its
/// static spine: the refusal leaves `contracts` and the active revision
/// untouched. Removing one it never declared changes no such fact, so that is
/// a no-op rather than a refusal.
#[test]
fn a_declared_contract_cannot_be_removed() {
    for (source, expected) in [
        (
            "contract C { fun m() -> Nil } class A for C { public impl fun m() -> Nil { nil } } \
             let refused = try { A.remove_contract(C) } catch e { e }; \
             [refused, A.contracts, A.active_revision]",
            "[:TypeContractError, [<contract>], 1]",
        ),
        (
            "contract C { fun m() -> Nil } class A for C { public impl fun m() -> Nil { nil } } \
             try { Reflection::Class.remove_contract(A, C) } catch e { e }",
            ":TypeContractError",
        ),
        // Control: an UNDECLARED contract is a no-op, so the refusal is about
        // the spine rather than about the selector being rejected outright.
        (
            "contract C { fun m() -> Nil } contract D { fun n() -> Nil } \
             class A for C { public impl fun m() -> Nil { nil } } A.remove_contract(D)",
            "nil",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A correctly synchronized observer SEES a concurrent write.
///
/// `C060` makes the write visible because the writer thread is JOINED before
/// the read: the observer reads the replacement, not the text the value was
/// built with, and its length is the replacement's.
#[test]
fn a_joined_writer_is_observed() {
    let source = "module M { public fun run() -> Object { let text = m\"ee\"; \
                  NativeFixture.concurrently_replace(text, \"uoa\"); \
                  [text.to_string(), text.length] } } M.run()";
    let agreement = crate::backend::compare_backends(
        source,
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[\"uoa\", 3]".to_owned())
    );

    // Control: with NO replacement the original content stands, so the read
    // reports the value's own text rather than always the last write.
    let agreement = crate::backend::compare_backends(
        "module M { public fun run() -> Object { let text = m\"ee\"; \
         [text.to_string(), text.length] } } M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[\"ee\", 2]".to_owned())
    );
}

/// A contract INHERITS its parents' requirements, and `open` only annotates.
///
/// `open` governs whether the contract may be reopened, which is a separate
/// surface - the requirement set is the same either way. `extends` is not an
/// annotation: a child carries its parents' requirements as well as its own,
/// so a class implementing the child must satisfy what the parent required.
#[test]
fn a_contract_inherits_its_parents_requirements() {
    for (source, expected) in [
        // The parent's requirement is satisfied THROUGH the child.
        (
            "contract PA { fun a() -> Nil } contract Child extends PA {} \
             class A for Child { public impl fun a() -> Nil { nil } } A.new().a()",
            "nil",
        ),
        // `open` changes no requirement, so the same program still runs.
        (
            "open contract C { fun m() -> Nil } \
             class A for C { public impl fun m() -> Nil { nil } } A.new().m()",
            "nil",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A child inherits from EVERY parent it names, so both requirements are
    // carried rather than only the first.
    let agreement = crate::backend::compare_backends(
        "contract PA { fun a() -> Symbol } contract PB { fun b() -> Symbol } \
         contract Child extends PA, PB {} \
         class A for Child { public impl fun a() -> Symbol { :ay } \
         public impl fun b() -> Symbol { :bee } } let x = A.new(); [x.a(), x.b()]",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[:ay, :bee]".to_owned())
    );

    // Control: a parent must EXIST for its requirements to be inherited, so an
    // unbound one RAISES rather than contributing nothing silently. The
    // reference refuses it when the program runs, so raising is what agrees.
    let agreement = crate::backend::compare_backends(
        "contract Child extends Missing {} 1",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );
}

/// A declared RETURN Type is guarded before the value reaches the caller.
///
/// `IRIS-V1-TYPES-C004` guards the return boundary whether the body fell off
/// its end or returned explicitly, so a method annotated `-> Nil` cannot
/// answer a Symbol. Without this the backend RAN a program the reference
/// refuses, which is a wrong answer rather than a missing feature. An
/// annotation the backend cannot decide stays permissive, so the check catches
/// a definite mismatch instead of narrowing the accepted surface.
#[test]
fn a_declared_return_type_is_guarded() {
    for (source, expected) in [
        // Falling off the end with a value the annotation excludes.
        (
            "class A { public fun m() -> Nil { :done } } A.new().m()",
            None,
        ),
        (
            "class A { public fun m() -> Integer { :done } } A.new().m()",
            None,
        ),
        // The EXPLICIT return path is guarded the same way.
        (
            "class A { public fun m() -> Integer { return :bad } } A.new().m()",
            None,
        ),
        // A module function is not a special case.
        (
            "module M { public fun m() -> Integer { :bad } } M.m()",
            None,
        ),
        // Controls: a body that SATISFIES its annotation still answers, so the
        // guard rejects a mismatch rather than every annotated return.
        (
            "class A { public fun m() -> Nil { nil } } A.new().m()",
            Some("nil"),
        ),
        (
            "class A { public fun m() -> Integer { 7 } } A.new().m()",
            Some("7"),
        ),
        // `Object` admits every value, so an annotation that decides nothing
        // narrows nothing.
        (
            "class A { public fun m() -> Object { :ok } } A.new().m()",
            Some(":ok"),
        ),
        // A UNION admits a value satisfying any constituent.
        (
            "class A { public fun m() -> Integer | Symbol { :ok } } A.new().m()",
            Some(":ok"),
        ),
        // The guard RAISES, so a caller can catch it like any other error.
        (
            "class A { public fun m() -> Nil { :bad } } \
             module M { public fun run() -> Object { try { A.new().m() } catch e { e } } } M.run()",
            Some(":TypeContractError"),
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("TypeContractError".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }
}

/// A contract BOUND is decided at the construction it governs.
///
/// `IRIS-V1-TYPES-C067` checks a `where T: SomeContract` bound at
/// MATERIALIZATION, not where the class is declared - so the declaration
/// alone runs, and `Box<String>.new()` is the failure because String declares
/// no such contract. Declining the declaration refused a program the reference
/// runs; accepting the construction would answer an object where the language
/// answers a failure.
#[test]
fn a_contract_bound_is_decided_at_construction() {
    for (source, expected) in [
        // The DECLARATION alone changes nothing, so the program still answers.
        (
            "contract Comparable<T> {} class Box<T> where T: Comparable<T> {} 1",
            Some("1"),
        ),
        // A built-in argument declares no contract, so this is a violation.
        (
            "contract Comparable<T> {} class Box<T> where T: Comparable<T> {} Box<String>.new()",
            None,
        ),
        // Control: an argument that DOES declare the bound contract passes, so
        // the check rejects a violation rather than every bounded construction.
        (
            "contract Comparable<T> {} class Key for Comparable {} \
             class Box<T> where T: Comparable<T> {} Box<Key>.new()",
            Some("<object>"),
        ),
        // Control: with NO bound the construction is untouched.
        ("class Box<T> {} Box<String>.new()", Some("<object>")),
        // The failure RAISES, so a caller catches it like any other error.
        (
            "contract Comparable<T> {} class Box<T> where T: Comparable<T> {} \
             module M { public fun run() -> Object { try { Box<String>.new() } catch e { e } } } \
             M.run()",
            Some(":TypeContractError"),
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("TypeContractError".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }
}

/// A loop binding may DESTRUCTURE each item, and a mismatch raises.
///
/// `IRIS-V1-CONTROL-C045` binds `for [a, b] in source` from each yielded
/// Array, and raises `PatternMatchError` when the item is not an Array of
/// exactly that arity. The arity is carried into the binding rather than the
/// element being read with a plain index, because an index would answer nil
/// for a missing position instead of failing.
#[test]
fn a_loop_binding_destructures_each_item() {
    for (source, expected) in [
        // Exact arity binds both names.
        ("for [a, b] in [[1, 2]] { a + b }", Some("nil")),
        // Too FEW names for the item, and too MANY, both fail.
        ("for [a, b] in [[1, 2, 3]] { a }", None),
        ("for [a, b, c] in [[1, 2]] { a }", None),
        // An item that is not an Array at all cannot be destructured.
        ("for [a, b] in [7] { a }", None),
        // Control: a plain NAME binding takes the item itself, so adding
        // destructuring did not change the ordinary form.
        ("for x in [[1, 2]] { x }", Some("nil")),
        // Each iteration rebinds, so both names carry that item's elements.
        (
            "mut t = 0; for [a, b] in [[1, 2], [3, 4]] { t = t + a * b }; t",
            Some("[nil, 14]"),
        ),
        // The failure RAISES, so an enclosing `try` catches it - returning it
        // directly would escape the handler and make it uncatchable.
        (
            "module M { public fun run() -> Object { \
             try { for [a, b] in [[1, 2, 3]] { a } } catch e { e } } } M.run()",
            Some(":PatternMatchError"),
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("PatternMatchError".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }

    // Control: a NESTED sub-pattern decides more than an arity check can
    // express, so it stays declined rather than being approximated.
    let crate::backend::Support::Unsupported(reason) =
        <crate::backend::Bytecode as crate::backend::Backend>::execute(
            &crate::backend::Bytecode,
            "for [a, [b]] in [[1, [2]]] { a }",
        )
    else {
        unreachable!("a nested destructuring sub-pattern must be declined")
    };
    assert_eq!(reason, "statement for");
}

/// A declaration naming a target that does not EXIST raises when it runs.
///
/// A reopen of an undeclared class and a contract inheriting an undeclared
/// parent are program errors the reference raises at RUN time, exactly as a
/// parse rejection is. Declining made both backends refuse the same program
/// while describing it differently, which holds the row rather than agreeing.
#[test]
fn an_absent_declaration_target_raises() {
    for source in [
        "contract Child extends ParentA, ParentB {}",
        "contract Child extends ParentA, ParentB {} 1",
        "open class Box<String> { fun m() -> Nil {} }",
        "class Present {} open class NotThere { public fun m() -> Nil { nil } } 1",
        "class Present {} open class NotThere { public fun m() -> Nil { nil } } NotThere.new()",
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Error("UnsupportedConstruct".to_owned()),
            "{source}"
        );
    }

    // Control: a reopen whose target DOES exist still applies, so raising is
    // about the missing target rather than about reopening at all.
    let agreement = crate::backend::compare_backends(
        "class P { public fun m() -> Symbol { :old } } \
         open class P { override public fun m() -> Symbol { :new } } P.new().m()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value(":new".to_owned())
    );
}

/// A module's `shared class property` is READ as a member.
///
/// `M.first` answers it, unlike a `const`, which is visible only lexically
/// inside the module's own methods. It is module state with no receiver, so it
/// is synthesized into a receiverless reader and resolved by index rather than
/// dispatched - there is no module receiver value to send to.
#[test]
fn a_module_property_is_read_as_a_member() {
    for (source, expected) in [
        (
            "module M { shared class property first: Integer = 1 } M.first",
            "1",
        ),
        (
            "module M { shared class property first: Integer = 1 \
             shared class property second: Integer = 2 } [M.first, M.second]",
            "[1, 2]",
        ),
        // The property is reachable from the module's OWN methods too.
        (
            "module M { shared class property base: Integer = 10 \
             public fun scaled() -> Integer { M.base * 3 } } [M.base, M.scaled()]",
            "[10, 30]",
        ),
        // A module body's ordinary statements still run at the declaration's
        // source position, so a raise between two properties propagates.
        (
            "module M { shared class property first: Integer = 1 raise :stop \
             shared class property second: Integer = 2 } M",
            "Raised(Symbol(\"stop\"))",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = if expected.starts_with("Raised") {
            crate::backend::Observation::Error(expected.to_owned())
        } else {
            crate::backend::Observation::Value(expected.to_owned())
        };
        assert_eq!(observation, &wanted, "{source}");
    }

    // Control: a `const` is NOT a member, so reading it that way still fails -
    // the reader synthesis applies to a property rather than to every name a
    // module body binds.
    let agreement = crate::backend::compare_backends(
        "module M { const first = 1 public fun get() -> Integer { first } } M.get()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("1".to_owned())
    );
}

/// `from S import K` binds the module's CONSTANT under the imported name.
///
/// The binding happens at the import's own source position, and only a
/// constant is bound: a module's methods are reached as `S.f()` rather than by
/// name. An imported name does not disturb the module's own lexical scope -
/// `M.declared()` still reads `M`'s `K`, and a local `let K` still shadows it.
#[test]
fn an_import_binds_a_module_constant() {
    for (source, expected) in [
        ("module S { const K = 5 } from S import K; K", "5"),
        // An ALIAS binds under the written name instead.
        ("module S { const K = 5 } from S import K as J; J", "5"),
        // Three scopes stay distinct: a local `let`, the importing module's
        // own constant, and the imported one.
        (
            "module S { const K = 5 } \
             module M { const K = 1 public module fun lexical() { let K = 9; K } \
             public module fun declared() { K } } \
             from S import K; [M.lexical(), M.declared(), K]",
            "[9, 1, 5]",
        ),
        // A spec naming nothing the module declares binds no name, and the
        // program still RUNS - so it is a no-op rather than a refusal.
        ("module S { const K = 5 } from S import Missing; 1", "1"),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A left side that names no assignable place RAISES when the assignment runs.
///
/// The reference refuses it at run time rather than statically, so declining
/// made both backends refuse the same program while describing it differently
/// - which holds the row rather than agreeing.
#[test]
fn an_unassignable_target_raises() {
    let agreement = crate::backend::compare_backends(
        "a.b(c)[d] ** -e * f + g << h & i ^ j | k ..< l < m == n named o && p || q = r",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("UnsupportedConstruct".to_owned())
    );

    // Control: an ordinary NAME target still assigns, so raising is about the
    // unassignable shape rather than about assignment itself.
    let agreement = crate::backend::compare_backends(
        "mut x = 1; x = 2; x",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[2, 2]".to_owned())
    );
}

/// A module REOPEN adds to the module it names, and the LAST definition wins.
///
/// `open module M { fun b() }` publishes into the same owner rather than
/// declaring a new module, so `M.a()` and `M.b()` both resolve. A republished
/// selector wins because resolution takes the last definition - searching
/// forwards answered from the body the reopen replaced.
#[test]
fn a_module_reopen_adds_to_its_target() {
    for (source, expected) in [
        (
            "module M { public fun a() -> Integer { 1 } } \
             open module M { public fun b() -> Integer { 2 } } [M.a(), M.b()]",
            "[1, 2]",
        ),
        (
            "module M { public fun a() -> Integer { 1 } } \
             open module M { override public fun a() -> Integer { 2 } } M.a()",
            "2",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a reopen whose target does not EXIST adds to nothing, and the
    // program still RUNS - so its methods are dropped rather than a new module
    // being declared or the program refused.
    let agreement = crate::backend::compare_backends(
        "open module Absent { public fun b() -> Integer { 2 } } 1",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("1".to_owned())
    );
}

/// A GENERIC module mixin names the same module whatever its argument.
///
/// `mixin Helpers<_>` composes `Helpers` exactly as `mixin Helpers<String>`
/// does, since the backend specialises a module per argument no more than the
/// reference publishes one - so a wildcard reaches the same methods. A
/// `where Self: T` constraint annotates the module the same way.
#[test]
fn a_wildcard_module_mixin_composes_the_module() {
    for (source, expected) in [
        (
            "module Helpers<T> { public fun h() -> Integer { 7 } } \
             class Host mixin Helpers<_> {} Host.new().h()",
            "7",
        ),
        (
            "module Helpers<T> where Self: T { public fun h() -> Integer { 7 } } \
             class Host mixin Helpers<_> {} Host.new().h()",
            "7",
        ),
        ("module Helpers<T> where Self: T {} 1", "1"),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // A PRIVATE-access class mixin grants the module reach into the class's
    // private methods, so the marker travels into the composition EDGE rather
    // than being refused - the module's method reaches the composing object's
    // own private one.
    for (source, expected) in [
        (
            "module M { public fun h() -> Integer { 7 } } class A mixin M private {} A.new().h()",
            "7",
        ),
        (
            "module M { public fun reach() -> Object { try { secret() } catch e { e } } } \
             class G mixin M private { private fun secret() -> Symbol { :g } } G.new().reach()",
            ":g",
        ),
        // A module method mixed into a class binds `self` to the COMPOSING
        // object, so it reaches that object's own methods - and an argument
        // lands in its own parameter rather than being displaced by the
        // receiver.
        (
            "module M { public fun reach() -> Object { self.own() } } \
             class G mixin M { public fun own() -> Symbol { :g } } G.new().reach()",
            ":g",
        ),
        (
            "module M { public fun reach(x) -> Object { x } } class G mixin M { } G.new().reach(9)",
            "9",
        ),
        // Control: a module never composed keeps the receiverless form, where
        // `M.f()` passes only its arguments.
        (
            "module M { public fun reach() -> Object { 7 } } [M.reach(), 1]",
            "[7, 1]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A composed member may NOT contradict a declared contract requirement.
///
/// `D-173` puts the contract-visible SIGNATURE in the static spine, so a
/// method reached through a MIXIN whose parameter Type differs from the
/// requirement is an incompatible replacement rather than a satisfying one.
/// An unannotated position states nothing and is left alone.
#[test]
fn a_composed_member_may_not_contradict_a_requirement() {
    for (source, expected) in [
        // Reached through a REOPEN's mixin.
        (
            "contract C { fun draw(n: Integer) -> Nil } \
             module P { public fun draw(s: String) -> Nil { nil } } \
             class A for C { } open class A mixin P { } A",
            None,
        ),
        // The same clash written at the DECLARATION.
        (
            "contract C { fun draw(n: Integer) -> Nil } \
             module P { public fun draw(s: String) -> Nil { nil } } \
             class A for C mixin P { } A",
            None,
        ),
        // Control: a MATCHING signature satisfies the requirement.
        (
            "contract C { fun draw(n: Integer) -> Nil } \
             module P { public fun draw(n: Integer) -> Nil { nil } } \
             class A for C { } open class A mixin P { } A",
            Some("<class>"),
        ),
        // Control: with no contract there is nothing to contradict.
        (
            "module P { public fun draw(s: String) -> Nil { nil } } \
             class A { } open class A mixin P { } A",
            Some("<class>"),
        ),
        // An UNANNOTATED position states nothing, so it is not a mismatch.
        (
            "contract C { fun draw(n: Integer) -> Nil } \
             module P { public fun draw(x) -> Nil { nil } } \
             class A for C { } open class A mixin P { } A.new().draw(1)",
            Some("nil"),
        ),
        // A reopen may compose a module AND republish a method at once.
        (
            "module P { public fun h() -> Integer { 7 } } \
             class A { public fun m() -> Symbol { :old } } \
             open class A mixin P { override public fun m() -> Symbol { :new } } \
             let a = A.new(); [a.h(), a.m()]",
            Some("[7, :new]"),
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("TypeContractError".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }
}

/// A reopen may DECLARE a conformance, on a declared or a built-in class.
///
/// The conformance is observable through `A.contracts`, so it joins the
/// class's own list rather than being ignored. A BUILT-IN class is the
/// kernel's and has no entry to join, so its conformance is recorded on the
/// reopen - which is what makes `1 as N` a legitimate view. A contract view is
/// the value seen THROUGH a contract, so an operator applies to that value.
#[test]
fn a_reopen_may_declare_a_conformance() {
    for (source, expected) in [
        (
            "contract N { fun m() -> Integer } class A { public fun m() -> Integer { 1 } } \
             open class A for N { } A.new().m()",
            "1",
        ),
        // The conformance is OBSERVABLE, so it was recorded rather than dropped.
        (
            "contract N { fun m() -> Integer } class A { public fun m() -> Integer { 1 } } \
             open class A for N { } A.contracts",
            "[<contract>]",
        ),
        // A BUILT-IN class conforms through its reopen, and the view compares
        // as the Integer it wraps.
        (
            "contract N { fun m() -> Integer } \
             open class Integer for N { public impl fun m() -> Integer { 1 } } \
             (1 as N) == (1 as N)",
            "true",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: a class that never declared the contract cannot be viewed
    // through it, so recording a reopen's conformance did not make every cast
    // succeed.
    let agreement = crate::backend::compare_backends(
        "contract N { fun m() -> Integer } class A { public fun m() -> Integer { 1 } } \
         module M { public fun run() -> Object { try { (A.new() as N)..m() } catch e { e } } } \
         M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("Runtime(Type)".to_owned())
    );
}

/// A subclass INHERITS its superclass's conformances.
///
/// An `impl` marker on `class A extends B` names a requirement the ancestry
/// declares even when `A`'s own header does not, and `A` is viewable through
/// `B`'s contract - so the ancestry is walked rather than only the class's own
/// list. A CONTRACT is itself a value, so `C.hash()` sends to the contract,
/// and it is interned once per definition, which is why two reads hash alike.
#[test]
fn a_subclass_inherits_its_conformances() {
    for (source, expected) in [
        (
            "contract C { fun n() -> Symbol } \
             class B for C { public impl fun n() -> Symbol { :base } } \
             class A extends B {} (A.new() as C)..n()",
            ":base",
        ),
        // An `impl` on the SUBCLASS names the inherited requirement, and
        // `super()` still reaches the ancestor's body.
        (
            "contract C { fun n() -> Symbol } \
             class B for C { public impl fun n() -> Symbol { :base } } \
             class A extends B { public override impl fun n() -> Symbol { super() } } \
             module M { public fun run() -> Object { let a = A.new(); \
             [(a as C)..n(), a.n(), (a as C) == (a as C), C.hash() == C.hash()] } } M.run()",
            "[:base, :base, true, true]",
        ),
        (
            "contract C { fun n() -> Symbol } C.hash() == C.hash()",
            "true",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: an `impl` naming a requirement NO ancestor declares is still
    // undeclared, so walking the ancestry did not accept every marker.
    let crate::backend::Support::Unsupported(reason) =
        <crate::backend::Bytecode as crate::backend::Backend>::execute(
            &crate::backend::Bytecode,
            "contract C { fun n() -> Symbol } class B { } \
             class A extends B { public impl fun n() -> Symbol { :x } } 1",
        )
    else {
        unreachable!("an impl with no declared requirement must be declined")
    };
    assert_eq!(reason, "contract implementation undeclared");
}

/// A QUALIFIED `impl fun C::m()` is visible only through that contract's view.
///
/// `(a as C)..m()` answers it while `a.m()` answers the class's own method, so
/// it is recorded per contract rather than published onto the class. It
/// supplies the member itself, which is why the view answers it even when the
/// contract declares no matching requirement.
#[test]
fn a_qualified_implementation_belongs_to_the_view() {
    for (source, expected) in [
        (
            "contract C { } class A for C { impl fun C::m() { :qualified } \
             public fun m() { :ordinary } } let a = A.new(); [(a as C)..m(), a.m()]",
            "[:qualified, :ordinary]",
        ),
        // TWO contracts each get their OWN body, so a qualified impl is
        // matched to its own method rather than to the first of that name.
        (
            "contract C { } contract D { } \
             class A for C, D { impl fun C::m() { :cee } impl fun D::m() { :dee } \
             public fun m() { :own } } \
             let a = A.new(); [(a as C)..m(), (a as D)..m(), a.m()]",
            "[:cee, :dee, :own]",
        ),
        // Control: an ORDINARY `impl` publishes onto the class, so both the
        // view and the receiver answer the same body.
        (
            "contract C { fun m() -> Symbol } \
             class A for C { public impl fun m() -> Symbol { :only } } \
             let a = A.new(); [(a as C)..m(), a.m()]",
            "[:only, :only]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A class body's ordinary STATEMENTS run with `self` bound to the class.
///
/// They run at the declaration's own source position, which is what lets
/// `class A { if true { self.define_method(:x) { .. } } }` publish a method.
/// They declare nothing, so they contribute no signature and their values are
/// discarded.
#[test]
fn a_class_body_runs_its_statements() {
    for (source, expected) in [
        (
            "class A { if true { self.define_method(:x) { 7 } } }; A.new().x()",
            "7",
        ),
        ("class A { self.define_method(:x) { 7 } }; A.new().x()", "7"),
        // The statement runs at the declaration's SOURCE POSITION, so a name
        // bound before it is in scope.
        (
            "mut log = []; class A { log.append(:body) }; log",
            "[:body]",
        ),
        // A guard that does NOT hold defines nothing, so the body really ran
        // rather than being published unconditionally.
        (
            "class A { if false { self.define_method(:x) { 7 } } \
             public fun y() -> Integer { 1 } }; A.new().y()",
            "1",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }

    // Control: BEFORE that position the name is unbound, so the body is not
    // hoisted to run ahead of the program's own statements.
    let agreement = crate::backend::compare_backends(
        "class A { log.append(:body) }; mut log = []; log",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("NameError".to_owned())
    );
}

/// A contract may extend a GENERIC parent, or a kernel one.
///
/// A generic parent names the same contract as a bare one, since the backend
/// interns one contract per definition rather than per construction. `Iterable`
/// and `Iterator` are the KERNEL's rather than program declarations, so a child
/// extending one inherits nothing this backend records and the declaration
/// still runs.
#[test]
fn a_contract_may_extend_a_generic_parent() {
    for (source, expected) in [
        (
            "contract Numbers extends Iterable<Integer> { fun iterator() -> Iterator<Integer> } 1",
            Some("1"),
        ),
        (
            "contract P<T> { fun p() -> Nil } contract Numbers extends P<Integer> { fun m() -> Nil } \
             class A for Numbers { public impl fun m() -> Nil { nil } \
             public impl fun p() -> Nil { nil } } A.new().m()",
            Some("nil"),
        ),
        // Control: a parent that is neither declared NOR a kernel contract is
        // still absent, so it raises rather than being waved through.
        (
            "contract Numbers extends Missing<Integer> { fun m() -> Nil } 1",
            None,
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("UnsupportedConstruct".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }
}

/// `Reflection::Class.define_method` names its TARGET as the first argument.
///
/// `Reflection::Class.define_method(K, :m) { .. }` and
/// `self.define_method(:m) { .. }` publish the same way, so the reflective
/// form is rewritten to the direct one. A dynamic method's parameters come
/// from the block it was defined with, so a call supplying a different count
/// has no binding for them and answers ArgumentError rather than filling nil.
#[test]
fn a_reflective_define_method_names_its_target() {
    for (source, expected) in [
        (
            "class K { } Reflection::Class.define_method(K, :m) { 7 }; K.new().m()",
            Some("[nil, 7]"),
        ),
        // The published body REPLACES the class's own method.
        (
            "class K { public fun m(a) -> Symbol { :old } } \
             Reflection::Class.define_method(K, :m) { |o| :new }; K.new().m(1)",
            Some("[nil, :new]"),
        ),
        // A call whose arity does not match the block is an ArgumentError.
        (
            "class K { public fun m() -> Symbol { :old } } \
             Reflection::Class.define_method(K, :m) { |o| :new }; K.new().m()",
            None,
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        let wanted = match expected {
            Some(value) => crate::backend::Observation::Value(value.to_owned()),
            None => crate::backend::Observation::Error("ArgumentError".to_owned()),
        };
        assert_eq!(observation, &wanted, "{source}");
    }

    // The arity failure RAISES, so an enclosing `try` catches it by name.
    let agreement = crate::backend::compare_backends(
        "class K { public fun m() -> Symbol { :old } } \
         module M { public fun run() -> Object { \
         Reflection::Class.define_method(K, :m) { |o| :new }; \
         try { K.new().m() } catch e { e } } } M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value(":ArgumentError".to_owned())
    );

    // Control: the DIRECT form still publishes, so rewriting the reflective
    // one did not disturb it.
    let agreement = crate::backend::compare_backends(
        "class A { self.define_method(:x) { 7 } }; A.new().x()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("7".to_owned())
    );
}

/// A published spine is not REWOUND, and shutdown closes delivery.
///
/// `Reflection::Class.reactivate` names a revision that is no longer active,
/// so it is refused rather than performed - the target is resolved first, so
/// the refusal cannot be mistaken for an unknown class. `C050` names shutdown
/// as the condition under which a flush reports INCOMPLETE with the
/// accepted-but-undelivered count.
#[test]
fn a_revision_is_neither_rewound_nor_delivered_after_shutdown() {
    for (source, expected) in [
        (
            "class A { } try { Reflection::Class.reactivate(A, 1) } catch e { e }",
            ":MetaTransactionError",
        ),
        (
            "class B { } module M { public fun run() -> Tuple { \
             let subscriber = { |event| :seen }; Revision.subscribe(subscriber); \
             B.open() { |t| 1 }; Revision.shutdown(); Revision.flush() } } M.run()",
            "[:incomplete, [], 1, []]",
        ),
        // Control: WITHOUT a shutdown the same commit is delivered, so the
        // incomplete report is about the closure rather than about flushing.
        (
            "class B { } module M { public fun run() -> Tuple { \
             let subscriber = { |event| :seen }; Revision.subscribe(subscriber); \
             B.open() { |t| 1 }; Revision.flush() } } M.run()",
            "[:delivered, [[:RevisionEvent, 1, [:B]]], 0, []]",
        ),
    ] {
        let agreement = crate::backend::compare_backends(
            source,
            &[&crate::backend::Interpreter, &crate::backend::Bytecode],
        );
        let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must agree: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            &crate::backend::Observation::Value(expected.to_owned()),
            "{source}"
        );
    }
}

/// A configured audit SINK survives a prune, per `IRIS-V1-ASYNC-C055`.
///
/// The sink is a separate persistence layer, so `recover` answers what it
/// holds independently of what retained history still has - the in-memory
/// queue offers no zero-loss guarantee, which is C055's point. With no sink
/// configured there is nothing to recover from.
#[test]
fn an_audit_sink_survives_a_prune() {
    let agreement = crate::backend::compare_backends(
        "class B {} module M { public fun run() -> Object { \
         RevisionHistory.configure_sink(); B.open() { |t| 1 }; RevisionHistory.prune(1); \
         [try { RevisionHistory.events(1, 1) } catch e { e }, RevisionHistory.recover(1, 1)] } } \
         M.run()",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Value("[:AuditHistoryUnavailableError, [1]]".to_owned())
    );

    // Control: with NO sink there is no zero-loss guarantee, so recovery
    // fails rather than reading the retained queue.
    let agreement = crate::backend::compare_backends(
        "class B {} RevisionHistory.recover(1, 1)",
        &[&crate::backend::Interpreter, &crate::backend::Bytecode],
    );
    let crate::backend::Agreement::Agreed { observation, .. } = &agreement else {
        unreachable!("both backends must agree: {agreement:?}")
    };
    assert_eq!(
        observation,
        &crate::backend::Observation::Error("AuditHistoryUnavailable".to_owned())
    );
}
