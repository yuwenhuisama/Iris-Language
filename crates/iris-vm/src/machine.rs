//! Verifies and executes register instructions.

use iris_runtime::{Kernel, KernelError, NativeSelector, Runtime, Selector, Value};

use crate::compile::{Instruction, Program, Register};

mod composed_types;
mod execute;
mod operations;
mod runtime;
mod stdlib;

#[derive(Clone, Debug)]
struct ClosureRecord {
    function: usize,
    captures: Vec<Value>,
}

#[derive(Clone, Debug)]
struct RevisionSubscriber {
    callback: iris_runtime::ObjectId,
    queued: Vec<(u64, String)>,
}

#[derive(Clone, Debug)]
enum IteratorSource {
    Array {
        source: iris_runtime::ArrayRef,
        expected_version: u64,
    },
    Hash {
        source: iris_runtime::HashRef,
        keys: Vec<Value>,
        expected_version: u64,
    },
    Values(Vec<Value>),
}

#[derive(Clone, Debug)]
struct IteratorRecord {
    source: Option<IteratorSource>,
    position: usize,
}

type TaskOutcome = Result<Value, Box<MachineError>>;

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
    bindings: std::collections::HashMap<String, Value>,
    iterators: std::collections::HashMap<iris_runtime::ObjectId, IteratorRecord>,
    reflection_grants: Vec<(String, String)>,
    revision_subscribers: Vec<RevisionSubscriber>,
    revision_history: Vec<u64>,
    revision_event_errors: Vec<Value>,
    modules: Vec<(String, iris_runtime::ModuleId)>,
    next_commit: u64,
    next_closure: u64,
    next_context: u64,
    next_iterator: u64,
    tasks: std::collections::HashMap<iris_runtime::ObjectId, TaskOutcome>,
    unobserved_failures: Vec<iris_runtime::ObjectId>,
    async_depth: usize,
    closure_depth: usize,
    /// Gates by identity, holding the posted value once completed.
    gates: std::collections::HashMap<iris_runtime::ObjectId, Option<Value>>,
    /// Async frames PAUSED at an `await`, in the order they suspended.
    ///
    /// `IRIS-V1-ASYNC-C014` resumes them in that order, so this is a queue
    /// rather than a map: two tasks awaiting one Gate must observe their
    /// effects in the order they suspended, not in hash order.
    suspended: Vec<SuspendedTask>,
    /// The frame state a `Suspended` signal is carrying outward.
    ///
    /// The signal itself only names the Gate, because it travels through
    /// `Result` returns that cannot carry a register file. The async call that
    /// started the frame takes this and records it as a suspended task.
    pending_frame: Option<PendingFrame>,
}

/// The register file and position an `await` paused at.
pub(super) struct PendingFrame {
    pub(super) gate: iris_runtime::ObjectId,
    pub(super) registers: Vec<Value>,
    pub(super) counter: usize,
    pub(super) destination: Option<Register>,
    pub(super) handlers: Vec<(usize, Register, Register)>,
}

