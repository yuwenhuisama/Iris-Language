use super::decorated_module::module_error;
use super::decorator_phases::{PhaseTarget, kind_error};
use super::wrapper_chain::WrapperChain;
use super::{EvaluationError, SourceEvaluator};
use iris_runtime::decorator_protocol::{DecoratorKind, DecoratorReason, DecoratorValue, Operation};
use iris_runtime::{ModuleId, ModuleMethodDefinition, Value, Visibility};
use iris_syntax::{MethodKind, ModuleDeclaration, Statement};
use std::rc::Rc;

impl SourceEvaluator {
    pub(super) fn module_decorator_phases(
        &mut self,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        let module = *self
            .module_names
            .get(&declaration.name)
            .ok_or(EvaluationError::NameError)?;
        self.transform_module_phases(module, declaration)
    }

    pub(super) fn transform_module_phases(
        &mut self,
        module: ModuleId,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        let reason = if self.upgrade_state.active {
            DecoratorReason::Upgrade
        } else if self.rollback_state.context.is_some() {
            DecoratorReason::Rollback
        } else if declaration.reopen {
            DecoratorReason::Open
        } else {
            DecoratorReason::Origin
        };
        let owner = Value::Symbol(declaration.name.clone());
        for decorator in &declaration.decorators {
            let mut methods = Vec::new();
            for statement in &declaration.body {
                if let Statement::Method(method) = statement
                    && method.visibility == iris_syntax::Visibility::Public
                {
                    methods.push(self.owned_method_metadata(owner.clone(), method)?);
                }
            }
            let metadata = self.metadata_record(vec![
                ("name", owner.clone()),
                ("type", owner.clone()),
                ("package", Value::Text(self.package.clone())),
                ("methods", Value::ReadonlyArray(methods)),
            ])?;
            let result = self.execute_decorator_phase(
                decorator,
                PhaseTarget {
                    kind: DecoratorKind::Module,
                    reason,
                    metadata,
                    candidate: None,
                },
            )?;
            if self.decorator_planning {
                continue;
            }
            let Value::Decorator(record) = result else {
                return Err(self.wrapper_type_error());
            };
            let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                return Err(self.wrapper_type_error());
            };
            for operation in transformation.operations() {
                match operation {
                    Operation::AddMethod {
                        selector,
                        body: Value::Closure(closure),
                    } => {
                        let method = self.captured_declaration(selector, *closure)?;
                        self.wrapper_signature(&method)?;
                        let body = self.register_body(method);
                        let selector = self.selector(selector);
                        self.runtime
                            .registry_mut()
                            .stage_module_method(
                                module,
                                ModuleMethodDefinition::new(selector, body, Visibility::Private),
                            )
                            .map_err(module_error)?;
                        self.captured_methods.insert(body.raw(), *closure);
                    }
                    Operation::AddMethod { .. } => return Err(self.wrapper_type_error()),
                    Operation::WrapMethod(_)
                    | Operation::WrapGetter(_)
                    | Operation::WrapSetter(_) => return Err(kind_error(false)),
                }
            }
        }
        for statement in &declaration.body {
            let Statement::Method(method) = statement else {
                continue;
            };
            let mut wrappers = Vec::new();
            for decorator in &method.decorators {
                let metadata = self.owned_method_metadata(owner.clone(), method)?;
                let result = self.execute_decorator_phase(
                    decorator,
                    PhaseTarget {
                        kind: DecoratorKind::Method,
                        reason,
                        metadata,
                        candidate: None,
                    },
                )?;
                if self.decorator_planning {
                    continue;
                }
                let Value::Decorator(record) = result else {
                    return Err(self.wrapper_type_error());
                };
                let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                    return Err(self.wrapper_type_error());
                };
                for operation in transformation.operations() {
                    match operation {
                        Operation::WrapMethod(wrapper) => {
                            wrappers.push(self.admit_wrapper(wrapper, method.is_async)?)
                        }
                        Operation::AddMethod { .. }
                        | Operation::WrapGetter(_)
                        | Operation::WrapSetter(_) => return Err(kind_error(false)),
                    }
                }
            }
            if !wrappers.is_empty() {
                if method
                    .body
                    .as_ref()
                    .is_some_and(|body| super::body_yields(body))
                {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
                self.wrapper_signature(method)?;
                let selector = self.selector(&method.selector);
                let (original, published) = if method.kind == MethodKind::Class {
                    let backing = self.module_classes[&module];
                    self.runtime
                        .registry()
                        .require_module_candidate_capability(
                            module,
                            iris_runtime::Capability::MethodBody,
                        )
                        .map_err(module_error)?;
                    let original = self.singleton_identities[&(backing, selector)];
                    let published = self
                        .runtime
                        .registry_mut()
                        .publish_singleton_method(
                            backing,
                            selector,
                            original.body(),
                            original.visibility(),
                        )
                        .map_err(EvaluationError::Class)?;
                    self.singleton_identities
                        .insert((backing, selector), published);
                    (original, published)
                } else {
                    let original = self
                        .runtime
                        .registry()
                        .staged_module(module)
                        .map_err(module_error)?
                        .method(selector)
                        .ok_or(EvaluationError::UnsupportedConstruct)?;
                    let published = self
                        .runtime
                        .registry_mut()
                        .stage_module_method_body(module, selector, original.body())
                        .map_err(module_error)?;
                    (original, published)
                };
                self.wrapper_chains.insert(
                    published.id(),
                    Rc::new(WrapperChain {
                        original,
                        declaration: method.clone(),
                        wrappers,
                        qualifier: None,
                        canonical: None,
                    }),
                );
            }
        }
        Ok(())
    }
}
