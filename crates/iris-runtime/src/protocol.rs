use core::fmt;

use crate::Value;

/// Ordinary comparison slots derived from the current visible `<=>` by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonSlot {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Typed failure from an invalid dynamic `<=>` response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonError {
    Contract,
}

impl fmt::Display for ComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Iris comparison protocol returned an invalid ordering value")
    }
}

impl std::error::Error for ComparisonError {}

/// Minimal callable comparison surface until evaluator-owned Method bodies exist.
#[derive(Clone, Debug)]
pub struct ComparisonProtocol {
    spaceship: Value,
    slots: [Option<Value>; 6],
}

impl Default for ComparisonProtocol {
    fn default() -> Self {
        Self {
            spaceship: Value::Nil,
            slots: [None, None, None, None, None, None],
        }
    }
}

impl ComparisonProtocol {
    /// Returns the root default `<=>` result, always nil.
    pub fn compare(&self, _other: &Value) -> Result<Value, ComparisonError> {
        Ok(self.spaceship.clone())
    }

    /// Replaces the dynamic `<=>` response observed by still-default slots.
    pub fn replace_spaceship(&mut self, result: Value) {
        self.spaceship = result;
    }

    /// Replaces one comparison slot independently from `<=>`.
    pub fn replace_slot(&mut self, slot: ComparisonSlot, result: Value) {
        self.slots[Self::index(slot)] = Some(result);
    }

    /// Invokes one comparison slot, using identity equality before default delegation.
    pub fn call(
        &self,
        slot: ComparisonSlot,
        same_identity: bool,
    ) -> Result<Value, ComparisonError> {
        if let Some(result) = &self.slots[Self::index(slot)] {
            return Ok(result.clone());
        }
        if same_identity {
            return Ok(Value::Bool(matches!(
                slot,
                ComparisonSlot::Equal | ComparisonSlot::LessEqual | ComparisonSlot::GreaterEqual
            )));
        }
        let ordering = match &self.spaceship {
            Value::Nil => None,
            Value::Integer(value) if value == &(-1_i8).into() => Some(-1_i8),
            Value::Integer(value) if value == &0_u8.into() => Some(0_i8),
            Value::Integer(value) if value == &1_u8.into() => Some(1_i8),
            Value::Integer(_)
            | Value::Bool(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Array(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::MutableString(_)
            | Value::Regex(_)
            | Value::Match(_)
            | Value::Library(_)
            | Value::NativeResource(_)
            | Value::Gate(_)
            | Value::Tuple(_)
            | Value::Hash(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Class(_)
            | Value::Type(..)
            | Value::ComposedType(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::ReadonlyArray(_)
            | Value::SourceLocation(..)
            | Value::StackFrame(..)
            | Value::RaiseSite(_)
            | Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::ByteIterator(_)
            | Value::Generator(_)
            | Value::Task(_)
            | Value::Range(..)
            | Value::IterationDone
            | Value::Transformation { .. }
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::Object(_)
            | Value::BoundMethod(_)
            | Value::Method(_) => {
                return Err(ComparisonError::Contract);
            }
        };
        Ok(Value::Bool(match (slot, ordering) {
            (ComparisonSlot::Equal, Some(0)) => true,
            (ComparisonSlot::NotEqual, Some(0)) => false,
            (ComparisonSlot::Less, Some(-1)) => true,
            (ComparisonSlot::LessEqual, Some(-1 | 0)) => true,
            (ComparisonSlot::Greater, Some(1)) => true,
            (ComparisonSlot::GreaterEqual, Some(0 | 1)) => true,
            (ComparisonSlot::NotEqual, None) => true,
            _ => false,
        }))
    }

    const fn index(slot: ComparisonSlot) -> usize {
        match slot {
            ComparisonSlot::Equal => 0,
            ComparisonSlot::NotEqual => 1,
            ComparisonSlot::Less => 2,
            ComparisonSlot::LessEqual => 3,
            ComparisonSlot::Greater => 4,
            ComparisonSlot::GreaterEqual => 5,
        }
    }
}

/// Evaluator outcome from one `to_bool` send.
#[derive(Clone, Debug, PartialEq)]
pub enum TruthinessMethod {
    Default,
    Returns(Value),
    Raises(Value),
}

/// Typed failure from truthiness evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum TruthinessError {
    TypeContract,
    Raised(Value),
}

/// Dynamic truthiness helpers that invoke the selected protocol exactly once.
pub struct Truthiness;

impl Truthiness {
    /// Tests one operand through its selected `to_bool` result.
    pub fn test(value: &Value, method: TruthinessMethod) -> Result<bool, TruthinessError> {
        let result = match method {
            TruthinessMethod::Default => match value {
                Value::Nil => Value::Bool(false),
                Value::Bool(value) => Value::Bool(*value),
                Value::Integer(_)
                | Value::Float32(_)
                | Value::Float64(_)
                | Value::Array(_)
                | Value::Bytes(_)
                | Value::ByteArray(_)
                | Value::MutableString(_)
                | Value::Regex(_)
                | Value::Match(_)
                | Value::Library(_)
                | Value::NativeResource(_)
                | Value::Gate(_)
                | Value::Tuple(_)
                | Value::Hash(_)
                | Value::Text(_)
                | Value::Symbol(_)
                | Value::Class(_)
                | Value::Type(..)
                | Value::ComposedType(_)
                | Value::Contract(_)
                | Value::Closure(_)
                | Value::KeywordArgument(_, _)
                | Value::IterationYield(_)
                | Value::ReadonlyArray(_)
                | Value::SourceLocation(..)
                | Value::StackFrame(..)
                | Value::RaiseSite(_)
                | Value::ArrayIterator(_)
                | Value::HashIterator(_)
                | Value::ByteIterator(_)
                | Value::Generator(_)
                | Value::Task(_)
                | Value::Range(..)
                | Value::IterationDone
                | Value::Transformation { .. }
                | Value::ExceptionContext(..)
                | Value::ContractView(_, _)
                | Value::Object(_)
                | Value::BoundMethod(_)
                | Value::Method(_) => Value::Bool(true),
            },
            TruthinessMethod::Returns(value) => value,
            TruthinessMethod::Raises(error) => return Err(TruthinessError::Raised(error)),
        };
        match result {
            Value::Bool(value) => Ok(value),
            Value::Nil
            | Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Array(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::MutableString(_)
            | Value::Regex(_)
            | Value::Match(_)
            | Value::Library(_)
            | Value::NativeResource(_)
            | Value::Gate(_)
            | Value::Tuple(_)
            | Value::Hash(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Class(_)
            | Value::Type(..)
            | Value::ComposedType(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::ReadonlyArray(_)
            | Value::SourceLocation(..)
            | Value::StackFrame(..)
            | Value::RaiseSite(_)
            | Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::ByteIterator(_)
            | Value::Generator(_)
            | Value::Task(_)
            | Value::Range(..)
            | Value::IterationDone
            | Value::Transformation { .. }
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::Object(_)
            | Value::BoundMethod(_)
            | Value::Method(_) => Err(TruthinessError::TypeContract),
        }
    }

    /// Implements `a && b`, preserving the original falsy left operand.
    pub fn logical_and(
        left: Value,
        method: TruthinessMethod,
        right: impl FnOnce() -> Result<Value, TruthinessError>,
    ) -> Result<Value, TruthinessError> {
        if Self::test(&left, method)? {
            right()
        } else {
            Ok(left)
        }
    }
}
