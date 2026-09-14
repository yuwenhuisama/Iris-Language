use crate::{EvaluationError, Session, evaluate};
use iris_runtime::{ArrayRef, Value};

#[test]
fn block_channel_rejections_precede_body_effects() {
    for invocation in ["o.m({ :v })", "o.m({ :blk })", "o.m({ |x| :got })", "o.m()"] {
        let mut session = Session::new().unwrap();
        session.evaluate("mut calls = %[]; class A { public fun m(&b) { calls.append(:method); b.call() } } let o = A.new(); nil").unwrap();
        let result = session.evaluate(invocation);
        assert_eq!(result, Err(EvaluationError::ArgumentError), "{invocation}");
        assert_eq!(
            session.evaluate("calls"),
            Ok(Value::Array(ArrayRef::new(vec![])))
        );
    }
}

#[test]
fn typed_block_rejects_invariant_signature_mismatches_before_body_entry() {
    for invocation in [
        "o.m() { |x| :got }",
        "o.m() { |x: Integer| -> Object :got }",
        "o.m() { |x: String| -> Symbol :got }",
    ] {
        let mut session = Session::new().unwrap();
        session.evaluate("mut calls = %[]; class A { public fun m(&b: Block<(Integer) -> Symbol>) { calls.append(:method); b.call(1) } } let o = A.new(); nil").unwrap();
        let result = session.evaluate(invocation);
        assert_eq!(
            result,
            Err(EvaluationError::TypeContractError),
            "{invocation}"
        );
        assert_eq!(
            session.evaluate("calls"),
            Ok(Value::Array(ArrayRef::new(vec![])))
        );
    }
}

#[test]
fn required_block_cannot_be_omitted_from_a_full_parameter_signature() {
    for body in ["r", "%[a, b, k, kw]", "%[a, b, r, k, kw, blk]"] {
        let mut session = Session::new().unwrap();
        session.evaluate(&format!("mut calls = %[]; class A {{ public fun m(a, b = 2, *r, key k, **kw, &blk) {{ calls.append(:method); {body} }} }} nil")).unwrap();
        let result = session.evaluate("A.new().m(1, 9, k: 5)");
        assert_eq!(result, Err(EvaluationError::ArgumentError), "{body}");
        assert_eq!(
            session.evaluate("calls"),
            Ok(Value::Array(ArrayRef::new(vec![])))
        );
    }
}

#[test]
fn f_bounded_construction_rejects_raw_conformance() {
    let source = "contract Comparable<T> {} class Ok {} impl Ok for Comparable {} class Box<T> where T: Comparable<T> {} Box<Ok>.new()";
    assert_eq!(evaluate(source), Err(EvaluationError::TypeContractError));
}

#[test]
fn f_bounded_construction_rejects_wrong_closed_conformance() {
    let source = "contract Comparable<T> {} class Ok {} impl Ok for Comparable<String> {} class Box<T> where T: Comparable<T> {} Box<Ok>.new()";
    assert_eq!(evaluate(source), Err(EvaluationError::TypeContractError));
}

#[test]
fn generic_definition_cannot_construct_a_raw_instance() {
    let source = "class Box<T> where T: Object {} open class Box<T> where T: Object { public fun tag() -> Symbol { :b } } Box.new().tag()";
    assert_eq!(evaluate(source), Err(EvaluationError::TypeContractError));
}

#[test]
fn class_conformance_has_no_intrinsic_module_host_self() {
    for source in [
        "contract Comparable<T> {} class Ok {} impl Ok for Comparable<Self> {} class Box<T> where T: Comparable<T> {} Box<Ok>.new()",
        "contract Comparable<T> {} class Key {} impl Key for Comparable<Self> {} class Box<T> where T: Comparable<T> {} Box<Key>.new()",
    ] {
        assert_eq!(
            evaluate(source),
            Err(EvaluationError::NameError),
            "{source}"
        );
    }
}

