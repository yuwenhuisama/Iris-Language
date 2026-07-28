//! Source decoding, trivia scanning, and diagnostics for Iris v1.

mod diagnostic;
mod scanner;

pub use diagnostic::{ByteOffset, Diagnostic, SourcePosition};
pub use scanner::{LexedSource, Token, TokenKind, lex, lex_type};

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex, lex_type};

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
            2
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

    #[test]
    fn tokenizes_ranges_and_contract_views_as_distinct_contextual_tokens() {
        // Given
        let source = b"a ..= b; a ..< b; view..member";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::Identifier,
                TokenKind::RangeInclusive,
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Identifier,
                TokenKind::RangeExclusive,
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Identifier,
                TokenKind::ContractView,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn tokenizes_a_setter_selector_distinctly_from_inequality() {
        // Given
        let source = b"a != b; property fun ready?=(v: Bool) {}";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::Identifier,
                TokenKind::BangEqual,
                TokenKind::Identifier,
                TokenKind::Semicolon,
                TokenKind::Keyword,
                TokenKind::Keyword,
                TokenKind::SetterSelector,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Identifier,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
            ]
        );
    }

    #[test]
    fn splits_nested_generic_closers_only_in_type_context() {
        // Given
        let source = b"Box<Array<String>>";

        // When
        let result = lex_type(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::Identifier,
                TokenKind::LessThan,
                TokenKind::Identifier,
                TokenKind::LessThan,
                TokenKind::Identifier,
                TokenKind::GreaterThan,
                TokenKind::GreaterThan,
            ]
        );
    }

    #[test]
    fn tokenizes_hash_open_and_literal_prefix_families() {
        // Given
        let source = b"%{k: v}; m\"x\"; br\"x\"; mbr\"x\"";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::HashOpen,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Identifier,
                TokenKind::RightBrace,
                TokenKind::Semicolon,
                TokenKind::MutableStringLiteral,
                TokenKind::Semicolon,
                TokenKind::BytesLiteral,
                TokenKind::Semicolon,
                TokenKind::ByteArrayLiteral,
            ]
        );
    }

    #[test]
    fn recognizes_every_frozen_literal_prefix_family() {
        // Given
        let source = b"m\"x\" b\"x\" r\"x\" mr\"x\" br\"x\" mb\"x\" mbr\"x\"";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::MutableStringLiteral,
                TokenKind::BytesLiteral,
                TokenKind::StringLiteral,
                TokenKind::MutableStringLiteral,
                TokenKind::BytesLiteral,
                TokenKind::ByteArrayLiteral,
                TokenKind::ByteArrayLiteral,
            ]
        );
    }

    #[test]
    fn retains_right_shift_in_expression_context() {
        // Given
        let source = b"left >> right";

        // When
        let result = lex(source);

        // Then
        assert_eq!(
            kinds(&result),
            [
                TokenKind::Identifier,
                TokenKind::RightShift,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn rejects_invalid_literal_prefix_order() {
        // Given
        let source = b"rm\"x\"";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_BAD_LITERAL_PREFIX");
    }

    #[test]
    fn rejects_interpolation_outside_a_literal() {
        // Given
        let source = b"${x}";

        // When
        let result = lex(source);

        // Then
        assert_eq!(
            result.diagnostics()[0].code(),
            "LEX_INTERPOLATION_OUTSIDE_LITERAL"
        );
    }

    #[test]
    fn recognizes_interpolation_inside_an_interpolated_literal() {
        // Given
        let source = b"m\"${x}\"";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(kinds(&result), [TokenKind::MutableStringLiteral]);
    }

    #[test]
    fn tokenizes_slash_as_division_after_an_expression() {
        // Given
        let source = b"a / b";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(
            kinds(&result),
            [
                TokenKind::Identifier,
                TokenKind::Slash,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn tokenizes_slash_as_a_regex_at_expression_start() {
        // Given
        let source = b"/x/";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean());
        assert_eq!(kinds(&result), [TokenKind::RegexLiteral]);
    }

    #[test]
    fn rejects_unterminated_interpolation_in_an_interpolated_literal() {
        // Given
        let source = b"m\"${x\"";

        // When
        let result = lex(source);

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_UNTERMINATED_LITERAL");
    }

    fn kinds(result: &super::LexedSource) -> Vec<TokenKind> {
        result.tokens().iter().map(|token| token.kind).collect()
    }
}
