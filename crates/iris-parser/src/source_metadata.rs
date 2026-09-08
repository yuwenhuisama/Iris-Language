use crate::source::{NameSite, Span, SyntaxId};
use iris_syntax::ParameterCategory;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Documentation {
    pub declaration: SyntaxId,
    pub span: Span,
    pub text: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterSlot {
    pub owner: SyntaxId,
    pub span: Span,
    pub name: NameSite,
    pub category: ParameterCategory,
    pub declaration: Option<SyntaxId>,
    pub annotation: Option<SyntaxId>,
    pub default: Option<SyntaxId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureSite {
    pub owner: SyntaxId,
    pub span: Span,
    pub return_type: Option<SyntaxId>,
    /// The entire header parsed without errors, including checks after its closing delimiter.
    /// Captured before the body; independent of diagnostic positions and body recovery.
    pub valid: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallSite {
    pub call: SyntaxId,
    pub open: Span,
    pub close: Option<Span>,
    pub commas: Vec<Span>,
    pub arguments: Vec<ArgumentSlot>,
    pub end: usize,
    pub incomplete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgumentSlot {
    pub span: Span,
    pub kind: ArgumentKind,
    pub expression: Option<SyntaxId>,
    pub incomplete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentKind {
    Positional,
    Keyword(NameSite),
    TrailingBlock,
}
