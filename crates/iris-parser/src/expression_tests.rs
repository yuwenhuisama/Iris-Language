use crate::parse;
use iris_syntax::{BinaryOperator, Expression, PostfixPart, Statement, UnaryOperator};

#[test]
fn parses_array_literals_with_named_infix_and_unary_elements() {
    // Given
    let source = "%[-5 div 2, 5 div -2, -5 div -2]";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.program.statements,
        [Statement::Expression(Expression::Array(vec![
            named_infix(negate("5"), Expression::Literal("2".into())),
            named_infix(Expression::Literal("5".into()), negate("2")),
            named_infix(negate("5"), negate("2")),
        ]))]
    );
}

#[test]
fn parses_empty_and_trailing_comma_arrays() {
    // Given
    let source = "%[]; %[1,]";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.program.statements,
        [
            Statement::Expression(Expression::Array(Vec::new())),
            Statement::Expression(Expression::Array(vec![Expression::Literal("1".into())])),
        ]
    );
}

#[test]
fn reports_a_parse_diagnostic_when_an_array_element_is_missing() {
    // Given
    let source = "%[1,";

    // When
    let result = parse(source);

    // Then
    assert!(!result.program_accepted);
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn parses_symbols_and_postfix_chains() {
    // Given
    let source = ":added; :sentinel; Float64.from_bits(0x0000000000000000); obj.method; obj.method(); a.b(c).d";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.program.statements,
        [
            Statement::Expression(Expression::Symbol("added".into())),
            Statement::Expression(Expression::Symbol("sentinel".into())),
            Statement::Expression(Expression::Call {
                callee: Box::new(Expression::Member {
                    receiver: Box::new(Expression::Name("Float64".into())),
                    selector: "from_bits".into(),
                }),
                type_arguments: Vec::new(),
                arguments: vec![Expression::Literal("0x0000000000000000".into())],
            }),
            Statement::Expression(Expression::Member {
                receiver: Box::new(Expression::Name("obj".into())),
                selector: "method".into(),
            }),
            Statement::Expression(Expression::Call {
                callee: Box::new(Expression::Member {
                    receiver: Box::new(Expression::Name("obj".into())),
                    selector: "method".into(),
                }),
                type_arguments: Vec::new(),
                arguments: Vec::new(),
            }),
            Statement::Expression(Expression::Member {
                receiver: Box::new(Expression::Call {
                    callee: Box::new(Expression::Member {
                        receiver: Box::new(Expression::Name("a".into())),
                        selector: "b".into(),
                    }),
                    type_arguments: Vec::new(),
                    arguments: vec![Expression::Name("c".into())],
                }),
                selector: "d".into(),
            }),
        ],
        "{result:#?}"
    );
}

#[test]
fn parses_safe_navigation_as_one_guarded_postfix_chain() {
    // Given
    let source = "a?.b; a?.b?.c; a?.ready?(); a?.save!(); a.ready?(); a.save!(); a?.b.c()[index]; a?.each() { work() }";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.program.statements,
        [
            safe_chain("a", vec![safe_member("b")]),
            safe_chain("a", vec![safe_member("b"), safe_member("c")]),
            safe_chain("a", vec![safe_member("ready?"), call(Vec::new())]),
            safe_chain("a", vec![safe_member("save!"), call(Vec::new())]),
            Statement::Expression(Expression::Call {
                callee: Box::new(Expression::Member {
                    receiver: Box::new(Expression::Name("a".into())),
                    selector: "ready?".into(),
                }),
                type_arguments: Vec::new(),
                arguments: Vec::new(),
            }),
            Statement::Expression(Expression::Call {
                callee: Box::new(Expression::Member {
                    receiver: Box::new(Expression::Name("a".into())),
                    selector: "save!".into(),
                }),
                type_arguments: Vec::new(),
                arguments: Vec::new(),
            }),
            safe_chain(
                "a",
                vec![
                    safe_member("b"),
                    member("c"),
                    call(Vec::new()),
                    PostfixPart::Index(Box::new(Expression::Name("index".into()))),
                ],
            ),
            safe_chain(
                "a",
                vec![
                    safe_member("each"),
                    call(Vec::new()),
                    PostfixPart::TrailingBlock(Box::new(Expression::BlockArgument {
                        value: Box::new(Expression::Closure {
                            parameters: Vec::new(),
                            full_parameters: Vec::new(),
                            is_async: false,
                            return_type: None,
                            has_header: false,
                            body: vec![Statement::Expression(Expression::Call {
                                callee: Box::new(Expression::Name("work".into())),
                                type_arguments: Vec::new(),
                                arguments: Vec::new(),
                            })],
                        }),
                    })),
                ],
            ),
        ]
    );
}

