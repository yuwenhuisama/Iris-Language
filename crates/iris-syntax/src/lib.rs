//! Syntax tree types shared by the Iris v1 front end.

mod shape;

pub use shape::{PRECEDENCE_ROWS_COVERED, render_parse_shape, render_parse_shapes};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
    pub entries: Vec<ProgramEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramEntry {
    Declaration(Declaration),
    Statement(Statement),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Declaration {
    Class(ClassDeclaration),
    Module(ModuleDeclaration),
    Contract(ContractDeclaration),
    /// `import_decl ::= "import" qualified_type_name import_alias?
    ///                 | "from" qualified_type_name "import" import_spec_list`.
    Import(ImportDeclaration),
    /// `export_decl ::= "export" (declaration | ordinary_name ("," ordinary_name)* ","?)`.
    Export(Box<ExportDeclaration>),
    /// `type_alias_decl ::= "type" type_name generic_params? "=" type_expr`.
    ///
    /// `IRIS-V1-TYPES-C004` makes a Type alias TARGET an annotated boundary, so
    /// the target is retained rather than parsed and discarded.
    TypeAlias(TypeAliasDeclaration),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportDeclaration {
    /// The imported Module path, `qualified_type_name`.
    pub target: String,
    /// The local alias from `import_alias`, when written.
    pub alias: Option<String>,
    /// The `import_spec_list` of a `from ... import ...`, empty otherwise.
    pub specs: Vec<ImportSpec>,
    /// Whether the source wrote the `override` replacement marker.
    ///
    /// `IRIS-V1-META-C049` requires import-site replacement authorization
    /// before a direct import may replace an already merged static extension
    /// member, and the v1.24 errata `IRIS-V1-GRAMMAR-C069` places that marker
    /// before the keyword. `D-230` authorizes the replacements THAT import
    /// contributes, so the flag belongs to the declaration rather than to a
    /// single spec.
    pub replacement_authorized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportSpec {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportDeclaration {
    /// `export <declaration>`, which publishes the declaration it wraps.
    ///
    /// Boxed because a `Declaration` is far larger than a name list, and an
    /// unboxed variant would grow every `ExportDeclaration` to match.
    Declaration(Box<Declaration>),
    /// `export a, b`, which publishes already-declared names.
    Names(Vec<String>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeAliasDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub target: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decorator {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassDeclaration {
    pub decorators: Vec<Decorator>,
    pub reopen: bool,
    pub name: String,
    pub parameters: Vec<String>,
    pub extends: Option<TypeExpression>,
    pub implements: Vec<TypeExpression>,
    pub mixins: Vec<MixinEntry>,
    pub constraints: Vec<Constraint>,
    pub meta_deny: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDeclaration {
    pub decorators: Vec<Decorator>,
    /// Whether the source wrote `open module`.
    ///
    /// `module_decl ::= "open"? "module" ...` admits the marker, and
    /// `IRIS-V1-META-V416` loads an origin and an open revision of one Module
    /// from two files of the same package.
    pub reopen: bool,
    /// Whether the source wrote a `for` clause on this Module.
    ///
    /// `module_decl` admits no `class_for`, but `IRIS-V1-TYPES-V261` expects a
    /// STATIC `CONTRACT_FOR_CLASS_ONLY` rather than a parse error, so the
    /// clause is accepted here and rejected in analysis.
    pub contract_for: Vec<TypeExpression>,
    pub name: String,
    pub parameters: Vec<String>,
    pub mixins: Vec<MixinEntry>,
    pub constraints: Vec<Constraint>,
    pub meta_deny: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDeclaration {
    pub decorators: Vec<Decorator>,
    /// Whether the source wrote `open` before `contract`.
    ///
    /// `IRIS-V1-TYPES-V204` expects a STATIC `OPEN_CONTRACT_FORBIDDEN` rather
    /// than a parse error, so the word is accepted here and rejected in
    /// analysis. `contract_decl` itself admits no `open`.
    pub open: bool,
    pub name: String,
    pub parameters: Vec<String>,
    pub parents: Vec<TypeExpression>,
    pub constraints: Vec<Constraint>,
    pub meta_deny: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Constraint {
    pub parameter: String,
    pub bound: TypeExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MixinEntry {
    pub target: TypeExpression,
    pub private_access: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeExpression {
    Name(String),
    Typeof(Box<Expression>),
    Intersection(Vec<TypeExpression>),
    Union(Vec<TypeExpression>),
    Generic {
        name: String,
        arguments: Vec<TypeExpression>,
    },
    /// `function_type ::= "(" type_expr_list? ")" "->" type_expr`.
    ///
    /// This is the callable Type that `IRIS-V1-CONTROL-C018` writes as
    /// `(P1, P2, ...) -> R` and that `block_parameter` requires.
    Function {
        parameters: Vec<TypeExpression>,
        result: Box<TypeExpression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    /// `global_decl ::= "global" ("let" | "mut") global_name type_annotation? "=" expression`.
    ///
    /// `IRIS-V1-CONTROL-C013` makes `$name` reachable ONLY through a `global
    /// let` or `global mut` declaration, so the binding is recorded rather than
    /// created by use.
    GlobalBinding {
        mutable: bool,
        name: String,
        annotation: Option<TypeExpression>,
        value: Expression,
    },
    SharedBinding {
        mutable: bool,
        name: String,
        /// The declared Type, when the source wrote one.
        ///
        /// `IRIS-V1-TYPES-C004` guards a binding boundary whether the cell is
        /// local or class-level, so the annotation is retained rather than
        /// parsed and discarded.
        annotation: Option<TypeExpression>,
        value: Expression,
    },
    Binding {
        mutable: bool,
        /// Whether the source wrote `const`.
        ///
        /// `IRIS-V1-CONTROL-D-432` puts constants in the SAME qualified
        /// namespace as Class, Module, Contract and Type aliases, so a `const`
        /// must be distinguishable from an ordinary immutable `let`.
        constant: bool,
        name: String,
        /// The declared Type, when the source wrote one.
        ///
        /// `IRIS-V1-CONTROL-C005` makes an annotation a static contract for the
        /// binding cell, and `C004` fixes the binding's Type either way, so the
        /// annotation is retained rather than parsed and discarded.
        annotation: Option<TypeExpression>,
        value: Expression,
    },
    /// A binding declared WITHOUT an initializer.
    ///
    /// `IRIS-V1-CONTROL-D-427` makes this legal only for `mut` with an explicit
    /// type, so the declaration keyword and the presence of an annotation are
    /// both retained for static analysis rather than collapsing to a name.
    DeferredBinding {
        mutable: bool,
        annotated: bool,
        name: String,
    },
    StoredProperty {
        decorators: Vec<Decorator>,
        /// Whether the source wrote `shared` before `property`.
        ///
        /// `IRIS-V1-TYPES-C064` puts a `shared class property` on the
        /// UNAPPLIED generic definition rather than per closed construction,
        /// so the marker is retained rather than parsed and discarded.
        shared: bool,
        /// Whether the property is declared at Class or Module level.
        class_level: bool,
        name: String,
        /// The written Type of the stored slot.
        ///
        /// `C064` forbids a `shared` property from referencing the definition's
        /// type parameters, which can only be checked against what was written.
        annotation: TypeExpression,
        initializer: Expression,
    },
    Method(MethodDeclaration),
    Expression(Expression),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    Return(Option<Expression>),
    Break {
        label: Option<String>,
        value: Option<Expression>,
    },
    Continue(Option<String>),
    While {
        label: Option<String>,
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        label: Option<String>,
        /// `IRIS-V1-CONTROL-C045` lets `for` destructure, so this is a pattern
        /// rather than a plain name.
        binding: Pattern,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
        fallback: Option<MatchBody>,
    },
    Raise(Option<Raise>),
    Try {
        body: Vec<Statement>,
        catches: Vec<CatchClause>,
        finally: Option<Vec<Statement>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Raise {
    pub value: Expression,
    pub cause: Option<Expression>,
    /// Byte offset of the `raise` keyword.
    ///
    /// `IRIS-V1-CONTROL-C065` exposes `raise_location` as the INITIAL raise
    /// source location, and `C079` defines it as a one-based line and column,
    /// so the offset is carried here and converted when the context is built.
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatchClause {
    pub binding: Option<CatchBinding>,
    pub filter: Option<TypeExpression>,
    pub context: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatchBinding {
    Name(String),
    Discard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodDeclaration {
    pub decorators: Vec<Decorator>,
    /// Whether the source wrote the `async` modifier.
    ///
    /// `IRIS-V1-ASYNC-C003` makes an async Method return `Task<T>` rather than
    /// `T`, and `C012` starts its body synchronously until the first
    /// incomplete `await`.
    pub is_async: bool,
    pub is_override: bool,
    /// The Contract this member satisfies, from `impl` or `impl C::member`.
    ///
    /// `IRIS-V1-TYPES-C046` requires `impl` on a member satisfying a declared
    /// Contract requirement, and `IRIS-V1-TYPES-C048` writes a qualified
    /// implementation as `impl C::member`, whose `Some(name)` selects a slot in
    /// the Contract-qualified namespace rather than the ordinary one.
    pub impl_contract: Option<Option<String>>,
    pub kind: MethodKind,
    pub selector: String,
    /// The Method's own generic parameters, from `method_decl`'s
    /// `generic_params?`.
    ///
    /// `IRIS-V1-TYPES-C059` infers these from the call site, so the declared
    /// names are retained rather than parsed and discarded.
    pub type_parameters: Vec<String>,
    pub parameters: Vec<Parameter>,
    /// The written return Type, when the source declared one.
    ///
    /// `IRIS-V1-TYPES-C004` guards the return boundary too, so the annotation
    /// is retained rather than parsed and discarded.
    pub return_type: Option<TypeExpression>,
    pub visibility: Visibility,
    /// The Method body, absent when the source wrote a bodyless requirement.
    ///
    /// `IRIS-V1-GRAMMAR-C062` makes `block_body` optional so a Contract can
    /// state the Method requirements `IRIS-V1-TYPES-C042` already presupposes.
    /// `None` is a REQUIREMENT declaring an obligation with no implementation;
    /// it is distinct from `Some(vec![])`, which is a body that happens to be
    /// empty and still supplies an implementation.
    pub body: Option<Vec<Statement>>,
}

/// One declared parameter with the category `IRIS-V1-CONTROL-C023` gives it.
///
/// The category decides how an argument binds, so it is kept rather than
/// flattened to a name: a rest parameter collects an Array, a keyword binds by
/// name, and an optional one falls back to its default expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub category: ParameterCategory,
    /// The written Type of this parameter, when the source declared one.
    ///
    /// `IRIS-V1-TYPES-C004` makes a parameter annotation a runtime boundary
    /// guard, which can only be enforced against what was actually written, so
    /// the annotation is retained rather than parsed and discarded.
    pub annotation: Option<TypeExpression>,
    pub default: Option<Expression>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterCategory {
    Positional,
    Rest,
    Keyword,
    KeywordRest,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MethodKind {
    Instance,
    Class,
    Module,
    Property,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: MatchBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchBody {
    Expression(Expression),
    Block(Vec<Statement>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    Name(String),
    Alternatives(Vec<Pattern>),
    Literal(String),
    /// `[a, b]` array destructuring from the C051 pattern vocabulary.
    Array(Vec<Pattern>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    /// A parenthesized Type expression reified as a value, as in
    /// `(String | Nil).type`.
    ///
    /// `IRIS-V1-GRAMMAR-C065` admits this only when `.type` follows, which is
    /// what keeps `(a | b)` an ordinary bitwise or.
    ReifiedType(TypeExpression),
    /// A closed generic construction used as a value, as in `Box<String>.new()`.
    ///
    /// `IRIS-V1-GRAMMAR-C063` admits this in `primary_expr`; a BARE generic
    /// name without arguments stays definition metadata under
    /// `IRIS-V1-TYPES-C061` and never becomes one of these.
    ClosedGeneric {
        name: String,
        arguments: Vec<TypeExpression>,
    },
    /// `await expr`, the suspension operator.
    ///
    /// `IRIS-V1-GRAMMAR-C071` binds it at `unary_expr`, so `await f()` awaits
    /// the CALL's result. `IRIS-V1-META-C037` and `IRIS-V1-ASYNC-C018` forbid
    /// it inside an open or revision transaction body.
    Await(Box<Expression>),
    /// `yield expr?`, a generator suspension point.
    ///
    /// `IRIS-V1-GRAMMAR-C072` makes a callable containing one a GENERATOR
    /// whose invocation returns an `Iterator<T>`. `IRIS-V1-META-C037` forbids
    /// it inside a transaction body exactly as it forbids `await`.
    Yield(Option<Box<Expression>>),
    Name(String),
    Literal(String),
    Symbol(String),
    Array(Vec<Expression>),
    Member {
        receiver: Box<Expression>,
        selector: String,
    },
    ContractView {
        receiver: Box<Expression>,
        selector: String,
    },
    /// A `%{ key: value, ... }` Hash literal.
    ///
    /// `IRIS-V1-RUNTIME-C134` rejects a NaN key at CONSTRUCTION, so the entries
    /// stay unevaluated here and the evaluator applies that check.
    Hash(Vec<(Expression, Expression)>),
    /// A `{ |params| body }` closure literal.
    ///
    /// `IRIS-V1-RUNTIME-C042` gives every EVALUATION of this expression a fresh
    /// identity, so the node describes the code and the evaluator allocates a
    /// distinct Closure each time it is reached.
    Closure {
        parameters: Vec<String>,
        /// The `-> Type` header annotation, absent when omitted.
        ///
        /// `IRIS-V1-CONTROL-C017` diagnoses an omitted Closure return
        /// annotation where no unique expected callable type exists, so an
        /// annotated Closure must be distinguishable from a bare one.
        return_type: Option<TypeExpression>,
        /// Whether a `|...|` header was written.
        ///
        /// A header-less `{ ... }` block body shares this node but is not a
        /// Closure literal, and `{ || 7 }` has an EMPTY header rather than
        /// none, so the two cannot be told apart by parameters alone.
        has_header: bool,
        body: Vec<Statement>,
    },
    Call {
        callee: Box<Expression>,
        /// Explicit Method type arguments from `call_type_arguments`.
        ///
        /// `IRIS-V1-GRAMMAR-C066` admits them before the argument list, and
        /// `IRIS-V1-TYPES-C060` makes a missing trailing one an arity error
        /// rather than an inferred default, so the written list is retained.
        type_arguments: Vec<TypeExpression>,
        arguments: Vec<Expression>,
    },
    /// `receiver[index]`, the index read of `IRIS-V1-COLLECTIONS-C051`.
    ///
    /// It is a postfix part rather than an Array literal in argument position,
    /// so `a[0]` is one expression instead of the two statements `a` and `[0]`.
    Index {
        receiver: Box<Expression>,
        index: Box<Expression>,
    },
    /// `name: value` in an argument list, the keyword channel of
    /// `IRIS-V1-CONTROL-C023`.
    ///
    /// A keyword argument is an argument rather than a separate list, so it
    /// stays in `arguments` and keeps the left-to-right evaluation order
    /// `IRIS-V1-CONTROL-C026` requires across both channels.
    KeywordArgument {
        name: String,
        value: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Assignment {
        left: Box<Expression>,
        operator: AssignmentOperator,
        right: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    /// `while condition { body }` in expression position.
    ///
    /// `IRIS-V1-CONTROL-C043` gives a loop a VALUE: natural completion yields
    /// `nil` and `break expr` yields `expr`, so `IRIS-V1-CONTROL-V355` can read
    /// the result of a loop.
    While {
        label: Option<String>,
        condition: Box<Expression>,
        body: Vec<Statement>,
    },
    /// `try { ... } catch ... finally { ... }` in expression position.
    ///
    /// `IRIS-V1-CONTROL-V311` through `V313` read the result of a `try`, so it
    /// is value-producing like `if` rather than a statement-only form.
    Try {
        body: Vec<Statement>,
        catches: Vec<CatchClause>,
        finally: Option<Vec<Statement>>,
    },
    Grouped(Box<Expression>),
    /// `()`, `(a,)`, `(a, b, ...)`.
    ///
    /// `IRIS-V1-COLLECTIONS-C021` makes a Tuple an immutable identity-less
    /// heterogeneous product value, distinct from a parenthesized expression:
    /// `(a)` groups, `(a,)` is a one-element Tuple.
    Tuple(Vec<Expression>),
    RawIvar(String),
    ClassVar(String),
    /// `$name`, a declared package-qualified runtime global.
    ///
    /// `IRIS-V1-CONTROL-C013` makes a missing one a declaration error rather
    /// than creating the cell.
    GlobalVar(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Negate,
    BitwiseNot,
    Not,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Power,
    Multiply,
    Divide,
    Add,
    Subtract,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    RangeInclusive,
    RangeExclusive,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Compare,
    /// `=~`, the match operator `IRIS-V1-GRAMMAR-C016` lists.
    Match,
    /// `!~`, the negated match operator.
    NotMatch,
    RegexMatches,
    RegexDoesNotMatch,
    Is,
    As,
    AsOptional,
    Equal,
    NotEqual,
    NamedInfix {
        selector: String,
    },
    Identity,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignmentOperator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    LogicalAnd,
    LogicalOr,
}

#[cfg(test)]
mod tests {
    use super::{
        AssignmentOperator, BinaryOperator, ClassDeclaration, Constraint, ContractDeclaration,
        Declaration, Expression, PRECEDENCE_ROWS_COVERED, Program, Statement, TypeExpression,
        UnaryOperator, render_parse_shape, render_parse_shapes,
    };

    #[test]
    fn expression_nodes_preserve_power_and_unary_shape() {
        let power = Expression::Binary {
            left: Box::new(Expression::Literal("2".into())),
            operator: BinaryOperator::Power,
            right: Box::new(Expression::Literal("2".into())),
        };
        let expression = Expression::Unary {
            operator: UnaryOperator::Negate,
            operand: Box::new(power),
        };

        assert!(matches!(expression, Expression::Unary { .. }));
    }

    #[test]
    fn render_parse_shapes_renders_v003_byte_exactly() {
        let program = Program {
            declarations: Vec::new(),
            statements: vec![
                Statement::Expression(power("2", power("3", leaf("2")))),
                Statement::Expression(Expression::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(power("2", leaf("2"))),
                }),
                Statement::Expression(power(
                    "2",
                    Expression::Unary {
                        operator: UnaryOperator::Negate,
                        operand: Box::new(leaf("3")),
                    },
                )),
            ],
            entries: Vec::new(),
        };

        assert_eq!(
            render_parse_shapes(&program),
            ["2 ** (3 ** 2)", "-(2 ** 2)", "2 ** (-3)"]
        );
    }

    #[test]
    fn renders_v163_class_where_constraints_byte_exactly() {
        // Given
        let program = Program {
            declarations: vec![Declaration::Class(ClassDeclaration {
                decorators: Vec::new(),
                reopen: false,
                name: "Pair".into(),
                parameters: vec!["T".into(), "U".into()],
                extends: None,
                implements: Vec::new(),
                mixins: Vec::new(),
                constraints: vec![
                    Constraint {
                        parameter: "T".into(),
                        bound: TypeExpression::Intersection(vec![
                            TypeExpression::Name("A".into()),
                            TypeExpression::Name("B".into()),
                        ]),
                    },
                    Constraint {
                        parameter: "U".into(),
                        bound: TypeExpression::Name("C".into()),
                    },
                ],
                meta_deny: Vec::new(),
                body: Vec::new(),
            })],
            statements: Vec::new(),
            entries: Vec::new(),
        };

        // When
        let shapes = render_parse_shapes(&program);

        // Then
        assert_eq!(shapes, ["constraints(T: A & B, U: C)"]);
    }

    #[test]
    fn renders_v192_contract_parents_byte_exactly() {
        // Given
        let program = Program {
            declarations: vec![Declaration::Contract(ContractDeclaration {
                decorators: Vec::new(),
                open: false,
                name: "Child".into(),
                parameters: Vec::new(),
                parents: vec![
                    TypeExpression::Name("ParentA".into()),
                    TypeExpression::Name("ParentB".into()),
                ],
                constraints: Vec::new(),
                meta_deny: Vec::new(),
                body: Vec::new(),
            })],
            statements: Vec::new(),
            entries: Vec::new(),
        };

        // When
        let shapes = render_parse_shapes(&program);

        // Then
        assert_eq!(shapes, ["Contract(name=Child, extends=[ParentA, ParentB])"]);
    }

    #[test]
    fn renders_v193_class_roots_byte_exactly() {
        // Given
        let program = Program {
            declarations: vec![
                Declaration::Class(empty_class("ImplicitRoot", None)),
                Declaration::Class(empty_class(
                    "ExplicitRoot",
                    Some(TypeExpression::Name("Object".into())),
                )),
            ],
            statements: Vec::new(),
            entries: Vec::new(),
        };

        // When
        let shapes = render_parse_shapes(&program);

        // Then
        assert_eq!(
            shapes,
            [
                "Class(name=ImplicitRoot, extends=absent)",
                "Class(name=ExplicitRoot, extends=Object)",
            ]
        );
    }

    #[test]
    fn render_parse_shape_renders_conformance_v011_chain_byte_exactly() {
        let chain = assign(
            logical_or(logical_and(named_infix(
                equality(
                    relational(
                        range(
                            bit_or(
                                bit_xor(
                                    bit_and(
                                        shift(
                                            add(mul(power("a.b(c)[d]", unary("e")), "f"), "g"),
                                            "h",
                                        ),
                                        "i",
                                    ),
                                    "j",
                                ),
                                "k",
                            ),
                            "l",
                        ),
                        "m",
                    ),
                    "n",
                ),
                "o",
            ))),
            "r",
        );

        assert_eq!(
            render_parse_shape(&chain),
            "assign(q_or_chain(logical_or(logical_and(named_infix(equality(relational(range(bit_or(bit_xor(bit_and(shift(add(mul(pow(primary_chain(a.b(c)[d]), unary(-e)), f), g), h), i), j), k), l), m), n), named, o), p)), r)"
        );
        assert_eq!(PRECEDENCE_ROWS_COVERED, 17);
    }

    #[test]
    fn render_parse_shapes_rejects_one_character_mutation() {
        let rendered = render_parse_shapes(&Program {
            declarations: Vec::new(),
            statements: vec![Statement::Expression(power("2", power("3", leaf("2"))))],
            entries: Vec::new(),
        });

        assert_ne!(rendered[0], "2 ** (3 ** 3)");
    }

    #[test]
    fn render_parse_shapes_renders_new_expression_forms() {
        // Given
        let program = Program {
            declarations: Vec::new(),
            statements: vec![Statement::Expression(Expression::Call {
                callee: Box::new(Expression::Member {
                    receiver: Box::new(Expression::Name("Float64".into())),
                    selector: "from_bits".into(),
                }),
                type_arguments: Vec::new(),
                arguments: vec![Expression::Symbol("zero".into())],
            })],
            entries: Vec::new(),
        };

        // When
        let shapes = render_parse_shapes(&program);

        // Then
        assert_eq!(shapes, ["Float64.from_bits(:zero)"]);
    }

    fn leaf(value: &str) -> Expression {
        Expression::Name(value.into())
    }

    fn empty_class(name: &str, extends: Option<TypeExpression>) -> ClassDeclaration {
        ClassDeclaration {
            decorators: Vec::new(),
            reopen: false,
            name: name.into(),
            parameters: Vec::new(),
            extends,
            implements: Vec::new(),
            mixins: Vec::new(),
            constraints: Vec::new(),
            meta_deny: Vec::new(),
            body: Vec::new(),
        }
    }

    fn unary(value: &str) -> Expression {
        Expression::Unary {
            operator: UnaryOperator::Negate,
            operand: Box::new(leaf(value)),
        }
    }

    fn binary(left: Expression, operator: BinaryOperator, right: Expression) -> Expression {
        Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    fn power(left: &str, right: Expression) -> Expression {
        binary(leaf(left), BinaryOperator::Power, right)
    }
    fn mul(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::Multiply, leaf(right))
    }
    fn add(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::Add, leaf(right))
    }
    fn shift(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::ShiftLeft, leaf(right))
    }
    fn bit_and(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::BitwiseAnd, leaf(right))
    }
    fn bit_xor(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::BitwiseXor, leaf(right))
    }
    fn bit_or(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::BitwiseOr, leaf(right))
    }
    fn range(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::RangeExclusive, leaf(right))
    }
    fn relational(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::Less, leaf(right))
    }
    fn equality(left: Expression, right: &str) -> Expression {
        binary(left, BinaryOperator::Equal, leaf(right))
    }
    fn named_infix(left: Expression, right: &str) -> Expression {
        binary(
            left,
            BinaryOperator::NamedInfix {
                selector: "named".into(),
            },
            leaf(right),
        )
    }
    fn logical_and(left: Expression) -> Expression {
        binary(left, BinaryOperator::LogicalAnd, leaf("p"))
    }
    fn logical_or(left: Expression) -> Expression {
        binary(left, BinaryOperator::LogicalOr, leaf("q"))
    }
    fn assign(left: Expression, right: &str) -> Expression {
        Expression::Assignment {
            left: Box::new(left),
            operator: AssignmentOperator::Assign,
            right: Box::new(leaf(right)),
        }
    }
}
