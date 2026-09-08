use super::program;
use iris_syntax::{Declaration, Statement, TypeExpression};

#[test]
fn preserves_captured_and_parameter_contracts_when_preparing_method_bodies() {
    let given = program(
        "class Test { public fun probe(value: Integer) { mut local = value; let callback = { || -> Integer; mut captured = local; captured }; callback } }",
    );
    let when = iris_parser::prepare_bindings(&given).expect("valid method");
    let Declaration::Class(class) = &when.declarations[0] else {
        panic!()
    };
    let Statement::Method(method) = &class.body[0] else {
        panic!()
    };
    let body = method.body.as_ref().unwrap();
    let Statement::Binding { annotation, .. } = &body[0] else {
        panic!()
    };
    assert_eq!(annotation, &Some(TypeExpression::Name("Integer".into())));
    let Statement::Binding {
        value: iris_syntax::Expression::Closure { body, .. },
        ..
    } = &body[1]
    else {
        panic!()
    };
    let Statement::Binding { annotation, .. } = &body[0] else {
        panic!()
    };
    assert_eq!(annotation, &Some(TypeExpression::Name("Integer".into())));
    assert_eq!(
        when.entries[0],
        iris_syntax::ProgramEntry::Declaration(when.declarations[0].clone())
    );
}

#[test]
fn infers_branch_local_names_when_preparing_union_contract() {
    let given = program(
        "mut result = if true { let branch = 1; branch } else { let branch = \"x\"; branch }",
    );
    let when = iris_parser::prepare_bindings(&given).expect("valid branches");
    let Statement::Binding { annotation, .. } = &when.statements[0] else {
        panic!()
    };
    assert_eq!(
        annotation,
        &Some(TypeExpression::Union(vec![
            TypeExpression::Name("Integer".into()),
            TypeExpression::Name("String".into())
        ]))
    );
}

#[test]
fn leaves_shadowing_closure_parameters_unknown_when_outer_name_is_typed() {
    let given =
        program("let value = 1; let callback = { |value| -> Object; mut copied = value; copied }");
    let when = iris_parser::prepare_bindings(&given).expect("valid shadowing");
    let Statement::Binding {
        value: iris_syntax::Expression::Closure { body, .. },
        ..
    } = &when.statements[1]
    else {
        panic!()
    };
    assert!(matches!(
        &body[0],
        Statement::Binding {
            annotation: None,
            ..
        }
    ));
}

#[test]
fn preserves_explicit_union_and_dynamic_when_values_are_narrower() {
    let given = program(
        "mut wide: Integer | String = 1; mut copy = wide; mut dynamic: Dynamic<Object> = 1; mut other = dynamic; wide = \"x\"; dynamic = true",
    );
    let when = iris_parser::prepare_bindings(&given).expect("explicit wider types");
    for (original, copied) in [(0, 1), (2, 3)] {
        let Statement::Binding {
            annotation: expected,
            ..
        } = &given.statements[original]
        else {
            panic!()
        };
        let Statement::Binding {
            annotation: actual, ..
        } = &when.statements[copied]
        else {
            panic!()
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn does_not_capture_outer_local_when_preparing_method() {
    let given = program(
        "let source = 1; class Test { public fun probe() { mut copied = source; copied } }",
    );
    let when = iris_parser::prepare_bindings(&given).expect("unknown noncaptured name");
    let Declaration::Class(class) = &when.declarations[0] else {
        panic!()
    };
    let Statement::Method(method) = &class.body[0] else {
        panic!()
    };
    assert!(matches!(
        &method.body.as_ref().unwrap()[0],
        Statement::Binding {
            annotation: None,
            ..
        }
    ));
}

#[test]
fn shadows_outer_local_when_catch_binding_has_same_name() {
    let given = program("let error = 1; try { raise :bad } catch error { mut copied = error }");
    let when = iris_parser::prepare_bindings(&given).expect("unknown caught value");
    let Statement::Try { catches, .. } = &when.statements[1] else {
        panic!()
    };
    assert!(matches!(
        &catches[0].body[0],
        Statement::Binding {
            annotation: None,
            ..
        }
    ));
}

#[test]
fn infers_rest_binding_as_container_when_parameter_annotation_is_element_type() {
    let given = program(
        "class Test { public fun probe(*values: Integer) { mut copied = values; copied } }",
    );
    let when = iris_parser::prepare_bindings(&given).expect("rest container");
    let Declaration::Class(class) = &when.declarations[0] else {
        panic!()
    };
    let Statement::Method(method) = &class.body[0] else {
        panic!()
    };
    let Statement::Binding { annotation, .. } = &method.body.as_ref().unwrap()[0] else {
        panic!()
    };
    assert_eq!(annotation, &Some(TypeExpression::Name("Array".into())));
}
