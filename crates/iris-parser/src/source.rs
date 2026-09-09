//! Source facts emitted by grammar productions, independently of the runtime AST.

pub use crate::source_headers::{DeclarationHeader, ImportSeparator, ImportSeparatorKind};
pub use crate::source_metadata::{
    ArgumentKind, ArgumentSlot, CallSite, Documentation, ParameterSlot, SignatureSite,
};
use iris_syntax::{MethodKind, ParameterCategory, TypeExpression, Visibility};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SyntaxId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScopeId(pub usize);

/// Half-open UTF-8 byte range in the original input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameSite {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceParse {
    pub parse: crate::ParseResult,
    pub source: SourceDocument,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SourceDocument {
    pub span: Span,
    pub nodes: Vec<SyntaxNode>,
    pub roots: Vec<SyntaxId>,
    pub scopes: Vec<Scope>,
    pub tokens: Vec<iris_lexer::Token>,
    pub protected: Vec<Span>,
    pub recovery: Vec<Recovery>,
    pub comments: Vec<iris_lexer::Comment>,
    pub documentation: Vec<Documentation>,
    pub parameter_slots: Vec<ParameterSlot>,
    pub signatures: Vec<SignatureSite>,
    pub calls: Vec<CallSite>,
}

impl SourceDocument {
    pub fn node(&self, id: SyntaxId) -> &SyntaxNode {
        &self.nodes[id.0]
    }

    pub fn scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxNode {
    pub id: SyntaxId,
    pub span: Span,
    pub scope: ScopeId,
    pub children: Vec<SyntaxId>,
    pub kind: SourceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    Document,
    Class,
    Module,
    Contract,
    TypeAlias,
    Method,
    Closure,
    Block,
    Loop,
    MatchArm,
    Catch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub owner: Option<SyntaxId>,
    pub kind: ScopeKind,
    pub span: Span,
    /// Resolution must not cross a damaged scope without treating it as uncertain.
    pub damaged: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recovery {
    pub span: Span,
    pub scope: ScopeId,
    pub code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceKind {
    Name(NameSite),
    Declaration(Box<SourceDeclaration>),
    Type(TypeExpression),
    Expression(ExpressionFact),
    Import(ImportFact),
    Export,
    Body,
    Statement,
    Pattern,
    Decorator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclarationKind {
    Class,
    Module,
    Contract,
    TypeAlias,
    Binding,
    Constant,
    Global,
    Shared,
    Property,
    Method,
    Parameter,
    TypeParameter,
    PatternBinding,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub mutable: bool,
    pub asynchronous: bool,
    pub reopen: bool,
    pub override_member: bool,
    pub implementation: bool,
    pub shared: bool,
    pub exported: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDeclaration {
    pub kind: DeclarationKind,
    pub name: NameSite,
    pub path: Vec<NameSite>,
    pub visibility: Visibility,
    pub surface: Option<MethodKind>,
    pub modifiers: Modifiers,
    /// Lexical declarations become visible only at this byte offset.
    pub visible_from: usize,
    pub annotation: Option<SyntaxId>,
    pub initializer: Option<SyntaxId>,
    pub parameters: Vec<SyntaxId>,
    pub return_type: Option<SyntaxId>,
    pub return_hint_offset: Option<usize>,
    pub header: Option<DeclarationHeader>,
    pub parameter_category: Option<ParameterCategory>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportFact {
    pub target: Vec<NameSite>,
    pub separators: Vec<ImportSeparator>,
    pub alias: Option<NameSite>,
    pub specs: Vec<ImportSite>,
    pub replacement_authorized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportSite {
    pub name: NameSite,
    pub alias: Option<NameSite>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionFact {
    Name {
        path: Vec<NameSite>,
    },
    Literal {
        text: String,
        kind: LiteralKind,
    },
    Array {
        elements: Vec<SyntaxId>,
    },
    Tuple {
        elements: Vec<SyntaxId>,
    },
    Hash {
        entries: Vec<(SyntaxId, SyntaxId)>,
    },
    Range {
        start: SyntaxId,
        end: SyntaxId,
        operator: RangeOperator,
        operator_span: Span,
    },
    Member {
        receiver: SyntaxId,
        name: NameSite,
        contract: bool,
    },
    IncompleteMember {
        receiver: SyntaxId,
        dot: Span,
    },
    Call {
        callee: SyntaxId,
        arguments: Vec<SyntaxId>,
        type_arguments: Vec<SyntaxId>,
    },
    Assignment {
        target: SyntaxId,
        value: SyntaxId,
        operator: iris_syntax::AssignmentOperator,
    },
    Grouped {
        value: SyntaxId,
    },
    Construction {
        path: Vec<NameSite>,
        type_arguments: Vec<SyntaxId>,
    },
    ReifiedType {
        annotation: SyntaxId,
    },
    Closure {
        parameters: Vec<SyntaxId>,
        return_type: Option<SyntaxId>,
    },
    KeywordArgument {
        name: NameSite,
        value: SyntaxId,
    },
    /// No inferred result is promised. The node still retains production children.
    Unsupported {
        form: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeOperator {
    Inclusive,
    Exclusive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteralKind {
    Integer,
    Float,
    String,
    MutableString,
    Bytes,
    ByteArray,
    Regex,
    Symbol,
    Bool,
    Nil,
}
