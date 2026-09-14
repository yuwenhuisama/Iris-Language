use super::{Machine, MachineError, selector_id};
use crate::compile::Program;
use iris_runtime::{ClassId, Method, MethodOwner, Selector, Visibility};
use std::rc::Rc;

impl Machine {
    pub(super) fn link_history(
        &mut self,
        mut program: Program,
        current: &Program,
    ) -> Result<Rc<Program>, MachineError> {
        let current_link = current
            .link
            .as_ref()
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        let mut classes = Vec::new();
        for declaration in &program.classes {
            let index = current
                .classes
                .iter()
                .position(|known| known.name == declaration.name)
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            let class = current_link
                .classes
                .borrow()
                .get(index)
                .copied()
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            let active = self
                .runtime
                .registry()
                .active(class)
                .map_err(MachineError::Class)?;
            if current.classes[index].generic
                || !active.properties().is_empty()
                || !active.class_vars().is_empty()
                || !active.modules().is_empty()
                || current.classes[index].superclass.is_some()
            {
                return Err(MachineError::UnsupportedConstruct);
            }
            classes.push(class);
        }
        for contract in &mut program.contracts {
            let index = current
                .contracts
                .iter()
                .position(|known| known.name == contract.name)
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            if contract.requirements != current.contracts[index].requirements {
                return Err(MachineError::TypeContractError);
            }
            contract.core_identity = Some(current.contract_identity(index));
        }
        program.package = current.package.clone();
        super::verify(&program).map_err(MachineError::Invalid)?;
        let program = self.code.load(&program)?;
        self.code.retain_classes(&program, &classes)?;
        let link = program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        link.modules.replace(current_link.modules.borrow().clone());
        for index in program
            .decorator_applications
            .iter()
            .map(|application| application.decorator)
        {
            let class = classes[index];
            for (name, function) in &program.classes[index].methods {
                let selector = selector_id(&program, name).ok_or(MachineError::NameError)?;
                if link
                    .history_methods
                    .borrow()
                    .contains_key(&(class, selector))
                {
                    continue;
                }
                let visibility = match program.functions[*function]
                    .signature
                    .as_ref()
                    .map(|signature| signature.visibility)
                {
                    Some(iris_syntax::Visibility::Private) => Visibility::Private,
                    Some(iris_syntax::Visibility::Protected) => Visibility::Protected,
                    Some(iris_syntax::Visibility::Public) | None => Visibility::Public,
                };
                let method = Method::new(
                    self.allocate_managed_method_identity()?,
                    MethodOwner::Class(class),
                    selector,
                    self.code.body(*function, &program)?,
                    visibility,
                );
                self.remember_signature(method, &program);
                link.history_methods
                    .borrow_mut()
                    .insert((class, selector), method);
            }
        }
        Ok(program)
    }

    pub(super) fn historical_method(
        &self,
        class: ClassId,
        selector: Selector,
        program: &Program,
    ) -> Option<Method> {
        program
            .link
            .as_ref()?
            .history_methods
            .borrow()
            .get(&(class, selector))
            .copied()
    }

    pub(super) fn historical_instance_method(
        &self,
        object: iris_runtime::ObjectId,
        selector: Selector,
    ) -> Option<Method> {
        let program = self.history_context.as_ref()?;
        self.historical_method(self.runtime.class_of(object).ok()?, selector, program)
    }

    pub(super) fn constructor_method(&self, object: iris_runtime::ObjectId) -> Option<Method> {
        let class = self.runtime.class_of(object).ok()?;
        if let Some(program) = &self.history_context
            && let Some(link) = &program.link
            && link
                .history_methods
                .borrow()
                .keys()
                .any(|(owner, _)| *owner == class)
        {
            return self.historical_method(class, Selector::INITIALIZE, program);
        }
        self.runtime
            .dispatch_instance(object, Selector::INITIALIZE)
            .ok()
    }
}
