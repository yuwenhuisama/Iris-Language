use iris_parser::source::{ExpressionFact, RangeOperator, SourceDocument, SourceKind, SyntaxId};
use iris_parser::{parse, parse_editor, parse_with_source};

fn expression_at<'document>(
    document: &'document SourceDocument,
    text: &str,
    expression: &str,
) -> Result<&'document iris_parser::source::SyntaxNode, &'static str> {
    document
        .nodes
        .iter()
        .find(|node| {
            matches!(node.kind, SourceKind::Expression(_))
                && &text[node.span.start..node.span.end] == expression
        })
        .ok_or("expression source node")
}

fn child_texts<'text>(
    document: &SourceDocument,
    text: &'text str,
    children: &[SyntaxId],
) -> Vec<&'text str> {
    children
        .iter()
        .map(|id| {
            let node = document.node(*id);
            assert!(matches!(node.kind, SourceKind::Expression(_)));
            &text[node.span.start..node.span.end]
        })
        .collect()
}

#[test]
fn collection_facts_when_literals_have_nested_children() -> Result<(), &'static str> {
    let text = "[1, (2,), %{3: [4], 5: 6}]";
    let result = parse_with_source(text);
    assert!(
        result.parse.program_accepted,
        "{:?}",
        result.parse.diagnostics
    );
    let array = expression_at(&result.source, text, text)?;
    let SourceKind::Expression(ExpressionFact::Array { elements }) = &array.kind else {
        return Err("typed array fact");
    };
    assert_eq!(elements, &array.children);
    assert_eq!(
        child_texts(&result.source, text, elements),
        ["1", "(2,)", "%{3: [4], 5: 6}"]
    );
    let tuple = result.source.node(elements[1]);
    let SourceKind::Expression(ExpressionFact::Tuple { elements }) = &tuple.kind else {
        return Err("typed tuple fact");
    };
    assert_eq!(elements, &tuple.children);
    assert_eq!(child_texts(&result.source, text, elements), ["2"]);
    let hash = expression_at(&result.source, text, "%{3: [4], 5: 6}")?;
    let SourceKind::Expression(ExpressionFact::Hash { entries }) = &hash.kind else {
        return Err("typed hash fact");
    };
    let children: Vec<_> = entries
        .iter()
        .flat_map(|(key, value)| [*key, *value])
        .collect();
    assert_eq!(children, hash.children);
    assert_eq!(
        child_texts(&result.source, text, &children),
        ["3", "[4]", "5", "6"]
    );
    Ok(())
}

#[test]
fn collection_facts_when_literals_are_empty() -> Result<(), &'static str> {
    let text = "[]; (); %{}";
    let result = parse_with_source(text);
    assert!(result.parse.program_accepted);
    assert!(matches!(&expression_at(&result.source, text, "[]")?.kind,
        SourceKind::Expression(ExpressionFact::Array { elements }) if elements.is_empty()));
    assert!(matches!(&expression_at(&result.source, text, "()")?.kind,
        SourceKind::Expression(ExpressionFact::Tuple { elements }) if elements.is_empty()));
    assert!(matches!(&expression_at(&result.source, text, "%{}")?.kind,
        SourceKind::Expression(ExpressionFact::Hash { entries }) if entries.is_empty()));
    Ok(())
}

#[test]
fn range_facts_when_operators_have_distinct_bounds() -> Result<(), &'static str> {
    for (spelling, expected) in [
        ("..=", RangeOperator::Inclusive),
        ("..<", RangeOperator::Exclusive),
    ] {
        let text = format!("(start + 1 /* left */ {spelling} /* right */ finish * 2).size()");
        let result = parse_with_source(&text);
        assert!(
            result.parse.program_accepted,
            "{:?}",
            result.parse.diagnostics
        );
        let range = result
            .source
            .nodes
            .iter()
            .find(|node| {
                matches!(
                    node.kind,
                    SourceKind::Expression(ExpressionFact::Range { .. })
                )
            })
            .ok_or("range fact")?;
        let SourceKind::Expression(ExpressionFact::Range {
            start,
            end,
            operator,
            operator_span,
        }) = &range.kind
        else {
            return Err("typed range fact");
        };
        assert_eq!(*operator, expected);
        assert_eq!(&text[operator_span.start..operator_span.end], spelling);
        assert_eq!(range.children, [*start, *end]);
        assert_eq!(
            child_texts(&result.source, &text, &range.children),
            ["start + 1", "finish * 2"]
        );
    }
    Ok(())
}

