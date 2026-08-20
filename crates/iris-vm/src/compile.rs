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
    /// Sends a native zero-argument selector to the topmost value.
    Nullary(&'static str),
    /// Pushes a Bool.
    PushBool(bool),
    /// Pushes nil.
    PushNil,
    /// Builds an Array from the topmost `count` values, in source order.
    BuildArray(usize),
    /// Builds a float of the given width from the topmost Integer's bits.
    ///
    /// `IRIS-V1-RUNTIME-C113` fixes the accepted range per width and requires
    /// `RangeError` outside it, and `C114` requires `from_bits(b).to_bits()`
    /// to round-trip every pattern INCLUDING signaling NaN, so the width is
    /// carried explicitly rather than inferred from the value.
    FromBits(FloatWidth),
}

/// Which IEEE interchange width a `from_bits` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatWidth {
    /// IEEE-754 binary32.
    Bits32,
    /// IEEE-754 binary64.
    Bits64,
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
        // As a RECEIVER these keyword values arrive as a Name rather than a
        // Literal, so `nil.hash()` reached this backend as an unresolved name.
        Expression::Name(name) if matches!(name.as_str(), "nil" | "true" | "false") => {
            lower_literal(name, instructions)
        }
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
                // The runtime spells this `negate`; `-@` is not a native
                // selector, so emitting it produced a machine defect rather
                // than the RangeError the interpreter answers.
                UnaryOperator::Negate => "negate",
                UnaryOperator::Not => return Err(CompileError::new("unary not")),
                other => return Err(CompileError::new(format!("unary {other:?}"))),
            };
            lower(operand, instructions)?;
            instructions.push(Instruction::Unary(selector));
            Ok(())
        }
        // An Array literal is a pure aggregate of its elements, so it needs
        // no frames: each element is lowered in source order and collected.
        Expression::Array(elements) => {
            for element in elements {
                lower(element, instructions)?;
            }
            instructions.push(Instruction::BuildArray(elements.len()));
            Ok(())
        }
        // `Float32.from_bits(bits)` and `value.to_bits()` are the two call
        // shapes RUNTIME-V066 needs. Only these are covered: a general call
        // needs frames and user Methods this subset does not have.
        Expression::Call {
            callee, arguments, ..
        } => lower_call(callee, arguments, instructions),
        other => Err(CompileError::new(construct_name(other))),
    }
}

fn lower_call(
    callee: &Expression,
    arguments: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Result<(), CompileError> {
    let Expression::Member { receiver, selector } = callee else {
        return Err(CompileError::new("call"));
    };
    if let Expression::Name(name) = receiver.as_ref()
        && selector == "from_bits"
    {
        let width = match name.as_str() {
            "Float32" => FloatWidth::Bits32,
            "Float64" => FloatWidth::Bits64,
            _ => return Err(CompileError::new("call")),
        };
        let [bits] = arguments else {
            return Err(CompileError::new("from_bits arity"));
        };
        lower(bits, instructions)?;
        instructions.push(Instruction::FromBits(width));
        return Ok(());
    }
    // `IRIS-V1-RUNTIME-C146` fixes the public hash of each numeric and
    // singleton value, and `V073` compares those across backends.
    if arguments.is_empty()
        && let Some(native) = match selector.as_str() {
            "to_bits" => Some("to_bits"),
            "hash" => Some("hash"),
            _ => None,
        }
    {
        lower(receiver, instructions)?;
        instructions.push(Instruction::Nullary(native));
        return Ok(());
    }
    Err(CompileError::new("call"))
}

/// Lowers a literal by RE-LEXING its text.
///
/// The parser keeps a literal as source text, and the lexer is what turns it
/// into a value under the chapter 02 rules - suffixes, radix prefixes,
/// separators and precision warnings. Parsing the text here instead would be a
/// second literal implementation, and a differential row would then be
/// comparing this backend's literal rules against the lexer's.
fn lower_literal(text: &str, instructions: &mut Vec<Instruction>) -> Result<(), CompileError> {
    // The parser keeps these keyword values as literal TEXT, and the lexer
    // does not convert them, so they are recognised here rather than being
    // rejected as an unconvertible literal.
    match text {
        "nil" => {
            instructions.push(Instruction::PushNil);
            return Ok(());
        }
        "true" | "false" => {
            instructions.push(Instruction::PushBool(text == "true"));
            return Ok(());
        }
        _ => {}
    }
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
