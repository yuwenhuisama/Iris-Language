use iris_parser::source::{ImportSeparatorKind, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn return_hint_when_defaults_contain_nested_parentheses() {
    let text = "fun first(value = call(1)) { value }; fun second() -> Integer { 1 }";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    let anchors: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) => value.return_hint_offset,
            _ => None,
        })
        .collect();
    assert_eq!(anchors, [26, 50]);
}

#[test]
fn import_provenance_when_dotted_and_qualified_names_match() {
    let text = "import org . dep :: Core; import org::dep::Core";
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
    assert_eq!(imports[0].separators[0].kind, ImportSeparatorKind::Dot);
    assert_eq!(
        imports[1].separators[0].kind,
        ImportSeparatorKind::Qualified
    );
    for import in imports {
        assert_eq!(import.separators.len(), import.target.len() - 1);
        for separator in &import.separators {
            let expected = match separator.kind {
                ImportSeparatorKind::Dot => ".",
                ImportSeparatorKind::Qualified => "::",
            };
            assert_eq!(&text[separator.span.start..separator.span.end], expected);
        }
    }
}

#[test]
fn composition_flags_when_nominal_headers_are_present() -> Result<(), &'static str> {
    let text = "class Box<T> extends Base<T> for Show mixin Helpers where T: Object meta deny method_set {} module Main mixin Helpers {} contract Show extends Parent {}";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    let headers: Vec<_> = result
        .source
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            SourceKind::Declaration(value) => value.header.as_ref(),
            _ => None,
        })
        .collect();
    assert_eq!(headers.len(), 3);
    let class = headers.first().ok_or("class header")?;
    assert!(class.complete && class.has_extends && class.has_implements && class.has_mixins);
    assert!(class.has_type_parameters && class.has_constraints && class.has_meta_policy);
    assert_eq!(&text[class.span.end..class.span.end + 1], "{");
    assert!(headers[1].has_mixins);
    assert!(headers[2].has_extends);
    Ok(())
}

#[test]
fn header_completeness_when_only_body_closer_is_missing() {
    let result = parse_editor("class Box { let value = 1");
    assert!(!result.parse.program_accepted);
    assert!(result.source.nodes.iter().any(|node| matches!(&node.kind,
        SourceKind::Declaration(value) if value.header.as_ref().is_some_and(|header| header.complete))));
}

#[test]
fn damaged_header_when_extends_type_is_missing() {
    let result = parse_editor("class Box extends {} ");
    assert!(!result.parse.program_accepted);
    assert!(result.source.nodes.iter().any(|node| matches!(&node.kind,
        SourceKind::Declaration(value) if value.header.as_ref().is_some_and(|header| !header.complete))));
}

#[test]
fn return_hint_when_method_is_bodyless_or_editor_body_is_unclosed() -> Result<(), &'static str> {
    for text in ["fun get()", "fun get() { let value = 1"] {
        let result = parse_editor(text);
        let method = result
            .source
            .nodes
            .iter()
            .find_map(|node| match &node.kind {
                SourceKind::Declaration(value) if value.name.text == "get" => Some(value),
                _ => None,
            })
            .ok_or("method declaration")?;
        assert_eq!(method.return_hint_offset, Some(9));
        assert_eq!(method.return_type, None);
    }
    Ok(())
}

#[test]
fn simple_header_when_declaration_has_no_composition() -> Result<(), &'static str> {
    let result = parse_with_source("class Box {}");
    let header = result
        .source
        .nodes
        .iter()
        .find_map(|node| match &node.kind {
            SourceKind::Declaration(value) => value.header.as_ref(),
            _ => None,
        })
        .ok_or("class header")?;
    assert!(header.complete);
    assert!(!header.has_extends && !header.has_implements && !header.has_mixins);
    assert!(!header.has_type_parameters && !header.has_constraints);
    assert!(!header.has_meta_policy && !header.has_decorators);
    Ok(())
}

#[test]
fn closure_token_when_reified_type_speculation_rolls_back() {
    let text = "(typeof({ ||; 1 }))";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    assert_eq!(
        result
            .source
            .nodes
            .iter()
            .filter(|node| matches!(
                node.kind,
                SourceKind::Expression(iris_parser::source::ExpressionFact::Closure { .. })
            ))
            .count(),
        1
    );
    assert!(
        !result
            .source
            .nodes
            .iter()
            .any(|node| matches!(node.kind, SourceKind::Type(_)))
    );
    let original = result
        .source
        .tokens
        .iter()
        .filter(|token| token.kind == iris_lexer::TokenKind::PipePipe)
        .count();
    assert_eq!(original, 1);
}
