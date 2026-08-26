//! Bytecode verification and execution error definitions.

use iris_runtime::{ClassError, ConstructionError, KernelError, Value};

use crate::compile::{Instruction, Program, Register};

/// `IRIS-V1-CONTROL-C022` makes exactly `false` and `nil` falsey.
pub(crate) fn truthy(value: &Value) -> bool {
    !matches!(value, Value::Bool(false) | Value::Nil)
}

/// Why execution stopped.
#[derive(Clone, Debug, PartialEq)]
pub enum MachineError {
    /// The program raised through the kernel.
    Kernel(KernelError),
    /// The program failed verification.
    ///
    /// This is a COMPILER defect rather than a program error.
    Invalid(VerifyError),
    /// A selector the runtime does not treat as native.
    UnknownSelector(String),
    Class(ClassError),
    Construction(ConstructionError),
    NameError,
    ClosedGenericOpenForbidden,
    DefiniteAssignment,
    /// An Array changed while an iterator over it was active.
    ///
    /// C026 versions every length-changing or element-replacing operation so
    /// the iterator raises on its NEXT advance rather than quietly walking a
    /// collection that is no longer the one it started on.
    ConcurrentModification,
    IteratorState,
    /// A write to a position outside the Array.
    ///
    /// A READ past the end answers nil, but a WRITE has no position to store
    /// into, so it raises rather than silently discarding the value.
    IndexError,
    TypeContractError,
    ReflectionAccess,
    JsonSyntaxError,
    SerializationError,
    UnsupportedConstruct,
    AuditHistoryUnavailable,
    HostDriveUnavailable,
    NoActiveException,
    MessageNotFound {
        receiver_class: String,
        selector: String,
    },
    /// An Iris value propagated beyond the current frame.
    Raised(Box<(Value, Value)>),
}

/// Why a program is not well formed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    /// An instruction names a register outside the program's register file.
    RegisterOutOfRange { register: Register },
    /// An instruction reads a register never written.
    ///
    /// The old C++ VM read operands with `vector::operator[]` and no verifier,
    /// so corrupt bytecode was undefined behaviour. Refusing an unwritten read
    /// is what keeps a malformed program a reportable error instead.
    ReadBeforeWrite { register: Register },
    /// A `BuildArray` or `Call` names a range that leaves the register file.
    ArrayRangeOutOfRange { first: Register, count: u16 },
    /// A jump names an instruction outside the body.
    JumpOutOfRange { target: usize },
    /// A call names a function the program does not define.
    UnknownFunction { function: usize },
    /// A frame ran past its last instruction without returning.
    MissingReturn,
}

/// Checks that every register an instruction reads is in range and written.
///
/// This is ONE linear pass, which is a direct benefit of the register IR:
/// there is no operand stack whose depth must be simulated per control path.
///
/// # Errors
/// Returns the first malformation found.
pub fn verify(program: &Program) -> Result<(), VerifyError> {
    for function in &program.functions {
        verify_body(
            &function.instructions,
            function.registers,
            program.functions.len(),
            // A parameter arrives pre-bound in the frame, so it counts as
            // written before the body's first instruction.
            function.parameters,
            None,
        )?;
    }
    verify_body(
        &program.instructions,
        program.registers,
        program.functions.len(),
        0,
        Some(program.result),
    )
}

