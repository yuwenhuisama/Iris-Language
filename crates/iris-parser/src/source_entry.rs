use crate::source::{SourceDocument, SourceParse, Span};

pub fn parse_with_source(source: &str) -> SourceParse {
    crate::parse_internal(source, true, false)
}

pub fn parse_editor(source: &str) -> SourceParse {
    crate::parse_internal(source, true, true)
}

pub(super) fn protected_regions(source: &str, document: &mut SourceDocument) {
    let mut end = 0;
    for token in &document.tokens {
        if end < token.offset.0 {
            let gap = &source[end..token.offset.0];
            if !gap.trim().is_empty() {
                document.protected.push(Span {
                    start: end,
                    end: token.offset.0,
                });
            }
        }
        if matches!(
            token.kind,
            iris_lexer::TokenKind::StringLiteral
                | iris_lexer::TokenKind::MutableStringLiteral
                | iris_lexer::TokenKind::BytesLiteral
                | iris_lexer::TokenKind::ByteArrayLiteral
                | iris_lexer::TokenKind::RegexLiteral
        ) {
            document.protected.push(Span {
                start: token.offset.0,
                end: token.end.0,
            });
        }
        end = token.end.0;
    }
    if end < source.len() && !source[end..].trim().is_empty() {
        document.protected.push(Span {
            start: end,
            end: source.len(),
        });
    }
}
