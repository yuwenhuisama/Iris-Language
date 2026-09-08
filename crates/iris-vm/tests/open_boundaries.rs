#![expect(
    clippy::unwrap_used,
    reason = "fixtures must compile before runtime boundary checks"
)]

use iris_vm::MachineError;

#[test]
fn reopen_rejects_when_return_signature_is_incompatible() {
    let source = r#"
        class ShippingRule {
            public fun total(subtotal: Integer) -> Integer { subtotal + 10 }
        }
        let rule = ShippingRule.new()
        open class ShippingRule {
            public fun sibling() -> Integer { 7 }
            public override fun total(subtotal: Integer) -> String { "free shipping" }
        }
        rule.total(90)
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn singleton_reopen_rejects_when_return_signature_is_incompatible() {
    let source = r#"
        class Rule { public class fun total() -> Integer { 10 } }
        open class Rule { public override class fun total() -> String { "free" } }
        Rule.total()
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn callback_rejects_when_dynamic_method_weakens_return_promise() {
    let source = r#"
        class Rule { public fun total() -> Integer { 10 } }
        Rule.open({ |candidate|; candidate.define_method(:total) { "free" } })
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn direct_definition_rejects_when_dynamic_method_weakens_return_promise() {
    let source = r#"
        class Rule { public fun total() -> Integer { 10 } }
        Rule.define_method(:total) { "free" }
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn reopen_accepts_when_parameters_widen_and_returns_narrow() {
    let source = r#"
        class Animal { }
        class Dog extends Animal { }
        class Rule { public fun total(value: Dog) -> Animal { value } }
        let rule = Rule.new()
        open class Rule { public override fun total(value: Animal) -> Dog { Dog.new() } }
        rule.total(Dog.new()) is Dog
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(iris_runtime::Value::Bool(true)));
}

#[test]
fn reopen_rejects_when_replacement_adds_required_parameter() {
    let source = r#"
        class Rule { public fun total(value: Integer = 1) -> Integer { value } }
        open class Rule { public override fun total(value: Integer) -> Integer { value } }
        Rule.new().total()
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn alias_rejects_when_it_replaces_a_typed_method() {
    let source = r#"
        class Rule {
            public fun total() -> Integer { 10 }
            public fun bad() -> String { "bad" }
        }
        Rule.open({ |candidate|; candidate.alias_method(:total, :bad) })
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn successive_reopen_checks_active_promise_when_prior_replacement_narrowed() {
    let source = r#"
        class Rule { public fun total() -> Object { 10 } }
        open class Rule { public override fun total() -> Integer { 11 } }
        open class Rule { public override fun total() -> String { "bad" } }
        Rule.new().total()
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Err(MachineError::TypeContractError));
}

#[test]
fn callback_accepts_final_signature_when_intermediate_candidate_is_incompatible() {
    let source = r#"
        class Rule {
            public fun total() -> Integer { 10 }
            public fun good() -> Integer { 12 }
        }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:total) { "bad" }
            candidate.alias_method(:total, :good)
        })
        Rule.new().total()
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(iris_runtime::Value::Integer(12_u64.into())));
}

#[test]
fn outer_group_rolls_back_when_nested_staging_failure_is_caught() {
    let source = r#"
        class Rule { }
        class Locked meta deny method_set { }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            try { Locked.open({ |locked|; locked.define_method(:sibling) { 8 } }) } catch error { nil }
        })
        Rule.active_revision
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(iris_runtime::Value::Integer(1_u64.into())));
}

#[test]
fn outer_group_rolls_back_when_direct_staging_failure_is_caught() {
    let source = r#"
        class Rule { }
        class Locked meta deny method_set { }
        let changed = Rule.open({ |candidate|;
            candidate.define_method(:sibling) { 7 }
            try { Locked.define_method(:sibling) { 8 } } catch error { nil }
        })
        Rule.active_revision
    "#;
    let program = iris_vm::compile(source).unwrap();
    let outcome = iris_vm::run(&program);
    assert_eq!(outcome, Ok(iris_runtime::Value::Integer(1_u64.into())));
}
