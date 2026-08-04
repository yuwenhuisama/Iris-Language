use iris_runtime::{
    BuiltinClass, ClassId, Kernel, KernelError, MethodBody, NativeSelector, Runtime, Selector,
    StaticSpine, Value as RuntimeValue, Visibility,
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("local".into()),
            RuntimeValue::Symbol("base_g".into()),
            RuntimeValue::Symbol("missing".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Array(vec![RuntimeValue::Symbol("@x".into())]),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Nil,
        ]))
    );
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("module".into()),
            RuntimeValue::Symbol("module".into()),
            RuntimeValue::Nil,
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            // These ids are ALLOCATION ORDER, not identities the spec fixes:
            // declared Classes are numbered after the builtin ones, so adding a
            // builtin Class shifts every declared id. What the row asserts is
            // the ANCESTOR RELATIONSHIP, `B` then its current superclass then
            // `Object`, which the ids below spell out positionally.
            RuntimeValue::Array(vec![
                RuntimeValue::Class(ClassId::new(8)),
                RuntimeValue::Class(ClassId::new(9)),
                RuntimeValue::Class(ClassId::new(0))
            ]),
            RuntimeValue::Nil,
            RuntimeValue::Array(vec![
                RuntimeValue::Class(ClassId::new(8)),
                RuntimeValue::Class(ClassId::new(7)),
                RuntimeValue::Class(ClassId::new(0))
            ]),
        ]))
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
    assert_eq!(reflection_result, Ok(RuntimeValue::Array(vec![])));
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Array(vec![RuntimeValue::Symbol("@x".into())]),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Symbol("value".into()),
            RuntimeValue::Symbol("value".into()),
        ]))
    );
    assert_eq!(
        class_result,
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Nil,
        ]))
    );
    assert_eq!(
        module_result,
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("m".into()),
            RuntimeValue::Symbol("m".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
            RuntimeValue::Bool(true),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("a".into()),
            RuntimeValue::Symbol("b".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            RuntimeValue::Array(vec![
                RuntimeValue::Symbol("a".into()),
                RuntimeValue::Symbol("b".into()),
            ]),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("handled".into()),
            RuntimeValue::Array(vec![
                RuntimeValue::Symbol("try".into()),
                RuntimeValue::Symbol("catch".into()),
                RuntimeValue::Symbol("finally".into()),
            ]),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("yes".into()),
            RuntimeValue::Symbol("no".into()),
            RuntimeValue::Nil,
            RuntimeValue::Symbol("second".into()),
        ]))
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
            Ok(RuntimeValue::Array(vec![RuntimeValue::Symbol("y".into())])),
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Symbol("y".into()),
            ])),
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
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Symbol("no".into()),
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Symbol("no".into()),
            ])),
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("selected".into()),
            RuntimeValue::Integer(0_u8.into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Bool(true),
            RuntimeValue::Integer(2_u8.into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Bool(true),
            RuntimeValue::Symbol("old".into()),
            RuntimeValue::Symbol("new".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("old".into()),
            RuntimeValue::Symbol("added".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("get".into()),
            RuntimeValue::Symbol("set".into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Array(vec![RuntimeValue::Integer(2_u8.into())]),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Float64(2.0),
            RuntimeValue::Float64(3.0),
            RuntimeValue::Array(vec![RuntimeValue::Float64(3.0)]),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            RuntimeValue::Nil
        ]))
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
    // Given
    let source = "class C mixin A { public fun t() -> Integer { 1 } }; C.new().t()";

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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Integer(2_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Symbol("TypeContractError".into()),
            RuntimeValue::Bool(true),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Integer(2_u8.into()),
        ]))
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
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("a".into()),
        ]))
    );

    // D-175 recomputes MRO before commit and one Module holds one position in
    // it, so including a Module already composed is a no-op. Appending a
    // duplicate edge let a redundant `add_module` silently reorder dispatch.
    let redundant = format!("{base} H.open() {{ |t| t.add_module(:A) }}; H.new().tag()");
    assert_eq!(
        evaluate(&redundant),
        Ok(RuntimeValue::Array(vec![
            RuntimeValue::Nil,
            RuntimeValue::Symbol("b".into()),
        ]))
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
