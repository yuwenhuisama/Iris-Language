//! Source decoding, trivia scanning, and diagnostics for Iris v1.

mod diagnostic;
mod literal;
mod scanner;

pub use diagnostic::{ByteOffset, Diagnostic, SourcePosition};
pub use literal::{Literal, LiteralConversion, convert_literals};
pub use scanner::{LexedSource, Token, TokenKind, lex, lex_type};

#[cfg(test)]
mod tests {
    use super::{Literal, TokenKind, convert_literals, lex, lex_type};

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
    fn tokenizes_longest_match_relational_shift_and_assignment_operators() {
        // Given
        let cases = [
            (b"8 << -2".as_slice(), TokenKind::LeftShift),
            (b"a <=> b".as_slice(), TokenKind::Spaceship),
            (b"a <= b".as_slice(), TokenKind::LessEqual),
            (b"a >= b".as_slice(), TokenKind::GreaterEqual),
            (b"a < b".as_slice(), TokenKind::LessThan),
            (b"a > b".as_slice(), TokenKind::GreaterThan),
            (b"a <<= b".as_slice(), TokenKind::LeftShiftEqual),
            (b"a >>= b".as_slice(), TokenKind::RightShiftEqual),
        ];

        // When / Then
        for (source, operator) in cases {
            let result = lex(source);
            assert!(result.is_clean(), "{result:#?}");
            assert_eq!(
                kinds(&result)
                    .iter()
                    .filter(|kind| **kind == operator)
                    .count(),
                1
            );
        }
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
    fn tokenizes_slash_as_division_after_an_integer_literal() {
        // Given
        let source = b"1 / 2";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean(), "{result:#?}");
        assert_eq!(
            kinds(&result),
            [
                TokenKind::SourceCharacter,
                TokenKind::Slash,
                TokenKind::SourceCharacter,
            ]
        );
    }

