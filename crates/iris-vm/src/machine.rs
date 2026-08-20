//! Verifies and executes register instructions.

use iris_runtime::{
    ClassError, ClassId, ConstructionError, Kernel, KernelError, MethodBody, NativeSelector,
    NumericError, Runtime, Selector, StaticSpine, Value, Visibility,
};

use crate::compile::{FloatWidth, Instruction, Program, Register};

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
        match instruction {
            Instruction::JumpUnless { target, .. } | Instruction::Jump { target } => {
                if *target > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *target });
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
            Instruction::BuildArray { first, count, .. }
            | Instruction::New { first, count, .. }
            | Instruction::Send { first, count, .. }
            | Instruction::SendClass { first, count, .. } => {
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
        let mut next = state;
        if let Some(destination) = instruction.destination() {
            next[destination as usize] = true;
        }

        let successors: Vec<usize> = match instruction {
            // A return leaves the frame, so it has no successor at all.
            Instruction::Return { .. } => Vec::new(),
            Instruction::Jump { target } => vec![*target],
            // Both edges are live: the branch may be taken or not.
            Instruction::JumpUnless { target, .. } => vec![*target, at + 1],
            _ => vec![at + 1],
        };
        for successor in successors {
            if successor >= instructions.len() {
                continue;
            }
            let merged = match &entry[successor] {
                // Merging keeps only what is written on BOTH paths, which is
                // what makes a skipped write stop counting as written.
                Some(existing) => {
                    let merged: Vec<bool> = existing
                        .iter()
                        .zip(&next)
                        .map(|(held, incoming)| *held && *incoming)
                        .collect();
                    (merged != *existing).then_some(merged)
                }
                None => Some(next.clone()),
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
            Instruction::Jump { target } => *target >= instructions.len(),
            Instruction::JumpUnless { target, .. } => {
                *target >= instructions.len() || at + 1 >= instructions.len()
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
        Instruction::Binary { left, right, .. } | Instruction::Identity { left, right, .. } => {
            vec![*left, *right]
        }
        Instruction::Unary { operand, .. } => vec![*operand],
        Instruction::FromBits { bits, .. } => vec![*bits],
        Instruction::JumpUnless { condition, .. } => vec![*condition],
        Instruction::Return { value } => vec![*value],
        Instruction::BuildArray { first, count, .. }
        | Instruction::Call { first, count, .. }
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
        Instruction::Index {
            receiver, index, ..
        } => vec![*receiver, *index],
        Instruction::GetIvar { receiver, .. } => vec![*receiver],
        Instruction::SetIvar {
            receiver, value, ..
        } => vec![*receiver, *value],
        Instruction::LoadInteger { .. }
        | Instruction::LoadFloat64 { .. }
        | Instruction::LoadFloat32 { .. }
        | Instruction::LoadText { .. }
        | Instruction::LoadSymbol { .. }
        | Instruction::LoadBool { .. }
        | Instruction::LoadNil { .. }
        | Instruction::Jump { .. } => Vec::new(),
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
}

impl Machine {
    /// Builds a machine with a fresh runtime.
    ///
    /// # Errors
    /// Returns the kernel failure when the built-in classes cannot be defined.
    pub fn new() -> Result<Self, KernelError> {
        let mut runtime = Runtime::new();
        let kernel = Kernel::new(runtime.registry_mut())?;
        Ok(Self { runtime, kernel })
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
        while let Some(instruction) = instructions.get(counter) {
            counter += 1;
            let produced =
                match instruction {
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
                    Instruction::Move { source, .. } => registers[*source as usize].clone(),
                    Instruction::Binary {
                        selector,
                        left,
                        right,
                        ..
                    } => {
                        let left = registers[*left as usize].clone();
                        let right = registers[*right as usize].clone();
                        self.send(selector, left, &[right])?
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
                    Instruction::Index {
                        receiver, index, ..
                    } => self.index(
                        registers[*receiver as usize].clone(),
                        registers[*index as usize].clone(),
                    )?,
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
                        let returned = self.run_body(
                            &callee.instructions,
                            callee.registers,
                            arguments,
                            program,
                            classes,
                        )?;
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
                        if let Ok(method) =
                            self.runtime.dispatch_instance(object, Selector::INITIALIZE)
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
                            let _ = self.run_body(
                                &callee.instructions,
                                callee.registers,
                                passed,
                                program,
                                classes,
                            )?;
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
                        let object = match receiver {
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
                        let returned = self.run_body(
                            &callee.instructions,
                            callee.registers,
                            arguments,
                            program,
                            classes,
                        )?;
                        returned.into_iter().next().unwrap_or(Value::Nil)
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
                            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing {
                                selector,
                            } => {
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
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let returned = self.run_body(
                            &callee.instructions,
                            callee.registers,
                            arguments,
                            program,
                            classes,
                        )?;
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
            classes.push(class);
        }
        Ok(classes)
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
            return Err(MachineError::UnknownSelector(selector.to_owned()));
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
        })
        .chain(
            program
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter())
                .filter_map(|instruction| match instruction {
                    Instruction::Send { selector, .. }
                    | Instruction::SendClass { selector, .. }
                    | Instruction::GetIvar { name: selector, .. }
                    | Instruction::SetIvar { name: selector, .. } => Some(selector.as_str()),
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
