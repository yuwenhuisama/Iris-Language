#![allow(clippy::unwrap_used, reason = "test fixtures and assertions")]

use super::{LOCAL_PACKAGE, SourceEvaluator};
use crate::{EvaluationError, Session};
use iris_runtime::Value;

pub(super) fn execute(
    evaluator: &mut SourceEvaluator,
    source: &str,
) -> Result<Value, EvaluationError> {
    let parsed = iris_parser::parse(source);
    assert!(parsed.program_accepted, "{parsed:?}");
    evaluator.program(&parsed.program)
}

#[test]
fn literal_violation_prevents_prior_effects() {
    let mut evaluator = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    let given = "mut marker = 1; mut port = 8080; port = \"9090\"; port";
    let when = execute(&mut evaluator, given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert!(!evaluator.names.contains_key("marker"));
}

#[test]
fn dynamic_write_preserves_old_cell() {
    for declaration in [
        "mut port = 8080",
        "mut port: Integer = 8080",
        "mut port: Integer | String = 8080",
    ] {
        let mut evaluator = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
        execute(
            &mut evaluator,
            &format!(
                "module Input {{ public module fun read(value) {{ value }} }}; {declaration}; nil"
            ),
        )
        .unwrap();
        let when = execute(&mut evaluator, "port = Input.read(true)");
        assert_eq!(when, Err(EvaluationError::TypeContractError));
        assert_eq!(
            evaluator.names["port"].value(),
            Value::Integer(8080_u64.into())
        );
    }
}

#[test]
fn session_infers_from_prior_contract_not_current_value() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("mut port: Integer | String = 8080; nil")
        .unwrap();
    given.evaluate("mut copy = port; nil").unwrap();
    let when = given.evaluate("copy = \"ok\"");
    assert_eq!(when, Ok(Value::Text("ok".into())));
    assert_eq!(
        given.evaluate("copy = true"),
        Err(EvaluationError::TypeContractError)
    );
}

#[test]
fn escaped_mutable_capture_retains_contract_and_cell() {
    let given = "module M { public module fun make() { mut port = 8080; [{ |value| port = value }, { || port }] } }; let pair = M.make(); let update = pair[0]; let read = pair[1]; let valid = update.call(9090); let refused = try { update.call(\"bad\"); false } catch error { true }; [refused, read.call()]";
    let when = crate::evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(9090_u64.into())
        ])))
    );
}

#[test]
fn logical_and_compound_writes_check_final_values() {
    for (initial, assignment, expected) in [
        ("true", "port &&= Input.read(1)", Value::Bool(true)),
        ("false", "port ||= Input.read(1)", Value::Bool(false)),
        ("8", "port /= Input.read(2)", Value::Integer(8_u8.into())),
    ] {
        let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
        execute(&mut given, &format!("module Input {{ public module fun read(value) {{ value }} }}; mut port = {initial}; nil")).unwrap();
        let when = execute(&mut given, assignment);
        assert_eq!(when, Err(EvaluationError::TypeContractError));
        assert_eq!(given.names["port"].value(), expected);
    }
}

#[test]
fn skipped_logical_write_does_not_evaluate_rhs() {
    let given = "module Input { public module fun read() { raise :rhs } }; mut port = false; port &&= Input.read()";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Bool(false)));
}

#[test]
fn shadowed_contract_is_restored_after_failure() {
    let mut given = Session::new().unwrap();
    given.evaluate("mut port = 8; nil").unwrap();
    given
        .evaluate("if true { mut port: Object = \"inner\"; port = true }; nil")
        .unwrap();
    let when = given.evaluate("port = \"bad\"");
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert_eq!(given.evaluate("port"), Ok(Value::Integer(8_u8.into())));
}

#[test]
fn unknown_initializer_is_not_fixed_from_runtime_value() {
    let given = "module Input { public module fun read(value) { value } }; mut port = Input.read(8); port = \"ok\"";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Text("ok".into())));
}

