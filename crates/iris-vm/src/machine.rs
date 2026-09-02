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
    /// Commits the queue holds before older ones are DROPPED.
    ///
    /// `IRIS-V1-ASYNC-C051` bounds the queue at a capacity the subscriber
    /// names, so a subscriber that falls behind loses the oldest events rather
    /// than growing without limit. An unnamed capacity is unbounded.
    capacity: usize,
    /// The inclusive commit range dropped for capacity, if any.
    ///
    /// `C052` makes the range INCLUSIVE and forbids pretending no change
    /// occurred, so successive drops extend one range rather than each
    /// reporting separately.
    gap: Option<(u64, u64)>,
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
    /// A LIVE view over a MutableString's own content.
    ///
    /// `IRIS-V1-COLLECTIONS-C061` makes the cursor fail fast: it captures the
    /// content version, and ANY change to the receiver invalidates it rather
    /// than letting it walk a snapshot the program can no longer see.
    Text {
        source: iris_runtime::MutableStringRef,
        values: Vec<Value>,
        expected_version: u64,
    },
}

#[derive(Clone, Debug)]
struct IteratorRecord {
    source: Option<IteratorSource>,
    position: usize,
    /// Whether the CURRENT entry was already removed.
    ///
    /// `IRIS-V1-COLLECTIONS-C026` lets a Hash iterator remove the entry it
    /// just yielded, once - a second removal names no entry, so it is an
    /// iterator-state failure rather than a silent no-op.
    removed_current: bool,
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
    /// Audit events a configured SINK persisted, per `IRIS-V1-ASYNC-C055`.
    ///
    /// The sink is a separate persistence layer that survives a `prune`, which
    /// is C055's point: the in-memory queue offers no zero-loss guarantee, so
    /// `recover` answers what the sink holds independently of what retained
    /// history still has.
    audit_sink: Option<Vec<u64>>,
    /// A snapshot of each ACTIVE frame's registers, for the collector.
    ///
    /// The live frames ARE the root set, and a frame's register file lives on
    /// the Rust stack where a collector cannot walk it. Each frame therefore
    /// publishes its registers here while it runs, which is what makes a
    /// collection inside a body safe rather than unsound.
    frame_roots: Vec<Vec<Value>>,
    /// Instructions the run may still execute.
    ///
    /// A machine with no bound HANGS on a program that never terminates, which
    /// is worse than failing: a caller cannot tell a slow run from a stuck
    /// one. The reference charges a step per operation and fails the run when
    /// the budget is gone, so this matches that guarantee.
    remaining_steps: u64,
    /// The context a running CLEANUP body would chain as a cause.
    ///
    /// `IRIS-V1-CONTROL-C067` makes a `finally` that raises while another
    /// exception propagates report the interrupted one as its `cause`, so the
    /// propagating context is held for the duration of the cleanup.
    pending_cleanup_cause: Option<Value>,
    revision_event_errors: Vec<Value>,
    /// Whether `Revision.shutdown` closed delivery.
    ///
    /// `IRIS-V1-ASYNC-C050` names shutdown as the condition under which a
    /// flush must report INCOMPLETE delivery: nothing queued can reach a
    /// terminal state afterwards, so the accepted-but-undelivered count is the
    /// structured state a diagnostic needs.
    revision_delivery_closed: bool,
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
    /// The last DECODER refusal, as `(decoder, offset, expected)`.
    ///
    /// `IRIS-V1-LIBRARY-C036` requires a safe decoding diagnostic to identify
    /// the decoder, the offset when available and the violated limit or
    /// expected construct - so a caught failure can say WHERE the document
    /// stopped being readable rather than only that it did.
    decoder_diagnostic: Option<(&'static str, usize, &'static str)>,
    /// Propagations a `finally` transfer DISCARDED, per `IRIS-V1-ASYNC-C028`.
    discarded_contexts: Vec<Value>,
    /// How many META TRANSACTION bodies are currently running.
    ///
    /// `IRIS-V1-META-C037` makes a transaction body NON-SUSPENDING, so an
    /// `await` inside one is refused rather than parking a frame the
    /// transaction would have to publish or roll back around.
    open_depth: usize,
    /// Class-level initializers that have not RUN yet, by class and selector.
    ///
    /// A class-level initializer is an ordinary EXPRESSION evaluated the first
    /// time the property is read rather than when the class is defined, so a
    /// body that raises is retried on the next read and one that succeeds runs
    /// exactly once.
    pending_class_initializers:
        std::collections::HashMap<(iris_runtime::ClassId, iris_runtime::Selector), usize>,
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
            audit_sink: None,
            frame_roots: Vec::new(),
            remaining_steps: STEP_BUDGET,
            pending_cleanup_cause: None,
            revision_event_errors: Vec::new(),
            revision_delivery_closed: false,
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
            decoder_diagnostic: None,
            discarded_contexts: Vec::new(),
            open_depth: 0,
            pending_class_initializers: std::collections::HashMap::new(),
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
/// Instructions one run may execute before it is declared non-terminating.
///
/// The reference uses the same bound, so a program that exhausts it fails
/// alike on both rather than hanging on one.
const STEP_BUDGET: u64 = 1_000_000;

/// The value a failure CARRIES, as a program would catch it.
///
/// A raise carries its own operand, and a failure the specification NAMES
/// carries that name as a Symbol - which is what a diagnostic report shows
/// rather than the machine's own spelling of the error.
pub(super) fn captured_value(error: &MachineError) -> Value {
    match error {
        MachineError::Raised(propagation) => propagation.0.clone(),
        error => catchable_name(error).map_or(Value::Nil, |name| Value::Symbol(name.to_owned())),
    }
}

/// The `C081` NAME of one capability.
///
/// The vocabulary is fixed, so a denial is reported by the name the source
/// wrote rather than by an implementation spelling of its own.
pub(super) const fn capability_name(capability: iris_runtime::Capability) -> &'static str {
    match capability {
        iris_runtime::Capability::MethodSet => "method_set",
        iris_runtime::Capability::MethodBody => "method_body",
        iris_runtime::Capability::PropertySet => "property_set",
        iris_runtime::Capability::PropertyBody => "property_body",
        iris_runtime::Capability::Modules => "modules",
        iris_runtime::Capability::Superclass => "superclass",
        iris_runtime::Capability::Subclass => "subclass",
        iris_runtime::Capability::Shape => "shape",
        iris_runtime::Capability::ClassStateSet => "class_state_set",
        iris_runtime::Capability::ClassStateWrite => "class_state_write",
        iris_runtime::Capability::InstanceState => "instance_state",
        iris_runtime::Capability::Native => "native",
    }
}

