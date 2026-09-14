use super::decorated_module::module_error;
use super::{EvaluationError, SourceEvaluator, visibility};
use iris_runtime::{Capability, ClassId, ModuleId, ModuleMethodDefinition, StaticSpine, Value};
use iris_syntax::{MethodDeclaration, MethodKind, ModuleDeclaration, Statement};
use std::collections::HashMap;

impl SourceEvaluator {
    pub(super) fn stage_module_declaration(
        &mut self,
        module: ModuleId,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        let mut owners = Vec::new();
        for (index, existing) in [
            self.module_classes.get(&module),
            self.module_mains.get(&module),
        ]
        .into_iter()
        .enumerate()
        {
            let owner = match existing.copied() {
                Some(owner) => {
                    if index == 1 || declaration.body.iter().any(|statement| matches!(statement,
                        Statement::StoredProperty { class_level: true, .. } | Statement::SharedBinding { .. })
                        || matches!(statement, Statement::Method(method) if method.kind == MethodKind::Class))
                    {
                        self.runtime.registry_mut().begin_transaction(owner)
                            .map_err(EvaluationError::Class)?;
                    }
                    owner
                }
                None => self
                    .runtime
                    .registry_mut()
                    .stage_class_origin(StaticSpine::new(1), None, &[])
                    .map_err(EvaluationError::Class)?,
            };
            owners.push(owner);
        }
        let (backing, main) = (owners[0], owners[1]);
        self.module_names.insert(declaration.name.clone(), module);
        self.module_packages
            .insert(declaration.name.clone(), self.package.clone());
        self.module_classes.insert(module, backing);
        self.module_mains.insert(module, main);
        for statement in &declaration.body {
            if let Statement::Method(method) = statement {
                self.stage_module_declaration_method(module, (backing, main), method)?;
            }
        }
        self.transform_module_phases(module, declaration)?;
        let methods = self
            .selectors
            .values()
            .filter_map(|selector| {
                self.runtime
                    .registry()
                    .staged_module(module)
                    .ok()?
                    .method(*selector)
            })
            .collect::<Vec<_>>();
        for method in methods {
            self.module_methods
                .insert((module, method.selector()), method);
            if let Some(main_method) = self
                .runtime
                .registry()
                .staged_method(main, method.selector())
                && let Some(chain) = self.wrapper_chains.get(&method.id()).cloned()
            {
                self.wrapper_chains.insert(main_method, chain);
            }
        }
        for statement in &declaration.body {
            match statement {
                Statement::Method(_) => {}
                Statement::StoredProperty {
                    class_level: true,
                    name,
                    initializer,
                    shared,
                    ..
                } => {
                    if declaration.reopen {
                        let operation = if self.is_class_level_property(backing, name) {
                            Capability::PropertyBody
                        } else {
                            Capability::PropertySet
                        };
                        self.runtime
                            .registry()
                            .require_module_candidate_capability(module, operation)
                            .map_err(module_error)?;
                    }
                    self.record_class_property_accessors(backing, statement)?;
                    if *shared {
                        self.shared_class_properties
                            .entry(backing)
                            .or_default()
                            .push(name.clone());
                    }
                    let value =
                        self.expression(initializer, &HashMap::new(), Some(Value::Class(backing)))?;
                    let slot = self.selector(name);
                    self.runtime
                        .assign_candidate_class_raw_ivar(backing, slot, value)
                        .map_err(EvaluationError::Construction)?;
                    self.class_level_properties
                        .entry(backing)
                        .or_default()
                        .push(slot);
                }
                Statement::SharedBinding {
                    mutable,
                    name,
                    value,
                    ..
                } => {
                    self.shared_binding(backing, *mutable, name, value)?;
                }
                Statement::Expression(_) | Statement::Binding { .. } | Statement::Raise(_) => {}
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        self.module_candidate_body(module, main, declaration)
    }

    fn stage_module_declaration_method(
        &mut self,
        module: ModuleId,
        (backing, main): (ClassId, ClassId),
        method: &MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        if method.kind == MethodKind::Property || method.impl_contract.is_some() {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let selector = self.selector(&method.selector);
        let body = self.register_body(method.clone());
        if method.kind == MethodKind::Class {
            let original = self.singleton_declarations.get(&(backing, selector));
            if self.runtime.registry().active_module(module).is_ok() {
                let operation = if original.is_some() {
                    Capability::MethodBody
                } else {
                    Capability::MethodSet
                };
                self.runtime
                    .registry()
                    .require_module_candidate_capability(module, operation)
                    .map_err(module_error)?;
            }
            if let Some(promise) = original
                && (promise.visibility != method.visibility
                    || !iris_syntax::method_signature_compatible(
                        method,
                        promise,
                        |source, target| source == target,
                    ))
            {
                return Err(EvaluationError::TypeContractError);
            }
            let defined = self
                .runtime
                .registry_mut()
                .publish_singleton_method(backing, selector, body, visibility(method))
                .map_err(EvaluationError::Class)?;
            self.singleton_identities
                .insert((backing, selector), defined);
            self.singleton_declarations
                .insert((backing, selector), method.clone());
            return Ok(());
        }
        let original = self
            .runtime
            .registry()
            .staged_module(module)
            .map_err(module_error)?
            .method(selector);
        let defined = match original {
            Some(original) => {
                let promise = self
                    .bodies
                    .get(&original.body().raw())
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                if promise.kind != method.kind
                    || promise.visibility != method.visibility
                    || !iris_syntax::method_signature_compatible(
                        method,
                        promise,
                        |source, target| source == target,
                    )
                {
                    return Err(EvaluationError::TypeContractError);
                }
                self.runtime
                    .registry_mut()
                    .stage_module_method_body(module, selector, body)
                    .map_err(module_error)?
            }
            None => {
                let definition = ModuleMethodDefinition::new(selector, body, visibility(method));
                if self.runtime.registry().active_module(module).is_ok() {
                    self.runtime
                        .registry_mut()
                        .stage_module_declaration_method(module, definition)
                } else {
                    self.runtime
                        .registry_mut()
                        .stage_module_origin_method(module, definition)
                }
                .map_err(module_error)?
            }
        };
        self.module_methods.insert((module, selector), defined);
        self.module_method_overrides
            .insert(defined.id(), method.is_override);
        if method.kind == MethodKind::Instance {
            self.runtime
                .registry_mut()
                .publish_origin_method(main, selector, body, visibility(method))
                .map_err(EvaluationError::Class)?;
        }
        Ok(())
    }
}
