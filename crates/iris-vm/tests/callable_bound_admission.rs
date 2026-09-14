use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn callable_bounds_accept_normalized_equivalent_signatures_directly_and_through_aliases() {
    for (bound, argument) in [
        (
            "Closure<(String | Integer) -> Integer>",
            "Closure<(Integer | String) -> Integer>",
        ),
        (
            "Closure<(Integer | String) -> Integer>",
            "Closure<(String | Integer) -> Integer>",
        ),
        (
            "Closure<(Integer & NonNil) -> Integer>",
            "Closure<(Integer) -> Integer>",
        ),
        (
            "Closure<(Integer) -> Integer>",
            "Closure<(Integer & NonNil) -> Integer>",
        ),
        (
            "Closure<(Object | Integer) -> Integer>",
            "Closure<(Object) -> Integer>",
        ),
        (
            "Closure<(Integer & Object) -> Integer>",
            "Closure<(Integer) -> Integer>",
        ),
        (
            "Closure<(Closure<(String | Integer) -> Integer>) -> Integer>",
            "Closure<(Closure<(Integer | String) -> Integer>) -> Integer>",
        ),
        (
            "Closure<(String) -> Integer>",
            "Closure<((String | Nil) & NonNil) -> Integer>",
        ),
        (
            "Closure<((String | Nil) & NonNil) -> Integer>",
            "Closure<(String) -> Integer>",
        ),
        (
            "Closure<(NonNil) -> Integer>",
            "Closure<(NonNil & NonNil) -> Integer>",
        ),
        (
            "Closure<(NonNil & NonNil) -> Integer>",
            "Closure<(NonNil) -> Integer>",
        ),
    ] {
        for construction in [
            format!("Box<{argument}>.new()"),
            format!("let target = Box<{argument}>; target.new()"),
        ] {
            let given = format!("class Box<T> where T: {bound} {{}}; {construction}");

            let when = run(&compile(&given).expect("normalized callable bound source compiles"));

            assert!(when.is_ok(), "{bound}: {construction}: {when:?}");
        }
    }
}

#[test]
fn callable_bounds_reject_invariant_signature_and_kind_differences() {
    for (bound, argument) in [
        (
            "Closure<(String | Integer) -> Integer>",
            "Closure<(String) -> Integer>",
        ),
        (
            "Closure<(String | Integer) -> Integer>",
            "Closure<(String | Integer) -> String>",
        ),
        (
            "Closure<(String | Integer) -> Integer>",
            "BoundMethod<(String | Integer) -> Integer>",
        ),
        (
            "Closure<(String | Integer | Nil) -> Integer>",
            "Closure<((String & Integer) | Nil) -> Integer>",
        ),
        (
            "Closure<((String & Integer) | Nil) -> Integer>",
            "Closure<(String | Integer | Nil) -> Integer>",
        ),
        (
            "Closure<(Object) -> Integer>",
            "Closure<(Object & NonNil) -> Integer>",
        ),
        (
            "Closure<(Object & NonNil) -> Integer>",
            "Closure<(Object) -> Integer>",
        ),
    ] {
        for construction in [
            format!("Box<{argument}>.new()"),
            format!("let target = Box<{argument}>; target.new()"),
        ] {
            let given = format!(
                "class Box<T> where T: {bound} {{}}; try {{ {construction}; false }} catch error {{ error == :TypeContractError }}"
            );

            let when = run(&compile(&given).expect("invariant callable bound source compiles"));

            assert_eq!(when, Ok(Value::Bool(true)), "{bound}: {construction}");
        }
    }
}

#[test]
fn composed_type_identity_preserves_normalized_structure() {
    for (given, expected) in [
        (
            "(String | Integer).type same? (String & Integer).type",
            false,
        ),
        ("Object.type same? (Object & NonNil).type", false),
        ("((String | Nil) & NonNil).type same? String.type", true),
        ("(String | String).type same? String.type", true),
        ("(String & String).type same? String.type", true),
        ("(Nil & NonNil).type same? (Never).type", true),
        (
            "let compact = (String & (Integer | Nil)).type; compact.kind == :intersection && compact.members.length() == 2 && (compact.members[0] same? (Integer | Nil).type || compact.members[1] same? (Integer | Nil).type)",
            true,
        ),
    ] {
        let when = run(&compile(given).expect("composed Type identity source compiles"));

        assert_eq!(when, Ok(Value::Bool(expected)), "{given}");
    }
}

#[test]
fn composed_identity_collapses_markers_and_singletons_when_nil_is_removed()
-> Result<(), iris_vm::CompileError> {
    for (given, expected) in [
        ("NonNil & NonNil", "NonNil"),
        ("NonNil & NonNil & NonNil", "NonNil"),
        ("Nil & NonNil & String", "Never"),
        ("(String | Integer | Nil) & NonNil", "String | Integer"),
        ("((String & Integer) | Nil) & NonNil", "String & Integer"),
        ("((Object & NonNil) | Nil) & NonNil", "Object & NonNil"),
        ("(String & Integer) | Never", "String & Integer"),
        (
            "(String | Integer) & (String | Integer)",
            "String | Integer",
        ),
        ("(Object | Nil) & NonNil", "Object & NonNil"),
        ("String & NonNil & Object", "String"),
        ("NonNil & Object", "Object & NonNil"),
        (
            "((String & Integer) | Nil) | ((Integer & String) | Nil)",
            "Nil | (String & Integer)",
        ),
    ] {
        let source = format!("({given}).type same? ({expected}).type");
        let program = compile(&source)?;

        let when = run(&program);

        assert_eq!(when, Ok(Value::Bool(true)), "{source}");
    }
    Ok(())
}

#[test]
fn composed_identity_stays_compact_when_unions_contain_intersections()
-> Result<(), iris_vm::CompileError> {
    let given = r#"
let compact = ((String & Integer) | Nil).type
compact.kind == :union && compact.members.length() == 2 &&
(compact.members[0] same? (String & Integer).type || compact.members[1] same? (String & Integer).type) &&
!(compact same? (String | Integer | Nil).type) &&
!((String & (Integer | Nil)).type same? ((String & Integer) | (String & Nil)).type)
"#;
    let program = compile(given)?;

    let when = run(&program);

    assert_eq!(when, Ok(Value::Bool(true)));
    Ok(())
}

#[test]
fn composed_admission_checks_every_nested_intersection_member_when_testing_values()
-> Result<(), iris_vm::CompileError> {
    let given = r#"
let mixed = ((String & Integer) | Nil).type
let present = (Object & NonNil).type
(nil is? mixed) && !(1 is? mixed) && !("text" is? mixed) &&
!(nil is? present) && (1 is? present)
"#;
    let program = compile(given)?;

    let when = run(&program);

    assert_eq!(when, Ok(Value::Bool(true)));
    Ok(())
}
