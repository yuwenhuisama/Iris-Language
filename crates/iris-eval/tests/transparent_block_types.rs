use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

#[test]
fn block_is_transparent_when_compared_with_explicit_union() {
    let given = "(Block<(Integer) -> Object>).type == (BoundMethod<(Integer) -> Object> | Closure<(Integer) -> Object>).type";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn callable_types_are_exact_when_bound_and_closure_are_reified() {
    let given = r#"
        class Receiver { public fun value(value: Integer = 7) -> Integer { value } }
        let bound = Receiver.new().value
        let closure = { |value: Integer = 7| -> Integer; value }
        bound.type == (BoundMethod<(Integer) -> Integer>).type &&
        bound.type.kind == :boundmethod &&
        closure.type == (Closure<(Integer) -> Integer>).type &&
        !(closure is? BoundMethod<(Integer) -> Integer>) &&
        !(bound is? Closure<(Integer) -> Integer>) &&
        bound is? Block<(Integer) -> Integer> && closure is? Block<(Integer) -> Integer> &&
        !(bound is? Block<(Integer) -> Object>)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn casts_preserve_identity_when_block_union_accepts_either_callable() {
    for callable in ["Receiver.new().value", "{ || -> Integer; 7 }"] {
        let given = format!(
            r#"
            class Receiver {{ public fun value() -> Integer {{ 7 }} }}
            let callable = {callable}
            let exact = callable as Block<() -> Integer>;
            (exact same? callable) && (callable as? Block<() -> Object>) == nil
        "#
        );
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{callable}");
    }
}

#[test]
fn optional_block_admits_nil_when_reified_as_union() {
    let given = "let optional = (Block<() -> Integer> | Nil).type; (nil is? optional) && !(nil is? Block<() -> Integer>)";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn rest_elements_reject_wrong_callable_when_parameter_collects_arguments() {
    for (parameter, argument) in [
        ("*callbacks: Block<() -> Integer>", "callback"),
        ("**callbacks: BoundMethod<() -> Integer>", "wrong: callback"),
    ] {
        let given = format!(
            "class Receiver {{ public fun value() -> Object {{ 7 }} }};
             class Target {{ public fun value({parameter}) {{ raise :entered }} }};
             let callback = Receiver.new().value; Target.new().value({argument})"
        );
        let when = evaluate(&given);
        assert_eq!(when, Err(EvaluationError::TypeContractError));
    }
}

#[test]
fn ordinary_block_rejects_wrong_signature_when_body_would_ignore_it() -> Result<(), EvaluationError>
{
    for callable in [
        "Wrong.new().value",
        "{ || -> Object; 7 }",
        "{ |value: Integer| -> Integer; value }",
        "{ async || -> Integer; 7 }",
    ] {
        let mut given = Session::new()?;
        given.evaluate(&format!(
            r#"
            class Wrong {{ public fun value() -> Object {{ 7 }} }}
            class Target {{
                public class property entered: Integer = 0
                public fun value(&block: Block<() -> Integer>) -> Integer {{
                    Target.entered = 1; 7
                }}
            }}
            let callback = {callable}; 0
        "#
        ))?;
        let when = given.evaluate("Target.new().value(&callback)");
        assert_eq!(when, Err(EvaluationError::TypeContractError), "{callable}");
        assert_eq!(
            given.evaluate("Target.entered"),
            Ok(Value::Integer(0u64.into()))
        );
    }
    Ok(())
}

#[test]
fn bound_signature_wraps_result_when_method_is_async() {
    let given = r#"
        class Receiver { public async fun value() -> Integer { 7 } }
        let bound = Receiver.new().value
        bound.type == (BoundMethod<() -> Task<Integer>>).type &&
        bound is? Block<() -> Task<Integer>> && !(bound is? Block<() -> Integer>)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn bound_result_contract_rejects_mismatch_when_method_returns_bound_value() {
    let given = r#"
        class Receiver { public fun value() -> Integer { 7 } }
        class Factory {
            public fun make() -> BoundMethod<() -> Object> { Receiver.new().value }
        }
        Factory.new().make()
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn next_keeps_signature_when_optional_invocation_metadata_allows_omission() {
    let given = r#"
        class Probe {}
        impl Probe for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    if next.type != (Closure<(ArgumentChanges) -> Object>).type { raise :signature }
                    if !(next is? Block<(ArgumentChanges) -> Object>) { raise :block }
                    if next is? BoundMethod<(ArgumentChanges) -> Object> { raise :kind }
                    next.call()
                })
            }
        }
        class Target { @Probe() public fun value(&body: Block<() -> Integer> = nil) -> Integer { 7 } }
        Target.new().value()
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}
