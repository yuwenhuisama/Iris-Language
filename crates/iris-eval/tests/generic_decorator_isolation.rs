use iris_eval::evaluate;
use iris_runtime::{ArrayRef, Value};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if invocation.method_type_arguments[0] != invocation.signature.result { raise :metadata }
            if invocation.signature.parameters[0].type != invocation.signature.result { raise :parameter }
            if invocation.signature.result == Integer.type { calls } else { next.call() }
        })
    }
}
"#;

#[test]
fn captures_are_isolated_when_class_object_method_closes_over_different_types() {
    let given = format!(
        r#"{WRAP}
        class Target {{ @Wrap() public class fun echo<Element>(value: Element) -> Element {{ value }} }}
        %[Target.echo<Integer>(7), Target.echo<String>("s"), Target.echo<Integer>(9)]
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
fn applications_keep_independent_captures_when_closed_calls_are_interleaved() {
    let given = format!(
        r#"{WRAP}
        class Target {{
            @Wrap() public class fun first<Element>(value: Element) -> Element {{ value }}
            @Wrap() public class fun second<Element>(value: Element) -> Element {{ value }}
        }}
        %[Target.first<Integer>(7), Target.second<Integer>(7),
         Target.first<String>("s"), Target.first<Integer>(9), Target.second<Integer>(9)]
        "#
    );

    let when = evaluate(&given);

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1u64.into()),
            Value::Integer(1u64.into()),
            Value::Text("s".into()),
            Value::Integer(2u64.into()),
            Value::Integer(2u64.into()),
        ])))
    );
}

#[test]
fn transform_runs_once_per_closed_materialization_when_a_type_is_reused() {
    let given = r#"
    class Effects {
        public class property transforms: Integer = 0
        public class property reason: Symbol = :unset
        public class property fresh: Boolean = true
    }
    class Wrap { fun initialize() { @phase = 0 } }
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { @phase = 1; Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Effects.transforms = Effects.transforms + 1
            Effects.reason = c.reason
            Effects.fresh = Effects.fresh && (@phase == 0)
            @phase = 2
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                next.call()
            })
        }
    }
    class Target { @Wrap() public class fun echo<Element>(value: Element) -> Element { value } }
    let origin = %[Effects.transforms, Effects.reason]
    let first = Target.echo<Integer>(7)
    let integer = %[Effects.transforms, Effects.reason]
    let second = Target.echo<String>("s")
    let string = %[Effects.transforms, Effects.reason]
    let third = Target.echo<Integer>(9)
    %[origin, integer, string, %[Effects.transforms, Effects.reason], Effects.fresh]
    "#;

    let when = evaluate(given);

    let phases = [
        (1u64, "origin"),
        (2, "closed_materialization"),
        (3, "closed_materialization"),
        (3, "closed_materialization"),
    ];
    let mut expected: Vec<Value> = phases
        .into_iter()
        .map(|(count, reason)| {
            Value::Array(ArrayRef::new(vec![
                Value::Integer(count.into()),
                Value::Symbol(reason.into()),
            ]))
        })
        .collect();
    expected.push(Value::Bool(true));
    assert_eq!(when, Ok(Value::Array(ArrayRef::new(expected))));
}

#[test]
fn async_captures_share_only_the_same_close_when_calls_suspend_together() {
    let given = r#"
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            mut calls = 0
            Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                calls = calls + 1
                let value = await next.call()
                if invocation.method_type_arguments[0] != invocation.signature.result { raise :metadata }
                if invocation.signature.result == Integer.type { calls } else { value }
            })
        }
    }
    class Target {
        @Wrap() public async class fun echo<Element>(value: Element, gate) -> Element {
            await gate
            value
        }
    }
    let first_gate = Gate.new()
    let string_gate = Gate.new()
    let second_gate = Gate.new()
    let first = Target.echo<Integer>(7, first_gate)
    let string = Target.echo<String>("s", string_gate)
    let second = Target.echo<Integer>(9, second_gate)
    let first_posted = Gate.complete(first_gate, nil)
    let first_value = Host.run(first)
    let string_posted = Gate.complete(string_gate, nil)
    let string_value = Host.run(string)
    let second_posted = Gate.complete(second_gate, nil)
    let second_value = Host.run(second)
    %[first_value, string_value, second_value, Host.run(Target.echo<Integer>(11, first_gate))]
    "#;

    let when = evaluate(given);

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(2u64.into()),
            Value::Text("s".into()),
            Value::Integer(2u64.into()),
            Value::Integer(3u64.into()),
        ])))
    );
}
