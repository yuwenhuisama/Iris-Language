use iris_parser::source::{DeclarationKind, ExpressionFact, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn binding_links_when_names_repeat() -> Result<(), &'static str> {
    let text = "let value = 1; let copy = value";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    let declarations: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) if value.kind == DeclarationKind::Binding => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        &text[declarations[0].name.span.start..declarations[0].name.span.end],
        "value"
    );
    assert_eq!(declarations[0].visible_from, 13);
    let initializer = declarations[1].initializer.ok_or("copy initializer")?;
    let initializer = result.source.node(initializer);
    assert!(
        matches!(&initializer.kind, SourceKind::Expression(ExpressionFact::Name { path }) if path[0].span.start == 26)
    );
    Ok(())
}

#[test]
fn signatures_when_annotations_are_discarded_by_ast() -> Result<(), &'static str> {
    let text = "class Box<T> { public fun get?(x: T = 1) -> T { mut later: T; let block = { |arg: T| -> T; arg }; x } }";
    let result = parse_with_source(text);
    assert!(
        result.parse.program_accepted,
        "{:?}",
        result.parse.diagnostics
    );
    assert_eq!(result.parse, parse(text));
    let declarations: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let method = declarations
        .iter()
        .find(|value| value.name.text == "get?")
        .ok_or("method declaration")?;
    assert_eq!(&text[method.name.span.start..method.name.span.end], "get?");
    assert_eq!(method.parameters.len(), 1);
    assert!(method.return_type.is_some());
    for name in ["later", "arg"] {
        let declaration = declarations
            .iter()
            .find(|value| value.name.text == name)
            .ok_or("annotated declaration")?;
        assert!(declaration.annotation.is_some(), "{name}");
    }
    Ok(())
}

#[test]
fn receiver_links_when_calls_chain() {
    let text = "let copy = Box<String>.new(1).get?(option: 2); copy = copy";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    let calls: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Expression(ExpressionFact::Call {
                callee, arguments, ..
            }) => Some((*callee, arguments)),
            _ => None,
        })
        .collect();
    assert_eq!(calls.len(), 2);
    for (callee, arguments) in calls {
        assert_eq!(arguments.len(), 1);
        assert!(matches!(
            result.source.node(callee).kind,
            SourceKind::Expression(ExpressionFact::Member { .. })
        ));
    }
}

#[test]
fn imports_when_segments_have_aliases() {
    let text = "from org.dep::Core import Thing as Local, Other\nimport Core as C";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    let imports: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Import(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(imports.len(), 2);
    assert_eq!(
        imports[0]
            .target
            .iter()
            .map(|site| site.text.as_str())
            .collect::<Vec<_>>(),
        ["org", "dep", "Core"]
    );
    assert_eq!(
        imports[0].specs[0]
            .alias
            .as_ref()
            .map(|site| site.text.as_str()),
        Some("Local")
    );
    assert_eq!(
        imports[1].alias.as_ref().map(|site| site.text.as_str()),
        Some("C")
    );
}

#[test]
fn scopes_when_shadowing_initializer_reads_outer() -> Result<(), &'static str> {
    let text = "module Main { let value = 1; if true { let value = value; value } }";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    let bindings: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) if value.kind == DeclarationKind::Binding => {
                Some((node, value))
            }
            _ => None,
        })
        .collect();
    assert_eq!(bindings.len(), 2);
    assert_ne!(bindings[0].0.scope, bindings[1].0.scope);
    let inner = bindings[1].1;
    let initializer = inner.initializer.ok_or("inner initializer")?;
    assert!(result.source.node(initializer).span.end <= inner.visible_from);
    assert_eq!(
        result.source.scope(bindings[1].0.scope).parent,
        Some(bindings[0].0.scope)
    );
    Ok(())
}

#[test]
fn editor_anchor_when_member_and_brace_are_incomplete() {
    let text = "module Main { let value = 1; value.";
    let result = parse_editor(text);
    assert!(!result.parse.program_accepted);
    assert!(result.source.nodes.iter().any(|node| matches!(
        node.kind,
        SourceKind::Expression(ExpressionFact::IncompleteMember { .. })
    )));
    assert!(result.source.nodes.iter().any(
        |node| matches!(&node.kind, SourceKind::Declaration(value) if value.name.text == "value")
    ));
    assert!(result.source.scopes.iter().any(|scope| scope.damaged));
}

#[test]
fn original_tokens_when_generic_and_closure_tokens_change() {
    let text = "let generic: Box<Array<String>> = Box<Array<String>>.new(); let closure = { ||; 1 }; (a | b)";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(
        result.parse.program_accepted,
        "{:?}",
        result.parse.diagnostics
    );
    assert!(
        result
            .source
            .tokens
            .iter()
            .any(|token| token.kind == iris_lexer::TokenKind::RightShift)
    );
    assert!(
        result
            .source
            .tokens
            .iter()
            .any(|token| token.kind == iris_lexer::TokenKind::PipePipe)
    );
    let sites: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Name(site) if site.text == "a" => Some(site),
            _ => None,
        })
        .collect();
    assert_eq!(sites.len(), 1);
}

#[test]
fn byte_ranges_when_names_are_unicode_and_types_optional() {
    let text = "let 名: String? = '值'; 名";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    for node in &result.source.nodes {
        if let SourceKind::Name(site) = &node.kind {
            assert_eq!(&text[site.span.start..site.span.end], site.text);
            assert_ne!(site.text, "Nil");
        }
    }
}
