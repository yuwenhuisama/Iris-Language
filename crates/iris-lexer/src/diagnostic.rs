/// A byte offset in the original source input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteOffset(pub usize);

/// A physical source position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourcePosition {
    /// One-based physical line number.
    pub line: usize,
    /// One-based Unicode scalar column number.
    pub column: usize,
}

/// A stable lexical diagnostic for consumers of the front end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: &'static str,
    offset: ByteOffset,
    position: SourcePosition,
}

impl Diagnostic {
    /// Returns the stable conformance diagnostic code.
    pub const fn code(self) -> &'static str {
        self.code
    }

    /// Returns the diagnostic's byte offset in the original source.
    pub const fn offset(self) -> ByteOffset {
        self.offset
    }

    /// Returns the diagnostic's physical source position.
    pub const fn position(self) -> SourcePosition {
        self.position
    }

    pub(crate) const fn new(
        code: &'static str,
        offset: ByteOffset,
        position: SourcePosition,
    ) -> Self {
        Self {
            code,
            offset,
            position,
        }
    }
}
