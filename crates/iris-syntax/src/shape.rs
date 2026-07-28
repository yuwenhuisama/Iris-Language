use crate::{
    AssignmentOperator, BinaryOperator, Declaration, Expression, Program, Statement,
    TypeExpression, UnaryOperator,
};

/// Number of precedence rows covered by the CONFORMANCE-V011 structural fixture.
pub const PRECEDENCE_ROWS_COVERED: u8 = 17;

/// Renders the source-form `expect.artifact.parse_shapes` observations.
#[must_use]
pub fn render_parse_shapes(program: &Program) -> Vec<String> {
    let mut shapes = program
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Class(value) if !value.constraints.is_empty() => Some(format!(
                "constraints({})",
                value
                    .constraints
                    .iter()
                    .map(|constraint| format!(
                        "{}: {}",
                        constraint.parameter,
                        type_expression_shape(&constraint.bound)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Declaration::Class(_) | Declaration::Module(_) | Declaration::Contract(_) => None,
        })
        .collect::<Vec<_>>();
    shapes.extend(
        program
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Expression(expression) => Some(source_shape(expression, 0)),
                Statement::Return(_)
                | Statement::Break { .. }
                | Statement::Continue(_)
                | Statement::While { .. }
                | Statement::For { .. }
                | Statement::Match { .. } => None,
            }),
    );
    shapes
}

/// Renders the structural `expect.artifact.parse_shape` observation.
#[must_use]
pub fn render_parse_shape(expression: &Expression) -> String {
    structural_shape(expression)
}

fn source_shape(expression: &Expression, enclosing_precedence: u8) -> String {
    let (shape, precedence) = match expression {
        Expression::Name(value) | Expression::Literal(value) => (value.clone(), 17),
        Expression::Grouped(value) => (format!("({})", source_shape(value, 0)), 17),
        Expression::Unary { operator, operand } => (
            format!("{}{}", unary_operator(*operator), source_shape(operand, 16)),
            14,
        ),
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let precedence = binary_precedence(*operator);
            let right_precedence = precedence + 1;
            (
                format!(
                    "{} {} {}",
                    source_shape(left, precedence + 1),
                    binary_operator(*operator),
                    source_shape(right, right_precedence)
                ),
                precedence,
            )
        }
        Expression::Assignment {
            left,
            operator,
            right,
        } => (
            format!(
                "{} {} {}",
                source_shape(left, 2),
                assignment_operator(*operator),
                source_shape(right, 1)
            ),
            1,
        ),
    };
    if precedence < enclosing_precedence {
        format!("({shape})")
    } else {
        shape
    }
}

fn structural_shape(expression: &Expression) -> String {
    match expression {
        Expression::Name(value) | Expression::Literal(value) => primary_shape(value),
        Expression::Grouped(value) => structural_shape(value),
        Expression::Unary { operator, operand } => format!(
            "unary({}{})",
            unary_operator(*operator),
            structural_shape(operand)
        ),
        Expression::Binary {
            left,
            operator,
            right,
        } => match operator {
            BinaryOperator::NamedInfix => format!(
                "named_infix({}, named, {})",
                structural_shape(left),
                structural_shape(right)
            ),
            _ => format!(
                "{}({}, {})",
                structural_operator(*operator),
                structural_shape(left),
                structural_shape(right)
            ),
        },
        Expression::Assignment {
            left,
            operator,
            right,
        } => {
            let left = structural_assignment_left(left);
            format!(
                "{}({left}, {})",
                structural_assignment_operator(*operator),
                structural_shape(right)
            )
        }
    }
}

fn structural_assignment_left(expression: &Expression) -> String {
    match expression {
        Expression::Binary {
            left,
            operator: BinaryOperator::LogicalOr,
            ..
        } => format!("q_or_chain(logical_or({})", structural_shape(left)),
        _ => structural_shape(expression),
    }
}

fn primary_shape(value: &str) -> String {
    if value.contains(['.', '(', '[']) {
        format!("primary_chain({value})")
    } else {
        value.into()
    }
}

