use iris_eval::evaluate;
use iris_runtime::{ArrayRef, Value};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan {
        if d.signature.parameters[0].type.kind != :parameter { raise :open }
        Plan.empty
    }
    public fun transform(d, a, c) -> Transformation {
        if c.reason == :closed_materialization && d.owner != Box { raise :logical_owner }
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if invocation.owner_type_arguments[0] != invocation.signature.result { raise :owner }
            if invocation.method_type_arguments.length != 0 { raise :method }
            if invocation.signature.parameters[0].type != invocation.signature.result { raise :signature }
            if invocation.signature.result == Integer.type && invocation.slot[0] != Box<Integer>.type { raise :slot }
            if invocation.signature.result == String.type && invocation.slot[0] != Box<String>.type { raise :slot }
            if invocation.signature.result == Integer.type { calls } else { next.call() }
        })
    }
}
"#;

#[test]
fn captures_are_isolated_when_closed_owners_alternate() {
    let given = format!(
        r#"{WRAP}
        class Box<T> {{ @Wrap() public fun value(value: T) -> T {{ value }} }}
        %[Box<Integer>.new().value(7), Box<String>.new().value("s"), Box<Integer>.new().value(9)]
    "#
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1u64.into()),
            Value::Text("s".into()),
            Value::Integer(2u64.into()),
        ])))
    );
}

#[test]
fn bindings_are_separate_when_method_parameters_shadow_owner_names() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    if invocation.owner_type_arguments[0] != Integer.type { raise :owner }
                    if invocation.method_type_arguments[0] != String.type { raise :method }
                    if invocation.signature.result != String.type { raise :result }
                    next.call()
                })
            }
        }
        class Box<T> {
            @Wrap() public fun value<U>(owner: T, value: U) -> U { value }
            @Wrap() public fun shadow<T>(value: T) -> T { value }
        }
        let box = Box<Integer>.new()
        %[box.value<String>(1, "a"), box.shadow<String>("b")]
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Text("a".into()),
            Value::Text("b".into())
        ])))
    );
}

#[test]
fn wrong_patch_is_rejected_when_owner_signature_is_closed() {
    let given = r#"
        class Patch {}
        impl Patch for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    next.call(ArgumentChanges.new(positional: %{:value: "wrong"}))
                })
            }
        }
        class Box<T> { @Patch() public fun value(value: T) -> T { raise :body } }
        try { Box<Integer>.new().value(1) } catch error { error is? TypeError }
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn wrong_input_is_rejected_when_owner_wrapper_would_short_circuit() {
    let given = format!(
        r#"{WRAP}
        class Box<T> {{ @Wrap() public fun value(value: T) -> T {{ raise :body }} }}
        let input: Object = "wrong"
        try {{ Box<Integer>.new().value(input) }} catch error {{ error is? TypeError }}
    "#
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn wrong_result_is_rejected_when_owner_wrapper_skips_original() {
    let given = r#"
        class Wrong {}
        impl Wrong for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; "wrong" })
            }
        }
        class Box<T> { @Wrong() public fun value(value: T) -> T { raise :body } }
        try { Box<Integer>.new().value(1) } catch error { error is? TypeError }
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn async_owner_context_is_preserved_when_calls_suspend_together() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                mut calls = 0
                Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                    calls = calls + 1
                    let result = await next.call()
                    if invocation.owner_type_arguments[0] != invocation.signature.result { raise :owner }
                    if invocation.signature.result == Integer.type { calls } else { result }
                })
            }
        }
        class Box<T> { @Wrap() public async fun value(value: T, gate) -> T { await gate; let result: T = value; result } }
        let integer = Box<Integer>.new()
        let string = Box<String>.new()
        let first_gate = Gate.new()
        let string_gate = Gate.new()
        let second_gate = Gate.new()
        let first = integer.value(7, first_gate)
        let middle = string.value("s", string_gate)
        let second = integer.value(9, second_gate)
        let posted_string = Gate.complete(string_gate, nil)
        let string_result = Host.run(middle)
        let posted_first = Gate.complete(first_gate, nil)
        let first_result = Host.run(first)
        let posted_second = Gate.complete(second_gate, nil)
        %[first_result, string_result, Host.run(second)]
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(2u64.into()),
            Value::Text("s".into()),
            Value::Integer(2u64.into()),
        ])))
    );
}

#[test]
fn failed_materialization_retries_when_no_chain_was_published() {
    let given = r#"
        class Effects {
            public class property count: Integer = 0
            public class property initialized: Integer = 0
        }
        class Wrap { fun initialize() { @fresh = true } }
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { @fresh = false; Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                if !@fresh { raise :stale }
                @fresh = false
                if c.reason == :origin { return Transformation.empty }
                Effects.count = Effects.count + 1
                if Effects.count == 1 { raise :retry }
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
            }
        }
        class Box<T> {
            fun initialize() { Effects.initialized = Effects.initialized + 1 }
            @Wrap() public fun value(value: T) -> T { value }
        }
        let integer = Box<Integer>.new()
        let failed = try { integer.value(1) } catch error { error == :retry }
        let retried = integer.value(2)
        let cached = integer.value(3)
        let string = Box<String>.new().value("s")
        %[failed, retried, cached, string, Effects.count, Effects.initialized]
    "#;
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(2u64.into()),
            Value::Integer(3u64.into()),
            Value::Text("s".into()),
            Value::Integer(3u64.into()),
            Value::Integer(2u64.into()),
        ])))
    );
}
