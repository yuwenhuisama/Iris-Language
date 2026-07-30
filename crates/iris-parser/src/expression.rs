use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

use crate::{Associativity, Parser, is_identifier, is_reserved_keyword};

impl Parser {
    pub(super) fn expression(&mut self, minimum: u8) -> Option<Expression> {
        let mut left = self.prefix()?;
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
                Expression::Name(type_expression_name(&target)?)
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
        }
        if minimum == 0 && !(self.check("=") && self.peek_next() == Some(">")) {
            if let Some(operator) = self.assignment_operator() {
                self.advance();
                let right = self.expression(0)?;
                left = Expression::Assignment {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else if self.consume("%=") {
                self.error("PARSE_INVALID_ASSIGNMENT_OPERATOR");
                return None;
            }
        }
        Some(left)
    }

    fn prefix(&mut self) -> Option<Expression> {
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
        let mut expression = self.primary()?;
        loop {
            if self.consume(".") {
                expression = Expression::Member {
                    receiver: Box::new(expression),
                    selector: self.selector()?,
                };
            } else if self.consume("..") {
                expression = Expression::ContractView {
                    receiver: Box::new(expression),
                    selector: self.selector()?,
                };
            } else if self.check("[") && !self.newline_before_cursor() {
                // A `[` on the SAME logical line is an index, while one that
                // starts a line is an Array literal statement. Without this the
                // postfix form would swallow a following literal.
                self.advance();
                let index = self.expression(0)?;
                self.expect("]")?;
                expression = Expression::Index {
                    receiver: Box::new(expression),
                    index: Box::new(index),
                };
            } else if self.consume("(") {
                let mut arguments = self.arguments()?;
                // `trailing_block ::= closure_literal` is a postfix part, so a
                // Closure written after the argument list is one more argument.
                // IRIS-V1-RUNTIME-C099 receives it as the `block` parameter.
                if self.check("{") && !self.no_trailing_block {
                    arguments.push(self.closure_literal()?);
                }
                expression = Expression::Call {
                    callee: Box::new(expression),
                    arguments,
                };
            } else {
                return Some(expression);
            }
        }
    }

    fn primary(&mut self) -> Option<Expression> {
        if self.consume("@@") {
            return self.name().map(Expression::ClassVar);
        }
        if self.consume("@") {
            return self.name().map(Expression::RawIvar);
        }
        if self.consume("(") {
            let value = self.expression(0)?;
            self.expect(")")?;
            return Some(Expression::Grouped(Box::new(value)));
        }
        if self.consume("[") {
            return self.array();
        }
        if self.consume("%{") {
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
            }
            self.expect("}")?;
            return Some(Expression::Hash(entries));
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
        if self.consume("try") {
            let (body, catches, finally) = self.try_parts()?;
            return Some(Expression::Try {
                body,
                catches,
                finally,
            });
        }
        if self.is_name() {
            return self.qualified_name().map(Expression::Name);
        }
        let Some(value) = self.advance().map(|token| token.text) else {
            self.error("PARSE_UNEXPECTED_TOKEN");
            return None;
        };
        Some(if self.is_literal(&value) {
            Expression::Literal(value)
        } else {
            Expression::Name(value)
        })
    }

    pub(super) fn if_expression(&mut self) -> Option<Expression> {
        let outer = std::mem::replace(&mut self.no_trailing_block, true);
        let condition = self.expression(0);
        self.no_trailing_block = outer;
        let condition = condition?;
        let then_body = self.body()?;
        let else_body = if self.consume("else") {
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
    fn argument(&mut self) -> Option<Expression> {
        if let Some(name) = self.peek_keyword_argument_name() {
            self.advance();
            self.advance();
            let value = self.expression(0)?;
            return Some(Expression::KeywordArgument {
                name,
                value: Box::new(value),
            });
        }
        self.expression(0)
    }

    fn delimited_expressions(&mut self, closer: &str) -> Option<Vec<Expression>> {
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
            "<" | "<=" | ">" | ">=" | "<=>" | "is" | "as" | "as?" => {
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
            "is" => BinaryOperator::Is,
            "as" => BinaryOperator::As,
            "as?" => BinaryOperator::AsOptional,
            "==" => BinaryOperator::Equal,
            "!=" => BinaryOperator::NotEqual,
            "&&" => BinaryOperator::LogicalAnd,
            "||" => BinaryOperator::LogicalOr,
            selector if is_identifier(selector) && !is_reserved_keyword(selector) => {
                let selector = self.selector_suffix(token)?;
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
