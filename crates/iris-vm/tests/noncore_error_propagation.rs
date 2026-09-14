#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests inspect VM propagation"
)]

use iris_runtime::Value;
use iris_vm::{MachineError, compile, run};

fn evaluate(source: &str) -> Result<Value, MachineError> {
    run(&compile(source).expect("source compiles"))
}

#[test]
fn semantic_error_survives_when_nested_cleanup_unwinds() {
    for (body, expected) in [
        (
            "let values = %[1, 2]; for item in values { values[0] = 9 }",
            MachineError::ConcurrentModification,
        ),
        (
            "let values = %{:a: 1}; for item in values { values[:b] = 2 }",
            MachineError::ConcurrentModification,
        ),
        (
            "for item in %[1, 2] { item = 9 }",
            MachineError::ImmutableBinding,
        ),
        (
            "try { raise :x } catch error, context { context.suppressed.to_string() }",
            MachineError::MessageNotFound {
                receiver_class: "ReadonlyArray".into(),
                selector: "to_string".into(),
            },
        ),
        (
            "try { for item in 5 { item } } finally { nil }",
            MachineError::MessageNotFound {
                receiver_class: "Integer".into(),
                selector: "iterator".into(),
            },
        ),
    ] {
        for given in [
            body.to_owned(),
            format!(
                "module Run {{ public fun inner() {{ {body} }}; public fun outer() {{ inner() }} }}; Run.outer()"
            ),
        ] {
            let when = evaluate(&given);
            assert_eq!(when, Err(expected.clone()), "{given}");
        }
    }
}

#[test]
fn semantic_error_survives_when_caught_and_bare_rethrown() {
    let given = "module Run { public fun inner() { try { %[1].size() } catch error { raise } }; public fun outer() { try { inner() } finally { nil } } }; Run.outer()";
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(MachineError::MessageNotFound {
            receiver_class: "Array".into(),
            selector: "size".into()
        })
    );
}

#[test]
fn saved_context_survives_when_nested_failure_is_caught_again() {
    let given = "class State { public class property context: Object = nil }; module Run { public fun inner() { try { %[1].size() } catch error, context { State.context = context; raise } }; public fun outer() { try { inner() } finally { nil } } }; try { Run.outer() } catch error, context { error == :MessageNotFound && context.value == error && context same? State.context && context.re_raise_sites.length == 1 }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn user_symbol_stays_raised_when_a_same_name_failure_was_handled() {
    for (body, name) in [
        ("%[1].size()", "MessageNotFound"),
        (
            "let values = %[1, 2]; for item in values { values[0] = 9 }",
            "ConcurrentModificationError",
        ),
        ("for item in %[1] { item = 2 }", "ImmutableBindingError"),
    ] {
        let given = format!(
            "module Run {{ public fun inner() {{ try {{ {body} }} catch error {{ nil }}; try {{ raise :{name} }} catch error {{ raise }} }} }}; Run.inner()"
        );
        let when = evaluate(&given);
        let Err(MachineError::Raised(payload)) = when else {
            panic!("expected user propagation: {when:?}")
        };
        assert_eq!(payload.0, Value::Symbol(name.into()));
        let Value::ExceptionContext(_, value, _, _, sites, _) = payload.1 else {
            panic!("expected context")
        };
        assert_eq!(*value, payload.0);
        assert_eq!(sites.len(), 1);
    }
}

#[test]
fn explicit_raise_is_fresh_when_caught_semantic_value_is_raised_again() {
    let given = "module Run { public fun inner() { try { %[1].size() } catch error { raise error } } }; Run.inner()";
    let when = evaluate(given);
    assert!(
        matches!(when, Err(MachineError::Raised(payload)) if payload.0 == Value::Symbol("MessageNotFound".into()))
    );
}

#[test]
fn outer_semantic_failure_survives_when_cleanup_handles_inner_failure() {
    let given = "module Run { public fun inner() { try { %[1].size() } finally { try { for item in %[1] { item = 2 } } catch error { nil } } } }; Run.inner()";
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(MachineError::MessageNotFound {
            receiver_class: "Array".into(),
            selector: "size".into()
        })
    );
}