#[test]
fn undefined_decorators_do_not_publish_a_target() {
    for decorator in ["one", "two", "logged", "sealed", "first", "second"] {
        let source = format!("@{decorator}() class Box {{ }}");
        let (result, published) = crate::evaluate_with_class_publication(&source, "Box");
        assert_eq!(result, Err(EvaluationError::NameError), "{source}");
        assert!(!published);
    }
}

#[test]
fn missing_plan_rejects_before_transform_effects() {
    let mut session = Session::new().unwrap();
    session.evaluate("mut calls = %[]; nil").unwrap();
    let result = session.evaluate("class Stamp {} impl Stamp for ClassDecorator { public fun transform(d, a, c) -> Transformation { calls.append(:transform); Transformation.empty } } @Stamp() class Box {}");
    assert_eq!(result, Err(EvaluationError::TypeContractError));
    assert_eq!(
        session.evaluate("calls"),
        Ok(Value::Array(ArrayRef::new(vec![])))
    );
    assert_eq!(session.evaluate("Box"), Err(EvaluationError::NameError));
}

#[test]
fn user_contract_cannot_replace_the_builtin_decorator_protocol() {
    let source = "contract ClassDecorator { fun transform(d, a, c) } class Stamp {} impl Stamp for ClassDecorator { public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Stamp() class Box {}";
    let (result, published) = crate::evaluate_with_class_publication(source, "Box");
    assert_eq!(
        result,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "static"
        })
    );
    assert!(!published);
}

#[test]
fn add_method_rejects_a_trailing_block_without_running_its_body() {
    let mut session = Session::new().unwrap();
    session.evaluate("mut calls = %[]; nil").unwrap();
    let result = session.evaluate_host("class Stamp {} impl Stamp for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty.add_method(:stamped) { calls.append(:body); 7 } } } @Stamp() class Box {}");
    assert_eq!(
        session.evaluate("calls"),
        Ok(Value::Array(ArrayRef::new(vec![])))
    );
    assert_eq!(session.evaluate("Box"), Err(EvaluationError::NameError));
    let Err(EvaluationError::HostException(exception)) = result else {
        panic!("expected owned ArgumentError metadata: {result:?}");
    };
    assert_eq!(exception.kind, crate::CoreErrorKind::ArgumentError);
    assert!(matches!(exception.value, Value::Object(_)));
    let Some(Value::ExceptionContext(_, value, ..)) = exception.context else {
        panic!("expected the failed transformation's exception context");
    };
    assert_eq!(*value, exception.value);
}

#[test]
fn generated_method_is_private_after_publication() {
    let mut session = Session::new().unwrap();
    session.evaluate("mut calls = %[]; class Stamp {} impl Stamp for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.add_method(:stamped, { calls.append(:body); 7 }) } } @Stamp() class Box { public fun read_stamp() { stamped() } } nil").unwrap();
    let result = session.evaluate("Box.new().stamped()");
    assert!(
        matches!(
            result,
            Err(EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::VisibilityDenied { .. }
                )
            ))
        ),
        "{result:?}"
    );
    assert_eq!(
        session.evaluate("calls"),
        Ok(Value::Array(ArrayRef::new(vec![])))
    );
    assert_eq!(
        session.evaluate("Box.new().read_stamp()"),
        Ok(Value::Integer(7_u8.into()))
    );
}

#[test]
fn generic_stored_property_remains_private_without_a_visibility_prefix() {
    let source = "class Box<T> { property held: Object = nil } Box<Integer>.new().held";
    let result = evaluate(source);
    assert!(
        matches!(
            result,
            Err(EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::VisibilityDenied { .. }
                )
            ))
        ),
        "{result:?}"
    );
}

#[test]
fn wrong_decorator_target_reports_static_kind_diagnostic() {
    let source = "class Stamp {} impl Stamp for ModuleDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Stamp() class Box {}";
    let (result, published) = crate::evaluate_with_class_publication(source, "Box");
    assert_eq!(
        result,
        Err(EvaluationError::DecoratorDiagnostic {
            code: "IRIS-DECORATOR-KIND",
            phase: "static"
        })
    );
    assert!(!published);
}
