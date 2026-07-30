//! Minimal literal evaluation.

mod source_method;
mod source_runtime;

use iris_lexer::{Literal, convert_literals};
use iris_parser::parse;
use iris_runtime::{BuiltinClass, Kernel, KernelError, NativeSelector, Value as RuntimeValue};
use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

/// Observable literal values supported by the Iris v1 grammar vectors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// An arbitrary-precision integer represented in its canonical decimal form.
    Integer(String),
    /// Exact IEEE-754 binary32 bits.
    Float32Bits(u32),
    /// Exact IEEE-754 binary64 bits.
    Float64Bits(u64),
    /// A Unicode string.
    String(String),
    /// A literal array.
    Array(Vec<Value>),
}

/// Evaluation failure for a source form outside the literal-only evaluator.
#[derive(Clone, Debug, PartialEq)]
pub enum EvaluationError {
    /// Literal conversion emitted a stable lexical diagnostic.
    LexicalDiagnostic(&'static str),
    /// The source did not produce a literal value this evaluator can observe.
    UnsupportedConstruct,
    /// Source parsing rejected the requested expression.
    ParseDiagnostic,
    /// The runtime rejected a native send.
    Runtime(KernelError),
    /// The declaration publisher rejected a Class mutation.
    Class(iris_runtime::ClassError),
    /// Construction or dynamic instance dispatch failed.
    Construction(iris_runtime::ConstructionError),
    /// A source Method raised during execution.
    Execution(iris_runtime::ExecutionError),
    /// An Iris raise escaped the current source evaluation.
    Raised(RuntimeValue),
    /// Source code attempted to write an immutable lexical binding.
    ImmutableBinding,
    /// Evaluation exceeded its step budget and was abandoned.
    ///
    /// This is a HARNESS limit, not an Iris semantic. It exists so a vector
    /// that fails to terminate is reported as evidence instead of hanging the
    /// conformance suite indefinitely.
    StepBudgetExhausted,
    /// A write was attempted against a read-only property.
    ///
    /// `IRIS-V1-CONTROL-V302A` names this for an `ExceptionContext` value,
    /// which `D-159` makes readable but never assignable.
    ReadonlyProperty,
    /// An unqualified name resolved to no binding.
    ///
    /// `IRIS-V1-CONTROL-C009` requires assignment to an absent ordinary local to
    /// fail as `NameError` and to create NO binding, which is what stops a typo
    /// from silently introducing a local.
    NameError,
    /// Source symbols are not yet representable as runtime Values.
    Symbol(String),
    /// A truthiness `to_bool` Method returned a value other than Bool.
    TypeContractError,
    /// A `<=>` Method returned a value outside `Integer(-1|0|1)` and `nil`.
    ComparisonContractError,
    /// A send supplied the wrong number of arguments for the selected Method.
    ArgumentError,
    /// A binding-only destructuring context did not match its value.
    PatternMatchError,
    /// A bare `raise` occurred outside any catch dynamic extent.
    NoActiveExceptionError,
    /// A cause or suppressed edge would have formed a cycle.
    ExceptionChainError,
    /// A `continue` is unwinding to start the next iteration of its target loop.
    ///
    /// `IRIS-V1-CONTROL-C043` gives `continue` NO value, so unlike `LoopBreak`
    /// it carries only the target label.
    LoopContinue(Option<String>),
    /// A `break` is unwinding to its target loop, carrying the loop result.
    ///
    /// `IRIS-V1-CONTROL-C043` makes `break expr` exit the target loop with
    /// `expr` as the LOOP result, so this travels as a control signal rather
    /// than an ordinary value and is consumed by the loop that catches it.
    LoopBreak(Option<String>, RuntimeValue),
    /// A `return` unwinding to its nearest callable boundary.
    ///
    /// `IRIS-V1-CONTROL-D-421` makes `return` inside a Closure end only THAT
    /// Closure invocation, so this is caught at the nearest Method or Closure
    /// boundary rather than propagating to an enclosing one.
    Return(RuntimeValue),
    /// An ordinary selector was absent and the default `method_missing` applied.
    MessageNotFound {
        receiver_class: String,
        selector: String,
    },
}

/// Evaluates a source expression by sending every supported operator through the runtime kernel.
pub fn evaluate(source: &str) -> Result<RuntimeValue, EvaluationError> {
    let parsed = parse(source);
    if !parsed.program_accepted {
        return Err(EvaluationError::ParseDiagnostic);
    }
    if !parsed.program.declarations.is_empty()
        || parsed
            .program
            .statements
            .iter()
            .any(source_runtime_statement)
    {
        return source_runtime::evaluate(&parsed.program);
    }
    let mut registry = iris_runtime::ClassRegistry::new();
    let kernel = Kernel::new(&mut registry).map_err(EvaluationError::Runtime)?;
    let mut evaluator = Evaluator { kernel, registry };
    let values = parsed
        .program
        .statements
        .iter()
        .map(|statement| evaluator.statement(statement))
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [] => Err(EvaluationError::UnsupportedConstruct),
        [value] => Ok(value.clone()),
        _ => Ok(RuntimeValue::Array(values)),
    }
}

