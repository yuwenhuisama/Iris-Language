use super::decorator_wrappers::WrapperChain;
use super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::decorator_protocol::Operation;
use iris_runtime::{ClassId, DecoratorValue, Value};

impl Machine {
    pub(super) fn apply_decorators(
        &mut self,
        target: usize,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let applications: Vec<_> = program
            .decorator_applications
            .iter()
            .filter(|application| {
                application.target == crate::compile::decorators::Target::Class(target)
            })
            .collect();
        if applications.is_empty() {
            return Ok(());
        }
        let class = classes[target];
        self.install_decorator_applications(class, &applications, program, classes)
    }

    pub(super) fn install_decorator_applications(
        &mut self,
        class: ClassId,
        applications: &[&crate::compile::decorators::Application],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let root_base = self.active_values.len();
        let result = (|| {
            let mut wrappers: std::collections::BTreeMap<usize, Vec<iris_runtime::ObjectId>> =
                std::collections::BTreeMap::new();
            for application in applications {
                let value = self.execute_decorator_phase(application, program, classes)?;
                if self.decorator_planning {
                    continue;
                }
                let Value::Decorator(record) = value else {
                    return Err(self.decorator_type_error());
                };
                let DecoratorValue::Transformation(transformation) = record.as_ref() else {
                    return Err(self.decorator_type_error());
                };
                for operation in transformation.operations() {
                    match operation {
                        Operation::WrapMethod(wrapper) => {
                            let function = application
                                .method
                                .ok_or(MachineError::UnsupportedConstruct)?;
                            let wrapper = self.admit_wrapper(
                                wrapper,
                                program,
                                program.functions[function].is_async,
                            )?;
                            self.active_values.push(Value::Closure(wrapper));
                            wrappers.entry(function).or_default().push(wrapper);
                        }
                        Operation::AddMethod { selector, body } => {
                            if program
                                .link
                                .as_ref()
                                .is_some_and(|link| !link.history_methods.borrow().is_empty())
                            {
                                return Err(MachineError::UnsupportedConstruct);
                            }
                            let Value::Closure(identity) = body else {
                                return Err(self.decorator_type_error());
                            };
                            let closure = self
                                .closures
                                .get(identity)
                                .cloned()
                                .ok_or(MachineError::UnsupportedConstruct)?;
                            let program = closure.program.as_ref();
                            let function = closure.function;
                            let metadata = program.functions[function]
                                .signature
                                .as_ref()
                                .ok_or(MachineError::UnsupportedConstruct)?;
                            if metadata.is_async
                                || metadata.return_type.is_none()
                                || metadata.parameters.iter().any(|parameter| {
                                    parameter.default.is_some()
                                        || parameter.category
                                            != iris_syntax::ParameterCategory::Positional
                                })
                            {
                                return Err(MachineError::UnsupportedConstruct);
                            }
                            let selector = selector_id(program, selector)
                                .unwrap_or_else(|| self.dynamic_selector(selector));
                            if self.origin_method(class, selector).is_some() {
                                return Err(MachineError::Class(
                                    iris_runtime::ClassError::OverrideRequired { class, selector },
                                ));
                            }
                            let method = self
                                .runtime
                                .registry_mut()
                                .publish_method(
                                    class,
                                    selector,
                                    self.code.body(function, program)?,
                                    iris_runtime::Visibility::Private,
                                )
                                .map_err(MachineError::Class)?;
                            self.wrapper_chains.insert(
                                method.id(),
                                WrapperChain {
                                    program: self.code.owner(program)?,
                                    function,
                                    wrappers: Vec::new(),
                                    body_closure: Some(*identity),
                                },
                            );
                        }
                        Operation::WrapGetter(wrapper) | Operation::WrapSetter(wrapper) => {
                            let logical = if let Some(property) = &application.property {
                                property.name.as_str()
                            } else {
                                let declaration =
                                    application.method.ok_or(MachineError::ArgumentError)?;
                                let signature = program.functions[declaration]
                                    .signature
                                    .as_ref()
                                    .ok_or(MachineError::ArgumentError)?;
                                signature.selector.trim_end_matches('=')
                            };
                            let addressed = match operation {
                                Operation::WrapGetter(_) => logical.to_owned(),
                                Operation::WrapSetter(_) => format!("{logical}="),
                                _ => return Err(MachineError::ArgumentError),
                            };
                            let selector = selector_id(program, &addressed)
                                .ok_or(MachineError::ArgumentError)?;
                            let original = self
                                .runtime
                                .registry()
                                .staged_method(class, selector)
                                .and_then(|identity| self.runtime.registry().method_by_id(identity))
                                .ok_or(MachineError::ArgumentError)?;
                            let function = self
                                .resolve_method_body(original.body(), program)
                                .map_err(|_| MachineError::ArgumentError)?;
                            let owner = function;
                            let (program, function) = (owner.program.as_ref(), owner.function);
                            let metadata = program.functions[function]
                                .signature
                                .as_ref()
                                .ok_or(MachineError::ArgumentError)?;
                            if metadata.kind != iris_syntax::MethodKind::Property
                                || metadata.selector != addressed
                            {
                                return Err(MachineError::ArgumentError);
                            }
                            let wrapper =
                                self.admit_wrapper(wrapper, program, metadata.is_async)?;
                            self.active_values.push(Value::Closure(wrapper));
                            wrappers.entry(function).or_default().push(wrapper);
                        }
                    }
                }
            }
            for (function, wrappers) in wrappers {
                let signature = program.functions[function]
                    .signature
                    .as_ref()
                    .ok_or(MachineError::UnsupportedConstruct)?;
                let property = signature.kind == iris_syntax::MethodKind::Property;
                self.runtime
                    .registry()
                    .require_candidate_meta_capability(
                        class,
                        if property {
                            iris_runtime::Capability::PropertyBody
                        } else {
                            iris_runtime::Capability::MethodBody
                        },
                    )
                    .map_err(MachineError::Class)?;
                let selector =
                    selector_id(program, &signature.selector).ok_or(MachineError::NameError)?;
                if signature.kind == iris_syntax::MethodKind::Class {
                    let visibility = match signature.visibility {
                        iris_syntax::Visibility::Public => iris_runtime::Visibility::Public,
                        iris_syntax::Visibility::Protected => iris_runtime::Visibility::Protected,
                        iris_syntax::Visibility::Private => iris_runtime::Visibility::Private,
                    };
                    let method = self
                        .runtime
                        .registry_mut()
                        .publish_singleton_method(
                            class,
                            selector,
                            self.code.body(function, program)?,
                            visibility,
                        )
                        .map_err(MachineError::Class)?;
                    self.remember_signature(method, program);
                    self.wrapper_chains.insert(
                        method.id(),
                        WrapperChain {
                            program: self.code.owner(program)?,
                            function,
                            wrappers,
                            body_closure: None,
                        },
                    );
                    continue;
                }
                if let Some(Some(qualifier)) = &signature.impl_contract {
                    let contract = program
                        .contracts
                        .iter()
                        .position(|contract| contract.name == *qualifier)
                        .ok_or(MachineError::NameError)?;
                    let slot = (class, program.contract_identity(contract), selector);
                    let original = self.qualified_method(slot).ok_or(MachineError::NameError)?;
                    let function = self
                        .resolve_method_body(original.body(), program)
                        .map_err(|_| MachineError::UnsupportedConstruct)?;
                    let owner = function;
                    let (program, function) = (owner.program.as_ref(), owner.function);
                    let method = self.publish_qualified_method(slot, function, program)?;
                    self.wrapper_chains.insert(
                        method.id(),
                        WrapperChain {
                            program: self.code.owner(program)?,
                            function,
                            wrappers,
                            body_closure: None,
                        },
                    );
                    continue;
                }
                let original = self
                    .runtime
                    .registry()
                    .staged_method(class, selector)
                    .and_then(|method| self.runtime.registry().method_by_id(method))
                    .ok_or(MachineError::NameError)?;
                let method = if property {
                    self.runtime.registry_mut().publish_origin_method(
                        class,
                        selector,
                        original.body(),
                        original.visibility(),
                    )
                } else {
                    self.runtime.registry_mut().publish_method(
                        class,
                        selector,
                        original.body(),
                        original.visibility(),
                    )
                }
                .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
                self.wrapper_chains.insert(
                    method.id(),
                    WrapperChain {
                        program: self.code.owner(program)?,
                        function,
                        wrappers,
                        body_closure: None,
                    },
                );
            }
            Ok(())
        })();
        self.active_values.truncate(root_base);
        if result.is_err() {
            self.runtime.registry_mut().roll_back_transaction(class);
        }
        result
    }
}
