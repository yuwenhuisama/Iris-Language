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
use lowering::{ProgramBinding, lower_function};

mod calls;
mod expressions_lowering;
mod ir;
mod lowering;
mod statements;

pub(crate) use ir::{
    Class, ClassReopen, ClassVariable, Contract, ContractRequirement, Function, LiteralValue,
    StoredProperty,
};
pub use ir::{FloatWidth, Instruction, ParameterKind, Program, Register};

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
    // A source the PARSER refuses is a program error the reference raises when
    // the program runs, so the backend answers a program that raises it rather
    // than declining - both refuse it either way, but only one of those can
    // agree with the reference.
    if !parsed.program_accepted {
        return Ok(Program {
            source: source.to_owned(),
            instructions: vec![Instruction::RaiseParseDiagnostic { destination: 0 }],
            registers: 1,
            result: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            contracts: Vec::new(),
            modules: Vec::new(),
            builtin_reopens: Vec::new(),
        });
    }

    // A declaration naming a target that does not EXIST - a reopen of an
    // undeclared class, a contract inheriting an undeclared parent - is a
    // program error the reference raises when the program runs, exactly as a
    // parse rejection is. Declining made both backends refuse the same program
    // while describing it differently, which holds the row rather than
    // agreeing, so the backend answers a program that raises instead.
    if let Err(error) = collect_signatures(&parsed.program.declarations)
        && matches!(
            error.construct.as_str(),
            "class reopen target"
                | "contract parent unbound"
                | "module mixin unbound"
                | "contract signature clash"
        )
    {
        // A contract SIGNATURE clash is a type failure rather than a missing
        // construct, so it raises the error the reference raises for it.
        let raise = if error.construct == "contract signature clash" {
            Instruction::RaiseTypeContract { destination: 0 }
        } else {
            Instruction::RaiseUnsupported { destination: 0 }
        };
        return Ok(Program {
            source: source.to_owned(),
            instructions: vec![raise],
            registers: 1,
            result: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            contracts: Vec::new(),
            modules: Vec::new(),
            builtin_reopens: Vec::new(),
        });
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
        modules,
        builtin_reopens,
    } = collect_signatures(&parsed.program.declarations)?;

    let mut functions = Vec::with_capacity(signatures.len());
    let mut closures = Vec::new();
    let program_bindings: Vec<ProgramBinding> = parsed
        .program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::Binding { mutable, name, .. } => Some(ProgramBinding {
                name: name.clone(),
                shared: *mutable,
            }),
            _ => None,
        })
        .collect();
    for signature in &signatures {
        functions.push(lower_function(
            signature,
            &signatures,
            &classes,
            &contracts,
            signatures.len(),
            &mut closures,
            &program_bindings,
        )?);
    }

    let mut lowering = lowering::Lowering::new(
        &signatures,
        &classes,
        &contracts,
        signatures.len(),
        &mut closures,
        &program_bindings,
        true,
    );
    // A top-level program answers the values of its non-BINDING statements:
    // one value directly, several as an Array. That convention belongs to the
    // reference evaluator, and a backend that answered only the last statement
    // would disagree with it for a reason that is not semantic - which is
    // exactly what the differential harness exists to catch.
    //
    // `entries` is walked rather than `statements` because a MODULE body's
    // ordinary statements run at its declaration's SOURCE POSITION, not in a
    // separate phase: `module M { order.append(:body) }` after `order` is
    // bound appends, and before it is a NameError. Their values are discarded,
    // since a module body contributes nothing to the program's answer.
    let mut produced = Vec::new();
    let mut applied: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    let lowering_classes = classes.clone();
    for entry in &parsed.program.entries {
        match entry {
            iris_syntax::ProgramEntry::Statement(statement) => {
                let value = lowering.statement(statement)?;
                if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
                    produced.push(value);
                }
            }
            // `from S import K` binds the module's CONSTANT under the
            // imported name, at the import's own source position. Only a
            // constant is bound: a module's methods are reached as `S.f()`
            // rather than by name, so importing one would need a callable
            // binding this backend has no other support for.
            iris_syntax::ProgramEntry::Declaration(iris_syntax::Declaration::Import(import)) => {
                for spec in &import.specs {
                    let Some(value) =
                        parsed.program.declarations.iter().find_map(
                            |declaration| match declaration {
                                iris_syntax::Declaration::Module(module)
                                    if module.name == import.target =>
                                {
                                    module.body.iter().find_map(|statement| match statement {
                                        Statement::Binding {
                                            constant: true,
                                            name,
                                            value,
                                            ..
                                        } if *name == spec.name => Some(value),
                                        _ => None,
                                    })
                                }
                                _ => None,
                            },
                        )
                    else {
                        // A spec naming nothing the module declares binds no
                        // name, and the reference still RUNS the program -
                        // `from S import Missing; 1` answers `1` - so this is
                        // a no-op rather than a refusal.
                        continue;
                    };
                    let register = lowering.expression(value)?;
                    let name = spec.alias.clone().unwrap_or_else(|| spec.name.clone());
                    lowering
                        .names
                        .push(lowering::Binding::value(name, register));
                }
            }
            iris_syntax::ProgramEntry::Declaration(iris_syntax::Declaration::Module(module)) => {
                lowering.enclosing_module = Some(module.name.clone());
                for statement in &module.body {
                    // A stored property DECLARES a member rather than being an
                    // ordinary body statement, so it is skipped here the way a
                    // method and a constant are: it was already synthesized
                    // into a reader while the declarations were collected.
                    if matches!(
                        statement,
                        Statement::Method(_)
                            | Statement::Binding { constant: true, .. }
                            | Statement::StoredProperty { .. }
                    ) {
                        continue;
                    }
                    lowering.statement(statement)?;
                }
                lowering.enclosing_module = None;
            }
            // A class REOPEN takes effect where it was written, so it is
            // applied from this position rather than at load. Reopens of one
            // class are counted in source order, which is the order
            // `collect_signatures` recorded them in.
            iris_syntax::ProgramEntry::Declaration(iris_syntax::Declaration::Class(class))
                if class.reopen =>
            {
                if let Some(target) = lowering_classes
                    .iter()
                    .position(|known: &crate::compile::ir::Class| known.name == class.name)
                {
                    let reopen = *applied.entry(target).or_insert(0);
                    applied.insert(target, reopen + 1);
                    lowering.instructions.push(Instruction::ApplyReopen {
                        class: target,
                        reopen,
                    });
                }
            }
            iris_syntax::ProgramEntry::Declaration(_) => {}
        }
    }
    let result = match produced.as_slice() {
        // A program of only declarations, or only bindings, answers no value.
        // The reference raises UnsupportedConstruct when it RUNS, so refusing
        // at compile time made both backends refuse the same program while
        // describing it differently - which holds the row rather than agreeing.
        [] => {
            let destination = lowering.allocate()?;
            lowering
                .instructions
                .push(Instruction::RaiseUnsupported { destination });
            destination
        }
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
        modules,
        builtin_reopens,
    })
}
