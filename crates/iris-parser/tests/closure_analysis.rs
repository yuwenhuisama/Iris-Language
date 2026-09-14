use iris_parser::{parse, prepare_bindings};
use iris_syntax::{Expression, Program, Statement, TypeExpression};

fn program(source: &str) -> Program {
    let parsed = parse(source);
    assert!(
        parsed.program_accepted,
        "{source}: {:?}",
        parsed.diagnostics
    );
    parsed.program
}

fn value(statement: &Statement) -> &Expression {
    let Statement::Binding { value, .. } = statement else {
        panic!("expected binding")
    };
    value
}

fn annotation(statement: &Statement) -> &Option<TypeExpression> {
    let Statement::Binding { annotation, .. } = statement else {
        panic!("expected binding")
    };
    annotation
}

#[test]
fn preserves_canonical_signature_when_aliases_and_parameter_channels_are_prepared() {
    let header = "value: Port, optional: Port = 1, *rest: Port, key named: Port, **keywords: Port, &block: Block<() -> Port> = nil";
    let body = "mut copied = value; mut items = rest; mut options = keywords; mut callback = block";
    let given = program(&format!(
        "type Port = Integer; fun probe({header}) -> Port {{ {body} }}; let closure = {{ async |{header}| -> Port; {body} }}"
    ));

    let when = prepare_bindings(&given).expect("valid callable signatures");

    let Statement::Method(method) = &when.statements[0] else {
        panic!("expected method")
    };
    let Expression::Closure {
        parameters,
        full_parameters,
        is_async,
        has_header,
        return_type,
        body,
    } = value(&when.statements[1])
    else {
        panic!("expected closure")
    };
    assert_eq!(full_parameters, &method.parameters);
    assert_eq!(return_type, &method.return_type);
    assert_eq!(body, method.body.as_ref().expect("method body"));
    assert_eq!(
        parameters,
        &["value", "optional", "rest", "named", "keywords", "block"]
    );
    assert!(*is_async);
    assert!(*has_header);
    let Expression::Closure {
        full_parameters, ..
    } = value(&given.statements[1])
    else {
        panic!("expected original closure")
    };
    assert_eq!(
        full_parameters[0].annotation,
        Some(TypeExpression::Name("Port".into()))
    );
}

#[test]
fn resolves_aliases_when_default_contains_nested_closure() {
    let given = program(
        "type Port = Integer; let callback = { |nested = { |value: Port| -> Port; mut copied: Port = value; copied }| -> Object; nested }",
    );

    let when = prepare_bindings(&given).expect("nested default closure");

    let Expression::Closure {
        full_parameters, ..
    } = value(&when.statements[0])
    else {
        panic!("expected outer closure")
    };
    let Expression::Closure {
        full_parameters,
        return_type,
        body,
        ..
    } = full_parameters[0]
        .default
        .as_ref()
        .expect("default closure")
    else {
        panic!("expected nested closure")
    };
    let integer = Some(TypeExpression::Name("Integer".into()));
    assert_eq!(full_parameters[0].annotation, integer);
    assert_eq!(return_type, &integer);
    assert_eq!(annotation(&body[0]), &integer);
}

#[test]
fn uses_earlier_contract_when_default_closure_captures_shadowing_parameter() {
    let given = program(
        "let earlier = 'outer'; let callback = { |earlier: Integer, nested = { || -> Object; mut copied = earlier; copied }| -> Object; earlier }",
    );

    let when = prepare_bindings(&given).expect("earlier parameter scope");

    let Expression::Closure {
        full_parameters, ..
    } = value(&when.statements[1])
    else {
        panic!("expected outer closure")
    };
    let Expression::Closure { body, .. } = full_parameters[1].default.as_ref().expect("default")
    else {
        panic!("expected default closure")
    };
    assert_eq!(
        annotation(&body[0]),
        &Some(TypeExpression::Name("Integer".into()))
    );
}

#[test]
fn uses_outer_contract_when_default_precedes_own_or_later_parameter() {
    for header in [
        "current: Object = { || -> Object; mut copied = current; copied }",
        "nested = { || -> Object; mut copied = current; copied }, current: Integer = 1",
    ] {
        let given = program(&format!(
            "let current = 'outer'; let callback = {{ |{header}| -> Object; current }}"
        ));

        let when = prepare_bindings(&given).expect("default activation order");

        let Expression::Closure {
            full_parameters, ..
        } = value(&when.statements[1])
        else {
            panic!("expected outer closure")
        };
        let Expression::Closure { body, .. } =
            full_parameters[0].default.as_ref().expect("default")
        else {
            panic!("expected default closure")
        };
        assert_eq!(
            annotation(&body[0]),
            &Some(TypeExpression::Name("String".into())),
            "{header}"
        );
    }
}

#[test]
fn preserves_unknown_contract_when_nested_parameter_shadows_earlier_parameter() {
    let given = program(
        "let callback = { |earlier: Integer, nested = { |earlier| -> Object; mut copied = earlier; copied }| -> Object; mut copied = earlier; copied }",
    );

    let when = prepare_bindings(&given).expect("nested parameter shadowing");

    let Expression::Closure {
        full_parameters,
        body,
        ..
    } = value(&when.statements[0])
    else {
        panic!("expected outer closure")
    };
    assert_eq!(
        annotation(&body[0]),
        &Some(TypeExpression::Name("Integer".into()))
    );
    let Expression::Closure { body, .. } = full_parameters[1].default.as_ref().expect("default")
    else {
        panic!("expected default closure")
    };
    assert_eq!(annotation(&body[0]), &None);
}
