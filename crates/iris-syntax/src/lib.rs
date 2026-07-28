//! Syntax tree types shared by the Iris v1 front end.

mod shape;

pub use shape::{PRECEDENCE_ROWS_COVERED, render_parse_shape, render_parse_shapes};

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Program {
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Declaration {
    Class(ClassDeclaration),
    Module(ModuleDeclaration),
    Contract(ContractDeclaration),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub extends: Option<TypeExpression>,
    pub implements: Vec<TypeExpression>,
    pub mixins: Vec<TypeExpression>,
    pub constraints: Vec<Constraint>,
    pub meta_deny: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub mixins: Vec<TypeExpression>,
    pub constraints: Vec<Constraint>,
    pub meta_deny: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDeclaration {
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
pub enum TypeExpression {
    Name(String),
    Intersection(Vec<TypeExpression>),
    Union(Vec<TypeExpression>),
    Generic {
        name: String,
        arguments: Vec<TypeExpression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Binding {
        name: String,
        value: Expression,
    },
    Method(MethodDeclaration),
    Expression(Expression),
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
        binding: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
        fallback: Option<MatchBody>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodDeclaration {
    pub selector: String,
    pub parameters: Vec<String>,
    pub visibility: Visibility,
    pub body: Vec<Statement>,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
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
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
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
    Grouped(Box<Expression>),
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
    RegexMatches,
    RegexDoesNotMatch,
    Is,
    As,
    AsOptional,
    Equal,
    NotEqual,
    NamedInfix { selector: String },
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
                arguments: vec![Expression::Symbol("zero".into())],
            })],
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
