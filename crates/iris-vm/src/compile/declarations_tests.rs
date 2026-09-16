#![expect(clippy::expect_used, reason = "test fixtures require accepted syntax")]

use super::declarations::written_constructions;
use iris_syntax::{
    Constraint, Declaration, Expression, ImplDeclaration, MethodDeclaration, MethodKind, Parameter,
    ParameterCategory, Statement, TypeExpression, Visibility,
};

fn construction(argument: &str) -> Expression {
    Expression::ClosedGeneric {
        name: "Box".into(),
        arguments: vec![TypeExpression::Name(argument.into())],
    }
}

fn construction_type(argument: &str) -> TypeExpression {
    TypeExpression::Typeof(Box::new(construction(argument)))
}

#[test]
fn construction_discovery_recurses_through_v136_nodes() {
    let declarations = vec![
        Declaration::Class(iris_syntax::ClassDeclaration {
            decorators: Vec::new(),
            reopen: false,
            name: "Holder".into(),
            parameters: Vec::new(),
            extends: None,
            implements: Vec::new(),
            mixins: Vec::new(),
            constraints: Vec::new(),
            meta_deny: Vec::new(),
            body: vec![Statement::InstanceField {
                mutable: false,
                name: "value".into(),
                annotation: Some(construction_type("FieldAnnotation")),
                value: Expression::NonNull(Box::new(construction("FieldInitializer"))),
            }],
        }),
        Declaration::Impl(ImplDeclaration {
            target: construction_type("ImplTarget"),
            contract: construction_type("ImplContract"),
            constraints: vec![Constraint {
                parameter: "T".into(),
                bound: construction_type("ImplConstraint"),
            }],
            methods: vec![MethodDeclaration {
                decorators: Vec::new(),
                is_async: false,
                is_override: false,
                impl_contract: None,
                kind: MethodKind::Instance,
                selector: "show".into(),
                type_parameters: Vec::new(),
                parameters: vec![Parameter {
                    name: "value".into(),
                    category: ParameterCategory::Positional,
                    annotation: Some(construction_type("MethodParameter")),
                    default: Some(construction("MethodDefault")),
                }],
                return_type: Some(construction_type("MethodReturn")),
                visibility: Visibility::Public,
                body: Some(vec![Statement::Expression(construction("MethodBody"))]),
            }],
        }),
    ];

    let found = written_constructions(&declarations, &[], "Box");

    assert_eq!(
        found,
        [
            "FieldAnnotation",
            "FieldInitializer",
            "ImplTarget",
            "ImplContract",
            "ImplConstraint",
            "MethodReturn",
            "MethodParameter",
            "MethodDefault",
            "MethodBody",
        ]
        .map(|name| vec![TypeExpression::Name(name.into())])
    );
}

#[test]
fn construction_discovery_recurses_through_safe_navigation_parts() {
    // Given
    let safe = Expression::SafeNavigation {
        receiver: Box::new(construction("Receiver")),
        parts: vec![
            iris_syntax::PostfixPart::Member {
                selector: "call".into(),
                safe: true,
            },
            iris_syntax::PostfixPart::Call {
                type_arguments: vec![construction_type("TypeArgument")],
                arguments: vec![construction("Argument")],
            },
            iris_syntax::PostfixPart::Index(Box::new(construction("Index"))),
            iris_syntax::PostfixPart::TrailingBlock(Box::new(construction("Trailing"))),
        ],
    };

    // When
    let found = written_constructions(&[], &[Statement::Expression(safe)], "Box");

    // Then
    assert_eq!(
        found,
        ["Receiver", "TypeArgument", "Argument", "Index", "Trailing"]
            .map(|name| vec![TypeExpression::Name(name.into())])
    );
}

#[test]
fn v136_runtime_forms_lower_to_vm_metadata() {
    let field = super::compile("class Pair { let @left: Integer = 1 } Pair.new()")
        .expect("instance field lowers");
    let implementation = super::compile("contract Show { fun show() -> String } class Box {} impl Box for Show { public fun show() -> String { \"box\" } } Box.new().show()")
        .expect("top-level impl lowers");

    assert_eq!(field.classes[0].instance_fields.len(), 1);
    assert_eq!(field.classes[0].instance_fields[0].name, "@left");
    assert_eq!(implementation.classes[0].static_impls, ["show"]);
    assert_eq!(implementation.classes[0].qualified_impls.len(), 1);
}

#[test]
fn non_null_lowers_to_a_dedicated_instruction() {
    let program = super::compile("1!").expect("non-null lowers");

    assert!(matches!(
        program.instructions.last(),
        Some(super::Instruction::AssertNonNil { source: 0, .. })
    ));
}
