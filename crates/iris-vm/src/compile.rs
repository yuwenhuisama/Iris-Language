//! Lowers a parsed program to register instructions.
//!
//! The execution IR is REGISTER-based rather than stack-based, per the design
//! review's section 5.7: it lowers more directly to Cranelift IR, has no stack
//! effect to track, gives a simpler verifier, disassembles readably, cannot
//! underflow an operand stack, and suits later SSA lowering.

use iris_syntax::Statement;

mod declarations;
mod expressions;

use declarations::{CollectedDeclarations, collect_signatures};
use lowering::lower_function;

mod calls;
mod expressions_lowering;
mod ir;
mod lowering;
mod statements;

pub(crate) use ir::{
    Class, ClassReopen, ClassVariable, Contract, Function, LiteralValue, StoredProperty,
};
pub use ir::{FloatWidth, Instruction, Program, Register};

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

    // Name resolution happens HERE, before any instruction is emitted: a call
    // is lowered to a function INDEX, never to a name looked up at run time.
    // The design review places this responsibility in a HIR layer between AST
    // and execution IR; this is that resolution, done in one pass while the
    // covered surface is small enough not to need a separate representation.
    let CollectedDeclarations {
        signatures,
        classes,
        contracts,
    } = collect_signatures(&parsed.program.declarations)?;

    let mut functions = Vec::with_capacity(signatures.len());
    let mut closures = Vec::new();
    for signature in &signatures {
        functions.push(lower_function(
            signature,
            &signatures,
            &classes,
            &contracts,
            signatures.len(),
            &mut closures,
        )?);
    }

    if parsed.program.statements.is_empty() {
        return Err(CompileError::new("empty program"));
    }
    let mut lowering = lowering::Lowering::new(
        &signatures,
        &classes,
        &contracts,
        signatures.len(),
        &mut closures,
    );
    // A top-level program answers the values of its non-BINDING statements:
    // one value directly, several as an Array. That convention belongs to the
    // reference evaluator, and a backend that answered only the last statement
    // would disagree with it for a reason that is not semantic - which is
    // exactly what the differential harness exists to catch.
    let mut produced = Vec::new();
    for statement in &parsed.program.statements {
        let value = lowering.statement(statement)?;
        if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
            produced.push(value);
        }
    }
    let result = match produced.as_slice() {
        [] => return Err(CompileError::new("no program value")),
        [single] => *single,
        _ => {
            let first = lowering.next_register;
            let count =
                u16::try_from(produced.len()).map_err(|_| CompileError::new("program too wide"))?;
            for source in produced {
                let destination = lowering.allocate()?;
                lowering.instructions.push(Instruction::Move {
                    destination,
                    source,
                });
            }
            let destination = lowering.allocate()?;
            lowering.instructions.push(Instruction::BuildArray {
                destination,
                first,
                count,
            });
            destination
        }
    };
    let registers = lowering.next_register as usize;
    let instructions = std::mem::take(&mut lowering.instructions);
    drop(lowering);
    functions.extend(closures);
    Ok(Program {
        source: source.to_owned(),
        instructions,
        registers,
        result,
        functions,
        classes,
        contracts,
    })
}
