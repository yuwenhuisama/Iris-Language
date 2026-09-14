use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ArrayRef, Value};

#[test]
fn ordered_applications_are_cached_when_owner_and_method_types_repeat() {
    let given = r#"
        class Effects { public class property phases: Integer = 0 }
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Effects.phases = Effects.phases + 1
                mut calls = 0
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    calls = calls + 1
                    next.call() * a[0] + calls
                })
            }
        }
        class Box<T> { @Wrap(10) @Wrap(2) public fun value<U>(owner: T, value: U) -> Integer { 0 } }
        let integer = Box<Integer>.new()
        let string = Box<String>.new()
        let first = integer.value<String>(1, "a")
        let owner = string.value<String>("s", "b")
        let method = integer.value<Integer>(2, 3)
        let repeat = integer.value<String>(4, "c")
        %[first, owner, method, repeat, Effects.phases]
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(
            [11u64, 11, 11, 22, 8]
                .into_iter()
                .map(|value| Value::Integer(value.into()))
                .collect()
        )))
    );
}

#[test]
fn retained_chain_keeps_original_when_generic_owner_is_reopened() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
            }
        }
        class Box<T> { @Wrap() public fun value(value: T) -> T { value } }
        let integer = Box<Integer>.new()
        let retained = integer.value
        let before = retained.call(1)
        open class Box<T> { @Wrap() public override fun value(value: T) -> T { raise :replacement } }
        let current = try { integer.value(2) } catch error { error == :replacement }
        %[before, current, retained.call(3)]
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1u64.into()),
            Value::Bool(true),
            Value::Integer(3u64.into()),
        ])))
    );
}

#[test]
fn existing_owner_bounds_reject_nil_before_decorated_method_entry() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation { Transformation.empty }
        }
        class Box<T> where T: NonNil { @Wrap() public fun value(value: T) -> T { value } }
        Box<Nil>.new().value(nil)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}
