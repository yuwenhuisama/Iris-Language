#![allow(clippy::unwrap_used, reason = "test fixtures and assertions")]

use super::boundary_tests::execute;
use super::{LOCAL_PACKAGE, SourceEvaluator};
use crate::{EvaluationError, Session};
use iris_runtime::Value;

#[test]
fn inferred_collection_rejects_dynamic_write_and_retains_old_cell() {
    for initial in ["[]", "%{}", ":old"] {
        let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
        execute(&mut given, &format!("module Input {{ public module fun read(value) {{ value }} }}; mut cell = {initial}; nil")).unwrap();
        let before = given.names["cell"].value();
        let when = execute(&mut given, "cell = Input.read(1)");
        assert_eq!(when, Err(EvaluationError::TypeContractError));
        assert_eq!(given.names["cell"].value(), before);
    }
}

#[test]
fn replacement_accepts_contract_parameter_contravariance() {
    let given = "contract Parent { }; contract Child extends Parent { }; class Dog for Child { }; class Rule { public fun choose(value: Child) -> Integer { 1 } }; open class Rule { public override fun choose(value: Parent) -> Integer { 2 } }; Rule.new().choose(Dog.new())";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Integer(2_u8.into())));
}

#[test]
fn replacement_accepts_class_return_covariance_through_parent_contract() {
    let given = "contract Parent { }; contract Child extends Parent { }; class Dog for Child { }; class Rule { public fun choose() -> Parent { Dog.new() } }; open class Rule { public override fun choose() -> Dog { Dog.new() } }; Rule.new().choose() is Dog";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn removed_replacement_validates_final_inherited_signature() {
    let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(&mut given, "class Parent { public fun total() -> Object { \"bad\" } }; class Rule extends Parent { public override fun total() -> Integer { 10 } }; let old = Rule.new(); nil").unwrap();
    let class = given.class_name("Rule").unwrap().unwrap();
    let before = given.runtime.registry().active(class).unwrap().clone();
    let when = execute(
        &mut given,
        "Rule.open() { |target| target.define_method(:sibling) { 2 }; target.define_method(:total) { \"bad\" }; target.remove_method(:total) }",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    let after = given.runtime.registry().active(class).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    assert_eq!(
        execute(&mut given, "old.total()"),
        Ok(Value::Integer(10_u8.into()))
    );
}

#[test]
fn direct_signature_mutations_preserve_active_revision_when_invalid() {
    for operation in [
        "Rule.define_method(:total) { \"bad\" }",
        "Rule.alias_method(:total, :bad)",
    ] {
        let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
        execute(&mut given, "class Rule { public fun total() -> Integer { 10 }; public fun bad() -> String { \"bad\" } }; let old = Rule.new(); nil").unwrap();
        let class = given.class_name("Rule").unwrap().unwrap();
        let before = given.runtime.registry().active(class).unwrap().clone();
        let when = execute(&mut given, operation);
        assert_eq!(when, Err(EvaluationError::TypeContractError));
        let after = given.runtime.registry().active(class).unwrap();
        assert_eq!(after.id(), before.id());
        assert_eq!(after.methods(), before.methods());
        assert_eq!(
            execute(&mut given, "old.total()"),
            Ok(Value::Integer(10_u8.into()))
        );
    }
}

#[test]
fn callback_accepts_intermediate_mismatch_repaired_by_alias() {
    let given = "class Rule { public fun total() -> Integer { 10 }; public fun good() -> Integer { 20 } }; let old = Rule.new(); let changed = Rule.open() { |target| target.define_method(:total) { \"bad\" }; target.alias_method(:total, :good) }; old.total()";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Integer(20_u8.into())));
}

#[test]
fn caught_meta_failure_aborts_group_and_preserves_sibling_revision() {
    for operation in [
        "target.remove_method(:total)",
        "target.undef_method(:total)",
        "target.alias_method(:total, :total)",
        "target.define_method(:extra) { 2 }",
    ] {
        let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
        execute(&mut given, "class Rule meta deny method_set { public fun total() -> Integer { 10 } }; class Sibling { }; nil").unwrap();
        let class = given.class_name("Sibling").unwrap().unwrap();
        let before = given.runtime.registry().active(class).unwrap().clone();
        let when = execute(
            &mut given,
            &format!(
                "Rule.open() {{ |target| Sibling.open() {{ |other| other.define_method(:added) {{ 1 }} }}; try {{ {operation} }} catch error {{ nil }} }}"
            ),
        );
        assert!(when.is_ok(), "{when:?}");
        let after = given.runtime.registry().active(class).unwrap();
        assert_eq!(after.id(), before.id());
        assert_eq!(after.methods(), before.methods());
    }
}

#[test]
fn session_preserves_alias_contracts_for_later_static_preparation() {
    let mut given = Session::new().unwrap();
    given.evaluate("type Port = Integer; nil").unwrap();
    let when = given.evaluate("mut marker = 1; mut port: Port = 8; port = \"bad\"");
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert_eq!(given.evaluate("marker"), Err(EvaluationError::NameError));
}

#[test]
fn session_normalizes_alias_signatures_from_prior_chunk() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("type Port = Integer; class Rule { public fun total() -> Port { 10 } }; nil")
        .unwrap();
    let when = given.evaluate(
        "open class Rule { public override fun total() -> Integer { 20 } }; Rule.new().total()",
    );
    assert_eq!(when, Ok(Value::Integer(20_u8.into())));
}

#[test]
fn same_chunk_alias_rejects_literal_before_effects() {
    let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    let when = execute(
        &mut given,
        "type Port = Integer; mut marker = 1; mut port: Port = 8; port = \"bad\"",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert!(!given.names.contains_key("marker"));
}

#[test]
fn same_chunk_alias_and_target_signatures_are_equivalent() {
    let given = "type Port = Integer; class Rule { public fun total() -> Port { 10 } }; open class Rule { public override fun total() -> Integer { 20 } }; Rule.new().total()";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Integer(20_u8.into())));
}

#[test]
fn prior_alias_normalizes_later_method_parameter_and_return_contracts() {
    let mut given = Session::new().unwrap();
    given.evaluate("type Port = Integer; nil").unwrap();
    given
        .evaluate("class Rule { public fun total(port: Port) -> Port { port } }; nil")
        .unwrap();
    let when = given.evaluate("open class Rule { public override fun total(port: Integer) -> Integer { port + 1 } }; Rule.new().total(19)");
    assert_eq!(when, Ok(Value::Integer(20_u8.into())));
}

#[test]
fn contract_binding_rejects_nonconformer_and_retains_old_cell() {
    let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(&mut given, "contract Parent { }; contract Child extends Parent { }; class Dog for Child { }; module Input { public module fun read(value) { value } }; mut pet: Parent = Dog.new(); nil").unwrap();
    let before = given.names["pet"].value();
    let when = execute(&mut given, "pet = Input.read(1)");
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert_eq!(given.names["pet"].value(), before);
}
