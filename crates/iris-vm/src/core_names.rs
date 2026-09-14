pub(crate) const RECORDS: &[&str] = &[
    "Invocation",
    "InvocationSignature",
    "InvocationParameter",
    "ArgumentChanges",
    "DecoratorContext",
    "DecoratorProtocolError",
    "Plan",
    "Transformation",
];

pub(crate) const SUPPORT_TYPES: &[&str] = &[
    "Symbol",
    "Type",
    "Tuple",
    "ArgumentError",
    "TypeError",
    "MetaCapabilityError",
];

pub(crate) const DECORATOR_CONTRACTS: &[&str] = &[
    "ClassDecorator",
    "ModuleDecorator",
    "ContractDecorator",
    "MethodDecorator",
    "PropertyDecorator",
];

pub(crate) fn core_name(name: &str) -> Option<&str> {
    let name = name.strip_prefix("Kernel::").unwrap_or(name);
    (RECORDS.contains(&name) || SUPPORT_TYPES.contains(&name) || matches!(name, "Array" | "Hash"))
        .then_some(name)
}

pub(crate) fn requires_core(program: &crate::compile::Program) -> bool {
    use crate::compile::Instruction;
    program
        .instructions
        .iter()
        .chain(
            program
                .functions
                .iter()
                .flat_map(|function| &function.instructions),
        )
        .any(|instruction| match instruction {
            Instruction::AssertNonNil { .. } => true,
            Instruction::LoadBuiltinClass { name, .. }
            | Instruction::LoadBuiltinType { name, .. } => core_name(name).is_some(),
            Instruction::BuildType { expression, .. }
            | Instruction::CheckAnnotation {
                annotation: expression,
                ..
            }
            | Instruction::CheckReturn {
                annotation: expression,
                ..
            } => type_requires_core(expression),
            _ => false,
        })
}

fn type_requires_core(expression: &iris_syntax::TypeExpression) -> bool {
    use iris_syntax::TypeExpression;
    match expression {
        TypeExpression::Name(name) => core_name(name).is_some(),
        TypeExpression::Generic { name, arguments } => {
            core_name(name).is_some() || arguments.iter().any(type_requires_core)
        }
        TypeExpression::Union(members) | TypeExpression::Intersection(members) => {
            members.iter().any(type_requires_core)
        }
        TypeExpression::Function { parameters, result } => {
            parameters.iter().any(type_requires_core) || type_requires_core(result)
        }
        TypeExpression::Typeof(_) => false,
    }
}
