#![expect(
    clippy::unwrap_used,
    reason = "fixtures must compile before signature validation"
)]

use iris_runtime::Value;
use iris_vm::{MachineError, compile, run};

#[test]
fn defaults_are_preserved_when_replacement_accepts_more_arguments() {
    let source = r#"
        class Rule { public fun total(value: Integer) -> Integer { value } }
        open class Rule { public override fun total(value: Object = 1) -> Integer { 12 } }
        Rule.new().total(1)
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Ok(Value::Integer(12_u64.into())));
}

#[test]
fn unions_are_structural_when_replacement_widens_input() {
    let source = r#"
        class Rule { public fun total(value: Integer) -> Integer | String { value } }
        open class Rule { public override fun total(value: Integer | String) -> Integer { 12 } }
        Rule.new().total(1)
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Ok(Value::Integer(12_u64.into())));
}

#[test]
fn omitted_return_is_dynamic_when_replacement_has_no_annotation() {
    let source = r#"
        class Rule { public fun total() -> Integer { 1 } }
        open class Rule { public override fun total() { 12 } }
        Rule.new().total()
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn untyped_parameter_is_dynamic_when_replacement_narrows_input() {
    let source = r#"
        class Rule { public fun total(value) -> Integer { 1 } }
        open class Rule { public override fun total(value: Integer) -> Integer { value } }
        Rule.new().total(1)
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn rest_shape_is_preserved_when_replacement_removes_rest() {
    let source = r#"
        class Rule { public fun total(*values) -> Integer { 1 } }
        open class Rule { public override fun total(value) -> Integer { 12 } }
        Rule.new().total(1)
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn builtin_source_method_is_checked_when_reopened_again() {
    let source = r#"
        open class Integer { public fun boundary_total() -> Integer { 1 } }
        open class Integer { public override fun boundary_total() -> String { "bad" } }
        (1).boundary_total()
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn keyword_shape_is_preserved_when_replacement_renames_keyword() {
    let source = r#"
        class Rule { public fun total(key amount: Integer) -> Integer { amount } }
        open class Rule { public override fun total(key other: Integer) -> Integer { other } }
        1
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn block_shape_is_preserved_when_replacement_drops_block() {
    let source = r#"
        class Rule { public fun total(&block) -> Integer { 1 } }
        open class Rule { public override fun total() -> Integer { 2 } }
        1
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn parameter_shapes_are_compatible_when_full_signature_is_retained() {
    let source = r#"
        class Rule { public fun total(value = 1, *rest, key amount = 2, **options, &block) -> Integer { 1 } }
        open class Rule { public override fun total(value = 1, *rest, key amount = 2, **options, &block) -> Integer { 2 } }
        42
    "#;
    let program = compile(source).unwrap();
    let outcome = run(&program);
    assert_eq!(outcome, Ok(Value::Integer(42_u64.into())));
}
