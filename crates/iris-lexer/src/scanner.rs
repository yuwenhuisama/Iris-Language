use crate::{ByteOffset, Diagnostic, SourcePosition};

/// Tokens currently emitted by the source-text scanner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    /// A physical newline not removed by continuation or comments.
    Newline,
    /// Any non-trivia source character reserved for later tokenization work.
    SourceCharacter,
}

/// A token with its original source offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    /// The kind of token recognized by this lexer stage.
    pub kind: TokenKind,
    /// The token's byte offset in the original source.
    pub offset: ByteOffset,
}

/// The successful or failed result of scanning source text.
#[derive(Debug, Default)]
pub struct LexedSource {
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl LexedSource {
    /// Returns source-text tokens available to later lexer stages.
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Returns diagnostics emitted while scanning the source.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Reports whether source text passed this scanning stage.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Scans Iris source bytes according to source-text rules C003 through C007.
pub fn lex(source: &[u8]) -> LexedSource {
    let decoded = match decode_source(source) {
        Ok(decoded) => decoded,
        Err(diagnostic) => return diagnostic_result(diagnostic),
    };

    scan_trivia(decoded)
}

fn decode_source(source: &[u8]) -> Result<DecodedSource<'_>, Diagnostic> {
    let content = match source {
        [0x00, 0x00, 0xFE, 0xFF, ..]
        | [0xFF, 0xFE, 0x00, 0x00, ..]
        | [0xFF, 0xFE, ..]
        | [0xFE, 0xFF, ..] => return Err(invalid_utf8(ByteOffset(0), source)),
        [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
        _ => source,
    };
    let offset_base = source.len() - content.len();
    let text = match std::str::from_utf8(content) {
        Ok(text) => text,
        Err(error) => {
            return Err(invalid_utf8(
                ByteOffset(offset_base + error.valid_up_to()),
                source,
            ));
        }
    };

    Ok(DecodedSource { text, offset_base })
}

fn invalid_utf8(offset: ByteOffset, source: &[u8]) -> Diagnostic {
    Diagnostic::new(
        "LEX_INVALID_UTF8",
        offset,
        source_position_at(source, offset),
    )
}

fn source_position_at(source: &[u8], offset: ByteOffset) -> SourcePosition {
    let prefix = match std::str::from_utf8(&source[..offset.0]) {
        Ok(prefix) => prefix,
        Err(_) => return SourcePosition { line: 1, column: 1 },
    };
    let mut position = SourcePosition { line: 1, column: 1 };
    for character in prefix.chars() {
        if character == '\n' || character == '\r' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }
    }
    position
}

struct DecodedSource<'a> {
    text: &'a str,
    offset_base: usize,
}

fn scan_trivia(source: DecodedSource<'_>) -> LexedSource {
    let bytes = source.text.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut position = SourcePosition { line: 1, column: 1 };

    while index < bytes.len() {
        let offset = ByteOffset(source.offset_base + index);
        match bytes[index..] {
            [b'#', b'!', ..] if index != 0 => {
                return diagnostic_result(Diagnostic::new(
                    "LEX_SHEBANG_NOT_FIRST",
                    offset,
                    position,
                ));
            }
            [b'#', b'!', ..] => {
                let end = line_end(bytes, index);
                advance_to(&mut index, end, bytes, &mut position)
            }
            [b'/', b'/', ..] => {
                let end = line_end(bytes, index);
                advance_to(&mut index, end, bytes, &mut position)
            }
            [b'/', b'*', ..] => match block_comment_end(bytes, index) {
                Some(end) => advance_comment_to(
                    &mut index,
                    end,
                    bytes,
                    source.offset_base,
                    &mut tokens,
                    &mut position,
                ),
                None => {
                    return diagnostic_result(Diagnostic::new(
                        "LEX_UNTERMINATED_COMMENT",
                        offset,
                        position,
                    ));
                }
            },
            [b'\\', b'\n', ..] | [b'\\', b'\r', b'\n', ..] | [b'\\', b'\r', ..] => {
                let end = continuation_end(bytes, index);
                advance_to(&mut index, end, bytes, &mut position);
            }
            [b'\\', ..] => {
                return diagnostic_result(Diagnostic::new(
                    "LEX_BAD_CONTINUATION",
                    offset,
                    position,
                ));
            }
            [b'\n', ..] | [b'\r', ..] => {
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    offset,
                });
                let end = newline_end(bytes, index);
                advance_to(&mut index, end, bytes, &mut position);
            }
            _ => match source.text[index..].chars().next() {
                Some(character) => {
                    tokens.push(Token {
                        kind: TokenKind::SourceCharacter,
                        offset,
                    });
                    index += character.len_utf8();
                    position.column += 1;
                }
                None => break,
            },
        }
    }

    LexedSource {
        tokens,
        diagnostics: Vec::new(),
    }
}

fn diagnostic_result(diagnostic: Diagnostic) -> LexedSource {
    LexedSource {
        tokens: Vec::new(),
        diagnostics: vec![diagnostic],
    }
}

fn block_comment_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        match (bytes[index], bytes[index + 1]) {
            (b'/', b'*') => {
                depth += 1;
                index += 2;
            }
            (b'*', b'/') => {
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

fn line_end(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
        index += 1;
    }
    index
}

fn continuation_end(bytes: &[u8], index: usize) -> usize {
    index + 1 + usize::from(bytes[index + 1] == b'\r' && bytes.get(index + 2) == Some(&b'\n'))
}

fn newline_end(bytes: &[u8], index: usize) -> usize {
    index + 1 + usize::from(bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n'))
}

fn advance_to(index: &mut usize, end: usize, bytes: &[u8], position: &mut SourcePosition) {
    while *index < end {
        if bytes[*index] == b'\n' {
            *index += 1;
            position.line += 1;
            position.column = 1;
        } else if bytes[*index] == b'\r' {
            *index += 1;
            if bytes.get(*index) == Some(&b'\n') && *index < end {
                *index += 1;
            }
            position.line += 1;
            position.column = 1;
        } else {
            *index += 1;
            position.column += 1;
        }
    }
}

fn advance_comment_to(
    index: &mut usize,
    end: usize,
    bytes: &[u8],
    offset_base: usize,
    tokens: &mut Vec<Token>,
    position: &mut SourcePosition,
) {
    while *index < end {
        if bytes[*index] == b'\n' || bytes[*index] == b'\r' {
            tokens.push(Token {
                kind: TokenKind::Newline,
                offset: ByteOffset(offset_base + *index),
            });
            let newline = newline_end(bytes, *index);
            advance_to(index, newline, bytes, position);
        } else {
            *index += 1;
            position.column += 1;
        }
    }
}
