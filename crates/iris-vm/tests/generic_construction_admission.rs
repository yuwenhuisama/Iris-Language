use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn construction_rejects_when_closed_bound_or_arity_is_invalid() {
    for (declarations, construction) in [
        (
            "contract Comparable<T> {}; class Key {}; impl Key for Comparable<Object> {}; class Box<T> where T: Comparable<T>",
            "Box<Key>",
        ),
        (
            "contract Comparable<T> {}; class Key {}; impl Key for Comparable {}; class Box<T> where T: Comparable<T>",
            "Box<Key>",
        ),
        (
            "contract Comparable<T> {}; class Key<T> {}; impl Key<T> for Comparable<Key<String>> {}; class Box<T> where T: Comparable<T>",
            "Box<Key<Integer>>",
        ),
        (
            "class Base {}; class Other {}; class Box<T> where T: Base",
            "Box<Other>",
        ),
        ("class Box<T, U>", "Box<Integer>"),
        ("class Box<T, U>", "Box<Integer, String, Nil>"),
        ("class Box<T, U>", "Box"),
        ("class Inner<T> {}; class Box<T>", "Box<Inner>"),
        (
            "class Base {}; class Other {}; class Box<T, U> where U: T & NonNil",
            "Box<Base, Other>",
        ),
        (
            "contract Relation<T> {}; class Box<T> where T: Relation<T>",
            "Box<Unknown>",
        ),
    ] {
        let given = format!(
            "class Effects {{ public class property count: Integer = 0 }}; {declarations} {{ public property mark: Integer = Effects.count = Effects.count + 1; public fun initialize(value) {{ Effects.count = Effects.count + 1 }} }}; let rejected = try {{ {construction}.new(7); false }} catch error {{ error == :TypeContractError }}; rejected && Effects.count == 0"
        );
        let program = compile(&given).expect("admission source compiles");
        let when = run(&program);
        assert_eq!(
            when,
            Ok(Value::Bool(true)),
            "{declarations}: {construction}"
        );
    }
}

#[test]
fn construction_accepts_when_full_substitution_satisfies_bounds() {
    for (declarations, construction) in [
        (
            "contract Comparable<T> {}; class Key {}; impl Key for Comparable<Key> {}; class Box<T> where T: Comparable<T>",
            "Box<Key>",
        ),
        (
            "contract Relation<T> {}; class Key {}; impl Key for Relation<Integer> {}; class Box<T> where T: Relation<Integer>",
            "Box<Key>",
        ),
        (
            "contract Comparable<T> {}; class Key<T> {}; impl Key<T> for Comparable<Key<T>> {}; class Box<T> where T: Comparable<T>",
            "Box<Key<Integer>>",
        ),
        (
            "class Base {}; class Child extends Base {}; class Box<T> where T: Base",
            "Box<Child>",
        ),
        ("class Box<T, U>", "Box<Integer, Nil>"),
        ("class Box<T> where T: Object", "Box<Nil>"),
        (
            "class Base {}; class Child extends Base {}; class Box<T, U> where U: T & NonNil",
            "Box<Base, Child>",
        ),
        (
            "contract Relation<T> {}; class Base {}; impl Base for Relation<Integer> {}; class Child extends Base {}; class Box<T> where T: Relation<Integer>",
            "Box<Child>",
        ),
    ] {
        let given = format!(
            "{declarations} {{ public property held: Integer; public fun initialize(value: Integer) {{ @held = value }} }}; {construction}.new(42).held == 42"
        );
        let program = compile(&given).expect("admission source compiles");
        let when = run(&program);
        assert_eq!(
            when,
            Ok(Value::Bool(true)),
            "{declarations}: {construction}"
        );
    }
}

