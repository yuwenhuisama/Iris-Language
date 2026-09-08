#![expect(
    clippy::unwrap_used,
    reason = "regression fixtures must compile before execution"
)]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

#[test]
fn array_cell_keeps_value_when_dynamic_integer_is_rejected() {
    let source = "module Input { public module fun read(value) { value } }; mut cell = []; let failure = try { cell = Input.read(1) } catch error { error }; [cell, failure]";
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Array(ArrayRef::new(vec![])),
            Value::Symbol("TypeContractError".into())
        ])))
    );
}

#[test]
fn hash_cell_keeps_value_when_dynamic_integer_is_rejected() {
    let source = "module Input { public module fun read(value) { value } }; mut cell = %{}; let original = cell; let failure = try { cell = Input.read(1) } catch error { error }; [cell.same?(original), failure]";
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Symbol("TypeContractError".into())
        ])))
    );
}

#[test]
fn symbol_cell_keeps_value_when_dynamic_integer_is_rejected() {
    let source = "module Input { public module fun read(value) { value } }; mut cell = :ready; let failure = try { cell = Input.read(1) } catch error { error }; [cell, failure]";
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("ready".into()),
            Value::Symbol("TypeContractError".into())
        ])))
    );
}

#[test]
fn final_removal_rejects_when_inherited_method_breaks_active_promise() {
    let source = r#"
        class Parent { public fun total() -> Object { "bad" } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:total) { "bad" }
            candidate.remove_method(:total)
        })
        Rule.new().total()
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Err(MachineError::TypeContractError));
}

#[test]
fn direct_removal_rejects_when_inherited_method_breaks_active_promise() {
    let source = r#"
        class Parent { public fun total() -> Object { "bad" } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Rule.remove_method(:total)
        Rule.new().total()
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Err(MachineError::TypeContractError));
}

#[test]
fn contract_input_widening_succeeds_when_parent_is_transitive() {
    let source = r#"
        contract Parent { }
        contract Middle extends Parent { }
        contract Child extends Middle { }
        class Rule { public fun total(value: Child) -> Object { value } }
        open class Rule { public override fun total(value: Parent) -> Object { value } }
        42
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn class_conformance_widening_succeeds_when_contract_parent_is_transitive() {
    let source = r#"
        contract Parent { }
        contract Child extends Parent { }
        class Item for Child { }
        class Rule { public fun total(value: Item) -> Object { value } }
        open class Rule { public override fun total(value: Parent) -> Object { value } }
        42
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn caught_removal_failure_aborts_outer_group() {
    let source = r#"
        class Locked meta deny method_set { }
        class Rule { }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            try { Locked.remove_method(:sibling) } catch error { nil }
        })
        Rule.active_revision
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn caught_undef_failure_aborts_outer_group() {
    let source = r#"
        class Locked meta deny method_set { }
        class Rule { }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            try { Locked.undef_method(:sibling) } catch error { nil }
        })
        Rule.active_revision
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn removal_keeps_instance_and_sibling_when_inherited_signature_is_incompatible() {
    let source = r#"
        class Parent { public fun total() -> Object { "bad" } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let rule = Rule.new()
        let failure = try { Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            candidate.remove_method(:total)
        }) } catch error { error }
        [rule.total(), Rule.active_revision, failure, Rule.method(:sibling)]
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(10_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Symbol("TypeContractError".into()),
            Value::Nil
        ])))
    );
}

#[test]
fn undefinition_succeeds_when_noncontract_method_is_removed_without_fallback() {
    let source = r#"
        class Parent { public fun total() -> Object { "bad" } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:total) { "bad" }
            candidate.undef_method(:total)
        })
        Rule.method(:total)
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Nil));
}

#[test]
fn removal_succeeds_when_inherited_signature_preserves_promise() {
    let source = r#"
        class Parent { public fun total() -> Integer { 11 } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Rule.remove_method(:total)
        Rule.new().total()
    "#;
    let program = compile(source).unwrap();
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(11_u64.into())));
}
