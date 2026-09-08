use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

use crate::source::{ExpressionFact, NameSite, SourceKind, Span};
use crate::{Associativity, Parser, is_identifier, is_reserved_keyword};

impl Parser {
    pub(super) fn expression(&mut self, minimum: u8) -> Option<Expression> {
        if self.expression_depth == 0 {
            self.expression_nodes = 0;
        }
        self.expression_node()?;
        self.expression_depth += 1;
        let result = self.nested(|parser| parser.expression_contents(minimum));
        self.expression_depth -= 1;
        result
    }

    fn expression_contents(&mut self, minimum: u8) -> Option<Expression> {
        self.skip_newlines();
        let start = self.current_offset();
        let mark = self.recorder.mark();
        let mut left = self.prefix()?;
        self.expression_layout();
        while let Some((precedence, associativity)) = self.infix() {
            if precedence < minimum {
                break;
            }
            let operator = self.consume_infix_operator()?;
            let next = if associativity == Associativity::Right {
                precedence
            } else {
                precedence + 1
            };
            // The chapter 02 grammar puts a TYPE on the right of `is`, `as` and
            // `as?`. Parsing it as an expression would read the `<` of a closed
            // generic such as `Dynamic<A>` as a comparison operator.
            let right = if matches!(
                operator,
                BinaryOperator::Is | BinaryOperator::As | BinaryOperator::AsOptional
            ) {
                let target = self.type_expression()?;
                let name = type_expression_name(&target)?;
                if matches!(target, iris_syntax::TypeExpression::Generic { .. }) {
                    Expression::ReifiedType(target)
                } else {
                    Expression::Name(name)
                }
            } else {
                self.expression(next)?
            };
            if associativity == Associativity::NonAssociative
                && self
                    .infix()
                    .is_some_and(|(candidate, _)| candidate == precedence)
            {
                self.error("PARSE_NONASSOCIATIVE_CHAIN");
                return None;
            }
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
            self.recorder.wrap(
                mark,
                Span {
                    start,
                    end: self.consumed_end,
                },
                SourceKind::Expression(ExpressionFact::Unsupported { form: "binary" }),
            );
            self.expression_layout();
        }
        if minimum == 0 && !(self.check("=") && self.peek_next() == Some(">")) {
            if let Some(operator) = self.assignment_operator() {
                let target = self.recorder.last();
                self.advance();
                let right = self.expression(0)?;
                if let Some((target, value)) = target.zip(self.recorder.last()) {
                    self.recorder.wrap(
                        mark,
                        Span {
                            start,
                            end: self.consumed_end,
                        },
                        SourceKind::Expression(ExpressionFact::Assignment {
                            target,
                            value,
                            operator,
                        }),
                    );
                }
                left = Expression::Assignment {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else if self.consume("%=") {
                // `IRIS-V1-CONTROL-V324` NAMES this code, so it is used rather
                // than the locally invented one: `%` is not an Iris v1 operator
                // and therefore has no compound form.
                self.error("PARSE_UNSUPPORTED_COMPOUND_ASSIGNMENT");
                return None;
            }
        }
        Some(left)
    }

    fn prefix(&mut self) -> Option<Expression> {
        if matches!(self.peek(), Some("yield" | "await" | "+" | "-" | "~" | "!")) {
            let frame = self.recorder.begin(self.current_offset());
            let result = self.prefix_contents();
            let kind = result
                .as_ref()
                .filter(|_| self.recorder.enabled)
                .map(|value| SourceKind::Expression(self.primary_fact(frame.id, value)));
            self.recorder.finish(frame, self.consumed_end, kind);
            result
        } else {
            self.postfix()
        }
    }

    fn prefix_contents(&mut self) -> Option<Expression> {
        // C071 binds `await` at `unary_expr`, so its operand is a unary
        // expression: tighter than any binary operator, looser than a postfix
        // call, making `await f()` an await of the call's result.
        // C072 puts `yield` at unary precedence beside `await`. Its operand is
        // OPTIONAL, so a bare `yield` suspends with nil.
        if self.peek() == Some("yield") {
            self.advance();
            // A bare `yield` suspends with nil, so an operand is read only
            // when one actually follows on this line.
            let value = if matches!(self.peek(), None | Some("\n") | Some(";") | Some("}")) {
                None
            } else {
                self.expression(14).map(Box::new)
            };
            return Some(Expression::Yield(value));
        }
        if self.peek() == Some("await") {
            self.advance();
            return self
                .expression(14)
                .map(|operand| Expression::Await(Box::new(operand)));
        }
        let unary = match self.peek() {
            Some("+") => Some(UnaryOperator::Plus),
            Some("-") => Some(UnaryOperator::Negate),
            Some("~") => Some(UnaryOperator::BitwiseNot),
            Some("!") => Some(UnaryOperator::Not),
            _ => None,
        };
        if let Some(operator) = unary {
            self.advance();
            return self.expression(14).map(|operand| Expression::Unary {
                operator,
                operand: Box::new(operand),
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Option<Expression> {
        let start = self.current_offset();
        let mark = self.recorder.mark();
        let mut expression = self.primary()?;
        loop {
            self.expression_layout();
            self.expression_node()?;
            let receiver = self.recorder.last();
            if self.consume(".") {
                let dot = Span {
                    start: self.consumed_end - 1,
                    end: self.consumed_end,
                };
                if self.editor && matches!(self.peek(), None | Some("\n" | ";" | "}")) {
                    self.error("PARSE_UNEXPECTED_TOKEN");
                    if let Some(receiver) = receiver {
                        self.recorder.wrap(
                            mark,
                            Span {
                                start,
                                end: dot.end,
                            },
                            SourceKind::Expression(ExpressionFact::IncompleteMember {
                                receiver,
                                dot,
                            }),
                        );
                    }
                    return Some(expression);
                }
                let selector_start = self.current_offset();
                let selector = self.selector()?;
                if let Some(receiver) = receiver {
                    let name = NameSite {
                        text: selector.clone(),
                        span: Span {
                            start: selector_start,
                            end: self.consumed_end,
                        },
                    };
                    self.recorder.wrap(
                        mark,
                        Span {
                            start,
                            end: self.consumed_end,
                        },
                        SourceKind::Expression(ExpressionFact::Member {
                            receiver,
                            name,
                            contract: false,
                        }),
                    );
                }
                expression = Expression::Member {
                    receiver: Box::new(expression),
                    selector,
                };
            } else if self.consume("..") {
                let selector_start = self.current_offset();
                let selector = self.selector()?;
                if let Some(receiver) = receiver {
                    let name = NameSite {
                        text: selector.clone(),
                        span: Span {
                            start: selector_start,
                            end: self.consumed_end,
                        },
                    };
                    self.recorder.wrap(
                        mark,
                        Span {
                            start,
                            end: self.consumed_end,
                        },
                        SourceKind::Expression(ExpressionFact::Member {
                            receiver,
                            name,
                            contract: true,
                        }),
                    );
                }
                expression = Expression::ContractView {
                    receiver: Box::new(expression),
                    selector,
                };
            } else if self.check("[") && !self.newline_before_cursor() {
                // A `[` on the SAME logical line is an index, while one that
                // starts a line is an Array literal statement. Without this the
                // postfix form would swallow a following literal.
                self.advance();
                let index = self.with_layout(true, |parser| parser.expression(0))?;
                self.expect("]")?;
                expression = Expression::Index {
                    receiver: Box::new(expression),
                    index: Box::new(index),
                };
                self.recorder.wrap(
                    mark,
                    Span {
                        start,
                        end: self.consumed_end,
                    },
                    SourceKind::Expression(ExpressionFact::Unsupported { form: "index" }),
                );
            } else if self.check("<")
                && let Some(type_arguments) = self.call_type_arguments()
            {
                // C066 admits explicit Method type arguments before the
                // argument list. The reading is taken ONLY when the bracket
                // pair closes with a `>` immediately followed by `(`, which is
                // what keeps `a < b` a comparison; the helper rewinds
                // otherwise, so reaching here means the `(` is already next.
                self.expect("(")?;
                let (mut arguments, mut site) = self.call_arguments()?;
                if self.check("{") && !self.no_trailing_block {
                    arguments.push(self.trailing_call_block(&mut site)?);
                }
                expression = Expression::Call {
                    callee: Box::new(expression),
                    type_arguments,
                    arguments,
                };
                self.record_call((mark, start, receiver), site);
            } else if self.consume("(") {
                let (mut arguments, mut site) = self.call_arguments()?;
                // `trailing_block ::= closure_literal` is a postfix part, so a
                // Closure written after the argument list is one more argument.
                // IRIS-V1-RUNTIME-C099 receives it as the `block` parameter.
                if self.check("{") && !self.no_trailing_block {
                    arguments.push(self.trailing_call_block(&mut site)?);
                }
                expression = Expression::Call {
                    callee: Box::new(expression),
                    type_arguments: Vec::new(),
                    arguments,
                };
                self.record_call((mark, start, receiver), site);
            } else {
                return Some(expression);
            }
        }
    }

    /// Parses `call_type_arguments`, or rewinds entirely.
    ///
    /// `IRIS-V1-GRAMMAR-C066` takes the type-argument reading ONLY when the
    /// bracket pair closes with a `>` IMMEDIATELY followed by `(`. Where both
    /// readings would be well formed the OPERATOR reading wins, so this
    /// speculates and restores diagnostics as well as the cursor.
    ///
    /// `call_type_argument ::= type_expr | "_"`; C072 keeps `_` forbidden in
    /// every persistent Type position, so it is admitted only here.
    fn call_type_arguments(&mut self) -> Option<Vec<iris_syntax::TypeExpression>> {
        let checkpoint = self.checkpoint();
        self.advance();
        self.skip_newlines();
        let mut arguments = Vec::new();
        loop {
            let argument = if self.check("_") {
                self.advance();
                iris_syntax::TypeExpression::Name("_".into())
            } else {
                match self.type_expression() {
                    Some(argument) => argument,
                    None => {
                        self.restore(checkpoint);
                        return None;
                    }
                }
            };
            arguments.push(argument);
            self.skip_newlines();
            if !self.consume(",") {
                break;
            }
            self.skip_newlines();
        }
        if self.expect_generic_close().is_none() {
            self.restore(checkpoint);
            return None;
        }
        if !self.check("(") {
            self.restore(checkpoint);
            return None;
        }
        Some(arguments)
    }

    /// Parses `generic_args` in expression position, or rewinds entirely.
    ///
    /// `IRIS-V1-GRAMMAR-C063` takes the generic reading only when the bracket
    /// pair closes with a `>` that is followed by a `postfix_part`, so
    /// `a < b` and `Box < C` keep the comparison meaning C020 gives them. Any
    /// other shape restores the cursor and yields `None`.
    fn closed_generic_arguments(&mut self) -> Option<Vec<iris_syntax::TypeExpression>> {
        let checkpoint = self.checkpoint();
        // Speculation must leave NO trace when it rewinds. The Type grammar
        // reports its own failures, so a rejected generic reading would
        // otherwise leave a diagnostic behind and turn the operator reading
        // C063 requires -- `A < B` -- into a parse error.
        self.advance();
        self.skip_newlines();
        let mut arguments = Vec::new();
        loop {
            let Some(argument) = self.type_expression() else {
                self.restore(checkpoint);
                return None;
            };
            arguments.push(argument);
            self.skip_newlines();
            if !self.consume(",") {
                break;
            }
            self.skip_newlines();
        }
        if self.expect_generic_close().is_none() {
            self.restore(checkpoint);
            return None;
        }
        // C067 admits a closed generic name as a COMPLETE expression, so the
        // closing `>` may be followed by a `postfix_part` OR by a token that
        // cannot continue an expression. Requiring a `postfix_part` left
        // `Box<String>` unusable as a value, which is what D-456 needs in order
        // to distinguish an interned Type object from the Class object.
        //
        // C020 is still not weakened: anything that COULD continue an
        // expression keeps the operator reading, so `a < b` stays a comparison.
        let complete = self.check(";")
            || self.check("\n")
            || self.check(",")
            || self.check("]")
            || self.check(")")
            || self.check("}")
            || self.at_end();
        if !self.check(".") && !self.check("(") && !self.check("..") && !complete {
            self.restore(checkpoint);
            return None;
        }
        Some(arguments)
    }

    /// Parses `reified_type_expr`, or rewinds entirely.
    ///
    /// `IRIS-V1-GRAMMAR-C065` takes the Type reading ONLY when the closing
    /// parenthesis is immediately followed by `.type`. The `&"." "type"` is a
    /// LOOKAHEAD rather than consumed input, so the caller still parses the
    /// `.type` member access itself. Where both readings would be well formed
    /// the OPERATOR reading wins, which is why this speculates and restores
    /// diagnostics as well as the cursor.
    fn reified_type_expression(&mut self) -> Option<iris_syntax::TypeExpression> {
        let checkpoint = self.checkpoint();
        self.advance();
        self.skip_newlines();
        let Some(annotation) = self.type_expression() else {
            self.restore(checkpoint);
            return None;
        };
        if !self.consume_after_newlines(")") {
            self.restore(checkpoint);
            return None;
        }
        if !(self.check(".") && self.peek_next() == Some("type")) {
            self.restore(checkpoint);
            return None;
        }
        Some(annotation)
    }

    fn primary(&mut self) -> Option<Expression> {
        if self.check("{") && !self.no_trailing_block {
            return self.closure_literal();
        }
        let frame = self.recorder.begin(self.current_offset());
        let result = self.primary_contents();
        let kind = result
            .as_ref()
            .filter(|_| self.recorder.enabled)
            .map(|value| SourceKind::Expression(self.primary_fact(frame.id, value)));
        self.recorder.finish(frame, self.consumed_end, kind);
        result
    }

    fn primary_contents(&mut self) -> Option<Expression> {
        if self.consume("@@") {
            return self.name().map(Expression::ClassVar);
        }
        if self.consume("$") {
            return self.name().map(Expression::GlobalVar);
        }
        if self.consume("@") {
            // IRIS-V1-RUNTIME-C161 makes a stored property named `name` create
            // the slot `@name`, and a declared property publishes under that
            // exact spelling. Dropping the sigil here gave a raw `@x = 1` the
            // DIFFERENT slot `x`, so one ivar had two slots and reflection
            // could not see what source had written.
            return self
                .name()
                .map(|name| Expression::RawIvar(format!("@{name}")));
        }
        if self.check("(") {
            // C065 reifies a parenthesized Type expression, but ONLY when the
            // closing parenthesis is immediately followed by `.type`. That
            // lookahead is what keeps `(a | b)` a bitwise or, so the Type
            // reading is speculated first and rewound when it does not apply.
            if let Some(annotation) = self.reified_type_expression() {
                return Some(Expression::ReifiedType(annotation));
            }
            self.advance();
            return self.with_layout(true, Self::grouped_or_tuple);
        }
        if self.consume("[") {
            return self.array();
        }
        if self.consume("%{") {
            return self.with_layout(true, Self::hash_literal);
        }
        self.primary_value()
    }

    fn grouped_or_tuple(&mut self) -> Option<Expression> {
        self.skip_newlines();
        // C021 spells the Tuple forms `()`, `(a,)` and `(a, b, ...)`, so
        // the empty and trailing-comma forms are what separate a one-element
        // Tuple from an ordinary grouped expression.
        if self.consume(")") {
            return Some(Expression::Tuple(Vec::new()));
        }
        let value = self.expression(0)?;
        if self.consume(",") {
            self.skip_newlines();
            let mut elements = vec![value];
            while !self.check(")") && !self.at_end() {
                elements.push(self.expression(0)?);
                if !self.consume(",") {
                    break;
                }
                self.skip_newlines();
            }
            self.expect(")")?;
            return Some(Expression::Tuple(elements));
        }
        self.expect(")")?;
        Some(Expression::Grouped(Box::new(value)))
    }

    fn hash_literal(&mut self) -> Option<Expression> {
        self.skip_newlines();
        // `hash_literal ::= "%{" hash_entry_list? "}"` with
        // `hash_entry ::= expression ":" expression`.
        let mut entries = Vec::new();
        while !self.check("}") && !self.at_end() {
            let key = self.expression(0)?;
            self.expect(":")?;
            let value = self.expression(0)?;
            entries.push((key, value));
            if !self.consume(",") {
                break;
            }
            self.skip_newlines();
        }
        self.expect("}")?;
        Some(Expression::Hash(entries))
    }

    fn primary_value(&mut self) -> Option<Expression> {
        if self.editor && self.call_argument_recovery && matches!(self.peek(), Some(")" | "}" | "]")) {
            self.error("PARSE_UNEXPECTED_TOKEN");
            return None;
        }
        if self.check("{") && !self.no_trailing_block {
            return self.closure_literal();
        }
        if self.consume(":") {
            if self.consume("@") {
                return self
                    .name()
                    .map(|name| Expression::Symbol(format!("@{name}")));
            }
            if let Some(quoted) = self.quoted_symbol() {
                return Some(Expression::Symbol(quoted));
            }
            return self.selector().map(Expression::Symbol);
        }
        if self.consume("if") {
            return self.if_expression();
        }
        if self.consume("while") {
            let outer = std::mem::replace(&mut self.no_trailing_block, true);
            let condition = self.expression(0);
            self.no_trailing_block = outer;
            return Some(Expression::While {
                label: None,
                condition: Box::new(condition?),
                body: self.body()?,
            });
        }
        if self.consume("try") {
            let (body, catches, finally) = self.try_parts()?;
            return Some(Expression::Try {
                body,
                catches,
                finally,
            });
        }
        if self.is_name() {
            let name = self.qualified_name()?;
            // C063 admits a CLOSED generic construction in expression position.
            // C020 is not weakened, so the generic reading is taken only when
            // the name is a Type name AND the bracket pair closes with a `>`
            // followed by a postfix part. Where both readings would be well
            // formed the OPERATOR reading wins, which is why this speculates
            // and rewinds instead of committing on the opening `<`.
            if self.check("<")
                && name
                    .rsplit("::")
                    .next()
                    .and_then(|segment| segment.chars().next())
                    .is_some_and(char::is_uppercase)
                && let Some(arguments) = self.closed_generic_arguments()
            {
                return Some(Expression::ClosedGeneric { name, arguments });
            }
            return Some(Expression::Name(name));
        }
        let Some(value) = self.advance().map(|token| token.text) else {
            self.error("PARSE_UNEXPECTED_TOKEN");
            return None;
        };
        if self.is_literal(&value) {
            // `IRIS-V1-COLLECTIONS-C051` concatenates ADJACENT String literal
            // segments into one expression. Only String literals join: a
            // Symbol, Bytes or arbitrary expression beside one does not.
            if value.starts_with('"') || value.starts_with('\'') {
                let mut joined = value;
                while let Some(next) = self.peek()
                    && (next.starts_with('"') || next.starts_with('\''))
                    && self.is_literal(next)
                {
                    let Some(segment) = self.advance().map(|token| token.text) else {
                        break;
                    };
                    joined = crate::join_string_literals(&joined, &segment)?;
                }
                return Some(Expression::Literal(joined));
            }
            return Some(Expression::Literal(value));
        }
        // Only an ordinary identifier may become a Name here. Accepting any
        // leftover token silently turned a rejected operator into a bare name,
        // so `5 % 2` parsed as three statements instead of being rejected by
        // the grammar as `IRIS-V1-RUNTIME-V102` requires.
        if value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        {
            return Some(Expression::Name(value));
        }
        self.error("PARSE_UNEXPECTED_TOKEN");
        None
    }

    pub(super) fn if_expression(&mut self) -> Option<Expression> {
        self.nested(Self::if_expression_contents)
    }

    fn if_expression_contents(&mut self) -> Option<Expression> {
        let outer = std::mem::replace(&mut self.no_trailing_block, true);
        let condition = self.expression(0);
        self.no_trailing_block = outer;
        let condition = condition?;
        let then_body = self.body()?;
        let else_body = if self.consume_after_newlines("else") {
            self.skip_newlines();
            if self.consume("if") {
                Some(vec![Statement::Expression(self.if_expression()?)])
            } else {
                Some(self.body()?)
            }
        } else {
            None
        };
        Some(Expression::If {
            condition: Box::new(condition),
            then_body,
            else_body,
        })
    }

    fn array(&mut self) -> Option<Expression> {
        let values = self.delimited_expressions("]")?;
        Some(Expression::Array(values))
    }

    pub(super) fn arguments(&mut self) -> Option<Vec<Expression>> {
        self.delimited_expressions(")")
    }

    /// Parses one argument, which `IRIS-V1-CONTROL-C023` allows to be keyword.
    ///
    /// `name:` is distinguished from a Symbol literal by position: a Symbol
    /// writes the colon BEFORE the name, so an identifier followed by a colon
    /// is unambiguously a keyword argument.
    pub(super) fn argument(&mut self) -> Option<Expression> {
        if let Some(name) = self.peek_keyword_argument_name() {
            if is_reserved_keyword(&name) {
                self.error("PARSE_UNEXPECTED_TOKEN");
            }
            let start = self.current_offset();
            let mark = self.recorder.mark();
            self.advance();
            let site = NameSite {
                text: name.clone(),
                span: Span {
                    start,
                    end: self.consumed_end,
                },
            };
            self.advance();
            let value = self.expression(0)?;
            if let Some(value) = self.recorder.last() {
                self.recorder.wrap(
                    mark,
                    Span {
                        start,
                        end: self.consumed_end,
                    },
                    SourceKind::Expression(ExpressionFact::KeywordArgument { name: site, value }),
                );
            }
            return Some(Expression::KeywordArgument {
                name,
                value: Box::new(value),
            });
        }
        self.expression(0)
    }

    fn delimited_expressions(&mut self, closer: &str) -> Option<Vec<Expression>> {
        self.with_layout(true, |parser| parser.expression_list(closer))
    }

    fn expression_list(&mut self, closer: &str) -> Option<Vec<Expression>> {
        self.skip_newlines();
        let mut values = Vec::new();
        if self.consume(closer) {
            return Some(values);
        }
        loop {
            values.push(if closer == ")" {
                self.argument()?
            } else {
                self.expression(0)?
            });
            if self.consume(closer) {
                return Some(values);
            }
            self.expect(",")?;
            self.skip_newlines();
            if self.consume(closer) {
                return Some(values);
            }
        }
    }

    fn infix(&self) -> Option<(u8, Associativity)> {
        let operator = match self.peek()? {
            "**" => (14, Associativity::Right),
            "*" => (12, Associativity::Left),
            "/" => (12, Associativity::Left),
            "+" => (11, Associativity::Left),
            "-" => (11, Associativity::Left),
            "<<" => (10, Associativity::Left),
            ">>" => (10, Associativity::Left),
            "&" => (9, Associativity::Left),
            "^" => (8, Associativity::Left),
            "|" => (7, Associativity::Left),
            "..=" | "..<" => (6, Associativity::NonAssociative),
            // C016 lists `=~` and `!~` among the fixed spellings, and
            // `relational_expr` places them with the other relational
            // operators.
            "<" | "<=" | ">" | ">=" | "<=>" | "=~" | "!~" | "is" | "as" | "as?" => {
                (5, Associativity::NonAssociative)
            }
            "==" | "!=" => (4, Associativity::NonAssociative),
            "&&" => (2, Associativity::Left),
            "||" => (1, Associativity::Left),
            value if is_identifier(value) && !is_reserved_keyword(value) => {
                (3, Associativity::Left)
            }
            _ => return None,
        };
        Some(operator)
    }

    fn consume_infix_operator(&mut self) -> Option<BinaryOperator> {
        let start = self.current_offset();
        let token = self.advance()?.text;
        let operator = match token.as_str() {
            "**" => BinaryOperator::Power,
            "*" => BinaryOperator::Multiply,
            "/" => BinaryOperator::Divide,
            "+" => BinaryOperator::Add,
            "-" => BinaryOperator::Subtract,
            "<<" => BinaryOperator::ShiftLeft,
            ">>" => BinaryOperator::ShiftRight,
            "&" => BinaryOperator::BitwiseAnd,
            "^" => BinaryOperator::BitwiseXor,
            "|" => BinaryOperator::BitwiseOr,
            "..=" => BinaryOperator::RangeInclusive,
            "..<" => BinaryOperator::RangeExclusive,
            "<" => BinaryOperator::Less,
            "<=" => BinaryOperator::LessEqual,
            ">" => BinaryOperator::Greater,
            ">=" => BinaryOperator::GreaterEqual,
            "<=>" => BinaryOperator::Compare,
            "=~" => BinaryOperator::Match,
            "!~" => BinaryOperator::NotMatch,
            "is" => BinaryOperator::Is,
            "as" => BinaryOperator::As,
            "as?" => BinaryOperator::AsOptional,
            "==" => BinaryOperator::Equal,
            "!=" => BinaryOperator::NotEqual,
            "&&" => BinaryOperator::LogicalAnd,
            "||" => BinaryOperator::LogicalOr,
            selector if is_identifier(selector) && !is_reserved_keyword(selector) => {
                let selector = self.selector_suffix(token)?;
                self.recorder.name(NameSite {
                    text: selector.clone(),
                    span: Span {
                        start,
                        end: self.consumed_end,
                    },
                });
                if selector == "same?" {
                    BinaryOperator::Identity
                } else {
                    BinaryOperator::NamedInfix { selector }
                }
            }
            _ => return None,
        };
        Some(operator)
    }
}

/// Renders a Type expression back to the single name the evaluator resolves.
///
/// `is`, `as` and `as?` take a Type on the right, but the evaluator resolves it
/// through the ordinary name table, so a closed generic such as `Dynamic<A>`
/// contributes its constructor name and its argument is checked separately.
fn type_expression_name(value: &iris_syntax::TypeExpression) -> Option<String> {
    match value {
        iris_syntax::TypeExpression::Name(name) => Some(name.clone()),
        iris_syntax::TypeExpression::Generic { name, .. } => Some(name.clone()),
        _ => None,
    }
}
