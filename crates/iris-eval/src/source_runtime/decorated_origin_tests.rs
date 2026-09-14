use crate::{EvaluationError, Session};
use iris_runtime::Value;

const STAMP: &str = "class Stamp {}
impl Stamp for ClassDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation { Transformation.empty }
}; 0";

#[test]
fn one_commit_is_consumed_when_origin_succeeds() {
    let mut given = Session::new().unwrap();
    given.evaluate(STAMP).unwrap();
    let stamp = given.evaluator.class_name("Stamp").unwrap().unwrap();
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(stamp)
        .unwrap()
        .clone();
    given
        .evaluate("@Stamp() class Target { public fun value() { 7 } }; 0")
        .unwrap();
    let target = given.evaluator.class_name("Target").unwrap().unwrap();
    let when = given.evaluator.runtime.registry().active(target).unwrap();
    assert_eq!(when.number(), 1);
    assert_eq!(when.commit_id(), before.commit_id() + 1);
    assert_eq!(when.id().raw(), before.id().raw() + 1);
}

#[test]
fn no_revision_is_consumed_when_origin_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stop {}; impl Stop for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :abort } }; 0").unwrap();
    given.evaluate(STAMP).unwrap();
    let stamp = given.evaluator.class_name("Stamp").unwrap().unwrap();
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(stamp)
        .unwrap()
        .clone();
    assert_eq!(
        given.evaluate("@Stop() class Target { public property stale: Integer = 1 }; 0"),
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
    given.evaluate("@Stamp() class Target { }; 0").unwrap();
    let target = given.evaluator.class_name("Target").unwrap().unwrap();
    let when = given.evaluator.runtime.registry().active(target).unwrap();
    assert_eq!(when.id().raw(), before.id().raw() + 1);
    assert_eq!(when.commit_id(), before.commit_id() + 1);
}

#[test]
fn backend_metadata_is_discarded_when_later_method_transform_fails() {
    let mut given = Session::new().unwrap();
    given.evaluate("class Stop {}; impl Stop for MethodDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :abort } }; 0").unwrap();
    let properties = given.evaluator.property_types.len();
    let methods = given.evaluator.property_methods.len();
    let packages = given.evaluator.package_contexts.classes.len();
    let metadata = given.evaluator.decorator_metadata.len();
    let bodies = given.evaluator.bodies.len();
    let when = given.evaluate(
        "class Target { public property stale: Integer = 1; @Stop() public fun value() { 7 } }; 0",
    );
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("abort".into())))
    );
    assert_eq!(given.evaluator.property_types.len(), properties);
    assert_eq!(given.evaluator.property_methods.len(), methods);
    assert_eq!(given.evaluator.package_contexts.classes.len(), packages);
    assert_eq!(given.evaluator.decorator_metadata.len(), metadata);
    assert_eq!(given.evaluator.bodies.len(), bodies);
    assert!(given.evaluator.origin_names.is_empty());
}

#[test]
fn source_self_in_origin_is_rejected_before_execution() {
    let when = iris_parser::parse("@Stamp() class Target { self.new() }; 0");
    assert!(!when.program_accepted, "{when:#?}");
    assert!(
        when.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"),
        "{when:#?}"
    );
}

#[test]
fn private_initial_state_is_discarded_when_initializer_or_transform_fails() {
    for failure in [
        "public class property broken: Object = { raise :abort }.call()",
        "",
    ] {
        let mut given = Session::new().unwrap();
        given.evaluate(STAMP).unwrap();
        given.evaluate("class Payload {}; class Effects { public class property kept: Object = nil }; class Stop {}; impl Stop for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty }; public fun transform(d, a, c) -> Transformation { raise :abort } }; 0").unwrap();
        let stop = given.evaluator.class_name("Stop").unwrap().unwrap();
        let before = given
            .evaluator
            .runtime
            .registry()
            .active(stop)
            .unwrap()
            .clone();
        given.evaluator.runtime.collect_garbage([]);
        let when = given.evaluate(&format!(
            "@Stop() class Target {{
            public class property payload: Object = Payload.new()
            shared let @@payload: Object = Payload.new()
            public class property effect: Object = {{ Effects.kept = Payload.new(); nil }}.call()
            {failure}
        }}; 0"
        ));
        assert_eq!(
            when,
            Err(EvaluationError::Raised(Value::Symbol("abort".into())))
        );
        assert_eq!(given.evaluate("Target"), Err(EvaluationError::NameError));
        assert_eq!(
            given.evaluator.runtime.collect_garbage([]).0,
            if failure.is_empty() { 3 } else { 2 }
        );
        assert_eq!(
            given.evaluate("Effects.kept is? Payload"),
            crate::evaluate("true")
        );
        given.evaluate("@Stamp() class Target { }; 0").unwrap();
        let target = given.evaluator.class_name("Target").unwrap().unwrap();
        let published = given.evaluator.runtime.registry().active(target).unwrap();
        assert_eq!(published.number(), 1);
        assert_eq!(published.id().raw(), before.id().raw() + 1);
        assert_eq!(published.commit_id(), before.commit_id() + 1);
    }
}