fn type_expression_shape(value: &TypeExpression) -> String {
    match value {
        TypeExpression::Name(value) => value.clone(),
        TypeExpression::Intersection(values) => values
            .iter()
            .map(type_expression_shape)
            .collect::<Vec<_>>()
            .join(" & "),
        TypeExpression::Union(values) => values
            .iter()
            .map(type_expression_shape)
            .collect::<Vec<_>>()
            .join(" | "),
        TypeExpression::Generic { name, arguments } => format!(
            "{}<{}>",
            name,
            arguments
                .iter()
                .map(type_expression_shape)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

const fn unary_operator(operator: UnaryOperator) -> &'static str {
    match operator {
        UnaryOperator::Plus => "+",
        UnaryOperator::Negate => "-",
        UnaryOperator::BitwiseNot => "~",
        UnaryOperator::Not => "!",
    }
}

const fn binary_precedence(operator: BinaryOperator) -> u8 {
    match operator {
        BinaryOperator::Power => 15,
        BinaryOperator::Multiply | BinaryOperator::Divide => 13,
        BinaryOperator::Add | BinaryOperator::Subtract => 12,
        BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight => 11,
        BinaryOperator::BitwiseAnd => 10,
        BinaryOperator::BitwiseXor => 9,
        BinaryOperator::BitwiseOr => 8,
        BinaryOperator::RangeInclusive | BinaryOperator::RangeExclusive => 7,
        BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::Compare
        | BinaryOperator::RegexMatches
        | BinaryOperator::RegexDoesNotMatch
        | BinaryOperator::Is
        | BinaryOperator::As
        | BinaryOperator::AsOptional => 6,
        BinaryOperator::Equal | BinaryOperator::NotEqual => 5,
        BinaryOperator::NamedInfix => 4,
        BinaryOperator::LogicalAnd => 3,
        BinaryOperator::LogicalOr => 2,
    }
}

const fn binary_operator(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Power => "**",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::ShiftLeft => "<<",
        BinaryOperator::ShiftRight => ">>",
        BinaryOperator::BitwiseAnd => "&",
        BinaryOperator::BitwiseXor => "^",
        BinaryOperator::BitwiseOr => "|",
        BinaryOperator::RangeInclusive => "..=",
        BinaryOperator::RangeExclusive => "..<",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Compare => "<=>",
        BinaryOperator::RegexMatches => "=~",
        BinaryOperator::RegexDoesNotMatch => "!~",
        BinaryOperator::Is => "is",
        BinaryOperator::As => "as",
        BinaryOperator::AsOptional => "as?",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::NamedInfix => "named",
        BinaryOperator::LogicalAnd => "&&",
        BinaryOperator::LogicalOr => "||",
    }
}

const fn structural_operator(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Power => "pow",
        BinaryOperator::Multiply | BinaryOperator::Divide => "mul",
        BinaryOperator::Add | BinaryOperator::Subtract => "add",
        BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight => "shift",
        BinaryOperator::BitwiseAnd => "bit_and",
        BinaryOperator::BitwiseXor => "bit_xor",
        BinaryOperator::BitwiseOr => "bit_or",
        BinaryOperator::RangeInclusive | BinaryOperator::RangeExclusive => "range",
        BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::Compare
        | BinaryOperator::RegexMatches
        | BinaryOperator::RegexDoesNotMatch
        | BinaryOperator::Is
        | BinaryOperator::As
        | BinaryOperator::AsOptional => "relational",
        BinaryOperator::Equal | BinaryOperator::NotEqual => "equality",
        BinaryOperator::NamedInfix => "named_infix",
        BinaryOperator::LogicalAnd => "logical_and",
        BinaryOperator::LogicalOr => "logical_or",
    }
}

const fn assignment_operator(operator: AssignmentOperator) -> &'static str {
    match operator {
        AssignmentOperator::Assign => "=",
        AssignmentOperator::Add => "+=",
        AssignmentOperator::Subtract => "-=",
        AssignmentOperator::Multiply => "*=",
        AssignmentOperator::Divide => "/=",
        AssignmentOperator::Power => "**=",
        AssignmentOperator::BitwiseAnd => "&=",
        AssignmentOperator::BitwiseOr => "|=",
        AssignmentOperator::BitwiseXor => "^=",
        AssignmentOperator::ShiftLeft => "<<=",
        AssignmentOperator::ShiftRight => ">>=",
        AssignmentOperator::LogicalAnd => "&&=",
        AssignmentOperator::LogicalOr => "||=",
    }
}

const fn structural_assignment_operator(operator: AssignmentOperator) -> &'static str {
    match operator {
        AssignmentOperator::Assign => "assign",
        AssignmentOperator::Add => "assign_add",
        AssignmentOperator::Subtract => "assign_subtract",
        AssignmentOperator::Multiply => "assign_multiply",
        AssignmentOperator::Divide => "assign_divide",
        AssignmentOperator::Power => "assign_power",
        AssignmentOperator::BitwiseAnd => "assign_bit_and",
        AssignmentOperator::BitwiseOr => "assign_bit_or",
        AssignmentOperator::BitwiseXor => "assign_bit_xor",
        AssignmentOperator::ShiftLeft => "assign_shift_left",
        AssignmentOperator::ShiftRight => "assign_shift_right",
        AssignmentOperator::LogicalAnd => "assign_logical_and",
        AssignmentOperator::LogicalOr => "assign_logical_or",
    }
}
