use super::Aliases;
use iris_syntax::Expression;

impl Aliases<'_> {
    pub(super) fn expression(&mut self, expression: &mut Expression) {
        match expression {
            Expression::Closure {
                return_type, body, ..
            } => {
                self.optional(return_type);
                self.body(body);
            }
            Expression::Call {
                callee,
                type_arguments,
                arguments,
            } => {
                self.expression(callee);
                self.types(type_arguments);
                for argument in arguments {
                    self.expression(argument);
                }
            }
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expression(condition);
                self.body(then_body);
                if let Some(body) = else_body {
                    self.body(body);
                }
            }
            Expression::While {
                condition, body, ..
            } => {
                self.expression(condition);
                self.body(body);
            }
            Expression::Try {
                body,
                catches,
                finally,
            } => self.try_body(body, catches, finally),
            Expression::Assignment { left, right, .. }
            | Expression::Binary { left, right, .. }
            | Expression::Index {
                receiver: left,
                index: right,
            } => {
                self.expression(left);
                self.expression(right);
            }
            Expression::Grouped(value)
            | Expression::Await(value)
            | Expression::Unary { operand: value, .. }
            | Expression::KeywordArgument { value, .. }
            | Expression::Member {
                receiver: value, ..
            }
            | Expression::ContractView {
                receiver: value, ..
            } => self.expression(value),
            Expression::Yield(value) => {
                if let Some(value) = value {
                    self.expression(value);
                }
            }
            Expression::Array(values) | Expression::Tuple(values) => {
                for value in values {
                    self.expression(value);
                }
            }
            Expression::Hash(entries) => {
                for (key, value) in entries {
                    self.expression(key);
                    self.expression(value);
                }
            }
            Expression::ReifiedType(value) => self.annotation(value),
            Expression::ClosedGeneric { arguments, .. } => self.types(arguments),
            Expression::Name(_)
            | Expression::Literal(_)
            | Expression::Symbol(_)
            | Expression::RawIvar(_)
            | Expression::ClassVar(_)
            | Expression::GlobalVar(_) => {}
        }
    }
}
