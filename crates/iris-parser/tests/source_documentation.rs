use iris_parser::source::SourceKind;
use iris_parser::{parse, parse_with_source};

#[test]
fn attachment_when_decorator_export_or_member_owns_declaration() -> Result<(), &'static str> {
    for (text, expected) in [
        ("\u{feff}  /// doc\r\n  @Stamp()\r\nclass Same {}", "Same"),
        ("/// doc\nexport class Same {}", "Same"),
        (
            "class Same {\n  /// doc\n  @Stamp()\n  public fun member() {}\n}",
            "member",
        ),
        ("/** doc */\nmodule Same {}", "Same"),
    ] {
        let result = parse_with_source(text);
        assert_eq!(result.parse, parse(text));
        assert!(
            result.parse.program_accepted,
            "{text}: {:?}",
            result.parse.diagnostics
        );
        assert_eq!(result.source.documentation.len(), 1, "{text}");
        let SourceKind::Declaration(declaration) = &result
            .source
            .node(result.source.documentation[0].declaration)
            .kind
        else {
            return Err("declaration");
        };
        assert_eq!(declaration.name.text, expected);
    }
    Ok(())
}

#[test]
fn no_attachment_when_trivia_or_damage_breaks_adjacency() {
    for text in [
        "/// doc\n\nfun target() {}",
        "/// doc\n// break\nfun target() {}",
        "/// doc\n/* break */\nfun target() {}",
        "let previous=1; /// trailing\nfun target() {}",
        "/// doc\nlet previous=1\nfun target() {}",
        "/// doc\n@Stamp()\n\nclass Target {}",
        "/// doc\n@Stamp()\n// break\nclass Target {}",
        "class Container {\n/// doc\n@Stamp()\n// break\nfun target() {} }",
        "/// doc\nclass Broken extends {}",
    ] {
        let result = parse_with_source(text);
        assert_eq!(result.parse, parse(text));
        assert!(
            result.source.documentation.iter().all(|doc| {
                match &result.source.node(doc.declaration).kind {
                    SourceKind::Declaration(declaration) => declaration.name.text == "previous",
                    _ => false,
                }
            }),
            "{text}: {:?}",
            result.source.documentation
        );
    }
}

#[test]
fn block_text_when_stars_and_paragraphs_are_present() {
    let result =
        parse_with_source("/**\n * Summary\n *\n * Paragraph /* nested */\n */\nfun target() {}");
    assert_eq!(
        result.source.documentation[0].text,
        "Summary\n\nParagraph /* nested */"
    );
}

#[test]
fn paragraphs_when_documentation_line_is_empty() {
    let result = parse_with_source("///\n/// Summary\n///\n/// Paragraph\nfun target() {}");
    assert_eq!(
        result.source.documentation[0].text,
        "\nSummary\n\nParagraph"
    );
}

#[test]
fn text_bound_when_documentation_exceeds_two_kibibytes() {
    let text = format!("/// {}\nfun target() {{}}", "界".repeat(1000));
    let result = parse_with_source(&text);
    assert!(result.source.documentation[0].text.len() <= 2048);
    assert!(result.source.documentation[0].truncated);
}
