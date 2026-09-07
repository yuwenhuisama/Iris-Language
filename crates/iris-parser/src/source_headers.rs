use crate::{Parser, source::Span};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DeclarationHeader {
    pub span: Span,
    pub complete: bool,
    pub has_extends: bool,
    pub has_implements: bool,
    pub has_mixins: bool,
    pub has_type_parameters: bool,
    pub has_constraints: bool,
    pub has_meta_policy: bool,
    pub has_decorators: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportSeparatorKind {
    Dot,
    Qualified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportSeparator {
    pub kind: ImportSeparatorKind,
    pub span: Span,
}

impl Parser {
    pub(super) fn finish_header(&self, header: &mut DeclarationHeader, recovery: usize) {
        header.span.end = self.current_offset();
        header.complete =
            self.check("{") && !self.exhausted && self.recorder.document.recovery.len() == recovery;
    }
}