/// Evaluates source and reports whether a named Class was published before failure.
pub fn evaluate_with_class_publication(
    source: &str,
    class_name: &str,
) -> (Result<RuntimeValue, EvaluationError>, bool) {
    let parsed = parse(source);
    if !parsed.program_accepted {
        return (Err(EvaluationError::ParseDiagnostic), false);
    }
    let mut evaluator = match source_runtime::SourceEvaluator::new() {
        Ok(evaluator) => evaluator,
        Err(error) => return (Err(error), false),
    };
    let outcome = evaluator.program(&parsed.program);
    let published = matches!(evaluator.class_name(class_name), Ok(Some(_)));
    (outcome, published)
}

struct Evaluator {
    kernel: Kernel,
    registry: iris_runtime::ClassRegistry,
}

enum Evaluated {
    Value(RuntimeValue),
    Member(RuntimeValue, String),
    UnresolvedClass(String),
    UnresolvedClassMember {
        class_name: String,
        selector: String,
    },
}

impl Evaluator {
    fn statement(&mut self, statement: &Statement) -> Result<RuntimeValue, EvaluationError> {
        match statement {
            Statement::SharedBinding { .. }
            | Statement::Binding { .. }
            | Statement::DeferredBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_) => Err(EvaluationError::UnsupportedConstruct),
            Statement::Expression(expression) => self
                .expression(expression)
                .and_then(|value| self.value(value)),
            Statement::If { .. }
            | Statement::Return(_)
            | Statement::Break { .. }
            | Statement::Continue(_)
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Match { .. } => Err(EvaluationError::UnsupportedConstruct),
            Statement::Raise(_) | Statement::Try { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
        }
    }

