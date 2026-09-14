use crate::{EvaluationError, RevisionArtifact, Session};
use iris_runtime::{MethodOwner, Value};

const CURRENT: &str = "module Provider { public fun value() -> Integer { 9 } }; 0";
const HISTORY: &str = "module Provider { public fun value() -> Integer { 1 } }";

fn attach(session: &mut Session, source: &str) {
    session.evaluator.enter_artifact(None);
    session
        .evaluator
        .enter_revision_artifacts(vec![RevisionArtifact {
            package_id: super::super::LOCAL_PACKAGE.into(),
            api_major: 1,
            logical_owner: "Provider".into(),
            revision: 1,
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(source.as_bytes()),
                source.into(),
            ),
        }]);
}

#[test]
fn one_structural_commit_when_module_and_main_rebuild() {
    let mut given = Session::new().unwrap();
    given.evaluate(CURRENT).unwrap();
    attach(&mut given, HISTORY);
    let module = given.evaluator.module_names["Provider"];
    let main = given.evaluator.module_mains[&module];
    let backing = given.evaluator.module_classes[&module];
    let before = given
        .evaluator
        .runtime
        .registry()
        .active_module(module)
        .unwrap()
        .clone();
    let before_main = given
        .evaluator
        .runtime
        .registry()
        .active(main)
        .unwrap()
        .clone();
    let before_backing = given
        .evaluator
        .runtime
        .registry()
        .active(backing)
        .unwrap()
        .clone();
    let selector = given.evaluator.selectors["value"];
    let old = before.method(selector).unwrap();

    given.evaluate("Provider.rollback(1)").unwrap();

    let after = given
        .evaluator
        .runtime
        .registry()
        .active_module(module)
        .unwrap();
    let after_main = given.evaluator.runtime.registry().active(main).unwrap();
    assert_eq!(after.number(), before.number() + 1);
    assert_eq!(after.commit_id(), before_main.commit_id() + 1);
    assert_eq!(after_main.commit_id(), after.commit_id());
    assert_eq!(after_main.number(), before_main.number() + 1);
    assert_eq!(given.evaluator.module_names["Provider"], module);
    assert_eq!(given.evaluator.module_mains[&module], main);
    assert_eq!(given.evaluator.module_classes[&module], backing);
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active(backing)
            .unwrap()
            .id(),
        before_backing.id()
    );
    let fresh = after.method(selector).unwrap();
    assert_eq!(fresh.owner(), MethodOwner::Module(module));
    assert_ne!(fresh.id(), old.id());
    assert_eq!(
        given
            .evaluator
            .invoke_method(old, Value::Symbol("Provider".into()), &[]),
        crate::evaluate("9")
    );
    assert_eq!(given.evaluate("Provider.value()"), crate::evaluate("1"));
    let main_method = given
        .evaluator
        .runtime
        .registry()
        .dispatch(main, selector)
        .unwrap();
    let iris_runtime::DispatchOutcome::Invoke(main_method) = main_method else {
        panic!("main method")
    };
    assert_eq!(
        given.evaluator.invoke_method(main_method, Value::Nil, &[]),
        crate::evaluate("1")
    );
}

#[test]
fn no_publication_or_commit_consumption_when_transform_fails() {
    let decorator = "class Fail {} impl Fail for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { if c.reason == :rollback { raise :failed }; Transformation.empty } }";
    let current = format!(
        "{decorator} module Provider {{ public fun first() -> Integer {{ 8 }}; @Fail() public fun value() -> Integer {{ 9 }} }}; 0"
    );
    let history = format!(
        "{decorator} module Provider {{ public fun first() -> Integer {{ 1 }}; @Fail() public fun value() -> Integer {{ 2 }} }}"
    );
    let mut given = Session::new().unwrap();
    given.evaluate(&current).unwrap();
    attach(&mut given, &history);
    let module = given.evaluator.module_names["Provider"];
    let main = given.evaluator.module_mains[&module];
    let backing = given.evaluator.module_classes[&module];
    let before = given
        .evaluator
        .runtime
        .registry()
        .active_module(module)
        .unwrap()
        .clone();
    let main_revision = given
        .evaluator
        .runtime
        .registry()
        .active(main)
        .unwrap()
        .id();
    let backing_revision = given
        .evaluator
        .runtime
        .registry()
        .active(backing)
        .unwrap()
        .id();
    let retained = given
        .evaluate("let retained = Provider.method(:value); Provider.value()")
        .unwrap();

    let when = given.evaluate("Provider.rollback(1)");

    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("failed".into())))
    );
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active_module(module)
            .unwrap(),
        &before
    );
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active(main)
            .unwrap()
            .id(),
        main_revision
    );
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active(backing)
            .unwrap()
            .id(),
        backing_revision
    );
    assert_eq!(given.evaluate("Provider.first()"), crate::evaluate("8"));
    assert_eq!(
        given.evaluate("Provider.invoke(retained, Provider, %[])"),
        Ok(retained)
    );
    attach(
        &mut given,
        "module Provider { public fun first() -> Integer { 1 }; public fun value() -> Integer { 2 } }",
    );
    given.evaluate("Provider.rollback(1)").unwrap();
    assert_eq!(
        given
            .evaluator
            .runtime
            .registry()
            .active_module(module)
            .unwrap()
            .commit_id(),
        before.commit_id() + 1
    );
}

#[path = "module_history_rejections.rs"]
mod rejections;

#[test]
fn unavailable_when_decorator_class_belongs_to_another_package() {
    let decorator = "class Keep {} impl Keep for MethodDecorator { public fun plan(d,a) -> Plan { Plan.empty }; public fun transform(d,a,c) -> Transformation { Transformation.empty } }";
    let mut given = Session::new().unwrap();
    given.evaluate(&format!("{decorator}; {CURRENT}")).unwrap();
    let owner = given.evaluator.class_name("Keep").unwrap().unwrap();
    given
        .evaluator
        .package_contexts
        .classes
        .insert(owner, ("foreign".into(), 1));
    attach(
        &mut given,
        &format!("{decorator} module Provider {{ @Keep() public fun value() -> Integer {{ 1 }} }}"),
    );

    let when = given.evaluate("Provider.rollback(1)");

    assert_eq!(when, Err(EvaluationError::RevisionArtifactUnavailable));
    assert_eq!(given.evaluate("Provider.value()"), crate::evaluate("9"));
}
