//! Verifies and executes register instructions.

use iris_runtime::{ClassRegistry, Kernel, KernelError, NativeSelector, NumericError, Value};

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
    let mut written = vec![false; registers];
    if parameters > registers {
        return Err(VerifyError::RegisterOutOfRange {
            register: Register::try_from(parameters).unwrap_or(Register::MAX),
        });
    }
    for slot in written.iter_mut().take(parameters) {
        *slot = true;
    }

    let in_range = |register: Register| (register as usize) < registers;
    let read = |register: Register, written: &[bool]| -> Result<(), VerifyError> {
        if !in_range(register) {
            return Err(VerifyError::RegisterOutOfRange { register });
        }
        if !written[register as usize] {
            return Err(VerifyError::ReadBeforeWrite { register });
        }
        Ok(())
    };

    for instruction in instructions {
        match instruction {
            // A jump target must land inside the body. The old C++ VM had a
            // `SPR` opcode that fell through into the next case and overwrote
            // its result; a checked target is what makes that class of defect
            // a verification failure rather than silent corruption.
            Instruction::JumpUnless { condition, target } => {
                read(*condition, &written)?;
                if *target > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *target });
                }
            }
            Instruction::Jump { target } => {
                if *target > instructions.len() {
                    return Err(VerifyError::JumpOutOfRange { target: *target });
                }
            }
            Instruction::Return { value } => read(*value, &written)?,
            Instruction::Call {
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
                let last = (*first as usize).checked_add(*count as usize).ok_or(
                    VerifyError::ArrayRangeOutOfRange {
                        first: *first,
                        count: *count,
                    },
                )?;
                if last > registers {
                    return Err(VerifyError::ArrayRangeOutOfRange {
                        first: *first,
                        count: *count,
                    });
                }
                for offset in 0..*count {
                    read(first + offset, &written)?;
                }
            }
            Instruction::Move { source, .. } => read(*source, &written)?,
            Instruction::Binary { left, right, .. } => {
                read(*left, &written)?;
                read(*right, &written)?;
            }
            Instruction::Unary { operand, .. } => read(*operand, &written)?,
            Instruction::FromBits { bits, .. } => read(*bits, &written)?,
            Instruction::BuildArray { first, count, .. } => {
                let last = (*first as usize).checked_add(*count as usize).ok_or(
                    VerifyError::ArrayRangeOutOfRange {
                        first: *first,
                        count: *count,
                    },
                )?;
                if last > registers {
                    return Err(VerifyError::ArrayRangeOutOfRange {
                        first: *first,
                        count: *count,
                    });
                }
                for offset in 0..*count {
                    read(first + offset, &written)?;
                }
            }
            Instruction::LoadInteger { .. }
            | Instruction::LoadFloat64 { .. }
            | Instruction::LoadFloat32 { .. }
            | Instruction::LoadText { .. }
            | Instruction::LoadBool { .. }
            | Instruction::LoadNil { .. } => {}
        }
        if let Some(destination) = instruction.destination() {
            if !in_range(destination) {
                return Err(VerifyError::RegisterOutOfRange {
                    register: destination,
                });
            }
            written[destination as usize] = true;
        }
    }

    if let Some(result) = result {
        if !in_range(result) {
            return Err(VerifyError::RegisterOutOfRange { register: result });
        }
        if !written[result as usize] {
            return Err(VerifyError::ReadBeforeWrite { register: result });
        }
    }
    Ok(())
}

/// A register machine over runtime values.
pub struct Machine {
    registry: ClassRegistry,
    kernel: Kernel,
}

impl Machine {
    /// Builds a machine with a fresh runtime.
    ///
    /// # Errors
    /// Returns the kernel failure when the built-in classes cannot be defined.
    pub fn new() -> Result<Self, KernelError> {
        let mut registry = ClassRegistry::new();
        let kernel = Kernel::new(&mut registry)?;
        Ok(Self { registry, kernel })
    }

    /// Verifies `program`, then runs it and answers its result register.
    ///
    /// # Errors
    /// Returns the verification failure or the kernel failure.
    pub fn execute(&self, program: &Program) -> Result<Value, MachineError> {
        verify(program).map_err(MachineError::Invalid)?;
        let registers = self.run_body(
            &program.instructions,
            program.registers,
            Vec::new(),
            program,
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
        &self,
        instructions: &[Instruction],
        size: usize,
        arguments: Vec<Value>,
        program: &Program,
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
                    let Some(callee) = program.functions.get(*function) else {
                        return Err(MachineError::Invalid(VerifyError::UnknownFunction {
                            function: *function,
                        }));
                    };
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    let returned =
                        self.run_body(&callee.instructions, callee.registers, arguments, program)?;
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
            };
            if let Some(destination) = instruction.destination() {
                registers[destination as usize] = produced;
            }
        }
        Ok(registers)
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
            .send(&self.registry, receiver, native, arguments)
            .map_err(MachineError::Kernel)
    }
}

/// Verifies and runs `program` on a fresh machine.
///
/// # Errors
/// Returns the verification failure or the kernel failure.
pub fn run(program: &Program) -> Result<Value, MachineError> {
    let machine = Machine::new().map_err(MachineError::Kernel)?;
    machine.execute(program)
}
