use iris_syntax::{
    AssignmentOperator, Declaration, Expression, MethodDeclaration, MethodKind, Parameter,
    ParameterCategory, PropertyAccessor, PropertyAccessorKind, Statement,
};

pub(super) fn prepare(declarations: &mut [Declaration]) {
    for declaration in declarations {
        let Declaration::Class(class) = declaration else {
            continue;
        };
        let mut generated = Vec::new();
        for statement in &class.body {
            let Statement::StoredProperty {
                class_level: false,
                name,
                annotation,
                accessors,
                visibility,
                ..
            } = statement
            else {
                continue;
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
                if class.body.iter().any(|statement| matches!(statement, Statement::Method(method) if method.kind == MethodKind::Property && method.selector == selector)) {
                    continue;
                }
                generated.push(Statement::Method(MethodDeclaration {
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
                }));
            }
        }
        class.body.extend(generated);
    }
}
