use crate::{
    Parser,
    source::{ImportFact, ImportSite, SourceKind, SyntaxId},
};

impl Parser {
    pub(super) fn import_spec(&mut self) -> Option<iris_syntax::ImportSpec> {
        let frame = self.recorder.begin(self.current_offset());
        let result = (|| {
            let name = self.name()?;
            let alias = self.consume("as").then(|| self.name()).flatten();
            Some(iris_syntax::ImportSpec { name, alias })
        })();
        self.recorder.finish(
            frame,
            self.consumed_end,
            result.as_ref().map(|_| SourceKind::Statement),
        );
        result
    }

    pub(super) fn import_fact(
        &self,
        id: SyntaxId,
        import: &iris_syntax::ImportDeclaration,
        separators: &[crate::source::ImportSeparator],
    ) -> SourceKind {
        let mut groups = self.recorder.children(id).iter().filter(|child| {
            matches!(
                self.recorder.document.node(**child).kind,
                SourceKind::Statement
            )
        });
        let target = groups
            .next()
            .map_or_else(Vec::new, |child| self.recorder.names(*child));
        let specs = groups
            .filter_map(|child| {
                let mut names = self.recorder.names(*child).into_iter();
                Some(ImportSite {
                    name: names.next()?,
                    alias: names.next(),
                })
            })
            .collect();
        SourceKind::Import(ImportFact {
            target,
            separators: separators.to_vec(),
            alias: self.recorder.names(id).into_iter().next(),
            specs,
            replacement_authorized: import.replacement_authorized,
        })
    }
}
