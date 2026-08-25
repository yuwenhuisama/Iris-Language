//! Bytecode instruction and program representation.

/// A virtual register index.
///
/// Registers are virtual and unbounded at this stage. Allocation to a fixed
/// bank belongs to a later pass; assigning them here would bake a machine
/// constraint into the IR before any backend needs it.
pub type Register = u16;

/// One three-address instruction.
///
/// Every instruction names its operands and its destination explicitly, so an
/// instruction's meaning does not depend on execution history. That is what
/// makes the verifier a single linear pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Loads an arbitrary-precision Integer, held as canonical decimal text.
    LoadInteger {
        destination: Register,
        digits: String,
    },
    /// Loads an IEEE-754 binary64 value, held as BITS so a literal cannot
    /// drift through a decimal round trip.
    LoadFloat64 {
        destination: Register,
        bits: u64,
    },
    /// Loads an IEEE-754 binary32 value, held as bits for the same reason.
    LoadFloat32 {
        destination: Register,
        bits: u32,
    },
    /// Loads a String.
    LoadText {
        destination: Register,
        text: String,
    },
    /// Loads an interned Symbol spelling.
    LoadSymbol {
        destination: Register,
        name: String,
    },
    /// Loads a Bool.
    LoadBool {
        destination: Register,
        value: bool,
    },
    /// Loads nil.
    LoadNil {
        destination: Register,
    },
    LoadIterationDone {
        destination: Register,
    },
    BuildIterationYield {
        destination: Register,
        value: Register,
    },
    LoadClass {
        destination: Register,
        class: usize,
    },
    LoadType {
        destination: Register,
        class: usize,
    },
    LoadContract {
        destination: Register,
        contract: usize,
    },
    LoadBuiltinType {
        destination: Register,
        name: String,
    },
    BuildType {
        destination: Register,
        expression: iris_syntax::TypeExpression,
    },
    LoadBuiltinClass {
        destination: Register,
        name: String,
    },
    LoadGlobal {
        destination: Register,
        name: String,
    },
    DeclareDeferred {
        register: Register,
    },
    RaiseDefiniteAssignment {
        destination: Register,
    },
    /// Refuses a program the reference refuses at RUN time.
    ///
    /// A program of only declarations, or only bindings, has no value to
    /// answer. The reference raises UnsupportedConstruct when it runs, so
    /// declining at COMPILE time made the two backends describe the same
    /// refusal differently and held the row instead of agreeing.
    RaiseUnsupported {
        destination: Register,
    },
    StoreGlobal {
        destination: Register,
        name: String,
        value: Register,
    },
    PublishBinding {
        destination: Register,
        name: String,
        source: Register,
    },
    LoadBinding {
        destination: Register,
        name: String,
        shared: bool,
    },
    StoreBinding {
        destination: Register,
        name: String,
        source: Register,
    },
    /// Copies one register to another.
    Move {
        destination: Register,
        source: Register,
    },
    MakeCell {
        destination: Register,
        source: Register,
    },
    LoadCell {
        destination: Register,
        cell: Register,
    },
    StoreCell {
        destination: Register,
        cell: Register,
        source: Register,
    },
    /// Sends a native binary selector: `destination = left <selector> right`.
    Binary {
        destination: Register,
        selector: &'static str,
        left: Register,
        right: Register,
    },
    /// Sends a native selector with no arguments to one register.
    Unary {
        destination: Register,
        selector: &'static str,
        operand: Register,
    },
    /// Builds an Array from a contiguous register range, in source order.
    BuildArray {
        destination: Register,
        first: Register,
        count: u16,
    },
    BuildTuple {
        destination: Register,
        first: Register,
        count: u16,
    },
    BuildRange {
        destination: Register,
        start: Register,
        end: Register,
        inclusive_end: bool,
    },
    BuildHash {
        destination: Register,
        first: Register,
        count: u16,
    },
    Index {
        destination: Register,
        receiver: Register,
        index: Register,
    },
    SetIndex {
        destination: Register,
        receiver: Register,
        index: Register,
        value: Register,
    },
    BindMember {
        destination: Register,
        receiver: Register,
        selector: String,
    },
    Identity {
        destination: Register,
        left: Register,
        right: Register,
    },
    TypeTest {
        destination: Register,
        value: Register,
        target: Register,
    },
    /// Reinterprets an Integer register's bits as a float of the given width.
    FromBits {
        destination: Register,
        width: FloatWidth,
        bits: Register,
    },
    Await {
        destination: Register,
        task: Register,
    },
    HostRun {
        destination: Register,
        task: Register,
    },
    UnobservedFailures {
        destination: Register,
    },
    /// Jumps to `target` when `condition` holds FALSE.
    ///
    /// Only the false branch is conditional. One conditional form plus an
    /// unconditional `Jump` expresses every shape this subset needs, and each
    /// extra branch opcode is another case the verifier must reason about.
    JumpUnless {
        condition: Register,
        target: usize,
    },
    /// Jumps to `target` unconditionally.
    Jump {
        target: usize,
    },
    /// Reads an Array's C026 content version into a register.
    ArrayVersion {
        destination: Register,
        array: Register,
    },
    ArrayNext {
        destination: Register,
        array: Register,
        index: Register,
        /// The Array's content version captured when the loop began.
        ///
        /// C026 increments that version on every length-changing or
        /// element-replacing operation so an ACTIVE iterator can detect the
        /// change. Advancing without comparing it let a loop that mutated its
        /// own Array run to a different length and answer a plausible wrong
        /// number instead of raising.
        version: Register,
        exhausted: usize,
    },
    RangeNext {
        destination: Register,
        range: Register,
        index: Register,
        exhausted: usize,
    },
    IteratorOpen {
        destination: Register,
        iterable: Register,
    },
    IteratorNext {
        destination: Register,
        iterator: Register,
        exhausted: usize,
    },
    IteratorClose {
        iterator: Register,
    },
    /// Installs an exception handler for the following protected region.
    EnterTry {
        handler: usize,
        cleanup: usize,
        exception: Register,
        context: Register,
    },
    CatchMatch {
        destination: Register,
        exception: Register,
        class: String,
    },
    /// Removes the innermost handler after normal completion.
    LeaveTry,
    /// Raises the value in the current frame.
    Raise {
        value: Register,
        cause: Option<Register>,
        offset: usize,
    },
    Propagate {
        value: Register,
        context: Register,
    },
    /// Allocates a Closure with a snapshot of a contiguous capture window.
    MakeClosure {
        destination: Register,
        function: usize,
        first: Register,
        count: u16,
    },
    /// Calls function `function` with a contiguous argument window.
    ///
    /// The arguments occupy `first .. first+count`, mirroring `BuildArray`, so
    /// a call names a REGISTER WINDOW rather than carrying an operand list.
    /// That is what keeps the callee's parameters addressable as ordinary
    /// registers once the frame is pushed.
    Call {
        destination: Register,
        function: usize,
        first: Register,
        count: u16,
    },
    BareCall {
        destination: Register,
        callee: Option<Register>,
        name: String,
        first: Register,
        count: u16,
    },
    Using {
        destination: Register,
        resource: Register,
        block: Register,
    },
    New {
        destination: Register,
        class: usize,
        first: Register,
        count: u16,
    },
    Send {
        destination: Register,
        receiver: Register,
        selector: String,
        first: Register,
        count: u16,
    },
    SendSuper {
        destination: Register,
        receiver: Register,
        owner: usize,
        selector: String,
        first: Register,
        count: u16,
    },
    SendClass {
        destination: Register,
        class: usize,
        selector: String,
        first: Register,
        count: u16,
    },
    Reflection {
        destination: Register,
        namespace: String,
        selector: String,
        first: Register,
        count: u16,
    },
    Revision {
        destination: Register,
        namespace: String,
        selector: String,
        first: Register,
        count: u16,
    },
    OpenClass {
        destination: Register,
        class: usize,
        callback: Register,
    },
    DefineMethod {
        destination: Register,
        receiver: Register,
        name: Register,
        function: usize,
    },
    Json {
        destination: Register,
        selector: String,
        first: Register,
        count: u16,
    },
    ContractCast {
        destination: Register,
        receiver: Register,
        contract: usize,
    },
    SendContract {
        destination: Register,
        receiver: Register,
        selector: String,
        first: Register,
        count: u16,
    },
    GetIvar {
        destination: Register,
        receiver: Register,
        name: String,
    },
    SetIvar {
        destination: Register,
        receiver: Register,
        name: String,
        value: Register,
    },
    GetClassVar {
        destination: Register,
        receiver: Register,
        name: String,
    },
    SetClassVar {
        destination: Register,
        receiver: Register,
        name: String,
        value: Register,
    },
    /// Returns `value` from the current frame.
    Return {
        value: Register,
    },
}

