//! Lowers a parsed program to register instructions.
//!
//! The execution IR is REGISTER-based rather than stack-based, per the design
//! review's section 5.7: it lowers more directly to Cranelift IR, has no stack
//! effect to track, gives a simpler verifier, disassembles readably, cannot
//! underflow an operand stack, and suits later SSA lowering.

use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

/// A virtual register index.
///
/// Registers are virtual and unbounded at this stage. Allocation to a fixed
/// bank belongs to a later pass; assigning them here would bake a machine
/// constraint into the IR before any backend needs it.
pub type Register = u16;

/// One three-address instruction.
///
/// Every instruction names its operands and its destination explicitly, so an
/// instruction's meaning does not depend on execution history. That is what
/// makes the verifier a single linear pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Loads an arbitrary-precision Integer, held as canonical decimal text.
    LoadInteger {
        destination: Register,
        digits: String,
    },
    /// Loads an IEEE-754 binary64 value, held as BITS so a literal cannot
    /// drift through a decimal round trip.
    LoadFloat64 { destination: Register, bits: u64 },
    /// Loads an IEEE-754 binary32 value, held as bits for the same reason.
    LoadFloat32 { destination: Register, bits: u32 },
    /// Loads a String.
    LoadText { destination: Register, text: String },
    /// Loads a Bool.
    LoadBool { destination: Register, value: bool },
    /// Loads nil.
    LoadNil { destination: Register },
    /// Copies one register to another.
    Move {
        destination: Register,
        source: Register,
    },
    /// Sends a native binary selector: `destination = left <selector> right`.
    Binary {
        destination: Register,
        selector: &'static str,
        left: Register,
        right: Register,
    },
    /// Sends a native selector with no arguments to one register.
    Unary {
        destination: Register,
        selector: &'static str,
        operand: Register,
    },
    /// Builds an Array from a contiguous register range, in source order.
    BuildArray {
        destination: Register,
        first: Register,
        count: u16,
    },
    /// Reinterprets an Integer register's bits as a float of the given width.
    FromBits {
        destination: Register,
        width: FloatWidth,
        bits: Register,
    },
}

impl Instruction {
    /// The register this instruction writes.
    pub(crate) const fn destination(&self) -> Register {
        match self {
            Self::LoadInteger { destination, .. }
            | Self::LoadFloat64 { destination, .. }
            | Self::LoadFloat32 { destination, .. }
            | Self::LoadText { destination, .. }
            | Self::LoadBool { destination, .. }
            | Self::LoadNil { destination }
            | Self::Move { destination, .. }
            | Self::Binary { destination, .. }
            | Self::Unary { destination, .. }
            | Self::BuildArray { destination, .. }
            | Self::FromBits { destination, .. } => *destination,
        }
    }
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
    /// How many registers the program uses.
    pub(crate) registers: usize,
    /// The register holding the program's answer.
    pub(crate) result: Register,
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
    let Some((last, leading)) = parsed.program.statements.split_last() else {
        return Err(CompileError::new("empty program"));
    };
    let mut lowering = Lowering::default();
    for statement in leading {
        // An earlier statement's value is not the program's answer. Its
        // register is simply not read again; nothing has to be popped, which
        // is one of the reasons the IR is register-based.
        lowering.statement(statement)?;
    }
    let result = lowering.statement(last)?;
    Ok(Program {
        instructions: lowering.instructions,
        registers: lowering.next_register as usize,
        result,
    })
}

#[derive(Debug, Default)]
struct Lowering {
    instructions: Vec<Instruction>,
    next_register: Register,
    /// Names bound so far, each pinned to the register holding its value.
    names: Vec<(String, Register)>,
}

impl Lowering {
    /// Reserves a fresh register.
    fn allocate(&mut self) -> Result<Register, CompileError> {
        let register = self.next_register;
        self.next_register = self
            .next_register
            .checked_add(1)
            .ok_or_else(|| CompileError::new("register exhaustion"))?;
        Ok(register)
    }

    fn lookup(&self, name: &str) -> Option<Register> {
        self.names
            .iter()
            .rev()
            .find_map(|(held, register)| (held == name).then_some(*register))
    }

