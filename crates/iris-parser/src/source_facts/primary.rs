use crate::{Parser, source::*};
use iris_syntax::Expression;

impl Parser {
    pub(crate) fn primary_fact(&self, id: SyntaxId, expression: &Expression) -> ExpressionFact {
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
            Expression::Array(_) => ExpressionFact::Array {
                elements: self.expression_children(id),
            },
            Expression::Tuple(_) => ExpressionFact::Tuple {
                elements: self.expression_children(id),
            },
            Expression::Hash(_) => ExpressionFact::Hash {
                entries: self
                    .expression_children(id)
                    .chunks_exact(2)
                    .map(|entry| (entry[0], entry[1]))
                    .collect(),
            },
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
