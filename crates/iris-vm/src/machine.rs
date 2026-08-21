//! Verifies and executes register instructions.

use iris_runtime::{
    BoundReceiver, BuiltinClass, ClassError, ClassId, ConstructionError, ContractId, Kernel,
    KernelError, MethodBody, NativeSelector, NumericError, Runtime, Selector, StaticSpine, Value,
    Visibility,
};

use crate::compile::{FloatWidth, Instruction, Program, Register};

mod stdlib;

#[derive(Clone, Debug)]
struct ClosureRecord {
    function: usize,
    captures: Vec<Value>,
}

/// `IRIS-V1-CONTROL-C022` makes exactly `false` and `nil` falsey.
fn truthy(value: &Value) -> bool {
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
    DefiniteAssignment,
    /// A write to a position outside the Array.
    ///
    /// A READ past the end answers nil, but a WRITE has no position to store
    /// into, so it raises rather than silently discarding the value.
    IndexError,
    TypeContractError,
    MessageNotFound {
        receiver_class: String,
        selector: String,
    },
    /// An Iris value propagated beyond the current frame.
    Raised(Value),
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
        if let Instruction::DeclareDeferred { register } = instruction
            && *register as usize >= registers
        {
            return Err(VerifyError::RegisterOutOfRange {
                register: *register,
            });
        }
        match instruction {
            Instruction::JumpUnless { target, .. } | Instruction::Jump { target } => {
                if *target > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *target });
                }
            }
            Instruction::ArrayNext { exhausted, .. } => {
                if *exhausted > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *exhausted });
                }
            }
            Instruction::EnterTry {
                handler, cleanup, ..
            } => {
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
            Instruction::BuildArray { first, count, .. }
            | Instruction::New { first, count, .. }
            | Instruction::Send { first, count, .. }
            | Instruction::SendClass { first, count, .. }
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
        if let Instruction::DeclareDeferred { register } = instruction {
            next[*register as usize] = false;
        }

        let successors: Vec<(usize, Vec<bool>)> = match instruction {
            // A return leaves the frame, so it has no successor at all.
            Instruction::Return { .. } => Vec::new(),
            Instruction::Raise { .. } => Vec::new(),
            Instruction::Jump { target } => vec![(*target, next.clone())],
            // Both edges are live: the branch may be taken or not.
            Instruction::JumpUnless { target, .. } => {
                vec![(*target, next.clone()), (at + 1, next.clone())]
            }
            Instruction::ArrayNext { exhausted, .. } => {
                vec![(*exhausted, state.clone()), (at + 1, next.clone())]
            }
            Instruction::EnterTry {
                handler, exception, ..
            } => {
                let mut exceptional = state.clone();
                exceptional[*exception as usize] = true;
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
            Instruction::Raise { .. } => false,
            Instruction::Jump { target } => *target >= instructions.len(),
            Instruction::JumpUnless { target, .. } => {
                *target >= instructions.len() || at + 1 >= instructions.len()
            }
            Instruction::ArrayNext { exhausted, .. } => {
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
        Instruction::Move { source, .. } => vec![*source],
        Instruction::StoreGlobal { value, .. } => vec![*value],
        Instruction::Binary { left, right, .. } | Instruction::Identity { left, right, .. } => {
            vec![*left, *right]
        }
        Instruction::Unary { operand, .. } => vec![*operand],
        Instruction::FromBits { bits, .. } => vec![*bits],
        Instruction::JumpUnless { condition, .. } => vec![*condition],
        Instruction::Return { value } | Instruction::Raise { value } => vec![*value],
        Instruction::BuildArray { first, count, .. }
        | Instruction::Call { first, count, .. }
        | Instruction::MakeClosure { first, count, .. }
        | Instruction::New { first, count, .. } => {
            (0..*count).map(|offset| first + offset).collect()
        }
        Instruction::BuildHash { first, count, .. } => (0..count.saturating_mul(2))
            .map(|offset| first + offset)
            .collect(),
        Instruction::Send {
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
        Instruction::ArrayNext { array, index, .. } => vec![*array, *index],
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
        Instruction::LoadInteger { .. }
        | Instruction::LoadFloat64 { .. }
        | Instruction::LoadFloat32 { .. }
        | Instruction::LoadText { .. }
        | Instruction::LoadSymbol { .. }
        | Instruction::LoadBool { .. }
        | Instruction::LoadNil { .. }
        | Instruction::LoadClass { .. }
        | Instruction::LoadContract { .. }
        | Instruction::LoadGlobal { .. }
        | Instruction::EnterTry { .. }
        | Instruction::LeaveTry
        | Instruction::Jump { .. }
        | Instruction::DeclareDeferred { .. } => Vec::new(),
        Instruction::RaiseDefiniteAssignment { .. } => Vec::new(),
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

/// A register machine over runtime values.
pub struct Machine {
    /// Owns the class registry, the heap and the ivar tables together.
    ///
    /// A bare registry could describe classes but not INSTANTIATE them: there
    /// was nowhere to put an object, so the backend could never produce a
    /// `Value::Object` and had nothing for a collector to walk.
    runtime: Runtime,
    kernel: Kernel,
    closures: std::collections::HashMap<iris_runtime::ObjectId, ClosureRecord>,
    globals: std::collections::HashMap<String, Value>,
    next_closure: u64,
}

impl Machine {
    /// Builds a machine with a fresh runtime.
    ///
    /// # Errors
    /// Returns the kernel failure when the built-in classes cannot be defined.
    pub fn new() -> Result<Self, KernelError> {
        let mut runtime = Runtime::new();
        let kernel = Kernel::new(runtime.registry_mut())?;
        Ok(Self {
            runtime,
            kernel,
            closures: std::collections::HashMap::new(),
            globals: std::collections::HashMap::new(),
            next_closure: 1,
        })
    }

    /// Verifies `program`, then runs it and answers its result register.
    ///
    /// # Errors
    /// Returns the verification failure or the kernel failure.
    pub fn execute(&mut self, program: &Program) -> Result<Value, MachineError> {
        verify(program).map_err(MachineError::Invalid)?;
        let classes = self.register_classes(program)?;
        let registers = self.run_body(
            &program.instructions,
            program.registers,
            Vec::new(),
            program,
            &classes,
        )?;
        Ok(registers
            .get(program.result as usize)
            .cloned()
            .unwrap_or(Value::Nil))
    }

    /// Runs ONE frame to completion, answering its register file.
    ///
    /// Each call gets a fresh file, so nothing is shared with the caller: a
    /// callee cannot read a caller's registers, and recursion needs no
    /// save/restore of individual registers. The frame is also the unit a
    /// future collector would walk, since the live frames ARE the root set.
    fn run_body(
        &mut self,
        instructions: &[Instruction],
        size: usize,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Vec<Value>, MachineError> {
        // Verification proved every read is in range and written, so indexing
        // below cannot be out of bounds and no operand check is repeated.
        let mut registers = vec![Value::Nil; size];
        // Parameters arrive pre-bound in the leading registers.
        for (slot, argument) in arguments.into_iter().enumerate() {
            registers[slot] = argument;
        }

        let mut counter = 0;
        let mut handlers: Vec<(usize, Register)> = Vec::new();
        macro_rules! run_frame {
            ($label:lifetime, $call:expr) => {
                match $call {
                    Ok(values) => values,
                    Err(MachineError::Raised(value)) => {
                        let Some((handler, exception)) = handlers.pop() else {
                            return Err(MachineError::Raised(value));
                        };
                        registers[exception as usize] = value;
                        counter = handler;
                        continue $label;
                    }
                    Err(error) => return Err(error),
                }
            };
        }
        'frame: while let Some(instruction) = instructions.get(counter) {
            counter += 1;
            let produced = match instruction {
                Instruction::LoadInteger { digits, .. } => {
                    let Ok(number) = digits.parse() else {
                        // The lexer produced this text, so a rejection would
                        // mean the two disagree about integer syntax.
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::Integer(number)
                }
                Instruction::LoadFloat64 { bits, .. } => Value::Float64(f64::from_bits(*bits)),
                Instruction::LoadFloat32 { bits, .. } => Value::Float32(f32::from_bits(*bits)),
                Instruction::LoadText { text, .. } => Value::Text(text.clone()),
                Instruction::LoadSymbol { name, .. } => Value::Symbol(name.clone()),
                Instruction::LoadBool { value, .. } => Value::Bool(*value),
                Instruction::LoadNil { .. } => Value::Nil,
                Instruction::LoadClass { class, .. } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    Value::Class(class)
                }
                Instruction::LoadContract { contract, .. } => {
                    Value::Contract(ContractId::new(*contract as u64 + 1))
                }
                Instruction::LoadGlobal { name, .. } => self
                    .globals
                    .get(name)
                    .cloned()
                    .ok_or(MachineError::NameError)?,
                Instruction::StoreGlobal { name, value, .. } => {
                    let value = registers[*value as usize].clone();
                    self.globals.insert(name.clone(), value.clone());
                    value
                }
                Instruction::Move { source, .. } => registers[*source as usize].clone(),
                Instruction::DeclareDeferred { .. } => continue,
                Instruction::RaiseDefiniteAssignment { .. } => {
                    return Err(MachineError::DefiniteAssignment);
                }
                Instruction::Binary {
                    selector,
                    left,
                    right,
                    ..
                } => {
                    let left = registers[*left as usize].clone();
                    let right = registers[*right as usize].clone();
                    run_frame!(
                        'frame,
                        self.binary_send(selector, left, right, program, classes)
                    )
                }
                Instruction::Unary {
                    selector, operand, ..
                } => {
                    let operand = registers[*operand as usize].clone();
                    self.send(selector, operand, &[])?
                }
                Instruction::BuildArray { first, count, .. } => {
                    let start = *first as usize;
                    let elements = registers[start..start + *count as usize].to_vec();
                    Value::Array(iris_runtime::ArrayRef::new(elements))
                }
                Instruction::BuildHash { first, count, .. } => {
                    let start = *first as usize;
                    let mut entries = Vec::with_capacity(*count as usize);
                    for pair in registers[start..start + *count as usize * 2].chunks_exact(2) {
                        iris_runtime::public_hash(&pair[0])
                            .map_err(KernelError::StableHash)
                            .map_err(MachineError::Kernel)?;
                        if let Some((_, value)) =
                            entries.iter_mut().find(|(key, _)| *key == pair[0])
                        {
                            *value = pair[1].clone();
                        } else {
                            entries.push((pair[0].clone(), pair[1].clone()));
                        }
                    }
                    Value::Hash(iris_runtime::HashRef::new(entries))
                }
                Instruction::MakeClosure {
                    function,
                    first,
                    count,
                    ..
                } => {
                    let start = *first as usize;
                    let identity = iris_runtime::ObjectId::new(self.next_closure);
                    self.next_closure = self.next_closure.saturating_add(1);
                    self.closures.insert(
                        identity,
                        ClosureRecord {
                            function: *function,
                            captures: registers[start..start + *count as usize].to_vec(),
                        },
                    );
                    Value::Closure(identity)
                }
                Instruction::Index {
                    receiver, index, ..
                } => self.index(
                    registers[*receiver as usize].clone(),
                    registers[*index as usize].clone(),
                )?,
                Instruction::SetIndex {
                    receiver,
                    index,
                    value,
                    ..
                } => self.set_index(
                    registers[*receiver as usize].clone(),
                    registers[*index as usize].clone(),
                    registers[*value as usize].clone(),
                )?,
                Instruction::BindMember {
                    receiver, selector, ..
                } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::UnknownSelector(selector.clone()));
                    };
                    let bound_selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let class = self
                        .runtime
                        .class_of(object)
                        .map_err(MachineError::Construction)?;
                    let class_index = classes
                        .iter()
                        .position(|known| *known == class)
                        .ok_or(MachineError::Class(ClassError::ClassIdentityExhausted))?;
                    if program.classes[class_index]
                        .stored_properties
                        .iter()
                        .any(|property| property.name == *selector)
                    {
                        let selector = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        self.runtime
                            .raw_ivar(object, selector)
                            .map_err(MachineError::Construction)?
                    } else if program.classes[class_index]
                        .property_methods
                        .iter()
                        .any(|property| property == selector)
                    {
                        let selector_id = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        let method = self
                            .runtime
                            .dispatch_instance(object, selector_id)
                            .map_err(MachineError::Construction)?;
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let returned = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            vec![Value::Object(object)],
                            program,
                            classes,
                        ));
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    } else {
                        self.runtime
                            .registry_mut()
                            .bind_instance(object, class, bound_selector)
                            .map(Value::BoundMethod)
                            .map_err(iris_runtime::ConstructionError::from)
                            .map_err(MachineError::Construction)?
                    }
                }
                Instruction::Identity { left, right, .. } => {
                    self.identity(&registers[*left as usize], &registers[*right as usize])?
                }
                // C113 fixes the accepted range per WIDTH and requires
                // RangeError outside it; C114 requires the round trip to hold
                // for every pattern including signaling NaN, so the bits are
                // reinterpreted rather than converted numerically.
                Instruction::FromBits { width, bits, .. } => {
                    let Value::Integer(bits) = &registers[*bits as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    // The runtime spells an out-of-range width `Numeric(Range)`
                    // and the reference answers exactly that, so inventing a
                    // separate error would make the backends disagree for a
                    // reason that is not semantic.
                    let Some(bits) = bits.to_u64() else {
                        return Err(MachineError::Kernel(KernelError::Numeric(
                            NumericError::Range,
                        )));
                    };
                    match width {
                        FloatWidth::Bits32 => {
                            let Ok(bits) = u32::try_from(bits) else {
                                return Err(MachineError::Kernel(KernelError::Numeric(
                                    NumericError::Range,
                                )));
                            };
                            Value::Float32(f32::from_bits(bits))
                        }
                        FloatWidth::Bits64 => Value::Float64(f64::from_bits(bits)),
                    }
                }
                // Truth is decided by the RUNTIME rather than re-derived here.
                // `IRIS-V1-CONTROL-C022` makes only `false` and `nil` falsey,
                // and a second copy of that rule would be one more place for
                // the backends to diverge.
                Instruction::JumpUnless { condition, target } => {
                    if !truthy(&registers[*condition as usize]) {
                        counter = *target;
                    }
                    continue;
                }
                Instruction::Jump { target } => {
                    counter = *target;
                    continue;
                }
                Instruction::ArrayNext {
                    array,
                    index,
                    exhausted,
                    ..
                } => {
                    let Value::Array(array) = &registers[*array as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Integer(index) = &registers[*index as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Some(index) = index.to_usize() else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let elements = array.elements();
                    let Some(value) = elements.get(index).cloned() else {
                        counter = *exhausted;
                        continue;
                    };
                    value
                }
                Instruction::EnterTry {
                    handler, exception, ..
                } => {
                    handlers.push((*handler, *exception));
                    continue;
                }
                Instruction::CatchMatch {
                    exception, class, ..
                } => Value::Bool(self.catch_matches(
                    &registers[*exception as usize],
                    class,
                    program,
                    classes,
                )?),
                Instruction::LeaveTry => {
                    handlers.pop();
                    continue;
                }
                Instruction::Raise { value } => {
                    let value = registers[*value as usize].clone();
                    let Some((handler, exception)) = handlers.pop() else {
                        return Err(MachineError::Raised(value));
                    };
                    registers[exception as usize] = value;
                    counter = handler;
                    continue;
                }
                Instruction::Return { value } => {
                    let value = registers[*value as usize].clone();
                    // The answer is handed back in the frame's own result
                    // slot, so a caller reads it without knowing the callee's
                    // register layout.
                    return Ok(vec![value]);
                }
                Instruction::Call {
                    function,
                    first,
                    count,
                    ..
                } => {
                    let Some(callee) = program.functions.get(*function).cloned() else {
                        return Err(MachineError::Invalid(VerifyError::UnknownFunction {
                            function: *function,
                        }));
                    };
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::New {
                    class,
                    first,
                    count,
                    ..
                } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    let object = self
                        .runtime
                        .allocate(class)
                        .map_err(MachineError::Construction)?;
                    self.initialize_properties(program, classes, class, object)?;
                    if let Ok(method) = self.runtime.dispatch_instance(object, Selector::INITIALIZE)
                    {
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let mut passed = Vec::with_capacity(arguments.len() + 1);
                        passed.push(Value::Object(object));
                        passed.extend(arguments);
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let _ = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            passed,
                            program,
                            classes,
                        ));
                    }
                    Value::Object(object)
                }
                Instruction::Send {
                    receiver,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let receiver = registers[*receiver as usize].clone();
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    if let Some(value) = run_frame!(
                        'frame,
                        self.authored_send(
                            &receiver,
                            selector,
                            &arguments,
                            program,
                            classes,
                        )
                    ) {
                        if let Some(destination) = instruction.destination() {
                            registers[destination as usize] = value;
                        }
                        continue;
                    }
                    if selector == "call" {
                        let (callee, passed) = match receiver {
                            Value::Closure(identity) => {
                                let Some(closure) = self.closures.get(&identity).cloned() else {
                                    return Err(MachineError::Kernel(KernelError::Type));
                                };
                                let Some(callee) = program.functions.get(closure.function).cloned()
                                else {
                                    return Err(MachineError::Invalid(
                                        VerifyError::UnknownFunction {
                                            function: closure.function,
                                        },
                                    ));
                                };
                                let mut passed = closure.captures;
                                passed.extend(arguments);
                                (callee, passed)
                            }
                            Value::BoundMethod(bound) => {
                                let (class, receiver) = match bound.receiver() {
                                    BoundReceiver::Object(object) => (
                                        self.runtime
                                            .class_of(object)
                                            .map_err(MachineError::Construction)?,
                                        Value::Object(object),
                                    ),
                                    BoundReceiver::Class(class) => (class, Value::Class(class)),
                                };
                                self.runtime
                                    .registry()
                                    .validate_method_binding(class, bound.method())
                                    .map_err(ConstructionError::from)
                                    .map_err(MachineError::Construction)?;
                                let function = usize::try_from(bound.method().body().raw())
                                    .map_err(|_| {
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function: usize::MAX,
                                        })
                                    })?;
                                let Some(callee) = program.functions.get(function).cloned() else {
                                    return Err(MachineError::Invalid(
                                        VerifyError::UnknownFunction { function },
                                    ));
                                };
                                let mut passed = Vec::with_capacity(arguments.len() + 1);
                                passed.push(receiver);
                                passed.extend(arguments);
                                (callee, passed)
                            }
                            receiver => {
                                let value = self.send(selector, receiver, &arguments)?;
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                        };
                        if passed.len() != callee.parameters {
                            return Err(MachineError::Kernel(KernelError::Arity));
                        }
                        let returned = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            passed,
                            program,
                            classes,
                        ));
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    } else {
                        let object = match receiver {
                            Value::Class(class) if selector == "new" => {
                                let object = self
                                    .runtime
                                    .allocate(class)
                                    .map_err(MachineError::Construction)?;
                                self.initialize_properties(program, classes, class, object)?;
                                if let Ok(method) =
                                    self.runtime.dispatch_instance(object, Selector::INITIALIZE)
                                {
                                    let function =
                                        usize::try_from(method.body().raw()).map_err(|_| {
                                            MachineError::Invalid(VerifyError::UnknownFunction {
                                                function: usize::MAX,
                                            })
                                        })?;
                                    let mut passed = Vec::with_capacity(arguments.len() + 1);
                                    passed.push(Value::Object(object));
                                    passed.extend(arguments);
                                    let callee = program.functions.get(function).cloned().ok_or(
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function,
                                        }),
                                    )?;
                                    let _ = run_frame!('frame, self.run_body(
                                        &callee.instructions,
                                        callee.registers,
                                        passed,
                                        program,
                                        classes,
                                    ));
                                }
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = Value::Object(object);
                                }
                                continue;
                            }
                            Value::Class(class) => {
                                let selector_id =
                                    selector_id(program, selector).ok_or_else(|| {
                                        MachineError::UnknownSelector(selector.clone())
                                    })?;
                                let method = match self
                                    .runtime
                                    .registry()
                                    .dispatch_class_object(class, selector_id)
                                    .map_err(iris_runtime::ConstructionError::from)
                                    .map_err(MachineError::Construction)?
                                {
                                    iris_runtime::DispatchOutcome::Invoke(method) => method,
                                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing {
                                        selector,
                                    } => {
                                        return Err(MachineError::Construction(
                                            iris_runtime::DispatchError::MissingMethod { selector }
                                                .into(),
                                        ));
                                    }
                                };
                                let function =
                                    usize::try_from(method.body().raw()).map_err(|_| {
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function: usize::MAX,
                                        })
                                    })?;
                                let mut passed = Vec::with_capacity(arguments.len() + 1);
                                passed.push(Value::Class(class));
                                passed.extend(arguments);
                                let callee = program.functions.get(function).cloned().ok_or(
                                    MachineError::Invalid(VerifyError::UnknownFunction {
                                        function,
                                    }),
                                )?;
                                let returned = run_frame!('frame, self.run_body(
                                    &callee.instructions,
                                    callee.registers,
                                    passed,
                                    program,
                                    classes,
                                ));
                                let value = returned.into_iter().next().unwrap_or(Value::Nil);
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                            Value::Object(object) => object,
                            receiver => {
                                let value = self.send(selector, receiver, &arguments)?;
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                        };
                        let selector = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        let method = self
                            .runtime
                            .dispatch_instance(object, selector)
                            .map_err(MachineError::Construction)?;
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let mut arguments = Vec::with_capacity(*count as usize + 1);
                        arguments.push(Value::Object(object));
                        arguments.extend_from_slice(&registers[start..start + *count as usize]);
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let returned = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            arguments,
                            program,
                            classes,
                        ));
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    }
                }
                Instruction::SendClass {
                    class,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let method = match self
                        .runtime
                        .registry()
                        .dispatch_class_object(class, selector)
                        .map_err(iris_runtime::ConstructionError::from)
                        .map_err(MachineError::Construction)?
                    {
                        iris_runtime::DispatchOutcome::Invoke(method) => method,
                        iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                            return Err(MachineError::Construction(
                                iris_runtime::DispatchError::MissingMethod { selector }.into(),
                            ));
                        }
                    };
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let start = *first as usize;
                    let mut arguments = Vec::with_capacity(*count as usize + 1);
                    arguments.push(Value::Class(class));
                    arguments.extend_from_slice(&registers[start..start + *count as usize]);
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::ContractCast {
                    receiver, contract, ..
                } => {
                    let receiver = registers[*receiver as usize].clone();
                    let Value::Object(object) = receiver else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let class = self
                        .runtime
                        .class_of(object)
                        .map_err(MachineError::Construction)?;
                    let conforms = classes
                        .iter()
                        .position(|known| *known == class)
                        .is_some_and(|index| program.classes[index].contracts.contains(contract));
                    if !conforms {
                        return Err(MachineError::Kernel(KernelError::Type));
                    }
                    Value::ContractView(
                        Box::new(Value::Object(object)),
                        ContractId::new(*contract as u64 + 1),
                    )
                }
                Instruction::SendContract {
                    receiver,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let Value::ContractView(receiver, contract) =
                        registers[*receiver as usize].clone()
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Object(object) = *receiver else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let contract_index = usize::try_from(contract.raw().saturating_sub(1))
                        .map_err(|_| MachineError::Kernel(KernelError::Type))?;
                    let required = program
                        .contracts
                        .get(contract_index)
                        .is_some_and(|contract| {
                            contract
                                .requirements
                                .iter()
                                .any(|(name, arity)| name == selector && *arity == *count as usize)
                        });
                    if !required {
                        return Err(MachineError::MessageNotFound {
                            receiver_class: "ContractView".to_owned(),
                            selector: selector.clone(),
                        });
                    }
                    let selector_id = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let method = self
                        .runtime
                        .dispatch_instance(object, selector_id)
                        .map_err(MachineError::Construction)?;
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
                    let start = *first as usize;
                    let mut arguments = Vec::with_capacity(*count as usize + 1);
                    arguments.push(Value::Object(object));
                    arguments.extend_from_slice(&registers[start..start + *count as usize]);
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::GetIvar { receiver, name, .. } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .raw_ivar(object, selector)
                        .map_err(MachineError::Construction)?
                }
                Instruction::SetIvar {
                    receiver,
                    name,
                    value,
                    ..
                } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .assign_raw_ivar(object, selector, registers[*value as usize].clone())
                        .map_err(MachineError::Construction)?
                }
                Instruction::GetClassVar { receiver, name, .. } => {
                    let class = self.receiver_class(&registers[*receiver as usize])?;
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .class_var(class, selector)
                        .map_err(MachineError::Construction)?
                        .ok_or(MachineError::NameError)?
                }
                Instruction::SetClassVar {
                    receiver,
                    name,
                    value,
                    ..
                } => {
                    let class = self.receiver_class(&registers[*receiver as usize])?;
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .assign_class_var(class, selector, registers[*value as usize].clone())
                        .map_err(MachineError::Construction)?
                }
            };
            if let Some(destination) = instruction.destination() {
                registers[destination as usize] = produced;
            }
        }
        Ok(registers)
    }

    fn register_classes(&mut self, program: &Program) -> Result<Vec<ClassId>, MachineError> {
        let mut classes = Vec::with_capacity(program.classes.len());
        for (index, declaration) in program.classes.iter().enumerate() {
            let superclass = declaration
                .superclass
                .and_then(|parent| classes.get(parent).copied());
            let class = self
                .runtime
                .registry_mut()
                .define_class(StaticSpine::new(index as u64 + 1), superclass)
                .map_err(MachineError::Class)?;
            self.runtime
                .registry_mut()
                .begin_origin_transaction(class)
                .map_err(MachineError::Class)?;
            for (selector, function) in &declaration.methods {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                self.runtime
                    .registry_mut()
                    .publish_origin_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
            }
            for variable in &declaration.class_variables {
                let selector = selector_id(program, &variable.name)
                    .ok_or_else(|| MachineError::UnknownSelector(variable.name.clone()))?;
                let value = literal_runtime_value(&variable.initializer)?;
                self.runtime
                    .declare_class_var(class, selector, value, variable.mutable)
                    .map_err(MachineError::Construction)?;
            }
            for (selector, function) in &declaration.class_methods {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                self.runtime
                    .registry_mut()
                    .publish_singleton_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
            }
            self.runtime
                .registry_mut()
                .commit_origin_transaction(class)
                .map_err(MachineError::Class)?;
            for reopen in &declaration.reopens {
                self.runtime
                    .registry_mut()
                    .begin_transaction(class)
                    .map_err(MachineError::Class)?;
                for (selector, function) in &reopen.methods {
                    let selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    self.runtime
                        .registry_mut()
                        .publish_method(
                            class,
                            selector,
                            MethodBody::new(*function as u64),
                            Visibility::Public,
                        )
                        .map_err(MachineError::Class)?;
                }
                self.runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(MachineError::Class)?;
            }
            classes.push(class);
        }
        Ok(classes)
    }

    fn catch_matches(
        &self,
        value: &Value,
        name: &str,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<bool, MachineError> {
        let filter = match name {
            "Symbol" => return Ok(matches!(value, Value::Symbol(_))),
            "Integer" => return Ok(matches!(value, Value::Integer(_))),
            "Nil" => return Ok(matches!(value, Value::Nil)),
            "Bool" => return Ok(matches!(value, Value::Bool(_))),
            "Object" => self
                .kernel
                .class(BuiltinClass::Object)
                .map_err(MachineError::Kernel),
            _ => program
                .classes
                .iter()
                .position(|class| class.name == name)
                .and_then(|index| classes.get(index).copied())
                .map_or_else(|| Err(MachineError::Kernel(KernelError::Type)), Ok),
        }?;
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let mut class = self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?;
        loop {
            if class == filter {
                return Ok(true);
            }
            let Some(superclass) = self
                .runtime
                .registry()
                .active(class)
                .map_err(MachineError::Class)?
                .runtime_superclass()
            else {
                return Ok(false);
            };
            class = superclass;
        }
    }

    fn receiver_class(&self, receiver: &Value) -> Result<ClassId, MachineError> {
        match receiver {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction),
            Value::Class(class) => Ok(*class),
            _ => Err(MachineError::Kernel(KernelError::Type)),
        }
    }

    fn initialize_properties(
        &mut self,
        program: &Program,
        classes: &[ClassId],
        class: ClassId,
        object: iris_runtime::ObjectId,
    ) -> Result<(), MachineError> {
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
        };
        for property in &program.classes[index].stored_properties {
            let selector = selector_id(program, &property.name)
                .ok_or_else(|| MachineError::UnknownSelector(property.name.clone()))?;
            let value = literal_runtime_value(&property.initializer)?;
            self.runtime
                .assign_raw_ivar(object, selector, value)
                .map_err(MachineError::Construction)?;
        }
        Ok(())
    }

    /// Sends a native selector through the SHARED kernel.
    ///
    /// Arithmetic is not reimplemented here. A second bigint or IEEE-754
    /// implementation is exactly the silent divergence RUNTIME-V052 and
    /// RUNTIME-V066 exist to detect, and two copies of one bug would agree.
    fn send(
        &self,
        selector: &str,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        let Some(native) = NativeSelector::from_source(selector) else {
            return Err(MachineError::MessageNotFound {
                receiver_class: value_class_name(&receiver).to_owned(),
                selector: selector.to_owned(),
            });
        };
        self.kernel
            .send(self.runtime.registry(), receiver, native, arguments)
            .map_err(MachineError::Kernel)
    }

    fn identity(&self, left: &Value, right: &Value) -> Result<Value, MachineError> {
        let same = match (left, right) {
            (Value::Nil, Value::Nil)
            | (Value::Bool(false), Value::Bool(false))
            | (Value::Bool(true), Value::Bool(true)) => true,
            (Value::Nil, _) | (Value::Bool(_), _) | (_, Value::Nil) | (_, Value::Bool(_)) => false,
            (Value::Object(left), Value::Object(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,
            (Value::Array(left), Value::Array(right)) => left.same(right),
            (Value::Hash(left), Value::Hash(right)) => left.same(right),
            _ => return Err(MachineError::Kernel(KernelError::Identity)),
        };
        Ok(Value::Bool(same))
    }

    fn index(&self, receiver: Value, index: Value) -> Result<Value, MachineError> {
        match receiver {
            Value::Array(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                let elements = values.elements();
                Ok(resolve_index(&index, elements.len())
                    .and_then(|index| elements.get(index).cloned())
                    .unwrap_or(Value::Nil))
            }
            Value::Hash(entries) => Ok(entries.get(&index).unwrap_or(Value::Nil)),
            _ => Err(MachineError::UnknownSelector("[]".to_owned())),
        }
    }

    fn set_index(
        &self,
        receiver: Value,
        index: Value,
        value: Value,
    ) -> Result<Value, MachineError> {
        match receiver {
            Value::Array(values) => {
                let Value::Integer(index) = index else {
                    return Err(MachineError::Kernel(KernelError::Type));
                };
                // The reference answers nil for an out-of-range READ and
                // raises for an out-of-range WRITE, because a write has no
                // position to store into. Discarding it silently would leave
                // the program believing the element was stored.
                let Some(index) = resolve_index(&index, values.len()) else {
                    return Err(MachineError::IndexError);
                };
                let mut stored = false;
                values.mutate(|elements| {
                    if let Some(slot) = elements.get_mut(index) {
                        *slot = value.clone();
                        stored = true;
                    }
                });
                if stored {
                    Ok(value)
                } else {
                    Err(MachineError::IndexError)
                }
            }
            Value::Hash(entries) => {
                iris_runtime::public_hash(&index)
                    .map_err(KernelError::StableHash)
                    .map_err(MachineError::Kernel)?;
                entries.insert(index, value.clone());
                Ok(value)
            }
            _ => Err(MachineError::UnknownSelector("[]=".to_owned())),
        }
    }
}

fn value_class_name(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "Array",
        Value::Hash(_) => "Hash",
        Value::Text(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Float32(_) => "Float32",
        Value::Float64(_) => "Float64",
        Value::Bool(_) => "Bool",
        Value::Nil => "Nil",
        Value::Symbol(_) => "Symbol",
        Value::Class(_) => "Class",
        Value::Object(_) => "Object",
        Value::Closure(_) => "Closure",
        Value::BoundMethod(_) => "BoundMethod",
        _ => "Object",
    }
}

fn literal_runtime_value(value: &crate::compile::LiteralValue) -> Result<Value, MachineError> {
    match value {
        crate::compile::LiteralValue::Integer(digits) => digits
            .parse()
            .map(Value::Integer)
            .map_err(|_| MachineError::Kernel(KernelError::Type)),
        crate::compile::LiteralValue::Text(text) => Ok(Value::Text(text.clone())),
        crate::compile::LiteralValue::Bool(value) => Ok(Value::Bool(*value)),
        crate::compile::LiteralValue::Nil => Ok(Value::Nil),
    }
}

fn resolve_index(index: &iris_runtime::IntegerValue, length: usize) -> Option<usize> {
    let index = index.to_i128()?;
    let length = i128::try_from(length).ok()?;
    let resolved = if index < 0 {
        length.checked_add(index)?
    } else {
        index
    };
    usize::try_from(resolved)
        .ok()
        .filter(|index| *index < length as usize)
}

/// Verifies and runs `program` on a fresh machine.
///
/// # Errors
/// Returns the verification failure or the kernel failure.
pub fn run(program: &Program) -> Result<Value, MachineError> {
    let mut machine = Machine::new().map_err(MachineError::Kernel)?;
    machine.execute(program)
}

fn selector_id(program: &Program, name: &str) -> Option<Selector> {
    if name == "initialize" {
        return Some(Selector::INITIALIZE);
    }
    if let Some(native) = NativeSelector::from_source(name) {
        return Some(native.id());
    }
    let mut names = program
        .classes
        .iter()
        .flat_map(|class| {
            class
                .methods
                .iter()
                .chain(&class.class_methods)
                .map(|(name, _)| name.as_str())
                .chain(
                    class
                        .class_variables
                        .iter()
                        .map(|variable| variable.name.as_str()),
                )
                .chain(
                    class
                        .stored_properties
                        .iter()
                        .map(|property| property.name.as_str()),
                )
        })
        .chain(
            program
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter())
                .filter_map(|instruction| match instruction {
                    Instruction::Send { selector, .. }
                    | Instruction::SendClass { selector, .. }
                    | Instruction::SendContract { selector, .. }
                    | Instruction::BindMember { selector, .. }
                    | Instruction::GetIvar { name: selector, .. }
                    | Instruction::SetIvar { name: selector, .. } => Some(selector.as_str()),
                    Instruction::GetClassVar { name, .. }
                    | Instruction::SetClassVar { name, .. } => Some(name.as_str()),
                    _ => None,
                }),
        )
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
        .iter()
        .position(|candidate| *candidate == name)
        .and_then(|index| u64::try_from(index).ok())
        .and_then(|index| index.checked_add(10_000))
        .map(Selector::new)
}
