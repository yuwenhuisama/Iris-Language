//! Executes compiled instructions.

use iris_runtime::{ClassRegistry, Kernel, KernelError, NativeSelector, Value};

use crate::compile::{Instruction, Program};

/// Why execution stopped.
#[derive(Clone, Debug, PartialEq)]
pub enum MachineError {
    /// The program raised through the kernel.
    Kernel(KernelError),
    /// An instruction found too few operands.
    ///
    /// This is a COMPILER defect rather than a program error: the compiler is
    /// responsible for emitting balanced pushes and sends.
    StackUnderflow,
    /// The program left no single result.
    UnbalancedStack,
    /// A selector the runtime does not treat as native.
    UnknownSelector(String),
}

/// A stack machine over runtime values.
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

    /// Runs `program` and answers its single result.
    ///
    /// # Errors
    /// Returns the kernel failure, or a machine defect.
    pub fn execute(&self, program: &Program) -> Result<Value, MachineError> {
        let mut stack: Vec<Value> = Vec::new();
        for instruction in &program.instructions {
            match instruction {
                Instruction::PushInteger(digits) => {
                    let Ok(number) = digits.parse() else {
                        // The lexer produced this text, so a rejection here
                        // would mean the two disagree about integer syntax.
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    stack.push(Value::Integer(number));
                }
                Instruction::PushFloat64(bits) => {
                    stack.push(Value::Float64(f64::from_bits(*bits)));
                }
                Instruction::PushFloat32(bits) => {
                    stack.push(Value::Float32(f32::from_bits(*bits)));
                }
                Instruction::PushText(text) => stack.push(Value::Text(text.clone())),
                Instruction::Binary(selector) => {
                    let Some(right) = stack.pop() else {
                        return Err(MachineError::StackUnderflow);
                    };
                    let Some(left) = stack.pop() else {
                        return Err(MachineError::StackUnderflow);
                    };
                    stack.push(self.send(selector, left, &[right])?);
                }
                Instruction::Unary(selector) => {
                    let Some(operand) = stack.pop() else {
                        return Err(MachineError::StackUnderflow);
                    };
                    stack.push(self.send(selector, operand, &[])?);
                }
            }
        }
        let [result] = stack.as_slice() else {
            return Err(MachineError::UnbalancedStack);
        };
        Ok(result.clone())
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

/// Compiles and runs `source` on a fresh machine.
///
/// # Errors
/// Returns the kernel failure, or a machine defect.
pub fn run(program: &Program) -> Result<Value, MachineError> {
    let machine = Machine::new().map_err(MachineError::Kernel)?;
    machine.execute(program)
}