pub(super) fn catchable_name(error: &MachineError) -> Option<&'static str> {
    match error {
        MachineError::IndexError => Some("IndexError"),
        MachineError::RangeError => Some("RangeError"),
        // `C012` refuses invalid UTF-8 at the boundary, and that refusal is an
        // ordinary catchable failure - leaving it off this list let it escape
        // a `try` that plainly names it.
        MachineError::EncodingError => Some("EncodingError"),
        MachineError::IteratorState => Some("IteratorStateError"),
        MachineError::ConcurrentModification => Some("ConcurrentModificationError"),
        MachineError::TypeContractError => Some("TypeContractError"),
        MachineError::InvalidKeyError => Some("InvalidKeyError"),
        // `fetch` REFUSES an absent key, which is what makes it an assertion
        // that the key is present - and that refusal is catchable by name.
        MachineError::KeyError => Some("KeyError"),
        MachineError::KeyConflictError => Some("KeyConflictError"),
        MachineError::IdentityError => Some("IdentityError"),
        MachineError::ImmutableBinding => Some("ImmutableBindingError"),
        MachineError::InvalidInstanceVariableName => Some("InvalidInstanceVariableNameError"),
        MachineError::InstanceStateError => Some("InstanceStateError"),
        MachineError::MetaTransactionError => Some("MetaTransactionError"),
        // `C077` names the visibility failure, and `V434` observes a private
        // call being refused from every path but the declaring class.
        MachineError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::VisibilityDenied { .. },
        )) => Some("MethodVisibilityError"),
        // `C015` binds a REFLECTED Method against the receiver's CURRENT MRO,
        // and that refusal is an ordinary catchable failure a program names.
        MachineError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::MethodBinding { .. },
        )) => Some("MethodBindingError"),
        // A call whose arity does not match its body is an ordinary catchable
        // Iris error, so `try { .. } catch e { e }` binds it by name.
        MachineError::ArgumentError => Some("ArgumentError"),
        MachineError::PatternMatchError => Some("PatternMatchError"),
        MachineError::ReflectionAccess => Some("ReflectionAccessError"),
        MachineError::JsonSyntaxError => Some("JSONSyntaxError"),
        MachineError::JsonDuplicateNameError => Some("JSONDuplicateNameError"),
        MachineError::JsonLimitError => Some("JSONLimitError"),
        MachineError::ComparisonContractError => Some("ComparisonContractError"),
        // `C160` expects a RESOURCE refusal for an allocation the host cannot
        // satisfy, and an ordinary catchable failure is what lets a program
        // observe it rather than dying undiagnosed.
        MachineError::Kernel(iris_runtime::KernelError::Numeric(
            iris_runtime::NumericError::Resource,
        )) => Some("ResourceError"),
        // `C081` refuses a DENIED meta operation, and that refusal is an
        // ordinary catchable failure a program can name. A BUILT-IN class
        // protecting its superclass is a different FACT with the same name to
        // a program: uncaught it reports the protection, and caught it binds
        // the capability name like any other meta refusal.
        MachineError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
            | iris_runtime::ClassError::ProtectedSuperclass { .. },
        ) => Some("MetaCapabilityError"),
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

/// The slot a `@name` ivar reference addresses.
///
/// A stored PROPERTY declares its slot under the bare name while the source
/// writes `@n` for it, so the sigil is stripped when a property of that name
/// exists. An ivar the class never declared keeps its written spelling, which
/// is what lets `@z = 5` work in a class with no such property.
pub(super) fn ivar_slot_name(program: &Program, name: &str) -> String {
    let bare = name.trim_start_matches('@');
    let declared = program.classes.iter().any(|class| {
        class
            .stored_properties
            .iter()
            .any(|property| property.name == bare)
    });
    if declared {
        bare.to_owned()
    } else {
        name.to_owned()
    }
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
