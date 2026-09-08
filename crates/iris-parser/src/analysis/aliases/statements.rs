use super::Aliases;
use iris_syntax::{CatchClause, MatchBody, Statement};

impl Aliases<'_> {
    pub(super) fn body(&mut self, body: &mut [Statement]) {
        for statement in body {
            self.statement(statement);
        }
    }

    pub(super) fn statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::Binding {
                annotation, value, ..
            }
            | Statement::GlobalBinding {
                annotation, value, ..
            }
            | Statement::SharedBinding {
                annotation, value, ..
            } => {
                self.optional(annotation);
                self.expression(value);
            }
            Statement::StoredProperty {
                decorators,
                annotation,
                initializer,
                ..
            } => {
                self.decorators(decorators);
                self.annotation(annotation);
                self.expression(initializer);
            }
            Statement::Method(method) => {
                let scope = self.shadowed.len();
                self.shadowed.extend(method.type_parameters.iter().cloned());
                self.decorators(&mut method.decorators);
                for parameter in &mut method.parameters {
                    self.optional(&mut parameter.annotation);
                    if let Some(default) = &mut parameter.default {
                        self.expression(default);
                    }
                }
                self.optional(&mut method.return_type);
                if let Some(body) = &mut method.body {
                    self.body(body);
                }
                self.shadowed.truncate(scope);
            }
            Statement::Expression(value) => self.expression(value),
            Statement::If {
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
            Statement::While {
                condition, body, ..
            } => {
                self.expression(condition);
                self.body(body);
            }
            Statement::For { iterable, body, .. } => {
                self.expression(iterable);
                self.body(body);
            }
            Statement::Return(value) | Statement::Break { value, .. } => {
                if let Some(value) = value {
                    self.expression(value);
                }
            }
            Statement::Raise(raise) => {
                if let Some(raise) = raise {
                    self.expression(&mut raise.value);
                    if let Some(cause) = &mut raise.cause {
                        self.expression(cause);
                    }
                }
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_body(body, catches, finally),
            Statement::Match {
                subject,
                arms,
                fallback,
            } => {
                self.expression(subject);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.expression(guard);
                    }
                    self.match_body(&mut arm.body);
                }
                if let Some(body) = fallback {
                    self.match_body(body);
                }
            }
            Statement::DeferredBinding { .. } | Statement::Continue(_) => {}
        }
    }

    pub(super) fn try_body(
        &mut self,
        body: &mut [Statement],
        catches: &mut [CatchClause],
        finally: &mut Option<Vec<Statement>>,
    ) {
        self.body(body);
        for catch in catches {
            self.optional(&mut catch.filter);
            self.body(&mut catch.body);
        }
        if let Some(body) = finally {
            self.body(body);
        }
    }

    fn match_body(&mut self, body: &mut MatchBody) {
        match body {
            MatchBody::Expression(value) => self.expression(value),
            MatchBody::Block(body) => self.body(body),
        }
    }
}