fn verify_body(
    instructions: &[Instruction],
    registers: usize,
    functions: usize,
    parameters: usize,
    result: Option<Register>,
) -> Result<(), VerifyError> {
    if parameters > registers {
        return Err(VerifyError::RegisterOutOfRange {
            register: Register::try_from(parameters).unwrap_or(Register::MAX),
        });
    }

    // Structural checks first: every register and target must be nameable
    // before any dataflow over them means anything.
    for instruction in instructions {
        for register in reads(instruction) {
            if register as usize >= registers {
                return Err(VerifyError::RegisterOutOfRange { register });
            }
        }
        if let Some(destination) = instruction.destination()
            && destination as usize >= registers
        {
            return Err(VerifyError::RegisterOutOfRange {
                register: destination,
            });
        }
        if let Instruction::DeclareDeferred { assigned, .. } = instruction
            && *assigned as usize >= registers
        {
            return Err(VerifyError::RegisterOutOfRange {
                register: *assigned,
            });
        }
        match instruction {
            Instruction::JumpUnless { target, .. } | Instruction::Jump { target } => {
                if *target > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *target });
                }
            }
            Instruction::ArrayNext { exhausted, .. }
            | Instruction::RangeNext { exhausted, .. }
            | Instruction::IteratorNext { exhausted, .. } => {
                if *exhausted > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *exhausted });
                }
            }
            Instruction::EnterTry {
                handler,
                cleanup,
                exception,
                context,
            } => {
                for register in [exception, context] {
                    if *register as usize >= registers {
                        return Err(VerifyError::RegisterOutOfRange {
                            register: *register,
                        });
                    }
                }
                for target in [handler, cleanup] {
                    if *target > instructions.len() {
                        return Err(VerifyError::JumpOutOfRange { target: *target });
                    }
                }
            }
            Instruction::Call { function, .. } => {
                if *function >= functions {
                    return Err(VerifyError::UnknownFunction {
                        function: *function,
                    });
                }
                if let Instruction::Call { first, count, .. } = instruction {
                    range(*first, *count, registers)?;
                }
            }
            Instruction::BareCall { first, count, .. } => {
                range(*first, *count, registers)?;
            }
            Instruction::MakeClosure {
                function,
                first,
                count,
                ..
            } => {
                if *function >= functions {
                    return Err(VerifyError::UnknownFunction {
                        function: *function,
                    });
                }
                range(*first, *count, registers)?;
            }
            Instruction::DefineMethod { function, .. } => {
                if *function >= functions {
                    return Err(VerifyError::UnknownFunction {
                        function: *function,
                    });
                }
            }
            Instruction::BuildArray { first, count, .. }
            | Instruction::BuildTuple { first, count, .. }
            | Instruction::New { first, count, .. }
            | Instruction::Send { first, count, .. }
            | Instruction::SendSuper { first, count, .. }
            | Instruction::SendClass { first, count, .. }
            | Instruction::Reflection { first, count, .. }
            | Instruction::Json { first, count, .. }
            | Instruction::SendContract { first, count, .. } => {
                range(*first, *count, registers)?;
            }
            Instruction::BuildHash { first, count, .. } => {
                let slots = count
                    .checked_mul(2)
                    .ok_or(VerifyError::ArrayRangeOutOfRange {
                        first: *first,
                        count: *count,
                    })?;
                range(*first, slots, registers)?;
            }
            _ => {}
        }
    }

    // Definite assignment is a DATAFLOW question, not a linear one.
    //
    // A single pass over the instruction list is unsound the moment control
    // flow exists: a forward jump that SKIPS a write leaves the scan believing
    // the register was written, because the scan walked past the instruction
    // that execution never ran. A backward jump breaks it the other way, since
    // a loop body is entered before its own writes have happened.
    //
    // This is therefore a fixpoint over the control-flow graph, keeping the
    // INTERSECTION of what is written along every path reaching a point. It
    // terminates because each entry set only ever shrinks.
    let mut entry: Vec<Option<Vec<bool>>> = vec![None; instructions.len()];
    let mut start = vec![false; registers];
    // A parameter arrives pre-bound, so it is written on entry.
    for slot in start.iter_mut().take(parameters) {
        *slot = true;
    }
    if !instructions.is_empty() {
        entry[0] = Some(start);
    }

    let mut pending: Vec<usize> = if instructions.is_empty() {
        Vec::new()
    } else {
        vec![0]
    };
    while let Some(at) = pending.pop() {
        let Some(Some(state)) = entry.get(at).cloned() else {
            continue;
        };
        let Some(instruction) = instructions.get(at) else {
            continue;
        };
        let mut next = state.clone();
        if let Some(destination) = instruction.destination() {
            next[destination as usize] = true;
        }
        // A declaration writes BOTH of its registers, and the flag is not the
        // instruction's value, so the fixpoint would otherwise prove it
        // unwritten at the read that consults it.
        if let Instruction::DeclareDeferred { assigned, .. } = instruction {
            next[*assigned as usize] = true;
        }

        let successors: Vec<(usize, Vec<bool>)> = match instruction {
            // A return leaves the frame, so it has no successor at all.
            Instruction::Return { .. } => Vec::new(),
            Instruction::Raise { .. }
            | Instruction::ReRaise { .. }
            | Instruction::RaiseNoActiveException
            | Instruction::Propagate { .. } => Vec::new(),
            Instruction::Jump { target } => vec![(*target, next.clone())],
            // Both edges are live: the branch may be taken or not.
            Instruction::JumpUnless { target, .. } => {
                vec![(*target, next.clone()), (at + 1, next.clone())]
            }
            Instruction::ArrayNext { exhausted, .. }
            | Instruction::RangeNext { exhausted, .. }
            | Instruction::IteratorNext { exhausted, .. } => {
                vec![(*exhausted, state.clone()), (at + 1, next.clone())]
            }
            Instruction::EnterTry {
                handler,
                exception,
                context,
                ..
            } => {
                let mut exceptional = state.clone();
                exceptional[*exception as usize] = true;
                exceptional[*context as usize] = true;
                vec![(*handler, exceptional), (at + 1, next.clone())]
            }
            _ => vec![(at + 1, next.clone())],
        };
        for (successor, incoming) in successors {
            if successor >= instructions.len() {
                continue;
            }
            let merged = match &entry[successor] {
                // Merging keeps only what is written on BOTH paths, which is
                // what makes a skipped write stop counting as written.
                Some(existing) => {
                    let merged: Vec<bool> = existing
                        .iter()
                        .zip(&incoming)
                        .map(|(held, incoming)| *held && *incoming)
                        .collect();
                    (merged != *existing).then_some(merged)
                }
                None => Some(incoming),
            };
            if let Some(merged) = merged {
                entry[successor] = Some(merged);
                pending.push(successor);
            }
        }
    }

    // Now check every reachable instruction against what is definitely
    // written when it runs. An UNREACHABLE instruction is not checked: it
    // never executes, so it cannot read anything.
    for (at, instruction) in instructions.iter().enumerate() {
        let Some(state) = &entry[at] else { continue };
        for register in reads(instruction) {
            if !state[register as usize] {
                return Err(VerifyError::ReadBeforeWrite { register });
            }
        }
    }

    if let Some(result) = result {
        if result as usize >= registers {
            return Err(VerifyError::RegisterOutOfRange { register: result });
        }
        // The result is read after the LAST instruction, so it must be
        // written on every path that falls off the end.
        let final_state = if instructions.is_empty() {
            let mut start = vec![false; registers];
            for slot in start.iter_mut().take(parameters) {
                *slot = true;
            }
            Some(start)
        } else {
            fall_through(instructions, &entry, registers)
        };
        if let Some(state) = final_state
            && !state[result as usize]
        {
            return Err(VerifyError::ReadBeforeWrite { register: result });
        }
    }
    Ok(())
}

