use crate::source::{
    NameSite, Recovery, Scope, ScopeId, ScopeKind, SourceDocument, SourceKind, Span, SyntaxId,
    SyntaxNode,
};

pub(super) struct Recorder {
    pub document: SourceDocument,
    pub enabled: bool,
    pub frames: Vec<SyntaxId>,
    pub scope: ScopeId,
    pub documentation_starts: Vec<(SyntaxId, usize)>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn strict_parse_when_recording_is_disabled_keeps_arenas_unallocated() {
        let result =
            crate::parse_internal("fun make(value = (A<B>>C)) { { ||; value } }", false, false);
        assert!(result.parse.program_accepted);
        assert_eq!(result.source.nodes.capacity(), 0);
        assert_eq!(result.source.scopes.capacity(), 0);
        assert_eq!(result.source.roots.capacity(), 0);
        assert_eq!(result.source.tokens.capacity(), 0);
        assert_eq!(result.source.protected.capacity(), 0);
        assert_eq!(result.source.recovery.capacity(), 0);
        assert_eq!(result.source.comments.capacity(), 0);
        assert_eq!(result.source.documentation.capacity(), 0);
        assert_eq!(result.source.parameter_slots.capacity(), 0);
        assert_eq!(result.source.signatures.capacity(), 0);
        assert_eq!(result.source.calls.capacity(), 0);
    }
}

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub id: SyntaxId,
    scopes: usize,
    scope: ScopeId,
}

impl Recorder {
    pub fn new(end: usize, enabled: bool) -> Self {
        let span = Span { start: 0, end };
        let scopes = if enabled {
            vec![Scope {
                id: ScopeId(0),
                parent: None,
                owner: None,
                kind: ScopeKind::Document,
                span,
                damaged: false,
            }]
        } else {
            Vec::new()
        };
        Self {
            document: SourceDocument {
                span,
                scopes,
                ..SourceDocument::default()
            },
            enabled,
            frames: Vec::new(),
            scope: ScopeId(0),
            documentation_starts: Vec::new(),
        }
    }

    pub fn begin(&mut self, start: usize) -> Frame {
        let id = SyntaxId(self.document.nodes.len());
        let frame = Frame {
            id,
            scopes: self.document.scopes.len(),
            scope: self.scope,
        };
        if self.enabled {
            self.document.nodes.push(SyntaxNode {
                id,
                span: Span { start, end: start },
                scope: self.scope,
                children: Vec::new(),
                kind: SourceKind::Statement,
            });
            self.frames.push(id);
        }
        frame
    }

    pub fn finish(&mut self, frame: Frame, end: usize, kind: Option<SourceKind>) {
        if !self.enabled {
            return;
        }
        self.frames.pop();
        for scope in &mut self.document.scopes[frame.scopes..] {
            if scope.span.end == usize::MAX {
                scope.span.end = end;
            }
        }
        self.scope = frame.scope;
        match kind {
            Some(kind) => {
                let node = &mut self.document.nodes[frame.id.0];
                node.span.end = end;
                node.kind = kind;
                if matches!(node.kind, SourceKind::Declaration(_)) {
                    self.documentation_starts.push((frame.id, node.span.start));
                }
                if matches!(node.kind, SourceKind::Export) {
                    for child in &node.children {
                        self.documentation_starts.push((*child, node.span.start));
                    }
                }
                self.attach(frame.id);
            }
            None => {
                self.document.nodes.truncate(frame.id.0);
                self.prune_metadata(frame.id.0);
                self.document.scopes.truncate(frame.scopes);
                for recovery in &mut self.document.recovery {
                    if recovery.scope.0 >= frame.scopes {
                        recovery.scope = frame.scope;
                    }
                }
                self.document.scopes[frame.scope.0].damaged = true;
            }
        }
    }

    pub fn prune_metadata(&mut self, nodes: usize) {
        self.document.parameter_slots.retain(|slot| {
            slot.owner.0 < nodes
                && slot.declaration.is_none_or(|id| id.0 < nodes)
                && slot.annotation.is_none_or(|id| id.0 < nodes)
                && slot.default.is_none_or(|id| id.0 < nodes)
        });
        self.document
            .signatures
            .retain(|site| site.owner.0 < nodes && site.return_type.is_none_or(|id| id.0 < nodes));
        self.document.calls.retain(|site| site.call.0 < nodes);
        self.documentation_starts.retain(|(id, _)| id.0 < nodes);
    }

    fn attach(&mut self, id: SyntaxId) {
        match self.frames.last() {
            Some(parent) => self.document.nodes[parent.0].children.push(id),
            None => self.document.roots.push(id),
        }
    }

    pub fn enter_scope(&mut self, kind: ScopeKind, start: usize) {
        if !self.enabled {
            return;
        }
        let id = ScopeId(self.document.scopes.len());
        self.document.scopes.push(Scope {
            id,
            parent: Some(self.scope),
            owner: self.frames.last().copied(),
            kind,
            span: Span {
                start,
                end: usize::MAX,
            },
            damaged: false,
        });
        self.scope = id;
    }

    pub fn name(&mut self, site: NameSite) {
        if !self.enabled {
            return;
        }
        let frame = self.begin(site.span.start);
        self.finish(frame, site.span.end, Some(SourceKind::Name(site)));
    }

    pub fn children(&self, id: SyntaxId) -> &[SyntaxId] {
        self.document
            .nodes
            .get(id.0)
            .map_or(&[], |node| &node.children)
    }

    pub fn names(&self, id: SyntaxId) -> Vec<NameSite> {
        self.children(id)
            .iter()
            .filter_map(|child| match &self.document.node(*child).kind {
                SourceKind::Name(site) => Some(site.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn damage(&mut self, span: Span, code: &'static str) {
        if !self.enabled {
            return;
        }
        self.document.scopes[self.scope.0].damaged = true;
        self.document.recovery.push(Recovery {
            span,
            scope: self.scope,
            code,
        });
    }

    pub fn mark(&self) -> usize {
        match self.frames.last() {
            Some(parent) => self.document.nodes[parent.0].children.len(),
            None => self.document.roots.len(),
        }
    }

    pub fn wrap(&mut self, mark: usize, span: Span, kind: SourceKind) {
        if !self.enabled {
            return;
        }
        let children = match self.frames.last() {
            Some(parent) => self.document.nodes[parent.0].children.split_off(mark),
            None => self.document.roots.split_off(mark),
        };
        let id = SyntaxId(self.document.nodes.len());
        self.document.nodes.push(SyntaxNode {
            id,
            span,
            scope: self.scope,
            children,
            kind,
        });
        self.attach(id);
    }

    pub fn last(&self) -> Option<SyntaxId> {
        match self.frames.last() {
            Some(parent) => self.document.nodes[parent.0].children.last().copied(),
            None => self.document.roots.last().copied(),
        }
    }
}
