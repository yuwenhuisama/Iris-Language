use crate::parse;
use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

#[test]
fn parses_array_literals_with_named_infix_and_unary_elements() {
    // Given
    let source = "[-5 div 2, 5 div -2, -5 div -2]";

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
    let source = "[]; [1,]";

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
    let source = "[1,";

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
                arguments: Vec::new(),
            }),
            Statement::Expression(Expression::Member {
                receiver: Box::new(Expression::Call {
                    callee: Box::new(Expression::Member {
                        receiver: Box::new(Expression::Name("a".into())),
                        selector: "b".into(),
                    }),
                    arguments: vec![Expression::Name("c".into())],
                }),
                selector: "d".into(),
            }),
        ],
        "{result:#?}"
    );
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
fn rejects_if_in_expression_positions() {
    // Given
    let sources = [
        "let result = if true { :yes } else { :no }; result",
        "[if true { :yes } else { :no }]",
    ];

    // When
    let results = sources.map(parse);

    // Then
    assert!(
        results
            .into_iter()
            .all(|result| !result.program_accepted && !result.diagnostics.is_empty())
    );
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
