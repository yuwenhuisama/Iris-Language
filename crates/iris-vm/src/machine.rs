//! Verifies and executes register instructions.

use iris_runtime::{Kernel, KernelError, NativeSelector, Runtime, Selector, Value};

use crate::compile::{Instruction, Program};

mod execute;
mod operations;
mod runtime;
mod stdlib;

#[derive(Clone, Debug)]
struct ClosureRecord {
    function: usize,
    captures: Vec<Value>,
}

mod verify;

pub(super) use verify::truthy;
pub use verify::{MachineError, VerifyError, verify};

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
    next_context: u64,
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
            next_context: 900_000,
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
}

pub(super) fn value_class_name(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "Array",
        Value::Tuple(_) => "Tuple",
        Value::Range(_) => "Range",
        Value::Hash(_) => "Hash",
        Value::Text(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Float32(_) => "Float32",
        Value::Float64(_) => "Float64",
        Value::Bool(_) => "Bool",
        Value::Nil => "Nil",
        Value::Symbol(_) => "Symbol",
        Value::Class(_) => "Class",
        Value::Type(..) => "Type",
        Value::Object(_) => "Object",
        Value::Closure(_) => "Closure",
        Value::BoundMethod(_) => "BoundMethod",
        Value::ExceptionContext(..) => "ExceptionContext",
        // Every family the reference names must be named the SAME way here.
        // A refusal reporting `Object` where the reference reports
        // `ReadonlyArray` is still a DISAGREEMENT: both backends reject the
        // program, and they describe the rejection differently.
        Value::ReadonlyArray(_) => "ReadonlyArray",
        Value::ContractView(..) => "ContractView",
        Value::Contract(_) => "Contract",
        Value::SourceLocation(..) => "SourceLocation",
        Value::StackFrame(..) => "StackFrame",
        Value::RaiseSite(_) => "RaiseSite",
        Value::Method(_) => "Method",
        Value::MutableString(_) => "MutableString",
        Value::Bytes(_) => "Bytes",
        Value::ByteArray(_) => "ByteArray",
        Value::Regex(_) => "Regex",
        Value::Match(_) => "Match",
        Value::Gate(_) => "Gate",
        Value::Task(..) => "Task",
        Value::ComposedType(_) => "Type",
        _ => "Object",
    }
}

pub(super) fn literal_runtime_value(
    value: &crate::compile::LiteralValue,
) -> Result<Value, MachineError> {
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

pub(super) fn resolve_index(index: &iris_runtime::IntegerValue, length: usize) -> Option<usize> {
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
