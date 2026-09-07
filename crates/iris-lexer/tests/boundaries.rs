use iris_lexer::{ByteOffset, TokenKind, convert_literals, lex, lex_type};

#[test]
fn separates_arithmetic_when_signs_are_not_exponents() {
    let source = "1+2-3;0x1e+2;1e-2+3;0x1p+2-4";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result
            .tokens()
            .iter()
            .map(|token| token.offset.0)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 17, 18, 19, 20, 26, 27]
    );
}

#[test]
fn preserves_literal_boundaries_when_delimiters_are_complex() {
    let literals = [
        "'''one\ntwo'''",
        "\"\"\"one\ntwo\"\"\"",
        "r##\"a\"#b\"##",
        "mr#'a\\'b'#",
        "br#\"a\"b\"#",
        "mb'''a\nb'''",
        "mbr##\"a\"#b\"##",
        "'${not interpolation'",
        "r\"ends with slash\\\"",
        "\"${call(\"}\", %{key: \"value\"})} tail\"",
        "/${call(\"/\")}/i",
        "r/${not interpolation}/im",
        "r##/a/b/##im",
        "\"${call(/* } \" 0b102 /* nested */ */ %{key: \"value\"})} tail\"",
        "\"${call(\"${nested(\"}\")}\")} tail\"",
        "/[0-9]+\\/tail/im",
    ];

    for literal in literals {
        let source = format!("{literal};next");
        let result = lex(source.as_bytes());

        assert!(result.is_clean(), "{literal}: {result:#?}");
        assert_eq!(result.tokens().len(), 3, "{literal}: {result:#?}");
        assert_eq!(result.tokens()[1].offset.0, literal.len(), "{literal}");
        assert_eq!(
            result.tokens()[0].end,
            ByteOffset(literal.len()),
            "{literal}"
        );
    }
}

#[test]
fn retains_only_exponent_signs_when_numeric_tokens_are_compact() {
    let source = "-1+2--3;2E-4+0X1.P+2-0x1e3;0b10+0o7;.5+1.;1..=2";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    let slices: Vec<_> = result
        .tokens()
        .iter()
        .map(|token| &source[token.offset.0..token.end.0])
        .collect();
    assert_eq!(
        slices,
        [
            "-", "1", "+", "2", "-", "-", "3", ";", "2E-4", "+", "0X1.P+2", "-", "0x1e3", ";",
            "0b10", "+", "0o7", ";", ".5", "+", "1.", ";", "1", "..=", "2"
        ]
    );
}

#[test]
fn preserves_all_prefix_families_when_quotes_and_fences_vary() {
    let prefixes = [
        ("", TokenKind::StringLiteral),
        ("r", TokenKind::StringLiteral),
        ("m", TokenKind::MutableStringLiteral),
        ("mr", TokenKind::MutableStringLiteral),
        ("b", TokenKind::BytesLiteral),
        ("br", TokenKind::BytesLiteral),
        ("mb", TokenKind::ByteArrayLiteral),
        ("mbr", TokenKind::ByteArrayLiteral),
    ];

    for (prefix, kind) in prefixes {
        for quote in ["'", "\"", "'''", "\"\"\""] {
            for fence in ["", "#", "###"] {
                if !prefix.ends_with('r') && !fence.is_empty() {
                    continue;
                }
                let literal = format!("{prefix}{fence}{quote}body{quote}{fence}");
                let source = format!("  {literal}  + next");
                let result = lex(source.as_bytes());

                assert!(result.is_clean(), "{source}: {result:#?}");
                assert_eq!(result.tokens()[0].kind, kind);
                assert_eq!(
                    &source[result.tokens()[0].offset.0..result.tokens()[0].end.0],
                    literal
                );
                assert_eq!(result.tokens().len(), 3);
            }
        }
    }
}

#[test]
fn emits_one_regex_when_raw_prefix_precedes_a_slash() {
    let source = "r/a\\/b/ims";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.tokens(),
        [iris_lexer::Token {
            kind: TokenKind::RegexLiteral,
            offset: ByteOffset(0),
            end: ByteOffset(source.len()),
        }]
    );
}

#[test]
fn ignores_regex_contents_when_converting_fenced_raw_regex() {
    let source = "r##/a/\"unterminated 0b102/##i; 2";

    let result = convert_literals(source);

    assert!(result.diagnostics().is_empty(), "{result:#?}");
    assert_eq!(result.values(), [iris_lexer::Literal::Integer("2".into())]);
}

#[test]
fn reports_fixed_token_ends_when_whitespace_follows() {
    let spellings = [
        "..=", "..<", "..", "%{", "!=", "=~", "!~", "<", "<=", "<=>", ">", ">=", "<<", ">>", "**",
        "==", "&&", "||", "=>", "+=", "-=", "*=", "/=", "**=", "&=", "|=", "^=", "<<=", ">>=",
        "&&=", "||=", "(", ")", "{", "}", ":", ";", ".", "@", "@@", "+", "-", "*", "!", "~", "[",
        "]", ",", "$", "ready?=",
    ];

    for spelling in spellings {
        let source = format!("{spelling}  ");
        let result = lex(source.as_bytes());

        assert!(result.is_clean(), "{spelling}: {result:#?}");
        assert_eq!(result.tokens().len(), 1, "{spelling}: {result:#?}");
        assert_eq!(result.tokens()[0].end.0, spelling.len(), "{spelling}");
    }
}

#[test]
fn reports_exclusive_spans_when_trivia_and_unicode_are_present() {
    let source = "\u{feff}#! iris\r\n\u{e9} /* outer\r\n/* nested\r */\n*/ +\\\r\n2\r";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    let slices: Vec<_> = result
        .tokens()
        .iter()
        .map(|token| &source[token.offset.0..token.end.0])
        .collect();
    assert_eq!(
        slices,
        ["\r\n", "\u{e9}", "\r\n", "\r", "\n", "+", "2", "\r"]
    );
    assert!(
        result
            .tokens()
            .windows(2)
            .all(|pair| pair[0].end.0 <= pair[1].offset.0)
    );
}

#[test]
fn splits_end_spans_when_type_closers_are_adjacent() {
    let source = "Box<Array<T>>";

    let result = lex_type(source.as_bytes());

    let slices: Vec<_> = result
        .tokens()
        .iter()
        .map(|token| &source[token.offset.0..token.end.0])
        .collect();
    assert_eq!(slices, ["Box", "<", "Array", "<", "T", ">", ">"]);
}

#[test]
fn ignores_literal_syntax_when_inside_comments() {
    let source = "// \"unterminated 0b102\n/* r###\"bad /* nested */ 1__0 */\n1+2";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(result.tokens().len(), 5);
    assert_eq!(result.tokens()[0].kind, TokenKind::Newline);
}

#[test]
fn ignores_comment_literals_when_converting_source() {
    let source = "/* 0b102 \"unterminated /* nested */ */ 1 // r#\"bad\n + 2";

    let result = convert_literals(source);

    assert!(result.diagnostics().is_empty(), "{result:#?}");
    assert_eq!(result.values().len(), 2);
}

#[test]
fn starts_regex_after_newline_when_a_block_comment_contains_it() {
    let source = "1/* outer\r\n/* inner */ */ /a/";

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        result.tokens().last().map(|token| token.kind),
        Some(TokenKind::RegexLiteral)
    );
}