#[test]
fn receiver_facts_when_collection_and_range_are_grouped() -> Result<(), &'static str> {
    for literal in ["[1]", "(1, 2)", "%{1: 2}", "1 ..< 3"] {
        let text = format!("(({literal})).size()");
        let result = parse_with_source(&text);
        assert!(result.parse.program_accepted);
        let member = expression_at(&result.source, &text, &format!("(({literal})).size"))?;
        let SourceKind::Expression(ExpressionFact::Member { receiver, .. }) = member.kind else {
            return Err("member fact");
        };
        let mut value = receiver;
        for _ in 0..2 {
            let SourceKind::Expression(ExpressionFact::Grouped { value: child }) =
                result.source.node(value).kind
            else {
                return Err("grouped receiver");
            };
            value = child;
        }
        assert_eq!(value, expression_at(&result.source, &text, literal)?.id);
        assert!(matches!(
            result.source.node(value).kind,
            SourceKind::Expression(
                ExpressionFact::Array { .. }
                    | ExpressionFact::Tuple { .. }
                    | ExpressionFact::Hash { .. }
                    | ExpressionFact::Range { .. }
            )
        ));
    }
    Ok(())
}

#[test]
fn receiver_facts_when_editor_member_is_incomplete() -> Result<(), &'static str> {
    for literal in ["[1]", "(1,)", "%{1: 2}", "(1 ..= 3)"] {
        let text = format!("{literal}.");
        let result = parse_editor(&text);
        assert!(!result.parse.program_accepted);
        assert!(!result.source.recovery.is_empty());
        let member = expression_at(&result.source, &text, &text)?;
        let SourceKind::Expression(ExpressionFact::IncompleteMember { receiver, .. }) = member.kind
        else {
            return Err("incomplete member fact");
        };
        assert_eq!(receiver, expression_at(&result.source, &text, literal)?.id);
        assert!(result.source.scope(member.scope).damaged);
    }
    Ok(())
}

#[test]
fn receiver_facts_when_failed_parent_rolls_back_children() {
    for broken in ["[1,", "(1,", "%{1:", "1 ..=", "1 ..<", "[1, (2 ..= 3),"] {
        let text = format!("let kept = []; let broken = {broken}");
        let result = parse_editor(&text);
        assert!(!result.parse.program_accepted);
        let expressions: Vec<_> = result
            .source
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, SourceKind::Expression(_)))
            .collect();
        assert_eq!(expressions.len(), 1, "{text}: {expressions:?}");
        assert!(matches!(&expressions[0].kind,
            SourceKind::Expression(ExpressionFact::Array { elements }) if elements.is_empty()));
        for node in &result.source.nodes {
            assert!(
                node.children
                    .iter()
                    .all(|id| id.0 < result.source.nodes.len())
            );
        }
    }
}

#[test]
fn strict_parse_when_receiver_facts_are_recorded_is_unchanged() {
    for text in [
        "[]; (); %{}",
        "[1, (2,), %{3: [4]}]",
        "(1 ..= 3).size()",
        "(1 ..< 3).size()",
        "[1,",
        "1 ..=",
        "1 ..< 2 ..< 3",
        "[1].",
    ] {
        let strict = parse(text);
        let result = parse_with_source(text);
        assert_eq!(result.parse, strict, "{text}");
    }
}

#[test]
fn binary_facts_when_operator_is_not_a_range_remain_unsupported() -> Result<(), &'static str> {
    for text in ["1 + 2", "1 < 2", "1 same? 2", "1 is Integer"] {
        let result = parse_with_source(text);
        assert!(result.parse.program_accepted);
        assert!(matches!(
            &expression_at(&result.source, text, text)?.kind,
            SourceKind::Expression(ExpressionFact::Unsupported { form: "binary" })
        ));
    }
    Ok(())
}
