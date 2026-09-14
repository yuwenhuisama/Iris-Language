use crate::{EvaluationError, Session};
use iris_runtime::{CallableKind, Value};
use iris_syntax::TypeExpression;

fn signature(kind: &str, parameter: TypeExpression) -> TypeExpression {
    TypeExpression::Generic {
        name: kind.into(),
        arguments: vec![TypeExpression::Function {
            parameters: vec![parameter],
            result: Box::new(TypeExpression::Name("Object".into())),
        }],
    }
}

#[test]
fn reification_preserves_publication_sequences_when_signature_is_interned() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Before { }; 0").unwrap();
    let before = given.evaluator.class_name("Before").unwrap().unwrap();
    let revision = given
        .evaluator
        .runtime
        .registry()
        .active(before)
        .unwrap()
        .clone();
    let when = given
        .evaluator
        .wrapper_type_value(&signature(
            "Closure",
            TypeExpression::Name("Integer".into()),
        ))
        .unwrap();
    let Value::Type(identity, arguments) = when else {
        panic!("Type expected")
    };
    assert!(arguments.is_empty());
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .callable_type(identity)
            .unwrap()
            .kind(),
        CallableKind::Closure
    );
    assert!(given.evaluator.runtime.registry().active(identity).is_err());
    given.evaluate("class After { }; 0").unwrap();
    let after = given.evaluator.class_name("After").unwrap().unwrap();
    assert_eq!(after.raw(), before.raw() + 1);
    let after = given.evaluator.runtime.registry().active(after).unwrap();
    assert_eq!(after.commit_id(), revision.commit_id() + 2);
    assert_eq!(after.id().raw(), revision.id().raw() + 2);
}

#[test]
fn source_aliases_are_canonical_when_callable_types_are_reified() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("type Number = Integer; (Closure<(Number) -> Object>).type == (Closure<(Integer) -> Object>).type");
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn normalized_composition_is_canonical_when_signature_order_differs() {
    let mut given = Session::new().unwrap();
    let first = signature(
        "Closure",
        TypeExpression::Union(vec![
            TypeExpression::Name("Integer".into()),
            TypeExpression::Name("String".into()),
        ]),
    );
    let second = signature(
        "Closure",
        TypeExpression::Union(vec![
            TypeExpression::Name("String".into()),
            TypeExpression::Name("Integer".into()),
        ]),
    );
    let when = given.evaluator.wrapper_type_value(&first).unwrap();
    assert_eq!(when, given.evaluator.wrapper_type_value(&second).unwrap());
}

#[test]
fn default_metadata_does_not_change_signature_when_closure_is_admitted() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("let callback = { |value: Integer = 7| -> Integer; value }; callback is? Closure<(Integer) -> Integer>");
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn kind_and_result_are_invariant_when_block_is_tested() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("let callback = { |value: Integer| -> Integer; value }; 0")
        .unwrap();
    assert_eq!(
        given.evaluate("callback is? Block<(Integer) -> Integer>"),
        Ok(Value::Bool(true))
    );
    assert_eq!(
        given.evaluate("callback is? Closure<(Integer) -> Object>"),
        Ok(Value::Bool(false))
    );
    assert_eq!(
        given.evaluate("callback is? Block<(Integer) -> Object>"),
        Ok(Value::Bool(false))
    );
}

#[test]
fn block_metadata_matches_source_type_when_signature_is_reflected() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("class Probe {}
    impl Probe for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                if invocation.signature.parameters[0].type != (Block<(Integer) -> Object>).type { raise :metadata }
                if !invocation.signature.parameters[0].optional { raise :optional }
                next.call()
            })
        }
    } class Target { @Probe() public fun value(&body: Block<(Integer) -> Object> = nil) -> Integer { 7 } }; Target.new().value()");
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn logical_metadata_is_available_when_callable_has_no_active_class() {
    let mut given = Session::new().unwrap();
    assert_eq!(
        given.evaluate("(Block<(Integer) -> Object>).type.kind"),
        Ok(Value::Symbol("union".into()))
    );
    assert_eq!(
        given.evaluate("(Closure<(Integer) -> Object>).type.kind"),
        Ok(Value::Symbol("closure".into()))
    );
    assert_eq!(
        given.evaluate("(Closure<(Integer) -> Object>).type.new()"),
        Err(EvaluationError::UnsupportedConstruct)
    );
}

#[test]
fn block_channel_preserves_kind_when_callback_is_reified() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("class Target { public fun value(&body: Block<() -> Integer>) { body is? Block<() -> Integer> } }; Target.new().value() { || -> Integer; 7 }");
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn exact_cast_preserves_callable_when_signature_matches() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("let callback = { |value: Integer| -> Integer; value }; let exact = callback as Closure<(Integer) -> Integer>; exact.call(7)");
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
    assert_eq!(
        given.evaluate("callback as? Closure<(Integer) -> Object>"),
        Ok(Value::Nil)
    );
}

#[test]
fn async_callable_signature_differs_when_task_result_is_reified() {
    let mut given = Session::new().unwrap();
    given
        .evaluate("let callback = { async || -> Integer; 7 }; 0")
        .unwrap();
    let when = given.evaluate("callback.type == (Closure<() -> Task<Integer>>).type");
    assert_eq!(when, Ok(Value::Bool(true)));
    assert_eq!(
        given.evaluate("callback.type == (Closure<() -> Task<Object>>).type"),
        Ok(Value::Bool(false))
    );
}

#[test]
fn nested_callable_parameters_are_canonical_when_reified_twice() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("type Callback = Closure<(Integer) -> Object>; (Closure<(Callback) -> Object>).type == (Closure<(Closure<(Integer) -> Object>) -> Object>).type");
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn original_closure_kind_is_preserved_when_bound_to_block_channel() {
    let mut given = Session::new().unwrap();
    assert_eq!(
        given.evaluate("let callback = { || -> Integer; 7 }; class Target { public fun value(&body: Block<() -> Integer>) { body is? Block<() -> Integer> } }; Target.new().value() { || -> Integer; callback.call() }") ,
        Ok(Value::Bool(true))
    );
    assert_eq!(
        given.evaluate("callback is? Closure<() -> Integer> && callback is? Block<() -> Integer>"),
        Ok(Value::Bool(true))
    );
    assert_eq!(
        given.evaluate("Target.new().value(callback)"),
        Err(EvaluationError::ArgumentError)
    );
}

#[test]
fn mismatched_callable_return_is_rejected_when_method_returns_closure() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("class Factory { public fun make() -> Closure<(Integer) -> Object> { { |value: Integer| -> Integer; value } } }; Factory.new().make()");
    assert!(
        matches!(when, Err(EvaluationError::TypeContractError)),
        "{when:?}"
    );
}
