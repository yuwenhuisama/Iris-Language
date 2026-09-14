use iris_runtime::decorator_protocol::DecoratorKind;
use iris_syntax::{ClassDeclaration, Decorator, MethodDeclaration, Statement};

use super::super::lowering::Declarations;
use super::CompileError;

pub(super) type MemberTarget<'a> = (Option<usize>, DecoratorKind, &'a Vec<Decorator>);

pub(super) fn method_target<'a>(
    method: &'a MethodDeclaration,
    owner: (&str, bool),
    declarations: Declarations<'_, '_>,
) -> Result<MemberTarget<'a>, CompileError> {
    if !method.decorators.is_empty()
        && (!matches!(
            method.kind,
            iris_syntax::MethodKind::Instance
                | iris_syntax::MethodKind::Property
                | iris_syntax::MethodKind::Class
        ) || (!method.type_parameters.is_empty()
            && (!matches!(
                method.kind,
                iris_syntax::MethodKind::Class | iris_syntax::MethodKind::Instance
            ) || (owner.1 && method.kind != iris_syntax::MethodKind::Instance)
                || method.impl_contract == Some(None))))
    {
        return Err(CompileError::new("decorator target outside instance slice"));
    }
    let function = declarations.signatures.iter().position(|signature| {
        signature
            .declaration
            .is_some_and(|known| std::ptr::eq(known, method))
            && signature.module == owner.0
    });
    let kind = match method.kind {
        iris_syntax::MethodKind::Property => DecoratorKind::Property,
        _ => DecoratorKind::Method,
    };
    Ok((function, kind, &method.decorators))
}

pub(super) fn class_targets<'a>(
    class: &'a ClassDeclaration,
    declarations: Declarations<'_, '_>,
) -> Result<Vec<MemberTarget<'a>>, CompileError> {
    let mut targets = vec![(None, DecoratorKind::Class, &class.decorators)];
    for statement in &class.body {
        if let Statement::Method(method) = statement {
            targets.push(method_target(
                method,
                (&class.name, !class.parameters.is_empty()),
                declarations,
            )?);
        }
        if let Statement::StoredProperty {
            decorators,
            class_level,
            name,
            ..
        } = statement
        {
            if *class_level && !decorators.is_empty() {
                return Err(CompileError::new("class-level stored property decorators"));
            }
            let function = declarations.signatures.iter().position(|signature| {
                signature.module == class.name
                    && signature.selector == name
                    && signature.declaration.is_some()
            });
            targets.push((function, DecoratorKind::Property, decorators));
        }
    }
    Ok(targets)
}
