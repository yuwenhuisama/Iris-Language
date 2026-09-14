use super::{Binding, EvaluationError, SourceEvaluator, meta_capabilities};
use iris_runtime::{
    ClassId, CompositionEdge, Method, MethodOwner, MroEntry, Selector, StaticSpine, Value,
};
use iris_syntax::{
    ClassDeclaration, Expression, MethodDeclaration, MethodKind, Statement, TypeExpression,
};

impl SourceEvaluator {
    pub(super) fn decorated_origin(
        &mut self,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        let superclass = match &declaration.extends {
            Some(TypeExpression::Name(name)) => {
                Some(self.class_name(name)?.ok_or(EvaluationError::NameError)?)
            }
            None => self.class_name("Object")?,
            Some(_) => return Err(EvaluationError::UnsupportedConstruct),
        };
        let mut edges = Vec::new();
        let mut class_mixins = Vec::new();
        for mixin in &declaration.mixins {
            let name = match &mixin.target {
                TypeExpression::Name(name) | TypeExpression::Generic { name, .. } => name,
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            if let Some(module) = self.module_names.get(name) {
                edges.push(CompositionEdge::new(*module, mixin.private_access));
            } else {
                class_mixins.push(self.class_name(name)?.ok_or(EvaluationError::NameError)?);
            }
        }
        let mut capabilities = meta_capabilities(&declaration.meta_deny)?;
        let mut contracts = Vec::new();
        for target in &declaration.implements {
            let (TypeExpression::Name(name) | TypeExpression::Generic { name, .. }) = target else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let contract = *self
                .contract_names
                .get(name)
                .ok_or(EvaluationError::NameError)?;
            if let Some(policy) = self.contract_capabilities.get(&contract) {
                capabilities = capabilities.narrowed_by(*policy);
            }
            contracts.push(contract);
        }
        self.validate_module_overrides(superclass, &edges)?;
        let class = self
            .runtime
            .registry_mut()
            .stage_class_origin(
                StaticSpine::new(1).with_meta_capabilities(capabilities),
                superclass,
                &edges,
            )
            .map_err(EvaluationError::Class)?;
        self.origin_names.insert(class, declaration.name.clone());
        self.origin_bodies.insert(class, Vec::new());
        self.package_contexts
            .classes
            .insert(class, (self.package.clone(), self.api_major));
        self.static_superclasses.insert(class, superclass);
        self.class_mixins.insert(class, class_mixins);
        self.class_contracts.insert(class, contracts);
        if !declaration.parameters.is_empty() {
            self.generic_definitions.push(class);
        }
        let metadata = self.decorator_metadata.len();
        let outcome = (|| {
            self.record_qualified_contracts(class, declaration)?;
            let body = self.register_body(MethodDeclaration {
                decorators: Vec::new(),
                is_async: false,
                is_override: false,
                impl_contract: None,
                kind: MethodKind::Instance,
                selector: "to_bool".into(),
                type_parameters: Vec::new(),
                parameters: Vec::new(),
                return_type: None,
                visibility: iris_syntax::Visibility::Public,
                body: Some(vec![Statement::Expression(Expression::Literal(
                    "true".into(),
                ))]),
            });
            let selector = self.selector("to_bool");
            self.track_origin_body(class, body);
            self.runtime
                .registry_mut()
                .publish_origin_method(class, selector, body, iris_runtime::Visibility::Public)
                .map_err(EvaluationError::Class)?;
            self.class_body(class, false, declaration)?;
            self.validate_candidate_contracts(class)?;
            self.runtime
                .commit_declaration_group()
                .map_err(EvaluationError::Class)?;
            Ok(())
        })();
        self.origin_names.remove(&class);
        let bodies = self.origin_bodies.remove(&class).unwrap_or_default();
        self.singleton_declarations
            .retain(|(owner, _), _| *owner != class);
        self.singleton_identities
            .retain(|(owner, _), _| *owner != class);
        match outcome {
            Ok(()) => {
                self.names.insert(
                    declaration.name.clone(),
                    Binding::immutable(Value::Class(class)),
                );
                if !declaration.constraints.is_empty() {
                    self.generic_bounds.insert(
                        declaration.name.clone(),
                        (
                            declaration.parameters.clone(),
                            declaration.constraints.clone(),
                        ),
                    );
                }
                self.decorator_definitions.push(declaration.clone());
                Ok(())
            }
            Err(error) => {
                self.bodies.retain(|identity, _| !bodies.contains(identity));
                self.runtime.roll_back_group();
                self.package_contexts.classes.remove(&class);
                self.static_superclasses.remove(&class);
                self.class_mixins.remove(&class);
                self.class_contracts.remove(&class);
                self.qualified_contracts
                    .owners
                    .retain(|(owner, _), _| *owner != class);
                self.generic_definitions.retain(|owner| *owner != class);
                self.property_types.retain(|(owner, _), _| *owner != class);
                self.class_property_accessors
                    .retain(|(owner, _), _| *owner != class);
                self.dynamic_members.retain(|(owner, _)| *owner != class);
                self.qualified_methods
                    .retain(|(owner, _, _), _| *owner != class);
                self.class_level_properties.remove(&class);
                self.lazy_class_properties
                    .retain(|(owner, _), _| *owner != class);
                self.shared_class_properties.remove(&class);
                self.pending_class_properties.remove(&class);
                self.wrapper_chains
                    .retain(|_, chain| chain.original.owner() != MethodOwner::Class(class));
                self.property_methods.retain(|identity, _| {
                    self.runtime
                        .registry()
                        .method_by_id(*identity)
                        .is_some_and(|method| method.owner() != MethodOwner::Class(class))
                });
                self.decorator_metadata.truncate(metadata);
                Err(error)
            }
        }
    }

    pub(super) fn track_origin_body(&mut self, class: ClassId, body: iris_runtime::MethodBody) {
        if let Some(bodies) = self.origin_bodies.get_mut(&class) {
            bodies.push(body.raw());
        }
    }

    pub(super) fn origin_method(&self, class: ClassId, selector: Selector) -> Option<Method> {
        let registry = self.runtime.registry();
        let origin = registry.staged_origin(class).ok()?;
        for entry in origin.mro() {
            let method = match entry {
                MroEntry::Class(owner) if *owner == class => registry
                    .staged_method(class, selector)
                    .and_then(|identity| registry.method_by_id(identity)),
                MroEntry::Class(owner) => registry
                    .resolve_local_or_ancestor_method(*owner, selector)
                    .ok(),
                MroEntry::Module(module) => registry.module_method(*module, selector),
            };
            if method.is_some() {
                return method;
            }
        }
        None
    }
}
