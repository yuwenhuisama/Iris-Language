use crate::source::{ExpressionFact, RangeOperator, Span, SyntaxId};
use iris_syntax::BinaryOperator;

pub(crate) fn binary_fact(
    operator: &BinaryOperator,
    operands: Option<(SyntaxId, SyntaxId)>,
    operator_span: Span,
) -> ExpressionFact {
    let range = match operator {
        BinaryOperator::RangeInclusive => Some(RangeOperator::Inclusive),
        BinaryOperator::RangeExclusive => Some(RangeOperator::Exclusive),
        BinaryOperator::Power
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseXor
        | BinaryOperator::BitwiseOr
        | BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::Compare
        | BinaryOperator::Match
        | BinaryOperator::NotMatch
        | BinaryOperator::RegexMatches
        | BinaryOperator::RegexDoesNotMatch
        | BinaryOperator::Is
        | BinaryOperator::As
        | BinaryOperator::AsOptional
        | BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::NamedInfix { .. }
        | BinaryOperator::Identity
        | BinaryOperator::LogicalAnd
        | BinaryOperator::LogicalOr => None,
    };
    match range.zip(operands) {
        Some((operator, (start, end))) => ExpressionFact::Range {
            start,
            end,
            operator,
            operator_span,
        },
        None => ExpressionFact::Unsupported { form: "binary" },
    }
}
