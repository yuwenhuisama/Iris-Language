use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{DispatchOutcome, Value};

fn run(evaluator: &mut SourceEvaluator, source: &str) -> Result<Value, EvaluationError> {
    let parsed = iris_parser::parse(source);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    evaluator.set_source(source);
    evaluator.program(&parsed.program)
}

#[test]
fn retained_generic_materializes_old_artifact_when_replacement_is_active()
-> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package("generic.tests")?;
    run(
        &mut given,
        r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                mut calls = 0
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    calls = calls + 1
                    if invocation.signature.result == Integer.type { calls } else { next.call() }
                })
            }
        }
        class Target { @Wrap() public class fun value<Element>(value: Element) -> Element { value } }
        Target.value<Integer>(7)
    "#,
    )?;
    let owner = given
        .class_name("Target")?
        .ok_or(EvaluationError::NameError)?;
    let selector = given.selector("value");
    let DispatchOutcome::Invoke(retained) = given
        .runtime
        .registry()
        .dispatch_class_object(owner, selector)
        .map_err(iris_runtime::ConstructionError::from)
        .map_err(EvaluationError::Construction)?
    else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    run(
        &mut given,
        r#"open class Target {
        @Wrap() public override class fun value<Element>(value: Element) -> Element { "replacement" }
    }; 0"#,
    )?;

    let when = [Value::Text("old".into()), Value::Integer(9u64.into())]
        .into_iter()
        .map(|value| given.reflective_invoke(retained, Value::Class(owner), &[value]))
        .collect::<Result<Vec<_>, _>>();

    assert_eq!(
        when,
        Ok(vec![Value::Text("old".into()), Value::Integer(2u64.into())])
    );
    Ok(())
}

#[test]
fn failed_closed_transformation_retries_when_same_type_is_requested() -> Result<(), EvaluationError>
{
    let mut given = SourceEvaluator::new_in_package("generic.tests")?;
    run(
        &mut given,
        r#"
        class Effects { public class property transforms: Integer = 0 }
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Effects.transforms = Effects.transforms + 1
                let transformation = Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    next.call()
                })
                if c.reason == :closed_materialization && Effects.transforms < 4 {
                    transformation.wrap_method(7)
                } else { transformation }
            }
        }
        class Target { @Wrap() public class fun value<Element>(value: Element) -> Element { value } }
        0
    "#,
    )?;
    let chains = given.wrapper_chains.len();

    let when = run(
        &mut given,
        r#"
        let first = try { Target.value<String>("bad"); false } catch error { error is? TypeError }
        let second = try { Target.value<String>("bad"); false } catch error { error is? TypeError }
        let good = Target.value<Integer>(7)
        let again = Target.value<Integer>(9)
        %[first, second, good, again, Effects.transforms]
    "#,
    );

    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Integer(7u64.into()),
            Value::Integer(9u64.into()),
            Value::Integer(4u64.into()),
        ])))
    );
    assert_eq!(given.wrapper_chains.len(), chains);
    Ok(())
}

#[test]
fn retained_singleton_keeps_old_chain_when_new_revision_is_published() {
    let mut given = SourceEvaluator::new_in_package("singleton.tests").unwrap();
    run(&mut given, r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                mut calls = 0
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    calls = calls + 1
                    next.call() + calls
                })
            }
        }
        class Target { @Wrap() public class fun value<Element>(value: Element) -> Element { value } }
        Target.value<Integer>(4)
    "#).unwrap();
    let owner = given.class_name("Target").unwrap().unwrap();
    let selector = given.selector("value");
    let DispatchOutcome::Invoke(retained) = given
        .runtime
        .registry()
        .dispatch_class_object(owner, selector)
        .unwrap()
    else {
        panic!("missing singleton")
    };
    assert_ne!(
        retained.id(),
        given.wrapper_chains[&retained.id()].original.id()
    );
    run(&mut given, "open class Target { @Wrap() public override class fun value<Element>(value: Element) -> Element { value + 10 } }; 0").unwrap();
    let DispatchOutcome::Invoke(published) = given
        .runtime
        .registry()
        .dispatch_class_object(owner, selector)
        .unwrap()
    else {
        panic!("missing singleton")
    };
    assert_ne!(retained.id(), published.id());

    let when = given.reflective_invoke(
        retained,
        Value::Class(owner),
        &[Value::Integer(4u64.into())],
    );
    assert_eq!(when, Ok(Value::Integer(6u64.into())));
    assert_eq!(
        given.reflective_invoke(
            published,
            Value::Class(owner),
            &[Value::Integer(4u64.into())]
        ),
        Ok(Value::Integer(15u64.into()))
    );
}
