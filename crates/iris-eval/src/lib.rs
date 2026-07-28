//! Minimal literal evaluation.

use iris_lexer::{Literal, convert_literals};

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    /// Literal conversion emitted a stable lexical diagnostic.
    LexicalDiagnostic(&'static str),
    /// The source did not produce a literal value this evaluator can observe.
    UnsupportedConstruct,
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
