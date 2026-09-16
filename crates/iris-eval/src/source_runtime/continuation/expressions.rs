use super::*;

impl Continuation {
    pub(super) fn expression(
        &mut self,
        evaluator: &mut SourceEvaluator,
        expression: Expression,
    ) -> Result<(), EvaluationError> {
        match expression {
            Expression::Await(operand) => {
                if evaluator.open_target.is_some() {
                    return Err(EvaluationError::MetaTransactionSuspension);
                }
                self.work.push(Step::Await);
                self.work.push(Step::Expression(*operand));
            }
            Expression::Grouped(operand) => {
                self.work.push(Step::Expression(*operand));
            }
            Expression::NonNull(operand) => {
                self.work.push(Step::NonNull);
                self.work.push(Step::Expression(*operand));
            }
            Expression::SafeNavigation { receiver, parts } => {
                self.work.push(Step::SafeNavigation(parts, 0));
                self.work.push(Step::Expression(*receiver));
            }
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                self.work.push(Step::Branch(then_body, else_body));
                self.work.push(Step::Expression(*condition));
            }
            Expression::While {
                label,
                condition,
                body,
            } => self.work.push(Step::While(Loop {
                label,
                condition: *condition,
                body,
            })),
            Expression::Try {
                body,
                catches,
                finally,
            } => {
                self.work.push(Step::Try(catches, finally));
                self.enter(evaluator, body, self.locals());
            }
            Expression::Binary {
                left,
                operator:
                    operator @ (iris_syntax::BinaryOperator::LogicalAnd
                    | iris_syntax::BinaryOperator::LogicalOr),
                right,
            } => {
                self.work.push(Step::Logical(operator, *right));
                self.work.push(Step::Expression(*left));
            }
            mut expression => {
                let mut pending = std::collections::VecDeque::new();
                self.prepare(evaluator, &mut expression, &mut pending);
                self.work.push(Step::Operands(Operands {
                    expression,
                    pending,
                    values: HashMap::new(),
                }));
            }
        }
        Ok(())
    }

    pub(super) fn safe_navigation(
        &mut self,
        evaluator: &mut SourceEvaluator,
        parts: Vec<iris_syntax::PostfixPart>,
        position: usize,
    ) -> Result<(), EvaluationError> {
        let current = self.value()?;
        if current == Value::Nil || position == parts.len() {
            self.outcome = Ok(current);
            return Ok(());
        }
        match parts[position].clone() {
            iris_syntax::PostfixPart::Member { selector, .. }
                if matches!(
                    parts.get(position + 1),
                    Some(iris_syntax::PostfixPart::Call { .. })
                ) =>
            {
                let iris_syntax::PostfixPart::Call {
                    type_arguments,
                    mut arguments,
                } = parts[position + 1].clone()
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let name = format!("\0safe{}", self.temporary);
                self.temporary += 1;
                if let Some(mut locals) = self.scopes.last_mut() {
                    locals.insert(name.clone(), current);
                }
                let mut next = position + 2;
                if let Some(iris_syntax::PostfixPart::TrailingBlock(block)) = parts.get(next) {
                    arguments.push((**block).clone());
                    next += 1;
                }
                self.work.push(Step::SafeNavigation(parts, next));
                self.work.push(Step::RemoveTemporary(name.clone()));
                self.work.push(Step::Expression(Expression::Call {
                    callee: Box::new(Expression::Member {
                        receiver: Box::new(Expression::Name(name)),
                        selector,
                    }),
                    type_arguments,
                    arguments,
                }));
            }
            iris_syntax::PostfixPart::Member { selector, .. } => {
                self.outcome = evaluator.member_read(current, &selector);
                self.work.push(Step::SafeNavigation(parts, position + 1));
            }
            iris_syntax::PostfixPart::Call {
                type_arguments,
                arguments,
            } => {
                let name = format!("\0safe{}", self.temporary);
                self.temporary += 1;
                if let Some(mut locals) = self.scopes.last_mut() {
                    locals.insert(name.clone(), current);
                }
                let mut arguments = arguments;
                let mut next = position + 1;
                if let Some(iris_syntax::PostfixPart::TrailingBlock(block)) = parts.get(next) {
                    arguments.push((**block).clone());
                    next += 1;
                }
                self.work.push(Step::SafeNavigation(parts, next));
                self.work.push(Step::RemoveTemporary(name.clone()));
                self.work.push(Step::Expression(Expression::Call {
                    callee: Box::new(Expression::Name(name)),
                    type_arguments,
                    arguments,
                }));
            }
            iris_syntax::PostfixPart::Index(index) => {
                self.work
                    .push(Step::SafeNavigationIndex(current, parts, position + 1));
                self.work.push(Step::Expression(*index));
            }
            iris_syntax::PostfixPart::TrailingBlock(_) => {
                return Err(EvaluationError::UnsupportedConstruct);
            }
        }
        Ok(())
    }

    pub(super) fn safe_navigation_index(
        &mut self,
        evaluator: &mut SourceEvaluator,
        receiver: Value,
        parts: Vec<iris_syntax::PostfixPart>,
        position: usize,
    ) -> Result<(), EvaluationError> {
        let index = self.value()?;
        self.outcome = evaluator.index_read(receiver, index);
        self.work.push(Step::SafeNavigation(parts, position));
        Ok(())
    }

    fn operand(
        &mut self,
        expression: &mut Expression,
        pending: &mut std::collections::VecDeque<(String, Expression)>,
    ) {
        let name = format!("\0async{}", self.temporary);
        self.temporary += 1;
        let expression = std::mem::replace(expression, Expression::Name(name.clone()));
        pending.push_back((name, expression));
    }

    fn prepare(
        &mut self,
        evaluator: &SourceEvaluator,
        expression: &mut Expression,
        pending: &mut std::collections::VecDeque<(String, Expression)>,
    ) {
        match expression {
            Expression::Array(values) | Expression::Tuple(values) => {
                for value in values {
                    self.operand(value, pending);
                }
            }
            Expression::Hash(entries) => {
                for (key, value) in entries {
                    self.operand(key, pending);
                    self.operand(value, pending);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.operand(left, pending);
                if !matches!(
                    right.as_ref(),
                    Expression::ClosedGeneric { .. }
                        | Expression::ReifiedType(_)
                        | Expression::Name(_)
                ) {
                    self.operand(right, pending);
                }
            }
            Expression::Unary { operand, .. }
            | Expression::NonNull(operand)
            | Expression::BlockArgument { value: operand }
            | Expression::KeywordArgument { value: operand, .. } => self.operand(operand, pending),
            Expression::Index { receiver, index } => {
                self.operand(receiver, pending);
                self.operand(index, pending);
            }
            Expression::SafeNavigation { .. } => {}
            Expression::Member { receiver, .. } => {
                if !matches!(
                    receiver.as_ref(),
                    Expression::Name(_) | Expression::ClosedGeneric { .. }
                ) {
                    self.operand(receiver, pending);
                }
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                match callee.as_mut() {
                    Expression::Member { receiver, .. }
                    | Expression::ContractView { receiver, .. } => {
                        let local_receiver = matches!(receiver.as_ref(), Expression::Name(name) if self.locals().contains_key(name) || evaluator.names.get(name).is_some_and(|binding| !matches!(binding.value(), Value::Class(_) | Value::ClosedClass(..))));
                        if local_receiver
                            || !matches!(
                                receiver.as_ref(),
                                Expression::Name(_) | Expression::ClosedGeneric { .. }
                            )
                        {
                            self.operand(receiver, pending);
                        }
                    }
                    Expression::Name(_) => {}
                    expression => self.operand(expression, pending),
                }
                for argument in arguments {
                    self.operand(argument, pending);
                }
            }
            Expression::Assignment { left, .. } => match left.as_mut() {
                Expression::Index { receiver, index } => {
                    self.operand(receiver, pending);
                    self.operand(index, pending);
                }
                Expression::Member { receiver, .. }
                    if !matches!(receiver.as_ref(), Expression::ClosedGeneric { .. }) =>
                {
                    self.operand(receiver, pending);
                }
                _ => {}
            },
            Expression::ReifiedType(_)
            | Expression::ClosedGeneric { .. }
            | Expression::Name(_)
            | Expression::Literal(_)
            | Expression::Symbol(_)
            | Expression::Closure { .. }
            | Expression::RawIvar(_)
            | Expression::ClassVar(_)
            | Expression::GlobalVar(_)
            | Expression::ContractView { .. }
            | Expression::Yield(_)
            | Expression::Await(_)
            | Expression::If { .. }
            | Expression::While { .. }
            | Expression::Try { .. }
            | Expression::Grouped(_) => {}
        }
    }

    pub(super) fn operands(
        &mut self,
        evaluator: &mut SourceEvaluator,
        mut operands: Operands,
    ) -> Result<(), EvaluationError> {
        if let Some((name, expression)) = operands.pending.pop_front() {
            self.work.push(Step::Operand(operands, name));
            self.work.push(Step::Expression(expression));
            return Ok(());
        }
        let mut values = self.locals();
        values.extend(operands.values);
        operands.values = values;
        let operands = evaluator.rooted(operands);
        if matches!(operands.expression, Expression::Assignment { .. }) {
            return self.assignment(
                evaluator,
                Rc::try_unwrap(operands).unwrap_or_else(|_| unreachable!()),
            );
        }
        if let Expression::Call {
            callee, arguments, ..
        } = &operands.expression
            && matches!(callee.as_ref(), Expression::Name(name) if name == "using" && !evaluator.names.contains_key(name) && evaluator.main_bound_method(name, self.receiver.as_ref()).is_none())
        {
            let values = arguments
                .iter()
                .map(|argument| {
                    evaluator.expression(argument, &operands.values, self.receiver.clone())
                })
                .collect::<Result<Vec<_>, _>>()?;
            crate::source_runtime::call_channels::block(&values)?;
            let [resource, block] = values.as_slice() else {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            };
            let block = match crate::source_runtime::call_channels::ArgumentChannel::of(block) {
                crate::source_runtime::call_channels::ArgumentChannel::Block(value)
                | crate::source_runtime::call_channels::ArgumentChannel::Positional(value) => value,
                crate::source_runtime::call_channels::ArgumentChannel::Keyword(_, _) => {
                    return Err(EvaluationError::ArgumentError);
                }
            };
            let Value::Closure(identity) = block else {
                return Err(EvaluationError::ArgumentError);
            };
            let record = evaluator
                .closures
                .get(identity)
                .cloned()
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            if record.is_async {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            self.work.push(Step::Close(resource.clone()));
            self.work.push(Step::RestoreCall(
                self.receiver.clone(),
                evaluator.wrapper_context(),
                evaluator.names.clone(),
            ));
            evaluator.names.extend(record.cells);
            evaluator.restore_wrapper_context(record.lexical_context);
            self.receiver = record.receiver;
            let mut locals = record.captured;
            if let Some(name) = record.parameters.first() {
                locals.insert(name.clone(), resource.clone());
            }
            self.enter(evaluator, record.body, locals);
            return Ok(());
        }
        self.outcome = evaluator.expression(
            &operands.expression,
            &operands.values,
            self.receiver.clone(),
        );
        if matches!(self.outcome, Err(EvaluationError::AwaitSuspended(_))) {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        Ok(())
    }
}