impl Instruction {
    /// The register this instruction writes, when it writes one.
    pub(crate) const fn destination(&self) -> Option<Register> {
        match self {
            Self::LoadInteger { destination, .. }
            | Self::LoadFloat64 { destination, .. }
            | Self::LoadFloat32 { destination, .. }
            | Self::LoadText { destination, .. }
            | Self::LoadSymbol { destination, .. }
            | Self::LoadBool { destination, .. }
            | Self::LoadNil { destination }
            | Self::LoadIterationDone { destination }
            | Self::BuildIterationYield { destination, .. }
            | Self::LoadClass { destination, .. }
            | Self::LoadType { destination, .. }
            | Self::LoadContract { destination, .. }
            | Self::LoadBuiltinType { destination, .. }
            | Self::BuildType { destination, .. }
            | Self::LoadBuiltinClass { destination, .. }
            | Self::LoadGlobal { destination, .. }
            | Self::StoreGlobal { destination, .. }
            | Self::PublishBinding { destination, .. }
            | Self::LoadBinding { destination, .. }
            | Self::StoreBinding { destination, .. }
            | Self::Move { destination, .. }
            | Self::MakeCell { destination, .. }
            | Self::LoadCell { destination, .. }
            | Self::StoreCell { destination, .. }
            | Self::Binary { destination, .. }
            | Self::Unary { destination, .. }
            | Self::BuildArray { destination, .. }
            | Self::BuildTuple { destination, .. }
            | Self::BuildRange { destination, .. }
            | Self::BuildHash { destination, .. }
            | Self::Index { destination, .. }
            | Self::SetIndex { destination, .. }
            | Self::BindMember { destination, .. }
            | Self::Identity { destination, .. }
            | Self::TypeTest { destination, .. }
            | Self::Call { destination, .. }
            | Self::BareCall { destination, .. }
            | Self::Using { destination, .. }
            | Self::New { destination, .. }
            | Self::Send { destination, .. }
            | Self::SendSuper { destination, .. }
            | Self::SendClass { destination, .. }
            | Self::Reflection { destination, .. }
            | Self::Revision { destination, .. }
            | Self::OpenClass { destination, .. }
            | Self::DefineMethod { destination, .. }
            | Self::Json { destination, .. }
            | Self::ContractCast { destination, .. }
            | Self::SendContract { destination, .. }
            | Self::GetIvar { destination, .. }
            | Self::SetIvar { destination, .. }
            | Self::GetClassVar { destination, .. }
            | Self::SetClassVar { destination, .. }
            | Self::MakeClosure { destination, .. }
            | Self::FromBits { destination, .. }
            | Self::Await { destination, .. }
            | Self::HostRun { destination, .. } => Some(*destination),
            Self::UnobservedFailures { destination } => Some(*destination),
            Self::IteratorOpen { destination, .. } | Self::IteratorNext { destination, .. } => {
                Some(*destination)
            }
            Self::RaiseDefiniteAssignment { destination }
            | Self::RaiseUnsupported { destination } => Some(*destination),
            Self::CatchMatch { destination, .. } => Some(*destination),
            // A branch or a return produces no value.
            Self::JumpUnless { .. }
            | Self::Jump { .. }
            | Self::EnterTry { .. }
            | Self::LeaveTry
            | Self::Raise { .. }
            | Self::Propagate { .. }
            | Self::Return { .. } => None,
            Self::IteratorClose { .. } => None,
            Self::DeclareDeferred { .. } => None,
            Self::ArrayVersion { destination, .. } => Some(*destination),
            Self::ArrayNext { destination, .. } | Self::RangeNext { destination, .. } => {
                Some(*destination)
            }
        }
    }
}

