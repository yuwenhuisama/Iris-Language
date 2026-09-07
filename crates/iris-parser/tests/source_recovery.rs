use iris_parser::source::{DeclarationKind, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn rollback_when_a_type_reading_splits_shift_then_fails() -> Result<(), &'static str> {
    let text = "let value = (A<B>>C)";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(
        result.parse.program_accepted,
        "{:?}",
        result.parse.diagnostics
    );
    assert!(
        !result
            .source
            .nodes
            .iter()
            .any(|node| matches!(node.kind, SourceKind::Type(_)))
    );
    let iris_syntax::Statement::Binding { value, .. } = &result.parse.program.statements[0] else {
        return Err("binding");
    };
    let iris_syntax::Expression::Grouped(value) = value else {
        return Err("group");
    };
    let iris_syntax::Expression::Binary { right, .. } = value.as_ref() else {
        return Err("comparison");
    };
    assert!(matches!(
        right.as_ref(),
        iris_syntax::Expression::Binary {
            operator: iris_syntax::BinaryOperator::ShiftRight,
            ..
        }
    ));
    Ok(())
}

#[test]
fn declarations_when_headers_have_constraints_and_capabilities() {
    let text = "class Box<T> where T: Object meta deny method_set { public fun get() {} }";
    let result = parse_with_source(text);
    assert!(result.parse.program_accepted);
    assert!(result.source.nodes.iter().any(|node| matches!(&node.kind, SourceKind::Declaration(value) if value.kind == DeclarationKind::Class && value.name.text == "Box")));
}

#[test]
fn no_phantoms_when_a_declaration_initializer_fails() {
    let text = "module Main { let valid = 1; let broken = ; valid. }";
    let result = parse_editor(text);
    assert!(!result.parse.program_accepted);
    assert!(!result.source.nodes.iter().any(
        |node| matches!(&node.kind, SourceKind::Declaration(value) if value.name.text == "broken")
    ));
    for node in &result.source.nodes {
        for child in &node.children {
            assert!(child.0 < result.source.nodes.len());
        }
        assert!(node.scope.0 < result.source.scopes.len());
    }
    for scope in &result.source.scopes {
        assert!(scope.span.end <= text.len());
        assert!(
            scope
                .owner
                .is_none_or(|id| id.0 < result.source.nodes.len())
        );
    }
}

#[test]
fn resource_limits_when_editor_input_is_hostile() {
    for text in [
        "(".repeat(300),
        format!("let x = {}", "a.".repeat(2000)),
        "class A {".repeat(300),
    ] {
        let strict = parse_with_source(&text);
        assert_eq!(strict.parse, parse(&text));
        let editor = parse_editor(&text);
        assert!(!editor.parse.program_accepted);
        assert!(editor.source.nodes.len() <= text.len() * 8 + 1);
        for node in &editor.source.nodes {
            assert!(
                node.children
                    .iter()
                    .all(|id| id.0 < editor.source.nodes.len())
            );
        }
    }
}

#[test]
fn valid_links_when_each_header_prefix_is_incomplete() {
    for fixture in [
        "class Box<T> extends Base<T> { public fun get?(item: T = 1) -> T { item.call(1) } }",
        "from org.dep::Core import Thing as Local, Other",
        "let block = { |arg: String| -> String; arg }; (Box<Array<String>>).type",
    ] {
        for end in 0..=fixture.len() {
            let text = &fixture[..end];
            let result = parse_editor(text);
            for node in &result.source.nodes {
                assert!(
                    node.children
                        .iter()
                        .all(|id| id.0 < result.source.nodes.len()),
                    "{text}"
                );
                assert!(node.scope.0 < result.source.scopes.len(), "{text}");
                assert!(
                    node.span.start <= node.span.end && node.span.end <= text.len(),
                    "{text}: {node:?}"
                );
            }
            for scope in &result.source.scopes {
                assert!(
                    scope
                        .owner
                        .is_none_or(|id| id.0 < result.source.nodes.len()),
                    "{text}"
                );
                assert!(scope.span.end <= text.len(), "{text}: {scope:?}");
            }
            assert!(
                result
                    .source
                    .roots
                    .iter()
                    .all(|id| id.0 < result.source.nodes.len()),
                "{text}"
            );
        }
    }
}

#[test]
fn parameter_and_body_binding_share_method_scope() {
    let text = "fun method(value: Integer) { let value = 1 }";
    let result = parse_with_source(text);
    let scopes: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) if value.name.text == "value" => Some(node.scope),
            _ => None,
        })
        .collect();
    assert_eq!(scopes.len(), 2);
    assert_eq!(scopes[0], scopes[1]);
}

#[test]
fn expression_range_when_optional_else_lookahead_crosses_newline() -> Result<(), &'static str> {
    let text = "let value = if true { 1 }\nlet next = 2";
    let result = parse_with_source(text);
    let declaration = result
        .source
        .nodes
        .iter()
        .find_map(|node| match &node.kind {
            SourceKind::Declaration(value) if value.name.text == "value" => Some((node, value)),
            _ => None,
        })
        .ok_or("binding")?;
    assert_eq!(declaration.0.span.end, 25);
    assert_eq!(declaration.1.visible_from, 25);
    Ok(())
}

#[test]
fn stored_property_surface_when_ast_discards_visibility() {
    let text = "module Main { protected module property item: Integer = 1 }";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.source.nodes.iter().any(|node| matches!(&node.kind,
        SourceKind::Declaration(value) if value.name.text == "item"
            && value.visibility == iris_syntax::Visibility::Protected
            && value.surface == Some(iris_syntax::MethodKind::Module))));
}

#[test]
fn pattern_bindings_when_loop_and_catch_scopes_end() {
    let text = "let item = 1; for item in item { item }; try { item } catch failure: Error { failure }; item";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    for node in &result.source.nodes {
        if let SourceKind::Declaration(value) = &node.kind
            && value.kind == DeclarationKind::PatternBinding
        {
            let scope = result.source.scope(node.scope);
            assert!(matches!(
                scope.kind,
                iris_parser::source::ScopeKind::Loop | iris_parser::source::ScopeKind::Catch
            ));
            assert!(scope.span.end < text.len());
            if value.name.text == "item" {
                assert_eq!(value.visible_from, 30);
            }
        }
    }
}