    #[test]
    fn lexes_integer_division_by_zero_cleanly() {
        // Given
        let source = b"1 / 0";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean(), "{result:#?}");
        assert!(kinds(&result).contains(&TokenKind::Slash));
    }

    #[test]
    fn lexes_integer_division_without_whitespace_cleanly() {
        // Given
        let source = b"1/0";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean(), "{result:#?}");
        assert!(kinds(&result).contains(&TokenKind::Slash));
    }

    #[test]
    fn lexes_float_division_cleanly() {
        // Given
        let source = b"1.5 / 2.0";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean(), "{result:#?}");
        assert!(kinds(&result).contains(&TokenKind::Slash));
    }

    #[test]
    fn lexes_hexadecimal_integer_division_cleanly() {
        // Given
        let source = b"0xFF / 2";

        // When
        let result = lex(source);

        // Then
        assert!(result.is_clean(), "{result:#?}");
        assert!(kinds(&result).contains(&TokenKind::Slash));
    }

    #[test]
    fn tokenizes_slash_as_a_regex_at_expression_start() {
        // Given
        let source = b"/abc/";

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

    #[test]
    fn converts_radix_integers_and_preserves_leading_zero_decimal() {
        // Given
        let source = "[0b1010, 0o755, 00755, 0xFF]";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [
                Literal::Integer("10".into()),
                Literal::Integer("493".into()),
                Literal::Integer("755".into()),
                Literal::Integer("255".into()),
            ]
        );
    }

    #[test]
    fn converts_hexadecimal_floats_without_reclassifying_hex_integer() {
        // Given
        let source = "[0x1.fp3, 0x1p0, 0x1.p0, 0x.8p0, 0x1e3]";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [
                Literal::Float64(15.5),
                Literal::Float64(1.0),
                Literal::Float64(1.0),
                Literal::Float64(0.5),
                Literal::Integer("483".into()),
            ]
        );
    }

    #[test]
    fn concatenates_escaped_raw_and_triple_string_segments() {
        // Given
        let source = "\"a\" 'b' r\"c\" \"\"\"d\"\"\"";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(result.values(), [Literal::String("abcd".into())]);
    }

    #[test]
    fn converts_separated_numeric_literals_exactly() {
        // Given
        let source = "1_000 0xFF_FF 1.234_567";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [
                Literal::Integer("1000".into()),
                Literal::Integer("65535".into()),
                Literal::Float64(1.234_567),
            ]
        );
    }

    #[test]
    fn rejects_bad_numeric_separators_and_empty_radix_prefixes_without_values() {
        // Given
        let source = "1__0 0x_FF";

        // When
        let result = convert_literals(source);

        // Then
        assert!(result.values().is_empty());
        assert_eq!(
            result.diagnostics(),
            ["LEX_BAD_NUMERIC_SEPARATOR", "LEX_EMPTY_RADIX_PREFIX"]
        );
    }

    #[test]
    fn ignores_digits_that_continue_identifiers_during_literal_conversion() {
        // Given
        let source = "Float64.from_bits(0x0000000000000000) Float32.nan Float64.infinity x2.y abc64.from_bits(0)";

        // When
        let result = lex(source.as_bytes());

        // Then
        assert!(result.is_clean(), "{result:#?}");
    }

    #[test]
    fn converts_standalone_floats_after_skipping_identifier_digits() {
        // Given
        let source = "1.5 1e3 0x1.fp3 64.5";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [
                Literal::Float64(1.5),
                Literal::Float64(1_000.0),
                Literal::Float64(15.5),
                Literal::Float64(64.5),
            ]
        );
        assert!(result.diagnostics().is_empty(), "{result:#?}");
    }

    #[test]
    fn retains_malformed_numeric_separator_diagnostics_after_skipping_identifiers() {
        // Given
        let source = "1__0";

        // When
        let result = lex(source.as_bytes());

        // Then
        assert_eq!(result.diagnostics()[0].code(), "LEX_BAD_NUMERIC_SEPARATOR");
    }

    #[test]
    fn preserves_only_valid_dot_floats_when_invalid_separator_candidates_follow() {
        // Given
        let source = ".5; 1.; .; ._5; 1._0; 1_.";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [Literal::Float64(0.5), Literal::Float64(1.0)]
        );
        assert!(result.diagnostics().contains(&"LEX_BAD_NUMERIC_SEPARATOR"));
        assert!(kinds(&lex(source.as_bytes())).contains(&TokenKind::Dot));
    }

    #[test]
    fn preserves_integers_larger_than_u64() {
        // Given
        let source = "18446744073709551616";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(
            result.values(),
            [Literal::Integer("18446744073709551616".into())]
        );
    }

    #[test]
    fn reports_each_required_literal_diagnostic_through_the_lexer() {
        // Given
        let cases = [
            (b"1__0".as_slice(), "LEX_BAD_NUMERIC_SEPARATOR"),
            (b"0x_FF".as_slice(), "LEX_EMPTY_RADIX_PREFIX"),
            (b"0b102".as_slice(), "LEX_INVALID_RADIX_DIGIT"),
            (b"1.0F32".as_slice(), "LEX_BAD_FLOAT_SUFFIX"),
            (b"\"\\q\"".as_slice(), "LEX_BAD_ESCAPE"),
            (b"r###\"x\"##".as_slice(), "LEX_BAD_RAW_FENCE"),
            (
                b"\"\"\"\n  good\n bad\n  \"\"\"".as_slice(),
                "LEX_BAD_MULTILINE_INDENT",
            ),
        ];

        // When / Then
        for (source, code) in cases {
            assert_eq!(lex(source).diagnostics()[0].code(), code);
        }
    }

    #[test]
    fn warns_only_when_a_nonzero_float_rounds_to_infinity_or_zero() {
        // Given
        let overflow = convert_literals("1e400");
        let zero = convert_literals("1e-4000");
        let subnormal = convert_literals("5e-324");

        // When / Then
        assert_eq!(overflow.warnings(), ["LEX_PRECISION_LOSS"]);
        assert_eq!(zero.warnings(), ["LEX_PRECISION_LOSS"]);
        assert!(subnormal.warnings().is_empty());
    }

    #[test]
    fn rounds_hexadecimal_f64_midpoints_to_even_and_neighbors_up() {
        // Given
        let midpoint_down = convert_literals("0x1.00000000000008p0");
        let midpoint_up = convert_literals("0x1.00000000000018p0");
        let above_midpoint = convert_literals("0x1.000000000000081p0");

        // When / Then
        assert_eq!(float64_bits(&midpoint_down), 0x3ff0_0000_0000_0000);
        assert_eq!(float64_bits(&midpoint_up), 0x3ff0_0000_0000_0002);
        assert_eq!(float64_bits(&above_midpoint), 0x3ff0_0000_0000_0001);
    }

    #[test]
    fn rounds_hexadecimal_f32_without_f64_double_rounding() {
        // Given
        let above_f32_midpoint = convert_literals("0x1.0000010000000001p0f32");

        // When / Then
        assert_eq!(float32_bits(&above_f32_midpoint), 0x3f80_0001);
    }

    #[test]
    fn rounds_hexadecimal_f32_midpoints_to_even_and_neighbors_up() {
        // Given
        let midpoint = convert_literals("0x1.000001p0f32");
        let midpoint_up = convert_literals("0x1.000003p0f32");
        let above_midpoint = convert_literals("0x1.0000011p0f32");

        // When / Then
        assert_eq!(float32_bits(&midpoint), 0x3f80_0000);
        assert_eq!(float32_bits(&midpoint_up), 0x3f80_0002);
        assert_eq!(float32_bits(&above_midpoint), 0x3f80_0001);
    }

    #[test]
    fn uses_hexadecimal_sticky_bits_beyond_sixty_four_bits() {
        // Given
        let source = "0x1.0000000000000800000000000001p0";

        // When
        let result = convert_literals(source);

        // Then
        assert_eq!(float64_bits(&result), 0x3ff0_0000_0000_0001);
    }

    #[test]
    fn converts_hexadecimal_subnormal_results_at_both_widths() {
        // Given
        let f64_result = convert_literals("0x0.0000000000001p-1022");
        let f32_result = convert_literals("0x0.000002p-126f32");

        // When / Then
        assert_eq!(float64_bits(&f64_result), 0x0000_0000_0000_0001);
        assert_eq!(float32_bits(&f32_result), 0x0000_0001);
    }

    #[test]
    fn parses_decimal_f32_directly_at_a_double_rounding_boundary() {
        // Given
        let midpoint = convert_literals("1.000000059604644775390625f32");
        let above_midpoint = convert_literals("1.000000059604644830901699066162109375f32");

        // When / Then
        assert_eq!(float32_bits(&midpoint), 0x3f80_0000);
        assert_eq!(float32_bits(&above_midpoint), 0x3f80_0001);
    }

    #[test]
    fn warns_for_hexadecimal_overflow_and_underflow_to_zero() {
        // Given
        let overflow = convert_literals("0x1p1024");
        let underflow = convert_literals("0x1p-1075");

        // When / Then
        assert_eq!(float64_bits(&overflow), 0x7ff0_0000_0000_0000);
        assert_eq!(float64_bits(&underflow), 0);
        assert_eq!(overflow.warnings(), ["LEX_PRECISION_LOSS"]);
        assert_eq!(underflow.warnings(), ["LEX_PRECISION_LOSS"]);
    }

    fn float32_bits(result: &super::LiteralConversion) -> u32 {
        match result.values() {
            [Literal::Float32(value)] => value.to_bits(),
            _ => 0,
        }
    }

    fn float64_bits(result: &super::LiteralConversion) -> u64 {
        match result.values() {
            [Literal::Float64(value)] => value.to_bits(),
            _ => 0,
        }
    }

    fn kinds(result: &super::LexedSource) -> Vec<TokenKind> {
        result.tokens().iter().map(|token| token.kind).collect()
    }
}
