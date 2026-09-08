use crate::{
    Parser,
    source::{
        ArgumentKind, ArgumentSlot, CallSite, ExpressionFact, NameSite, SourceKind, Span, SyntaxId,
    },
};

impl Parser {
    pub(super) fn record_call(
        &mut self,
        (mark, start, callee): (usize, usize, Option<SyntaxId>),
        site: Option<CallSite>,
    ) {
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
        if let Some(site) = &site {
            arguments = site
                .arguments
                .iter()
                .filter_map(|slot| slot.expression)
                .collect();
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
        if let Some(mut site) = site
            && let Some(call) = self.recorder.last()
        {
            site.call = call;
            self.recorder.document.calls.push(site);
        }
    }

    pub(super) fn call_arguments(
        &mut self,
    ) -> Option<(Vec<iris_syntax::Expression>, Option<CallSite>)> {
        if !self.recorder.enabled {
            return self.arguments().map(|arguments| (arguments, None));
        }
        let open = Span {
            start: self.consumed_end - 1,
            end: self.consumed_end,
        };
        self.with_layout(true, |parser| {
            let mut site = CallSite {
                call: SyntaxId(0),
                open,
                close: None,
                commas: Vec::new(),
                arguments: Vec::new(),
                end: open.end,
                incomplete: false,
            };
            let mut values = Vec::new();
            parser.skip_newlines();
            loop {
                if parser.check(")") {
                    let start = parser.current_offset();
                    parser.advance();
                    site.close = Some(Span {
                        start,
                        end: parser.consumed_end,
                    });
                    site.end = parser.consumed_end;
                    return Some((values, Some(site)));
                }
                let start = parser.current_offset();
                let outer = std::mem::replace(&mut parser.call_argument_recovery, true);
                let kind = match parser.peek_keyword_argument_name() {
                    Some(text) => ArgumentKind::Keyword(NameSite {
                        span: Span {
                            start,
                            end: start + text.len(),
                        },
                        text,
                    }),
                    None => ArgumentKind::Positional,
                };
                let recovery = parser.recorder.document.recovery.len();
                let value = parser.argument();
                parser.call_argument_recovery = outer;
                let expression = value.as_ref().and_then(|_| parser.recorder.last());
                let incomplete =
                    value.is_none() || parser.recorder.document.recovery.len() > recovery;
                site.arguments.push(ArgumentSlot {
                    span: Span {
                        start,
                        end: parser.consumed_end.max(start),
                    },
                    kind,
                    expression,
                    incomplete,
                });
                site.incomplete |= incomplete;
                match value {
                    Some(value) => values.push(value),
                    None if parser.editor
                        && !parser.exhausted
                        && matches!(parser.peek(), None | Some(")" | "}" | "]")) =>
                    {
                        parser.error("PARSE_UNEXPECTED_TOKEN");
                        site.end = parser.current_offset();
                        if parser.check(")") {
                            parser.advance();
                            site.close = Some(Span {
                                start: site.end,
                                end: parser.consumed_end,
                            });
                            site.end = parser.consumed_end;
                        }
                        return Some((values, Some(site)));
                    }
                    None => return None,
                }
                if parser.check(")") {
                    continue;
                }
                if parser.editor
                    && !parser.exhausted
                    && matches!(parser.peek(), None | Some("}" | "]"))
                {
                    parser.error("PARSE_UNEXPECTED_TOKEN");
                    site.incomplete = true;
                    site.end = parser.current_offset();
                    return Some((values, Some(site)));
                }
                let start = parser.current_offset();
                parser.expect(",")?;
                site.commas.push(Span {
                    start,
                    end: parser.consumed_end,
                });
                parser.skip_newlines();
            }
        })
    }

    pub(super) fn trailing_call_block(
        &mut self,
        site: &mut Option<CallSite>,
    ) -> Option<iris_syntax::Expression> {
        let start = self.current_offset();
        let recovery = self.recorder.document.recovery.len();
        let result = self.closure_literal()?;
        if let Some(site) = site {
            let incomplete = self.recorder.document.recovery.len() > recovery;
            site.arguments.push(ArgumentSlot {
                span: Span {
                    start,
                    end: self.consumed_end,
                },
                kind: ArgumentKind::TrailingBlock,
                expression: self.recorder.last(),
                incomplete,
            });
            site.incomplete |= incomplete;
            site.end = self.consumed_end;
        }
        Some(result)
    }
}
