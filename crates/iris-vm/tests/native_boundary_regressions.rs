#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

#[test]
fn rehash_accepts_a_trailing_block_when_the_hash_is_empty() {
    let given = "%{}.rehash() { |kept, left, incoming, right| (kept, left + right) }";
    let when = run(&compile(given).expect("rehash compiles"));
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn rehash_calls_each_collision_once_when_keys_change() {
    let given = r#"class Key { public property equal_id: Integer = 0 public property hash_code: Integer = 0 public fun ==(other: Object) -> Bool { @equal_id == other.equal_id() } public fun hash() -> Integer { @hash_code } public fun equal_id() -> Integer { @equal_id } } module M { public fun run() -> Array { mut k1 = Key.new(); k1.equal_id = 1; k1.hash_code = 10; mut k2 = Key.new(); k2.equal_id = 2; k2.hash_code = 10; mut k3 = Key.new(); k3.equal_id = 3; k3.hash_code = 20; mut k4 = Key.new(); k4.equal_id = 4; k4.hash_code = 20; mut h = %{}; h[k1] = 1; h[k2] = 2; h[k3] = 4; h[k4] = 8; k2.equal_id = 1; k4.equal_id = 3; mut calls = 0; h.rehash() { |kept, left, incoming, right| calls = calls + 1; (kept, left + right) }; %[h.length(), calls, h.fetch(k1), h.fetch(k3)] } } M.run()"#;
    let when = run(&compile(given).expect("collision merge compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [2_u64, 2, 3, 12]
                .map(|value| Value::Integer(value.into()))
                .to_vec()
        )))
    );
}

#[test]
fn rehash_keeps_entries_when_the_merge_returns_the_wrong_shape() {
    let given = r#"class Key { public property equal_id: Integer = 0 public fun ==(other: Object) -> Bool { @equal_id == other.equal_id() } public fun hash() -> Integer { 0 } public fun equal_id() -> Integer { @equal_id } } module M { public fun run() { let first = Key.new(); let second = Key.new(); second.equal_id = 1; let entries = %{}; entries[first] = 2; entries[second] = 3; second.equal_id = 0; mut calls = 0; let error = try { entries.rehash() { |kept, left, incoming, right| calls = calls + 1; :bad } } catch error { error }; %[error, calls, entries.length(), entries.values()] } } M.run()"#;
    let when = run(&compile(given).expect("invalid merge compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("TypeContractError".into()),
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into()),
            Value::Array(ArrayRef::new(vec![
                Value::Integer(2_u64.into()),
                Value::Integer(3_u64.into())
            ])),
        ])))
    );
}

#[test]
fn native_blocks_reject_arguments_when_the_shape_is_invalid() {
    for given in [
        "%{}.rehash(1) { |kept, left, incoming, right| (kept, left) }",
        "%{}.rehash({ |kept, left, incoming, right| (kept, left) }) { |kept, left, incoming, right| (kept, right) }",
        "%[1].map(1) { |value| value }",
        "%[1].reduce(1, 2) { |left, right| left + right }",
        "%[1].length() { 1 }",
        "%{}.length() { 1 }",
    ] {
        let when = run(&compile(given).expect("invalid native call compiles"));
        assert_eq!(when, Err(MachineError::ArgumentError), "{given}");
    }
}

#[test]
fn native_predicates_short_circuit_when_the_block_decides_the_result() {
    let given = "module M { public fun run() { mut calls = 0; let all = %[1, 2, 3].all?() { |value| calls = calls + 1; value < 2 }; let any = %[1, 2, 3].any?() { |value| calls = calls + 1; value == 2 }; %[all, any, calls] } } M.run()";
    let when = run(&compile(given).expect("predicates compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(false),
            Value::Bool(true),
            Value::Integer(4_u64.into())
        ])))
    );
}

#[test]
fn bytes_return_accepts_text_conversion_when_annotated() {
    for (given, expected) in [
        (
            r#"module M { public fun run() -> Bytes { "ab".to_bytes() } } M.run()"#,
            vec![0x61, 0x62],
        ),
        (
            r#"module M { public fun run() -> Bytes { "\xFF".to_bytes() } } M.run()"#,
            vec![0xc3, 0xbf],
        ),
    ] {
        let when = run(&compile(given).expect("Bytes return compiles"));
        assert_eq!(when, Ok(Value::Bytes(expected)));
    }
}

#[test]
fn bytes_return_rejects_other_values_when_annotated() {
    for value in ["1", "nil", r#""ab""#, "%[97, 98]"] {
        let given = format!("module M {{ public fun run() -> Bytes {{ {value} }} }} M.run()");
        let when = run(&compile(&given).expect("incorrect Bytes return compiles"));
        assert_eq!(when, Err(MachineError::TypeContractError), "{given}");
    }
}

#[test]
fn bytes_parameter_and_binding_accept_only_bytes_when_annotated() {
    let given = r#"module M { public fun accept(value: Bytes) -> Bytes { let copy: Bytes = value; copy } public fun run() { %[accept("ab".to_bytes()), try { accept("ab") } catch error { error }] } } M.run()"#;
    let when = run(&compile(given).expect("Bytes parameter compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bytes(vec![0x61, 0x62]),
            Value::Symbol("TypeContractError".into())
        ])))
    );
}

#[test]
fn text_conversion_uses_scalars_when_the_text_contains_an_astral_character() {
    let given = r#"module M { public fun run() -> Array { "a\u{1f600}".to_array() } } M.run()"#;
    let when = run(&compile(given).expect("scalar conversion compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Text("a".into()),
            Value::Text("\u{1f600}".into())
        ])))
    );
}

#[test]
fn unknown_return_types_remain_errors_when_bytes_is_supported() {
    let given = "module M { public fun run() -> Missing { 1 } } M.run()";
    let when = run(&compile(given).expect("unknown annotation compiles"));
    assert_eq!(when, Err(MachineError::NameError));
}

#[test]
fn revision_subscription_places_callback_first_when_capacity_is_supplied() {
    let given = "Revision.subscribe(1) { |event| nil }";
    let when = run(&compile(given).expect("bounded subscription compiles"));
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn native_reduce_keeps_initial_value_when_a_trailing_block_is_supplied() {
    let given = "%[1, 2, 3].reduce(10) { |left, right| left + right }";
    let when = run(&compile(given).expect("reduce compiles"));
    assert_eq!(when, Ok(Value::Integer(16_u64.into())));
}

#[test]
fn bytes_generic_reification_is_refused_when_core_identity_is_unavailable() {
    let given = r#"class Box<T> {} Box<Bytes>.new()"#;
    let when = run(&compile(given).expect("generic Bytes syntax compiles"));
    assert_eq!(when, Err(MachineError::NameError));
}
