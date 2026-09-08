use crate::literal::{boundary, convert_number, convert_string};
use crate::{ByteOffset, Comment, CommentKind, Diagnostic, SourcePosition};

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
    MatchTilde,
    NotMatchTilde,
    LessThan,
    LessEqual,
    Spaceship,
    GreaterThan,
    GreaterEqual,
    LeftShift,
    RightShift,
    StarStar,
    EqualEqual,
    AndAnd,
    PipePipe,
    MatchArrow,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    StarStarEqual,
    AmpEqual,
    PipeEqual,
    CaretEqual,
    PercentEqual,
    LeftShiftEqual,
    RightShiftEqual,
    AndAndEqual,
    PipePipeEqual,
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
    At,
    DoubleAt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: ByteOffset,
    /// Exclusive byte offset in the original source, excluding following trivia.
    pub end: ByteOffset,
}

#[derive(Debug, Default)]
pub struct LexedSource {
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    comments: Vec<Comment>,
}

impl LexedSource {
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }
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
    scan(source, Mode::Expression, false)
}
pub fn lex_type(source: &[u8]) -> LexedSource {
    scan(source, Mode::Type, false)
}

pub fn lex_with_comments(source: &[u8]) -> LexedSource {
    scan(source, Mode::Expression, true)
}

fn scan(source: &[u8], mode: Mode, record_comments: bool) -> LexedSource {
    let (text, base) = match decode_source(source) {
        Ok(value) => value,
        Err(diagnostic) => return fail(diagnostic),
    };
    let mut diagnostics = Vec::new();
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut comments = Vec::new();
    let mut index = 0;
    let mut position = SourcePosition { line: 1, column: 1 };
    let mut expression_start = true;
    while index < bytes.len() {
        let offset = ByteOffset(base + index);
        match boundary::string_header(&bytes[index..]) {
            Ok(Some(header)) => {
                let segment = convert_string(&text[index..]);
                if let Some(code) = segment.diagnostic {
                    return fail(Diagnostic::new(code, offset, position));
                }
                tokens.push(Token {
                    kind: header.kind,
                    offset,
                    end: ByteOffset(offset.0 + segment.width),
                });
                advance(&mut index, segment.width, &mut position);
                expression_start = false;
                continue;
            }
            Err(code) => return fail(Diagnostic::new(code, offset, position)),
            Ok(None) => {}
        }
        match bytes[index] {
            b' ' | b'\t' => advance(&mut index, 1, &mut position),
            b'\n' | b'\r' => {
                let width = newline_width(bytes, index);
                push(&mut tokens, TokenKind::Newline, (offset, width));
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
                if record_comments {
                    comments.push(Comment {
                        kind: if bytes.get(index + 2) == Some(&b'/') {
                            CommentKind::DocumentationLine
                        } else {
                            CommentKind::Line
                        },
                        offset,
                        end: ByteOffset(offset.0 + width),
                    });
                }
                advance(&mut index, width, &mut position);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => match comment_end(bytes, index) {
                Some(end) => {
                    if record_comments {
                        comments.push(Comment {
                            kind: if bytes.get(index + 2) == Some(&b'*') {
                                CommentKind::DocumentationBlock
                            } else {
                                CommentKind::Block
                            },
                            offset,
                            end: ByteOffset(base + end),
                        });
                    }
                    if bytes[index..end]
                        .iter()
                        .any(|byte| matches!(byte, b'\n' | b'\r'))
                    {
                        expression_start = true;
                    }
                    advance_comment(&mut tokens, bytes, &mut index, end, base, &mut position);
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
                push(&mut tokens, TokenKind::HashOpen, (offset, 2));
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
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            byte if byte.is_ascii_digit()
                || (byte == b'.'
                    && (bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
                        || bytes.get(index + 1) == Some(&b'_'))) =>
            {
                let segment = convert_number(&text[index..]);
                if let Some(code) = segment.diagnostic {
                    let diagnostic = Diagnostic::new(code, offset, position);
                    if code == "LEX_BAD_NUMERIC_SEPARATOR" {
                        diagnostics.push(diagnostic);
                    } else {
                        return fail(diagnostic);
                    }
                }
                let width = segment.width;
                push(&mut tokens, TokenKind::SourceCharacter, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = false;
            }
            b'.' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Dot,
                offset,
                true,
                &mut expression_start,
            ),
            // C016 lists `=~` and `!~` among the fixed expression operator
            // spellings. Without them `=~` lexed as assignment plus bitwise
            // not, and `!~` failed to lex at all.
            b'=' if bytes.get(index + 1) == Some(&b'~') => {
                push(&mut tokens, TokenKind::MatchTilde, (offset, 2));
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'!' if bytes.get(index + 1) == Some(&b'~') => {
                push(&mut tokens, TokenKind::NotMatchTilde, (offset, 2));
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'!' if bytes.get(index + 1) == Some(&b'=') => {
                push(&mut tokens, TokenKind::BangEqual, (offset, 2));
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'<' => {
                let (kind, width) = match &bytes[index..] {
                    [b'<', b'<', b'=', ..] => (TokenKind::LeftShiftEqual, 3),
                    [b'<', b'=', b'>', ..] => (TokenKind::Spaceship, 3),
                    [b'<', b'<', ..] => (TokenKind::LeftShift, 2),
                    [b'<', b'=', ..] => (TokenKind::LessEqual, 2),
                    _ => (TokenKind::LessThan, 1),
                };
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            b'>' if bytes.get(index + 1) == Some(&b'>') && matches!(mode, Mode::Type) => {
                push(&mut tokens, TokenKind::GreaterThan, (offset, 1));
                push(
                    &mut tokens,
                    TokenKind::GreaterThan,
                    (ByteOffset(offset.0 + 1), 1),
                );
                advance(&mut index, 2, &mut position);
                expression_start = false;
            }
            b'>' if bytes.get(index + 1) == Some(&b'>') => {
                let (kind, width) = if bytes.get(index + 2) == Some(&b'=') {
                    (TokenKind::RightShiftEqual, 3)
                } else {
                    (TokenKind::RightShift, 2)
                };
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            b'>' => {
                let (kind, width) = if bytes.get(index + 1) == Some(&b'=') {
                    (TokenKind::GreaterEqual, 2)
                } else {
                    (TokenKind::GreaterThan, 1)
                };
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            b'/' if bytes.get(index + 1) == Some(&b'=') => {
                push(&mut tokens, TokenKind::SlashEqual, (offset, 2));
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'/' | b'r' if expression_start && boundary::starts_regex(&bytes[index..]) => {
                match boundary::regex_end(&bytes[index..], bytes[index] == b'r') {
                    Ok(width) => {
                        push(&mut tokens, TokenKind::RegexLiteral, (offset, width));
                        advance(&mut index, width, &mut position);
                        expression_start = false;
                    }
                    Err(code) => {
                        return fail(Diagnostic::new(code, offset, position));
                    }
                }
            }
            b'/' => {
                push(&mut tokens, TokenKind::Slash, (offset, 1));
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
                &mut expression_start,
            ),
            b')' => {
                punct(
                    &mut tokens,
                    &mut index,
                    &mut position,
                    TokenKind::RightParen,
                    offset,
                    false,
                    &mut expression_start,
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
                &mut expression_start,
            ),
            b'}' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::RightBrace,
                offset,
                false,
                &mut expression_start,
            ),
            b':' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Colon,
                offset,
                true,
                &mut expression_start,
            ),
            b';' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::Semicolon,
                offset,
                true,
                &mut expression_start,
            ),
            b'@' if bytes.get(index + 1) == Some(&b'@') => {
                push(&mut tokens, TokenKind::DoubleAt, (offset, 2));
                advance(&mut index, 2, &mut position);
                expression_start = true;
            }
            b'@' => punct(
                &mut tokens,
                &mut index,
                &mut position,
                TokenKind::At,
                offset,
                true,
                &mut expression_start,
            ),
            byte @ (b'+' | b'-' | b'*' | b'&' | b'|' | b'^' | b'%' | b'=') => {
                let (kind, width) = fixed_operator(bytes, index, byte);
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = true;
            }
            byte if is_identifier_start(byte) => {
                let end = identifier_end(bytes, index);
                // C009 forbids Pattern_Syntax, Pattern_White_Space,
                // controls and default-ignorable code points in an
                // identifier. Such a scalar yields an EMPTY identifier, and
                // reporting it here is what stops the scanner from making
                // no progress and looping forever.
                if end == index {
                    return fail(Diagnostic::new("LEX_INVALID_IDENTIFIER", offset, position));
                }
                // C019 admits a selector SUFFIX before `=` in a setter
                // selector, naming both `ready?=` and `value!=`. Only `?=`
                // was recognised, so `value!=` lexed as `value` plus the
                // inequality operator and could not be declared.
                //
                // `!==` is NOT a setter selector: that is `value!` compared
                // with `==`, so the byte after `=` must not be another `=`.
                let suffixed = matches!(bytes.get(end), Some(&b'?') | Some(&b'!'))
                    && bytes.get(end + 1) == Some(&b'=')
                    && bytes.get(end + 2) != Some(&b'=');
                let kind = if suffixed {
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
                push(&mut tokens, kind, (offset, width));
                advance(&mut index, width, &mut position);
                expression_start = false;
            }
            _ => {
                push(&mut tokens, TokenKind::SourceCharacter, (offset, 1));
                advance(&mut index, 1, &mut position);
                expression_start = true;
            }
        }
    }
    LexedSource {
        tokens,
        diagnostics,
        comments,
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
pub(crate) fn comment_end(bytes: &[u8], mut index: usize) -> Option<usize> {
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
            let width = newline_width(bytes, *index);
            tokens.push(Token {
                kind: TokenKind::Newline,
                offset: ByteOffset(base + *index),
                end: ByteOffset(base + *index + width),
            });
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
    starts_expression: bool,
    expression_start: &mut bool,
) {
    push(tokens, kind, (offset, 1));
    advance(index, 1, position);
    // `IRIS-V1-COLLECTIONS-C023` makes a slash open a Regex literal only where a
    // primary expression is expected. Every caller passed that answer in and it
    // was discarded, so an opening brace never restored the expression position
    // and `fun f() { /a/ }` could not lex a Regex at all.
    *expression_start = starts_expression;
}
fn push(tokens: &mut Vec<Token>, kind: TokenKind, (offset, width): (ByteOffset, usize)) {
    tokens.push(Token {
        kind,
        offset,
        end: ByteOffset(offset.0 + width),
    });
}
fn fixed_operator(bytes: &[u8], index: usize, byte: u8) -> (TokenKind, usize) {
    let remaining = &bytes[index..];
    match remaining {
        [b'*', b'*', b'=', ..] => (TokenKind::StarStarEqual, 3),
        [b'&', b'&', b'=', ..] => (TokenKind::AndAndEqual, 3),
        [b'|', b'|', b'=', ..] => (TokenKind::PipePipeEqual, 3),
        [b'*', b'*', ..] => (TokenKind::StarStar, 2),
        [b'&', b'&', ..] => (TokenKind::AndAnd, 2),
        [b'|', b'|', ..] => (TokenKind::PipePipe, 2),
        [b'=', b'=', ..] => (TokenKind::EqualEqual, 2),
        [b'=', b'>', ..] => (TokenKind::MatchArrow, 2),
        [b'+', b'=', ..] => (TokenKind::PlusEqual, 2),
        [b'-', b'=', ..] => (TokenKind::MinusEqual, 2),
        [b'*', b'=', ..] => (TokenKind::StarEqual, 2),
        [b'&', b'=', ..] => (TokenKind::AmpEqual, 2),
        [b'|', b'=', ..] => (TokenKind::PipeEqual, 2),
        [b'^', b'=', ..] => (TokenKind::CaretEqual, 2),
        [b'%', b'=', ..] => (TokenKind::PercentEqual, 2),
        _ => (
            match byte {
                b'+' | b'-' | b'*' | b'&' | b'|' | b'^' | b'%' | b'=' => TokenKind::SourceCharacter,
                _ => unreachable!(),
            },
            1,
        ),
    }
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
/// Whether a byte can OPEN an identifier.
///
/// `IRIS-V1-GRAMMAR-C009` makes identifier start Unicode XID_Start or `_`, so
/// a non-ASCII lead byte is admitted here and the scalar itself is classified
/// in `identifier_end`, which walks scalars rather than bytes.
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
}

/// The end of an identifier starting at `index`.
///
/// `C009` classifies start by XID_Start and continuation by XID_Continue, both
/// under the `IRIS-V1-COLLECTIONS-C042` Unicode version, and forbids
/// Pattern_Syntax, Pattern_White_Space, controls and default-ignorable code
/// points even when Unicode would otherwise admit them.
fn identifier_end(bytes: &[u8], index: usize) -> usize {
    let Ok(text) = core::str::from_utf8(&bytes[index..]) else {
        return index;
    };
    let mut end = index;
    for (offset, scalar) in text.char_indices() {
        let admitted = if offset == 0 {
            scalar == '_' || xid_start(scalar)
        } else {
            scalar == '_' || xid_continue(scalar)
        };
        if !admitted || forbidden_in_identifier(scalar) {
            break;
        }
        end = index + offset + scalar.len_utf8();
    }
    end
}

fn xid_start(scalar: char) -> bool {
    icu_properties::CodePointSetData::new::<icu_properties::props::XidStart>().contains(scalar)
}

fn xid_continue(scalar: char) -> bool {
    icu_properties::CodePointSetData::new::<icu_properties::props::XidContinue>().contains(scalar)
}

/// `C009` forbids these in an identifier even when Unicode classifies them.
fn forbidden_in_identifier(scalar: char) -> bool {
    icu_properties::CodePointSetData::new::<icu_properties::props::PatternSyntax>().contains(scalar)
        || icu_properties::CodePointSetData::new::<icu_properties::props::PatternWhiteSpace>()
            .contains(scalar)
        || icu_properties::CodePointSetData::new::<icu_properties::props::DefaultIgnorableCodePoint>()
            .contains(scalar)
        || scalar.is_control()
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
            | b"typeof"
            // C072 widens the inventory a second time, adding exactly `yield`.
            | b"yield"
    )
}
fn fail(diagnostic: Diagnostic) -> LexedSource {
    LexedSource {
        tokens: Vec::new(),
        diagnostics: vec![diagnostic],
        comments: Vec::new(),
    }
}
