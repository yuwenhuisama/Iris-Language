use iris_parser::source::{ArgumentKind, ExpressionFact, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn separators_when_nested_expressions_contain_commas() -> Result<(), &'static str> {
    let text = "target<A,B>(nested(1,2), [3,4], %{5:6,7:8}, 'comma,', label: 9) { |arg|; arg }";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(
        result.parse.program_accepted,
        "{:?}",
        result.parse.diagnostics
    );
    let call = result.source.calls.last().ok_or("call")?;
    assert_eq!(&text[call.open.start..call.open.end], "(");
    assert_eq!(call.commas.len(), 4);
    assert_eq!(call.arguments.len(), 6);
    assert!(matches!(call.arguments[4].kind, ArgumentKind::Keyword(_)));
    assert!(matches!(
        call.arguments[5].kind,
        ArgumentKind::TrailingBlock
    ));
    assert!(!call.incomplete);
    Ok(())
}

#[test]
fn ancestor_calls_when_editor_stops_at_eof_or_existing_closer() -> Result<(), &'static str> {
    for (text, count) in [
        ("f(", 1),
        ("f(a,", 1),
        ("f(label:", 1),
        ("f(label: value +", 1),
        ("f(g(", 2),
        ("f(g(1,", 2),
        ("f(g(1 + ))", 2),
        ("fun body() { f(g(1 + }", 2),
        ("fun body() { f(a, }", 1),
        ("f(g(1), h(2 + ))", 3),
    ] {
        let result = parse_editor(text);
        assert!(!result.parse.program_accepted, "{text}");
        assert_eq!(
            result.source.calls.len(),
            count,
            "{text}: {:?}",
            result.source
        );
        for site in &result.source.calls {
            if text != "f(g(1), h(2 + ))" {
                assert!(site.incomplete, "{text}");
            }
            let SourceKind::Expression(ExpressionFact::Call { callee, .. }) =
                result.source.node(site.call).kind
            else {
                return Err("call");
            };
            assert!(callee.0 < result.source.nodes.len());
            assert!(site.end <= text.len());
        }
        assert_eq!(parse_with_source(text).parse, parse(text));
    }
    Ok(())
}

#[test]
fn strict_parity_when_each_call_prefix_is_incomplete() {
    for fixture in [
        "target<A,B>(nested(1,2), [3,4], label: 9) { |arg|; arg }",
        "f(label: value + )",
        "f(key: value)",
    ] {
        for end in 0..=fixture.len() {
            let text = &fixture[..end];
            assert_eq!(parse_with_source(text).parse, parse(text), "{text}");
        }
    }
}

#[test]
fn progress_when_nested_editor_calls_exceed_resource_budget() {
    let text = "target(".repeat(1000);
    let result = parse_editor(&text);
    assert!(!result.parse.program_accepted);
    assert!(
        result
            .parse
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PARSE_RESOURCE_LIMIT")
    );
    assert!(
        result
            .source
            .calls
            .iter()
            .all(|call| call.call.0 < result.source.nodes.len())
    );
}

#[test]
fn incomplete_when_trailing_block_is_recovered() {
    let result = parse_editor("f() { |arg|; arg");
    assert!(!result.parse.program_accepted);
    let call = &result.source.calls[0];
    assert!(call.incomplete);
    assert!(call.arguments[0].incomplete);
    assert!(matches!(
        call.arguments[0].kind,
        ArgumentKind::TrailingBlock
    ));
}

#[test]
fn expression_links_when_only_a_partial_argument_was_parsed() -> Result<(), &'static str> {
    let result = parse_editor("f(value + )");
    let site = &result.source.calls[0];
    assert_eq!(site.arguments[0].expression, None);
    let SourceKind::Expression(ExpressionFact::Call { arguments, .. }) =
        &result.source.node(site.call).kind
    else {
        return Err("call");
    };
    assert!(arguments.is_empty());
    Ok(())
}

#[test]
fn valid_metadata_when_speculation_and_failed_productions_roll_back() {
    for fixture in [
        "fun f(_: Integer = seed(), value = (A<B>>C)) { f(value) }",
        "let block={ |_: Integer, arg|; let local: String; (Object).type }",
        "let broken = f(g()) + ; let valid = h()",
    ] {
        for end in 0..=fixture.len() {
            let result = parse_editor(&fixture[..end]);
            let nodes = result.source.nodes.len();
            for slot in &result.source.parameter_slots {
                assert!(slot.owner.0 < nodes);
                for id in [slot.declaration, slot.annotation, slot.default]
                    .into_iter()
                    .flatten()
                {
                    assert!(id.0 < nodes);
                }
            }
            for signature in &result.source.signatures {
                assert!(signature.owner.0 < nodes);
                assert!(signature.return_type.is_none_or(|id| id.0 < nodes));
            }
            for call in &result.source.calls {
                assert!(call.call.0 < nodes);
                for slot in &call.arguments {
                    assert!(slot.expression.is_none_or(|id| id.0 < nodes));
                }
            }
        }
    }
}

#[test]
fn closure_signature_when_body_has_types_and_nested_parameters() -> Result<(), &'static str> {
    let result = parse_with_source(
        "let block={ |_, arg|; (Object).type; let nested={ |other: String| -> Integer; 1 } }",
    );
    assert!(result.parse.program_accepted);
    let signature = &result.source.signatures[0];
    assert_eq!(signature.return_type, None);
    let SourceKind::Expression(ExpressionFact::Closure {
        parameters,
        return_type,
    }) = &result.source.node(signature.owner).kind
    else {
        return Err("closure");
    };
    assert_eq!(*return_type, None);
    assert_eq!(parameters.len(), 1);
    assert_eq!(
        result
            .source
            .parameter_slots
            .iter()
            .filter(|slot| slot.owner == signature.owner)
            .count(),
        2
    );
    Ok(())
}
