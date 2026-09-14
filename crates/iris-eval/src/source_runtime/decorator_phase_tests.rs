use super::{EvaluationError, SourceEvaluator};
use iris_runtime::Value;
use iris_runtime::decorator_protocol::{
    DecoratorKind, DecoratorPhase, DecoratorReason, DecoratorValue, Operation,
};

#[test]
fn operation_factories_record_persistent_order_without_running_bodies() {
    let mut evaluator = SourceEvaluator::new_in_package("phase-tests").unwrap();
    evaluator.decorator_phase_stack.push(DecoratorPhase::new(
        DecoratorKind::Method,
        DecoratorReason::Origin,
    ));
    let source = "let first = Transformation.wrap_method({ |invocation, next| -> Object raise :executed }); first.wrap_method({ |invocation, next| -> Object raise :executed })";
    let parsed = iris_parser::parse(source);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let Value::Decorator(record) = evaluator.program(&parsed.program).unwrap() else {
        panic!("missing record")
    };
    let DecoratorValue::Transformation(record) = record.as_ref() else {
        panic!("wrong record")
    };
    assert!(matches!(
        record.operations(),
        [
            Operation::WrapMethod(Value::Closure(_)),
            Operation::WrapMethod(Value::Closure(_))
        ]
    ));
    let first = evaluator.names.get("first").unwrap().value();
    let Value::Decorator(first) = first else {
        panic!("missing first")
    };
    let DecoratorValue::Transformation(first) = first.as_ref() else {
        panic!("wrong first")
    };
    assert_eq!(first.operations().len(), 1);
}

#[test]
fn phase_kind_diagnostic_preserves_runtime_validation_stage() {
    let mut evaluator = SourceEvaluator::new_in_package("phase-tests").unwrap();
    evaluator.decorator_phase_stack.push(DecoratorPhase::new(
        DecoratorKind::Class,
        DecoratorReason::Open,
    ));
    let class = evaluator.kernel.core_class("Transformation").unwrap();
    assert_eq!(
        evaluator.decorator_core_send(&Value::Class(class), "wrap_method", &[Value::Nil]),
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "candidate validation"
        })
    );
}

#[test]
fn failed_constructor_pops_phase_before_next_evaluation() {
    let mut evaluator = SourceEvaluator::new_in_package("phase-tests").unwrap();
    let source = "class Stamp { fun initialize() { raise :failure } }; impl Stamp for ClassDecorator { public fun plan(declaration, arguments) -> Plan { Plan.empty }; public fun transform(declaration, arguments, context) -> Transformation { Transformation.empty } }; 0";
    evaluator
        .program(&iris_parser::parse(source).program)
        .unwrap();
    let class = evaluator.class_name("Stamp").unwrap().unwrap();
    let metadata = evaluator.class_decorator_metadata(class).unwrap();
    let decorator = iris_syntax::Decorator {
        name: "Stamp".into(),
        arguments: Vec::new(),
    };
    let result = evaluator.execute_decorator_phase(
        &decorator,
        super::decorator_phases::PhaseTarget {
            kind: DecoratorKind::Class,
            reason: DecoratorReason::Origin,
            metadata,
            candidate: None,
        },
    );
    assert!(result.is_err());
    assert!(evaluator.decorator_phase_stack.is_empty());
    assert!(evaluator.open_target.is_none());
    assert!(evaluator.decorating_target.is_none());
}
