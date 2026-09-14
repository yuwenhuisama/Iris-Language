use super::*;
use iris_syntax::AssignmentOperator;

impl Continuation {
    pub(super) fn assignment(
        &mut self,
        evaluator: &mut SourceEvaluator,
        operands: Operands,
    ) -> Result<(), EvaluationError> {
        let operands = evaluator.rooted(operands);
        let Expression::Assignment {
            left,
            operator,
            right,
        } = &operands.expression
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let current = match operator {
            AssignmentOperator::Assign => None,
            _ => Some(evaluator.expression(left, &operands.values, self.receiver.clone())?),
        };
        if let Some(current) = &current {
            let writes = match operator {
                AssignmentOperator::LogicalAnd => evaluator.truthy(current.clone())?,
                AssignmentOperator::LogicalOr => !evaluator.truthy(current.clone())?,
                _ => true,
            };
            if !writes {
                self.outcome = Ok(current.clone());
                return Ok(());
            }
        }
        let right = *right.clone();
        self.work.push(Step::Assign(
            Rc::try_unwrap(operands).unwrap_or_else(|_| unreachable!()),
            current,
        ));
        self.work.push(Step::Expression(right));
        Ok(())
    }

    pub(super) fn assign(
        &mut self,
        evaluator: &mut SourceEvaluator,
        operands: Operands,
        current: Option<Value>,
    ) -> Result<(), EvaluationError> {
        let operands = evaluator.rooted(operands);
        let Expression::Assignment { operator, .. } = &operands.expression else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let value = self.value()?;
        let value = match (super::super::compound_selector(operator), current) {
            (Some(selector), Some(current)) => evaluator.send(current, selector, &[value])?,
            _ => value,
        };
        let mut operands = Rc::try_unwrap(operands).unwrap_or_else(|_| unreachable!());
        let Expression::Assignment {
            operator, right, ..
        } = &mut operands.expression
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        *operator = AssignmentOperator::Assign;
        **right = Expression::Name("\0assigned".into());
        operands.values.insert("\0assigned".into(), value);
        let operands = evaluator.rooted(operands);
        self.outcome = evaluator.expression(
            &operands.expression,
            &operands.values,
            self.receiver.clone(),
        );
        Ok(())
    }
}
