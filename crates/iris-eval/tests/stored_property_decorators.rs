use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{Capability, ClassError, Value};

const WRAP: &str = r#"class Wrap {}
impl Wrap for PropertyDecorator {
    public fun plan(declaration, arguments) -> Plan {
        if declaration.name != :value { raise :plan_name }
        if (declaration.type == Integer.type) == false { raise :plan_type }
        Plan.empty
    }
    public fun transform(declaration, arguments, context) -> Transformation {
        if declaration.name != :value { raise :name }
        if (declaration.type == Integer.type) == false { raise :type }
        if context.kind != :property { raise :kind }
        mut calls = 0
        let wrapper = { |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if invocation.slot[1] != :value { raise :selector }
            if invocation.slot[3] == :getter { next.call() + calls } else {
                next.call(ArgumentChanges.new(positional: %{:value: 9})) + calls
            }
        }
        Transformation.wrap_getter(wrapper).wrap_setter(wrapper)
    }
}; 0"#;

const PROPERTY: &str = "@Wrap() public property value: Integer = 4 { public get; public set; }";

#[test]
fn accessors_share_one_application_when_generated_over_the_raw_slot() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    let when = given.evaluate(&format!(
        "class Target {{ {PROPERTY} public fun raw() {{ @value }} }};
        let target = Target.new(); %[target.value, target.value = 2, target.value, target.raw(), Target.active_revision]"
    ));
    assert_eq!(when, evaluate("%[5, 11, 12, 9, 1]"));
}

#[test]
fn external_read_is_denied_when_getter_defaults_private() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    given.evaluate("class Target { @Wrap() public property value: Integer = 4 { get; public set; } public fun read() { self.value } }; let target = Target.new(); 0").unwrap();
    let when = given.evaluate("%[target.read(), try { target.value; false } catch error { error == :MethodVisibilityError }]");
    assert_eq!(when, evaluate("%[5, true]"));
}

#[test]
fn absent_accessors_reject_wrapping_without_publishing_origin() {
    for accessors in ["public get;", "public set;", ""] {
        let mut given = Session::new().unwrap();
        given.evaluate(WRAP).unwrap();
        let when = given.evaluate(&format!(
            "class Target {{ @Wrap() property value: Integer = 4 {{ {accessors} }} }}"
        ));
        assert!(
            matches!(when, Err(EvaluationError::Raised(_))),
            "{accessors}: {when:?}"
        );
        assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
    }
}

#[test]
fn setter_is_absent_when_only_getter_is_written() {
    let given = "class Target { property value: Integer = 4 { public get; } }; let target = Target.new(); %[try { target.value = 9; false } catch error { error == :MessageNotFound }, target.value]";
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[true, 4]"));
}

#[test]
fn shorthand_preserves_accessors_when_block_is_omitted() {
    let given = "class Target { public property value: Object }; let target = Target.new(); %[target.value, target.value = 9, target.value]";
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[nil, 9, 9]"));
}

#[test]
fn initializers_run_once_per_instance_before_initialize() {
    let given = "class Effects { public class property count: Integer = 0 }
        class Base { property first: Integer = seed() { public get; }
            public fun seed() -> Integer { Effects.count = Effects.count + 1 } }
        class Target extends Base { property second: Integer = self.first + seed() { public get; }
            public fun initialize() { Effects.count = Effects.count + 1 } }
        let first = Target.new(); let second = Target.new();
        %[first.first, first.second, second.first, second.second, Effects.count]";
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[1, 3, 4, 9, 6]"));
}

#[test]
fn replacement_reads_and_writes_the_existing_raw_slot() {
    let given = "class Target { property value: Integer = 7 { public get; public set; } }
        let target = Target.new(); open class Target {
            public override property fun value() -> Integer { @value + 1 }
            public override property fun value=(value: Integer) -> Integer { @value = value + 2 }
        }; %[target.value, target.value = 9, target.value]";
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[8, 11, 12]"));
}

#[test]
fn property_body_allows_wrapping_when_other_lanes_are_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    let when = given.evaluate(&format!(
        "class Target meta deny method_body, method_set, property_set {{ {PROPERTY} }}; Target.new().value"
    ));
    assert_eq!(when, evaluate("5"));
}

#[test]
fn property_body_denial_keeps_origin_private() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    let when = given.evaluate(&format!(
        "class Target meta deny property_body {{ {PROPERTY} }}"
    ));
    assert!(
        matches!(
            when,
            Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
                operation: Capability::PropertyBody,
                ..
            }))
        ),
        "{when:?}"
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn wrapper_result_is_checked_when_accessor_body_is_skipped() {
    for (operation, call) in [
        ("wrap_getter", "target.value"),
        ("wrap_setter", "target.value = 8"),
    ] {
        let given = format!("class Wrong {{}}
        impl Wrong for PropertyDecorator {{
            public fun plan(d, a) -> Plan {{ Plan.empty }}
            public fun transform(d, a, c) -> Transformation {{
                Transformation.{operation}({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; false }})
            }}
        }}; class Target {{ @Wrong() property value: Integer = 4 {{ public get; public set; }} }};
        let target = Target.new(); try {{ {call}; false }} catch error {{ error is? TypeError }}");
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{operation}");
    }
}

#[test]
fn input_guard_preserves_storage_when_setter_argument_is_wrong() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    let when = given.evaluate(&format!(
        "class Target {{ {PROPERTY} public fun raw() {{ @value }} }}; let target = Target.new();
        %[try {{ target.value = false; false }} catch error {{ error is? TypeError }}, target.raw()]"
    ));
    assert_eq!(when, evaluate("%[true, 4]"));
}

#[test]
fn readonly_application_runs_once_when_only_getter_is_present() {
    let given = "class Effects { public class property count: Integer = 0 }
        class Read {}
        impl Read for PropertyDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Effects.count = Effects.count + 1
                Transformation.wrap_getter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() })
            }
        }
        class Target { @Read() property value: Integer = 4 { public get; } }
        let target = Target.new(); %[target.value, target.value, Effects.count]";
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[4, 4, 1]"));
}

#[test]
fn callback_wraps_stored_accessors_after_the_declaration_group() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    given.evaluate("class Target { }; 0").unwrap();
    let when = given.evaluate(&format!(
        "let opened = Target.open() {{ |candidate|; {PROPERTY} }};
        let target = Target.new(); %[target.value, target.value = 2, target.value]"
    ));
    assert_eq!(when, evaluate("%[5, 11, 12]"));
}

#[test]
fn missing_accessor_rolls_back_callback_additions() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    given
        .evaluate("class Target { }; let revision = Target.active_revision; 0")
        .unwrap();
    let when = given.evaluate(
        "try { Target.open() { |candidate|;
        candidate.define_method(:staged, { 99 })
        @Wrap() property value: Integer = 4 { public get; }
    } } catch error { error is? ArgumentError }",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
    assert_eq!(given.evaluate("%[Target.active_revision == revision, Target.method(:staged) == nil, Target.method(:value) == nil]"), evaluate("%[true, true, true]"));
}

#[test]
fn planning_rejects_effects_when_only_stored_property_is_decorated() {
    let given = "class Impure {}
    impl Impure for PropertyDecorator {
        public fun plan(d, a) -> Plan { print(:forbidden); Plan.empty }
        public fun transform(d, a, c) -> Transformation { Transformation.empty }
    }; class Target { @Impure() property value: Integer = 4 { public get; } }";
    let when = evaluate(given);
    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-NONDETERMINISTIC",
            phase: "static",
        })
    );
}
