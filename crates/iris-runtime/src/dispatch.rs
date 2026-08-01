use crate::{
    BoundMethod, BoundReceiver, ClassError, ClassId, CompositionEdge, DecoratorTransform, Method,
    MethodBody, MethodId, MethodOwner, ModuleId, MroEntry, Selector, Visibility,
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
    /// A reflected Method cannot be invoked on the receiver's current MRO.
    MethodBinding { selector: Selector },
    /// No same-selector implementation follows the lexical owner in current MRO.
    NoSuperMethod { selector: Selector },
}

/// Lexical authority supplied by an evaluator for an ordinary message send.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchContext {
    lexical_class: Option<ClassId>,
    lexical_module: Option<ModuleId>,
    receiver_is_self: bool,
}

impl DispatchContext {
    /// Represents an external send with no implementation authority.
    pub const fn external() -> Self {
        Self {
            lexical_class: None,
            lexical_module: None,
            receiver_is_self: false,
        }
    }

    /// Represents code lexically owned by one Class.
    pub const fn implementation(lexical_class: ClassId, receiver_is_self: bool) -> Self {
        Self {
            lexical_class: Some(lexical_class),
            lexical_module: None,
            receiver_is_self,
        }
    }

    pub const fn module_implementation(lexical_module: ModuleId, receiver_is_self: bool) -> Self {
        Self {
            lexical_class: None,
            lexical_module: Some(lexical_module),
            receiver_is_self,
        }
    }
}