/// An async frame paused at an `await`, and everything needed to resume it.
///
/// A suspended frame keeps its OWN register file and instruction pointer,
/// because `await` pauses in the middle of a body: the prefix has already run
/// and its locals must survive until the Gate completes.
pub(super) struct SuspendedTask {
    pub(super) identity: iris_runtime::ObjectId,
    pub(super) frame: PendingFrame,
    pub(super) function: usize,
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
            bindings: std::collections::HashMap::new(),
            iterators: std::collections::HashMap::new(),
            reflection_grants: Vec::new(),
            revision_subscribers: Vec::new(),
            revision_history: Vec::new(),
            revision_event_errors: Vec::new(),
            modules: Vec::new(),
            next_commit: 1,
            next_closure: 1,
            next_context: 900_000,
            next_iterator: 1_000_000,
            tasks: std::collections::HashMap::new(),
            unobserved_failures: Vec::new(),
            async_depth: 0,
            closure_depth: 0,
            gates: std::collections::HashMap::new(),
            suspended: Vec::new(),
            pending_frame: None,
        })
    }

    /// Replaces the Host reflection grants governing this machine run.
    pub fn enter_reflection_grants(&mut self, grants: Vec<(String, String)>) {
        self.reflection_grants = grants;
    }

    /// Verifies `program`, then runs it and answers its result register.
    ///
    /// # Errors
    /// Returns the verification failure or the kernel failure.
    pub fn execute(&mut self, program: &Program) -> Result<Value, MachineError> {
        verify(program).map_err(MachineError::Invalid)?;
        self.bindings.clear();
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

/// The specification-named error a machine failure reports, when it names one.
///
/// C056 hands the ORIGINAL raised value to the catch, and the reference makes
/// a failure the specification names catchable under that name, binding the
/// Symbol. Returning `None` keeps a failure that names no such error - and
/// every control-flow unwind - travelling to its own boundary instead of being
/// intercepted by an unrelated handler.
pub(super) fn catchable_name(error: &MachineError) -> Option<&'static str> {
    match error {
        MachineError::IndexError => Some("IndexError"),
        MachineError::IteratorState => Some("IteratorStateError"),
        MachineError::ConcurrentModification => Some("ConcurrentModificationError"),
        MachineError::TypeContractError => Some("TypeContractError"),
        MachineError::ReflectionAccess => Some("ReflectionAccessError"),
        MachineError::JsonSyntaxError => Some("JSONSyntaxError"),
        MachineError::SerializationError => Some("SerializationError"),
        // `C047` rejects an incomplete signature as an ordinary catchable Iris
        // error, so a program may `try { lib.bind(..) } catch e { e }` and
        // observe the name rather than losing the frame.
        MachineError::IncompleteNativeSignature => Some("IncompleteNativeSignatureError"),
        MachineError::UnboundNativeSymbol => Some("UnboundNativeSymbolError"),
        MachineError::AuditHistoryUnavailable => Some("AuditHistoryUnavailableError"),
        MachineError::HostDriveUnavailable => Some("HostDriveUnavailableError"),
        // NOT catchable. A bare `raise` with nothing propagating is a
        // control-flow error rather than a raised value, and the reference
        // lets it travel to its own boundary: `try { raise } catch e { e }`
        // answers the FAILURE, not `:NoActiveExceptionError`.
        MachineError::MessageNotFound { .. } => Some("MessageNotFound"),
        MachineError::NameError => Some("NameError"),
        MachineError::ClosedGenericOpenForbidden => Some("CLOSED_GENERIC_OPEN_FORBIDDEN"),
        _ => None,
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
        Value::ArrayIterator(_) => "ArrayIterator",
        Value::HashIterator(_) => "HashIterator",
        Value::IterationYield(_) | Value::IterationDone => "Iteration",
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
                .contracts
                .iter()
                .flat_map(|contract| contract.requirements.iter())
                .map(|requirement| requirement.selector.as_str()),
        )
        .chain(
            program
                .functions
                .iter()
                .filter_map(|function| function.name.split_once('.').map(|(_, selector)| selector)),
        )
        .chain(
            program
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter())
                .chain(program.instructions.iter())
                .filter_map(|instruction| match instruction {
                    Instruction::Send { selector, .. }
                    | Instruction::SendClass { selector, .. }
                    | Instruction::Reflection { selector, .. }
                    | Instruction::Revision { selector, .. }
                    | Instruction::SendContract { selector, .. }
                    | Instruction::BindMember { selector, .. }
                    | Instruction::GetIvar { name: selector, .. }
                    | Instruction::SetIvar { name: selector, .. } => Some(selector.as_str()),
                    Instruction::GetClassVar { name, .. }
                    | Instruction::SetClassVar { name, .. }
                    | Instruction::LoadSymbol { name, .. } => Some(name.as_str()),
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
