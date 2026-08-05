//! Recursive-descent declarations plus Pratt expressions for Iris v1.

mod expression;

#[cfg(test)]
mod expression_tests;

use iris_lexer::{TokenKind, lex};
mod analysis;

pub use analysis::analyze;

use iris_syntax::{
    CatchBinding, CatchClause, ClassDeclaration, Constraint, ContractDeclaration, Declaration,
    Decorator, Expression, MatchArm, MatchBody, MethodDeclaration, MethodKind, MixinEntry,
    ModuleDeclaration, Parameter, ParameterCategory, Pattern, Program, ProgramEntry, Raise,
    Statement, TypeExpression, Visibility,
};

/// The three clauses of a `try`, shared by its statement and expression forms.
type TryParts = (Vec<Statement>, Vec<CatchClause>, Option<Vec<Statement>>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
    pub program_accepted: bool,
}

impl ParseResult {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub fn parse(source: &str) -> ParseResult {
    let lexed = lex(source.as_bytes());
    if let Some(diagnostic) = lexed.diagnostics().first() {
        return ParseResult {
            program: Program::default(),
            diagnostics: vec![Diagnostic {
                code: diagnostic.code(),
            }],
            program_accepted: false,
        };
    }
    let lexer_tokens = lexed.tokens();
    let mut raw = Vec::new();
    let mut cursor = 0;
    while cursor < lexer_tokens.len() {
        let token = lexer_tokens[cursor];
        if token.kind == TokenKind::Newline {
            raw.push((token.kind, "\n", token.offset.0));
            cursor += 1;
            continue;
        }
        let start = token.offset.0;
        let end = if token.kind == TokenKind::SourceCharacter
            && source.as_bytes().get(start).is_some_and(u8::is_ascii_digit)
        {
            numeric_end(source, start)
        } else {
            token_end(source, start, token.kind)
        };
        let text = source.get(start..end).unwrap_or_default().trim();
        if !text.is_empty() {
            raw.push((token.kind, text, start));
        }
        cursor += 1;
        while lexer_tokens
            .get(cursor)
            .is_some_and(|next| next.offset.0 < end)
        {
            cursor += 1;
        }
    }
    let tokens = combine_numeric_literals(&raw);
    let mut parser = Parser {
        tokens,
        cursor: 0,
        diagnostics: Vec::new(),
        no_trailing_block: false,
        no_type_union: false,
        empty_closure_header: false,
    };
    let program = parser.program();
    let program_accepted = parser.diagnostics.is_empty() && parser.at_end();
    if !parser.at_end() && parser.diagnostics.is_empty() {
        parser.error("PARSE_UNEXPECTED_TOKEN");
    }
    ParseResult {
        program,
        diagnostics: parser.diagnostics,
        program_accepted,
    }
}

fn token_end(source: &str, start: usize, kind: TokenKind) -> usize {
    let remaining = source.get(start..).unwrap_or_default();
    let width = match kind {
        TokenKind::RangeInclusive | TokenKind::RangeExclusive => 3,
        TokenKind::ContractView
        | TokenKind::BangEqual
        | TokenKind::MatchTilde
        | TokenKind::NotMatchTilde
        | TokenKind::LessEqual
        | TokenKind::GreaterEqual
        | TokenKind::LeftShift
        | TokenKind::RightShift
        | TokenKind::StarStar
        | TokenKind::EqualEqual
        | TokenKind::AndAnd
        | TokenKind::PipePipe
        | TokenKind::MatchArrow
        | TokenKind::PlusEqual
        | TokenKind::MinusEqual
        | TokenKind::StarEqual
        | TokenKind::SlashEqual
        | TokenKind::AmpEqual
        | TokenKind::PipeEqual
        | TokenKind::CaretEqual
        | TokenKind::PercentEqual => 2,
        TokenKind::Spaceship
        | TokenKind::StarStarEqual
        | TokenKind::LeftShiftEqual
        | TokenKind::RightShiftEqual
        | TokenKind::AndAndEqual
        | TokenKind::PipePipeEqual => 3,
        TokenKind::Identifier | TokenKind::Keyword | TokenKind::SetterSelector => remaining
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count(),
        TokenKind::StringLiteral
        | TokenKind::MutableStringLiteral
        | TokenKind::BytesLiteral
        | TokenKind::ByteArrayLiteral
        | TokenKind::RegexLiteral => literal_end(remaining),
        TokenKind::Newline => usize::from(remaining.starts_with("\r\n")) + 1,
        TokenKind::HashOpen => 2,
        TokenKind::SourceCharacter
        | TokenKind::LessThan
        | TokenKind::GreaterThan
        | TokenKind::Slash
        | TokenKind::LeftParen
        | TokenKind::RightParen
        | TokenKind::LeftBrace
        | TokenKind::RightBrace
        | TokenKind::Colon
        | TokenKind::Semicolon
        | TokenKind::Dot
        | TokenKind::At => 1,
        TokenKind::DoubleAt => 2,
    };
    start + width
}

fn numeric_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let hexadecimal = bytes
        .get(start..start + 2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"0x"));
    let mut end = start;
    while let Some(byte) = bytes.get(end) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'-') {
            end += 1;
            continue;
        }
        if *byte == b'.'
            && (hexadecimal || matches!(bytes.get(end + 1), Some(next) if next.is_ascii_digit()))
        {
            end += 1;
            continue;
        }
        break;
    }
    end
}

fn literal_end(remaining: &str) -> usize {
    let quote = remaining
        .bytes()
        .position(|byte| matches!(byte, b'\'' | b'"'))
        .unwrap_or(0);
    let delimiter = remaining.as_bytes().get(quote).copied().unwrap_or(b'"');
    let mut cursor = quote + 1;
    while let Some(byte) = remaining.as_bytes().get(cursor) {
        if *byte == b'\\' {
            cursor += 2;
            continue;
        }
        cursor += 1;
        if *byte == delimiter {
            return cursor;
        }
    }
    remaining.len()
}

#[derive(Clone, Debug)]
struct Token {
    text: String,
    /// Byte offset of this token in the source.
    ///
    /// `IRIS-V1-CONTROL-C079` defines `SourceLocation` with a one-based line
    /// and column, so the offset is retained here and converted at the point a
    /// location is built rather than being discarded during tokenization.
    offset: usize,
}

fn combine_numeric_literals(raw: &[(TokenKind, &str, usize)]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        let (kind, text, offset) = raw[cursor];
        if text == ":" && raw.get(cursor + 1).is_some_and(|(_, next, _)| *next == ":") {
            tokens.push(Token {
                text: "::".into(),
                offset,
            });
            cursor += 2;
        } else if text == "as" && raw.get(cursor + 1).is_some_and(|(_, next, _)| *next == "?") {
            // `as?` is one operator in the chapter 02 precedence table, but `?`
            // is a separate source character, so the two are joined here rather
            // than leaving `as` to bind and the `?` to dangle.
            tokens.push(Token {
                text: "as?".into(),
                offset,
            });
            cursor += 2;
        } else if kind == TokenKind::SourceCharacter && text.as_bytes()[0].is_ascii_digit() {
            let mut value = String::from(text);
            cursor += 1;
            while raw.get(cursor).is_some_and(|(next_kind, next, _)| {
                *next_kind == TokenKind::SourceCharacter && next.as_bytes()[0].is_ascii_digit()
            }) {
                value.push_str(raw[cursor].1);
                cursor += 1;
            }
            tokens.push(Token {
                text: value,
                offset,
            });
        } else {
            tokens.push(Token {
                text: text.into(),
                offset,
            });
            cursor += 1;
        }
    }
    tokens
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
    /// Suppresses `trailing_block` while parsing an `if` condition.
    ///
    /// Both a trailing block and an `if` branch body open with `{`, so in
    /// `if cond() { ... }` the brace belongs to the conditional. The chapter 02
    /// grammar resolves this by position, and this flag carries that position
    /// down through the expression parser.
    pub(crate) no_trailing_block: bool,
    /// Suppresses the `type_union` level while parsing a closure header.
    ///
    /// `|` both separates union members and CLOSES a closure parameter list, so
    /// inside `{ |x: Integer| ... }` the closing bar must not be read as a
    /// union operator.
    no_type_union: bool,
    /// Set when a `||` token was rewritten into an empty closure header.
    empty_closure_header: bool,
}

