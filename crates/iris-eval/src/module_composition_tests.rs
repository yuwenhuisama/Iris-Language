use iris_runtime::{ArrayRef, Capability, ClassError, PolicyOrigin, Value as RuntimeValue};

use super::{EvaluationError, evaluate};

#[test]
fn module_instance_method_dispatches_on_a_mixed_in_class_instance() {
    // Given
    let source =
        "module M { public fun m() -> Symbol { :module } }; class A mixin M { }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("module".into())));
}

#[test]
fn module_instance_method_reads_and_writes_the_receiver_raw_ivar() {
    // Given
    let source = "module M { public fun write() -> Integer { @x = 1 } public fun read() -> Integer { @x } }; class A mixin M { }; let a = A.new(); [a.write(), a.read()]";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(
        result,
        Ok(RuntimeValue::Array(ArrayRef::new(vec![
            RuntimeValue::Integer(1_u8.into()),
            RuntimeValue::Integer(1_u8.into()),
        ])))
    );
}

#[test]
fn private_mixin_authorization_changes_private_selector_access_but_not_raw_ivar_access() {
    // Given
    let authorized = "module M { public fun bump() -> Integer { @x = 1; secret() } }; class A mixin M private { private fun secret() -> Integer { @x } }; A.new().bump()";
    let unauthorized = "module M { public fun bump() -> Integer { @x = 1; secret() } public fun read() -> Integer { @x } }; class A mixin M { private fun secret() -> Integer { @x } }; let a = A.new(); [a.bump(), a.read()]";

    // When
    let authorized_result = evaluate(authorized);
    let unauthorized_result = evaluate(unauthorized);

    // Then
    assert_eq!(authorized_result, Ok(RuntimeValue::Integer(1_u8.into())));
    assert!(matches!(
        unauthorized_result,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::VisibilityDenied { .. }
            )
        ))
    ));
}

#[test]
fn class_method_precedes_a_module_method_in_c053_mro_order() {
    // Given: RUNTIME-C053 requires `C, B, A, S...` for `mixin A, B`.
    let source = "module M { public fun m() -> Symbol { :module } }; class A mixin M { public fun m() -> Symbol { :class } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}

#[test]
fn super_from_a_module_method_continues_after_the_module_in_current_mro() {
    // Given
    let source = "class Base { public fun m() -> Symbol { :base } }; module M { override public fun m() -> Symbol { super() } }; class A extends Base mixin M { }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("base".into())));
}

#[test]
fn module_meta_deny_is_reported_with_the_module_as_policy_origin() {
    // Given
    let source = "module M meta deny method_set { }; class A mixin M { }; open class A { public fun added() { :added } }";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
            operation: Capability::MethodSet,
            policy_origin: PolicyOrigin::Module(_),
            ..
        }))
    ));
}

#[test]
fn modules_capability_denial_blocks_declarative_mixin() {
    // Given
    let source = "class Base meta deny modules { }; module M { public fun m() { :m } }; class A extends Base mixin M { }";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
            operation: Capability::Modules,
            ..
        }))
    ));
}

#[test]
fn module_member_replacing_a_resolved_member_requires_override() {
    // Given
    let source = "class Base { public fun m() { :base } }; module M { public fun m() { :module } }; class A extends Base mixin M { }";

    // When
    let result = evaluate(source);

    // Then
    assert!(matches!(
        result,
        Err(EvaluationError::Class(ClassError::OverrideRequired { .. }))
    ));
}

#[test]
fn class_without_a_mixin_keeps_ordinary_dispatch() {
    // Given
    let source = "class A { public fun m() -> Symbol { :class } }; A.new().m()";

    // When
    let result = evaluate(source);

    // Then
    assert_eq!(result, Ok(RuntimeValue::Symbol("class".into())));
}
