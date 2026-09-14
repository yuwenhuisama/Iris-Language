use crate::{EvaluationError, Session};
use iris_runtime::Value;

const SOURCE: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1; next.call() + calls
        })
    }
}
contract Named { fun value() -> Integer }
class Target {
    @Wrap() public fun value() -> Integer { 100 }
    public class fun count() -> Integer { 30 }
    public property fun item() -> Integer { 40 }
}
impl Target for Named {}
; 0
"#;

fn attach(given: &mut Session, source: &str) {
    given.evaluator.enter_artifact(None);
    given
        .evaluator
        .enter_revision_artifacts(vec![crate::RevisionArtifact {
            package_id: super::super::LOCAL_PACKAGE.into(),
            api_major: 1,
            logical_owner: "Target".into(),
            revision: 1,
            artifact: (
                "opaque".into(),
                iris_runtime::artifact_digest(source.as_bytes()),
                source.into(),
            ),
        }]);
}

#[test]
fn retains_old_qualified_chain_when_whole_class_rebuild_commits_once() {
    let mut given = Session::new().unwrap();
    given.evaluate(SOURCE).unwrap();
    given
        .evaluate("open class Target { @Wrap() public override fun value() -> Integer { 200 } }; 0")
        .unwrap();
    attach(&mut given, SOURCE);
    let class = given.evaluator.class_name("Target").unwrap().unwrap();
    let selector = given.evaluator.selectors["value"];
    let iris_runtime::DispatchOutcome::Invoke(retained) = given
        .evaluator
        .runtime
        .registry()
        .dispatch(class, selector)
        .unwrap()
    else {
        panic!("missing retained method")
    };
    let receiver = Value::Object(given.evaluator.construct(class, &[]).unwrap());
    assert_eq!(
        given
            .evaluator
            .reflective_invoke(retained, receiver.clone(), &[]),
        crate::evaluate("201")
    );
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(class)
        .unwrap()
        .clone();

    given.evaluate("Target.rollback(1)").unwrap();

    let after = given.evaluator.runtime.registry().active(class).unwrap();
    assert_eq!(after.commit_id(), before.commit_id() + 1);
    assert_eq!(after.number(), before.number() + 1);
    for selector in ["value", "item"] {
        let selector = &given.evaluator.selectors[selector];
        assert_ne!(after.methods()[selector], before.methods()[selector]);
    }
    assert_ne!(after.singleton_methods(), before.singleton_methods());
    let iris_runtime::DispatchOutcome::Invoke(fresh) = given
        .evaluator
        .runtime
        .registry()
        .dispatch(class, selector)
        .unwrap()
    else {
        panic!("missing rebuilt method")
    };
    assert_ne!(fresh.id(), retained.id());
    assert_eq!(given.evaluator.wrapper_chains[&fresh.id()].qualifier, None);
    assert_eq!(
        given
            .evaluator
            .reflective_invoke(fresh, receiver.clone(), &[]),
        crate::evaluate("101")
    );
    assert_eq!(
        given.evaluator.reflective_invoke(retained, receiver, &[]),
        crate::evaluate("202")
    );
}

#[test]
fn restores_all_tables_when_late_qualified_transform_has_wrong_kind() {
    let mut given = Session::new().unwrap();
    given.evaluate(SOURCE).unwrap();
    let history = SOURCE.replace("Transformation.wrap_method", "Transformation.wrap_getter");
    attach(&mut given, &history);
    let class = given.evaluator.class_name("Target").unwrap().unwrap();
    let before = given
        .evaluator
        .runtime
        .registry()
        .active(class)
        .unwrap()
        .clone();
    let qualified = given.evaluator.qualified_methods.clone();
    let chains = given.evaluator.wrapper_chains.clone();
    let properties = given.evaluator.property_methods.clone();
    let singletons = given.evaluator.singleton_identities.clone();
    let declarations = given.evaluator.singleton_declarations.clone();
    let metadata = given.evaluator.decorator_metadata.len();

    let when = given.evaluate("Target.rollback(1)");

    assert_eq!(
        when,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "candidate validation"
        })
    );
    assert_eq!(
        given.evaluator.runtime.registry().active(class).unwrap(),
        &before
    );
    assert_eq!(given.evaluator.qualified_methods, qualified);
    assert_eq!(given.evaluator.property_methods, properties);
    assert_eq!(given.evaluator.singleton_identities, singletons);
    assert_eq!(given.evaluator.singleton_declarations, declarations);
    assert_eq!(given.evaluator.decorator_metadata.len(), metadata);
    assert_eq!(given.evaluator.wrapper_chains.len(), chains.len());
    for (identity, chain) in chains {
        assert!(std::rc::Rc::ptr_eq(
            &given.evaluator.wrapper_chains[&identity],
            &chain
        ));
    }
    assert_eq!(
        given.evaluate("(Target.new() as Named)..value()"),
        crate::evaluate("101")
    );
}
