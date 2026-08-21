//! Runtime class registration and exception matching.

use iris_runtime::{
    BuiltinClass, ClassError, ClassId, KernelError, MethodBody, StaticSpine, Value, Visibility,
};

use crate::compile::Program;

use super::{Machine, MachineError, literal_runtime_value, selector_id};

impl Machine {
    pub(super) fn register_classes(
        &mut self,
        program: &Program,
    ) -> Result<Vec<ClassId>, MachineError> {
        let mut classes = Vec::with_capacity(program.classes.len());
        for (index, declaration) in program.classes.iter().enumerate() {
            let superclass = declaration
                .superclass
                .and_then(|parent| classes.get(parent).copied());
            let class = self
                .runtime
                .registry_mut()
                .define_class(StaticSpine::new(index as u64 + 1), superclass)
                .map_err(MachineError::Class)?;
            self.runtime
                .registry_mut()
                .begin_origin_transaction(class)
                .map_err(MachineError::Class)?;
            for (selector, function) in &declaration.methods {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                self.runtime
                    .registry_mut()
                    .publish_origin_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
            }
            for variable in &declaration.class_variables {
                let selector = selector_id(program, &variable.name)
                    .ok_or_else(|| MachineError::UnknownSelector(variable.name.clone()))?;
                let value = literal_runtime_value(&variable.initializer)?;
                self.runtime
                    .declare_class_var(class, selector, value, variable.mutable)
                    .map_err(MachineError::Construction)?;
            }
            for (selector, function) in &declaration.class_methods {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                self.runtime
                    .registry_mut()
                    .publish_singleton_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
            }
            self.runtime
                .registry_mut()
                .commit_origin_transaction(class)
                .map_err(MachineError::Class)?;
            for reopen in &declaration.reopens {
                self.runtime
                    .registry_mut()
                    .begin_transaction(class)
                    .map_err(MachineError::Class)?;
                for (selector, function) in &reopen.methods {
                    let selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    self.runtime
                        .registry_mut()
                        .publish_method(
                            class,
                            selector,
                            MethodBody::new(*function as u64),
                            Visibility::Public,
                        )
                        .map_err(MachineError::Class)?;
                }
                self.runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(MachineError::Class)?;
            }
            classes.push(class);
        }
        Ok(classes)
    }

    pub(super) fn catch_matches(
        &self,
        value: &Value,
        name: &str,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<bool, MachineError> {
        let filter = match name {
            "Symbol" => return Ok(matches!(value, Value::Symbol(_))),
            "Integer" => return Ok(matches!(value, Value::Integer(_))),
            "Nil" => return Ok(matches!(value, Value::Nil)),
            "Bool" => return Ok(matches!(value, Value::Bool(_))),
            "Object" => self
                .kernel
                .class(BuiltinClass::Object)
                .map_err(MachineError::Kernel),
            _ => program
                .classes
                .iter()
                .position(|class| class.name == name)
                .and_then(|index| classes.get(index).copied())
                .map_or_else(|| Err(MachineError::Kernel(KernelError::Type)), Ok),
        }?;
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let mut class = self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?;
        loop {
            if class == filter {
                return Ok(true);
            }
            let Some(superclass) = self
                .runtime
                .registry()
                .active(class)
                .map_err(MachineError::Class)?
                .runtime_superclass()
            else {
                return Ok(false);
            };
            class = superclass;
        }
    }

    pub(super) fn receiver_class(&self, receiver: &Value) -> Result<ClassId, MachineError> {
        match receiver {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction),
            Value::Class(class) => Ok(*class),
            _ => Err(MachineError::Kernel(KernelError::Type)),
        }
    }

    pub(super) fn initialize_properties(
        &mut self,
        program: &Program,
        classes: &[ClassId],
        class: ClassId,
        object: iris_runtime::ObjectId,
    ) -> Result<(), MachineError> {
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
        };
        for property in &program.classes[index].stored_properties {
            let selector = selector_id(program, &property.name)
                .ok_or_else(|| MachineError::UnknownSelector(property.name.clone()))?;
            let value = literal_runtime_value(&property.initializer)?;
            self.runtime
                .assign_raw_ivar(object, selector, value)
                .map_err(MachineError::Construction)?;
        }
        Ok(())
    }
}
