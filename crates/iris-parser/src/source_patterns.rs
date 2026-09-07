use crate::{
    Parser,
    source::{DeclarationKind, SourceKind},
};

impl Parser {
    pub(super) fn extend_source_name(&mut self, name: &str) {
        if let Some(id) = self.recorder.last()
            && let SourceKind::Name(site) = &mut self.recorder.document.nodes[id.0].kind
        {
            site.text = name.to_owned();
            site.span.end = self.consumed_end;
            self.recorder.document.nodes[id.0].span.end = self.consumed_end;
        }
    }

    pub(super) fn catch_binding_name(&mut self) -> Option<String> {
        let frame = self.recorder.begin(self.current_offset());
        let result = self.binding_name();
        let kind = result
            .as_ref()
            .and_then(|_| self.source_declaration(frame.id, DeclarationKind::PatternBinding))
            .map(|value| SourceKind::Declaration(Box::new(value)));
        self.recorder.finish(frame, self.consumed_end, kind);
        result
    }

    pub(super) fn activate_pattern_bindings(&mut self) {
        if let Some(owner) = self.recorder.frames.last().copied() {
            let children = self.recorder.children(owner).to_vec();
            for child in children {
                if let SourceKind::Declaration(value) =
                    &mut self.recorder.document.nodes[child.0].kind
                    && value.kind == DeclarationKind::PatternBinding
                {
                    value.visible_from = self.consumed_end;
                }
            }
        }
    }

    pub(super) fn source_match_arm(&mut self) -> Option<iris_syntax::MatchArm> {
        let frame = self.recorder.begin(self.current_offset());
        self.recorder
            .enter_scope(crate::source::ScopeKind::MatchArm, self.current_offset());
        let result = (|| {
            let pattern = self.pattern()?;
            let guard = if self.consume("if") {
                self.expression(0)
            } else {
                None
            };
            self.expect_arrow()?;
            let body = self.match_body()?;
            Some(iris_syntax::MatchArm {
                pattern,
                guard,
                body,
            })
        })();
        self.recorder.finish(
            frame,
            self.consumed_end,
            result.as_ref().map(|_| SourceKind::Pattern),
        );
        result
    }

    pub(super) fn pattern_binding_name(&mut self) -> Option<String> {
        let frame = self.recorder.begin(self.current_offset());
        let result = self.name();
        let kind = result
            .as_ref()
            .filter(|name| *name != "_")
            .and_then(|_| self.source_declaration(frame.id, DeclarationKind::PatternBinding))
            .map(|value| SourceKind::Declaration(Box::new(value)));
        self.recorder.finish(
            frame,
            self.consumed_end,
            kind.or_else(|| result.as_ref().map(|_| SourceKind::Pattern)),
        );
        result
    }
}
