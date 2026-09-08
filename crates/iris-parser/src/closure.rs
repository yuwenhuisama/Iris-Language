use crate::{Parser, source};
use iris_syntax::{Expression, ParameterCategory};

impl Parser {
    pub(crate) fn closure_literal(&mut self) -> Option<Expression> {
        let frame = self.recorder.begin(self.current_offset());
        let result = self.with_layout(false, Self::closure_contents);
        let kind = result
            .as_ref()
            .filter(|_| self.recorder.enabled)
            .map(|value| source::SourceKind::Expression(self.primary_fact(frame.id, value)));
        self.recorder.finish(frame, self.consumed_end, kind);
        result
    }

    fn closure_contents(&mut self) -> Option<Expression> {
        let header_recovery = self.recorder.document.recovery.len();
        self.recorder
            .enter_scope(source::ScopeKind::Closure, self.current_offset());
        self.expect("{")?;
        self.skip_newlines();
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut has_header = false;
        // `closure_header ::= "|" closure_parameters? "|" ...` admits an EMPTY
        // parameter list, but `||` lexes as ONE logical-or token, so a bare
        // `{ ||; ... }` never reached the header at all. An empty header is
        // consumed whole here, which is the same contextual longest-match
        // C020 applies to `>>` closing two generic argument lists.
        if self.check("||") {
            self.save_token_edit();
            let token = &mut self.tokens[self.cursor];
            // Rewrite the pair into a single `|` and step past it, so the
            // header below sees the empty parameter list it expects.
            token.text = "|".into();
            token.offset += 1;
            self.empty_closure_header = true;
        }
        if self.empty_closure_header {
            self.empty_closure_header = false;
            has_header = true;
            self.advance();
            if self.consume("-") {
                self.expect(">")?;
                return_type = Some(self.type_expression()?);
            }
            self.consume_terminators();
        } else if self.consume("|") {
            has_header = true;
            let outer = std::mem::replace(&mut self.no_type_union, true);
            self.skip_newlines();
            while !self.check("|") && !self.at_end() {
                let frame = self.recorder.begin(self.current_offset());
                let Some(parameter) = self.binding_name() else {
                    self.recorder.finish(frame, self.consumed_end, None);
                    self.no_type_union = outer;
                    return None;
                };
                parameters.push(parameter);
                if self.consume(":") && self.type_expression().is_none() {
                    self.recorder.finish(frame, self.consumed_end, None);
                    self.no_type_union = outer;
                    return None;
                }
                let kind = self
                    .source_declaration(frame.id, source::DeclarationKind::Parameter)
                    .map_or(source::SourceKind::Statement, |value| {
                        source::SourceKind::Declaration(Box::new(value))
                    });
                self.record_parameter_slot(frame.id, ParameterCategory::Positional);
                self.recorder.finish(frame, self.consumed_end, Some(kind));
                self.skip_newlines();
                if !self.consume(",") {
                    break;
                }
                self.skip_newlines();
            }
            // The union level is restored BEFORE the return annotation, which
            // sits outside the parameter list and may legitimately be a union.
            self.no_type_union = outer;
            self.expect("|")?;
            if self.consume("-") {
                self.expect(">")?;
                // C017 needs the annotation to SURVIVE parsing: an annotated
                // and a bare Closure were the same AST node, so the omission
                // it diagnoses could not be observed.
                return_type = Some(self.type_expression()?);
            }
            self.consume_terminators();
        }
        self.record_signature(header_recovery);
        let body_start = self.current_offset();
        let body_frame = self.recorder.begin(body_start);
        let mut body = Vec::new();
        self.skip_newlines();
        while !self.check("}") && !self.at_end() {
            let start = self.cursor;
            if let Some(statement) = self.statement() {
                body.push(statement);
            } else {
                self.advance_to_terminator();
            }
            self.consume_terminators();
            self.ensure_progress(start);
        }
        let closed = self.expect("}");
        self.recorder.finish(
            body_frame,
            body_start.max(self.consumed_end),
            closed.map(|()| source::SourceKind::Body),
        );
        closed?;
        Some(Expression::Closure {
            parameters,
            return_type,
            has_header,
            body,
        })
    }
}
