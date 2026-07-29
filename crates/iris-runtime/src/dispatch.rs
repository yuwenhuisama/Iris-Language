use crate::{
    BoundMethod, BoundReceiver, ClassError, ClassId, DecoratorTransform, Method, MethodBody,
    MethodId, MethodOwner, ModuleId, MroEntry, Selector, Visibility,
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
    /// The retained Method lexical owner is not in the receiver's current MRO.
    InvalidSuper { selector: Selector },
    /// No same-selector implementation follows the lexical owner in current MRO.
    NoSuperMethod { selector: Selector },
}

/// Lexical authority supplied by an evaluator for an ordinary message send.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchContext {
    lexical_class: Option<ClassId>,
    receiver_is_self: bool,
}

impl DispatchContext {
    /// Represents an external send with no implementation authority.
    pub const fn external() -> Self {
        Self {
            lexical_class: None,
            receiver_is_self: false,
        }
    }

    /// Represents code lexically owned by one Class.
    pub const fn implementation(lexical_class: ClassId, receiver_is_self: bool) -> Self {
        Self {
            lexical_class: Some(lexical_class),
            receiver_is_self,
        }
    }
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
        let capability = if self.active(class)?.methods().contains_key(&selector) {
            crate::Capability::MethodBody
        } else {
            crate::Capability::MethodSet
        };
        if !self.active_meta_capabilities(class)?.allows(capability) {
            return Err(ClassError::MetaCapabilityDenied { class, capability });
        }
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

    /// Publishes one singleton Method on a specific Class object.
    pub fn publish_singleton_method(
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
        candidate.replace_singleton_method(selector, method.id());
        self.publish(candidate)?;
        self.methods.insert(method.id(), method);
        Ok(method)
    }

    /// Publishes a decorator-transformed Method through the ordinary capability-checked candidate.
    pub fn publish_decorated_method(
        &mut self,
        class: ClassId,
        selector: Selector,
        body: MethodBody,
        visibility: Visibility,
        decorators: impl IntoIterator<Item = DecoratorTransform>,
    ) -> Result<Method, ClassError> {
        let capability = if self.active(class)?.methods().contains_key(&selector) {
            crate::Capability::MethodBody
        } else {
            crate::Capability::MethodSet
        };
        if !self.active_meta_capabilities(class)?.allows(capability) {
            return Err(ClassError::MetaCapabilityDenied { class, capability });
        }
        let method = Method::new(
            self.next_method()?,
            MethodOwner::Class(class),
            selector,
            body,
            visibility,
        );
        let mut candidate = self.open(class)?;
        candidate.stage_decorators(decorators);
        candidate.replace_method(selector, method.id());
        self.publish(candidate)?;
        self.methods.insert(method.id(), method);
        Ok(method)
    }

    /// Publishes one stored-property initializer in declaration order.
    pub fn publish_stored_property(
        &mut self,
        class: ClassId,
        selector: Selector,
        initializer: MethodBody,
    ) -> Result<(), ClassError> {
        let capability = if self
            .active(class)?
            .properties()
            .iter()
            .any(|property| property.selector() == selector)
        {
            crate::Capability::PropertyBody
        } else {
            crate::Capability::PropertySet
        };
        if !self.active_meta_capabilities(class)?.allows(capability) {
            return Err(ClassError::MetaCapabilityDenied { class, capability });
        }
        let mut candidate = self.open(class)?;
        candidate.add_stored_property(crate::StoredProperty::new(selector, initializer));
        self.publish(candidate)?;
        Ok(())
    }

    /// Publishes a decorator-transformed stored property through normal capability checks.
    pub fn publish_decorated_stored_property(
        &mut self,
        class: ClassId,
        selector: Selector,
        initializer: MethodBody,
        decorators: impl IntoIterator<Item = DecoratorTransform>,
    ) -> Result<(), ClassError> {
        let capability = if self
            .active(class)?
            .properties()
            .iter()
            .any(|property| property.selector() == selector)
        {
            crate::Capability::PropertyBody
        } else {
            crate::Capability::PropertySet
        };
        if !self.active_meta_capabilities(class)?.allows(capability) {
            return Err(ClassError::MetaCapabilityDenied { class, capability });
        }
        let mut candidate = self.open(class)?;
        candidate.stage_decorators(decorators);
        candidate.add_stored_property(crate::StoredProperty::new(selector, initializer));
        self.publish(candidate)?;
        Ok(())
    }

    /// Declares a hierarchy Class-variable cell on a logical Class.
    pub fn declare_class_var(
        &mut self,
        class: ClassId,
        name: Selector,
        mutable: bool,
    ) -> Result<(), ClassError> {
        let mut candidate = self.open(class)?;
        if candidate.class_vars.contains(&name) {
            return Err(ClassError::DuplicateClassVariable { class, name });
        }
        candidate.add_class_var(name, mutable);
        self.publish(candidate)?;
        Ok(())
    }

    pub(crate) fn method(&self, id: MethodId) -> Option<Method> {
        self.methods.get(&id).copied()
    }

