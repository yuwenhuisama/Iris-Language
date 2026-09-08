use crate::{Parser, source::*};
use iris_syntax::ParameterCategory;

impl Parser {
    pub(super) fn record_parameter_slot(&mut self, id: SyntaxId, category: ParameterCategory) {
        if !self.recorder.enabled {
            return;
        }
        let Some(owner) = self.recorder.frames.iter().rev().nth(1).copied() else {
            return;
        };
        let Some(name) = self.recorder.names(id).into_iter().next() else {
            return;
        };
        let annotation = self.type_children(id).last().copied();
        let default = self.expression_children(id).last().copied();
        self.recorder.document.parameter_slots.push(ParameterSlot {
            owner,
            span: Span {
                start: self.recorder.document.node(id).span.start,
                end: self.consumed_end,
            },
            declaration: (name.text != "_").then_some(id),
            name,
            category,
            annotation,
            default,
        });
    }

    pub(super) fn record_signature(&mut self, header_recovery: usize) {
        let Some(owner) = self.recorder.frames.last().copied() else {
            return;
        };
        let return_type = self.type_children(owner).last().copied();
        self.recorder.document.signatures.push(SignatureSite {
            owner,
            span: Span {
                start: self.recorder.document.node(owner).span.start,
                end: self.consumed_end,
            },
            return_type,
            valid: self.recorder.document.recovery.len() == header_recovery,
        });
    }
}
