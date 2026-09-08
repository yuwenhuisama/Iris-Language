#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "Test fixture failures abort the test"
)]

use iris_parser::{analyze, parse, prepare_bindings, prepare_bindings_with_aliases};
use iris_syntax::{Declaration, Program, Statement, TypeAliasDeclaration, TypeExpression};

fn program(source: &str) -> Program {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics);
    parsed.program
}

fn annotation(statement: &Statement) -> Option<TypeExpression> {
    let Statement::Binding { annotation, .. } = statement else {
        panic!("expected binding")
    };
    annotation.clone()
}

#[test]
fn resolves_seeded_cells_when_alias_is_from_previous_chunk() {
    let given = program("mut copied = previous; copied = \"bad\"");
    let aliases = [TypeAliasDeclaration {
        name: "Port".into(),
        parameters: Vec::new(),
        target: TypeExpression::Name("Integer".into()),
    }];
    let contracts = [("previous".into(), Some(TypeExpression::Name("Port".into())))];
    let when = prepare_bindings_with_aliases(&given, &contracts, &aliases)
        .expect_err("prior static alias contract");
    assert_eq!(when[0].code, "BINDING_FIXED_LOCAL_TYPE");
}

#[test]
fn resolves_new_annotations_when_alias_is_from_previous_chunk() {
    let given = program("mut port: Port = external()");
    let aliases = [TypeAliasDeclaration {
        name: "Port".into(),
        parameters: Vec::new(),
        target: TypeExpression::Name("Integer".into()),
    }];
    let when = prepare_bindings_with_aliases(&given, &[], &aliases).expect("prior alias");
    assert_eq!(
        annotation(&when.statements[0]),
        Some(TypeExpression::Name("Integer".into()))
    );
}

#[test]
fn preserves_cycles_when_existing_checker_owns_alias_diagnostics() {
    for source in [
        "type Loop = Loop; mut value: Loop = external()",
        "type Loop = Array<Loop>; mut value: Loop = external()",
        "type First = Second; type Second = First; mut value: First = external()",
    ] {
        let given = program(source);
        let when = prepare_bindings(&given).expect("only fixed local errors are returned");
        assert_eq!(when, given);
    }
    let given = program("type Loop = Loop; mut value: Loop = external()");
    assert_eq!(analyze(&given)[0].code, "RECURSIVE_TYPE_ALIAS");
}

#[test]
fn resolves_generic_aliases_when_arguments_are_closed() {
    let given = program(
        "type Port = Integer; type Optional<T> = T | Nil; type Ports<T> = Array<Optional<T>>; mut value: Ports<Port> = external()",
    );
    let when = prepare_bindings(&given).expect("closed alias arguments");
    assert_eq!(
        annotation(&when.statements[0]),
        Some(TypeExpression::Generic {
            name: "Array".into(),
            arguments: vec![TypeExpression::Union(vec![
                TypeExpression::Name("Integer".into()),
                TypeExpression::Name("Nil".into())
            ])],
        })
    );
}

#[test]
fn preserves_generic_parameters_when_they_shadow_alias_names() {
    let given = program(
        "type Port = Integer; type Alias = Port; class Box<Port> { public fun first(value: Port) -> Port { value }; public fun second(value: Alias) -> Alias { value } }",
    );
    let when = prepare_bindings(&given).expect("generic scope");
    let Declaration::Class(class) = &when.declarations[2] else {
        panic!("class")
    };
    let Statement::Method(first) = &class.body[0] else {
        panic!("first method")
    };
    let Statement::Method(second) = &class.body[1] else {
        panic!("second method")
    };
    assert_eq!(first.return_type, Some(TypeExpression::Name("Port".into())));
    assert_eq!(
        second.return_type,
        Some(TypeExpression::Name("Integer".into()))
    );
}

#[test]
fn accepts_member_write_when_alias_union_contains_the_value() {
    let given = program(
        "type Port = Integer; type Input = Port | String; mut value: Input = 1; value = \"ok\"",
    );
    let when = prepare_bindings(&given).expect("union member");
    assert_eq!(
        annotation(&when.statements[0]),
        Some(TypeExpression::Union(vec![
            TypeExpression::Name("Integer".into()),
            TypeExpression::Name("String".into())
        ]))
    );
}

#[test]
fn resolves_nested_callable_annotations_when_method_contains_closure() {
    let given = program(
        "type Port = Integer; type Callback = Closure<(Port) -> Port>; class Runner { public fun run(callback: Callback) -> Port { let nested = { |value| -> Port; mut copied: Port = value; copied }; 1 } }",
    );
    let when = prepare_bindings(&given).expect("nested boundaries");
    let Declaration::Class(class) = &when.declarations[2] else {
        panic!("class")
    };
    let Statement::Method(method) = &class.body[0] else {
        panic!("method")
    };
    assert_eq!(
        method.parameters[0].annotation,
        Some(TypeExpression::Generic {
            name: "Closure".into(),
            arguments: vec![TypeExpression::Function {
                parameters: vec![TypeExpression::Name("Integer".into())],
                result: Box::new(TypeExpression::Name("Integer".into())),
            }],
        })
    );
    let Statement::Binding {
        value: iris_syntax::Expression::Closure {
            return_type, body, ..
        },
        ..
    } = &method.body.as_ref().expect("body")[0]
    else {
        panic!("closure")
    };
    assert_eq!(return_type, &Some(TypeExpression::Name("Integer".into())));
    assert_eq!(
        annotation(&body[0]),
        Some(TypeExpression::Name("Integer".into()))
    );
}