    fn expression(&mut self, expression: &Expression) -> Result<Evaluated, EvaluationError> {
        match expression {
            // A keyword argument is meaningless outside a call the literal
            // evaluator cannot make, so it is routed rather than evaluated.
            Expression::KeywordArgument { .. }
            | Expression::Index { .. }
            | Expression::Try { .. } => Err(EvaluationError::UnsupportedConstruct),
            Expression::Name(name) => self.name(name),
            Expression::Literal(source) => self.literal(source).map(Evaluated::Value),
            Expression::Array(expressions) => expressions
                .iter()
                .map(|expression| {
                    self.expression(expression)
                        .and_then(|value| self.value(value))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(RuntimeValue::Array)
                .map(Evaluated::Value),
            Expression::Grouped(expression) => self.expression(expression),
            Expression::Member { receiver, selector } => match self.expression(receiver)? {
                Evaluated::Value(receiver) => Ok(Evaluated::Member(receiver, selector.clone())),
                Evaluated::Member(receiver, previous_selector) => {
                    let receiver = self.value(Evaluated::Member(receiver, previous_selector))?;
                    Ok(Evaluated::Member(receiver, selector.clone()))
                }
                Evaluated::UnresolvedClass(class_name) => Ok(Evaluated::UnresolvedClassMember {
                    class_name,
                    selector: selector.clone(),
                }),
                Evaluated::UnresolvedClassMember { .. } => {
                    Err(EvaluationError::UnsupportedConstruct)
                }
            },
            Expression::Call { callee, arguments } => match self.expression(callee)? {
                Evaluated::Member(receiver, selector) => {
                    let arguments = self.arguments(arguments, None)?;
                    self.send(receiver, &selector, &arguments)
                }
                Evaluated::Value(RuntimeValue::Class(class)) => {
                    let arguments = self.arguments(arguments, Some(class))?;
                    self.kernel
                        .construct(class, &arguments)
                        .map(Evaluated::Value)
                        .map_err(EvaluationError::Runtime)
                }
                Evaluated::UnresolvedClassMember {
                    class_name,
                    selector,
                } => Err(EvaluationError::MessageNotFound {
                    receiver_class: class_name,
                    selector,
                }),
                Evaluated::Value(_) | Evaluated::UnresolvedClass(_) => {
                    Err(EvaluationError::UnsupportedConstruct)
                }
            },
            Expression::Unary { operator, operand } => {
                let operand = self.expression(operand)?;
                let operand = self.value(operand)?;
                let selector = match operator {
                    UnaryOperator::Negate => NativeSelector::Negate,
                    UnaryOperator::BitwiseNot => NativeSelector::BitwiseNot,
                    UnaryOperator::Not => {
                        return self
                            .truthy(operand)
                            .map(|value| Evaluated::Value(RuntimeValue::Bool(!value)));
                    }
                    UnaryOperator::Plus => {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                };
                self.kernel
                    .send(&self.registry, operand, selector, &[])
                    .map(Evaluated::Value)
                    .map_err(EvaluationError::Runtime)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.expression(left)?;
                let left = self.value(left)?;
                if matches!(operator, BinaryOperator::LogicalAnd) {
                    return if self.truthy(left.clone())? {
                        self.expression(right)
                    } else {
                        Ok(Evaluated::Value(left))
                    };
                }
                if matches!(operator, BinaryOperator::LogicalOr) {
                    return if self.truthy(left.clone())? {
                        Ok(Evaluated::Value(left))
                    } else {
                        self.expression(right)
                    };
                }
                let right = self.expression(right)?;
                let right = self.value(right)?;
                match operator {
                    BinaryOperator::Identity => Kernel::same_identity(&left, &right)
                        .map(RuntimeValue::Bool)
                        .map(Evaluated::Value)
                        .map_err(EvaluationError::Runtime),
                    _ => self.binary(left, operator, right),
                }
            }
            Expression::Symbol(_)
            | Expression::RawIvar(_)
            | Expression::ClassVar(_)
            | Expression::ContractView { .. }
            | Expression::Closure { .. }
            | Expression::Hash(_)
            | Expression::Assignment { .. }
            | Expression::If { .. } => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn name(&self, name: &str) -> Result<Evaluated, EvaluationError> {
        let value = match name {
            "nil" => RuntimeValue::Nil,
            "true" => RuntimeValue::Bool(true),
            "false" => RuntimeValue::Bool(false),
            "Integer" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Integer)
                    .map_err(EvaluationError::Runtime)?,
            ),
            "Float32" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Float32)
                    .map_err(EvaluationError::Runtime)?,
            ),
            "Float64" => RuntimeValue::Class(
                self.kernel
                    .class(BuiltinClass::Float64)
                    .map_err(EvaluationError::Runtime)?,
            ),
            _ if name.chars().next().is_some_and(char::is_uppercase) => {
                return Ok(Evaluated::UnresolvedClass(name.into()));
            }
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        Ok(Evaluated::Value(value))
    }

    fn arguments(
        &mut self,
        arguments: &[Expression],
        constructor: Option<iris_runtime::ClassId>,
    ) -> Result<Vec<RuntimeValue>, EvaluationError> {
        let float64 = self
            .kernel
            .class(BuiltinClass::Float64)
            .map_err(EvaluationError::Runtime)?;
        arguments
            .iter()
            .map(|argument| {
                if constructor == Some(float64)
                    && matches!(argument, Expression::Unary { operator: UnaryOperator::Negate, operand } if matches!(operand.as_ref(), Expression::Name(name) if name == "Infinity"))
                {
                    Ok(RuntimeValue::Float64(f64::NEG_INFINITY))
                } else {
                    self.expression(argument).and_then(|value| self.value(value))
                }
            })
            .collect()
    }

    fn literal(&self, source: &str) -> Result<RuntimeValue, EvaluationError> {
        match source {
            "nil" => return Ok(RuntimeValue::Nil),
            "true" => return Ok(RuntimeValue::Bool(true)),
            "false" => return Ok(RuntimeValue::Bool(false)),
            _ => {}
        }
        match evaluate_literals(source)? {
            Value::Integer(value) => value
                .parse()
                .map(RuntimeValue::Integer)
                .map_err(|_| EvaluationError::UnsupportedConstruct),
            Value::Float32Bits(bits) => Ok(RuntimeValue::Float32(f32::from_bits(bits))),
            Value::Float64Bits(bits) => Ok(RuntimeValue::Float64(f64::from_bits(bits))),
            Value::String(_) | Value::Array(_) => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn send(
        &self,
        receiver: RuntimeValue,
        selector: &str,
        arguments: &[RuntimeValue],
    ) -> Result<Evaluated, EvaluationError> {
        let selector = NativeSelector::from_source(selector).ok_or_else(|| {
            EvaluationError::MessageNotFound {
                receiver_class: receiver_class_name(&receiver).into(),
                selector: selector.into(),
            }
        })?;
        self.kernel
            .send(&self.registry, receiver, selector, arguments)
            .map(Evaluated::Value)
            .map_err(EvaluationError::Runtime)
    }

    fn binary(
        &self,
        left: RuntimeValue,
        operator: &BinaryOperator,
        right: RuntimeValue,
    ) -> Result<Evaluated, EvaluationError> {
        let selector = match operator {
            BinaryOperator::Add => NativeSelector::Add,
            BinaryOperator::Subtract => NativeSelector::Subtract,
            BinaryOperator::Multiply => NativeSelector::Multiply,
            BinaryOperator::Divide => NativeSelector::Divide,
            BinaryOperator::Power => NativeSelector::Power,
            BinaryOperator::ShiftLeft => NativeSelector::ShiftLeft,
            BinaryOperator::ShiftRight => NativeSelector::ShiftRight,
            BinaryOperator::Equal => NativeSelector::Equal,
            BinaryOperator::NotEqual => NativeSelector::NotEqual,
            BinaryOperator::Less => NativeSelector::Less,
            BinaryOperator::LessEqual => NativeSelector::LessEqual,
            BinaryOperator::Greater => NativeSelector::Greater,
            BinaryOperator::GreaterEqual => NativeSelector::GreaterEqual,
            BinaryOperator::Compare => NativeSelector::Compare,
            BinaryOperator::NamedInfix { selector } => NativeSelector::from_source(selector)
                .ok_or(EvaluationError::UnsupportedConstruct)?,
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        self.kernel
            .send(&self.registry, left, selector, &[right])
            .map(Evaluated::Value)
            .map_err(EvaluationError::Runtime)
    }

    fn truthy(&self, value: RuntimeValue) -> Result<bool, EvaluationError> {
        match self
            .kernel
            .send(&self.registry, value, NativeSelector::ToBool, &[])
            .map_err(EvaluationError::Runtime)?
        {
            RuntimeValue::Bool(value) => Ok(value),
            RuntimeValue::Nil
            | RuntimeValue::Integer(_)
            | RuntimeValue::Float32(_)
            | RuntimeValue::Float64(_)
            | RuntimeValue::Array(_)
            | RuntimeValue::Hash(_)
            | RuntimeValue::Symbol(_)
            | RuntimeValue::Class(_)
            | RuntimeValue::Type(_)
            | RuntimeValue::Contract(_)
            | RuntimeValue::Closure(_)
            | RuntimeValue::KeywordArgument(_, _)
            | RuntimeValue::IterationYield(_)
            | RuntimeValue::ArrayIterator(_)
            | RuntimeValue::IterationDone
            | RuntimeValue::ExceptionContext(..)
            | RuntimeValue::ContractView(_, _)
            | RuntimeValue::Object(_)
            | RuntimeValue::BoundMethod(_)
            | RuntimeValue::Method(_) => Err(EvaluationError::TypeContractError),
        }
    }

    fn value(&self, value: Evaluated) -> Result<RuntimeValue, EvaluationError> {
        match value {
            Evaluated::Value(value) => Ok(value),
            Evaluated::Member(receiver, selector) => self
                .send(receiver, &selector, &[])
                .and_then(|value| self.value(value)),
            Evaluated::UnresolvedClass(_) | Evaluated::UnresolvedClassMember { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
        }
    }
}

fn receiver_class_name(value: &RuntimeValue) -> &'static str {
    match value {
        RuntimeValue::Nil => "Nil",
        RuntimeValue::Bool(_) => "Bool",
        RuntimeValue::Integer(_) => "Integer",
        RuntimeValue::Float32(_) => "Float32",
        RuntimeValue::Float64(_) => "Float64",
        RuntimeValue::Array(_) => "Array",
        RuntimeValue::Hash(_) => "Hash",
        RuntimeValue::Symbol(_) => "Symbol",
        RuntimeValue::Class(_) => "Class",
        RuntimeValue::Type(_) => "Type",
        RuntimeValue::Contract(_) => "Contract",
        RuntimeValue::Closure(_) => "Closure",
        RuntimeValue::KeywordArgument(_, _) | RuntimeValue::IterationYield(_) => "Iteration",
        RuntimeValue::ArrayIterator(..) | RuntimeValue::IterationDone => "Iteration",
        RuntimeValue::ExceptionContext(..) => "ExceptionContext",
        RuntimeValue::ContractView(_, _) => "ContractView",
        RuntimeValue::Object(_) => "Object",
        RuntimeValue::BoundMethod(_) => "BoundMethod",
        RuntimeValue::Method(_) => "Method",
    }
}

fn source_runtime_statement(statement: &Statement) -> bool {
    match statement {
        Statement::SharedBinding { .. }
        | Statement::Binding { .. }
        | Statement::DeferredBinding { .. } => true,
        Statement::Expression(expression) => source_runtime_expression(expression),
        Statement::If { .. } => true,
        // A loop needs the source runtime: the literal evaluator has no heap and
        // no statement sequencing. Its BODY is checked too, since a `break`
        // there is what carries the loop result.
        Statement::While { .. }
        | Statement::For { .. }
        | Statement::Break { .. }
        | Statement::Continue(_)
        | Statement::Match { .. } => true,
        Statement::StoredProperty { .. } | Statement::Method(_) | Statement::Return(_) => false,
        Statement::Raise(_) | Statement::Try { .. } => true,
    }
}

/// Reports whether a callee is `Object.new`, which allocates an ordinary instance.
///
/// The literal evaluator has no heap, so construction of the
/// `IRIS-V1-RUNTIME-C005` root Class must route to the source runtime even when
/// the program is a single expression and would otherwise stay on the literal path.
fn constructs_root_object(callee: &Expression) -> bool {
    matches!(
        callee,
        Expression::Member { receiver, selector }
            if selector == "new"
                && matches!(receiver.as_ref(), Expression::Name(name) if name == "Object")
    )
}

/// Reports whether an expression names the `Iteration` results of C013.
///
/// They are value constructors rather than Class sends, and the literal
/// evaluator cannot build them, so they route to the source runtime.
fn builds_iteration(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Member { receiver, .. }
            if matches!(receiver.as_ref(), Expression::Name(name) if name == "Iteration")
    )
}

fn source_runtime_expression(expression: &Expression) -> bool {
    if builds_iteration(expression) {
        return true;
    }
    match expression {
        // A Closure needs the heap the literal evaluator does not have.
        // A keyword argument binds by name, which only the source runtime does.
        Expression::Closure { .. }
        | Expression::Hash(_)
        | Expression::If { .. }
        | Expression::KeywordArgument { .. }
        | Expression::Index { .. }
        | Expression::Try { .. } => true,
        Expression::Array(values) => values.iter().any(source_runtime_expression),
        Expression::Member { receiver, .. }
        | Expression::ContractView { receiver, .. }
        | Expression::Grouped(receiver)
        | Expression::Unary {
            operand: receiver, ..
        } => source_runtime_expression(receiver),
        Expression::Call { callee, arguments } => {
            builds_iteration(callee)
                || constructs_root_object(callee)
                || source_runtime_expression(callee)
                || arguments.iter().any(source_runtime_expression)
        }
        // An assignment needs the binding environment to decide whether the
        // target exists at all, which C009 makes a NameError when it does not.
        Expression::Assignment { .. } => true,
        Expression::Binary { left, right, .. } => {
            source_runtime_expression(left) || source_runtime_expression(right)
        }
        Expression::Name(_)
        | Expression::Literal(_)
        | Expression::Symbol(_)
        | Expression::RawIvar(_)
        | Expression::ClassVar(_) => false,
    }
}

/// Evaluates source containing only lexer-converted literal values.
///
/// # Errors
/// Returns the lexer diagnostic when conversion rejects a literal, or
/// [`EvaluationError::UnsupportedConstruct`] when no literal value is present.
pub fn evaluate_literals(source: &str) -> Result<Value, EvaluationError> {
    let conversion = convert_literals(source);
    if let Some(&diagnostic) = conversion.diagnostics().first() {
        return Err(EvaluationError::LexicalDiagnostic(diagnostic));
    }

    let values = conversion
        .values()
        .iter()
        .cloned()
        .map(Value::from)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [] => Err(EvaluationError::UnsupportedConstruct),
        [value] => Ok(value.clone()),
        _ => Ok(Value::Array(values)),
    }
}

#[cfg(test)]
mod evaluator_bridge_tests {
    use iris_runtime::{MethodBody, NativeSelector, Value as RuntimeValue, Visibility};

    use super::evaluate;

    #[test]
    fn evaluates_v039_floor_division_from_source() {
        // Given
        let source = "[-5 div 2, 5 div -2, -5 div -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer((-3_i8).into()),
                RuntimeValue::Integer((-3_i8).into()),
                RuntimeValue::Integer(2_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v040_modulo_from_source() {
        // Given
        let source = "[-5 mod 2, 5 mod -2, -5 mod -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(1_u8.into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-1_i8).into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v019_singleton_identity_from_source() {
        // Given
        let source = "[nil same? nil, true same? true, false same? false]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![RuntimeValue::Bool(true); 3]))
        );
    }

    #[test]
    fn evaluates_v046_integer_division_from_source() {
        // Given
        let source = "Integer(5) / Integer(2)";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Float64(2.5)));
    }

    #[test]
    fn evaluates_v036_signed_zero_equality_while_preserving_bits() {
        // Given
        let source =
            "Float64.from_bits(0x0000000000000000) == Float64.from_bits(0x8000000000000000)";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn evaluates_v036_signed_zero_hash_equality_through_ordinary_member_sends() {
        // Given
        let source = "Float64.from_bits(0x0000000000000000).hash == Float64.from_bits(0x8000000000000000).hash";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn equal_numbers_have_equal_public_hashes_through_ordinary_member_sends() {
        // Given
        let source = "Float64(1).hash == Float64.from_bits(0x3ff0000000000000).hash";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Bool(true)));
    }

    #[test]
    fn named_infix_and_member_send_have_the_same_result() {
        // Given
        let infix = "5 div 2";
        let member = "Integer(5).div(2)";

        // When
        let infix_result = evaluate(infix);
        let member_result = evaluate(member);

        // Then
        assert_eq!(infix_result, member_result);
        assert_eq!(infix_result, Ok(RuntimeValue::Integer(2_u8.into())));
    }

    #[test]
    fn executes_declared_method_through_member_and_named_infix_sends() {
        // Given
        let source = "class A { public fun scale(value: Integer) -> Integer { value * 2 } }; let a = A.new(); a.scale(3); a scale 3";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(6_u8.into()),
                RuntimeValue::Integer(6_u8.into()),
            ]))
        );
    }

    #[test]
    fn integer_division_by_zero_is_a_typed_runtime_error() {
        // Given
        let source = "1 div 0";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(result, Err(super::EvaluationError::Runtime(_))));
    }

    #[test]
    fn identity_primitive_bypasses_replaced_comparison_slots()
    -> Result<(), iris_runtime::KernelError> {
        // Given
        let mut registry = iris_runtime::ClassRegistry::new();
        let kernel = iris_runtime::Kernel::new(&mut registry)?;
        let bool_class = kernel.class(iris_runtime::BuiltinClass::Bool)?;
        registry.publish_method(
            bool_class,
            NativeSelector::Equal.id(),
            MethodBody::new(1),
            Visibility::Public,
        )?;

        // When
        let identity = iris_runtime::Kernel::same_identity(
            &RuntimeValue::Bool(true),
            &RuntimeValue::Bool(true),
        )?;

        // Then
        assert!(identity);
        Ok(())
    }

    #[test]
    fn evaluates_addition_at_the_float32_receiver_width() {
        // Given
        let source = "16777217 + 0.0f32";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(result, Ok(RuntimeValue::Float32(16_777_216.0)));
    }

    #[test]
    fn evaluates_v042_negative_float_power_as_nan() {
        // Given
        let source = "(-2.0f64) ** 0.5f64";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(result, Ok(RuntimeValue::Float64(value)) if value.is_nan()));
    }

    #[test]
    fn evaluates_integer_bitwise_not_and_right_shift_from_source() {
        // Given
        let source = "[~0, -3 >> 1, 8 << -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-2_i8).into()),
                RuntimeValue::Integer(2_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v047_infinity_member_chain_as_width_preserving_nan() {
        // Given
        let source = "Float64.infinity.mul_add(0, 1); Float32.infinity.mul_add(0, 1)";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float64(value), RuntimeValue::Float32(other)] if value.is_nan() && other.is_nan())
        ));
    }

    #[test]
    fn evaluates_v059_nan_equality_and_less_than() {
        // Given
        let equality = "Float64.nan == Float64.nan";
        let less_than = "Float64.nan < 1.0";
        let comparison = "Float64.nan <=> 1.0";

        // When
        let equality_result = evaluate(equality);
        let less_than_result = evaluate(less_than);
        let comparison_result = evaluate(comparison);

        // Then
        assert_eq!(equality_result, Ok(RuntimeValue::Bool(false)));
        assert_eq!(less_than_result, Ok(RuntimeValue::Bool(false)));
        assert_eq!(comparison_result, Ok(RuntimeValue::Nil));
    }

    #[test]
    fn evaluates_v063_large_and_negative_shift_counts() {
        // Given
        let source = "[1 >> 1000000, -1 >> 1000000, -3 >> 1000000, 1 << -2]";

        // When
        let result = evaluate(source);

        // Then
        assert_eq!(
            result,
            Ok(RuntimeValue::Array(vec![
                RuntimeValue::Integer(0_u8.into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer((-1_i8).into()),
                RuntimeValue::Integer(0_u8.into()),
            ]))
        );
    }

    #[test]
    fn evaluates_v065_class_getters_at_their_declared_widths() {
        // Given
        let source = "Float32.nan; Float64.infinity; -Float64.infinity";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float32(nan), RuntimeValue::Float64(infinity), RuntimeValue::Float64(negative_infinity)] if nan.is_nan() && infinity.is_infinite() && infinity.is_sign_positive() && negative_infinity.is_infinite() && negative_infinity.is_sign_negative())
        ));
    }

    #[test]
    fn rejects_v065_bare_special_value_names() {
        // Given
        let names = ["nan", "inf"];

        // When
        let results = names.map(evaluate);

        // Then
        assert!(
            results
                .into_iter()
                .all(|result| matches!(result, Err(super::EvaluationError::UnsupportedConstruct)))
        );
    }

    #[test]
    fn evaluates_v104_special_value_arithmetic_as_width_preserving_nan() {
        // Given
        let source = "Float32.nan + 1.0f32; Float64.infinity - Float64.infinity";

        // When
        let result = evaluate(source);

        // Then
        assert!(matches!(
            result,
            Ok(RuntimeValue::Array(values))
                if matches!(values.as_slice(), [RuntimeValue::Float32(value), RuntimeValue::Float64(other)] if value.is_nan() && other.is_nan())
        ));
    }

    #[test]
    fn evaluates_v069_out_of_range_float_bit_patterns_as_typed_errors() {
        // Given
        let sources = [
            "Float64.from_bits(-1)",
            "Float32.from_bits(2 ** 32)",
            "Float64.from_bits(2 ** 64)",
        ];

        // When
        let results = sources.map(evaluate);

        // Then
        assert!(results.into_iter().all(|result| matches!(
            result,
            Err(super::EvaluationError::Runtime(
                iris_runtime::KernelError::Numeric(iris_runtime::NumericError::Range)
            ))
        )));
    }
}

