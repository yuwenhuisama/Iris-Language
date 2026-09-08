#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "Test fixture shape failures must abort the test, not production execution"
)]

use iris_parser::{analyze, parse};
use iris_syntax::{Declaration, MethodDeclaration, Program, Statement, TypeExpression};

#[path = "binding_contracts/scopes.rs"]
mod scopes;
#[path = "binding_contracts/signatures.rs"]
mod signatures;

#[test]
fn prepares_unknown_binding_when_unrelated_diagnostic_exists() {
    let given = program("missing = 1; mut copied = external()");
    assert_eq!(
        analyze(&given)[0].code,
        "BINDING_UNRESOLVED_ASSIGNMENT_TARGET"
    );
    let when = iris_parser::prepare_bindings(&given).expect("backend owns unrelated diagnostics");
    assert!(matches!(
        &when.statements[1],
        Statement::Binding {
            annotation: None,
            ..
        }
    ));
}

#[test]
fn rejects_only_fixed_local_diagnostic_when_other_diagnostics_exist() {
    let given = program("missing = 1; mut value = 1; value = \"bad\"");
    let when = iris_parser::prepare_bindings(&given).expect_err("fixed local violation");
    assert_eq!(
        when.iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        ["BINDING_FIXED_LOCAL_TYPE"]
    );
}

#[test]
fn copies_seeded_static_contracts_without_runtime_inference() {
    let given = program(
        "mut copied = previous; mut wide_copy = wide; mut unknown_copy = unknown; mut dynamic_copy = dynamic; mut float_copy = single",
    );
    let contracts = vec![
        (
            "previous".into(),
            Some(TypeExpression::Name("Integer".into())),
        ),
        ("wide".into(), Some(TypeExpression::Name("Object".into()))),
        ("unknown".into(), None),
        (
            "dynamic".into(),
            Some(TypeExpression::Generic {
                name: "Dynamic".into(),
                arguments: vec![TypeExpression::Name("Object".into())],
            }),
        ),
        (
            "single".into(),
            Some(TypeExpression::Name("Float32".into())),
        ),
    ];
    let when =
        iris_parser::prepare_bindings_with_context(&given, &contracts).expect("session contracts");
    for (statement, (_, expected)) in when.statements.iter().zip(&contracts) {
        let Statement::Binding { annotation, .. } = statement else {
            panic!()
        };
        assert_eq!(annotation, expected);
    }
}

#[test]
fn rejects_seeded_fixed_contract_when_assignment_is_bad_literal() {
    for source in [
        "previous = \"bad\"",
        "mut copied = previous; copied = \"bad\"",
    ] {
        let given = program(source);
        let contracts = [(
            "previous".into(),
            Some(TypeExpression::Name("Integer".into())),
        )];
        let when = iris_parser::prepare_bindings_with_context(&given, &contracts)
            .expect_err("session type violation");
        assert_eq!(
            when.iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            ["BINDING_FIXED_LOCAL_TYPE"]
        );
    }
}

fn program(source: &str) -> Program {
    let parsed = parse(source);
    assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics);
    parsed.program
}

fn method(signature: &str) -> MethodDeclaration {
    let parsed = program(&format!(
        "class Test {{ public fun probe{signature} {{ nil }} }}"
    ));
    let Declaration::Class(class) = &parsed.declarations[0] else {
        panic!()
    };
    let Statement::Method(method) = &class.body[0] else {
        panic!()
    };
    method.clone()
}

#[test]
fn diagnoses_fixed_contract_when_assignment_is_provably_incompatible() {
    for source in [
        "mut port = 8080; port = \"9090\"",
        "mut port: Integer = 8080; port = \"9090\"",
        "mut port: Integer | String = 8080; port = true",
        "mut port = 1.0f32; port = 1.0f64",
        "mut port = (1); port = \"bad\"",
    ] {
        let given = program(source);
        let when = analyze(&given);
        assert_eq!(
            when.iter().map(|item| item.code).collect::<Vec<_>>(),
            ["BINDING_FIXED_LOCAL_TYPE"],
            "{source}"
        );
    }
}

#[test]
fn preserves_lexical_contract_when_preparing_execution_tree() {
    let given = program(
        "let first = 1; mut copied = first; mut single = 1.0f32; mut wide: Object = 1; mut alias = wide; mut uncertain = external(); mut branch = if true { 1 } else { \"x\" }",
    );
    let when = iris_parser::prepare_bindings(&given).expect("valid contracts");
    let annotations: Vec<_> = when
        .statements
        .iter()
        .map(|statement| {
            let Statement::Binding { annotation, .. } = statement else {
                panic!()
            };
            annotation.clone()
        })
        .collect();
    assert_eq!(
        annotations,
        vec![
            Some(TypeExpression::Name("Integer".into())),
            Some(TypeExpression::Name("Integer".into())),
            Some(TypeExpression::Name("Float32".into())),
            Some(TypeExpression::Name("Object".into())),
            Some(TypeExpression::Name("Object".into())),
            None,
            Some(TypeExpression::Union(vec![
                TypeExpression::Name("Integer".into()),
                TypeExpression::Name("String".into())
            ])),
        ]
    );
    assert_eq!(
        when.entries,
        when.statements
            .iter()
            .cloned()
            .map(iris_syntax::ProgramEntry::Statement)
            .collect::<Vec<_>>()
    );
    assert!(matches!(
        &given.statements[0],
        Statement::Binding {
            annotation: None,
            ..
        }
    ));
}

#[test]
fn accepts_compound_assignment_when_result_not_rhs_satisfies_contract() {
    let given = program("mut number: Float64 = 1.0; number += 2");
    let when = analyze(&given);
    assert!(when.is_empty(), "{when:?}");
}

#[test]
fn rejects_compound_assignment_when_operation_widens_integer() {
    let given = program("mut number = 2; number /= 2");
    let when = analyze(&given);
    assert_eq!(
        when.iter().map(|item| item.code).collect::<Vec<_>>(),
        ["BINDING_FIXED_LOCAL_TYPE"]
    );
}

#[test]
fn accepts_integer_power_when_exponent_is_nonnegative_literal() {
    let given = program("mut number = 2; number **= 2");
    let when = analyze(&given);
    assert!(when.is_empty(), "{when:?}");
}