/// What is definitely written where control falls off the end of a body.
fn fall_through(
    instructions: &[Instruction],
    entry: &[Option<Vec<bool>>],
    registers: usize,
) -> Option<Vec<bool>> {
    let mut reaching: Option<Vec<bool>> = None;
    for (at, instruction) in instructions.iter().enumerate() {
        let Some(state) = &entry[at] else { continue };
        let leaves = match instruction {
            Instruction::Return { .. } => false,
            Instruction::Raise { .. }
            | Instruction::ReRaise { .. }
            | Instruction::RaiseNoActiveException
            | Instruction::Propagate { .. } => false,
            Instruction::Jump { target } => *target >= instructions.len(),
            Instruction::JumpUnless { target, .. } => {
                *target >= instructions.len() || at + 1 >= instructions.len()
            }
            Instruction::ArrayNext { exhausted, .. }
            | Instruction::RangeNext { exhausted, .. }
            | Instruction::IteratorNext { exhausted, .. } => {
                *exhausted >= instructions.len() || at + 1 >= instructions.len()
            }
            _ => at + 1 >= instructions.len(),
        };
        if !leaves {
            continue;
        }
        let mut after = state.clone();
        if let Some(destination) = instruction.destination() {
            after[destination as usize] = true;
        }
        reaching = Some(match reaching {
            Some(held) => held
                .iter()
                .zip(&after)
                .map(|(one, other)| *one && *other)
                .collect(),
            None => after,
        });
    }
    reaching.or_else(|| Some(vec![true; registers]))
}

