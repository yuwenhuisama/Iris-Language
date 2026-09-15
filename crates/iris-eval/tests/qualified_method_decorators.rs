use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{Capability, ClassError, Value};

const WRAP: &str = r#"
class Wrap {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if (invocation.receiver is? Target) == false { raise :receiver }
            next.call() + calls
        })
    }
}
impl Wrap for MethodDecorator {}
contract Named { fun value(value: Integer) -> Integer }
"#;

#[test]
fn static_impl_reuses_the_decorated_ordinary_method() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(include_str!(
            "../../iris-cli/tests/decorator_targets/qualified_method.iris"
        ))
        .unwrap();
    let when = given.evaluate("%[target.name(), (target as Named)..name(), Effects.wrappers]");
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("name".into()),
            Value::Symbol("name".into()),
            Value::Integer(5u64.into())
        ])))
    );
}

#[test]
fn shared_signature_checks_inputs_before_entering_either_surface() {
    let given = format!(
        "{WRAP}
        class Target {{ @Wrap() public fun value(value: Integer) -> Integer {{ value }} }}
        impl Target for Named {{}}
        let target = Target.new()
        %[target.value(4),
         try {{ (target as Named)..value(false) }} catch error {{ error is? TypeError }},
         try {{ target.value(false) }} catch error {{ error is? TypeError }},
         (target as Named)..value(4)]"
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(5u64.into()),
            Value::Bool(true),
            Value::Bool(true),
            Value::Integer(6u64.into())
        ])))
    );
}

#[test]
fn wrapping_is_allowed_when_only_method_set_is_denied() {
    let given = format!(
        "{WRAP} class Target meta deny method_set {{ @Wrap() public fun value(value: Integer) -> Integer {{ value }} }}
        impl Target for Named {{}}
        (Target.new() as Named)..value(4)"
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("5"));
}

#[test]
fn origin_rolls_back_when_qualified_method_body_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate(&format!("{WRAP}; 0")).unwrap();
    let when = given.evaluate(
        "class Target meta deny method_body { @Wrap() public fun value(value: Integer) -> Integer { value } }
        impl Target for Named {}",
    );
    assert!(
        matches!(
            when,
            Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
                operation: Capability::MethodBody,
                ..
            }))
        ),
        "{when:?}"
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn qualified_slot_rolls_back_when_open_transform_fails() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&format!(
            "{WRAP}
        class Fail {{
            public fun plan(d, a) -> Plan {{ Plan.empty }}
            public fun transform(d, a, c) -> Transformation {{ raise :failed }}
        }}
        impl Fail for MethodDecorator {{}}
        class Target {{ @Wrap() public fun value(value: Integer) -> Integer {{ value }} }}
        impl Target for Named {{}}; let target = Target.new(); (target as Named)..value(4)"
        ))
        .unwrap();
    let when = given.evaluate(
        "open class Target {
        @Fail() public override fun value(value: Integer) -> Integer { 90 }
    }",
    );
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("failed".into())))
    );
    assert_eq!(
        given.evaluate("%[target.value(4), (target as Named)..value(4), Target.active_revision]"),
        evaluate("%[6, 7, 1]")
    );
}

#[test]
fn qualified_async_adapter_checks_the_shared_method_contract() {
    let given = r#"
        class Wrap {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                    await next.call()
                })
            }
        }
        impl Wrap for MethodDecorator {}
        contract Named { async fun value(value: Integer) -> Integer }
        class Target { @Wrap() public async fun value(value: Integer) -> Integer { value } }
        impl Target for Named {}
        let target = Target.new()
        let task = (target as Named)..value(7)
        %[task is? Task<Integer>, Host.run(task),
         try { Host.run((target as Named)..value(false)) } catch error { error is? TypeError }]
    "#;
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[true, 7, true]"));
}

#[test]
fn ordinary_decorator_drops_outer_qualified_context_for_nested_send() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    if invocation.slot[2] != nil { raise :qualifier }
                    next.call()
                })
            }
        }
        contract Named { fun value(value: Integer) -> Integer }
        class Target {
            @Wrap() public fun decorated(value: Integer) -> Integer { value }
            public fun value(value: Integer) -> Integer { decorated(value) }
        }
        impl Target for Named {}
        let target = Target.new()
        (target as Named)..value(7)
    "#;
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(7u64.into())));
}

#[test]
fn async_ordinary_decorator_drops_outer_qualified_context_for_nested_send() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                    let result = await next.call()
                    if invocation.slot[2] != nil { raise :qualifier }
                    result
                })
            }
        }
        contract Named { async fun value(value: Integer, gate) -> Integer }
        class Target {
            @Wrap() public async fun decorated(value: Integer, gate) -> Integer { await gate; value }
            public async fun value(value: Integer, gate) -> Integer { await decorated(value, gate) }
        }
        impl Target for Named {}
        let gate = Gate.new()
        let task = (Target.new() as Named)..value(7, gate)
        Gate.complete(gate, nil)
        Host.run(task)
    "#;
    assert_eq!(
        evaluate(given),
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Nil,
            Value::Integer(7u64.into()),
        ])))
    );
}

#[test]
fn contract_result_is_checked_when_wrapper_skips_the_body() {
    let given = r#"
        class Wrong {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; false })
            }
        }
        impl Wrong for MethodDecorator {}
        contract Named { fun value() -> Integer }
        class Target { @Wrong() public fun value() -> Integer { 7 } }
        impl Target for Named {}
        try { (Target.new() as Named)..value(); false } catch error { error is? TypeError }
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn qualified_replacement_gets_fresh_state_when_open_commits() {
    let mut given = Session::new().unwrap();
    given
        .evaluate(&format!(
            "{WRAP} class Target {{ @Wrap() public fun value(value: Integer) -> Integer {{ value }} }}
        impl Target for Named {{}}; let target = Target.new(); (target as Named)..value(4)"
        ))
        .unwrap();
    let when = given.evaluate(
        "open class Target {
        @Wrap() public override fun value(value: Integer) -> Integer { value + 10 }
    }; %[target.value(4), (target as Named)..value(4), (target as Named)..value(4)]",
    );
    assert_eq!(when, evaluate("%[15, 16, 17]"));
}
