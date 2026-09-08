use super::{Analyzer, StaticType, TypeMember};
use iris_syntax::{AssignmentOperator, BinaryOperator, Expression};

impl Analyzer {
    pub(super) fn assignment_type(
        &self,
        left: &Expression,
        operator: AssignmentOperator,
        right: &Expression,
    ) -> Option<StaticType> {
        match left {
            Expression::Member { .. } | Expression::Index { .. } => return None,
            Expression::Grouped(inner) => return self.assignment_type(inner, operator, right),
            _ => {}
        }
        let binary = match operator {
            AssignmentOperator::Assign => return self.expression_type(right),
            AssignmentOperator::Add => BinaryOperator::Add,
            AssignmentOperator::Subtract => BinaryOperator::Subtract,
            AssignmentOperator::Multiply => BinaryOperator::Multiply,
            AssignmentOperator::Divide => BinaryOperator::Divide,
            AssignmentOperator::Power => BinaryOperator::Power,
            AssignmentOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
            AssignmentOperator::BitwiseOr => BinaryOperator::BitwiseOr,
            AssignmentOperator::BitwiseXor => BinaryOperator::BitwiseXor,
            AssignmentOperator::ShiftLeft => BinaryOperator::ShiftLeft,
            AssignmentOperator::ShiftRight => BinaryOperator::ShiftRight,
            AssignmentOperator::LogicalAnd => BinaryOperator::LogicalAnd,
            AssignmentOperator::LogicalOr => BinaryOperator::LogicalOr,
        };
        self.binary_type(left, &binary, right)
    }

    pub(super) fn binary_type(
        &self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
    ) -> Option<StaticType> {
        match operator {
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => Some(StaticType::union(
                &self.expression_type(left)?,
                &self.expression_type(right)?,
            )),
            BinaryOperator::Is
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Identity => Some(StaticType::single(TypeMember::Bool)),
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Power
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight => {
                let exponent = right;
                let left = self.expression_type(left)?;
                let right = self.expression_type(right)?;
                if left.0.len() == 1
                    && right.0.len() == 1
                    && left.0.iter().chain(&right.0).all(|member| {
                        matches!(
                            member,
                            TypeMember::Integer | TypeMember::Float32 | TypeMember::Float64
                        )
                    })
                {
                    let integer = StaticType::single(TypeMember::Integer);
                    if matches!(operator, BinaryOperator::Divide)
                        && left == integer
                        && right == integer
                    {
                        return Some(StaticType::single(TypeMember::Float64));
                    }
                    if matches!(operator, BinaryOperator::Power)
                        && left == integer
                        && right == integer
                    {
                        if matches!(exponent, Expression::Literal(_)) {
                            return Some(integer);
                        }
                        return Some(StaticType::union(
                            &integer,
                            &StaticType::single(TypeMember::Float64),
                        ));
                    }
                    let member = if left.0.contains(&TypeMember::Float64)
                        || right.0.contains(&TypeMember::Float64)
                    {
                        TypeMember::Float64
                    } else if left.0.contains(&TypeMember::Float32)
                        || right.0.contains(&TypeMember::Float32)
                    {
                        TypeMember::Float32
                    } else {
                        TypeMember::Integer
                    };
                    return Some(StaticType::single(member));
                }
                if matches!(operator, BinaryOperator::Add)
                    && left == right
                    && left == StaticType::single(TypeMember::String)
                {
                    return Some(left);
                }
                None
            }
            BinaryOperator::As
            | BinaryOperator::AsOptional
            | BinaryOperator::RangeInclusive
            | BinaryOperator::RangeExclusive
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Compare
            | BinaryOperator::Match
            | BinaryOperator::NotMatch
            | BinaryOperator::RegexMatches
            | BinaryOperator::RegexDoesNotMatch
            | BinaryOperator::NamedInfix { .. } => None,
        }
    }
}
