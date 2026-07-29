use iris_runtime::{
    BuiltinClass, Kernel, KernelError, MethodBody, NativeSelector, Runtime, Selector, StaticSpine,
    Value as RuntimeValue, Visibility,
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
        "let mut log = []; let ignored_a = log.append(:a); let ignored_b = log.append(:b); log";

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
    let source = "let mut values = [:a]; [values.append(:b), values]";

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
    let source = "let mut log = []; try { log.append(:try); raise :x } catch _ { log.append(:catch); :handled } finally { log.append(:finally) }; log";

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

    // Then
    assert_eq!(add_result, Err(EvaluationError::Runtime(KernelError::Type)));
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
    let mut kernel = Kernel::new()?;
    let bool_class = kernel.class(BuiltinClass::Bool)?;
    kernel.registry_mut().publish_method(
        bool_class,
        NativeSelector::Equal.id(),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    kernel.registry_mut().publish_method(
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

    // Then
    assert_eq!(result, Err(EvaluationError::UnsupportedConstruct));
}

#[test]
fn source_method_returns_symbol_literal() {
    // Given
    let source = "class A { public fun m() -> Integer { :added } }; A.new().m()";

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
    assert_eq!(
        scoped_binding_result,
        Err(EvaluationError::UnsupportedConstruct)
    );
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
    let source =
        "class A { class fun build() -> Integer { :class } }; class B extends A { }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}

#[test]
fn source_class_object_dispatch_keeps_direct_singleton_method_lookup() {
    // Given
    let source = "class A { class fun build() -> Integer { :class } }; A.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}

#[test]
fn source_class_object_dispatch_prefers_its_own_singleton_method_over_an_inherited_one() {
    // Given
    let source = "class A { class fun build() -> Integer { :parent } }; class B extends A { class fun build() -> Integer { :child } }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("child".into())));
}

#[test]
fn source_class_object_super_continues_after_the_singleton_method_owner() {
    // Given
    let source = "class A { class fun build() -> Integer { :parent } }; class B extends A { class fun build() -> Integer { super() } }; B.build()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("parent".into())));
}

#[test]
fn source_class_object_dispatch_reports_missing_selector_after_superclass_chain() {
    // Given
    let source =
        "class A { class fun build() -> Integer { :class } }; class B extends A { }; B.unknown()";

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
    let source = "class A { property fun name() -> Integer { :get } property fun name=(value: Integer) -> Integer { :set } }; let a = A.new(); [a.name, a.name = 1]";

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
    let source = "module M { module fun helper() -> Integer { 3 } }; M.helper()";

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
    let source = "let mut escaped = nil; class A { public fun initialize() { escaped = self; raise :sentinel } }; try { A.new() } catch error { escaped.to_bool() }";

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
    let source = "let mut value = 1; class A { public fun update() -> Integer { value = 2 } }; A.new().update(); value";

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
