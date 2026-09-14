use super::decorator_wrappers::WrapperChain;
use super::{Machine, MachineError, selector_id};
use crate::compile::{Program, decorators::Target};
use iris_runtime::decorator_protocol::Operation;
use iris_runtime::{ClassId, DecoratorValue, ModuleMethodDefinition, Value, Visibility};

mod replay;

impl Machine {
    pub(super) fn publish_module(
        &mut self,
        index: usize,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let owner = self.code.register(program)?;
        let program = owner.as_ref();
        self.code.retain_classes(program, classes)?;
        program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .modules
            .replace(self.modules.clone());
        let declaration = &program.modules[index];
        let reopen = declaration
            .replay
            .as_ref()
            .is_some_and(|artifact| artifact.reopen);
        if reopen && self.decorator_planning {
            for application in program
                .decorator_applications
                .iter()
                .filter(|application| application.target == Target::Module(index))
            {
                self.execute_decorator_phase(application, program, classes)?;
            }
            return Ok(());
        }
        if self
            .modules
            .iter()
            .any(|(name, _)| name == &declaration.name)
            && !reopen
        {
            return Ok(());
        }
        let edges = declaration
            .mixins
            .iter()
            .filter(|_| !reopen)
            .map(|name| {
                self.active_module_id(name)
                    .map(|module| iris_runtime::CompositionEdge::new(module, false))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let policy = super::runtime::meta_capabilities(&declaration.meta_deny)?;
        let module = if reopen {
            self.active_module_id(&declaration.name)?
        } else {
            self.runtime
                .registry_mut()
                .stage_module_origin(&edges, policy)
                .map_err(|error| self.module_candidate_error(error))?
        };
        let metadata = self.decorator_metadata.len();
        let signatures = self.method_signatures.clone();
        let chains = self.wrapper_chains.clone();
        let selectors = self.dynamic_selectors.clone();
        let next_selector = self.next_dynamic_selector;
        self.open_depth += usize::from(reopen);
        let root_base = self.active_values.len();
        let result =
            (|| {
                if reopen {
                    self.stage_module_replay((module, index), program)?;
                }
                for (function, body) in program.functions.iter().enumerate() {
                    if reopen
                        || declaration
                            .replay
                            .as_ref()
                            .is_some_and(|artifact| !artifact.functions.contains(&function))
                    {
                        continue;
                    }
                    let Some((owner, name)) = body.name.split_once('.') else {
                        continue;
                    };
                    if owner != declaration.name {
                        continue;
                    }
                    let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
                    let visibility = match body
                        .signature
                        .as_ref()
                        .map(|signature| signature.visibility)
                    {
                        Some(iris_syntax::Visibility::Private) => Visibility::Private,
                        Some(iris_syntax::Visibility::Protected) => Visibility::Protected,
                        Some(iris_syntax::Visibility::Public) | None => Visibility::Public,
                    };
                    let body = self.code.body(function, program)?;
                    let method = self
                        .runtime
                        .registry_mut()
                        .stage_module_origin_method(
                            module,
                            ModuleMethodDefinition::new(selector, body, visibility),
                        )
                        .map_err(|error| self.module_candidate_error(error))?;
                    self.remember_signature(method, program);
                }
                let mut wrappers =
                    std::collections::BTreeMap::<usize, Vec<iris_runtime::ObjectId>>::new();
                for application in program
                    .decorator_applications
                    .iter()
                    .filter(|application| application.target == Target::Module(index))
                {
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
                            Operation::AddMethod { selector, body } => {
                                let Value::Closure(identity) = body else {
                                    return Err(self.decorator_type_error());
                                };
                                let closure = self
                                    .closures
                                    .get(identity)
                                    .cloned()
                                    .ok_or(MachineError::UnsupportedConstruct)?;
                                let program = closure.program.as_ref();
                                let owned_classes = program
                                    .link
                                    .as_ref()
                                    .ok_or(MachineError::UnsupportedConstruct)?
                                    .classes
                                    .borrow()
                                    .clone();
                                let classes = owned_classes.as_slice();
                                let function = closure.function;
                                let signature = program.functions[function]
                                    .signature
                                    .as_ref()
                                    .ok_or(MachineError::UnsupportedConstruct)?;
                                if signature.return_type.is_none()
                                    || signature.parameters.iter().any(|parameter| {
                                        parameter.annotation.is_none()
                                            || parameter.default.is_some()
                                            || parameter.category
                                                != iris_syntax::ParameterCategory::Positional
                                    })
                                {
                                    return Err(MachineError::UnsupportedConstruct);
                                }
                                for annotation in signature
                                    .parameters
                                    .iter()
                                    .filter_map(|parameter| parameter.annotation.as_ref())
                                    .chain(signature.return_type.as_ref())
                                {
                                    self.reify_type(annotation, program, classes)?;
                                }
                                let selector = selector_id(program, selector)
                                    .unwrap_or_else(|| self.dynamic_selector(selector));
                                let body = self.code.body(function, program)?;
                                let method = self
                                    .runtime
                                    .registry_mut()
                                    .stage_module_method(
                                        module,
                                        ModuleMethodDefinition::new(
                                            selector,
                                            body,
                                            Visibility::Private,
                                        ),
                                    )
                                    .map_err(|error| self.module_candidate_error(error))?;
                                self.remember_signature(method, program);
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
                            Operation::WrapMethod(wrapper) => {
                                let function = application
                                    .method
                                    .ok_or(MachineError::UnsupportedConstruct)?;
                                if program.functions[function].instructions.iter().any(
                                    |instruction| {
                                        matches!(
                                            instruction,
                                            crate::Instruction::NativeCall { .. }
                                                | crate::Instruction::NativeFixture { .. }
                                        )
                                    },
                                ) {
                                    return Err(MachineError::UnsupportedConstruct);
                                }
                                let wrapper = self.admit_wrapper(
                                    wrapper,
                                    program,
                                    program.functions[function].is_async,
                                )?;
                                self.active_values.push(Value::Closure(wrapper));
                                wrappers.entry(function).or_default().push(wrapper);
                            }
                            Operation::WrapGetter(_) | Operation::WrapSetter(_) => {
                                return Err(MachineError::LexicalDiagnostic("IRIS-DECORATOR-KIND"));
                            }
                        }
                    }
                }
                for (function, wrappers) in wrappers {
                    let signature = program.functions[function]
                        .signature
                        .as_ref()
                        .ok_or(MachineError::UnsupportedConstruct)?;
                    let selector =
                        selector_id(program, &signature.selector).ok_or(MachineError::NameError)?;
                    let original = self
                        .runtime
                        .registry()
                        .staged_module(module)
                        .map_err(|_| MachineError::MetaTransactionError)?
                        .method(selector)
                        .ok_or(MachineError::NameError)?;
                    let method = self
                        .runtime
                        .registry_mut()
                        .stage_module_method_body(module, selector, original.body())
                        .map_err(|error| self.module_candidate_error(error))?;
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
                if self.open_depth != usize::from(reopen) {
                    return Err(MachineError::MetaTransactionError);
                }
                self.validate_replacements(program)?;
                self.runtime
                    .commit_structural_group()
                    .map_err(|error| self.structural_error(error))?;
                Ok(())
            })();
        self.active_values.truncate(root_base);
        self.open_depth -= usize::from(reopen);
        self.pending_replacements.clear();
        if result.is_err() {
            self.runtime.roll_back_group();
            self.decorator_metadata.truncate(metadata);
            self.wrapper_chains = chains;
            self.method_signatures = signatures;
            self.dynamic_selectors = selectors;
            self.next_dynamic_selector = next_selector;
        } else if !reopen {
            self.modules.push((declaration.name.clone(), module));
            program
                .link
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?
                .modules
                .replace(self.modules.clone());
        }
        result
    }
}
