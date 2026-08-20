//! Lowers a parsed program to instructions.

use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

/// One stack-machine instruction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Pushes an arbitrary-precision Integer, held as canonical decimal text.
    PushInteger(String),
    /// Pushes an IEEE-754 binary64 value, held as BITS so a literal cannot
    /// drift through a decimal round trip.
    PushFloat64(u64),
    /// Pushes an IEEE-754 binary32 value, held as bits for the same reason.
    PushFloat32(u32),
    /// Pushes a String.
    PushText(String),
    /// Sends a native binary selector to the two topmost values.
    Binary(&'static str),
    /// Sends a native unary selector to the topmost value.
    Unary(&'static str),
}

/// A compiled program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub(crate) instructions: Vec<Instruction>,
}

/// Why a program could not be compiled.
///
/// This is NOT a program error. It means this backend does not yet cover the
/// construct, so the caller must decline rather than produce a result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    /// The construct that is not covered.
    pub construct: String,
}

impl CompileError {
    fn new(construct: impl Into<String>) -> Self {
        Self {
            construct: construct.into(),
        }
    }
}

/// Compiles `source`, or reports the first construct this backend lacks.
///
/// # Errors
/// Returns the uncovered construct, or a parse rejection.
pub fn compile(source: &str) -> Result<Program, CompileError> {
    let parsed = iris_parser::parse(source);
    if !parsed.program_accepted {
        return Err(CompileError::new("rejected source"));
    }
    if !parsed.program.declarations.is_empty() {
        return Err(CompileError::new("declaration"));
    }
    let [statement] = parsed.program.statements.as_slice() else {
        // Several statements need a value stack discipline this subset does
        // not define yet, so it declines rather than guessing which value the
        // program answers.
        return Err(CompileError::new("multiple statements"));
    };
    let Statement::Expression(expression) = statement else {
        return Err(CompileError::new("non-expression statement"));
    };
    let mut instructions = Vec::new();
    lower(expression, &mut instructions)?;
    Ok(Program { instructions })
}

fn lower(expression: &Expression, instructions: &mut Vec<Instruction>) -> Result<(), CompileError> {
    match expression {
        Expression::Literal(text) => lower_literal(text, instructions),
        Expression::Grouped(inner) => lower(inner, instructions),
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let selector = binary_selector(operator)?;
            lower(left, instructions)?;
            lower(right, instructions)?;
            instructions.push(Instruction::Binary(selector));
            Ok(())
        }
        Expression::Unary { operator, operand } => {
            let selector = match operator {
                UnaryOperator::Negate => "-@",
                UnaryOperator::Not => return Err(CompileError::new("unary not")),
                other => return Err(CompileError::new(format!("unary {other:?}"))),
            };
            lower(operand, instructions)?;
            instructions.push(Instruction::Unary(selector));
            Ok(())
        }
        other => Err(CompileError::new(construct_name(other))),
    }
}

/// Lowers a literal by RE-LEXING its text.
///
/// The parser keeps a literal as source text, and the lexer is what turns it
/// into a value under the chapter 02 rules - suffixes, radix prefixes,
/// separators and precision warnings. Parsing the text here instead would be a
/// second literal implementation, and a differential row would then be
/// comparing this backend's literal rules against the lexer's.
fn lower_literal(text: &str, instructions: &mut Vec<Instruction>) -> Result<(), CompileError> {
    let conversion = iris_lexer::convert_literals(text);
    if !conversion.diagnostics().is_empty() {
        return Err(CompileError::new("rejected literal"));
    }
    let [literal] = conversion.values() else {
        return Err(CompileError::new("literal"));
    };
    instructions.push(match literal {
        iris_lexer::Literal::Integer(digits) => Instruction::PushInteger(digits.clone()),
        iris_lexer::Literal::Float64(number) => Instruction::PushFloat64(number.to_bits()),
        iris_lexer::Literal::Float32(number) => Instruction::PushFloat32(number.to_bits()),
        iris_lexer::Literal::String(text) => Instruction::PushText(text.clone()),
    });
    Ok(())
}

/// The native selector for a binary operator, when this backend covers it.
///
/// Only operators whose whole meaning is a native send appear here. A
/// comparison, range, regex or type operator carries semantics beyond the
/// kernel send, so it is declined rather than approximated.
fn binary_selector(operator: &BinaryOperator) -> Result<&'static str, CompileError> {
    Ok(match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Power => "**",
        BinaryOperator::ShiftLeft => "<<",
        BinaryOperator::ShiftRight => ">>",
        BinaryOperator::BitwiseAnd => "&",
        BinaryOperator::BitwiseXor => "^",
        BinaryOperator::BitwiseOr => "|",
        other => return Err(CompileError::new(format!("operator {other:?}"))),
    })
}

fn construct_name(expression: &Expression) -> String {
    let name = match expression {
        Expression::Name(_) => "name",
        Expression::Symbol(_) => "symbol",
        Expression::Array(_) => "array",
        Expression::Hash(_) => "hash",
        Expression::Tuple(_) => "tuple",
        Expression::Call { .. } => "call",
        Expression::Member { .. } => "member",
        Expression::Index { .. } => "index",
        Expression::Closure { .. } => "closure",
        Expression::If { .. } => "if",
        Expression::While { .. } => "while",
        Expression::Try { .. } => "try",
        Expression::Await(_) => "await",
        Expression::Yield(_) => "yield",
        Expression::Assignment { .. } => "assignment",
        _ => "expression",
    };
    name.to_owned()
}
