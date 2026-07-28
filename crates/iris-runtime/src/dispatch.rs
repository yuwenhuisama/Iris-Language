use crate::{
    BoundMethod, ClassError, ClassId, Method, MethodBody, MethodId, MethodOwner, ModuleId,
    MroEntry, Selector, Visibility,
};

/// Result of resolving an ordinary send before evaluator invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    /// The evaluator must invoke this exact selected Method.
    Invoke(Method),
    /// The evaluator would invoke `method_missing`; execution is intentionally deferred.
    WouldInvokeMethodMissing { selector: Selector },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchError {
    /// Class lookup failed before selector resolution.
    Class(ClassError),
    /// Lookup found a Method but its access policy denied the caller.
    VisibilityDenied { selector: Selector },
    /// A qualified Contract slot is unavailable and never falls back to ordinary lookup.
    ContractDispatch {
        contract: ModuleId,
        selector: Selector,
    },
    /// Binding cannot create a BoundMethod for an absent ordinary selector.
    MissingMethod { selector: Selector },
}

impl crate::ClassRegistry {
    /// Defines a closed Module with its ordered composition edges.
    pub fn define_module(&mut self, components: &[ModuleId]) -> Result<ModuleId, ClassError> {
        self.modules.define(components).map_err(Self::module_error)
    }
    /// Defines or replaces a Module Method slot with a new Method identity.
    pub fn define_module_method(
        &mut self,
        module: ModuleId,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
    ) -> Result<Method, ClassError> {
        let method_id = self.next_method()?;
        let method = self
            .modules
            .define_method(module, method_id, selector, body, visibility)
            .map_err(Self::module_error)?;
        self.methods.insert(method.id(), method);
        Ok(method)
    }
    /// Publishes one replacement Class Method slot with a new Method identity.
    pub fn publish_method(
        &mut self,
        class: ClassId,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
    ) -> Result<Method, ClassError> {
        let method = Method::new(
            self.next_method()?,
            MethodOwner::Class(class),
            selector,
            body,
            visibility,
        );
        let mut candidate = self.open(class)?;
        candidate.replace_method(selector, method.id());
        self.publish(candidate)?;
        self.methods.insert(method.id(), method);
        Ok(method)
    }
    /// Resolves an ordinary selector through the Class's active stored MRO.
    pub fn dispatch(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<DispatchOutcome, DispatchError> {
        for entry in self.active(class).map_err(DispatchError::Class)?.mro() {
            let found = match entry {
                MroEntry::Class(owner) => self
                    .active(*owner)
                    .map_err(DispatchError::Class)?
                    .methods()
                    .get(&selector)
                    .and_then(|id| self.methods.get(id))
                    .copied(),
                MroEntry::Module(module) => self.modules.method(*module, selector),
            };
            if let Some(method) = found {
                return match method.visibility() {
                    Visibility::Public => Ok(DispatchOutcome::Invoke(method)),
                    Visibility::Private => Err(DispatchError::VisibilityDenied { selector }),
                };
            }
        }
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    }
    /// Binds the exact Method identity selected at binding time.
    pub fn bind(
        &mut self,
        class: ClassId,
        selector: Selector,
    ) -> Result<BoundMethod, DispatchError> {
        match self.dispatch(class, selector)? {
            DispatchOutcome::Invoke(method) => {
                Ok(BoundMethod::new(self.next_bound_method()?, class, method))
            }
            DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                Err(DispatchError::MissingMethod { selector })
            }
        }
    }

    /// Reports a missing qualified Contract slot without ordinary fallback.
    pub fn dispatch_contract(
        &self,
        class: ClassId,
        contract: ModuleId,
        selector: Selector,
    ) -> Result<DispatchOutcome, DispatchError> {
        self.active(class).map_err(DispatchError::Class)?;
        Err(DispatchError::ContractDispatch { contract, selector })
    }

    pub(crate) fn next_method(&mut self) -> Result<MethodId, ClassError> {
        let method = MethodId::new(self.next_method_id);
        self.next_method_id = self
            .next_method_id
            .checked_add(1)
            .ok_or(ClassError::MethodIdentityExhausted)?;
        Ok(method)
    }

    fn next_bound_method(&mut self) -> Result<crate::ObjectId, DispatchError> {
        let bound = crate::ObjectId::new(self.next_bound_method_id);
        self.next_bound_method_id = self
            .next_bound_method_id
            .checked_add(1)
            .ok_or(DispatchError::Class(ClassError::MethodIdentityExhausted))?;
        Ok(bound)
    }
}
