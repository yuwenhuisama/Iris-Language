#![expect(clippy::expect_used, reason = "tests require compiled source")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

fn evaluate(source: &str) -> Result<Value, MachineError> {
    run(&compile(source).expect("generic property source compiles"))
}

#[test]
fn public_storage_is_separate_when_owners_close_differently() {
    let given = r#"
class Box<T> { public class property made: Integer = 0; public property held: Object = nil }
let made_text = Box<String>.made = 2
let made_integer = Box<Integer>.made = 7
let text = Box<String>.new()
let integer = Box<Integer>.new()
let set_text = text.held = "text"
let set_integer = integer.held = 42
%[Box<String>.made, Box<Integer>.made, text.held, integer.held]
"#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(2_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Text("text".into()),
            Value::Integer(42_u64.into()),
        ])))
    );
}

#[test]
fn typed_storage_uses_owner_when_getters_and_setters_alternate() {
    let given = r#"
class Box<T> { public property held: T; public fun initialize(value: T) { @held = value } }
let text = Box<String>.new("old")
let integer = Box<Integer>.new(1)
let set_text = text.held = "text"
let set_integer = integer.held = 42
text.held == "text" && integer.held == 42
"#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn computed_accessors_use_owner_when_types_alternate() {
    let given = r#"
class Box<T> {
 public fun initialize(value: T) { @held = value }
 public property fun held() -> T { @held }
 public property fun held=(value: T) -> T { @held = value }
}
let text = Box<String>.new("old")
let integer = Box<Integer>.new(1)
let set_text = text.held = "text"
let set_integer = integer.held = 42
text.held == "text" && integer.held == 42
"#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn wrong_setter_type_has_zero_effect_when_owner_is_closed() {
    for declaration in [
        "public property held: T",
        "public property fun held() -> T { @held }; public property fun held=(value: T) -> T { Effects.count = Effects.count + 1; @held = value }",
    ] {
        let given = format!(
            r#"
class Effects {{ public class property count: Integer = 0 }}
class Box<T> {{ {declaration}; public fun initialize(value: T) {{ @held = value }} }}
let text = Box<String>.new("old")
let integer = Box<Integer>.new(7)
let bad_text = try {{ text.held = 42; false }} catch error {{ error == :TypeContractError }}
let bad_integer = try {{ integer.held = "wrong"; false }} catch error {{ error == :TypeContractError }}
bad_text && bad_integer && text.held == "old" && integer.held == 7 && Effects.count == 0
"#
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{declaration}");
    }
}

#[test]
fn getter_guard_is_retained_when_raw_storage_has_wrong_type() {
    let given = "class Box<T> { public property held: T; public fun initialize(value: Object) { @held = value } }; let text = Box<String>.new(42); try { text.held; false } catch error { error == :TypeContractError }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn compatible_computed_override_keeps_owner_and_storage_when_reopened() {
    let given = r#"
class Effects { public class property count: Integer = 0 }
class Box<T> { public property held: T; public fun initialize(value: T) { @held = value } }
let text = Box<String>.new("text")
let integer = Box<Integer>.new(42)
let before = text.held == "text" && integer.held == 42
open class Box {
 public override property fun held() -> T { Effects.count = Effects.count + 1; @held }
 public override property fun held=(value: T) -> T { Effects.count = Effects.count + 1; @held = value }
}
let set_text = text.held = "new"
let set_integer = integer.held = 7
let rejected = try { text.held = 42; false } catch error { error == :TypeContractError }
before && rejected && text.held == "new" && integer.held == 7 && Effects.count == 4
"#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn shorthand_is_private_when_declaration_omits_visibility() {
    for owner in ["Box", "Box<T>"] {
        let closed = if owner == "Box" {
            "Box"
        } else {
            "Box<Integer>"
        };
        let given = format!(
            "class {owner} {{ property held: Object = 7; public fun read() {{ self.held }}; public fun write(value) {{ self.held = value }} }}; let box = {closed}.new(); let denied_read = try {{ box.held; false }} catch error {{ error == :MethodVisibilityError }}; let denied_write = try {{ box.held = 9; false }} catch error {{ error == :MethodVisibilityError }}; let internal = box.write(8); denied_read && denied_write && internal == 8 && box.read() == 8"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{owner}");
    }
}

#[test]
fn explicit_accessor_visibility_wins_when_declaration_is_public() {
    let given = "class Box<T> { public property held: Object = 7 { get; public set; }; public fun read() { self.held } }; let box = Box<Integer>.new(); let set = box.held = 9; let denied = try { box.held; false } catch error { error == :MethodVisibilityError }; denied && box.read() == 9";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn explicit_public_accessors_work_when_declaration_is_private() {
    let given = "class Box<T> { property held: T { public get; public set; }; public fun initialize(value: T) { @held = value } }; let box = Box<Integer>.new(7); let set = box.held = 9; box.held == 9";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn owner_annotation_survives_when_computed_getter_returns_wrong_type() {
    let given = "class Box<T> { public property fun held() -> T { 42 } }; let integer = Box<Integer>.new(); let text = Box<String>.new(); let rejected = try { text.held; false } catch error { error == :TypeContractError }; integer.held == 42 && rejected";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}
