use iris_eval::{EvaluationError, evaluate};
use iris_runtime::{ArrayRef, Value};

fn integers(values: &[u64]) -> Value {
    Value::Array(ArrayRef::new(
        values
            .iter()
            .map(|value| Value::Integer((*value).into()))
            .collect(),
    ))
}

#[test]
fn required_block_is_absent_when_only_a_positional_closure_is_supplied() {
    let given = "class Target { public fun value(callback: Closure<() -> Integer> = { || -> Integer; 100 }, &block: Block<() -> Integer>) { callback.call() + block.call() } }; Target.new().value({ || -> Integer; 42 })";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::ArgumentError));
}

#[test]
fn escaped_original_block_keeps_identity_and_shared_counter_when_forwarded() {
    let given = r#"
        class Saved { public class property block: Object = nil }
        class Save {}
        impl Save for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    Saved.block = invocation.original_block
                    next.call()
                    next.call()
                })
            }
        }
        class Target {
            @Save() public fun run(&block: Block<() -> Integer>) -> Integer {
                if !(block same? Saved.block) { raise :identity }
                block.call()
            }
        }
        mut counter = 0
        let result = Target.new().run() { || -> Integer; counter = counter + 1; counter }
        let escaped = Saved.block.call()
        %[result, escaped, counter]
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(integers(&[2, 3, 3])));
}

#[test]
fn helper_calls_keep_channels_separate_when_nested_inside_a_block_call() {
    let given = "class Target { public fun helper(callback: Closure<() -> Integer>) { callback.call() }; public fun run(callback: Closure<() -> Integer>, &block: Block<() -> Integer>) { self.helper(callback) + block.call() } }; let target = Target.new(); target.run({ || -> Integer; 7 }) { || -> Integer; target.helper({ || -> Integer; 5 }) }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(12u64.into())));
}

#[test]
fn identical_call_shapes_keep_distinct_channels_when_cloned_into_async_operands() {
    let given = r#"
        class Target { public fun run(callback: Closure<() -> Integer> = { || -> Integer; 20 }, &block: Block<() -> Integer> = nil) {
            if block == nil { callback.call() } else { block.call() + callback.call() }
        }
        public async fun scenario() { %[self.run({ || -> Integer; 7 }), self.run() { || -> Integer; 7 }] } }
        Host.run(Target.new().scenario())
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(integers(&[7, 27])));
}

#[test]
fn closure_parameter_channels_are_bound_when_callback_takes_a_separate_block() {
    let given = "let run = { |callback: Closure<() -> Integer>, &block: Block<() -> Integer>| -> Integer; callback.call() + block.call() }; run.call({ || -> Integer; 7 }) { || -> Integer; 5 }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(12u64.into())));
}
