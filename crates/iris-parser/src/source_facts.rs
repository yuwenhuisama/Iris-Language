use crate::{Parser, source::*};
use iris_syntax::{Expression, Statement, Visibility};

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

    pub(super) fn primary_fact(&self, id: SyntaxId, expression: &Expression) -> ExpressionFact {
        let path = self.recorder.names(id);
        match expression {
            Expression::Name(text) if matches!(text.as_str(), "true" | "false" | "nil") => {
                ExpressionFact::Literal {
                    text: text.clone(),
                    kind: if text == "nil" {
                        LiteralKind::Nil
                    } else {
                        LiteralKind::Bool
                    },
                }
            }
            Expression::Name(_) => ExpressionFact::Name { path },
            Expression::Literal(text) => {
                let start = self.recorder.document.node(id).span.start;
                let index = self.tokens.partition_point(|token| token.offset < start);
                let kind = self.tokens.get(index).map(|token| token.kind);
                ExpressionFact::Literal {
                    text: text.clone(),
                    kind: literal_kind(kind, text),
                }
            }
            Expression::Symbol(text) => ExpressionFact::Literal {
                text: text.clone(),
                kind: LiteralKind::Symbol,
            },
            Expression::ClosedGeneric { .. } => ExpressionFact::Construction {
                path,
                type_arguments: self.type_children(id),
            },
            Expression::ReifiedType(_) => match self.type_children(id).first() {
                Some(annotation) => ExpressionFact::ReifiedType {
                    annotation: *annotation,
                },
                None => ExpressionFact::Unsupported {
                    form: "reified-type",
                },
            },
            Expression::Grouped(_) => match self.expression_children(id).first() {
                Some(value) => ExpressionFact::Grouped { value: *value },
                None => ExpressionFact::Unsupported { form: "grouped" },
            },
            Expression::Closure { .. } => {
                let parameters = self.recorder.children(id).iter().copied().filter(|child| matches!(&self.recorder.document.node(*child).kind, SourceKind::Declaration(value) if value.kind == DeclarationKind::Parameter)).collect();
                ExpressionFact::Closure {
                    parameters,
                    return_type: self.type_children(id).last().copied(),
                }
            }
            Expression::Array(_) => ExpressionFact::Unsupported { form: "array" },
            Expression::Tuple(_) => ExpressionFact::Unsupported { form: "tuple" },
            Expression::Hash(_) => ExpressionFact::Unsupported { form: "hash" },
            Expression::RawIvar(_) => ExpressionFact::Unsupported {
                form: "instance-variable",
            },
            Expression::ClassVar(_) => ExpressionFact::Unsupported {
                form: "class-variable",
            },
            Expression::GlobalVar(_) => ExpressionFact::Unsupported {
                form: "global-variable",
            },
            Expression::If { .. } => ExpressionFact::Unsupported { form: "if" },
            Expression::While { .. } => ExpressionFact::Unsupported { form: "while" },
            Expression::Try { .. } => ExpressionFact::Unsupported { form: "try" },
            Expression::Await(_) => ExpressionFact::Unsupported { form: "await" },
            Expression::Yield(_) => ExpressionFact::Unsupported { form: "yield" },
            Expression::Unary { .. } => ExpressionFact::Unsupported { form: "unary" },
            Expression::Binary { .. } => ExpressionFact::Unsupported { form: "binary" },
            Expression::Assignment { .. } => ExpressionFact::Unsupported { form: "assignment" },
            Expression::Member { .. } | Expression::ContractView { .. } => {
                ExpressionFact::Unsupported { form: "member" }
            }
            Expression::Call { .. } => ExpressionFact::Unsupported { form: "call" },
            Expression::Index { .. } => ExpressionFact::Unsupported { form: "index" },
            Expression::KeywordArgument { .. } => ExpressionFact::Unsupported {
                form: "keyword-argument",
            },
        }
    }
}

fn literal_kind(kind: Option<iris_lexer::TokenKind>, text: &str) -> LiteralKind {
    use iris_lexer::TokenKind;
    match kind {
        Some(TokenKind::StringLiteral) => LiteralKind::String,
        Some(TokenKind::MutableStringLiteral) => LiteralKind::MutableString,
        Some(TokenKind::BytesLiteral) => LiteralKind::Bytes,
        Some(TokenKind::ByteArrayLiteral) => LiteralKind::ByteArray,
        Some(TokenKind::RegexLiteral) => LiteralKind::Regex,
        _ if text == "nil" => LiteralKind::Nil,
        _ if matches!(text, "true" | "false") => LiteralKind::Bool,
        _ if text.contains('.')
            || text.ends_with("f32")
            || text.ends_with("f64")
            || (text.contains(['e', 'E'])
                && !text.starts_with("0x")
                && !text.starts_with("0X")) =>
        {
            LiteralKind::Float
        }
        _ => LiteralKind::Integer,
    }
}
