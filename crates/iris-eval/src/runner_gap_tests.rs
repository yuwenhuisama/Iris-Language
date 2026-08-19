use iris_runtime::{
    ArrayRef, BuiltinClass, ClassId, Kernel, KernelError, MethodBody, NativeSelector, Runtime,
    Selector, StaticSpine, Value as RuntimeValue, Visibility,
};

use super::{EvaluationError, evaluate};

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
