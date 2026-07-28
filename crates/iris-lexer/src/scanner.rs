use crate::{ByteOffset, Diagnostic, SourcePosition};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Newline,
    SourceCharacter,
    Identifier,
    Keyword,
    SetterSelector,
    RangeInclusive,
    RangeExclusive,
    ContractView,
    HashOpen,
    BangEqual,
    LessThan,
    GreaterThan,
    RightShift,
    Slash,
    RegexLiteral,
    MutableStringLiteral,
    BytesLiteral,
    ByteArrayLiteral,
    StringLiteral,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Semicolon,
    Dot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: ByteOffset,
}

#[derive(Debug, Default)]
pub struct LexedSource {
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl LexedSource {
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Expression,
    Type,
}

pub fn lex(source: &[u8]) -> LexedSource {
    scan(source, Mode::Expression)
}
pub fn lex_type(source: &[u8]) -> LexedSource {
    scan(source, Mode::Type)
}

fn scan(source: &[u8], mode: Mode) -> LexedSource {
    let (text, base) = match decode_source(source) {
        Ok(value) => value,
        Err(diagnostic) => return fail(diagnostic),
    };
    let literal_conversion = crate::convert_literals(text);
    let diagnostics = match literal_conversion.diagnostics().first() {
        Some(&"LEX_BAD_NUMERIC_SEPARATOR") => vec![Diagnostic::new(
            "LEX_BAD_NUMERIC_SEPARATOR",
            ByteOffset(base),
            SourcePosition { line: 1, column: 1 },
        )],
        Some(code) => {
            return fail(Diagnostic::new(
                code,
                ByteOffset(base),
                SourcePosition { line: 1, column: 1 },
            ));
        }
        None => Vec::new(),
    };
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut position = SourcePosition { line: 1, column: 1 };
    let mut expression_start = true;
    while index < bytes.len() {
        let offset = ByteOffset(base + index);
        match bytes[index] {
            b' ' | b'\t' => advance(&mut index, 1, &mut position),
            b'\n' | b'\r' => {
                let width = newline_width(bytes, index);
                push(&mut tokens, TokenKind::Newline, offset);
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            b'#' if bytes.get(index + 1) == Some(&b'!') => {
                if index != 0 {
                    return fail(Diagnostic::new("LEX_SHEBANG_NOT_FIRST", offset, position));
                }
                let width = line_end(bytes, index) - index;
                advance(&mut index, width, &mut position);
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let width = line_end(bytes, index) - index;
                advance(&mut index, width, &mut position);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => match comment_end(bytes, index) {
                Some(end) => {
                    advance_comment(&mut tokens, bytes, &mut index, end, base, &mut position)
                }
                None => {
                    return fail(Diagnostic::new(
                        "LEX_UNTERMINATED_COMMENT",
                        offset,
                        position,
                    ));
                }
            },
            b'\\' => match bytes.get(index + 1) {
                Some(b'\n') | Some(b'\r') => {
                    let width = 1 + newline_width(bytes, index + 1);
                    advance(&mut index, width, &mut position);
                }
                _ => return fail(Diagnostic::new("LEX_BAD_CONTINUATION", offset, position)),
            },
            b'$' if bytes.get(index + 1) == Some(&b'{') => {
                return fail(Diagnostic::new(
                    "LEX_INTERPOLATION_OUTSIDE_LITERAL",
                    offset,
                    position,
                ));
            }
            b'%' if bytes.get(index + 1) == Some(&b'{') => {
                push(&mut tokens, TokenKind::HashOpen, offset);
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'.' if bytes.get(index + 1) == Some(&b'.') => {
                let kind = match bytes.get(index + 2) {
                    Some(b'=') => TokenKind::RangeInclusive,
                    Some(b'<') => TokenKind::RangeExclusive,
                    _ => TokenKind::ContractView,
                };
                let width = match kind {
                    TokenKind::RangeInclusive | TokenKind::RangeExclusive => 3,
                    TokenKind::ContractView => 2,
                    _ => unreachable!(),
                };
                push(&mut tokens, kind, offset);
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            b'.' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Dot,
                offset,
                true,
            ),
            b'!' if bytes.get(index + 1) == Some(&b'=') => {
                push(&mut tokens, TokenKind::BangEqual, offset);
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'<' => {
                push(&mut tokens, TokenKind::LessThan, offset);
                advance(&mut index, 1, &mut position);
                expression_start = true;
            }
            b'>' if bytes.get(index + 1) == Some(&b'>') && matches!(mode, Mode::Type) => {
                push(&mut tokens, TokenKind::GreaterThan, offset);
                push(
                    &mut tokens,
                    TokenKind::GreaterThan,
                    ByteOffset(offset.0 + 1),
                );
                advance(&mut index, 2, &mut position);
                expression_start = false;
            }
            b'>' if bytes.get(index + 1) == Some(&b'>') => {
                push(&mut tokens, TokenKind::RightShift, offset);
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'>' => {
                push(&mut tokens, TokenKind::GreaterThan, offset);
                advance(&mut index, 1, &mut position);
                expression_start = true;
            }
            b'/' if expression_start => match regex_end(bytes, index + 1) {
                Some(end) => {
                    let width = end - index;
                    push(&mut tokens, TokenKind::RegexLiteral, offset);
                    advance(&mut index, width, &mut position);
                    expression_start = false;
                }
                None => {
                    return fail(Diagnostic::new(
                        "LEX_UNTERMINATED_LITERAL",
                        offset,
                        position,
                    ));
                }
            },
            b'/' => {
                push(&mut tokens, TokenKind::Slash, offset);
                advance(&mut index, 1, &mut position);
                expression_start = true;
            }
            b'(' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::LeftParen,
                offset,
                true,
            ),
            b')' => {
                punct(
                    &mut tokens,
                    &mut index,
                    &mut position,
                    TokenKind::RightParen,
                    offset,
                    false,
                );
                expression_start = false;
            }
            b'{' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::LeftBrace,
                offset,
                true,
            ),
            b'}' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::RightBrace,
                offset,
                false,
            ),
            b':' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Colon,
                offset,
                true,
            ),
            b';' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Semicolon,
                offset,
                true,
            ),
            b'"' | b'\'' => match quoted_end(bytes, index, bytes[index]) {
                Some(end) => {
                    let width = end - index;
                    push(&mut tokens, TokenKind::StringLiteral, offset);
                    advance(&mut index, width, &mut position);
                    expression_start = false;
                }
                None => {
                    return fail(Diagnostic::new(
                        "LEX_UNTERMINATED_LITERAL",
                        offset,
                        position,
                    ));
                }
            },
            byte if is_identifier_start(byte) => match prefixed_literal(bytes, index) {
                Ok(Some((kind, end))) => {
                    let width = end - index;
                    push(&mut tokens, kind, offset);
                    advance(&mut index, width, &mut position);
                    expression_start = false;
                }
                Ok(None) => {
                    let end = identifier_end(bytes, index);
                    let kind = if bytes.get(end) == Some(&b'?') && bytes.get(end + 1) == Some(&b'=')
                    {
                        TokenKind::SetterSelector
                    } else if keyword(&bytes[index..end]) {
                        TokenKind::Keyword
                    } else {
                        TokenKind::Identifier
                    };
                    let width = if kind == TokenKind::SetterSelector {
                        end + 2 - index
                    } else {
                        end - index
                    };
                    push(&mut tokens, kind, offset);
                    advance(&mut index, width, &mut position);
                    expression_start = false;
                }
                Err(unterminated) => {
                    return fail(Diagnostic::new(
                        if unterminated {
                            "LEX_UNTERMINATED_LITERAL"
                        } else {
                            "LEX_BAD_LITERAL_PREFIX"
                        },
                        offset,
                        position,
                    ));
                }
            },
            _ => {
                push(&mut tokens, TokenKind::SourceCharacter, offset);
                advance(&mut index, 1, &mut position);
                expression_start = true;
            }
        }
    }
    LexedSource {
        tokens,
        diagnostics,
    }
}

fn decode_source(source: &[u8]) -> Result<(&str, usize), Diagnostic> {
    let content = match source {
        [0x00, 0x00, 0xFE, 0xFF, ..]
        | [0xFF, 0xFE, 0x00, 0x00, ..]
        | [0xFF, 0xFE, ..]
        | [0xFE, 0xFF, ..] => {
            return Err(Diagnostic::new(
                "LEX_INVALID_UTF8",
                ByteOffset(0),
                SourcePosition { line: 1, column: 1 },
            ));
        }
        [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
        _ => source,
    };
    match std::str::from_utf8(content) {
        Ok(text) => Ok((text, source.len() - content.len())),
        Err(error) => Err(Diagnostic::new(
            "LEX_INVALID_UTF8",
            ByteOffset(source.len() - content.len() + error.valid_up_to()),
            SourcePosition { line: 1, column: 1 },
        )),
    }
}
fn prefixed_literal(bytes: &[u8], index: usize) -> Result<Option<(TokenKind, usize)>, bool> {
    let prefixes: &[(&[u8], TokenKind)] = &[
        (b"mbr", TokenKind::ByteArrayLiteral),
        (b"mr", TokenKind::MutableStringLiteral),
        (b"mb", TokenKind::ByteArrayLiteral),
        (b"br", TokenKind::BytesLiteral),
        (b"m", TokenKind::MutableStringLiteral),
        (b"b", TokenKind::BytesLiteral),
        (b"r", TokenKind::StringLiteral),
    ];
    for (prefix, kind) in prefixes {
        if bytes[index..].starts_with(prefix)
            && bytes
                .get(index + prefix.len())
                .is_some_and(|byte| *byte == b'"' || *byte == b'\'')
        {
            return quoted_end(bytes, index + prefix.len(), bytes[index + prefix.len()])
                .map(|end| Some((*kind, end)))
                .ok_or(true);
        }
    }
    if [b"rm".as_slice(), b"bm", b"rb", b"brm"]
        .iter()
        .any(|prefix| {
            bytes[index..].starts_with(prefix)
                && bytes
                    .get(index + prefix.len())
                    .is_some_and(|byte| *byte == b'"' || *byte == b'\'')
        })
    {
        return Err(false);
    }
    Ok(None)
}
fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> Option<usize> {
    let mut index = start + 1;
    let mut interpolation = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            byte if byte == quote && interpolation == 0 => return Some(index + 1),
            b'$' if bytes.get(index + 1) == Some(&b'{') => {
                interpolation += 1;
                index += 2;
            }
            b'}' if interpolation > 0 => {
                interpolation -= 1;
                index += 1;
            }
            b'\n' | b'\r' => return None,
            _ => index += 1,
        }
    }
    None
}
fn regex_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'/' => {
                index += 1;
                while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    index += 1;
                }
                return Some(index);
            }
            b'\n' | b'\r' => return None,
            _ => index += 1,
        }
    }
    None
}
fn comment_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    let mut depth = 1;
    index += 2;
    while index + 1 < bytes.len() {
        match &bytes[index..index + 2] {
            b"/*" => {
                depth += 1;
                index += 2;
            }
            b"*/" => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += 1,
        }
    }
    None
}
fn advance_comment(
    tokens: &mut Vec<Token>,
    bytes: &[u8],
    index: &mut usize,
    end: usize,
    base: usize,
    position: &mut SourcePosition,
) {
    while *index < end {
        if matches!(bytes[*index], b'\n' | b'\r') {
            push(tokens, TokenKind::Newline, ByteOffset(base + *index));
            let width = newline_width(bytes, *index);
            *index += width;
            position.line += 1;
            position.column = 1;
        } else {
            *index += 1;
            position.column += 1;
        }
    }
}
fn punct(
    tokens: &mut Vec<Token>,
    index: &mut usize,
    position: &mut SourcePosition,
    kind: TokenKind,
    offset: ByteOffset,
    expression_start: bool,
) {
    push(tokens, kind, offset);
    advance(index, 1, position);
    let _ = expression_start;
}
fn push(tokens: &mut Vec<Token>, kind: TokenKind, offset: ByteOffset) {
    tokens.push(Token { kind, offset });
}
fn advance(index: &mut usize, count: usize, position: &mut SourcePosition) {
    *index += count;
    position.column += count;
}
fn newline_width(bytes: &[u8], index: usize) -> usize {
    1 + usize::from(bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n'))
}
fn line_end(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && !matches!(bytes[index], b'\n' | b'\r') {
        index += 1;
    }
    index
}
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}
fn identifier_end(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        index += 1;
    }
    index
}
fn keyword(text: &[u8]) -> bool {
    matches!(
        text,
        b"property"
            | b"fun"
            | b"let"
            | b"class"
            | b"module"
            | b"contract"
            | b"return"
            | b"true"
            | b"false"
            | b"nil"
    )
}
fn fail(diagnostic: Diagnostic) -> LexedSource {
    LexedSource {
        tokens: Vec::new(),
        diagnostics: vec![diagnostic],
    }
}
