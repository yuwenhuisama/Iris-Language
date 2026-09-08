use crate::ByteOffset;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentKind {
    Line,
    Block,
    DocumentationLine,
    DocumentationBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Comment {
    pub kind: CommentKind,
    pub offset: ByteOffset,
    pub end: ByteOffset,
}

#[cfg(test)]
mod tests {
    #[test]
    fn comment_storage_when_lexing_is_not_opted_in() {
        let result = crate::lex(b"/// doc\n/** block */\n// ordinary");
        assert!(result.comments().is_empty());
    }
}
