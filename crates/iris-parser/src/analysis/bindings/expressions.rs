use super::Analyzer;
use iris_syntax::Expression;

impl Analyzer {
    pub(super) fn prepare_expression(&mut self, expression: &mut Expression) {
        match expression {
            Expression::Closure {
                parameters, body, ..
            } => {
                self.scopes.push(Vec::new());
                for name in parameters {
                    self.declare(name, false);
                }
                for statement in body {
                    self.prepare_statement(statement);
                }
                self.scopes.pop();
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                self.prepare_expression(callee);
                for argument in arguments {
                    self.prepare_expression(argument);
                }
            }
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                self.prepare_expression(condition);
                self.prepare_body(then_body);
                if let Some(body) = else_body {
                    self.prepare_body(body);
                }
            }
            Expression::While {
                condition, body, ..
            } => {
                self.prepare_expression(condition);
                self.prepare_body(body);
            }
            Expression::Try {
                body,
                catches,
                finally,
            } => {
                self.prepare_body(body);
                for catch in catches {
                    self.prepare_catch(catch);
                }
                if let Some(body) = finally {
                    self.prepare_body(body);
                }
            }
            Expression::Assignment { left, right, .. }
            | Expression::Binary { left, right, .. }
            | Expression::Index {
                receiver: left,
                index: right,
            } => {
                self.prepare_expression(left);
                self.prepare_expression(right);
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
            } => self.prepare_expression(value),
            Expression::Yield(value) => {
                if let Some(value) = value {
                    self.prepare_expression(value);
                }
            }
            Expression::Array(values) | Expression::Tuple(values) => {
                for value in values {
                    self.prepare_expression(value);
                }
            }
            Expression::Hash(entries) => {
                for (key, value) in entries {
                    self.prepare_expression(key);
                    self.prepare_expression(value);
                }
            }
            Expression::ReifiedType(_)
            | Expression::ClosedGeneric { .. }
            | Expression::Name(_)
            | Expression::Literal(_)
            | Expression::Symbol(_)
            | Expression::RawIvar(_)
            | Expression::ClassVar(_)
            | Expression::GlobalVar(_) => {}
        }
    }
}