#[test]
fn definition_metadata_survives_when_generic_arguments_are_unapplied() {
    let given = "class Box<T, U> {}; let definition = Box; definition same? Box";
    let program = compile(given).expect("metadata source compiles");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn constructor_parameter_count_is_checked_when_generic_arguments_are_valid() {
    let given = "class Effects { public class property count: Integer = 0 }; class Box<T> { public fun initialize(value) { Effects.count = Effects.count + 1 } }; let rejected = try { Box<Integer>.new(); false } catch error { true }; rejected && Effects.count == 0";
    let program = compile(given).expect("arity source compiles");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn aliased_generic_definition_rejects_before_construction_effects() {
    let given = "mut effects = 0; class Box<T> { public property mark: Integer = effects = effects + 1; public fun initialize() { effects = effects + 1 } }; let raw = Box; let rejected = try { raw.new(); false } catch error { true }; rejected && effects == 0";
    let program = compile(given).expect("alias source compiles");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn aliased_closed_generic_constructs_with_exact_arguments() {
    let given = "class Box<T> { public property held: T; public fun initialize(value: T) { @held = value } }; let closed = Box<String>; closed.new(\"iris\").held";
    let program = compile(given).expect("closed alias source compiles");
    let when = run(&program);
    assert_eq!(when, Ok(Value::Text("iris".into())));
}

#[test]
fn valid_closed_alias_constructs_with_nested_owner_bindings() {
    let given = "class Inner<T> {}; class Box<T> { public property held: T; public fun initialize(value: T) { @held = value } }; let closed = Box<Inner<Integer>>; let value = Inner<Integer>.new(); closed.new(value).held same? value";

    let program = compile(given).expect("closed alias source compiles");
    let when = run(&program);

    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn callable_signature_types_construct_directly_and_through_aliases() {
    for callable in ["Closure<() -> Integer>", "BoundMethod<(String) -> Integer>"] {
        for body in [
            format!("let value = Box<{callable}>.new(); value is? Box<{callable}>"),
            format!(
                "let closed = Box<{callable}>; let value = closed.new(); value is? Box<{callable}>"
            ),
        ] {
            let given = format!("class Box<T> {{}}; {body}");

            let when = run(&compile(&given).expect("callable argument source compiles"));

            assert_eq!(when, Ok(Value::Bool(true)), "{callable}: {body}");
        }
    }
}

#[test]
fn malformed_callable_generic_shapes_reject_at_construction_admission() {
    for callable in [
        "Closure",
        "Closure<Integer>",
        "Closure<() -> Integer, () -> Integer>",
        "BoundMethod",
        "Block<Integer>",
    ] {
        let given = format!("class Box<T> {{}}; Box<{callable}>.new()");

        let when = run(&compile(&given).expect("malformed callable shape parses"));

        assert_eq!(
            when,
            Err(iris_vm::MachineError::ParseDiagnostic),
            "{callable}"
        );
    }
}

#[test]
fn raw_generic_construction_evaluates_operands_once_before_rejection() {
    for construction in [
        "Box.new(Effects.bump())",
        "let raw = Box; raw.new(Effects.bump())",
    ] {
        let given = format!(
            "class Effects {{ public class property operands: Integer = 0; public class property construction: Integer = 0; public class fun bump() -> Integer {{ Effects.operands = Effects.operands + 1; 7 }} }}; class Box<T> {{ public property mark: Integer = Effects.construction = Effects.construction + 1; public fun initialize(value) {{ Effects.construction = Effects.construction + 1 }} }}; let rejected = try {{ {construction}; false }} catch error {{ error == :TypeContractError }}; rejected && Effects.operands == 1 && Effects.construction == 0"
        );

        let when = run(&compile(&given).expect("raw construction source compiles"));

        assert_eq!(when, Ok(Value::Bool(true)), "{construction}");
    }
}

#[test]
fn raw_generic_operand_raise_precedes_admission_and_constructor_defaults() {
    for construction in [
        "Box.new(Effects.fail())",
        "let raw = Box; raw.new(Effects.fail())",
    ] {
        let given = format!(
            "class Effects {{ public class property operands: Integer = 0; public class property construction: Integer = 0; public class fun fail() {{ Effects.operands = Effects.operands + 1; raise :operand }} public class fun default() -> Integer {{ Effects.construction = Effects.construction + 1; 7 }} }}; class Box<T> {{ public property mark: Integer = Effects.construction = Effects.construction + 1; public fun initialize(value = Effects.default()) {{ Effects.construction = Effects.construction + 1 }} }}; let observed = try {{ {construction}; :none }} catch error {{ error }}; observed == :operand && Effects.operands == 1 && Effects.construction == 0"
        );

        let when = run(&compile(&given).expect("raising operand source compiles"));

        assert_eq!(when, Ok(Value::Bool(true)), "{construction}");
    }
}

#[test]
fn invalid_closed_construction_evaluates_operands_before_admission() {
    let given = "class Effects { public class property operands: Integer = 0; public class property construction: Integer = 0; public class fun bump() -> Integer { Effects.operands = Effects.operands + 1; 7 } }; class Base {}; class Other {}; class Box<T> where T: Base { public property mark: Integer = Effects.construction = Effects.construction + 1; public fun initialize(value) { Effects.construction = Effects.construction + 1 } }; let rejected = try { Box<Other>.new(Effects.bump()); false } catch error { error == :TypeContractError }; rejected && Effects.operands == 1 && Effects.construction == 0";

    let when = run(&compile(given).expect("invalid closed construction compiles"));

    assert_eq!(when, Ok(Value::Bool(true)));
}
