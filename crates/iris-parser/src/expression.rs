use iris_syntax::{BinaryOperator, Expression, UnaryOperator};

use crate::{Associativity, Parser, is_identifier};

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
            let right = self.expression(next)?;
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
            } else if self.consume("(") {
                expression = Expression::Call {
                    callee: Box::new(expression),
                    arguments: self.arguments()?,
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
        if self.consume(":") {
            return self.selector().map(Expression::Symbol);
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

    fn array(&mut self) -> Option<Expression> {
        let values = self.delimited_expressions("]")?;
        Some(Expression::Array(values))
    }

    pub(super) fn arguments(&mut self) -> Option<Vec<Expression>> {
        self.delimited_expressions(")")
    }

    fn delimited_expressions(&mut self, closer: &str) -> Option<Vec<Expression>> {
        let mut values = Vec::new();
        if self.consume(closer) {
            return Some(values);
        }
        loop {
            values.push(self.expression(0)?);
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
            "<" | "<=" | ">" | ">=" | "<=>" => (5, Associativity::NonAssociative),
            "==" | "!=" => (4, Associativity::NonAssociative),
            "&&" => (2, Associativity::Left),
            "||" => (1, Associativity::Left),
            value if is_identifier(value) => (3, Associativity::Left),
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
            "==" => BinaryOperator::Equal,
            "!=" => BinaryOperator::NotEqual,
            "&&" => BinaryOperator::LogicalAnd,
            "||" => BinaryOperator::LogicalOr,
            selector if is_identifier(selector) => {
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
