use iris_parser::source::{DeclarationKind, SourceKind};
use iris_parser::{parse, parse_with_source};

#[test]
fn nominal_value_metadata_when_headers_contain_types() -> Result<(), &'static str> {
    for (text, kind, types) in [
        (
            "class Box extends Base for Show mixin Helpers { let value: Integer = 1 }",
            DeclarationKind::Class,
            vec!["Base", "Show", "Helpers"],
        ),
        (
            "module Main mixin Helpers { let value: Integer = 1 }",
            DeclarationKind::Module,
            vec!["Helpers"],
        ),
        (
            "contract Show extends Parent, Other { fun get() -> Integer }",
            DeclarationKind::Contract,
            vec!["Parent", "Other"],
        ),
    ] {
        let result = parse_with_source(text);

        assert_eq!(result.parse, parse(text));
        assert!(
            result.parse.program_accepted,
            "{:?}",
            result.parse.diagnostics
        );
        let (node, declaration) = result
            .source
            .nodes
            .iter()
            .find_map(|node| match &node.kind {
                SourceKind::Declaration(value) if value.kind == kind => Some((node, value)),
                _ => None,
            })
            .ok_or("nominal declaration")?;
        assert_eq!(declaration.annotation, None, "{kind:?}");
        assert_eq!(declaration.initializer, None, "{kind:?}");
        assert_eq!(declaration.return_type, None, "{kind:?}");
        let header_types: Vec<_> = node
            .children
            .iter()
            .filter_map(|child| {
                let child = result.source.node(*child);
                match child.kind {
                    SourceKind::Type(_) => Some(&text[child.span.start..child.span.end]),
                    _ => None,
                }
            })
            .collect();
        assert_eq!(header_types, types);
        for child in &node.children {
            let child = result.source.node(*child);
            for name in &child.children {
                if let SourceKind::Name(site) = &result.source.node(*name).kind {
                    assert_eq!(&text[site.span.start..site.span.end], site.text);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn value_metadata_when_declarations_have_annotations_and_initializers() -> Result<(), &'static str>
{
    let text = "type Alias = Integer; let value: Integer = 1; fun get(arg: Integer = 2) -> Integer { arg }";

    let result = parse_with_source(text);

    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    for (name, annotation, initializer, return_type) in [
        ("Alias", Some("Integer"), None, None),
        ("value", Some("Integer"), Some("1"), None),
        ("get", None, None, Some("Integer")),
        ("arg", Some("Integer"), Some("2"), None),
    ] {
        let declaration = result
            .source
            .nodes
            .iter()
            .find_map(|node| match &node.kind {
                SourceKind::Declaration(value) if value.name.text == name => Some(value),
                _ => None,
            })
            .ok_or("declaration")?;
        let source_text = |id| {
            let span = result.source.node(id).span;
            &text[span.start..span.end]
        };
        assert_eq!(
            declaration.annotation.map(source_text),
            annotation,
            "{name}"
        );
        assert_eq!(
            declaration.initializer.map(source_text),
            initializer,
            "{name}"
        );
        assert_eq!(
            declaration.return_type.map(source_text),
            return_type,
            "{name}"
        );
    }
    Ok(())
}
