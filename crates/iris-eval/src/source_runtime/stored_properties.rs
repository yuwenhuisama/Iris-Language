use super::{EvaluationError, SourceEvaluator};
use iris_runtime::ClassId;
use iris_syntax::{
    AssignmentOperator, Expression, MethodDeclaration, MethodKind, Parameter, ParameterCategory,
    PropertyAccessor, PropertyAccessorKind, Statement, Visibility,
};

impl SourceEvaluator {
    pub(super) fn install_dynamic_property_getter(
        &mut self,
        class: ClassId,
        name: &str,
    ) -> Result<(), EvaluationError> {
        let declaration = MethodDeclaration {
            decorators: Vec::new(),
            is_async: false,
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: name.into(),
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            visibility: Visibility::Public,
            body: Some(vec![Statement::Expression(Expression::RawIvar(format!(
                "@{name}"
            )))]),
        };
        let body = self.register_body(declaration);
        let selector = self.selector(name);
        let method = self
            .runtime
            .registry_mut()
            .publish_origin_method(class, selector, body, iris_runtime::Visibility::Public)
            .map_err(EvaluationError::Class)?;
        self.property_methods.insert(method.id(), true);
        Ok(())
    }

    pub(super) fn method_decorator_metadata(
        &mut self,
        class: ClassId,
        method: &MethodDeclaration,
    ) -> Result<iris_runtime::Value, EvaluationError> {
        use iris_runtime::Value;
        if method.kind != MethodKind::Property || method.body.is_some() {
            if self.generic_definitions.contains(&class) {
                return self.open_owner_method_metadata(class, method);
            }
            return self.owned_method_metadata(Value::Type(class, Vec::new()), method);
        }
        let annotation = method
            .return_type
            .as_ref()
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let property_type = self.wrapper_type_value(annotation)?;
        self.metadata_record(vec![
            ("name", Value::Symbol(method.selector.clone())),
            ("type", property_type),
            ("owner", Value::Type(class, Vec::new())),
        ])
    }

    pub(super) fn stored_property(
        &mut self,
        class: ClassId,
        statement: &Statement,
    ) -> Result<(), EvaluationError> {
        let Statement::StoredProperty {
            decorators,
            name,
            annotation,
            class_level: false,
            ..
        } = statement
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let (methods, initializer) = stored_property_bodies(statement)?;
        let origin = self.origin_names.contains_key(&class);
        let builtin = self.is_builtin_class(class);
        for method in methods {
            self.class_method_as(class, builtin, false, &method, origin)?;
        }
        let body = self.register_body(initializer);
        self.track_origin_body(class, body);
        let property = self.selector(&format!("@{name}"));
        self.property_types
            .insert((class, property), annotation.clone());
        let decorators = self.decorator_transforms(decorators);
        if origin {
            self.runtime
                .registry_mut()
                .stage_origin_property(class, property, body)
                .map_err(EvaluationError::Class)?;
            self.runtime
                .registry_mut()
                .stage_decorators(class, decorators)
                .map_err(EvaluationError::Class)
        } else {
            self.runtime
                .registry_mut()
                .publish_decorated_stored_property(class, property, body, decorators)
                .map_err(EvaluationError::Class)
        }
    }
}

pub(super) fn stored_property_bodies(
    statement: &Statement,
) -> Result<(Vec<MethodDeclaration>, MethodDeclaration), EvaluationError> {
    let Statement::StoredProperty {
        name,
        annotation,
        initializer,
        accessors,
        visibility,
        class_level: false,
        ..
    } = statement
    else {
        return Err(EvaluationError::UnsupportedConstruct);
    };
    let defaults = [
        PropertyAccessor {
            kind: PropertyAccessorKind::Get,
            visibility: *visibility,
        },
        PropertyAccessor {
            kind: PropertyAccessorKind::Set,
            visibility: *visibility,
        },
    ];
    let members = accessors.as_ref().map_or(defaults.as_slice(), |accessors| {
        accessors.members.as_slice()
    });
    let mut methods = Vec::new();
    for accessor in members {
        let (selector, parameters, expression) = match accessor.kind {
            PropertyAccessorKind::Get => (
                name.clone(),
                Vec::new(),
                Expression::RawIvar(format!("@{name}")),
            ),
            PropertyAccessorKind::Set => (
                format!("{name}="),
                vec![Parameter {
                    name: "value".into(),
                    category: ParameterCategory::Positional,
                    annotation: Some(annotation.clone()),
                    default: None,
                }],
                Expression::Assignment {
                    left: Box::new(Expression::RawIvar(format!("@{name}"))),
                    operator: AssignmentOperator::Assign,
                    right: Box::new(Expression::Name("value".into())),
                },
            ),
        };
        let method = MethodDeclaration {
            decorators: Vec::new(),
            is_async: false,
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector,
            type_parameters: Vec::new(),
            parameters,
            return_type: Some(annotation.clone()),
            visibility: accessor.visibility,
            body: Some(vec![Statement::Expression(expression)]),
        };
        methods.push(method);
    }
    let initializer = MethodDeclaration {
        decorators: Vec::new(),
        is_async: false,
        is_override: false,
        impl_contract: None,
        kind: MethodKind::Property,
        selector: name.clone(),
        type_parameters: Vec::new(),
        parameters: Vec::new(),
        return_type: None,
        visibility: Visibility::Private,
        body: Some(vec![Statement::Expression(Expression::Assignment {
            left: Box::new(Expression::RawIvar(format!("@{name}"))),
            operator: AssignmentOperator::Assign,
            right: Box::new(initializer.clone()),
        })]),
    };
    Ok((methods, initializer))
}