/// Which IEEE interchange width a `from_bits` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatWidth {
    /// IEEE-754 binary32.
    Bits32,
    /// IEEE-754 binary64.
    Bits64,
}

/// One callable body with its own register file.
///
/// A frame is a REGISTER WINDOW: each call gets a fresh file of `registers`
/// slots, and parameters arrive pre-bound in registers `0 .. parameters`.
/// Nothing is shared with the caller, so a callee cannot read a caller's
/// registers and recursion needs no save/restore of individual registers.
///
/// This is also what makes a GC root set enumerable: the live frames ARE the
/// roots. The tree-walking evaluator threads locals through a `&HashMap`
/// parameter, so its caller frames sit on the Rust stack and cannot be walked -
/// which is why a collection there refuses inside a method body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    /// The name this function was declared under, for diagnostics.
    pub(crate) name: String,
    /// How many leading registers hold parameters.
    pub(crate) parameters: usize,
    pub(crate) captures: usize,
    pub(crate) parameter_types: Vec<String>,
    pub(crate) return_type: String,
    pub(crate) is_async: bool,
    /// The size of this frame's register file.
    pub(crate) registers: usize,
    pub(crate) instructions: Vec<Instruction>,
}

/// A compiled program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub(crate) source: String,
    pub(crate) instructions: Vec<Instruction>,
    /// How many registers the top-level frame uses.
    pub(crate) registers: usize,
    /// The register holding the program's answer.
    pub(crate) result: Register,
    /// Callable bodies, addressed by index from `Call`.
    pub(crate) functions: Vec<Function>,
    pub(crate) classes: Vec<Class>,
    pub(crate) contracts: Vec<Contract>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Contract {
    pub(crate) name: String,
    pub(crate) requirements: Vec<ContractRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContractRequirement {
    pub(crate) selector: String,
    pub(crate) arity: usize,
    pub(crate) return_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) generic: bool,
    pub(crate) superclass: Option<usize>,
    pub(crate) methods: Vec<(String, usize)>,
    pub(crate) class_methods: Vec<(String, usize)>,
    pub(crate) reopens: Vec<ClassReopen>,
    pub(crate) contracts: Vec<usize>,
    pub(crate) property_methods: Vec<String>,
    pub(crate) class_variables: Vec<ClassVariable>,
    pub(crate) stored_properties: Vec<StoredProperty>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredProperty {
    pub(crate) name: String,
    pub(crate) initializer: LiteralValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassVariable {
    pub(crate) name: String,
    pub(crate) mutable: bool,
    pub(crate) initializer: LiteralValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LiteralValue {
    Integer(String),
    Text(String),
    Bool(bool),
    Nil,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassReopen {
    pub(crate) methods: Vec<(String, usize)>,
}
