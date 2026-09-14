use super::{SourceEvaluator, body_yields};
use crate::EvaluationError;
use iris_runtime::Value;
use iris_syntax::{Expression, Statement};

mod wrappers;

fn run(evaluator: &mut SourceEvaluator, source: &str) -> Result<Value, EvaluationError> {
    let parsed = iris_parser::parse(source);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    evaluator.set_source(source);
    let result = evaluator.program(&parsed.program);
    assert!(result.is_ok(), "setup {source}: {result:?}");
    result
}

fn block(value: Expression) -> Expression {
    Expression::BlockArgument {
        value: Box::new(value),
    }
}

fn call(receiver: &str, selector: &str, arguments: Vec<Expression>) -> Expression {
    Expression::Call {
        callee: Box::new(Expression::Member {
            receiver: Box::new(Expression::Name(receiver.into())),
            selector: selector.into(),
        }),
        type_arguments: Vec::new(),
        arguments,
    }
}

#[test]
fn canonical_nil_is_supplied_when_block_default_would_run() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
    run(
        &mut given,
        "class Target { public fun run(&block = 42) { block } }; let target = Target.new(); nil",
    )?;
    let expression = call(
        "target",
        "run",
        vec![block(Expression::Literal("nil".into()))],
    );

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(when, Value::Nil);
    assert_eq!(
        given.expression(&call("target", "run", vec![]), &Default::default(), None)?,
        Value::Integer(42_u8.into())
    );
    Ok(())
}

#[test]
fn canonical_block_binds_when_between_positionals() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
    run(
        &mut given,
        "class Target { public fun run(left, right, &block) { %[left, right, block.call()] } }; let target = Target.new(); let callback = { || 9 }; nil",
    )?;
    let expression = call(
        "target",
        "run",
        vec![
            Expression::Literal("1".into()),
            block(Expression::Name("callback".into())),
            Expression::Literal("2".into()),
        ],
    );

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(when, crate::evaluate("%[1, 2, 9]")?);
    Ok(())
}

#[test]
fn duplicate_canonical_blocks_fail_before_defaults_and_body() -> Result<(), EvaluationError> {
    for first in [
        Expression::Literal("nil".into()),
        Expression::Name("callback".into()),
    ] {
        let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
        run(
            &mut given,
            "mut effects = %[]; class Target { public fun run(value = effects.append(:default), &block = nil) { effects.append(:body) } }; let target = Target.new(); let callback = { || 9 }; nil",
        )?;
        let expression = call(
            "target",
            "run",
            vec![block(first), block(Expression::Name("callback".into()))],
        );

        let when = given.expression(&expression, &Default::default(), None);

        assert_eq!(when, Err(EvaluationError::ArgumentError));
        assert_eq!(run(&mut given, "effects")?, crate::evaluate("%[]")?);
    }
    Ok(())
}

#[test]
fn method_missing_extracts_canonical_block_from_between_positionals() -> Result<(), EvaluationError>
{
    let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
    run(
        &mut given,
        "class Target { public fun method_missing(name, args, block) { %[args, block.call()] } }; let target = Target.new(); let callback = { || 9 }; nil",
    )?;
    let expression = call(
        "target",
        "absent",
        vec![
            Expression::Literal("1".into()),
            block(Expression::Name("callback".into())),
            Expression::Literal("2".into()),
        ],
    );

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(when, crate::evaluate("%[%[1, 2], 9]")?);
    Ok(())
}

#[test]
fn canonical_channels_and_identity_survive_nested_async_operands() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
    run(
        &mut given,
        "class Target { public fun run(callback = nil, &block = nil) { if block == nil { callback } else { block } } }; let target = Target.new(); let gate = Gate.new(); let callback = { || 9 }; nil",
    )?;
    let plain = call("target", "run", vec![Expression::Name("callback".into())]);
    let tagged = call(
        "target",
        "run",
        vec![block(Expression::Name("callback".into()))],
    );
    let body = vec![Statement::Expression(Expression::Array(vec![
        plain.clone(),
        Expression::Await(Box::new(Expression::Name("gate".into()))),
        tagged.clone(),
        call("target", "run", vec![block(plain)]),
        call("target", "run", vec![tagged]),
    ]))];
    let expected = given.expression(
        &Expression::Name("callback".into()),
        &Default::default(),
        None,
    )?;

    let when = given.start_async(body, Default::default(), None, None)?;
    run(&mut given, "Gate.complete(gate, 7)")?;
    given.drive_ready_continuations()?;

    let Value::Task(identity) = when else {
        panic!("expected task")
    };
    assert_eq!(
        given.tasks[&identity],
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            expected.clone(),
            Value::Integer(7_u8.into()),
            expected.clone(),
            expected.clone(),
            expected
        ])))
    );
    Ok(())
}

#[test]
fn program_preserves_block_when_source_is_unavailable() -> Result<(), EvaluationError> {
    let source =
        "class Target { public fun run(&block) { block.call() } }; Target.new().run() { || 9 }";
    let parsed = iris_parser::parse(source);
    let mut evaluator = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;

    let when = evaluator.program(&parsed.program)?;

    assert_eq!(when, Value::Integer(9_u8.into()));
    Ok(())
}

#[test]
fn block_argument_evaluation_preserves_transport() -> Result<(), EvaluationError> {
    let mut given = SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?;
    let expression = Expression::BlockArgument {
        value: Box::new(Expression::Literal("7".into())),
    };

    let when = given.expression(&expression, &Default::default(), None)?;

    assert_eq!(
        when,
        Value::BlockArgument(Box::new(Value::Integer(7_u8.into())))
    );
    Ok(())
}

#[test]
fn block_argument_yield_scan_visits_operand() {
    let given = [Statement::Expression(Expression::BlockArgument {
        value: Box::new(Expression::Yield(Some(Box::new(Expression::Literal(
            "7".into(),
        ))))),
    })];

    let when = body_yields(&given);

    assert!(when);
}
