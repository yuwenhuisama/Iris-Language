#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

#[test]
fn dynamic_property_is_initialized_when_publication_commits() {
    let given = "class Target {} module Main { public fun run() { Target.open() { |target| target.define_property(:value) { 7 } }; Target.new().value } } Main.run()";
    let when = run(&compile(given).expect("dynamic property compiles"));
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn dynamic_property_mutation_shares_storage_when_raw_ivar_is_read() {
    let given = "class Target { public fun raw() { @value } } module Main { public fun run() { Target.open() { |target| target.define_property(:value) { 7 } }; let target = Target.new(); %[target.value, target.value = 9, target.raw(), target.value, Target.new().value] } } Main.run()";
    let when = run(&compile(given).expect("dynamic mutation compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [7_u64, 9, 9, 9, 7]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn ordinary_module_open_is_ignored_when_target_is_absent() {
    let given = "open module Absent { public fun value() { 2 } } 1";
    let when = compile(given).map(|program| run(&program));
    assert_eq!(
        when.expect("undecorated absent open compiles"),
        Ok(Value::Integer(1_u64.into()))
    );
}

#[test]
fn decorated_module_open_is_rejected_when_target_is_absent() {
    for given in [
        "@Unknown() open module Absent {} 1",
        "open module Absent { @Unknown() public fun value() { 2 } } 1",
        "open module Absent { @Unknown() property value: Integer = 2 } 1",
    ] {
        let when = compile(given);
        assert!(when.is_err(), "{given}");
    }
}

#[test]
fn explicit_accessors_keep_privacy_when_dynamic_property_uses_same_name() {
    let given = "class Dynamic {} class Private { property value: Integer = 3 { get; set; } } class Readonly { property value: Integer = 5 { public get; } } module Main { public fun run() { Dynamic.open() { |target| target.define_property(:value) { 7 } }; %[try { Private.new().value; false } catch error { error == :MethodVisibilityError }, try { Private.new().value = 9; false } catch error { error == :MethodVisibilityError }, try { Readonly.new().value = 9; false } catch error { error == :MessageNotFound }] } } Main.run()";
    let when = run(&compile(given).expect("accessor privacy compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![Value::Bool(true); 3])))
    );
}

#[test]
fn explicit_accessor_absence_is_preserved_when_dynamic_property_reuses_its_name() {
    let given = "class Target { property value: Integer = 3 { public get; } } module Main { public fun run() { Target.open() { |target| target.define_property(:value) { 7 } }; try { Target.new().value = 9; false } catch error { error == :MessageNotFound } } } Main.run()";
    let when = run(&compile(given).expect("explicit accessor compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn dynamic_property_is_absent_when_publication_rolls_back() {
    let given = "class Target {} module Main { public fun run() { try { Target.open() { |target| target.define_property(:value) { 7 }; raise :abort } } catch error { nil }; %[Target.properties, try { Target.new().value = 9; false } catch error { error == :MessageNotFound }] } } Main.run()";
    let when = run(&compile(given).expect("property rollback compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::ReadonlyArray(vec![]),
            Value::Bool(true)
        ])))
    );
}