#[test]
fn rejects_non_member_safe_navigation_forms_with_the_stable_diagnostic() {
    // Given
    let sources = [
        ("a?.()", "PARSE_UNSUPPORTED_SAFE_CALL"),
        ("a?[0]", "PARSE_UNSUPPORTED_SAFE_INDEX"),
        ("a.!", "PARSE_UNEXPECTED_TOKEN"),
    ];

    // When / Then
    for (source, diagnostic) in sources {
        let result = parse(source);
        assert!(!result.program_accepted, "{source}");
        assert_eq!(result.diagnostics[0].code, diagnostic);
    }
}

#[test]
fn keeps_contract_view_distinct_from_range_operators() {
    // Given
    let source = "value..member; start ..= finish";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{:#?}", result.diagnostics);
    assert_eq!(
        result.program.statements,
        [
            Statement::Expression(Expression::ContractView {
                receiver: Box::new(Expression::Name("value".into())),
                selector: "member".into(),
            }),
            Statement::Expression(Expression::Binary {
                left: Box::new(Expression::Name("start".into())),
                operator: BinaryOperator::RangeInclusive,
                right: Box::new(Expression::Name("finish".into())),
            }),
        ]
    );
}

#[test]
fn retains_each_named_infix_selector_in_the_ast() {
    // Given
    let sources = [("a div b", "div"), ("a mod b", "mod"), ("a foo b", "foo")];

    // When / Then
    for (source, selector) in sources {
        let result = parse(source);

        assert!(result.is_clean(), "{result:#?}");
        assert_eq!(
            result.program.statements,
            [Statement::Expression(Expression::Binary {
                left: Box::new(Expression::Name("a".into())),
                operator: BinaryOperator::NamedInfix {
                    selector: selector.into(),
                },
                right: Box::new(Expression::Name("b".into())),
            })]
        );
    }

    // Given
    let div = parse("a div b").program;
    let modulo = parse("a mod b").program;

    // Then
    assert_ne!(div, modulo);
}

#[test]
fn parses_suffixed_named_infix_selectors() {
    // Given
    let sources = [
        ("a same? b", BinaryOperator::Identity),
        (
            "a ready! b",
            BinaryOperator::NamedInfix {
                selector: "ready!".into(),
            },
        ),
    ];

    // When / Then
    for (source, operator) in sources {
        let result = parse(source);

        assert!(result.is_clean(), "{result:#?}");
        assert_eq!(
            result.program.statements,
            [Statement::Expression(Expression::Binary {
                left: Box::new(Expression::Name("a".into())),
                operator,
                right: Box::new(Expression::Name("b".into())),
            })]
        );
    }
}

#[test]
fn accepts_if_in_expression_positions() {
    // Given
    let sources = [
        "let result = if true { :yes } else { :no }; result",
        "%[if true { :yes } else { :no }]",
        "let result = if false { :yes }; result",
        "let result = if false { :first } else if true { :second } else { :third }; result",
    ];

    // When
    let results = sources.map(parse);

    // Then
    assert!(results.into_iter().all(|result| result.program_accepted));
}

#[test]
fn rejects_dot_sigil_storage_accesses() {
    // Given
    let sources = ["obj.@@x", "A.@@x", "other.@x"];

    // When / Then
    for source in sources {
        assert!(!parse(source).program_accepted, "{source}");
    }
}

fn negate(value: &str) -> Expression {
    Expression::Unary {
        operator: UnaryOperator::Negate,
        operand: Box::new(Expression::Literal(value.into())),
    }
}

fn named_infix(left: Expression, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        operator: BinaryOperator::NamedInfix {
            selector: "div".into(),
        },
        right: Box::new(right),
    }
}

fn safe_chain(receiver: &str, parts: Vec<PostfixPart>) -> Statement {
    Statement::Expression(Expression::SafeNavigation {
        receiver: Box::new(Expression::Name(receiver.into())),
        parts,
    })
}

fn member(selector: &str) -> PostfixPart {
    PostfixPart::Member {
        selector: selector.into(),
        safe: false,
    }
}

fn safe_member(selector: &str) -> PostfixPart {
    PostfixPart::Member {
        selector: selector.into(),
        safe: true,
    }
}

fn call(arguments: Vec<Expression>) -> PostfixPart {
    PostfixPart::Call {
        type_arguments: Vec::new(),
        arguments,
    }
}