    fn statement(&mut self, statement: &Statement) -> Result<Register, CompileError> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            // A plain immutable binding is covered. `mut`, `const`, globals and
            // deferred bindings carry rules - reassignment, definite
            // assignment, package-qualified identity - this subset lacks.
            Statement::Binding {
                mutable: false,
                name,
                annotation: None,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                // A rebinding SHADOWS rather than overwrites: the earlier
                // register may still be read by a closure or an earlier
                // instruction, so reusing it would corrupt that read.
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: value,
                });
                self.names.push((name.clone(), destination));
                Ok(destination)
            }
            _ => Err(CompileError::new("statement")),
        }
    }

    fn expression(&mut self, expression: &Expression) -> Result<Register, CompileError> {
        match expression {
            Expression::Literal(text) => self.literal(text),
            // As a RECEIVER these keyword values arrive as a Name rather than
            // a Literal, so `nil.hash()` would otherwise be an unbound name.
            Expression::Name(name) if matches!(name.as_str(), "nil" | "true" | "false") => {
                self.literal(name)
            }
            Expression::Name(name) => self
                .lookup(name)
                // An unbound name is not this backend's to resolve: it could
                // be a Class, a Module, or a method-scope local.
                .ok_or_else(|| CompileError::new("name")),
            Expression::Grouped(inner) => self.expression(inner),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let selector = binary_selector(operator)?;
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Binary {
                    destination,
                    selector,
                    left,
                    right,
                });
                Ok(destination)
            }
            Expression::Unary { operator, operand } => {
                let UnaryOperator::Negate = operator else {
                    return Err(CompileError::new(format!("unary {operator:?}")));
                };
                // The runtime spells this `negate`; `-@` is not a native
                // selector and emitting it produced a machine defect rather
                // than the RangeError the reference answers.
                let operand = self.expression(operand)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Unary {
                    destination,
                    selector: "negate",
                    operand,
                });
                Ok(destination)
            }
            // An Array literal is a pure aggregate, so it needs no frames. The
            // elements are lowered into a CONTIGUOUS run of registers, which
            // lets the instruction name the range instead of carrying a list.
            Expression::Array(elements) => {
                let count = u16::try_from(elements.len())
                    .map_err(|_| CompileError::new("array too long"))?;
                let mut lowered = Vec::with_capacity(elements.len());
                for element in elements {
                    lowered.push(self.expression(element)?);
                }
                let first = self.next_register;
                for source in lowered {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source,
                    });
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BuildArray {
                    destination,
                    first,
                    count,
                });
                Ok(destination)
            }
            Expression::Call {
                callee, arguments, ..
            } => self.call(callee, arguments),
            other => Err(CompileError::new(construct_name(other))),
        }
    }

    /// Lowers a literal by RE-LEXING its text.
    ///
    /// The parser keeps a literal as source text, and the lexer is what turns
    /// it into a value under the chapter 02 rules - suffixes, radix prefixes,
    /// separators and precision warnings. Parsing the text here would be a
    /// second literal implementation, and a differential row would then compare
    /// this backend's literal rules against the lexer's.
    fn literal(&mut self, text: &str) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        // The parser keeps these keyword values as literal TEXT and the lexer
        // does not convert them, so they are recognised here.
        match text {
            "nil" => {
                self.instructions.push(Instruction::LoadNil { destination });
                return Ok(destination);
            }
            "true" | "false" => {
                self.instructions.push(Instruction::LoadBool {
                    destination,
                    value: text == "true",
                });
                return Ok(destination);
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
        self.instructions.push(match literal {
            iris_lexer::Literal::Integer(digits) => Instruction::LoadInteger {
                destination,
                digits: digits.clone(),
            },
            iris_lexer::Literal::Float64(number) => Instruction::LoadFloat64 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::Float32(number) => Instruction::LoadFloat32 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::String(text) => Instruction::LoadText {
                destination,
                text: text.clone(),
            },
        });
        Ok(destination)
    }

    /// `Float32.from_bits(bits)`, `value.to_bits()` and `value.hash()` are the
    /// call shapes this subset covers. A general call needs frames and user
    /// Methods it does not have.
    fn call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<Register, CompileError> {
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
            let bits = self.expression(bits)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::FromBits {
                destination,
                width,
                bits,
            });
            return Ok(destination);
        }
        if arguments.is_empty()
            && let Some(native) = match selector.as_str() {
                "to_bits" => Some("to_bits"),
                // C146 fixes the public hash of each numeric and singleton
                // value, which V073 compares across backends.
                "hash" => Some("hash"),
                _ => None,
            }
        {
            let operand = self.expression(receiver)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Unary {
                destination,
                selector: native,
                operand,
            });
            return Ok(destination);
        }
        Err(CompileError::new("call"))
    }
}

/// The native selector for a binary operator, when this backend covers it.
///
/// Only operators whose whole meaning is a native send appear here. A range,
/// regex or type operator carries semantics beyond the kernel send, so it is
/// declined rather than approximated.
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
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Compare => "<=>",
        other => return Err(CompileError::new(format!("operator {other:?}"))),
    })
}

fn construct_name(expression: &Expression) -> String {
    let name = match expression {
        Expression::Symbol(_) => "symbol",
        Expression::Hash(_) => "hash",
        Expression::Tuple(_) => "tuple",
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