impl crate::ClassRegistry {
    /// Installs a runtime-required origin Method before source-level policy applies.
    pub fn publish_origin_method(
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

    /// Defines a closed Module with its ordered composition edges.
    pub fn define_module(&mut self, components: &[ModuleId]) -> Result<ModuleId, ClassError> {
        let edges = components
            .iter()
            .copied()
            .map(|module| CompositionEdge::new(module, false))
            .collect::<Vec<_>>();
        self.define_module_with_composition_edges(&edges, crate::MetaCapabilities::all())
    }

    /// Defines a closed Module with its immutable source-level meta policy.
    pub fn define_module_with_capabilities(
        &mut self,
        components: &[ModuleId],
        capabilities: crate::MetaCapabilities,
    ) -> Result<ModuleId, ClassError> {
        let edges = components
            .iter()
            .copied()
            .map(|module| CompositionEdge::new(module, false))
            .collect::<Vec<_>>();
        self.define_module_with_composition_edges(&edges, capabilities)
    }

    /// Defines a closed Module with immutable composition-edge authority metadata.
    pub fn define_module_with_composition_edges(
        &mut self,
        components: &[CompositionEdge],
        capabilities: crate::MetaCapabilities,
    ) -> Result<ModuleId, ClassError> {
        self.modules
            .define(components, capabilities)
            .map_err(Self::module_error)
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
        self.require_meta_capability(class, capability)?;
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

    /// Creates another local slot that references the selected Method identity.
    pub fn alias_method(
        &mut self,
        class: ClassId,
        alias: Selector,
        original: Selector,
    ) -> Result<(), ClassError> {
        self.require_meta_capability(class, crate::Capability::MethodSet)?;
        let method = self.resolve_local_or_ancestor_method(class, original)?;
        let mut candidate = self.open(class)?;
        candidate.replace_method(alias, method.id());
        self.publish(candidate)?;
        Ok(())
    }

    fn resolve_local_or_ancestor_method(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<Method, ClassError> {
        for entry in self.active(class)?.mro() {
            let method = match entry {
                MroEntry::Class(owner) => {
                    let revision = self.active(*owner)?;
                    if revision.tombstones().contains(&selector) {
                        return Err(ClassError::MethodSlotNotFound { class, selector });
                    }
                    revision
                        .methods()
                        .get(&selector)
                        .and_then(|id| self.methods.get(id))
                        .copied()
                }
                MroEntry::Module(module) => self.modules.method(*module, selector),
            };
            if let Some(method) = method {
                return Ok(method);
            }
        }
        Err(ClassError::MethodSlotNotFound { class, selector })
    }

    /// Removes only the current owner's local slot, allowing ancestors to resolve it.
    pub fn remove_method(&mut self, class: ClassId, selector: Selector) -> Result<(), ClassError> {
        self.require_meta_capability(class, crate::Capability::MethodSet)?;
        let mut candidate = self.open(class)?;
        candidate.remove_method(selector);
        self.publish(candidate)?;
        Ok(())
    }

    /// Installs a local tombstone that makes the selector absent despite ancestors.
    pub fn undef_method(&mut self, class: ClassId, selector: Selector) -> Result<(), ClassError> {
        self.require_meta_capability(class, crate::Capability::MethodSet)?;
        let mut candidate = self.open(class)?;
        candidate.undef_method(selector);
        self.publish(candidate)?;
        Ok(())
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
        self.require_meta_capability(class, capability)?;
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
        self.require_meta_capability(class, capability)?;
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
        self.require_meta_capability(class, capability)?;
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
                MroEntry::Class(owner) => {
                    let revision = self.active(*owner).map_err(DispatchError::Class)?;
                    if revision.tombstones().contains(&selector) {
                        return Ok(DispatchOutcome::WouldInvokeMethodMissing { selector });
                    }
                    revision
                        .methods()
                        .get(&selector)
                        .and_then(|id| self.methods.get(id))
                        .copied()
                }
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
            ) || matches!(
                context.lexical_module,
                Some(module) if self.module_has_private_access(receiver, module)
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

    fn module_has_private_access(&self, class: ClassId, module: ModuleId) -> bool {
        self.active(class).is_ok_and(|revision| {
            revision
                .composition_edges()
                .iter()
                .any(|edge| edge.module() == module && edge.private_access())
        })
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

    /// Binds a Method for one receiver under an explicit dispatch context.
    ///
    /// `IRIS-V1-CONTROL-C012` makes a top-level helper PRIVATE by default, so
    /// binding one from inside its own Module body needs the same privileged
    /// context an implicit send uses. The context-free `bind_instance` reports
    /// a visibility denial there.
    pub fn bind_instance_with_context(
        &mut self,
        receiver: crate::ObjectId,
        class: ClassId,
        selector: Selector,
        context: DispatchContext,
    ) -> Result<BoundMethod, DispatchError> {
        match self.dispatch_with_context(class, selector, context)? {
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

    /// Validates a retained Method against the receiver's current MRO before body entry.
    pub fn validate_method_binding(
        &self,
        receiver: ClassId,
        method: Method,
    ) -> Result<(), DispatchError> {
        let owner = match method.owner() {
            MethodOwner::Class(class) => MroEntry::Class(class),
            MethodOwner::Module(module) => MroEntry::Module(module),
        };
        self.active(receiver)
            .map_err(DispatchError::Class)?
            .mro()
            .contains(&owner)
            .then_some(())
            .ok_or(DispatchError::MethodBinding {
                selector: method.selector(),
            })
    }

    /// Validates a reflected Method before permitting its body to start.
    pub fn invoke_reflective<T, F>(
        &self,
        receiver: ClassId,
        method: Method,
        invoke: F,
    ) -> Result<T, DispatchError>
    where
        F: FnOnce(Method) -> Result<T, DispatchError>,
    {
        self.validate_method_binding(receiver, method)?;
        invoke(method)
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
        self.dispatch_super_selector(class, method, method.selector())
    }

    /// Resolves a selector after a Method's lexical owner in current receiver MRO.
    pub fn dispatch_super_selector(
        &self,
        class: ClassId,
        method: Method,
        selector: Selector,
    ) -> Result<Method, DispatchError> {
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
                MroEntry::Class(owner) => {
                    let revision = self.active(*owner).map_err(DispatchError::Class)?;
                    if revision.tombstones().contains(&selector) {
                        return Err(DispatchError::NoSuperMethod { selector });
                    }
                    revision
                        .methods()
                        .get(&selector)
                        .and_then(|id| self.methods.get(id))
                        .copied()
                }
                MroEntry::Module(module) => self.modules.method(*module, selector),
            };
            if let Some(successor) = successor {
                return Ok(successor);
            }
        }
        Err(DispatchError::NoSuperMethod { selector })
    }

    /// Resolves the same selector after a Class object's singleton Method owner.
    pub fn dispatch_class_object_super(
        &self,
        class: ClassId,
        method: Method,
    ) -> Result<Method, DispatchError> {
        self.dispatch_class_object_super_selector(class, method, method.selector())
    }

    /// Resolves a selector after a Class object's singleton Method owner.
    pub fn dispatch_class_object_super_selector(
        &self,
        class: ClassId,
        method: Method,
        selector: Selector,
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
                        .get(&selector)
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
        Err(DispatchError::NoSuperMethod { selector })
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
