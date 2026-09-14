use super::{Machine, MachineError, literal_runtime_value, selector_id};
use crate::compile::Program;
use iris_runtime::{
    ClassId, DecoratorTransform, Method, MethodBody, MethodOwner, MroEntry, Selector, StaticSpine,
    Value, Visibility,
};

impl Machine {
    pub(super) fn publish_origin(
        &mut self,
        program: &Program,
        classes: &mut Vec<ClassId>,
    ) -> Result<(), MachineError> {
        let owner = self.code.register(program)?;
        let program = owner.as_ref();
        let index = classes.len();
        let declaration = &program.classes[index];
        let superclass = declaration.superclass.map(|parent| classes[parent]);
        let mut edges = Vec::new();
        for (name, private) in &declaration.mixins {
            let module = match self
                .modules
                .iter()
                .find_map(|(known, module)| (known == name).then_some(*module))
            {
                Some(module) => module,
                None => {
                    let source = program
                        .classes
                        .iter()
                        .position(|source| source.name == *name)
                        .ok_or(MachineError::NameError)?;
                    let owner = classes
                        .get(source)
                        .copied()
                        .ok_or(MachineError::NameError)?;
                    self.runtime
                        .registry()
                        .class(owner)
                        .map_err(MachineError::Class)?;
                    let module = self
                        .runtime
                        .registry_mut()
                        .define_module(&[])
                        .map_err(MachineError::Class)?;
                    for (name, function) in &program.classes[source].methods {
                        let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
                        let method = self
                            .runtime
                            .registry_mut()
                            .define_module_method(
                                module,
                                selector,
                                self.code.body(*function, program)?,
                                Visibility::Public,
                            )
                            .map_err(MachineError::Class)?;
                        self.remember_signature(method, program);
                    }
                    self.modules.push((name.clone(), module));
                    program
                        .link
                        .as_ref()
                        .ok_or(MachineError::UnsupportedConstruct)?
                        .modules
                        .replace(self.modules.clone());
                    module
                }
            };
            edges.push(iris_runtime::CompositionEdge::new(module, *private));
        }
        let class = self
            .runtime
            .registry_mut()
            .stage_class_origin(
                StaticSpine::new(
                    u64::try_from(index).map_err(|_| MachineError::UnsupportedConstruct)? + 1,
                )
                .with_meta_capabilities(super::runtime::meta_capabilities(&declaration.meta_deny)?),
                superclass,
                &edges,
            )
            .map_err(|error| {
                self.runtime.roll_back_group();
                MachineError::Class(error)
            })?;
        let metadata = self.decorator_metadata.len();
        classes.push(class);
        self.code.retain_classes(program, classes)?;
        let result = (|| {
            if declaration.contract_signature_clash {
                return Err(MachineError::TypeContractError);
            }
            if let Some(name) = &declaration.duplicate_class_variable {
                let name = selector_id(program, name).ok_or(MachineError::NameError)?;
                return Err(MachineError::Class(
                    iris_runtime::ClassError::DuplicateClassVariable { class, name },
                ));
            }
            if let Some(name) = declaration.override_required.first() {
                let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
                return Err(MachineError::Class(
                    iris_runtime::ClassError::OverrideRequired { class, selector },
                ));
            }
            self.runtime
                .registry_mut()
                .stage_decorators(
                    class,
                    declaration.decorators.iter().map(|(identity, arguments)| {
                        DecoratorTransform::metadata(identity, arguments)
                    }),
                )
                .map_err(MachineError::Class)?;
            for (name, function) in &declaration.methods {
                let visibility = match program.functions[*function]
                    .signature
                    .as_ref()
                    .map(|signature| signature.visibility)
                {
                    Some(iris_syntax::Visibility::Protected) => Visibility::Protected,
                    Some(iris_syntax::Visibility::Private) if name != "initialize" => {
                        Visibility::Private
                    }
                    Some(iris_syntax::Visibility::Private | iris_syntax::Visibility::Public)
                    | None => {
                        if declaration.private_methods.contains(name) && name != "initialize" {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        }
                    }
                };
                let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_origin_method(
                        class,
                        selector,
                        self.code.body(*function, program)?,
                        visibility,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
            for property in &declaration.stored_properties {
                let selector =
                    selector_id(program, &property.name).ok_or(MachineError::NameError)?;
                self.runtime
                    .registry_mut()
                    .stage_origin_property(class, selector, MethodBody::new(0))
                    .map_err(MachineError::Class)?;
            }
            for (name, function) in &declaration.class_methods {
                let selector = selector_id(program, name).ok_or(MachineError::NameError)?;
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_singleton_method(
                        class,
                        selector,
                        self.code.body(*function, program)?,
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
            for variable in &declaration.class_variables {
                let selector =
                    selector_id(program, &variable.name).ok_or(MachineError::NameError)?;
                self.runtime
                    .registry_mut()
                    .declare_class_var(class, selector, variable.mutable)
                    .map_err(MachineError::Class)?;
            }
            self.register_qualified_methods((class, index), program)?;
            for selector in &declaration.static_impls {
                let selector = selector_id(program, selector).ok_or(MachineError::NameError)?;
                self.static_impl_slots.insert((class, selector));
            }
            for variable in &declaration.class_variables {
                let selector =
                    selector_id(program, &variable.name).ok_or(MachineError::NameError)?;
                let value = match variable.initializer_function {
                    Some(function) => {
                        self.invoke_function(function, vec![Value::Class(class)], program, classes)?
                    }
                    None => literal_runtime_value(&variable.initializer)?,
                };
                self.runtime
                    .initialize_staged_class_var(class, selector, value)
                    .map_err(MachineError::Construction)?;
            }
            self.apply_decorators(index, program, classes)?;
            self.runtime
                .commit_structural_group()
                .map_err(|error| self.structural_error(error))?;
            Ok(())
        })();
        if let Err(error) = result {
            self.discard_qualified_methods(class);
            self.static_impl_slots.retain(|(owner, _)| *owner != class);
            classes.pop();
            self.code.retain_classes(program, classes)?;
            self.runtime.roll_back_group();
            let registry = self.runtime.registry();
            self.wrapper_chains.retain(|method, _| {
                registry
                    .method_by_id(*method)
                    .is_none_or(|method| method.owner() != MethodOwner::Class(class))
            });
            self.method_signatures.retain(|method, _| {
                registry
                    .method_by_id(*method)
                    .is_some_and(|method| method.owner() != MethodOwner::Class(class))
            });
            self.decorator_metadata.truncate(metadata);
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn origin_method(&self, class: ClassId, selector: Selector) -> Option<Method> {
        let registry = self.runtime.registry();
        let origin = registry.staged_origin(class).ok()?;
        origin.mro().iter().find_map(|entry| match entry {
            MroEntry::Class(owner) if *owner == class => origin
                .method(selector)
                .and_then(|method| registry.method_by_id(method)),
            MroEntry::Class(owner) => registry
                .resolve_local_or_ancestor_method(*owner, selector)
                .ok(),
            MroEntry::Module(module) => registry.module_method(*module, selector),
        })
    }
}
