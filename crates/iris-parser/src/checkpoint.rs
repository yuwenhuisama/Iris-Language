use crate::{
    Parser, Token,
    source::{ScopeId, SyntaxId},
};

pub(super) struct Checkpoint {
    cursor: usize,
    diagnostics: usize,
    edits: usize,
    nodes: usize,
    scopes: usize,
    roots: usize,
    recovery: usize,
    frames: Vec<(SyntaxId, usize)>,
    scope: ScopeId,
    damaged: Vec<(ScopeId, bool)>,
    consumed_end: usize,
    flags: [bool; 4],
}

impl Parser {
    pub(super) fn checkpoint(&self) -> Checkpoint {
        let mut damaged = Vec::new();
        let mut ancestor = self.recorder.enabled.then_some(self.recorder.scope);
        while let Some(id) = ancestor {
            let scope = self.recorder.document.scope(id);
            damaged.push((id, scope.damaged));
            ancestor = scope.parent;
        }
        Checkpoint {
            cursor: self.cursor,
            diagnostics: self.diagnostics.len(),
            edits: self.token_edits.len(),
            nodes: self.recorder.document.nodes.len(),
            scopes: self.recorder.document.scopes.len(),
            roots: self.recorder.document.roots.len(),
            recovery: self.recorder.document.recovery.len(),
            frames: self
                .recorder
                .frames
                .iter()
                .map(|id| (*id, self.recorder.children(*id).len()))
                .collect(),
            scope: self.recorder.scope,
            damaged,
            consumed_end: self.consumed_end,
            flags: [
                self.no_trailing_block,
                self.no_type_union,
                self.empty_closure_header,
                self.delimited_layout,
            ],
        }
    }

    pub(super) fn restore(&mut self, checkpoint: Checkpoint) {
        self.cursor = checkpoint.cursor;
        self.diagnostics.truncate(checkpoint.diagnostics);
        self.consumed_end = checkpoint.consumed_end;
        [
            self.no_trailing_block,
            self.no_type_union,
            self.empty_closure_header,
            self.delimited_layout,
        ] = checkpoint.flags;
        self.recorder.document.nodes.truncate(checkpoint.nodes);
        self.recorder.document.scopes.truncate(checkpoint.scopes);
        self.recorder.document.roots.truncate(checkpoint.roots);
        self.recorder
            .document
            .recovery
            .truncate(checkpoint.recovery);
        self.recorder.frames.clear();
        for (id, children) in checkpoint.frames {
            self.recorder.document.nodes[id.0]
                .children
                .truncate(children);
            self.recorder.frames.push(id);
        }
        self.recorder.scope = checkpoint.scope;
        for (id, damaged) in checkpoint.damaged {
            self.recorder.document.scopes[id.0].damaged = damaged;
        }
        for (index, token) in self.token_edits.drain(checkpoint.edits..).rev() {
            self.tokens[index] = token;
        }
    }

    pub(super) fn save_token_edit(&mut self) {
        let token: Token = self.tokens[self.cursor].clone();
        self.token_edits.push((self.cursor, token));
    }
}