/// Every register an instruction reads.
fn reads(instruction: &Instruction) -> Vec<Register> {
    match instruction {
        Instruction::Move { source, .. }
        | Instruction::MakeCell { source, .. }
        | Instruction::MakeMutableString { source, .. } => vec![*source],
        Instruction::LoadCell { cell, .. } => vec![*cell],
        Instruction::StoreCell { cell, source, .. } => vec![*cell, *source],
        Instruction::BuildIterationYield { value, .. } => vec![*value],
        Instruction::StoreGlobal { value, .. } => vec![*value],
        Instruction::PublishBinding { source, .. } | Instruction::StoreBinding { source, .. } => {
            vec![*source]
        }
        Instruction::Binary { left, right, .. } | Instruction::Identity { left, right, .. } => {
            vec![*left, *right]
        }
        Instruction::TypeTest { value, target, .. } => vec![*value, *target],
        Instruction::BuildRange { start, end, .. } => vec![*start, *end],
        Instruction::Unary { operand, .. } => vec![*operand],
        Instruction::FromBits { bits, .. } => vec![*bits],
        Instruction::Await { task, .. } | Instruction::HostRun { task, .. } => vec![*task],
        Instruction::UnobservedFailures { .. } => Vec::new(),
        Instruction::JumpUnless { condition, .. } => vec![*condition],
        Instruction::Return { value } | Instruction::Raise { value, .. } => vec![*value],
        Instruction::ReRaise { value, context, .. } => vec![*value, *context],
        Instruction::Propagate { value, context } => vec![*value, *context],
        Instruction::BuildArray { first, count, .. }
        | Instruction::BuildTuple { first, count, .. }
        | Instruction::Call { first, count, .. }
        | Instruction::MakeClosure { first, count, .. }
        | Instruction::New { first, count, .. } => {
            (0..*count).map(|offset| first + offset).collect()
        }
        Instruction::BuildHash { first, count, .. } => (0..count.saturating_mul(2))
            .map(|offset| first + offset)
            .collect(),
        Instruction::BareCall {
            callee,
            first,
            count,
            ..
        } => callee
            .iter()
            .copied()
            .chain((0..*count).map(|offset| first + offset))
            .collect(),
        Instruction::Using {
            resource, block, ..
        } => vec![*resource, *block],
        Instruction::Send {
            receiver,
            first,
            count,
            ..
        } => std::iter::once(*receiver)
            .chain((0..*count).map(|offset| first + offset))
            .collect(),
        Instruction::SendSuper {
            receiver,
            first,
            count,
            ..
        } => std::iter::once(*receiver)
            .chain((0..*count).map(|offset| first + offset))
            .collect(),
        Instruction::SendClass { first, count, .. } => {
            (0..*count).map(|offset| first + offset).collect()
        }
        Instruction::Reflection { first, count, .. }
        | Instruction::Revision { first, count, .. }
        | Instruction::Json { first, count, .. } => {
            (0..*count).map(|offset| first + offset).collect()
        }
        Instruction::OpenClass { callback, .. } => vec![*callback],
        Instruction::DefineMethod { receiver, name, .. } => vec![*receiver, *name],
        Instruction::SendContract {
            receiver,
            first,
            count,
            ..
        } => std::iter::once(*receiver)
            .chain((0..*count).map(|offset| first + offset))
            .collect(),
        Instruction::ContractCast { receiver, .. } => vec![*receiver],
        Instruction::Index {
            receiver, index, ..
        } => vec![*receiver, *index],
        Instruction::ArrayVersion { array, .. } => vec![*array],
        Instruction::ArrayNext {
            array,
            index,
            version,
            ..
        } => vec![*array, *index, *version],
        Instruction::RangeNext { range, index, .. } => vec![*range, *index],
        Instruction::IteratorOpen { iterable, .. } => vec![*iterable],
        Instruction::IteratorNext { iterator, .. } | Instruction::IteratorClose { iterator } => {
            vec![*iterator]
        }
        Instruction::SetIndex {
            receiver,
            index,
            value,
            ..
        } => vec![*receiver, *index, *value],
        Instruction::BindMember { receiver, .. } => vec![*receiver],
        Instruction::GetIvar { receiver, .. } => vec![*receiver],
        Instruction::SetIvar {
            receiver, value, ..
        } => vec![*receiver, *value],
        Instruction::GetClassVar { receiver, .. } => vec![*receiver],
        Instruction::SetClassVar {
            receiver, value, ..
        } => vec![*receiver, *value],
        Instruction::CatchMatch { exception, .. } => vec![*exception],
        Instruction::ReadDeferred {
            value, assigned, ..
        } => vec![*value, *assigned],
        Instruction::TestTruth { value, .. }
        | Instruction::NegateTruth { value, .. }
        | Instruction::MakeKeywordArgument { value, .. } => vec![*value],
        Instruction::LoadInteger { .. }
        | Instruction::LoadFloat64 { .. }
        | Instruction::LoadFloat32 { .. }
        | Instruction::LoadText { .. }
        | Instruction::LoadBytes { .. }
        | Instruction::LoadByteArray { .. }
        | Instruction::LoadSymbol { .. }
        | Instruction::LoadBool { .. }
        | Instruction::LoadNil { .. }
        | Instruction::LoadIterationDone { .. }
        | Instruction::LoadClass { .. }
        | Instruction::LoadType { .. }
        | Instruction::LoadContract { .. }
        | Instruction::LoadBuiltinType { .. }
        | Instruction::BuildType { .. }
        | Instruction::LoadBuiltinClass { .. }
        | Instruction::LoadGlobal { .. }
        | Instruction::LoadBinding { .. }
        | Instruction::EnterTry { .. }
        | Instruction::LeaveTry
        | Instruction::Jump { .. }
        | Instruction::DeclareDeferred { .. }
        | Instruction::MarkAssigned { .. }
        | Instruction::RaiseNoActiveException => Vec::new(),
        Instruction::RaiseUnsupported { .. } | Instruction::RaiseNameError { .. } => Vec::new(),
    }
}

/// Checks a contiguous register range lies within the file.
fn range(first: Register, count: u16, registers: usize) -> Result<(), VerifyError> {
    let last = (first as usize)
        .checked_add(count as usize)
        .ok_or(VerifyError::ArrayRangeOutOfRange { first, count })?;
    if last > registers {
        return Err(VerifyError::ArrayRangeOutOfRange { first, count });
    }
    Ok(())
}
