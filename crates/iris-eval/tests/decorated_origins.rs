use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::Value;

const STAMP: &str = "class Stamp {}
impl Stamp for ClassDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation { Transformation.empty }
}; 0";
const WRAP: &str = "class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; next.call() + 1 })
    }
}; 0";

#[test]
fn class_state_initializes_once_before_explicit_open_executes() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    given.evaluate("mut effects = %[]; 0").unwrap();
    let when = given.evaluate(
        "@Stamp() class Target {
        public class property first: Integer = { effects.append(:first); 7 }.call()
        public class property second: Integer = self.first + 1
        shared mut @@count: Integer = { effects.append(:mutable); 10 }.call()
        shared let @@fixed: Integer = { effects.append(:immutable); 20 }.call()
        public class fun read() { %[@@count, @@fixed, @raw] }
        public class fun bump() { @@count = @@count + 1 }
    }; open class Target { @raw = self.second; self.first = @raw + 1 };
    %[Target.active_revision, Target.first, Target.second, Target.read(), Target.bump(), effects]",
    );
    assert_eq!(
        when,
        Session::new()
            .unwrap()
            .evaluate("%[2, 9, 8, %[10, 20, 8], 11, %[:first, :mutable, :immutable]]")
    );
}

#[test]
fn shared_immutable_cell_is_preserved_when_origin_publishes() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    given.evaluate("@Stamp() class Target { shared let @@fixed: Integer = 20; public class fun write() { @@fixed = 21 }; public class fun read() { @@fixed } }; 0").unwrap();
    let when = given.evaluate("Target.write()");
    assert!(matches!(
        when,
        Err(EvaluationError::Construction(
            iris_runtime::ConstructionError::ImmutableClassVariable { .. }
        ))
    ));
    assert_eq!(given.evaluate("Target.read()"), evaluate("20"));
}

#[test]
fn executable_origin_body_is_rejected_before_reflection_can_observe_it() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    let when = given.evaluate("@Stamp() class Target { public class property value: Integer = 7; @raw = 8; Reflection::Object.get_ivar(self, :@raw) }; 0");
    assert_eq!(when, Err(EvaluationError::ParseDiagnostic));
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn executable_origin_body_is_rejected_before_external_dispatch() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    given
        .evaluate("class Probe { public class fun read(target) { target.value } }; 0")
        .unwrap();
    let when = given.evaluate(
        "@Stamp() class Target { public class property value: Integer = 7; Probe.read(self) }; 0",
    );
    assert_eq!(when, Err(EvaluationError::ParseDiagnostic));
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn revision_is_one_when_decorated_origin_commits() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    let when = given.evaluate(
        "@Stamp() class Target { public property value: Integer = 7 }; Target.active_revision",
    );
    assert_eq!(when, evaluate("1"));
}

#[test]
fn public_name_is_absent_when_transform_executes() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Probe {} impl Probe for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Target; Transformation.empty } }; 0").unwrap();
    let when = given.evaluate("@Probe() class Target { }");
    assert_eq!(when, Err(EvaluationError::NameError));
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn session_recovers_when_origin_transform_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stop {} impl Stop for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :abort } }; 0").unwrap();
    let when = given.evaluate("@Stop() class Target { public property stale: Integer = 7 }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
    given.evaluate(STAMP).unwrap();
    assert_eq!(
        given.evaluate("@Stamp() class Target { }; Target.active_revision"),
        evaluate("1")
    );
}

#[test]
fn method_only_origin_wraps_when_method_set_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate(WRAP).unwrap();
    let when = given.evaluate("class Target meta deny method_set { @Wrap() public fun value() -> Integer { 7 } }; %[Target.active_revision, Target.new().value()]");
    assert_eq!(when, evaluate("%[1, 8]"));
}

#[test]
fn generated_addition_is_rejected_when_method_set_is_denied() {
    let mut given = Session::new().unwrap();
    given.evaluate("let generated = { 7 }; class Add {} impl Add for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { Transformation.add_method(:added, generated) } }; 0").unwrap();
    let when =
        given.evaluate("@Add() class Target meta deny method_set { public fun value() { 1 } }");
    assert!(
        matches!(
            when,
            Err(EvaluationError::Class(
                iris_runtime::ClassError::MetaCapabilityDenied {
                    operation: iris_runtime::Capability::MethodSet,
                    ..
                }
            ))
        ),
        "{when:?}"
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
}

#[test]
fn explicit_open_can_stage_a_method_after_origin_publication() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    let when = given.evaluate(
        "@Stamp() class Target {}; open class Target { self.define_method(:value, { 7 }) }; Target.new().value()",
    );
    assert_eq!(when, evaluate("7"));
}

#[test]
fn external_constructor_effects_survive_when_transform_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Effects { public class property count: Integer = 0 }; class Stop { fun initialize() { @ready = true } } impl Stop for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :abort } }; 0").unwrap();
    given.evaluate("Stop.open() { |candidate|; candidate.define_method(:initialize, { Effects.count = Effects.count + 1 }) }; 0").unwrap();
    let when = given.evaluate("@Stop() class Target { }");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluate("Effects.count"), evaluate("1"));
}
