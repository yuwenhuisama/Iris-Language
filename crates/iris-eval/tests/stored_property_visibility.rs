use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ConstructionError, DispatchError, Value};

#[test]
fn external_read_is_denied_when_shorthand_is_private() {
    for (parameters, arguments) in [("", ""), ("<T>", "<Integer>")] {
        for visibility in ["", "private "] {
            let given = format!(
                "class Box{parameters} {{ {visibility}property held: Object = 7 }}; Box{arguments}.new().held"
            );
            let when = evaluate(&given);
            assert!(
                matches!(
                    when,
                    Err(EvaluationError::Construction(ConstructionError::Dispatch(
                        DispatchError::VisibilityDenied { .. }
                    )))
                ),
                "{given}: {when:?}"
            );
        }
    }
}

#[test]
fn private_write_preserves_storage_when_external_assignment_is_denied() {
    for (parameters, arguments) in [("", ""), ("<T>", "<Integer>")] {
        let given = format!(
            "class Box{parameters} {{ property held: Integer = 7; public fun read() {{ self.held }} }}; let box = Box{arguments}.new(); let denied = try {{ box.held = 9; false }} catch error {{ error == :MethodVisibilityError }}; denied && box.read() == 7"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{given}");
    }
}

#[test]
fn internal_methods_access_private_accessors_when_lexically_authorized() {
    let given = "class Box<T> { property held: T = 7; public fun change(value: T) -> T { self.held = value; self.held } }; Box<Integer>.new().change(9)";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(9u64.into())));
}

#[test]
fn public_shorthand_remains_public_when_no_accessor_block_is_written() {
    let given = "class Box<T> { public property held: T = 7 }; let box = Box<Integer>.new(); let written = box.held = 9; box.held";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(9u64.into())));
}

#[test]
fn private_setter_preserves_storage_when_getter_is_public() {
    let given = "class Box<T> { public property held: T = 7 { public get; private set; } }; let box = Box<Integer>.new(); let denied = try { box.held = 9; false } catch error { error == :MethodVisibilityError }; denied && box.held == 7";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn accessor_default_is_private_when_property_declaration_is_public() {
    let given = "class Box { public property held: Integer = 7 { get; public set; }; public fun read() { self.held } }; let box = Box.new(); let written = box.held = 9; let denied = try { box.held; false } catch error { error == :MethodVisibilityError }; denied && box.read() == 9";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn generic_setter_rejects_wrong_type_without_mutating_storage() {
    let given = "class Box<T> { public property held: T = 7 }; let box = Box<Integer>.new(); let denied = try { box.held = \"wrong\"; false } catch error { error == :TypeContractError }; denied && box.held == 7";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn declaration_visibility_is_local_when_public_property_precedes_private_property() {
    let given = "class Box { public property shown: Integer = 1; property hidden: Integer = 2 }; Box.new().hidden";
    let when = evaluate(given);
    assert!(matches!(
        when,
        Err(EvaluationError::Construction(ConstructionError::Dispatch(
            DispatchError::VisibilityDenied { .. }
        )))
    ));
}

#[test]
fn class_property_reads_and_writes_are_denied_when_private() {
    for (parameters, arguments, shared) in [
        ("", "", ""),
        ("<T>", "<Integer>", ""),
        ("<T>", "", "shared "),
    ] {
        let given = format!(
            "class Box{parameters} {{ {shared}class property held: Integer = 7; public class fun read() {{ Box{arguments}.held }} }}; let read_denied = try {{ Box{arguments}.held; false }} catch error {{ error == :MethodVisibilityError }}; let write_denied = try {{ Box{arguments}.held = 9; false }} catch error {{ error == :MethodVisibilityError }}; read_denied && write_denied && Box.read() == 7"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{given}");
    }
}

#[test]
fn class_accessor_policy_is_local_when_getter_is_public_and_setter_private() {
    for (parameters, arguments) in [("", ""), ("<T>", "<Integer>")] {
        let given = format!(
            "class Box{parameters} {{ public class property held: Integer = 7 {{ public get; private set; }} }}; let denied = try {{ Box{arguments}.held = 9; false }} catch error {{ error == :MethodVisibilityError }}; denied && Box{arguments}.held == 7"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{given}");
    }
}

#[test]
fn module_property_remains_private_when_read_outside_its_owner() {
    let given = "module Store { shared class property held: Integer = 7; public module fun read() { Store.held } }; let denied = try { Store.held; false } catch error { error == :MethodVisibilityError }; denied && Store.read() == 7";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn protected_accessor_allows_subclass_code_but_denies_external_code() {
    let given = "class Base { protected property held: Integer = 7 }; class Child extends Base { public fun read() { self.held } }; let child = Child.new(); let denied = try { child.held; false } catch error { error == :MethodVisibilityError }; denied && child.read() == 7";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn public_class_shorthand_keeps_read_write_access_for_closed_and_plain_owners() {
    for (parameters, arguments) in [("", ""), ("<T>", "<Integer>")] {
        let given = format!(
            "class Box{parameters} {{ public class property held: Integer = 7 }}; let written = Box{arguments}.held = 9; Box{arguments}.held"
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Integer(9u64.into())), "{given}");
    }
}
