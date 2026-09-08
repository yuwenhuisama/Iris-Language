#![expect(
    clippy::expect_used,
    reason = "tests require valid source and preparation"
)]

use iris_syntax::{Statement, TypeExpression};

#[test]
fn setter_assignment_initializers_do_not_inherit_rhs_contracts() {
    for initializer in [
        "receiver.value = 1",
        "receiver[0] = 1",
        "receiver.value = other.value = 1",
        "receiver.value += 1",
        "receiver[0] += 1",
        "receiver.value ||= 1",
        "receiver[0] &&= 1",
    ] {
        let given = iris_parser::parse(&format!("let result = ({initializer})"));
        assert!(given.is_clean(), "{given:?}");
        let when = iris_parser::prepare_bindings(&given.program).expect("valid initializer");
        assert!(
            matches!(
                &when.statements[0],
                Statement::Binding {
                    annotation: None,
                    ..
                }
            ),
            "{initializer}: {when:?}"
        );
    }
}

#[test]
fn direct_storage_assignment_initializers_keep_stored_value_contracts() {
    for source in [
        "mut local = 0; let result = (local = 1)",
        "let result = (@value = 1)",
        "let result = (@@value = 1)",
        "let result = ($value = 1)",
    ] {
        let given = iris_parser::parse(source);
        assert!(given.is_clean(), "{given:?}");
        let when = iris_parser::prepare_bindings(&given.program).expect("stored value contract");
        assert!(
            matches!(
                when.statements.last(),
                Some(Statement::Binding {
                    annotation: Some(TypeExpression::Name(name)),
                    ..
                }) if name == "Integer"
            ),
            "{source}: {when:?}"
        );
    }
}

#[test]
fn setter_result_does_not_cause_false_fixed_local_diagnostic() {
    let given = iris_parser::parse("mut result = :initial; result = receiver.value = 1");
    assert!(given.is_clean(), "{given:?}");
    let when = iris_parser::prepare_bindings(&given.program);
    assert!(when.is_ok(), "{when:?}");
}

#[test]
fn nil_initializer_still_rejects_incompatible_logical_assignment() {
    let given = iris_parser::parse("mut value = nil; value ||= 7");
    assert!(given.is_clean(), "{given:?}");
    let when = iris_parser::prepare_bindings(&given.program).expect_err("fixed Nil contract");
    assert_eq!(
        when.iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        ["BINDING_FIXED_LOCAL_TYPE"]
    );
}