#[test]
fn group_signature_failure_rolls_back_sibling_target() {
    let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(
        &mut given,
        "class A { public fun value() -> Integer { 1 } }; class B { }; let old = A.new(); nil",
    )
    .unwrap();
    let first = given.class_name("A").unwrap().unwrap();
    let second = given.class_name("B").unwrap().unwrap();
    let before =
        [first, second].map(|class| given.runtime.registry().active(class).unwrap().clone());
    let when = execute(
        &mut given,
        "A.open() { |target| B.open() { |other| other.define_method(:sibling) { 2 } }; target.define_method(:value) { \"bad\" } }",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    for (class, revision) in [first, second].into_iter().zip(before) {
        let after = given.runtime.registry().active(class).unwrap();
        assert_eq!(after.id(), revision.id());
        assert_eq!(after.methods(), revision.methods());
    }
    assert_eq!(
        execute(&mut given, "old.value()"),
        Ok(Value::Integer(1_u8.into()))
    );
}

#[test]
fn replacement_obeys_nominal_variance() {
    let given = "class Base { }; class Child extends Base { }; class Rule { public fun choose(value: Child) -> Base { value } }; open class Rule { public override fun choose(value: Base) -> Child { Child.new() } }; Rule.new().choose(Base.new()) is Child";
    let when = crate::evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn class_body_mutable_local_checks_dynamic_write() {
    let given = "module Input { public module fun read(value) { value } }; class A { mut port = 8; port = Input.read(\"bad\") }; nil";
    let when = crate::evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn failed_singleton_group_does_not_poison_next_open() {
    let mut given = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(
        &mut given,
        "class A { public class fun value() -> Integer { 1 } }; nil",
    )
    .unwrap();
    let parsed = iris_parser::parse(
        "open class A { public override class fun value() -> String { \"bad\" } }",
    );
    assert!(parsed.program_accepted);
    let when = given.program(&parsed.program);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    assert_eq!(
        execute(
            &mut given,
            "open class A { public fun sibling() { 2 } }; A.value()"
        ),
        Ok(Value::Integer(1_u8.into()))
    );
}

#[test]
fn package_entrypoints_preflight_uncalled_method_locals() {
    let given = "module M { public fun invalid() { mut port = 8; port = true } }".to_owned();
    let sources = vec![("main.iris".to_owned(), given.clone())];
    let when = [
        crate::evaluate_packages(&[("app".to_owned(), given.clone())]),
        crate::load_package("app", &sources).map(|_| Value::Nil),
        crate::load_package_tree(&[("app".to_owned(), sources)], None).map(|_| Value::Nil),
        crate::evaluate_with_natives(
            &given,
            std::rc::Rc::new(iris_native_host::NativeRegistry::new()),
        ),
    ];
    for result in when {
        assert_eq!(result, Err(EvaluationError::TypeContractError));
    }
}

#[test]
fn inferred_collection_bindings_admit_their_runtime_representations() {
    let given = "mut array = []; array = [1]; mut hash = %{}; hash = %{ :key: 1 }; mut symbol = :old; symbol = :new; [array, hash[:key], symbol]";
    let when = crate::evaluate(given);
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn failed_open_preserves_revision_members_and_existing_instance() {
    let mut evaluator = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(&mut evaluator, "class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); nil").unwrap();
    let class = evaluator.class_name("ShippingRule").unwrap().unwrap();
    let before = evaluator.runtime.registry().active(class).unwrap().clone();
    let when = execute(
        &mut evaluator,
        "open class ShippingRule { public fun sibling() -> Integer { 1 }; public override fun total(subtotal: Integer) -> String { \"free\" } }; nil",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    let after = evaluator.runtime.registry().active(class).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.methods(), before.methods());
    assert_eq!(
        execute(&mut evaluator, "rule.total(90)"),
        Ok(Value::Integer(100_u8.into()))
    );
}

#[test]
fn singleton_signature_failure_preserves_revision() {
    let mut evaluator = SourceEvaluator::new_in_package(LOCAL_PACKAGE).unwrap();
    execute(
        &mut evaluator,
        "class ShippingRule { public class fun total(value: Integer) -> Integer { value } }; nil",
    )
    .unwrap();
    let class = evaluator.class_name("ShippingRule").unwrap().unwrap();
    let before = evaluator.runtime.registry().active(class).unwrap().clone();
    let when = execute(
        &mut evaluator,
        "open class ShippingRule { public fun sibling() { 1 }; public override class fun total(value: Integer) -> String { \"bad\" } }; nil",
    );
    assert_eq!(when, Err(EvaluationError::TypeContractError));
    let after = evaluator.runtime.registry().active(class).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.singleton_methods(), before.singleton_methods());
}
