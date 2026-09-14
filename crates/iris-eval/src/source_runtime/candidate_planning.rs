use super::candidate_declarations::stored_decorator_declaration;
use super::decorator_phases::{PhaseTarget, method_kind};
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::DecoratorReason;
use iris_syntax::{Declaration, Expression, MethodDeclaration, Program, ProgramEntry, Statement};

pub(super) struct OpenDeclaration {
    target: String,
    method: MethodDeclaration,
}

pub(super) fn collect(program: &Program) -> Vec<OpenDeclaration> {
    let mut declarations = Vec::new();
    for entry in &program.entries {
        match entry {
            ProgramEntry::Statement(statement) => {
                statements(std::slice::from_ref(statement), &mut declarations)
            }
            ProgramEntry::Declaration(Declaration::Class(class)) => {
                statements(&class.body, &mut declarations)
            }
            ProgramEntry::Declaration(Declaration::Module(module)) => {
                statements(&module.body, &mut declarations);
            }
            _ => {}
        }
    }
    declarations
}

impl SourceEvaluator {
    pub(super) fn plan_open_declarations(
        &mut self,
        declarations: &[OpenDeclaration],
    ) -> Result<(), EvaluationError> {
        for declaration in declarations {
            let class = self
                .class_name(&declaration.target)?
                .ok_or(EvaluationError::NameError)?;
            for decorator in &declaration.method.decorators {
                let metadata = self.method_decorator_metadata(class, &declaration.method)?;
                self.execute_decorator_phase(
                    decorator,
                    PhaseTarget {
                        kind: method_kind(&declaration.method),
                        reason: DecoratorReason::Open,
                        metadata,
                        candidate: None,
                    },
                )?;
            }
        }
        Ok(())
    }
}

fn statements(body: &[Statement], declarations: &mut Vec<OpenDeclaration>) {
    for statement in body {
        match statement {
            Statement::Expression(value)
            | Statement::InstanceField { value, .. }
            | Statement::Binding { value, .. }
            | Statement::GlobalBinding { value, .. }
            | Statement::SharedBinding { value, .. }
            | Statement::StoredProperty {
                initializer: value, ..
            } => expression(value, declarations),
            Statement::Method(method) => {
                if let Some(body) = &method.body {
                    statements(body, declarations);
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                expression(condition, declarations);
                statements(then_body, declarations);
                if let Some(body) = else_body {
                    statements(body, declarations);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                expression(condition, declarations);
                statements(body, declarations);
            }
            Statement::For { iterable, body, .. } => {
                expression(iterable, declarations);
                statements(body, declarations);
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => {
                statements(body, declarations);
                for catch in catches {
                    statements(&catch.body, declarations);
                }
                if let Some(body) = finally {
                    statements(body, declarations);
                }
            }
            Statement::Return(value) | Statement::Break { value, .. } => {
                if let Some(value) = value {
                    expression(value, declarations);
                }
            }
            Statement::Raise(Some(raise)) => {
                expression(&raise.value, declarations);
                if let Some(cause) = &raise.cause {
                    expression(cause, declarations);
                }
            }
            _ => {}
        }
    }
}

fn expression(value: &Expression, declarations: &mut Vec<OpenDeclaration>) {
    match value {
        Expression::Call {
            callee, arguments, ..
        } => {
            if let Expression::Member { receiver, selector } = callee.as_ref()
                && selector == "open"
                && let Expression::Name(target) = receiver.as_ref()
                && let [callback] = arguments.as_slice()
                && let Expression::Closure { body, .. } = super::call_channels::operand(callback)
            {
                for statement in body {
                    let method = match statement {
                        Statement::Method(method) => Some(method.clone()),
                        statement => stored_decorator_declaration(statement),
                    };
                    if let Some(method) = method
                        && !method.decorators.is_empty()
                    {
                        declarations.push(OpenDeclaration {
                            target: target.clone(),
                            method,
                        });
                    }
                }
            }
            expression(callee, declarations);
            for argument in arguments {
                expression(argument, declarations);
            }
        }
        Expression::Closure { body, .. } => statements(body, declarations),
        Expression::Try {
            body,
            catches,
            finally,
        } => {
            statements(body, declarations);
            for catch in catches {
                statements(&catch.body, declarations);
            }
            if let Some(body) = finally {
                statements(body, declarations);
            }
        }
        Expression::If {
            condition,
            then_body,
            else_body,
        } => {
            expression(condition, declarations);
            statements(then_body, declarations);
            if let Some(body) = else_body {
                statements(body, declarations);
            }
        }
        Expression::While {
            condition, body, ..
        } => {
            expression(condition, declarations);
            statements(body, declarations);
        }
        Expression::Grouped(value)
        | Expression::NonNull(value)
        | Expression::Await(value)
        | Expression::Unary { operand: value, .. }
        | Expression::KeywordArgument { value, .. }
        | Expression::BlockArgument { value }
        | Expression::Member {
            receiver: value, ..
        }
        | Expression::ContractView {
            receiver: value, ..
        } => expression(value, declarations),
        Expression::Binary { left, right, .. }
        | Expression::Assignment { left, right, .. }
        | Expression::Index {
            receiver: left,
            index: right,
        } => {
            expression(left, declarations);
            expression(right, declarations);
        }
        Expression::Array(values) | Expression::Tuple(values) => {
            for value in values {
                expression(value, declarations);
            }
        }
        Expression::Hash(entries) => {
            for (key, value) in entries {
                expression(key, declarations);
                expression(value, declarations);
            }
        }
        Expression::Yield(Some(value)) => expression(value, declarations),
        _ => {}
    }
}
