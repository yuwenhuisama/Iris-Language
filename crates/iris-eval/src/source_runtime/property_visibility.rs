use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, ConstructionError, DispatchError, Method, MethodOwner, Value};
use iris_syntax::{
    ClassDeclaration, PropertyAccessor, PropertyAccessorKind, Statement, Visibility,
};

impl SourceEvaluator {
    pub(super) fn declare_class_property(
        &mut self,
        (class, declaration): (ClassId, &ClassDeclaration),
        statement: &Statement,
    ) -> Result<(), EvaluationError> {
        let Statement::StoredProperty {
            name,
            initializer,
            shared,
            ..
        } = statement
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        self.record_class_property_accessors(class, statement)?;
        if *shared {
            self.shared_class_properties
                .entry(class)
                .or_default()
                .push(name.clone());
        }
        if !*shared && !declaration.parameters.is_empty() {
            self.pending_class_properties
                .entry(class)
                .or_default()
                .push((name.clone(), initializer.clone()));
            self.class_level_properties.entry(class).or_default();
            Ok(())
        } else if !*shared
            && !declaration.reopen
            && (self.decorator_planning
                || (declaration.decorators.is_empty()
                    && !declaration.body.iter().any(|statement| match statement {
                        Statement::Method(method) => !method.decorators.is_empty(),
                        Statement::StoredProperty { decorators, .. } => !decorators.is_empty(),
                        _ => false,
                    })))
        {
            let slot = self.selector(name);
            let context = self.wrapper_context();
            self.lazy_class_properties
                .insert((class, slot), (initializer.clone(), context));
            self.class_level_properties
                .entry(class)
                .or_default()
                .push(slot);
            Ok(())
        } else {
            self.class_level_property(class, name, initializer.clone())
        }
    }

    pub(super) fn record_class_property_accessors(
        &mut self,
        class: ClassId,
        statement: &Statement,
    ) -> Result<(), EvaluationError> {
        let Statement::StoredProperty {
            name,
            visibility,
            accessors,
            ..
        } = statement
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let members = match accessors {
            Some(accessors) => accessors.members.clone(),
            None => vec![
                PropertyAccessor {
                    kind: PropertyAccessorKind::Get,
                    visibility: *visibility,
                },
                PropertyAccessor {
                    kind: PropertyAccessorKind::Set,
                    visibility: *visibility,
                },
            ],
        };
        self.class_property_accessors
            .insert((class, name.clone()), members);
        Ok(())
    }

    pub(super) fn check_class_property_access(
        &mut self,
        class: ClassId,
        name: &str,
        writing: bool,
    ) -> Result<(), EvaluationError> {
        let Some(accessors) = self.class_property_accessors.get(&(class, name.into())) else {
            return Ok(());
        };
        let kind = if writing {
            PropertyAccessorKind::Set
        } else {
            PropertyAccessorKind::Get
        };
        let Some(accessor) = accessors.iter().find(|accessor| accessor.kind == kind) else {
            return Err(EvaluationError::MessageNotFound {
                receiver_class: "Class".into(),
                selector: if writing {
                    format!("{name}=")
                } else {
                    name.into()
                },
            });
        };
        let authorized = match accessor.visibility {
            Visibility::Public => true,
            Visibility::Private => self.lexical_class == Some(class),
            Visibility::Protected => match self.lexical_class {
                Some(owner) => self.subtype(owner, class)? == Value::Bool(true),
                None => false,
            },
        };
        if authorized {
            return Ok(());
        }
        let selector = self.selector(&if writing {
            format!("{name}=")
        } else {
            name.into()
        });
        Err(EvaluationError::Construction(ConstructionError::Dispatch(
            DispatchError::VisibilityDenied { selector },
        )))
    }

    pub(super) fn invoke_property_with_owner(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        if self.property_methods.get(&method.id()) != Some(&true) {
            return self.invoke_method_body(method, receiver, arguments);
        }
        let (MethodOwner::Class(owner), Value::Object(object)) = (method.owner(), &receiver) else {
            return self.invoke_method_body(method, receiver, arguments);
        };
        let owner_arguments = self
            .runtime
            .type_arguments_of(*object)
            .map_err(EvaluationError::Construction)?;
        let bindings = self
            .wrapper_owner_parameters
            .get(&owner)
            .into_iter()
            .flatten()
            .zip(owner_arguments)
            .map(|(name, argument)| Ok((name.clone(), self.wrapper_nominal_annotation(argument)?)))
            .collect::<Result<Vec<_>, EvaluationError>>()?;
        let previous = self.method_type_bindings.clone();
        self.method_type_bindings.extend(bindings);
        let result = self.invoke_method_body(method, receiver, arguments);
        self.method_type_bindings = previous;
        result
    }
}
