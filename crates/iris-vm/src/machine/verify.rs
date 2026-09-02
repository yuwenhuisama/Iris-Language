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
    /// A `break` or `continue` reached NO enclosing loop.
    ///
    /// `IRIS-V1-CONTROL-C069` requires every control transfer to have a
    /// target; one that does not is a program error the reference reports
    /// when the transfer runs, not a construct the backend lacks.
    LoopTransferOutsideLoop,
    /// An argument list did not satisfy the parameter list.
    ///
    /// `IRIS-V1-CONTROL-C025` raises this when a required parameter is left
    /// unbound or a supplied argument matches nothing, and `D-357` makes a
    /// DUPLICATE keyword an error rather than a silent last-one-wins.
    ArgumentError,
    /// A Hash `fetch` named a key the Hash does not hold.
    ///
    /// `fetch` differs from indexing exactly here: indexing answers nil for an
    /// absent key, and `fetch` refuses, which is what makes it usable as an
    /// assertion that the key is present.
    KeyError,
    /// A value fell outside the range its operation admits.
    ///
    /// `IRIS-V1-COLLECTIONS-C038` refuses a Range step of zero, and refuses one
    /// whose SIGN walks away from the end - a step that never terminates is
    /// rejected up front rather than looping forever.
    RangeError,
    /// A byte sequence was not valid in the selected Encoding.
    ///
    /// `IRIS-V1-LIBRARY-C022` makes STRICT handling the default, so a lossy
    /// result appears only when the caller asks for it by name.
    EncodingError,
    /// A native symbol was called without being BOUND.
    ///
    /// `IRIS-V1-FFI-C045` forbids invoking an unbound symbol and `C046` denies
    /// any signature-less escape hatch, so the refusal happens before any call
    /// rather than at the boundary.
    UnboundNativeSymbol,
    /// A native signature omitted data `IRIS-V1-FFI-C047` requires.
    ///
    /// `C047` lists what a signature MUST declare and requires missing data to
    /// reject the binding BEFORE any call occurs, so this is a presence check
    /// rather than a deferred one.
    IncompleteNativeSignature,
    /// A NAMED diagnostic the reference reports by code.
    ///
    /// A serialization header or limit failure is reported as its own code
    /// rather than a generic error, because a stream can fail for reasons a
    /// program distinguishes: an incompatible header is not a limit breach.
    LexicalDiagnostic(&'static str),
    /// The PARSER refused the source.
    ///
    /// The reference reports this when the program runs rather than refusing
    /// to compile, so the backends agree on the refusal rather than merely
    /// both refusing.
    ParseDiagnostic,
    ClosedGenericOpenForbidden,
    /// An async frame PAUSED at an `await` on an incomplete Gate.
    ///
    /// This is a control signal rather than a failure: it unwinds to the async
    /// call that started the frame, which records the paused state and answers
    /// a Task. It must never reach a `catch`, because `await` on a pending
    /// Gate is not an error the program can observe.
    Suspended(iris_runtime::ObjectId),
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
    /// A write to a binding that is not `mut`.
    ///
    /// `IRIS-V1-CONTROL-C009` makes `let`, a parameter and a loop variable
    /// IMMUTABLE, so only a `mut` binding may be assigned - a write to any
    /// other names a place the program cannot change.
    ImmutableBinding,
    /// Two entries that COLLIDE into one equality class on a rehash.
    ///
    /// `IRIS-V1-COLLECTIONS-C031` raises when previously distinct keys become
    /// equal and no merge block was supplied to resolve them.
    KeyConflictError,
    /// An ivar NAME that is not a Symbol spelling `@x`.
    InvalidInstanceVariableName,
    /// An ivar asked of a value that carries no INSTANCE STATE.
    InstanceStateError,
    /// An identity question asked of an identity-LESS value.
    ///
    /// `IRIS-V1-RUNTIME-C029` accepts only identity-bearing operands for
    /// `same?`, so a Text, a Symbol, a Tuple or a numeric raises rather than
    /// being compared by content - the question has no answer for them.
    IdentityError,
    /// A value with no specification-stable hash, per `IRIS-V1-COLLECTIONS-C087`.
    ///
    /// An Array or a Hash has no such hash, so asking for one is a KEY failure
    /// rather than an absent method - the selector exists on every value.
    InvalidKeyError,
    /// A run that consumed its STEP budget without terminating.
    ///
    /// A program may loop forever - `while true { }` with no reachable exit -
    /// and a machine with no bound would hang instead of answering. The
    /// reference charges a step per operation and fails the run when the
    /// budget is gone, so the machine does the same: a non-terminating program
    /// FAILS rather than never returning.
    StepBudgetExhausted,
    /// A meta transaction the runtime refuses to REACTIVATE.
    ///
    /// `Reflection::Class.reactivate` names a revision that is no longer the
    /// active one, and the runtime does not rewind a published spine, so the
    /// request is refused rather than performed.
    MetaTransactionError,
    /// A destructuring binding that the item does not MATCH.
    ///
    /// `IRIS-V1-CONTROL-C045` raises this when `for [a, b] in source` meets an
    /// item that is not an Array of exactly that arity.
    PatternMatchError,
    ReflectionAccess,
    JsonSyntaxError,
    /// A JSON object named one key TWICE.
    ///
    /// `IRIS-V1-LIBRARY-C014` makes rejecting a duplicate name the SAFE
    /// DEFAULT unless a caller selects last-wins, first-wins or collect-all.
    /// It is not a syntax failure: the text parses, and the object it
    /// describes is the thing being refused.
    JsonDuplicateNameError,
    /// A JSON document nested deeper than the caller ALLOWED.
    ///
    /// `IRIS-V1-LIBRARY-C013` makes a decode limit a REFUSAL before the
    /// offending container is allocated, not a truncation afterwards.
    JsonLimitError,
    /// A `<=>` body answered something that is neither an ordering nor nil.
    ///
    /// `IRIS-V1-RUNTIME-C092` makes an ordering an Integer and NO ORDER nil,
    /// so any other answer breaks the comparison contract itself rather than
    /// being an ordinary type failure.
    ComparisonContractError,
    /// A write was attempted against a READ-ONLY property.
    ///
    /// `D-159` makes an `ExceptionContext` member readable but never
    /// assignable, so a write names the readonly property rather than a
    /// missing setter the language never had.
    ReadonlyProperty,
    /// A `raise ... from` whose cause edge would form a CYCLE.
    ///
    /// `D-161` forbids a cycle among cause edges, and the check runs BEFORE
    /// linkage so a rejected attempt leaves the existing graph unchanged.
    ExceptionChainError,
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
        Instruction::IteratorNext { iterator, .. }
        | Instruction::IteratorClose { iterator, .. } => {
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
        Instruction::GateComplete { gate, value, .. } => vec![*gate, *value],
        Instruction::DefaultParameter { source, .. } => vec![*source],
        // The frame binds its own parameters from the argument window, which
        // the caller wrote before entry, so no register here is read.
        Instruction::BindParameters { .. } => Vec::new(),
        Instruction::IrisValueEncode { value, .. } => vec![*value],
        Instruction::FfiOpen { path, .. } => vec![*path],
        Instruction::EncodingDecode { value, .. } => vec![*value],
        Instruction::MakeRegex { pattern, .. } => vec![*pattern],
        Instruction::NativeFixture { .. } | Instruction::ApplyReopen { .. } => Vec::new(),
        Instruction::CheckReturn { value, .. } | Instruction::CheckAnnotation { value, .. } => {
            vec![*value]
        }
        Instruction::Print { .. } => Vec::new(),
        Instruction::EnterCleanup { context } => context.iter().copied().collect(),
        Instruction::DestructureElement { item, .. } => vec![*item],
        Instruction::EscapeRegex { value, .. } => vec![*value],
        Instruction::IrisValueDecode { stream, .. } => vec![*stream],
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
        | Instruction::LoadRegex { .. }
        | Instruction::GateNew { .. }
        | Instruction::UnicodeVersion { .. }
        | Instruction::DiscardedContexts { .. }
        | Instruction::PackageValidate { .. }
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
        Instruction::RaiseUnsupported { .. }
        | Instruction::RaiseNameError { .. }
        | Instruction::RaiseMessageNotFound { .. }
        | Instruction::RaiseTypeContract { .. }
        | Instruction::RaiseImmutableBinding { .. }
        | Instruction::RaiseVisibilityDenied { .. }
        | Instruction::RaiseArgumentError { .. }
        | Instruction::RaiseParseDiagnostic { .. }
        | Instruction::RaiseLoopTransfer { .. }
        | Instruction::RaiseEncodingSelection { .. } => Vec::new(),
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
