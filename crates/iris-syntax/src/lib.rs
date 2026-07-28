//! Syntax tree types shared by the Iris v1 front end.

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    NamedInfix,
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
    use super::{BinaryOperator, Expression, UnaryOperator};

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
}
