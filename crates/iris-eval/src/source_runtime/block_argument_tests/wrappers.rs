use super::*;

const SETUP: &str = r#"
    class State { public class property effects: Array = %[] }
    class Wrap {}
    impl Wrap for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                State.effects.append(:wrapper);
                %[invocation.original_block, invocation.omitted.length, invocation.defaulted.length, next.call()]
            })
        }
    }
    class Target {
        @Wrap() public fun run(value = State.effects.append(:default), &block: Block<() -> Integer> = nil) -> Object {
            State.effects.append(:body)
            block
        }
    }
    let target = Target.new()
    let callback = { || 9 }
    nil
"#;

#[test]
fn explicit_nil_is_not_omitted_when_wrapper_records_provenance() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::super::LOCAL_PACKAGE)?;
    run(&mut given, SETUP)?;
    let expression = call(
        "target",
        "run",
        vec![
            Expression::Literal("1".into()),
            block(Expression::Literal("nil".into())),
        ],
    );

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(when, crate::evaluate("%[nil, 0, 0, nil]")?);
    Ok(())
}

#[test]
fn omitted_block_is_defaulted_when_wrapper_records_provenance() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::super::LOCAL_PACKAGE)?;
    run(&mut given, SETUP)?;
    let expression = call("target", "run", vec![Expression::Literal("1".into())]);

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(when, crate::evaluate("%[nil, 1, 1, nil]")?);
    Ok(())
}

#[test]
fn duplicate_blocks_fail_before_wrapper_defaults_and_body() -> Result<(), EvaluationError> {
    for (first, second) in [
        ("nil", "nil"),
        ("nil", "callback"),
        ("callback", "nil"),
        ("callback", "callback"),
    ] {
        let mut given = SourceEvaluator::new_in_package(super::super::LOCAL_PACKAGE)?;
        run(&mut given, SETUP)?;
        let arguments = [first, second].map(|name| {
            block(match name {
                "nil" => Expression::Literal("nil".into()),
                name => Expression::Name(name.into()),
            })
        });
        let expression = call("target", "run", arguments.to_vec());

        let when = given.expression(&expression, &Default::default(), None);

        let Err(EvaluationError::Raised(Value::Object(error))) = when else {
            panic!("expected raised ArgumentError: {when:?}")
        };
        assert_eq!(
            given
                .runtime
                .class_of(error)
                .map_err(EvaluationError::Construction)?,
            given.class_name("ArgumentError")?.unwrap()
        );
        assert_eq!(run(&mut given, "State.effects")?, crate::evaluate("%[]")?);
    }
    Ok(())
}

#[test]
fn generic_inference_skips_block_when_between_positionals() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::super::LOCAL_PACKAGE)?;
    let parsed = iris_parser::parse(
        "fun generic<Element>(left, value: Element, &block = nil) -> Element { value }",
    );
    let Statement::Method(method) = &parsed.program.statements[0] else {
        panic!("expected method")
    };
    let arguments = [
        Value::Integer(1_u8.into()),
        Value::BlockArgument(Box::new(Value::Nil)),
        Value::Integer(7_u8.into()),
    ];

    given.select_wrapper_types(method, &arguments)?;

    assert_eq!(
        given.method_type_bindings.get("Element"),
        Some(&iris_syntax::TypeExpression::Name("Integer".into()))
    );
    Ok(())
}

#[test]
fn native_boundary_unwraps_only_the_block_channel() -> Result<(), EvaluationError> {
    let given = [
        Value::Integer(1_u8.into()),
        Value::BlockArgument(Box::new(Value::Nil)),
        Value::KeywordArgument("option".into(), Box::new(Value::Bool(true))),
    ];

    let when = super::super::call_channels::native_arguments(&given)?;

    assert_eq!(
        when.as_ref(),
        &[
            Value::Integer(1_u8.into()),
            Value::Nil,
            Value::KeywordArgument("option".into(), Box::new(Value::Bool(true)))
        ]
    );
    Ok(())
}

#[test]
fn native_boundary_rejects_duplicate_blocks_including_nil() {
    let given = [
        Value::BlockArgument(Box::new(Value::Nil)),
        Value::BlockArgument(Box::new(Value::Nil)),
    ];

    let when = super::super::call_channels::native_arguments(&given);

    assert_eq!(when, Err(EvaluationError::ArgumentError));
}
