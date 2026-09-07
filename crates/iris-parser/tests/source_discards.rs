use iris_parser::source::{DeclarationKind, ExpressionFact, ScopeKind, SourceKind};
use iris_parser::{parse, parse_with_source};

#[test]
fn callable_scope_when_method_parameter_discards() -> Result<(), &'static str> {
    for text in [
        "module Main { fun use(_, value) { value } }",
        "module Main { fun use(_: Integer = 1, value: String = 'text') { value } }",
    ] {
        let result = parse_with_source(text);

        assert_eq!(result.parse, parse(text));
        assert!(
            result.parse.program_accepted,
            "{:?}",
            result.parse.diagnostics
        );
        assert!(result.source.recovery.is_empty());
        assert!(result.source.scopes.iter().all(|scope| !scope.damaged));
        let method = result
            .source
            .nodes
            .iter()
            .find_map(|node| match &node.kind {
                SourceKind::Declaration(value) if value.kind == DeclarationKind::Method => {
                    Some(value)
                }
                _ => None,
            })
            .ok_or("method")?;
        assert_eq!(method.parameters.len(), 1);
        let parameter = result.source.node(method.parameters[0]);
        assert_eq!(result.source.scope(parameter.scope).kind, ScopeKind::Method);
        let references: Vec<_> = result.source.nodes.iter().filter(|node| matches!(
            &node.kind, SourceKind::Expression(ExpressionFact::Name { path }) if path[0].text == "value"
        )).collect();
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].scope, parameter.scope);
        assert!(result.source.nodes.iter().all(|node| !matches!(
            &node.kind, SourceKind::Declaration(value) if value.name.text == "_"
        )));
    }
    Ok(())
}

#[test]
fn callable_scope_when_closure_parameter_discards_and_body_captures() -> Result<(), &'static str> {
    for text in [
        "module Main { let outer=1; let block={ |_, arg|; arg; outer } }",
        "module Main { let outer=1; let block={ |_: Integer, arg: String|; arg; outer } }",
    ] {
        let result = parse_with_source(text);

        assert_eq!(result.parse, parse(text));
        assert!(
            result.parse.program_accepted,
            "{:?}",
            result.parse.diagnostics
        );
        assert!(result.source.recovery.is_empty());
        assert!(result.source.scopes.iter().all(|scope| !scope.damaged));
        let (closure, parameters) = result
            .source
            .nodes
            .iter()
            .find_map(|node| match &node.kind {
                SourceKind::Expression(ExpressionFact::Closure { parameters, .. }) => {
                    Some((node, parameters))
                }
                _ => None,
            })
            .ok_or("closure")?;
        assert_eq!(parameters.len(), 1);
        let parameter = result.source.node(parameters[0]);
        let scope = result.source.scope(parameter.scope);
        assert_eq!(scope.kind, ScopeKind::Closure);
        assert_eq!(scope.owner, Some(closure.id));
        let outer = result
            .source
            .nodes
            .iter()
            .find(|node| {
                matches!(
                    &node.kind, SourceKind::Declaration(value) if value.name.text == "outer"
                )
            })
            .ok_or("outer binding")?;
        assert_eq!(scope.parent, Some(outer.scope));
        let references: Vec<_> = result.source.nodes.iter().filter(|node| matches!(
            &node.kind, SourceKind::Expression(ExpressionFact::Name { path }) if matches!(path[0].text.as_str(), "arg" | "outer")
        )).collect();
        assert_eq!(references.len(), 2);
        assert!(references.iter().all(|node| node.scope == parameter.scope));
        assert!(result.source.nodes.iter().all(|node| !matches!(
            &node.kind, SourceKind::Declaration(value) if value.name.text == "_"
        )));
    }
    Ok(())
}

#[test]
fn source_children_when_discard_has_annotation_or_default() -> Result<(), &'static str> {
    for (text, discard_text, children) in [
        (
            "fun use(_: Integer = seed(), value: String = 'text') { value }",
            "_: Integer = seed()",
            vec!["_", "Integer", "seed()"],
        ),
        (
            "let block = { |_: Integer, arg: String|; arg }",
            "_: Integer",
            vec!["_", "Integer"],
        ),
    ] {
        let result = parse_with_source(text);

        assert_eq!(result.parse, parse(text));
        assert!(result.parse.program_accepted);
        let discard = result
            .source
            .nodes
            .iter()
            .find(|node| {
                matches!(node.kind, SourceKind::Statement)
                    && &text[node.span.start..node.span.end] == discard_text
            })
            .ok_or("retained non-declaration discard")?;
        let child_texts: Vec<_> = discard
            .children
            .iter()
            .map(|child| {
                let child = result.source.node(*child);
                assert_eq!(child.scope, discard.scope);
                &text[child.span.start..child.span.end]
            })
            .collect();
        assert_eq!(child_texts, children);
        assert!(matches!(
            result.source.node(discard.children[1]).kind,
            SourceKind::Type(_)
        ));
        if let Some(default) = discard.children.get(2) {
            assert!(matches!(
                result.source.node(*default).kind,
                SourceKind::Expression(ExpressionFact::Call { .. })
            ));
        }
        for node in &result.source.nodes {
            if let SourceKind::Name(site) = &node.kind {
                assert_eq!(&text[site.span.start..site.span.end], site.text);
            }
            if let SourceKind::Declaration(value) = &node.kind {
                assert_eq!(
                    &text[value.name.span.start..value.name.span.end],
                    value.name.text
                );
            }
        }
    }
    Ok(())
}

#[test]
fn recovery_damage_when_discard_annotation_fails() {
    for text in ["fun use(_: ) { 1 }", "let block = { |_: , arg|; arg }"] {
        let result = parse_with_source(text);

        assert_eq!(result.parse, parse(text));
        assert!(!result.parse.program_accepted);
        assert!(!result.source.recovery.is_empty());
        assert!(result.source.scopes.iter().any(|scope| scope.damaged));
        assert!(result.source.nodes.iter().all(|node| !matches!(
            &node.kind, SourceKind::Name(site) if site.text == "_"
        )));
    }
}
