use crate::{Parser, source};
use iris_syntax::{ClassDeclaration, Decorator};

impl Parser {
    pub(super) fn class_declaration(
        &mut self,
        decorators: Vec<Decorator>,
    ) -> Option<ClassDeclaration> {
        let frame = self.recorder.begin(self.current_offset());
        let mut header = source::DeclarationHeader::default();
        header.span.start = self.current_offset();
        let result = self.class_contents(decorators, &mut header);
        let kind = result.as_ref().and_then(|value| {
            let mut declaration =
                self.source_declaration(frame.id, source::DeclarationKind::Class)?;
            declaration.visible_from = 0;
            declaration.modifiers.reopen = value.reopen;
            declaration.header = Some(header);
            Some(source::SourceKind::Declaration(Box::new(declaration)))
        });
        self.recorder.finish(frame, self.consumed_end, kind);
        result
    }

    fn class_contents(
        &mut self,
        decorators: Vec<Decorator>,
        header: &mut source::DeclarationHeader,
    ) -> Option<ClassDeclaration> {
        let recovery = self.recorder.document.recovery.len();
        header.has_decorators = !decorators.is_empty();
        let reopen = self.consume("open");
        self.expect("class")?;
        let name = self.qualified_name()?;
        self.recorder
            .enter_scope(source::ScopeKind::Class, self.current_offset());
        header.has_type_parameters = self.check("<");
        let parameters = self.generic_parameters();
        let mut extends = None;
        let mut implements = Vec::new();
        let mut mixins = Vec::new();
        let mut constraints = Vec::new();
        let mut meta_deny = Vec::new();
        let mut rank = 0;
        self.skip_newlines();
        while !self.check("{") && !self.at_end() {
            let clause = match self.peek() {
                Some("extends") => 1,
                Some("for") => 2,
                Some("mixin") => 3,
                Some("where") => 4,
                Some("meta") => 5,
                _ => {
                    self.error("PARSE_BAD_HEADER_ORDER");
                    return None;
                }
            };
            if clause <= rank {
                self.error("PARSE_BAD_HEADER_ORDER");
                return None;
            }
            rank = clause;
            match clause {
                1 => {
                    header.has_extends = true;
                    self.advance();
                    extends = self.type_expression();
                }
                2 => {
                    header.has_implements = true;
                    self.advance();
                    implements = self.type_list();
                }
                3 => {
                    header.has_mixins = true;
                    self.advance();
                    mixins = self.mixin_entries();
                }
                4 => {
                    header.has_constraints = true;
                    self.advance();
                    constraints = self.constraints();
                }
                5 => {
                    header.has_meta_policy = true;
                    self.advance();
                    meta_deny = self.meta_deny()?;
                }
                _ => unreachable!(),
            }
            self.skip_newlines();
        }
        self.finish_header(header, recovery);
        let body = self.body()?;
        Some(ClassDeclaration {
            decorators,
            reopen,
            name,
            parameters,
            extends,
            implements,
            mixins,
            constraints,
            meta_deny,
            body,
        })
    }
}
