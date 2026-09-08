use iris_lexer::{CommentKind, lex, lex_with_comments};

#[test]
fn comments_when_scanner_consumes_only_real_outer_comments() {
    let text = "\u{feff}/// doc\r\n/** block /* nested */ end */\n/* /// fake /** nested */ */\n'/// fake'; r/\\/\\*\\*/; // ordinary";
    let result = lex_with_comments(text.as_bytes());
    assert!(result.is_clean(), "{:?}", result.diagnostics());
    assert_eq!(
        result
            .comments()
            .iter()
            .map(|comment| comment.kind)
            .collect::<Vec<_>>(),
        [
            CommentKind::DocumentationLine,
            CommentKind::DocumentationBlock,
            CommentKind::Block,
            CommentKind::Line
        ]
    );
    assert_eq!(result.comments()[0].offset.0, 3);
    assert_eq!(
        &text[result.comments()[1].offset.0..result.comments()[1].end.0],
        "/** block /* nested */ end */"
    );
    let strict = lex(text.as_bytes());
    assert_eq!(result.tokens(), strict.tokens());
    assert!(strict.comments().is_empty());
}

#[test]
fn comments_when_nested_documentation_block_is_unterminated() {
    let result = lex_with_comments(b"/** outer /* inner */");
    assert_eq!(result.diagnostics()[0].code(), "LEX_UNTERMINATED_COMMENT");
    assert!(result.comments().is_empty());
}
