use super::{Analyzer, Control};
use iris_syntax::{Declaration, ExportDeclaration, MatchBody, Statement};

impl Analyzer {
    pub(super) fn prepare_declaration(&mut self, declaration: &mut Declaration) {
        let body = match declaration {
            Declaration::Class(value) => &mut value.body,
            Declaration::Module(value) => &mut value.body,
            Declaration::Contract(value) => &mut value.body,
            Declaration::Export(value) => {
                if let ExportDeclaration::Declaration(inner) = value.as_mut() {
                    self.prepare_declaration(inner);
                }
                return;
            }
            Declaration::Import(_) | Declaration::TypeAlias(_) => return,
        };
        self.prepare_body(body);
    }

    pub(super) fn prepare_body(&mut self, body: &mut [Statement]) {
        self.scopes.push(Vec::new());
        for statement in body {
            self.prepare_statement(statement);
        }
        self.scopes.pop();
    }

    pub(super) fn prepare_statement(&mut self, statement: &mut Statement) {
        if matches!(statement, Statement::Method(_)) {
            self.statement(statement, Control::top_level());
        }
        match statement {
            Statement::Binding { value, .. } => {
                self.prepare_expression(value);
                self.statement(statement, Control::top_level());
                if let Statement::Binding {
                    name, annotation, ..
                } = statement
                    && annotation.is_none()
                {
                    *annotation = self.lookup(name).and_then(|local| local.contract.clone());
                }
            }
            Statement::Method(method) => {
                let outer_scopes = std::mem::replace(&mut self.scopes, vec![Vec::new()]);
                for parameter in &mut method.parameters {
                    if let Some(default) = &mut parameter.default {
                        self.prepare_expression(default);
                    }
                    self.declare_parameter(parameter);
                }
                if let Some(body) = &mut method.body {
                    for statement in body {
                        self.prepare_statement(statement);
                    }
                }
                self.scopes = outer_scopes;
            }
            Statement::Expression(value)
            | Statement::GlobalBinding { value, .. }
            | Statement::SharedBinding { value, .. }
            | Statement::StoredProperty {
                initializer: value, ..
            } => self.prepare_expression(value),
            Statement::If {
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
            Statement::While {
                condition, body, ..
            } => {
                self.prepare_expression(condition);
                self.prepare_body(body);
            }
            Statement::For {
                binding,
                iterable,
                body,
                ..
            } => {
                self.prepare_expression(iterable);
                self.scopes.push(Vec::new());
                for name in super::super::pattern_names(binding) {
                    self.declare(&name, false);
                }
                for statement in body {
                    self.prepare_statement(statement);
                }
                self.scopes.pop();
            }
            Statement::Return(value) | Statement::Break { value, .. } => {
                if let Some(value) = value {
                    self.prepare_expression(value);
                }
            }
            Statement::Raise(raise) => {
                if let Some(raise) = raise {
                    self.prepare_expression(&mut raise.value);
                    if let Some(cause) = &mut raise.cause {
                        self.prepare_expression(cause);
                    }
                }
            }
            Statement::Try {
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
            Statement::Match {
                subject,
                arms,
                fallback,
            } => {
                self.prepare_expression(subject);
                for arm in arms {
                    self.scopes.push(Vec::new());
                    for name in super::super::pattern_names(&arm.pattern) {
                        self.declare(&name, false);
                    }
                    if let Some(guard) = &mut arm.guard {
                        self.prepare_expression(guard);
                    }
                    self.prepare_match_body(&mut arm.body);
                    self.scopes.pop();
                }
                if let Some(body) = fallback {
                    self.prepare_match_body(body);
                }
            }
            Statement::DeferredBinding { name, mutable, .. } => self.declare(name, *mutable),
            Statement::Continue(_) => {}
        }
    }

    fn prepare_match_body(&mut self, body: &mut MatchBody) {
        match body {
            MatchBody::Expression(value) => self.prepare_expression(value),
            MatchBody::Block(body) => self.prepare_body(body),
        }
    }
}
