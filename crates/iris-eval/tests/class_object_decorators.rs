use iris_eval::{Session, evaluate};
use iris_runtime::{ArrayRef, Value};

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            if (invocation.receiver == Target) == false { raise :receiver }
            if (invocation.slot[0] == Target) == false { raise :slot }
            next.call() + 1
        })
    }
}
"#;

#[test]
fn singleton_is_wrapped_when_instance_selector_matches() {
    let given = format!(
        "{WRAP} class Target {{
        public fun echo() {{ :instance }}
        @Wrap() public class fun echo(value: Integer) -> Integer {{ value }}
    }}; %[Target.echo(7), Target.new().echo()]"
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(8u64.into()),
            Value::Symbol("instance".into())
        ])))
    );
}

#[test]
fn invocation_is_closed_when_method_arguments_are_explicit() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate(include_str!(
        "../../iris-cli/tests/decorator_targets/class_object_generic.iris"
    ));
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn generic_identity_preserves_integer_and_string_when_inferred() {
    let wrap = WRAP.replace("next.call() + 1", "next.call()");
    let given = format!(
        "{wrap} class Target {{
        @Wrap() public class fun echo<Element>(value: Element) -> Element {{ value }}
    }}; %[Target.echo(7), Target.echo(\"text\")]"
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("%[7, \"text\"]"));
}

fn generic_target(wrapper: &str) -> String {
    format!(
        r#"
    class Effects {{ public class property bodies: Integer = 0; public class property wrappers: Integer = 0 }}
    class Wrap {{}}
    impl Wrap for MethodDecorator {{
        public fun plan(d, a) -> Plan {{ Plan.empty }}
        public fun transform(d, a, c) -> Transformation {{
            Transformation.wrap_method({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                Effects.wrappers = Effects.wrappers + 1
                {wrapper}
            }})
        }}
    }}
    class Target {{ @Wrap() public class fun echo<Element>(value: Element) -> Element {{
        Effects.bodies = Effects.bodies + 1
        value
    }} }}
    "#
    )
}

#[test]
fn invalid_input_prevents_zero_attempt_wrapper_when_integer_is_selected() {
    let given = format!(
        "{}; let value: Object = \"wrong\"; let failed = try {{ Target.echo<Integer>(value); false }} catch error {{ error is? TypeError }}; %[failed, Effects.wrappers, Effects.bodies]",
        generic_target("7")
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("%[true, 0, 0]"));
}

#[test]
fn invalid_patch_prevents_body_when_selected_type_cannot_be_reinferred() {
    let given = format!(
        "{}; let failed = try {{ Target.echo<Integer>(7); false }} catch error {{ error is? TypeError }}; %[failed, Effects.wrappers, Effects.bodies]",
        generic_target("next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))")
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("%[true, 1, 0]"));
}

#[test]
fn zero_attempt_result_is_checked_when_closed_result_is_integer() {
    let given = format!(
        "{}; let failed = try {{ Target.echo<Integer>(7); false }} catch error {{ error is? TypeError }}; %[failed, Effects.wrappers, Effects.bodies]",
        generic_target("\"wrong\"")
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("%[true, 1, 0]"));
}

#[test]
fn callable_signature_arguments_are_not_owner_arguments_when_method_is_closed() {
    let given = r#"
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                if invocation.owner_type_arguments.length() != 0 { raise :owner }
                if invocation.method_type_arguments.length() != 1 { raise :method }
                if invocation.method_type_arguments[0] != Integer.type { raise :type }
                next.call()
            })
        }
    }
    class Target {
        @Wrap() public class fun apply<Element>(callback: Closure<(Element) -> Element>, value: Element) -> Element { callback.call(value) }
    }
    Target.apply<Integer>({ |value: Integer| -> Integer; value + 1 }, 7)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(8u64.into())));
}

#[test]
fn async_generic_admission_checks_input_and_result_when_closed() {
    let given = r#"
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                if invocation.signature.result != Integer.type { raise :type }
                await next.call()
            })
        }
    }
    class Target { @Wrap() public async class fun echo<Element>(value: Element) -> Element { value } }
    let task = Target.echo<Integer>(7)
    let input: Object = "wrong"
    %[task is? Task<Integer>, Host.run(task), try { Host.run(Target.echo<Integer>(input)); false } catch error { error is? TypeError }]
    "#;
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[true, 7, true]"));
}

#[test]
fn retry_keeps_defaults_when_original_preparation_has_effects() {
    let given = r#"
    class Effects { public class property defaults: Integer = 0
        public class fun value() -> Integer { self.defaults = self.defaults + 1; 7 }
    }
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                next.call(); next.call()
            })
        }
    }
    class Target { @Wrap() public class fun echo<Element>(value: Element = Effects.value()) -> Element { value } }
    %[Target.echo<Integer>(), Effects.defaults]
    "#;
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[7, 1]"));
}

#[test]
fn canonical_metadata_keeps_parameter_template_when_target_is_open() {
    let given = r#"
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan {
            if d.signature.parameters[0].type.kind != :parameter { raise :template }
            if d.signature.parameters[0].type.name != :Element { raise :name }
            Plan.empty
        }
        public fun transform(d, a, c) -> Transformation {
            if d.signature.parameters[0].type.kind != :parameter { raise :template }
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
        }
    }
    class Target { @Wrap() public class fun echo<Element>(value: Element) -> Element { value } }
    Target.echo<String>("text")
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Text("text".into())));
}

#[test]
fn wrapper_and_original_keep_lexical_receivers_when_method_is_generic() {
    let given = r#"
    class Wrap { fun initialize() { @increment = 2 } }
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                next.call() + @increment
            })
        }
    }
    class Target {
        public class fun own() { 7 }
        @Wrap() public class fun echo<Element>(value: Element) -> Element { self.own() }
    }
    Target.echo<Integer>(4)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(9u64.into())));
}

#[test]
fn closed_container_is_invariant_when_patch_changes_element_type() {
    let given = format!(
        "{}; let values: Array<Integer> = %[7]; let failed = try {{ Target.echo<Array<Integer>>(values); false }} catch error {{ error is? TypeError }}; %[failed, Effects.bodies]",
        generic_target(
            "let wrong: Array<String> = %[\"wrong\"]; next.call(ArgumentChanges.new(positional: %{:value: wrong}))"
        )
    );
    let when = evaluate(&given);
    assert_eq!(when, evaluate("%[true, 0]"));
}