impl Parser {
    fn program(&mut self) -> Program {
        let mut program = Program::default();
        while !self.at_end() {
            if self.consume(";") {
                self.error(if self.cursor == 1 {
                    "PARSE_LEGACY_LEADING_SEMICOLON"
                } else {
                    "PARSE_EMPTY_STATEMENT"
                });
                continue;
            }
            if self.consume("\n") {
                continue;
            }
            let decorators = self.decorators();
            match self.peek() {
                Some("open") if self.peek_next() == Some("contract") => {
                    self.contract_declaration(decorators).map(|value| {
                        let declaration = Declaration::Contract(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                Some("open") if self.peek_next() == Some("module") => {
                    self.module_declaration(decorators).map(|value| {
                        let declaration = Declaration::Module(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                Some("open") if self.peek_next() == Some("class") => {
                    self.class_declaration(decorators).map(|value| {
                        let declaration = Declaration::Class(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                Some("class") => self.class_declaration(decorators).map(|value| {
                    let declaration = Declaration::Class(value);
                    program.declarations.push(declaration.clone());
                    program.entries.push(ProgramEntry::Declaration(declaration));
                }),
                Some("module") => self.module_declaration(decorators).map(|value| {
                    let declaration = Declaration::Module(value);
                    program.declarations.push(declaration.clone());
                    program.entries.push(ProgramEntry::Declaration(declaration));
                }),
                // `type_alias_decl` is a declaration, not a statement, so it is
                // dispatched here. `type` stays an ordinary identifier when it
                // is not followed by a Type name, which keeps it usable as a
                // selector.
                Some("type")
                    if decorators.is_empty()
                        && self
                            .peek_next()
                            .is_some_and(|next| next.starts_with(char::is_uppercase)) =>
                {
                    self.type_alias_declaration().map(|value| {
                        let declaration = Declaration::TypeAlias(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                // `import_decl` and `export_decl` are declarations, dispatched
                // here rather than as statements.
                // C069 admits an `override` marker before the import keyword, so
                // the dispatch looks past it rather than treating the marker as
                // an unexpected token.
                Some("import") | Some("from") if decorators.is_empty() => {
                    self.import_declaration().map(|value| {
                        let declaration = Declaration::Import(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                Some("override")
                    if decorators.is_empty()
                        && matches!(self.peek_next(), Some("import") | Some("from")) =>
                {
                    self.import_declaration().map(|value| {
                        let declaration = Declaration::Import(value);
                        program.declarations.push(declaration.clone());
                        program.entries.push(ProgramEntry::Declaration(declaration));
                    })
                }
                Some("export") if decorators.is_empty() => self.export_declaration().map(|value| {
                    let declaration = Declaration::Export(Box::new(value));
                    program.declarations.push(declaration.clone());
                    program.entries.push(ProgramEntry::Declaration(declaration));
                }),
                Some("contract") => self.contract_declaration(decorators).map(|value| {
                    let declaration = Declaration::Contract(value);
                    program.declarations.push(declaration.clone());
                    program.entries.push(ProgramEntry::Declaration(declaration));
                }),
                _ if decorators.is_empty() => self.statement().map(|value| {
                    program.statements.push(value.clone());
                    program.entries.push(ProgramEntry::Statement(value));
                }),
                _ => {
                    self.error("PARSE_UNEXPECTED_TOKEN");
                    None
                }
            };
            self.consume_terminators();
        }
        program
    }

    fn class_declaration(&mut self, decorators: Vec<Decorator>) -> Option<ClassDeclaration> {
        let reopen = self.consume("open");
        self.expect("class")?;
        let name = self.qualified_name()?;
        let parameters = self.generic_parameters();
        let mut extends = None;
        let mut implements = Vec::new();
        let mut mixins = Vec::new();
        let mut constraints = Vec::new();
        let mut meta_deny = Vec::new();
        let mut rank = 0;
        while !self.check("{") && !self.at_end() {
            let clause = match self.peek() {
                Some("extends") => 1,
                Some("for") => 2,
                Some("mixin") => 3,
                Some("where") => 4,
                Some("meta") => 5,
                _ => {
                    self.error("PARSE_BAD_HEADER_ORDER");
                    return None;
                }
            };
            if clause <= rank {
                self.error("PARSE_BAD_HEADER_ORDER");
                return None;
            }
            rank = clause;
            match clause {
                1 => {
                    self.advance();
                    extends = self.type_expression();
                }
                2 => {
                    self.advance();
                    implements = self.type_list();
                }
                3 => {
                    self.advance();
                    mixins = self.mixin_entries();
                }
                4 => {
                    self.advance();
                    constraints = self.constraints();
                }
                5 => {
                    self.advance();
                    meta_deny = self.meta_deny()?;
                }
                _ => unreachable!(),
            }
        }
        let body = self.body()?;
        Some(ClassDeclaration {
            decorators,
            reopen,
            name,
            parameters,
            extends,
            implements,
            mixins,
            constraints,
            meta_deny,
            body,
        })
    }

    fn module_declaration(&mut self, decorators: Vec<Decorator>) -> Option<ModuleDeclaration> {
        let reopen = self.consume("open");
        self.expect("module")?;
        let name = self.qualified_name()?;
        let parameters = self.generic_parameters();
        // V261 expects `module M for C` to report a STATIC diagnostic, so the
        // clause `module_decl` does not admit is consumed here and rejected in
        // analysis rather than failing as a header-order parse error.
        let contract_for = if self.consume("for") {
            self.type_list()
        } else {
            Vec::new()
        };
        let mut mixins = Vec::new();
        let mut constraints = Vec::new();
        let mut meta_deny = Vec::new();
        let mut rank = 0;
        while !self.check("{") && !self.at_end() {
            let clause = match self.peek() {
                Some("mixin") => 1,
                Some("where") => 2,
                Some("meta") => 3,
                _ => {
                    self.error("PARSE_BAD_HEADER_ORDER");
                    return None;
                }
            };
            if clause <= rank {
                self.error("PARSE_BAD_HEADER_ORDER");
                return None;
            }
            rank = clause;
            match clause {
                1 => {
                    self.advance();
                    mixins = self.mixin_entries();
                }
                2 => {
                    self.advance();
                    constraints = self.constraints();
                }
                3 => {
                    self.advance();
                    meta_deny = self.meta_deny()?;
                }
                _ => unreachable!(),
            }
        }
        Some(ModuleDeclaration {
            reopen,
            decorators,
            contract_for,
            name,
            parameters,
            mixins,
            constraints,
            meta_deny,
            body: self.body()?,
        })
    }

    /// Parses `type_alias_decl ::= "type" type_name generic_params? "=" type_expr`.
    /// Parses `import_decl`.
    fn import_declaration(&mut self) -> Option<iris_syntax::ImportDeclaration> {
        // C069 places the C049 replacement-authorization marker BEFORE the
        // keyword, since D-230 authorizes the replacements that import
        // contributes rather than authorizing per name.
        let replacement_authorized = self.consume("override");
        if self.consume("from") {
            let target = self.import_path()?;
            self.expect("import")?;
            let mut specs = Vec::new();
            loop {
                let name = self.name()?;
                let alias = self.consume("as").then(|| self.name()).flatten();
                specs.push(iris_syntax::ImportSpec { name, alias });
                if !self.consume(",") || self.check("\n") {
                    break;
                }
            }
            return Some(iris_syntax::ImportDeclaration {
                target,
                alias: None,
                specs,
                replacement_authorized,
            });
        }
        self.expect("import")?;
        let target = self.import_path()?;
        let alias = self.consume("as").then(|| self.name()).flatten();
        Some(iris_syntax::ImportDeclaration {
            target,
            alias,
            replacement_authorized,
            specs: Vec::new(),
        })
    }

    /// Parses `export_decl`.
    fn export_declaration(&mut self) -> Option<iris_syntax::ExportDeclaration> {
        self.expect("export")?;
        // `export <declaration>` publishes the declaration it wraps; anything
        // else is a list of already-declared names.
        let declaration = match self.peek() {
            Some("class") | Some("open") => self
                .class_declaration(Vec::new())
                .map(|value| Box::new(iris_syntax::Declaration::Class(value))),
            Some("module") => self
                .module_declaration(Vec::new())
                .map(|value| Box::new(iris_syntax::Declaration::Module(value))),
            Some("contract") => self
                .contract_declaration(Vec::new())
                .map(|value| Box::new(iris_syntax::Declaration::Contract(value))),
            // `export_decl ::= "export" (declaration | ...)` and `declaration`
            // derives `import_decl`, so `export import pkg::M` and
            // `export from pkg::M import Name` are the facade re-export forms
            // IRIS-V1-META-C015 names. Only the three declaration keywords were
            // accepted, so both spellings failed to parse.
            Some("import") | Some("from") => self
                .import_declaration()
                .map(|value| Box::new(iris_syntax::Declaration::Import(value))),
            _ => None,
        };
        if let Some(declaration) = declaration {
            return Some(iris_syntax::ExportDeclaration::Declaration(declaration));
        }
        let mut names = vec![self.name()?];
        while self.consume(",") && !self.check("\n") {
            names.push(self.name()?);
        }
        Some(iris_syntax::ExportDeclaration::Names(names))
    }

    fn type_alias_declaration(&mut self) -> Option<iris_syntax::TypeAliasDeclaration> {
        self.expect("type")?;
        let name = self.name()?;
        let parameters = self.generic_parameters();
        self.expect("=")?;
        let target = self.type_expression()?;
        Some(iris_syntax::TypeAliasDeclaration {
            name,
            parameters,
            target,
        })
    }

    fn contract_declaration(&mut self, decorators: Vec<Decorator>) -> Option<ContractDeclaration> {
        // V204 expects a STATIC diagnostic, so `open` is consumed here and
        // reported by analysis rather than failing as a parse error.
        let open = self.consume("open");
        self.expect("contract")?;
        let name = self.name()?;
        let parameters = self.generic_parameters();
        let mut parents = Vec::new();
        let mut constraints = Vec::new();
        let mut meta_deny = Vec::new();
        let mut rank = 0;
        while !self.check("{") && !self.at_end() {
            let clause = match self.peek() {
                Some("extends") => 1,
                Some("where") => 2,
                Some("meta") => 3,
                _ => {
                    self.error("PARSE_BAD_HEADER_ORDER");
                    return None;
                }
            };
            if clause <= rank {
                self.error("PARSE_BAD_HEADER_ORDER");
                return None;
            }
            rank = clause;
            match clause {
                1 => {
                    self.advance();
                    parents = self.type_list();
                }
                2 => {
                    self.advance();
                    constraints = self.constraints();
                }
                3 => {
                    self.advance();
                    meta_deny = self.meta_deny()?;
                }
                _ => unreachable!(),
            }
        }
        Some(ContractDeclaration {
            decorators,
            open,
            name,
            parameters,
            parents,
            constraints,
            meta_deny,
            body: self.body()?,
        })
    }

    /// Parses `closure_literal ::= "{" closure_header? closure_body "}"`.
    ///
    /// The optional header is `"|" closure_parameters? "|"`, so a leading `|`
    /// after `{` distinguishes a parameterised Closure from a bare one. The body
    /// reuses the ordinary statement list, which is what `closure_body` names.
    /// Parses the shared shape of a `try`, used in statement and expression
    /// position alike so the two can never diverge.
    pub(crate) fn try_parts(&mut self) -> Option<TryParts> {
        let body = self.body()?;
        let mut catches = Vec::new();
        while self.consume("catch") {
            catches.push(self.catch_clause()?);
        }
        let finally = if self.consume("finally") {
            Some(self.body()?)
        } else {
            None
        };
        if catches.is_empty() && finally.is_none() {
            self.error("PARSE_TRY_REQUIRES_HANDLER");
            return None;
        }
        Some((body, catches, finally))
    }

    pub(crate) fn closure_literal(&mut self) -> Option<Expression> {
        self.expect("{")?;
        let mut parameters = Vec::new();
        // `closure_header ::= "|" closure_parameters? "|" ...` admits an EMPTY
        // parameter list, but `||` lexes as ONE logical-or token, so a bare
        // `{ ||; ... }` never reached the header at all. An empty header is
        // consumed whole here, which is the same contextual longest-match
        // C020 applies to `>>` closing two generic argument lists.
        if self.check("||")
            && let Some(token) = self.tokens.get_mut(self.cursor)
        {
            // Rewrite the pair into a single `|` and step past it, so the
            // header below sees the empty parameter list it expects.
            token.text = "|".into();
            token.offset += 1;
            self.empty_closure_header = true;
        }
        if self.empty_closure_header {
            self.empty_closure_header = false;
            self.advance();
            if self.consume("-") {
                self.expect(">")?;
                self.type_expression()?;
            }
            self.consume_terminators();
        } else if self.consume("|") {
            let outer = std::mem::replace(&mut self.no_type_union, true);
            while !self.check("|") && !self.at_end() {
                let Some(parameter) = self.binding_name() else {
                    self.no_type_union = outer;
                    return None;
                };
                parameters.push(parameter);
                if self.consume(":") && self.type_expression().is_none() {
                    self.no_type_union = outer;
                    return None;
                }
                if !self.consume(",") {
                    break;
                }
            }
            // The union level is restored BEFORE the return annotation, which
            // sits outside the parameter list and may legitimately be a union.
            self.no_type_union = outer;
            self.expect("|")?;
            if self.consume("-") {
                self.expect(">")?;
                self.type_expression()?;
            }
            self.consume_terminators();
        }
        let mut body = Vec::new();
        self.consume_terminators();
        while !self.check("}") && !self.at_end() {
            if let Some(statement) = self.statement() {
                body.push(statement);
            } else {
                self.advance_to_terminator();
            }
            self.consume_terminators();
        }
        self.expect("}")?;
        Some(Expression::Closure { parameters, body })
    }

    fn body(&mut self) -> Option<Vec<Statement>> {
        self.expect("{")?;
        let mut body = Vec::new();
        self.consume_terminators();
        while !self.check("}") && !self.at_end() {
            if self.check("meta") {
                self.error("PARSE_BAD_HEADER_ORDER");
                return None;
            }
            if let Some(statement) = self.statement() {
                body.push(statement);
            } else {
                self.advance_to_terminator();
            }
            self.consume_terminators();
        }
        self.expect("}")?;
        Some(body)
    }

    /// Consumes an optional `visibility`, which `method_decl` and
    /// `property_decl` share.
    fn method_visibility(&mut self) -> Option<Visibility> {
        if self.consume("public") {
            Some(Visibility::Public)
        } else if self.consume("protected") {
            Some(Visibility::Protected)
        } else if self.consume("private") {
            Some(Visibility::Private)
        } else {
            None
        }
    }

    fn statement(&mut self) -> Option<Statement> {
        let decorators = self.decorators();
        // C059's `shared_decl` and C064's `shared` property marker share the
        // keyword, so `shared_decl` claims it only when `let` or `mut` follows.
        // Consuming it unconditionally made `shared class property` fail before
        // the property form was ever reached.
        // `global_decl ::= "global" ("let"|"mut") global_name ...` has the same
        // shape as `shared_decl`, and C013 makes `$name` reachable ONLY through
        // one of them.
        if self.check("global")
            && matches!(self.peek_next(), Some("let") | Some("mut"))
            && self.consume("global")
        {
            if !decorators.is_empty() {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            }
            let mutable = if self.consume("let") {
                false
            } else if self.consume("mut") {
                true
            } else {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            };
            self.expect("$")?;
            let name = self.binding_name()?;
            let annotation = if self.consume(":") {
                Some(self.type_expression()?)
            } else {
                None
            };
            self.expect("=")?;
            return self.expression(0).map(|value| Statement::GlobalBinding {
                mutable,
                name,
                annotation,
                value,
            });
        }
        if self.check("shared")
            && matches!(self.peek_next(), Some("let") | Some("mut"))
            && self.consume("shared")
        {
            if !decorators.is_empty() {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            }
            let mutable = if self.consume("let") {
                false
            } else if self.consume("mut") {
                true
            } else {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            };
            self.expect("@@")?;
            let name = self.binding_name()?;
            let annotation = if self.consume(":") {
                Some(self.type_expression()?)
            } else {
                None
            };
            self.expect("=")?;
            return self.expression(0).map(|value| Statement::SharedBinding {
                mutable,
                annotation,
                name,
                value,
            });
        }
        if self.check("let") || self.check("mut") || self.check("const") {
            let mut constant = false;
            // `let_decl ::= ("let" | "mut" | "const") ...` makes the three
            // keywords MUTUALLY EXCLUSIVE. Consuming an optional `mut` after
            // `let` admitted `let mut x`, which the grammar does not spell.
            let mutable = if self.consume("let") {
                false
            } else if self.consume("mut") {
                true
            } else {
                constant = self.consume("const");
                false
            };
            if !decorators.is_empty() {
                self.error("PARSE_UNEXPECTED_TOKEN");
                return None;
            }
            let binding = self.binding_name()?;
            let annotation = if self.consume(":") {
                Some(self.type_expression()?)
            } else {
                None
            };
            let annotated = annotation.is_some();
            if self.consume("=") {
                return self.expression(0).map(|value| Statement::Binding {
                    mutable,
                    constant,
                    name: binding,
                    annotation,
                    value,
                });
            }
            return Some(Statement::DeferredBinding {
                mutable,
                annotated,
                name: binding,
            });
        }
        // `method_decl ::= visibility? "override"? "impl"? ...` puts visibility
        // FIRST, but the committed corpus also spells `override public fun`.
        // Both orders are accepted: reading visibility only after the modifiers
        // made `public impl fun` a parse error, and reading it only before
        // would reject every `override public` fixture already in the corpus.
        let mut visibility = self.method_visibility();
        let is_override = self.consume("override");
        if visibility.is_none() {
            visibility = self.method_visibility();
        }
        let is_impl = self.consume("impl");
        if visibility.is_none() {
            visibility = self.method_visibility();
        }
        // C064 admits `shared? ("class"|"module")? property`, so a Class-level
        // marker may precede `property` as well as `fun`. Consuming `class`
        // unconditionally would swallow the marker and then fail to see the
        // `property` that follows it.
        let shared = self.consume("shared");
        let level = if self.consume("class") {
            Some(MethodKind::Class)
        } else if self.consume("module") {
            Some(MethodKind::Module)
        } else {
            None
        };
        let kind = if self.consume("property") {
            MethodKind::Property
        } else {
            level.unwrap_or(MethodKind::Instance)
        };
        let visibility = visibility.unwrap_or(match kind {
            MethodKind::Property => Visibility::Public,
            MethodKind::Instance | MethodKind::Class | MethodKind::Module => Visibility::Private,
        });
        if self.consume("fun") {
            let mut impl_contract = is_impl.then_some(None);
            let mut selector = self.selector()?;
            if is_impl && self.consume("::") {
                impl_contract = Some(Some(selector));
                selector = self.selector()?;
            }
            if kind == MethodKind::Property && self.consume("=") {
                selector.push('=');
            }
            // `method_decl ::= ... "fun" selector generic_params? parameter_list`,
            // so a Method may declare its OWN type parameters. They were never
            // read, which made `fun id<T>(x: T) -> T` a parse error.
            let type_parameters = self.generic_parameters();
            self.expect("(")?;
            let mut parameters = Vec::new();
            while !self.check(")") && !self.at_end() {
                parameters.push(self.parameter()?);
                if !self.consume(",") {
                    break;
                }
            }
            self.expect(")")?;
            // `parameter_sequence` fixes the channel ORDER: positionals, then
            // positional rest, then keywords, then keyword rest, then block.
            // V007 names a positional written AFTER a keyword.
            if !Self::parameters_in_channel_order(&parameters) {
                self.error("PARSE_BAD_PARAMETER_ORDER");
            }
            let return_type = if self.consume("-") {
                self.expect(">")?;
                Some(self.type_expression()?)
            } else {
                None
            };
            // C062 makes `block_body` optional, so a signature that is NOT
            // followed by `{` is a bodyless requirement rather than a parse
            // error. Only a present `{` commits to parsing a body, which keeps
            // a malformed body reported as the body error it is.
            let body = if self.check("{") {
                Some(self.body()?)
            } else {
                None
            };
            return Some(Statement::Method(MethodDeclaration {
                decorators,
                is_override,
                impl_contract,
                kind,
                selector,
                type_parameters,
                parameters,
                return_type,
                visibility,
                body,
            }));
        }
        if kind == MethodKind::Property {
            let name = self.name()?;
            self.expect(":")?;
            let annotation = self.type_expression()?;
            let initializer = if self.consume("=") {
                self.expression(0)?
            } else {
                Expression::Literal("nil".into())
            };
            return Some(Statement::StoredProperty {
                decorators,
                shared,
                class_level: level.is_some(),
                name,
                annotation,
                initializer,
            });
        }
        if !decorators.is_empty() {
            self.error("PARSE_UNEXPECTED_TOKEN");
            return None;
        }
        if self.consume("if") {
            let outer = std::mem::replace(&mut self.no_trailing_block, true);
            let condition = self.expression(0);
            self.no_trailing_block = outer;
            let condition = condition?;
            let then_body = self.body()?;
            let else_body = if self.consume("else") {
                if self.check("if") {
                    Some(vec![self.statement()?])
                } else {
                    Some(self.body()?)
                }
            } else {
                None
            };
            return Some(Statement::If {
                condition,
                then_body,
                else_body,
            });
        }
        if self.consume("return") {
            return Some(Statement::Return(if self.is_terminator() {
                None
            } else {
                self.expression(0)
            }));
        }
        if self.consume("break") {
            let label = if self.is_name() && self.peek_next() == Some(":") {
                self.name()
            } else {
                None
            };
            if label.is_some() {
                self.expect(":")?;
            }
            return Some(Statement::Break {
                label,
                value: if self.is_terminator() {
                    None
                } else {
                    self.expression(0)
                },
            });
        }
        if self.consume("continue") {
            return Some(Statement::Continue(if self.is_name() {
                self.name()
            } else {
                None
            }));
        }
        if self.check("raise") {
            // The offset is read BEFORE consuming, so the location names the
            // `raise` keyword itself rather than whatever follows it.
            let offset = self.current_offset();
            self.advance();
            if self.is_terminator() {
                return Some(Statement::Raise(None));
            }
            let value = self.expression(0)?;
            let cause = if self.consume("from") {
                Some(self.expression(0)?)
            } else {
                None
            };
            return Some(Statement::Raise(Some(Raise {
                value,
                cause,
                offset,
            })));
        }
        // `IRIS-V1-CONTROL-V359` NAMES this code. `defer` is reserved but has no
        // v1 production, so it is rejected before any cleanup Closure is built.
        if self.check("defer") {
            self.advance();
            self.error("PARSE_UNSUPPORTED_DEFER");
            return None;
        }
        // `IRIS-V1-CONTROL-C078` names this code. D-509 leaves these spellings
        // ordinary identifiers, so they are rejected only in the STATEMENT
        // position a v1 production is required, which is the context-specific
        // rejection D-509 already permits.
        if matches!(
            self.peek(),
            Some("groan" | "throw" | "rescue" | "ensure" | "repeat" | "switch")
        ) {
            self.advance();
            self.error("PARSE_LEGACY_FORM");
            return None;
        }
        if self.consume("try") {
            let (body, catches, finally) = self.try_parts()?;
            return Some(Statement::Try {
                body,
                catches,
                finally,
            });
        }
        if self.is_name() && self.peek_next() == Some(":") {
            let label = self.name()?;
            self.expect(":")?;
            if self.consume("while") {
                let condition = self.expression(0)?;
                return Some(Statement::While {
                    label: Some(label),
                    condition,
                    body: self.body()?,
                });
            }
            if self.consume("for") {
                let binding = self.binding_pattern()?;
                self.expect("in")?;
                let iterable = self.expression(0)?;
                return Some(Statement::For {
                    label: Some(label),
                    binding,
                    iterable,
                    body: self.body()?,
                });
            }
            self.error("PARSE_UNEXPECTED_TOKEN");
            return None;
        }
        if self.consume("while") {
            let outer = std::mem::replace(&mut self.no_trailing_block, true);
            let condition = self.expression(0);
            self.no_trailing_block = outer;
            return Some(Statement::While {
                label: None,
                condition: condition?,
                body: self.body()?,
            });
        }
        // `for_statement ::= loop_label? "for" binding_pattern "in" expression
        // block_body`, so the unlabelled form is a statement in its own right.
        if self.consume("for") {
            let binding = self.binding_pattern()?;
            self.expect("in")?;
            let outer = std::mem::replace(&mut self.no_trailing_block, true);
            let iterable = self.expression(0);
            self.no_trailing_block = outer;
            return Some(Statement::For {
                label: None,
                binding,
                iterable: iterable?,
                body: self.body()?,
            });
        }
        if self.consume("match") {
            return self.match_statement();
        }
        let statement = self.expression(0).map(Statement::Expression)?;
        // `IRIS-V1-CONTROL-C078`: an ordinary call requires parentheses. A bare
        // `f 1` parses as a NAME followed by a stranded operand, since a named
        // infix would have consumed the following token as its operator. Only a
        // statement terminator or a closing brace may follow a complete
        // expression, so anything else here is the parenthesis-less call form.
        //
        // An expression that ITSELF ends in `}`, such as a closure or a block
        // form, already delimits itself, so a following declaration is a new
        // statement rather than a stranded operand. Requiring a terminator
        // there rejected `self.define_method(:m) { 1 } public fun v() { 8 }`,
        // which is the shape IRIS-V1-META-C023 uses.
        let self_delimited = self
            .cursor
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .is_some_and(|token| token.text == "}");
        if !self.at_end() && !self_delimited && !matches!(self.peek(), Some(";" | "}" | "\n")) {
            self.error("PARSE_CALL_REQUIRES_PARENTHESES");
            return None;
        }
        Some(statement)
    }

    fn catch_clause(&mut self) -> Option<CatchClause> {
        if self.check("{") {
            return Some(CatchClause {
                binding: None,
                filter: None,
                context: None,
                body: self.body()?,
            });
        }
        let binding = if self.consume("_") {
            CatchBinding::Discard
        } else {
            CatchBinding::Name(self.binding_name()?)
        };
        let filter = if self.consume(":") {
            Some(self.type_expression()?)
        } else {
            None
        };
        let context = if self.consume(",") {
            Some(self.binding_name()?)
        } else {
            None
        };
        Some(CatchClause {
            binding: Some(binding),
            filter,
            context,
            body: self.body()?,
        })
    }

    fn match_statement(&mut self) -> Option<Statement> {
        // The scrutinee is followed by `{`, which would otherwise be read as a
        // trailing block, exactly as for a `while` condition or a `for` iterable.
        let outer = std::mem::replace(&mut self.no_trailing_block, true);
        let subject = self.expression(0);
        self.no_trailing_block = outer;
        let subject = subject?;
        self.expect("{")?;
        self.consume_terminators();
        let mut arms = Vec::new();
        let mut fallback = None;
        while !self.check("}") && !self.at_end() {
            if self.consume("else") {
                self.expect_arrow()?;
                fallback = Some(self.match_body()?);
                self.consume_terminators();
                if !self.check("}") {
                    self.error("PARSE_UNEXPECTED_TOKEN");
                }
                break;
            }
            let pattern = self.pattern()?;
            let guard = if self.consume("if") {
                self.expression(0)
            } else {
                None
            };
            self.expect_arrow()?;
            let body = self.match_body()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
            self.consume(",");
            self.consume_terminators();
        }
        self.expect("}")?;
        Some(Statement::Match {
            subject,
            arms,
            fallback,
        })
    }

    fn match_body(&mut self) -> Option<MatchBody> {
        if self.check("{") {
            self.body().map(MatchBody::Block)
        } else {
            self.expression(0).map(MatchBody::Expression)
        }
    }
    fn pattern(&mut self) -> Option<Pattern> {
        let mut values = vec![self.pattern_alternative()?];
        while self.consume("|") {
            values.push(self.pattern_alternative()?);
        }
        if values.len() == 1 {
            values.pop()
        } else {
            Some(Pattern::Alternatives(values))
        }
    }

    /// Whether a parameter list follows the `parameter_sequence` channel order.
    ///
    /// The grammar fixes the order as required and optional positionals, then
    /// positional rest, then required and optional keywords, then keyword rest,
    /// then the block parameter. A parameter belonging to an EARLIER channel
    /// than one already seen is out of order, which `IRIS-V1-GRAMMAR-V007`
    /// observes for a positional written after a `key` parameter.
    fn parameters_in_channel_order(parameters: &[Parameter]) -> bool {
        const fn channel(category: ParameterCategory) -> u8 {
            match category {
                ParameterCategory::Positional => 0,
                ParameterCategory::Rest => 1,
                ParameterCategory::Keyword => 2,
                ParameterCategory::KeywordRest => 3,
                ParameterCategory::Block => 4,
            }
        }
        let mut highest = 0;
        for parameter in parameters {
            let channel = channel(parameter.category);
            if channel < highest {
                return false;
            }
            highest = channel;
        }
        true
    }

    /// Parses one parameter with the category `IRIS-V1-CONTROL-C023` assigns.
    ///
    /// `*args` is positional rest, `**kwargs` is keyword rest, `key name` is
    /// keyword-only, and `&block` is the dedicated block channel of
    /// `IRIS-V1-GRAMMAR-C050`, which binds by DECLARATION rather than by
    /// guessing which trailing argument is a Closure.
    fn parameter(&mut self) -> Option<Parameter> {
        let category = if self.consume("**") {
            ParameterCategory::KeywordRest
        } else if self.consume("*") {
            ParameterCategory::Rest
        } else if self.consume("&") {
            ParameterCategory::Block
        } else if self.consume("key") {
            ParameterCategory::Keyword
        } else {
            ParameterCategory::Positional
        };
        let name = self.binding_name()?;
        let annotation = if self.consume(":") {
            Some(self.type_expression()?)
        } else {
            None
        };
        let default = if self.consume("=") {
            self.expression(0)
        } else {
            None
        };
        Some(Parameter {
            name,
            category,
            annotation,
            default,
        })
    }

    /// Parses `binding_pattern`, the destructuring subset `for` accepts.
    ///
    /// The grammar admits a name, `_`, and bracketed forms. Only the name and
    /// array forms are built here, because a Tuple value does not exist yet and
    /// inventing one would fabricate semantics chapter 06 owns.
    fn binding_pattern(&mut self) -> Option<Pattern> {
        if self.consume("[") {
            let mut elements = Vec::new();
            while !self.check("]") && !self.at_end() {
                elements.push(self.binding_pattern()?);
                if !self.consume(",") {
                    break;
                }
            }
            self.expect("]")?;
            return Some(Pattern::Array(elements));
        }
        self.name().map(Pattern::Name)
    }

    /// Parses one `match_pattern_alternative` from the C051 vocabulary.
    ///
    /// A literal, `nil`, or a Bool literal compares by value; anything else is a
    /// binding pattern, and `_` discards without binding.
    fn pattern_alternative(&mut self) -> Option<Pattern> {
        if let Some(token) = self.peek()
            && (self.is_literal(token) || matches!(token, "nil" | "true" | "false"))
        {
            let literal = self.advance()?.text;
            return Some(Pattern::Literal(literal));
        }
        self.name().map(Pattern::Name)
    }

    fn generic_parameters(&mut self) -> Vec<String> {
        let mut values = Vec::new();
        if self.consume("<") {
            while !self.check(">") && !self.at_end() {
                if let Some(name) = self.name() {
                    values.push(name);
                }
                if !self.consume(",") {
                    break;
                }
            }
            let _ = self.expect(">");
        }
        values
    }
    fn constraints(&mut self) -> Vec<Constraint> {
        let mut values = Vec::new();
        while let Some(parameter) = self.name() {
            if self.expect(":").is_none() {
                break;
            }
            let Some(bound) = self.type_expression() else {
                break;
            };
            values.push(Constraint { parameter, bound });
            if !self.consume(",") {
                break;
            }
        }
        values
    }
    fn type_list(&mut self) -> Vec<TypeExpression> {
        let mut values = Vec::new();
        while let Some(value) = self.type_expression() {
            values.push(value);
            if !self.consume(",") {
                break;
            }
        }
        values
    }
    fn function_type(&mut self) -> Option<TypeExpression> {
        self.expect("(")?;
        let mut parameters = Vec::new();
        while !self.check(")") && !self.at_end() {
            parameters.push(self.type_expression()?);
            if !self.consume(",") {
                break;
            }
        }
        self.expect(")")?;
        self.expect("-")?;
        self.expect(">")?;
        let result = self.type_expression()?;
        Some(TypeExpression::Function {
            parameters,
            result: Box::new(result),
        })
    }

    /// Parses `type_expr ::= type_union`.
    ///
    /// `IRIS-V1-GRAMMAR-C013`'s grammar gives `type_union ::= type_intersection
    /// ("|" type_intersection)*`, and `IRIS-V1-CONTROL-C005` points at
    /// `String | Integer` as the way to declare a wider binding cell, so the
    /// union level is required rather than optional.
    fn type_expression(&mut self) -> Option<TypeExpression> {
        let first = self.type_intersection()?;
        if self.no_type_union {
            return Some(first);
        }
        let mut values = vec![first];
        while self.consume("|") {
            values.push(self.type_intersection()?);
        }
        if values.len() == 1 {
            values.pop()
        } else {
            Some(TypeExpression::Union(values))
        }
    }

    fn type_intersection(&mut self) -> Option<TypeExpression> {
        // IRIS-V1-TYPES-C094 makes the bare signature NOT a Type on its own:
        // `callable_type ::= ("Closure"|"BoundMethod"|"Block") "<" function_type ">"`.
        // The inner `(` form is parsed only as that generic argument.
        if matches!(self.peek(), Some("Closure" | "BoundMethod" | "Block")) {
            let kind = self.name()?;
            self.expect("<")?;
            let signature = self.function_type()?;
            self.expect(">")?;
            return Some(TypeExpression::Generic {
                name: kind,
                arguments: vec![signature],
            });
        }

        let first = if self.consume("typeof") {
            self.expect("(")?;
            let expression = self.expression(0)?;
            self.expect(")")?;
            TypeExpression::Typeof(Box::new(expression))
        } else if self.check("(") {
            // `type_primary ::= ... | "(" type_expr ")"`. A grouped Type was
            // never parsed, so `(String | Nil) & NonNil` could not be written.
            self.advance();
            let outer = std::mem::replace(&mut self.no_type_union, false);
            let grouped = self.type_expression();
            self.no_type_union = outer;
            self.expect(")")?;
            grouped?
        } else {
            let name = self.qualified_name()?;
            // `type_primary ::= ... | qualified_type_name generic_args? | ...`
            // so any nominal Type name may carry closed generic arguments,
            // including `Dynamic<T>` from IRIS-V1-TYPES-C014.
            if self.consume("<") {
                let mut arguments = Vec::new();
                while !self.check(">") && !self.at_end() {
                    arguments.push(self.type_expression()?);
                    if !self.consume(",") {
                        break;
                    }
                }
                self.expect_generic_close()?;
                TypeExpression::Generic { name, arguments }
            } else if self.check("[") {
                // `IRIS-V1-TYPES-V210` NAMES this code. Angle brackets are the
                // accepted spelling under IRIS-V1-GRAMMAR-C020, so `Box[String]`
                // is rejected here rather than derailing the whole annotation.
                self.error("GENERIC_BRACKET_SYNTAX_FORBIDDEN");
                return None;
            } else {
                TypeExpression::Name(name)
            }
        };
        // `type_postfix ::= type_primary "?"?`. C011 makes `T?` sugar for
        // `T | Nil`, which V011 states as a normalization law, so the sugar is
        // expanded here rather than carried as a distinct Type form.
        let first = if self.consume("?") {
            TypeExpression::Union(vec![first, TypeExpression::Name("Nil".into())])
        } else {
            first
        };
        let mut values = vec![first];
        while self.consume("&") {
            let member = self.type_intersection()?;
            values.push(member);
        }
        if values.len() == 1 {
            values.pop()
        } else {
            Some(TypeExpression::Intersection(values))
        }
    }
    fn meta_deny(&mut self) -> Option<Vec<String>> {
        self.expect("deny")?;
        let mut values = vec![self.name()?];
        while self.consume(",") {
            values.push(self.name()?);
        }
        if values.iter().any(|value| !is_meta_capability(value)) {
            self.error("PARSE_UNKNOWN_META_CAPABILITY");
            return None;
        }
        Some(values)
    }
    fn mixin_entries(&mut self) -> Vec<MixinEntry> {
        let mut entries = Vec::new();
        while let Some(target) = self.type_expression() {
            entries.push(MixinEntry {
                target,
                private_access: self.consume("private"),
            });
            if !self.consume(",") {
                break;
            }
        }
        entries
    }
    /// Reads a `quoted_symbol` body, the `:"..."` form of a Symbol literal.
    ///
    /// The chapter 02 grammar limits `selector_suffix` to `?` and `!`, so a
    /// setter selector such as `name=` is NOT expressible as a bare `:name=`.
    /// `symbol_literal` admits `quoted_symbol` precisely to name one, which
    /// reflection needs to reach the setter that `IRIS-V1-RUNTIME-C061` defines.
    /// The production is `string_double_body_without_interpolation`, so an
    /// interpolating body is rejected here.
    fn quoted_symbol(&mut self) -> Option<String> {
        let text = self.peek()?;
        let body = text.strip_prefix('"')?.strip_suffix('"')?.to_owned();
        if body.contains("#{") {
            return None;
        }
        self.advance();
        Some(body)
    }

    fn selector(&mut self) -> Option<String> {
        if let Some(operator) = self.operator_selector() {
            return Some(operator.into());
        }
        let name = self.name()?;
        self.selector_suffix(name)
    }

    fn operator_selector(&mut self) -> Option<&'static str> {
        let operator = match self.peek()? {
            "+" => "+",
            "-" => "-",
            "*" => "*",
            "**" => "**",
            "/" => "/",
            "&" => "&",
            "|" => "|",
            "^" => "^",
            "~" => "~",
            "<<" => "<<",
            ">>" => ">>",
            "<" => "<",
            "<=" => "<=",
            ">" => ">",
            ">=" => ">=",
            "<=>" => "<=>",
            "=~" => "=~",
            "!~" => "!~",
            "==" => "==",
            "!=" => "!=",
            _ => return None,
        };
        self.advance();
        Some(operator)
    }
    /// Reports whether an `@` path is followed by a decorator argument list.
    ///
    /// `IRIS-V1-GRAMMAR-C070` admits a `qualified_type_name` here, so the path
    /// may span several tokens. Scanning it before committing keeps a `@` that
    /// begins something else from being consumed as a decorator.
    fn decorator_arguments_follow(&self) -> bool {
        let mut index = self.cursor + 1;
        loop {
            if !self
                .tokens
                .get(index)
                .is_some_and(|token| is_identifier(&token.text))
            {
                return false;
            }
            index += 1;
            match self.tokens.get(index).map(|token| token.text.as_str()) {
                Some("(") => return true,
                Some("::") => index += 1,
                _ => return false,
            }
        }
    }

    fn decorators(&mut self) -> Vec<Decorator> {
        let mut decorators = Vec::new();
        // `IRIS-V1-GRAMMAR-C070` widens the application path to a
        // `qualified_type_name`, so a decorator declared in another Module is
        // applied as `@D::Stamp()`. The lookahead therefore scans the whole
        // path before requiring the argument list, rather than assuming the
        // name is one token.
        while self.check("@")
            && self
                .tokens
                .get(self.cursor + 1)
                .is_some_and(|token| is_identifier(&token.text))
            && self.decorator_arguments_follow()
        {
            self.advance();
            let Some(name) = self.qualified_name() else {
                break;
            };
            if self.expect("(").is_none() {
                break;
            }
            let Some(arguments) = self.arguments() else {
                break;
            };
            decorators.push(Decorator { name, arguments });
        }
        decorators
    }
    fn selector_suffix(&mut self, mut selector: String) -> Option<String> {
        if self.consume("?") {
            selector.push('?');
        } else if self.consume("!") {
            selector.push('!');
        }
        if self.check("?") || self.check("!") {
            self.error("PARSE_INVALID_SELECTOR_SUFFIX");
            while self.consume("?") || self.consume("!") {}
            return None;
        }
        Some(selector)
    }
    fn binding_name(&mut self) -> Option<String> {
        let value = self.peek()?;
        if is_reserved_keyword(value) {
            self.error("PARSE_RESERVED_KEYWORD_BINDING");
            self.advance();
            return None;
        }
        let name = self.name()?;
        if self.check("?") || self.check("!") {
            self.error("PARSE_INVALID_SELECTOR_SUFFIX");
            while self.consume("?") || self.consume("!") {}
            return None;
        }
        Some(name)
    }
    fn assignment_operator(&self) -> Option<iris_syntax::AssignmentOperator> {
        match self.peek()? {
            "=" => Some(iris_syntax::AssignmentOperator::Assign),
            "+=" => Some(iris_syntax::AssignmentOperator::Add),
            "-=" => Some(iris_syntax::AssignmentOperator::Subtract),
            "*=" => Some(iris_syntax::AssignmentOperator::Multiply),
            "/=" => Some(iris_syntax::AssignmentOperator::Divide),
            "**=" => Some(iris_syntax::AssignmentOperator::Power),
            "&=" => Some(iris_syntax::AssignmentOperator::BitwiseAnd),
            "|=" => Some(iris_syntax::AssignmentOperator::BitwiseOr),
            "^=" => Some(iris_syntax::AssignmentOperator::BitwiseXor),
            "<<=" => Some(iris_syntax::AssignmentOperator::ShiftLeft),
            ">>=" => Some(iris_syntax::AssignmentOperator::ShiftRight),
            "&&=" => Some(iris_syntax::AssignmentOperator::LogicalAnd),
            "||=" => Some(iris_syntax::AssignmentOperator::LogicalOr),
            _ => None,
        }
    }
    fn consume_terminators(&mut self) {
        let mut saw_semicolon = false;
        while self.consume(";") || self.consume("\n") {
            if self
                .tokens
                .get(self.cursor - 1)
                .is_some_and(|token| token.text == ";")
            {
                if saw_semicolon {
                    self.error("PARSE_EMPTY_STATEMENT");
                    return;
                }
                saw_semicolon = true;
            }
        }
    }
    fn is_terminator(&self) -> bool {
        self.at_end() || matches!(self.peek(), Some(";" | "}" | "\n"))
    }
    fn advance_to_terminator(&mut self) {
        while !self.is_terminator() {
            self.advance();
        }
    }
    fn name(&mut self) -> Option<String> {
        if self.is_name() {
            self.advance().map(|token| token.text)
        } else {
            self.error("PARSE_UNEXPECTED_TOKEN");
            None
        }
    }

    /// Parses the path of an `import_decl`.
    ///
    /// `IRIS-V1-GRAMMAR-C068` admits `package_name "::" qualified_type_name`,
    /// where `package_name` is a DOTTED reverse-domain identity such as
    /// `org.dep`. `IRIS-V1-META-C003` makes every publishable package carry such
    /// an identity, which `IRIS-V1-META-C013` then writes as `pkg::Module`.
    ///
    /// The dotted form is admitted ONLY here, before the `::`. A `.` elsewhere
    /// keeps its member-access meaning, and a path with no `::` continues to
    /// name a Module in the current package.
    fn import_path(&mut self) -> Option<String> {
        let mut name = self.name()?;
        while self.check(".") {
            let restore = self.cursor;
            self.advance();
            let Some(segment) = self.name() else {
                self.cursor = restore;
                break;
            };
            // Only a dotted run that REACHES a `::` is a package name. Anything
            // else is left to its ordinary reading rather than being consumed.
            name.push('.');
            name.push_str(&segment);
        }
        while self.consume_qualified_separator() {
            name.push_str("::");
            // IRIS-V1-META-C013 keeps wildcard imports out of Iris v1 source and
            // IRIS-V1-META-V417 names the diagnostic, so a `*` here is reported
            // under that name rather than as a generic unexpected token.
            if self.check("*") {
                self.error("IRIS-IMPORT-WILDCARD");
                // The `*` is consumed so the rejected declaration does not also
                // strand a token and report a second, generic diagnostic for one
                // cause. V417 names exactly one code for this input.
                self.advance();
                return Some(name);
            }
            name.push_str(&self.name()?);
        }
        Some(name)
    }

    pub(crate) fn qualified_name(&mut self) -> Option<String> {
        let mut name = self.name()?;
        while self.consume_qualified_separator() {
            name.push_str("::");
            name.push_str(&self.name()?);
        }
        Some(name)
    }

    pub(crate) fn consume_qualified_separator(&mut self) -> bool {
        if self.consume("::") {
            return true;
        }
        // A lone `:` must NOT be consumed here. Testing the second colon only
        // after eating the first advanced the cursor past a `:` that belongs to
        // its enclosing form, which made a bare identifier Hash key such as
        // `%{ a: 10 }` fail to parse at all.
        if self.check(":") && self.peek_next() == Some(":") {
            self.advance();
            self.advance();
            return true;
        }
        false
    }
    fn is_name(&self) -> bool {
        self.peek().is_some_and(is_identifier)
    }
    fn is_literal(&self, value: &str) -> bool {
        value.chars().next().is_some_and(|character| {
            character.is_ascii_digit() || character == '\'' || character == '"'
        }) || matches!(value, "nil" | "true" | "false")
            // C023 makes a slash start a Regex literal ONLY where a primary
            // expression is expected. This is reached from `primary`, which is
            // exactly such a position, so a slash here is a literal rather than
            // the division C023 assigns to continuation positions. The lexer
            // already produced one token for the whole literal.
            || (value.starts_with('/') && value.len() > 1)
    }
    fn expect(&mut self, expected: &str) -> Option<()> {
        if self.consume(expected) {
            Some(())
        } else {
            self.error("PARSE_UNEXPECTED_TOKEN");
            None
        }
    }
    /// Closes ONE generic argument list, splitting a `>>` token when needed.
    ///
    /// `IRIS-V1-GRAMMAR-C020` requires Type grammar to let `>>` close two
    /// nested generic argument lists WITHOUT changing expression right-shift
    /// tokenization. One token stream serves both grammars here, so the shared
    /// `>>` is split in place: the inner list consumes the first `>` and leaves
    /// a `>` for its enclosing list. Rewriting the token rather than consuming
    /// it keeps `a >> b` an ordinary right shift everywhere else.
    fn expect_generic_close(&mut self) -> Option<()> {
        if self.consume(">") {
            return Some(());
        }
        if self.check(">>")
            && let Some(token) = self.tokens.get_mut(self.cursor)
        {
            token.text = ">".into();
            token.offset += 1;
            return Some(());
        }
        self.error("PARSE_UNEXPECTED_TOKEN");
        None
    }
    fn expect_arrow(&mut self) -> Option<()> {
        if self.consume("=>") || (self.consume("=") && self.consume(">")) {
            Some(())
        } else {
            self.error("PARSE_UNEXPECTED_TOKEN");
            None
        }
    }
    fn consume(&mut self, expected: &str) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn check(&self, expected: &str) -> bool {
        self.peek() == Some(expected)
    }
    fn peek(&self) -> Option<&str> {
        self.tokens
            .get(self.cursor)
            .map(|token| token.text.as_str())
    }
    /// Returns the name when the cursor sits on a `name:` keyword argument.
    ///
    /// The name must be an ordinary identifier and the colon must be the very
    /// next token, so `a ? b : c` and a `:sym` Symbol literal are both left
    /// alone.
    fn peek_keyword_argument_name(&self) -> Option<String> {
        let name = self.peek()?;
        if self.peek_next() != Some(":") || is_reserved_keyword(name) {
            return None;
        }
        if !name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            return None;
        }
        Some(name.to_owned())
    }
    /// Reports whether the token before the cursor ends the logical line.
    ///
    /// `a[0]` is an index, but a `[` that STARTS a line is an Array literal
    /// statement. Distinguishing them keeps the postfix index from swallowing a
    /// following literal.
    fn newline_before_cursor(&self) -> bool {
        self.cursor == 0
            || self
                .tokens
                .get(self.cursor - 1)
                .is_some_and(|token| matches!(token.text.as_str(), "\n" | ";"))
    }
    /// The byte offset of the token at the cursor.
    fn current_offset(&self) -> usize {
        self.tokens.get(self.cursor).map_or(0, |token| token.offset)
    }
    fn peek_next(&self) -> Option<&str> {
        self.tokens
            .get(self.cursor + 1)
            .map(|token| token.text.as_str())
    }
    fn advance(&mut self) -> Option<Token> {
        let value = self.tokens.get(self.cursor).cloned();
        self.cursor += usize::from(value.is_some());
        value
    }
    fn at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }
    fn error(&mut self, code: &'static str) {
        if !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code)
        {
            self.diagnostics.push(Diagnostic { code });
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Associativity {
    Left,
    Right,
    NonAssociative,
}

fn is_identifier(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
}

fn is_meta_capability(value: &str) -> bool {
    matches!(
        value,
        "method_set"
            | "method_body"
            | "property_set"
            | "property_body"
            | "modules"
            | "superclass"
            | "subclass"
            | "shape"
            | "class_state_set"
            | "class_state_write"
            | "instance_state"
            | "native"
    )
}

fn is_reserved_keyword(value: &str) -> bool {
    matches!(
        value,
        "class"
            | "module"
            | "contract"
            | "open"
            | "extends"
            | "for"
            | "mixin"
            | "where"
            | "meta"
            | "deny"
            | "public"
            | "protected"
            | "private"
            | "override"
            | "impl"
            | "property"
            | "shared"
            | "key"
            | "async"
            | "await"
            | "fun"
            | "let"
            | "mut"
            | "const"
            | "global"
            | "import"
            | "from"
            | "as"
            | "export"
            | "type"
            | "if"
            | "else"
            | "while"
            | "in"
            | "break"
            | "continue"
            | "match"
            | "try"
            | "catch"
            | "finally"
            | "raise"
            | "return"
            | "is"
            | "nil"
            | "true"
            | "false"
            | "self"
            | "super"
            | "typeof"
    )
}

#[cfg(test)]
mod import_wildcard_tests {
    use crate::parse;

    fn codes(source: &str) -> Vec<&'static str> {
        parse(source)
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn c013_rejects_a_wildcard_import_under_its_named_code() {
        // IRIS-V1-META-C013 keeps wildcard imports out of Iris v1 source, and
        // IRIS-V1-META-V417 names the diagnostic. The rejection was already
        // correct but reported the generic parse code, and stranding the `*`
        // reported a SECOND diagnostic for one cause.
        assert_eq!(codes("import org.dep::*"), ["IRIS-IMPORT-WILDCARD"]);
        assert_eq!(
            codes("import org.dep::Core as C\nimport org.dep::*"),
            ["IRIS-IMPORT-WILDCARD"]
        );

        // A named import is unaffected.
        assert!(codes("import org.dep::Core as C").is_empty());
    }
}

#[cfg(test)]
mod export_facade_tests {
    use crate::parse;

    fn accepted(source: &str) -> bool {
        parse(source).program_accepted
    }

    #[test]
    fn c015_admits_the_two_facade_re_export_forms() {
        // `export_decl ::= "export" (declaration | ...)` and `declaration`
        // derives `import_decl`, so IRIS-V1-META-C015's re-export spellings
        // `export import` and `export from` are already in the grammar. Only
        // the three declaration keywords were accepted, so both failed to
        // parse and every facade row was unreachable.
        assert!(accepted("export import org.dep::Core"));
        assert!(accepted("export from org.dep::Core import Name"));
        assert!(accepted(
            "export from org.dep::Core import Name, Other as Alias"
        ));

        // The wrapped-declaration and name-list forms still parse.
        assert!(accepted(
            "export module M { public fun f() -> Integer { 1 } }"
        ));
        assert!(accepted("class A { } export A"));
    }
}

#[cfg(test)]
mod import_override_marker_tests {
    use crate::parse;

    fn authorized(source: &str) -> Vec<bool> {
        parse(source)
            .program
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                iris_syntax::Declaration::Import(value) => Some(value.replacement_authorized),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn c069_marks_an_import_that_authorizes_replacement() {
        // IRIS-V1-META-C049 requires import-site replacement authorization
        // using "the language's accepted `override` import marker", and D-230
        // left its placement for later standardization. The v1.24 errata
        // IRIS-V1-GRAMMAR-C069 places it before the keyword, since D-230
        // authorizes the replacements THAT IMPORT contributes rather than
        // authorizing per name.
        assert_eq!(
            authorized("override import org.dep::Core\nimport org.dep::Other"),
            [true, false]
        );
        assert_eq!(
            authorized("override from org.dep::Names import One"),
            [true]
        );

        // The marker is admitted ONLY before an import keyword, so
        // `method_decl`'s own `override` is untouched.
        let method = "class A { public fun m() -> Integer { 1 } } \
                      class B extends A { public override fun m() -> Integer { 2 } }";
        assert!(parse(method).program_accepted);

        // The unmarked forms still parse, which IRIS-V1-CONTROL-V351 depends on.
        assert!(parse("module S { const K = 5 } from S import K").program_accepted);
    }
}

#[cfg(test)]
mod decorator_application_tests {
    use crate::parse;

    fn accepted(source: &str) -> bool {
        parse(source).program_accepted
    }

    fn identities(source: &str) -> Vec<String> {
        parse(source)
            .program
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                iris_syntax::Declaration::Class(value) => Some(value),
                _ => None,
            })
            .flat_map(|value| value.decorators.iter().map(|d| d.name.clone()))
            .collect()
    }

    #[test]
    fn c070_applies_a_decorator_declared_in_another_module() {
        // IRIS-V1-GRAMMAR-C070 widens the application path to a
        // `qualified_type_name`, so a decorator declared elsewhere is applied
        // as `@D::Stamp()`. The lookahead assumed the name was ONE token, so
        // the qualified form did not parse and IRIS-V1-META-C122's Contracts
        // could not be referenced across Modules.
        assert_eq!(identities("@D::Stamp() class Box { }"), ["D::Stamp"]);
        assert_eq!(identities("@A::B::Stamp() class Box { }"), ["A::B::Stamp"]);

        // The simple form is unchanged, and so is a Method's own decorator.
        assert_eq!(identities("@Stamp() class Box { }"), ["Stamp"]);
        assert!(accepted(
            "class B { @mdec() public fun m() -> Integer { 1 } }"
        ));

        // A `@` that begins something without an argument list is not consumed
        // as a decorator.
        assert!(!accepted("@D::Stamp class Box { }"));
    }

    #[test]
    fn c124_admits_the_corrected_phase_signatures() {
        // IRIS-V1-META-C092 gives a Decorator Contract "immutable declaration
        // metadata PLUS a controlled transform context", which are two distinct
        // inputs. C122 named only `declaration` and `arguments`, where
        // `arguments` is C085's application-site argument list and is NOT that
        // context. The v1.27 errata IRIS-V1-META-C124 supplies the context to
        // the RUNTIME phase alone, since C088 makes the static phase pure.
        let source = "contract ClassDecorator { \
                        fun plan(declaration, arguments) \
                        fun transform(declaration, arguments, context) } \
                      class Stamp for ClassDecorator { \
                        public impl fun plan(declaration, arguments) -> Nil { nil } \
                        public impl fun transform(declaration, arguments, context) -> Nil { nil } } \
                      @Stamp() class Box { }";
        assert!(accepted(source));
    }

    #[test]
    fn c122_admits_the_decorator_contract_shape() {
        // IRIS-V1-META-C122 makes a decorator an ordinary Class declaring `for`
        // one of the five named Contracts, which IRIS-V1-TYPES-C044 requires
        // since only Classes declare conformance. No declaration production is
        // needed, so IRIS-V1-CONTROL-C014's three callable kinds stay intact.
        let source = "contract ClassDecorator { fun plan(d, a) fun transform(d, a) } \
                      class Stamp for ClassDecorator { \
                        public impl fun plan(d, a) -> Nil { nil } \
                        public impl fun transform(d, a) -> Nil { nil } } \
                      @Stamp() class Box { }";
        assert!(accepted(source));

        // The Contract names are ordinary declarations, so a user Class may
        // still carry one until the built-ins are published.
        assert!(accepted("class ClassDecorator { }"));
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use crate::parse;
    use iris_syntax::{
        BinaryOperator, Declaration, Expression, MixinEntry, Statement, TypeExpression,
    };

    #[test]
    fn powers_are_right_associative_and_bind_tighter_than_negation() {
        let result = parse("2 ** 3 ** 2; -2 ** 2");
        assert!(result.is_clean());
        let [
            Statement::Expression(Expression::Binary {
                left,
                operator: BinaryOperator::Power,
                right,
            }),
            Statement::Expression(Expression::Unary { operand, .. }),
        ] = result.program.statements.as_slice()
        else {
            panic!("expected power expression followed by unary expression");
        };
        assert!(matches!(left.as_ref(), Expression::Literal(_)));
        assert!(matches!(
            right.as_ref(),
            Expression::Binary {
                operator: BinaryOperator::Power,
                ..
            }
        ));
        assert!(matches!(
            operand.as_ref(),
            Expression::Binary {
                operator: BinaryOperator::Power,
                ..
            }
        ));
    }
    #[test]
    fn reordered_header_has_the_stable_diagnostic() {
        let result = parse("class A for C extends B {}");
        assert_eq!(result.diagnostics[0].code, "PARSE_BAD_HEADER_ORDER");
        assert!(!result.program_accepted);
    }
    #[test]
    fn leading_semicolon_has_the_stable_diagnostic() {
        let result = parse(";return nil");
        assert_eq!(result.diagnostics[0].code, "PARSE_LEGACY_LEADING_SEMICOLON");
        assert!(!result.program_accepted);
    }
    #[test]
    fn non_associative_chains_are_rejected() {
        let result = parse("a < b < c");
        assert_eq!(result.diagnostics[0].code, "PARSE_NONASSOCIATIVE_CHAIN");
        assert!(!result.program_accepted);
    }
    #[test]
    fn class_where_constraints_preserve_intersection_shape() {
        let result = parse("class Pair<T, U> where T: A & B, U: C {}");
        assert!(result.is_clean());
        let [Declaration::Class(class)] = result.program.declarations.as_slice() else {
            panic!("expected class declaration");
        };
        assert_eq!(class.constraints.len(), 2);
        assert!(matches!(
            class.constraints[0].bound,
            TypeExpression::Intersection(_)
        ));
        assert!(matches!(
            class.constraints[1].bound,
            TypeExpression::Name(_)
        ));
    }
    #[test]
    fn contract_parents_preserve_source_order() {
        let result = parse("contract Child extends ParentA, ParentB {}");
        assert!(result.is_clean());
        let [Declaration::Contract(contract)] = result.program.declarations.as_slice() else {
            panic!("expected contract declaration");
        };
        assert_eq!(
            contract.parents,
            [
                TypeExpression::Name("ParentA".into()),
                TypeExpression::Name("ParentB".into())
            ]
        );
    }

    #[test]
    fn empty_statements_and_meta_in_a_body_are_rejected() {
        let empty = parse("x;;y");
        let body_meta = parse("class A { meta deny shape }");

        assert_eq!(empty.diagnostics[0].code, "PARSE_EMPTY_STATEMENT");
        assert_eq!(body_meta.diagnostics[0].code, "PARSE_BAD_HEADER_ORDER");
    }

    #[test]
    fn meta_deny_accepts_only_the_complete_c073_vocabulary() {
        // Given
        let accepted = [
            "method_set",
            "method_body",
            "property_set",
            "property_body",
            "modules",
            "superclass",
            "subclass",
            "shape",
            "class_state_set",
            "class_state_write",
            "instance_state",
            "native",
        ];

        // When
        let results =
            accepted.map(|capability| parse(&format!("class A meta deny {capability} {{}}")));
        let unknown = parse("class A meta deny bogus_name {}");

        // Then
        assert!(results.iter().all(|result| result.is_clean()));
        assert!(!unknown.program_accepted);
        assert!(
            unknown
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "PARSE_UNKNOWN_META_CAPABILITY")
        );
    }

    #[test]
    fn diagnostics_cover_header_semicolons_reserved_bindings_selector_suffixes_and_assignments() {
        let header = parse("class A meta deny shape {}; class A { if true { meta deny shape } }");
        let semicolons = parse(";return nil; ; ; x;;y");
        let reserved = parse(
            "let alias = 1\nlet switch = 1\nlet when = 1\nlet and = 1\nlet or = 1\nlet not = 1\nlet class = 1",
        );
        let selectors = parse(
            "fun ready?() { true }\nfun save!() { nil }\nfun ready?!() { nil }\nlet done? = true",
        );
        let assignments = parse(
            "a += b; a -= b; a *= b; a /= b; a **= b; a &= b; a |= b; a ^= b; a <<= b; a >>= b; a %= b; a &&= b; a ||= b",
        );

        assert_eq!(header.diagnostics[0].code, "PARSE_BAD_HEADER_ORDER");
        assert_eq!(
            semicolons
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            ["PARSE_LEGACY_LEADING_SEMICOLON", "PARSE_EMPTY_STATEMENT"]
        );
        assert_eq!(
            reserved.diagnostics[0].code,
            "PARSE_RESERVED_KEYWORD_BINDING"
        );
        assert_eq!(
            selectors.diagnostics[0].code,
            "PARSE_INVALID_SELECTOR_SUFFIX"
        );
        assert_eq!(
            assignments.diagnostics[0].code,
            "PARSE_UNSUPPORTED_COMPOUND_ASSIGNMENT"
        );
    }

    #[test]
    fn parses_shift_assignments_from_single_lexer_tokens() {
        // Given
        let left = parse("a <<= b");
        let right = parse("a >>= b");
        let invalid = parse("a %= b");

        // When / Then
        assert!(left.program_accepted, "{left:#?}");
        assert!(right.program_accepted, "{right:#?}");
        assert_eq!(
            invalid.diagnostics[0].code,
            "PARSE_UNSUPPORTED_COMPOUND_ASSIGNMENT"
        );
    }

    #[test]
    fn c066_admits_call_type_arguments_before_an_argument_list() {
        // C066 admits explicit Method type arguments before the argument list.
        // C020 is NOT weakened: the reading is taken only when the bracket pair
        // closes with a `>` IMMEDIATELY followed by `(`.
        assert!(parse("choose<String, Integer>(\"x\", 1)").program_accepted);
        assert!(parse("choose<String, _>(\"x\", 1)").program_accepted);
        // Every other `<` keeps its operator tokenization.
        assert!(parse("let a = 1; let b = 2; a < b").program_accepted);
        assert!(parse("let a = 8; let b = 2; a >> b").program_accepted);
    }

    #[test]
    fn c020_splits_a_pipe_pair_into_an_empty_closure_header() {
        // `closure_header ::= "|" closure_parameters? "|" ...` admits an EMPTY
        // parameter list, but `||` lexes as ONE logical-or token, so a bare
        // `{ ||; ... }` never reached the header. The pair is split by the same
        // contextual longest-match C020 applies to `>>`.
        assert!(parse("let c = { ||; 1 }; c").program_accepted);
        assert!(parse("let c = { |x|; x }; c").program_accepted);
        // Every other `||` keeps its operator tokenization.
        assert!(parse("let a = nil; let b = a || 7; b").program_accepted);
        assert!(parse("mut a = nil; a ||= 7; a").program_accepted);
    }

    #[test]
    fn parses_the_declaration_forms_the_grammar_defines() {
        // `global_decl`, `import_decl`, and `export_decl` are in the frozen
        // grammar but were never implemented, so each was a parse error.
        assert!(parse("global let $g = 1").program_accepted);
        assert!(parse("global mut $g = 1").program_accepted);
        assert!(parse("import Foo").program_accepted);
        assert!(parse("import Foo as F").program_accepted);
        assert!(parse("from Foo import a, b").program_accepted);
        assert!(parse("export class A {}").program_accepted);
        assert!(parse("class A {} export A").program_accepted);
        // `global_decl` requires `let` or `mut`, exactly as `shared_decl` does.
        assert!(!parse("global $g = 1").program_accepted);
    }

    #[test]
    fn let_decl_keywords_are_mutually_exclusive() {
        // `let_decl ::= ("let" | "mut" | "const") ...` offers exactly one
        // keyword, so a mutable binding is `mut x`, never `let mut x`.
        assert!(parse("mut x = 1").program_accepted);
        assert!(parse("const x = 1").program_accepted);
        assert!(!parse("let mut x = 1").program_accepted);
        assert!(!parse("let const x = 1").program_accepted);
        assert!(!parse("mut const x = 1").program_accepted);
    }

    #[test]
    fn accepts_source_method_declarations_and_bindings_in_declaration_programs() {
        // Given
        let source = "class A { public fun scale(value: Integer) -> Integer { value * 2 } }; let a = A.new(); a.scale(3)";

        // When
        let result = parse(source);

        // Then
        assert!(result.program_accepted, "{result:#?}");
    }

    #[test]
    fn parses_shared_class_and_module_declarations() {
        // Given
        let class = "class A { shared mut @@count: Integer = 0 }";
        let module = "module M { shared let @@version: Integer = 1 }";

        // When
        let results = [parse(class), parse(module)];

        // Then
        assert!(results.iter().all(|result| result.program_accepted));
        assert!(matches!(
            results[0].program.declarations.as_slice(),
            [Declaration::Class(class)]
                if matches!(class.body.as_slice(), [Statement::SharedBinding { mutable: true, name, .. }] if name == "count")
        ));
        assert!(matches!(
            results[1].program.declarations.as_slice(),
            [Declaration::Module(module)]
                if matches!(module.body.as_slice(), [Statement::SharedBinding { mutable: false, name, .. }] if name == "version")
        ));
    }

    #[test]
    fn parses_private_authorization_on_each_class_and_module_mixin_edge() {
        // Given
        let source = "class A mixin M private, N { }; module B mixin M private, N { }";

        // When
        let result = parse(source);

        // Then
        assert!(result.program_accepted, "{result:#?}");
        assert!(matches!(
            result.program.declarations.as_slice(),
            [Declaration::Class(class), Declaration::Module(module)]
                if class.mixins == [
                    MixinEntry {
                        target: TypeExpression::Name("M".into()),
                        private_access: true,
                    },
                    MixinEntry {
                        target: TypeExpression::Name("N".into()),
                        private_access: false,
                    },
                ] && module.mixins == class.mixins
        ));
    }
    #[test]
    fn parses_stacked_decorators_without_reclassifying_raw_ivars() {
        // Given
        let decorated = "@outer(1) @inner(:tag) class A { @logged() public fun m() { @name } }";

        // When
        let result = parse(decorated);

        // Then
        assert!(result.program_accepted, "{result:#?}");
        assert!(matches!(
            result.program.declarations.as_slice(),
            [Declaration::Class(class)]
                if class.decorators.iter().map(|decorator| decorator.name.as_str()).eq(["outer", "inner"])
        ));
    }

    #[test]
    fn labeled_break_and_match_fallback_parse_without_semantic_checks() {
        let loop_result = parse("outer: while ready { break outer: 1 }");
        let match_result = parse("match value { first if ready => 1, else => 2 }");

        assert!(loop_result.is_clean());
        assert!(match_result.is_clean());
        assert!(matches!(
            loop_result.program.statements.as_slice(),
            [Statement::While { label: Some(label), .. }] if label == "outer"
        ));
        assert!(matches!(
            match_result.program.statements.as_slice(),
            [Statement::Match {
                fallback: Some(_),
                ..
            }]
        ));
    }

    #[test]
    fn raise_and_try_forms_have_statement_nodes() {
        // Given
        let source = "try { raise :boom } catch error: Symbol, context { error } finally { nil }";

        // When
        let result = parse(source);

        // Then
        assert!(result.program_accepted, "{result:#?}");
        assert!(matches!(
            result.program.statements.as_slice(),
            [Statement::Try { catches, finally: Some(_), .. }]
                if matches!(catches.as_slice(), [iris_syntax::CatchClause { binding: Some(iris_syntax::CatchBinding::Name(name)), filter: Some(TypeExpression::Name(filter)), context: Some(context), .. }]
                    if name == "error" && filter == "Symbol" && context == "context")
        ));
    }

    #[test]
    fn try_without_catch_or_finally_is_rejected() {
        // Given
        let source = "try { :value }";

        // When
        let result = parse(source);

        // Then
        assert!(!result.program_accepted);
    }
}
