use crate::{
    Parser,
    source::{ExpressionFact, SourceKind, Span, SyntaxId},
};

impl Parser {
    pub(super) fn record_call(&mut self, mark: usize, start: usize, callee: Option<SyntaxId>) {
        let Some(callee) = callee else {
            return;
        };
        let siblings = match self.recorder.frames.last() {
            Some(parent) => self.recorder.children(*parent),
            None => &self.recorder.document.roots,
        };
        let mut arguments = Vec::new();
        let mut type_arguments = Vec::new();
        for child in &siblings[mark + 1..] {
            match self.recorder.document.node(*child).kind {
                SourceKind::Expression(_) => arguments.push(*child),
                SourceKind::Type(_) => type_arguments.push(*child),
                _ => {}
            }
        }
        self.recorder.wrap(
            mark,
            Span {
                start,
                end: self.consumed_end,
            },
            SourceKind::Expression(ExpressionFact::Call {
                callee,
                arguments,
                type_arguments,
            }),
        );
    }
}