    pub(crate) fn module_method(&self, module: ModuleId, selector: Selector) -> Option<Method> {
        self.modules.method(module, selector)
    }
    /// Resolves an ordinary selector through the Class's active stored MRO.
    pub fn dispatch(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<DispatchOutcome, DispatchError> {
        self.dispatch_with_context(class, selector, DispatchContext::external())
    }

    /// Resolves an ordinary selector with the caller's lexical visibility authority.
    pub fn dispatch_with_context(
        &self,
        class: ClassId,
        selector: Selector,
        context: DispatchContext,
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
                if self.authorizes(class, method, context)? {
                    return Ok(DispatchOutcome::Invoke(method));
                }
                return Err(DispatchError::VisibilityDenied { selector });
            }
        }
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    }

    /// Resolves a selector sent to a Class object through singleton superclass lookup.
    pub fn dispatch_class_object(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<DispatchOutcome, DispatchError> {
        let mut current = Some(class);
        while let Some(owner) = current {
            let revision = self.active(owner).map_err(DispatchError::Class)?;
            if let Some(method) = revision
                .singleton_methods()
                .get(&selector)
                .and_then(|id| self.methods.get(id))
                .copied()
            {
                return Ok(DispatchOutcome::Invoke(method));
            }
            current = revision.runtime_superclass();
        }
        Ok(DispatchOutcome::WouldInvokeMethodMissing { selector })
    }

    fn authorizes(
        &self,
        receiver: ClassId,
        method: Method,
        context: DispatchContext,
    ) -> Result<bool, DispatchError> {
        match method.visibility() {
            Visibility::Public => Ok(true),
            Visibility::Private => Ok(matches!(
                (method.owner(), context.lexical_class),
                (MethodOwner::Class(owner), Some(caller)) if owner == caller
            )),
            Visibility::Protected => match (method.owner(), context.lexical_class) {
                (MethodOwner::Class(owner), Some(caller)) => Ok(context.receiver_is_self
                    && self
                        .active(caller)
                        .map_err(DispatchError::Class)?
                        .mro()
                        .contains(&MroEntry::Class(owner))
                    && self
                        .active(receiver)
                        .map_err(DispatchError::Class)?
                        .mro()
                        .contains(&MroEntry::Class(owner))),
                (MethodOwner::Module(_), _) | (MethodOwner::Class(_), None) => Ok(false),
            },
        }
    }
    /// Binds the exact Method identity selected at binding time.
    pub fn bind(
        &mut self,
        class: ClassId,
        selector: Selector,
    ) -> Result<BoundMethod, DispatchError> {
        match self.dispatch(class, selector)? {
            DispatchOutcome::Invoke(method) => Ok(BoundMethod::new(
                self.next_bound_method()?,
                BoundReceiver::Class(class),
                method,
            )),
            DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                Err(DispatchError::MissingMethod { selector })
            }
        }
    }

    /// Binds the exact Method identity selected for one ordinary object receiver.
    pub fn bind_instance(
        &mut self,
        receiver: crate::ObjectId,
        class: ClassId,
        selector: Selector,
    ) -> Result<BoundMethod, DispatchError> {
        match self.dispatch(class, selector)? {
            DispatchOutcome::Invoke(method) => Ok(BoundMethod::new(
                self.next_bound_method()?,
                BoundReceiver::Object(receiver),
                method,
            )),
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

    /// Resolves the same selector after a Method's lexical owner in current receiver MRO.
    pub fn dispatch_super(&self, class: ClassId, method: Method) -> Result<Method, DispatchError> {
        let mro = self.active(class).map_err(DispatchError::Class)?.mro();
        let owner = match method.owner() {
            MethodOwner::Class(owner) => MroEntry::Class(owner),
            MethodOwner::Module(owner) => MroEntry::Module(owner),
        };
        let index =
            mro.iter()
                .position(|entry| *entry == owner)
                .ok_or(DispatchError::InvalidSuper {
                    selector: method.selector(),
                })?;
        for entry in &mro[index + 1..] {
            let successor = match entry {
                MroEntry::Class(owner) => self
                    .active(*owner)
                    .map_err(DispatchError::Class)?
                    .methods()
                    .get(&method.selector())
                    .and_then(|id| self.methods.get(id))
                    .copied(),
                MroEntry::Module(module) => self.modules.method(*module, method.selector()),
            };
            if let Some(successor) = successor {
                return Ok(successor);
            }
        }
        Err(DispatchError::NoSuperMethod {
            selector: method.selector(),
        })
    }

    /// Resolves the same selector after a Class object's singleton Method owner.
    pub fn dispatch_class_object_super(
        &self,
        class: ClassId,
        method: Method,
    ) -> Result<Method, DispatchError> {
        let MethodOwner::Class(owner) = method.owner() else {
            return Err(DispatchError::InvalidSuper {
                selector: method.selector(),
            });
        };
        let mut current = Some(class);
        while let Some(candidate) = current {
            let revision = self.active(candidate).map_err(DispatchError::Class)?;
            if candidate == owner {
                current = revision.runtime_superclass();
                while let Some(successor) = current {
                    let successor_revision =
                        self.active(successor).map_err(DispatchError::Class)?;
                    if let Some(found) = successor_revision
                        .singleton_methods()
                        .get(&method.selector())
                        .and_then(|id| self.methods.get(id))
                        .copied()
                    {
                        return Ok(found);
                    }
                    current = successor_revision.runtime_superclass();
                }
                break;
            }
            current = revision.runtime_superclass();
        }
        Err(DispatchError::NoSuperMethod {
            selector: method.selector(),
        })
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
