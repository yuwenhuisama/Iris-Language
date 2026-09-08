#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "Test fixture failures abort the test"
)]

use iris_parser::{parse, prepare_bindings};
use iris_syntax::{
    Declaration, MethodDeclaration, Program, ProgramEntry, Statement, TypeExpression,
};

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

fn method(declaration: &Declaration) -> &MethodDeclaration {
    let body = match declaration {
        Declaration::Class(value) => &value.body,
        Declaration::Contract(value) => &value.body,
        _ => panic!("expected method owner"),
    };
    let Statement::Method(method) = &body[0] else {
        panic!("expected method")
    };
    method
}

#[test]
fn rejects_literal_write_when_binding_uses_transparent_alias() {
    let given =
        program("type Port = Integer; mut port: Port = 8080; let written = port = \"bad\"; port");
    let when = prepare_bindings(&given).expect_err("alias fixed contract");
    assert_eq!(
        when.iter().map(|item| item.code).collect::<Vec<_>>(),
        ["BINDING_FIXED_LOCAL_TYPE"]
    );
}

#[test]
fn accepts_equivalent_replacement_when_promise_uses_alias() {
    let given = program(
        "type Port = Integer; class Rule { public fun total() -> Port { 1 } }; open class Rule { public override fun total() -> Integer { 2 } }",
    );
    let when = prepare_bindings(&given).expect("equivalent signatures");
    assert!(iris_syntax::method_signature_compatible(
        method(&when.declarations[2]),
        method(&when.declarations[1]),
        |source, target| source == target,
    ));
}

#[test]
fn preserves_source_when_dynamic_binding_contract_is_normalized() {
    let given = program("type Port = Integer; mut port: Port = external(); port = external()");
    let original = given.clone();
    let when = prepare_bindings(&given).expect("dynamic source needs a guard");
    assert_eq!(
        annotation(&when.statements[0]),
        Some(TypeExpression::Name("Integer".into()))
    );
    assert_eq!(given, original);
    assert_eq!(when.declarations, given.declarations);
    assert_eq!(
        when.entries[1],
        ProgramEntry::Statement(when.statements[0].clone())
    );
}

#[test]
fn rejects_outside_union_when_alias_targets_an_alias_union() {
    let given = program(
        "type Port = Integer; type Input = Port | String; mut port: Input = 1; port = true",
    );
    let when = prepare_bindings(&given).expect_err("outside alias union");
    assert_eq!(when[0].code, "BINDING_FIXED_LOCAL_TYPE");
}

#[test]
fn keeps_wide_and_unknown_contracts_when_aliases_are_prepared() {
    let given = program(
        "type Wide = Object; type Loose = Dynamic<Object>; type Port = Integer; mut wide: Wide = 1; wide = true; mut loose: Loose = 1; loose = \"text\"; mut unknown = external(); unknown = true; mut nested: Array<Port> = external()",
    );
    let when = prepare_bindings(&given).expect("wide aliases accept values");
    assert_eq!(
        annotation(&when.statements[0]),
        Some(TypeExpression::Name("Object".into()))
    );
    assert_eq!(
        annotation(&when.statements[2]),
        Some(TypeExpression::Generic {
            name: "Dynamic".into(),
            arguments: vec![TypeExpression::Name("Object".into())]
        })
    );
    assert_eq!(annotation(&when.statements[4]), None);
    assert_eq!(
        annotation(&when.statements[6]),
        Some(TypeExpression::Generic {
            name: "Array".into(),
            arguments: vec![TypeExpression::Name("Integer".into())]
        })
    );
}

#[test]
fn resolves_bodyless_parameter_and_return_aliases_when_prepared() {
    let given = program("type Port = Integer; contract Reader { fun read(value: Port) -> Port }");
    let when = prepare_bindings(&given).expect("bodyless signature");
    let signature = method(&when.declarations[1]);
    assert_eq!(
        signature.parameters[0].annotation,
        Some(TypeExpression::Name("Integer".into()))
    );
    assert_eq!(
        signature.return_type,
        Some(TypeExpression::Name("Integer".into()))
    );
    assert!(signature.body.is_none());
    assert_eq!(
        when.entries[1],
        ProgramEntry::Declaration(when.declarations[1].clone())
    );
}
