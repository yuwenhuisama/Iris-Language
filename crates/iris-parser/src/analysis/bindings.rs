use super::{Analyzer, Control, StaticType, TypeMember};
use crate::Diagnostic;
use iris_syntax::{Expression, Program, ProgramEntry, TypeAliasDeclaration, TypeExpression};

mod expressions;
mod statements;

/// Builds an execution-only AST with initialized locals' effective lexical contracts.
/// Written contracts resolve transparent aliases; unknown initializers stay unannotated.
/// Both ordered entries and the duplicated declaration/statement collections agree.
///
/// # Errors
/// Returns only `BINDING_FIXED_LOCAL_TYPE`; other diagnostics remain backend policy.
pub fn prepare_bindings(program: &Program) -> Result<Program, Vec<Diagnostic>> {
    prepare_bindings_with_context(program, &[])
}

/// Seeds lexical inference from existing cells' static contracts, never their values.
/// `None` preserves an unknown type. Mutability enforcement remains backend policy.
///
/// # Errors
/// Returns only `BINDING_FIXED_LOCAL_TYPE`, including violations of seeded contracts.
pub fn prepare_bindings_with_context(
    program: &Program,
    contracts: &[(String, Option<TypeExpression>)],
) -> Result<Program, Vec<Diagnostic>> {
    prepare_bindings_with_aliases(program, contracts, &[])
}

/// Prepares a chunk using previously declared aliases and cells' static contracts.
/// Alias declarations remain source-shaped; execution annotations use their targets.
/// Cyclic and unresolved aliases retain their written form and existing diagnostic policy.
///
/// # Errors
/// Returns only `BINDING_FIXED_LOCAL_TYPE`, including violations of seeded contracts.
pub fn prepare_bindings_with_aliases(
    program: &Program,
    contracts: &[(String, Option<TypeExpression>)],
    aliases: &[TypeAliasDeclaration],
) -> Result<Program, Vec<Diagnostic>> {
    let mut prepared = program.clone();
    let mut resolver = super::aliases::Aliases::new(program, aliases);
    resolver.program(&mut prepared);
    let mut contracts = contracts.to_vec();
    for (_, contract) in &mut contracts {
        if let Some(contract) = contract {
            resolver.annotation(contract);
        }
    }
    let mut validation = Analyzer::with_binding_context(&contracts);
    validation.program(&prepared);
    let diagnostics: Vec<_> = validation
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.code == "BINDING_FIXED_LOCAL_TYPE")
        .collect();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut analyzer = Analyzer::with_binding_context(&contracts);
    for entry in &mut prepared.entries {
        match entry {
            ProgramEntry::Statement(statement) => analyzer.prepare_statement(statement),
            ProgramEntry::Declaration(declaration) => analyzer.prepare_declaration(declaration),
        }
    }
    super::aliases::synchronize(&mut prepared);
    Ok(prepared)
}

impl StaticType {
    pub(super) fn contract(&self) -> TypeExpression {
        let members: Vec<_> = self
            .0
            .iter()
            .map(|member| {
                TypeExpression::Name(
                    match member {
                        TypeMember::Bool => "Bool",
                        TypeMember::Integer => "Integer",
                        TypeMember::Float32 => "Float32",
                        TypeMember::Float64 => "Float64",
                        TypeMember::String => "String",
                        TypeMember::Symbol => "Symbol",
                        TypeMember::Nil => "Nil",
                        TypeMember::Array => "Array",
                        TypeMember::Hash => "Hash",
                    }
                    .into(),
                )
            })
            .collect();
        match members.as_slice() {
            [only] => only.clone(),
            _ => TypeExpression::Union(members),
        }
    }
}

impl Analyzer {
    fn with_binding_context(contracts: &[(String, Option<TypeExpression>)]) -> Self {
        let mut analyzer = Self {
            scopes: vec![Vec::new()],
            ..Self::default()
        };
        for (name, contract) in contracts {
            analyzer.declare_typed(
                name,
                true,
                contract
                    .as_ref()
                    .and_then(|value| analyzer.annotation_type(value)),
            );
            if let Some(local) = analyzer
                .scopes
                .last_mut()
                .and_then(|scope| scope.last_mut())
            {
                local.contract = contract.clone();
            }
        }
        analyzer
    }

    pub(super) fn declare_parameter(&mut self, parameter: &iris_syntax::Parameter) {
        let contract = match parameter.category {
            iris_syntax::ParameterCategory::Rest => Some(TypeExpression::Name("Array".into())),
            iris_syntax::ParameterCategory::KeywordRest => {
                Some(TypeExpression::Name("Hash".into()))
            }
            iris_syntax::ParameterCategory::Positional
            | iris_syntax::ParameterCategory::Keyword => parameter.annotation.clone(),
            iris_syntax::ParameterCategory::Block => parameter.annotation.clone().map(|value| {
                TypeExpression::Union(vec![value, TypeExpression::Name("Nil".into())])
            }),
        };
        self.declare_typed(
            &parameter.name,
            false,
            contract
                .as_ref()
                .and_then(|value| self.annotation_type(value)),
        );
        if let Some(local) = self.scopes.last_mut().and_then(|scope| scope.last_mut()) {
            local.contract = contract;
        }
    }

    pub(super) fn catch_body(&mut self, catch: &iris_syntax::CatchClause, control: Control) {
        self.scopes.push(Vec::new());
        self.declare_catch(catch);
        for statement in &catch.body {
            self.statement(statement, control);
        }
        self.scopes.pop();
    }

    fn declare_catch(&mut self, catch: &iris_syntax::CatchClause) {
        if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
            self.declare_typed(
                name,
                false,
                catch
                    .filter
                    .as_ref()
                    .and_then(|filter| self.annotation_type(filter)),
            );
        }
        if let Some(context) = &catch.context {
            self.declare(context, false);
        }
    }

    pub(super) fn prepare_catch(&mut self, catch: &mut iris_syntax::CatchClause) {
        self.scopes.push(Vec::new());
        self.declare_catch(catch);
        for statement in &mut catch.body {
            self.prepare_statement(statement);
        }
        self.scopes.pop();
    }

    pub(super) fn inferred_contract(&self, expression: &Expression) -> Option<TypeExpression> {
        match expression {
            Expression::Name(name) => self
                .lookup(name)
                .and_then(|local| local.contract.clone())
                .or_else(|| {
                    self.expression_type(expression)
                        .map(|value| value.contract())
                }),
            Expression::Grouped(inner) => self.inferred_contract(inner),
            _ => self
                .expression_type(expression)
                .map(|value| value.contract()),
        }
    }
}