#[cfg(test)]
mod runner_gap_tests;

#[cfg(test)]
mod operator_super_tests;

#[cfg(test)]
mod module_composition_tests;

#[cfg(test)]
mod builtin_protocol_tests;

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Integer(value) => Self::Integer(value),
            Literal::Float32(value) => Self::Float32Bits(value.to_bits()),
            Literal::Float64(value) => Self::Float64Bits(value.to_bits()),
            Literal::String(value) => Self::String(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EvaluationError, Value, evaluate_literals};

    #[test]
    fn evaluates_v152_hexadecimal_float_to_exact_float64_bits() -> Result<(), EvaluationError> {
        // Given
        let source = "0x1.fp3";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Float64Bits(0x402f_0000_0000_0000));
        Ok(())
    }

    #[test]
    fn evaluates_v151_float32_to_exact_bits() -> Result<(), EvaluationError> {
        // Given
        let source = "1e+3f32";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Float32Bits(0x447a_0000));
        Ok(())
    }

    #[test]
    fn preserves_integers_beyond_u64_through_evaluation() -> Result<(), EvaluationError> {
        // Given
        let source = "18446744073709551616";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::Integer("18446744073709551616".into()));
        Ok(())
    }

    #[test]
    fn evaluates_v152_array_with_exact_value_types() -> Result<(), EvaluationError> {
        // Given
        let source = "[0x1.fp3, 0x1p0, 0x1.p0, 0x.8p0, 0x1e3]";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(
            value,
            Value::Array(vec![
                Value::Float64Bits(0x402f_0000_0000_0000),
                Value::Float64Bits(0x3ff0_0000_0000_0000),
                Value::Float64Bits(0x3ff0_0000_0000_0000),
                Value::Float64Bits(0x3fe0_0000_0000_0000),
                Value::Integer("483".into()),
            ])
        );
        Ok(())
    }

    #[test]
    fn evaluates_v179_adjacent_strings_as_one_string() -> Result<(), EvaluationError> {
        // Given
        let source = "\"a\" 'b' r\"c\" \"\"\"d\"\"\"";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(value, Value::String("abcd".into()));
        Ok(())
    }

    #[test]
    fn evaluates_v173_interpolated_and_non_interpolated_string_forms() -> Result<(), EvaluationError>
    {
        // Given
        let source = "[\"a\\n${1 + 1}\", 'a\\n${x}', r#\"a\\n${x}\"#, \"\"\"\n  a\n  \"\"\"]";

        // When
        let value = evaluate_literals(source)?;

        // Then
        assert_eq!(
            value,
            Value::Array(vec![
                Value::String("a\n2".into()),
                Value::String("a\n${x}".into()),
                Value::String(r"a\n${x}".into()),
                Value::String("a".into()),
            ])
        );
        Ok(())
    }

    #[test]
    fn returns_the_lexer_diagnostic_for_malformed_literals() {
        // Given
        let source = "1__0";

        // When
        let result = evaluate_literals(source);

        // Then
        assert_eq!(
            result,
            Err(EvaluationError::LexicalDiagnostic(
                "LEX_BAD_NUMERIC_SEPARATOR"
            ))
        );
    }

    #[test]
    fn rejects_a_source_without_a_literal_value() {
        // Given
        let source = "identifier";

        // When
        let result = evaluate_literals(source);

        // Then
        assert_eq!(result, Err(EvaluationError::UnsupportedConstruct));
    }
}
