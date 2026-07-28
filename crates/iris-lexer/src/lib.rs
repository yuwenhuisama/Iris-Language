//! Source decoding, trivia scanning, and diagnostics for Iris v1.

mod diagnostic;
mod scanner;

pub use diagnostic::{ByteOffset, Diagnostic, SourcePosition};
pub use scanner::{LexedSource, Token, TokenKind, lex};

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex};

    #[test]
    fn rejects_malformed_utf8_with_the_stable_code() {
        // Given
        let source = [0xC3, 0x28];

        // When
        let result = lex(&source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_INVALID_UTF8");
    }

    #[test]
    fn rejects_a_shebang_after_the_first_physical_line() {
        // Given
        let source = b"let x = 1\n#! late";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_SHEBANG_NOT_FIRST");
    }

    #[test]
    fn rejects_a_shebang_not_at_the_start_of_the_first_physical_line() {
        // Given
        let source = b" #! late";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_SHEBANG_NOT_FIRST");
    }

    #[test]
    fn rejects_an_unterminated_nested_block_comment() {
        // Given
        let source = b"/* outer /* inner */";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_UNTERMINATED_COMMENT");
    }

    #[test]
    fn rejects_a_backslash_not_immediately_followed_by_a_newline() {
        // Given
        let source = b"let bad = 1 + \\ \n2";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_BAD_CONTINUATION");
    }

    #[test]
    fn accepts_a_bom_first_line_shebang_nested_comments_and_continuation() {
        // Given
        let source = b"\xEF\xBB\xBF#! /usr/bin/env iris\r\n/* outer /* inner */ outer */\r\nlet sum = 1 + \\\r\n2";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            result
                .tokens()
                .iter()
                .filter(|token| token.kind == TokenKind::Newline)
                .count(),
            3
        );
    }

    #[test]
    fn rejects_utf16le_bom_without_replacement() {
        // Given
        let source = [0xFF, 0xFE, b'l', 0x00];

        // When
        let result = lex(&source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_INVALID_UTF8");
    }
}
