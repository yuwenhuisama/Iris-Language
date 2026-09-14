use super::decorator_phases::PhaseTarget;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::Value;
use iris_runtime::decorator_protocol::{DecoratorKind, DecoratorReason};
use iris_syntax::{Declaration, Expression, Program, ProgramEntry, Statement};
use std::collections::HashMap;

pub(super) fn impure() -> EvaluationError {
    EvaluationError::DecoratorDiagnostic {
        code: "IRIS-DECORATOR-NONDETERMINISTIC",
        phase: "static",
    }
}

impl SourceEvaluator {
    pub(super) fn plan_decorators(&mut self, program: &Program) -> Result<(), EvaluationError> {
        let entries = program
            .entries
            .iter()
            .filter_map(|entry| {
                let ProgramEntry::Declaration(declaration) = entry else {
                    return None;
                };
                let mut declaration = declaration;
                while let Declaration::Export(export) = declaration {
                    let iris_syntax::ExportDeclaration::Declaration(inner) = export.as_ref() else {
                        return None;
                    };
                    declaration = inner;
                }
                Some(declaration)
            })
            .collect::<Vec<_>>();
        let declarations = entries
            .iter()
            .filter_map(|entry| match entry {
                Declaration::Class(class) => Some(class.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let open_declarations = super::candidate_planning::collect(program);
        let modules = entries
            .iter()
            .filter_map(|entry| match entry {
                Declaration::Module(module) => Some(module),
                _ => None,
            })
            .collect::<Vec<_>>();
        let contracts = entries
            .iter()
            .filter_map(|entry| match entry {
                Declaration::Contract(contract) => Some(contract),
                _ => None,
            })
            .collect::<Vec<_>>();
        let has_applications = contracts.iter().any(|contract| !contract.decorators.is_empty()) || modules.iter().any(|module| !module.decorators.is_empty() || module.body.iter().any(|statement| matches!(statement, Statement::Method(method) if !method.decorators.is_empty()))) || !open_declarations.is_empty() || declarations.iter().any(|class| !class.decorators.is_empty()
            || class.body.iter().any(|statement| match statement {
                Statement::Method(method) => !method.decorators.is_empty(),
                Statement::StoredProperty { decorators, .. } => !decorators.is_empty(),
                _ => false,
            }));
        if !has_applications {
            return Ok(());
        }
        let mut planner = Self::new_in_package(&self.package)?;
        planner.decorator_planning = true;
        planner.source = self.source.clone();
        planner.static_implementations = self.static_implementations.clone();
        planner.inherit_planning_contracts(self);
        for contract in &contracts {
            planner.contract(contract)?;
        }
        for module in &modules {
            let mut stub = (*module).clone();
            stub.decorators.clear();
            stub.body.clear();
            planner.module(&stub)?;
        }
        for declaration in self.decorator_definitions.iter().chain(&declarations) {
            let mut definition = declaration.clone();
            definition.decorators.clear();
            definition.meta_deny.clear();
            definition.body.retain(|statement| {
                matches!(
                    statement,
                    Statement::Method(_) | Statement::StoredProperty { .. }
                )
            });
            for statement in &mut definition.body {
                match statement {
                    Statement::Method(method) => method.decorators.clear(),
                    Statement::StoredProperty { decorators, .. } => decorators.clear(),
                    _ => {}
                }
            }
            planner.class(&definition)?;
        }
        for declaration in &declarations {
            let class = planner
                .class_name(&declaration.name)?
                .ok_or(EvaluationError::NameError)?;
            let reason = if declaration.reopen {
                DecoratorReason::Open
            } else {
                DecoratorReason::Origin
            };
            for decorator in &declaration.decorators {
                let metadata = planner.class_decorator_metadata(class)?;
                planner.execute_decorator_phase(
                    decorator,
                    PhaseTarget {
                        kind: DecoratorKind::Class,
                        reason,
                        metadata,
                        candidate: None,
                    },
                )?;
            }
            planner.plan_class_members(class, (&declaration.body, reason))?;
        }
        planner.plan_open_declarations(&open_declarations)?;
        for module in modules {
            planner.module_decorator_phases(module)?;
        }
        for declaration in contracts {
            let contract = planner.contract_names[&declaration.name];
            let metadata = planner.contract_metadata[&contract].clone();
            let binding = planner.names.remove(&declaration.name);
            planner.contract_decorator_phases(declaration, &metadata)?;
            if let Some(binding) = binding {
                planner.names.insert(declaration.name.clone(), binding);
            }
        }
        Ok(())
    }

    pub(super) fn check_planning_expression(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<&Value>,
    ) -> Result<(), EvaluationError> {
        if !self.decorator_planning {
            return Ok(());
        }
        match expression {
            Expression::GlobalVar(_)
            | Expression::ClassVar(_)
            | Expression::Await(_)
            | Expression::Yield(_)
            | Expression::ClosedGeneric { .. } => return Err(impure()),
            Expression::Closure { is_async: true, .. } => return Err(impure()),
            Expression::RawIvar(_) if !matches!(receiver, Some(Value::Object(_))) => {
                return Err(impure());
            }
            Expression::Name(name) if name != "self" && !locals.contains_key(name) => {
                if !self.names.contains_key(name) && super::builtin(name, &self.kernel).is_none() {
                    return Err(impure());
                }
            }
            Expression::Call { callee, .. } => {
                if let Expression::Member {
                    receiver: target, ..
                } = callee.as_ref()
                    && let Expression::Name(name) = target.as_ref()
                    && name != "self"
                    && name != "super"
                    && !locals.contains_key(name)
                    && !self.names.contains_key(name)
                    && super::builtin(name, &self.kernel).is_none()
                {
                    return Err(impure());
                }
                if let Expression::Name(name) = callee.as_ref() {
                    let user_method = receiver
                        .and_then(|value| self.receiver_bound_method(name, Some(value)))
                        .is_some();
                    if !locals.contains_key(name) && !user_method {
                        return Err(impure());
                    }
                }
            }
            Expression::Assignment { left, .. } => match left.as_ref() {
                Expression::RawIvar(_) if matches!(receiver, Some(Value::Object(_))) => {}
                Expression::Name(name)
                    if self.names.get(name).is_some_and(|binding| binding.mutable) => {}
                _ => return Err(impure()),
            },
            _ => {}
        }
        Ok(())
    }

    pub(super) fn check_planning_send(
        &self,
        receiver: &Value,
        selector: &str,
    ) -> Result<(), EvaluationError> {
        if !self.decorator_planning {
            return Ok(());
        }
        let allowed = match receiver {
            Value::Object(_) => true,
            Value::Class(class) => {
                matches!(selector, "new" | "type")
                    || super::decorator_core::RECORDS
                        .iter()
                        .any(|name| self.kernel.core_class(name) == Some(*class))
            }
            Value::Decorator(_) => true,
            Value::Type(_, _) | Value::Contract(_, _) => matches!(selector, "==" | "!="),
            Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Bool(_)
            | Value::Nil
            | Value::Symbol(_)
            | Value::Text(_) => matches!(
                selector,
                "==" | "!="
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "<"
                    | ">"
                    | "<="
                    | ">="
                    | "to_bool"
                    | "to_string"
                    | "hash"
                    | "length"
                    | "size"
                    | "class"
                    | "type"
            ),
            Value::ReadonlyArray(_)
            | Value::ImmutableArray(_)
            | Value::ImmutableHash(_)
            | Value::Tuple(_)
            | Value::Array(_)
            | Value::Hash(_) => matches!(selector, "[]" | "length" | "size" | "count" | "empty?"),
            Value::Closure(_) => selector == "call",
            _ => false,
        };
        if allowed { Ok(()) } else { Err(impure()) }
    }
}
