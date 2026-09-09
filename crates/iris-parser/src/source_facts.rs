use crate::{Parser, source::*};
use iris_syntax::{Statement, Visibility};

mod binary;
mod primary;
pub(super) use binary::binary_fact;

impl Parser {
    pub(super) fn source_declaration(
        &self,
        id: SyntaxId,
        kind: DeclarationKind,
    ) -> Option<SourceDeclaration> {
        let path = self.recorder.names(id);
        let name = path.last()?.clone();
        if name.text == "_" {
            return None;
        }
        let mut declaration = SourceDeclaration {
            kind,
            name,
            path,
            visibility: Visibility::Private,
            surface: None,
            modifiers: Modifiers::default(),
            visible_from: self.consumed_end,
            annotation: None,
            initializer: None,
            parameters: Vec::new(),
            return_type: None,
            return_hint_offset: None,
            header: None,
            parameter_category: None,
        };
        let (has_annotation, has_initializer) = match kind {
            DeclarationKind::Binding
            | DeclarationKind::Constant
            | DeclarationKind::Global
            | DeclarationKind::Shared
            | DeclarationKind::Property
            | DeclarationKind::Parameter => (true, true),
            DeclarationKind::TypeAlias
            | DeclarationKind::Method
            | DeclarationKind::PatternBinding => (true, false),
            DeclarationKind::Class
            | DeclarationKind::Module
            | DeclarationKind::Contract
            | DeclarationKind::TypeParameter => (false, false),
        };
        for child in self.recorder.children(id) {
            match &self.recorder.document.node(*child).kind {
                SourceKind::Type(_) if has_annotation => declaration.annotation = Some(*child),
                SourceKind::Expression(_) if has_initializer => {
                    declaration.initializer = Some(*child)
                }
                SourceKind::Declaration(parameter)
                    if parameter.kind == DeclarationKind::Parameter =>
                {
                    declaration.parameters.push(*child)
                }
                _ => {}
            }
        }
        Some(declaration)
    }

    pub(super) fn statement_fact(&self, id: SyntaxId, statement: &Statement) -> SourceKind {
        if self.recorder.enabled
            && let SourceKind::Declaration(value) = &self.recorder.document.node(id).kind
        {
            return SourceKind::Declaration(value.clone());
        }
        let kind = match statement {
            Statement::Binding { constant: true, .. } => DeclarationKind::Constant,
            Statement::Binding { .. } | Statement::DeferredBinding { .. } => {
                DeclarationKind::Binding
            }
            Statement::GlobalBinding { .. } => DeclarationKind::Global,
            Statement::SharedBinding { .. } => DeclarationKind::Shared,
            Statement::StoredProperty { .. } => DeclarationKind::Property,
            Statement::Method(_) => DeclarationKind::Method,
            _ => return SourceKind::Statement,
        };
        let Some(mut declaration) = self.source_declaration(id, kind) else {
            return SourceKind::Statement;
        };
        match statement {
            Statement::Binding { mutable, .. }
            | Statement::DeferredBinding { mutable, .. }
            | Statement::GlobalBinding { mutable, .. }
            | Statement::SharedBinding { mutable, .. } => declaration.modifiers.mutable = *mutable,
            Statement::StoredProperty {
                shared,
                class_level,
                ..
            } => {
                declaration.modifiers.shared = *shared;
                declaration.surface = Some(if *class_level {
                    iris_syntax::MethodKind::Class
                } else {
                    iris_syntax::MethodKind::Instance
                });
            }
            Statement::Method(method) => {
                declaration.visibility = method.visibility;
                declaration.surface = Some(method.kind);
                declaration.modifiers.asynchronous = method.is_async;
                declaration.modifiers.override_member = method.is_override;
                declaration.modifiers.implementation = method.impl_contract.is_some();
                declaration.return_type = declaration.annotation.take();
                declaration.initializer = None;
                declaration.visible_from = 0;
            }
            _ => {}
        }
        SourceKind::Declaration(Box::new(declaration))
    }

    pub(super) fn expression_children(&self, id: SyntaxId) -> Vec<SyntaxId> {
        self.recorder
            .children(id)
            .iter()
            .copied()
            .filter(|child| {
                matches!(
                    self.recorder.document.node(*child).kind,
                    SourceKind::Expression(_)
                )
            })
            .collect()
    }

    pub(super) fn type_children(&self, id: SyntaxId) -> Vec<SyntaxId> {
        self.recorder
            .children(id)
            .iter()
            .copied()
            .filter(|child| {
                matches!(
                    self.recorder.document.node(*child).kind,
                    SourceKind::Type(_)
                )
            })
            .collect()
    }
}
