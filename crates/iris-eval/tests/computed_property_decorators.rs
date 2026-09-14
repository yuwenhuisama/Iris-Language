use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{Capability, ClassError, Value};

const BOTH: &str = "class Both {}
impl Both for PropertyDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        mut calls = 0
        let wrapper = { |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if invocation.slot[1] != :value { raise :selector }
            if invocation.slot[3] == :getter { next.call() + calls } else { next.call(ArgumentChanges.new(positional: %{:value: 9})) }
        }
        Transformation.wrap_getter(wrapper).wrap_setter(wrapper)
    }
}; 0";

const ACCESSORS: &str = "@Both() public property fun value() -> Integer { 5 }
    public property fun value=(value: Integer) -> Symbol { :written }";

#[test]
fn accessors_share_application_captures_when_one_transform_wraps_both() {
    let mut given = Session::new().unwrap();
    given.evaluate(BOTH).unwrap();
    let when = given.evaluate(&format!(
        "class Target {{ {ACCESSORS} }}; let target = Target.new();
        %[target.value, target.value = 2, target.value]"
    ));
    assert_eq!(when, given.evaluate("%[6, :written, 8]"));
}

#[test]
fn accessors_resolve_after_the_group_when_declared_in_open_callback() {
    let mut given = Session::new().unwrap();
    given.evaluate(BOTH).unwrap();
    given.evaluate("class Target { }; 0").unwrap();
    let when = given.evaluate(&format!(
        "let opened = Target.open() {{ |candidate|; {ACCESSORS} }};
        let target = Target.new(); %[target.value, target.value = 2, target.value]"
    ));
    assert_eq!(when, given.evaluate("%[6, :written, 8]"));
}

#[test]
fn accessor_layers_keep_written_order_when_multiple_applications_wrap_both() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Scale {}
    impl Scale for PropertyDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            let wrapper = { |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() * 10 }
            Transformation.wrap_getter(wrapper).wrap_setter(wrapper)
        }
    }; class Add {}
    impl Add for PropertyDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            let wrapper = { |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + 2 }
            Transformation.wrap_getter(wrapper).wrap_setter(wrapper)
        }
    }; 0").unwrap();
    let when = given.evaluate(
        "class Target {
        @Scale() public property fun value() -> Integer { 1 }
        @Add() public property fun value=(value: Integer) -> Integer { value }
    }; let target = Target.new(); %[target.value, target.value = 3]",
    );
    assert_eq!(when, evaluate("%[30, 50]"));
}

#[test]
fn input_guard_precedes_zero_attempt_setter_wrapper() {
    let given = "class Cache {}
    impl Cache for PropertyDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_setter({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; raise :wrapper_entered })
        }
    }; class Target {
        @Cache() public property fun value=(value: Integer) -> Symbol { :written }
    }; let target = Target.new();
    try { target.value = \"wrong\" } catch error { error is? TypeError }";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn property_wrapping_uses_property_body_when_other_lanes_are_denied() {
    for policy in ["method_body", "method_set", "property_set"] {
        let mut given = Session::new().unwrap();
        given.evaluate(BOTH).unwrap();
        let when = given.evaluate(&format!(
            "class Target meta deny {policy} {{ {ACCESSORS} }};
            %[Target.active_revision, Target.new().value]"
        ));
        assert_eq!(when, evaluate("%[1, 6]"), "{policy}");
    }
}

#[test]
fn origin_stays_private_when_property_body_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate(BOTH).unwrap();
    let when = given.evaluate(&format!(
        "class Target meta deny property_body {{ {ACCESSORS} }}"
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
fn candidate_rolls_back_when_a_requested_accessor_is_absent() {
    let mut given = Session::new().unwrap();
    given.evaluate(BOTH).unwrap();
    given.evaluate("class Target { }; let revision = Target.active_revision; let commit = Reflection::Class.revision(Target).fetch(:commit_id); 0").unwrap();
    let when = given.evaluate(
        "try { Target.open() { |candidate|;
        candidate.define_method(:staged, { 99 })
        @Both() public property fun value() -> Integer { 5 }
    } } catch error { error is? ArgumentError }",
    );
    assert_eq!(when, Ok(Value::Bool(true)));
    assert_eq!(given.evaluate("%[Target.active_revision == revision, Reflection::Class.revision(Target).fetch(:commit_id) == commit, Target.method(:staged) == nil, Target.method(:value) == nil]"), evaluate("%[true, true, true, true]"));
}

#[test]
fn exact_result_is_checked_when_accessor_wrapper_makes_zero_attempts() {
    for (operation, accessor, invocation) in [
        ("wrap_getter", "value() -> Integer", "target.value"),
        (
            "wrap_setter",
            "value=(value: Integer) -> Symbol",
            "target.value = 3",
        ),
    ] {
        let given = format!("class Cached {{}}
        impl Cached for PropertyDecorator {{
            public fun plan(d, a) -> Plan {{ Plan.empty }}
            public fun transform(d, a, c) -> Transformation {{
                Transformation.{operation}({{ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; \"wrong\" }})
            }}
        }}
        class Target {{ @Cached() public property fun {accessor} {{ raise :body_entered }} }}
        let target = Target.new()
        try {{ {invocation} }} catch error {{ error is? TypeError }}");
        let when = evaluate(&given);
        assert_eq!(when, Ok(Value::Bool(true)), "{operation}");
    }
}
