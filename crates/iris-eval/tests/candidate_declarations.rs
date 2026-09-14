use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

const EMPTY_METHOD_DECORATOR: &str = "class Stamp {}
impl Stamp for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        if c.reason != :open { raise :reason }
        Transformation.empty
    }
}";

#[test]
fn direct_open_callback_publishes_method_and_properties() {
    let source = format!(
        "{EMPTY_METHOD_DECORATOR}; class Target {{ }};
        let opened = Target.open() {{ |candidate|;
            @Stamp() public fun value() -> Integer {{ 7 }}
            public property fun computed() -> Integer {{ 9 }}
            public property stored: Integer = 11
        }};
        let target = Target.new(); target.value() + target.computed + target.stored"
    );
    assert_eq!(evaluate(&source), Ok(Value::Integer(27u64.into())));
}

#[test]
fn helper_closure_does_not_inherit_candidate_declaration_authority() {
    let mut session = Session::new().unwrap();
    assert!(session.evaluate("class Target { }; 0").is_ok());
    let outcome = session.evaluate(
        "Target.open() { |candidate|;
        candidate.define_method(:staged, { 1 })
        let helper = { public fun leaked() { 2 } }; helper.call()
    }",
    );
    assert_eq!(outcome, Err(EvaluationError::UnsupportedConstruct));
    assert_eq!(
        session.evaluate("Target.method(:staged) == nil && Target.method(:leaked) == nil"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn open_decorator_plan_runs_before_earlier_runtime_effects() {
    let source = "class Stop {}
    impl Stop for MethodDecorator {
        public fun plan(d, a) -> Plan { raise :planning }
        public fun transform(d, a, c) -> Transformation { raise :runtime }
    }
    class Target { }
    raise :earlier_runtime
    Target.open() { |candidate|; @Stop() public fun value() { 7 } }";
    assert_eq!(
        evaluate(source),
        Err(EvaluationError::Raised(Value::Symbol("planning".into())))
    );
}

#[test]
fn retained_method_transformation_rejects_property_and_rolls_back() {
    let mut session = Session::new().unwrap();
    let fixture = include_str!("../../iris-cli/tests/decorator_protocol/wrong_kind_rollback.iris");
    assert_eq!(
        session.evaluate(fixture),
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "candidate validation",
        })
    );
    assert_eq!(
        session.evaluate("Target.method(:extra) == nil && Target.method(:staged) == nil"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn capability_failure_is_typed_and_preserves_old_receiver() {
    let mut session = Session::new().unwrap();
    let fixture = include_str!("../../iris-cli/tests/decorator_protocol/capability_rollback.iris");
    assert!(session.evaluate(fixture).is_ok());
    assert_eq!(
        session.evaluate("Target.method(:wrapped) == nil && Target.method(:staged) == nil"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn session_open_plans_against_prior_decorator_definitions() {
    let mut session = Session::new().unwrap();
    assert!(
        session
            .evaluate(&format!("{EMPTY_METHOD_DECORATOR}; class Target {{ }}; 0"))
            .is_ok()
    );
    assert!(
        session
            .evaluate("Target.open() { |candidate|; @Stamp() public fun value() -> Integer { 7 } }")
            .is_ok()
    );
    assert_eq!(
        session.evaluate("Target.new().value()"),
        Ok(Value::Integer(7u64.into()))
    );
}

#[test]
fn rollback_discards_property_metadata_and_preserves_retained_method() {
    let mut session = Session::new().unwrap();
    assert!(session.evaluate("class Target { public fun value() -> Integer { 7 } }; let target = Target.new(); let old = target.value; 0").is_ok());
    assert_eq!(
        session.evaluate(
            "Target.open() { |candidate|;
        public property extra: Integer = 11
        public override fun value() -> Integer { 99 }
        raise :abort
    }"
        ),
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(session.evaluate("let ignored = Reflection::Object.set_ivar(target, :@extra, :untyped); old.call() + target.value()"), Ok(Value::Integer(14u64.into())));
    assert_eq!(
        session.evaluate("Target.method(:extra) == nil"),
        Ok(Value::Bool(true))
    );
}

#[test]
fn caught_nested_failure_aborts_sibling_declarations() {
    let mut session = Session::new().unwrap();
    assert!(
        session
            .evaluate("class First { }; class Second { }; 0")
            .is_ok()
    );
    assert!(session.evaluate("First.open() { |candidate|;
        public property added: Integer = 1
        try { Second.open() { |sibling|; public fun other() { 2 }; raise :abort } } catch error { nil }
    }").is_ok());
    assert_eq!(
        session.evaluate("First.method(:added) == nil && Second.method(:other) == nil"),
        Ok(Value::Bool(true))
    );
}
