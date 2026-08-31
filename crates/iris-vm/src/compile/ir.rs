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
    LoadBytes {
        destination: Register,
        bytes: Vec<u8>,
    },
    LoadByteArray {
        destination: Register,
        bytes: Vec<u8>,
    },
    MakeMutableString {
        destination: Register,
        source: Register,
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
    /// Declares a deferred binding: a value register plus an ASSIGNED flag.
    ///
    /// The reference checks definite assignment when the binding is READ, at
    /// RUN time, and whether an assignment ran is not a compile-time fact:
    /// `mut x: Integer; if c { x = 1 }; x` answers `1` when `c` holds and
    /// fails when it does not, for the SAME code. So the flag is carried in a
    /// register and consulted by `ReadDeferred`, rather than being tracked as
    /// a set the compiler discharges.
    DeclareDeferred {
        register: Register,
        assigned: Register,
    },
    /// Marks a deferred binding assigned, whenever this instruction RUNS.
    MarkAssigned {
        assigned: Register,
    },
    /// Reads a deferred binding, failing unless it was actually assigned.
    ReadDeferred {
        destination: Register,
        value: Register,
        assigned: Register,
    },
    /// Refuses a program the reference refuses at RUN time.
    ///
    /// A program of only declarations, or only bindings, has no value to
    /// answer. The reference raises UnsupportedConstruct when it runs, so
    /// declining at COMPILE time made the two backends describe the same
    /// refusal differently and held the row instead of agreeing.
    /// Reduces a value to a Bool through the `to_bool` protocol.
    ///
    /// `IRIS-V1-CONTROL-C022` makes `false` and `nil` falsey by DEFAULT, but a
    /// class may define `to_bool`, and the answer is that method's result.
    /// Deciding truth structurally instead made `if p` take the then-branch
    /// for a `p` whose `to_bool` answers false - a wrong answer rather than a
    /// hold. A non-Bool result is a `TypeContractError`.
    TestTruth {
        destination: Register,
        value: Register,
    },
    /// Loads a Regex literal, already canonicalized and validated.
    ///
    /// `IRIS-V1-COLLECTIONS-C081` stores flags in `imsx` order with absent
    /// flags omitted, so `/a+/im` and `/a+/mi` compare and hash equal. The
    /// pattern is compiled at COMPILE time only to reject it - the engine is
    /// not carried into the value, which stays the canonical text pair.
    LoadRegex {
        destination: Register,
        pattern: String,
        flags: String,
    },
    /// Decodes bytes in a NAMED Encoding, per `IRIS-V1-LIBRARY-C022`.
    ///
    /// Strict handling is the DEFAULT, so an invalid sequence fails unless the
    /// caller asked for `errors: :replace` by name.
    EncodingDecode {
        destination: Register,
        encoding: &'static str,
        value: Register,
        first: Register,
        count: u16,
    },
    /// Refuses an IMPLICIT encoding selection, per `IRIS-V1-LIBRARY-C025`.
    ///
    /// A host default, an OS locale or a code page names no Encoding at all,
    /// and choosing one for decoding requires the caller to name a real one.
    RaiseEncodingSelection {
        destination: Register,
        code: &'static str,
    },
    /// Opens a native library, per `IRIS-V1-FFI-C043`.
    ///
    /// Each open takes a FRESH identity rather than being cached by path, so
    /// two opens of one path are two objects. A sidecar `declarations:` is
    /// validated exactly as a programmatic bind would be, per `C045`.
    FfiOpen {
        destination: Register,
        path: Register,
        first: Register,
        count: u16,
    },
    /// Encodes a value as `IrisValue` data, per `IRIS-V1-LIBRARY-C018`.
    ///
    /// An object is asked for its OWN representation through `serialize`, and
    /// only a class declaring `Serializable` may answer one - `C003` keeps a
    /// live resource out rather than emitting its identity.
    IrisValueEncode {
        destination: Register,
        value: Register,
    },
    /// Decodes an `IrisValue` stream, validating the header FIRST.
    ///
    /// `C016` checks magic and format version before any payload that depends
    /// on them, and `C017` forbids allocating from a DECLARED length before
    /// that length is validated - so a limit breach is refused before the
    /// payload is read at all.
    IrisValueDecode {
        destination: Register,
        stream: Register,
        first: Register,
        count: u16,
    },
    /// Escapes a value for LITERAL matching inside a pattern.
    EscapeRegex {
        destination: Register,
        value: Register,
    },
    /// Builds a Regex from text computed at RUN time.
    ///
    /// An interpolating literal splices a value the compiler cannot know, so
    /// the pattern is assembled and validated when it runs. Each spliced value
    /// is ESCAPED before it reaches the pattern, so `/${x}/` matches the text
    /// `x` holds rather than reinterpreting it as syntax.
    MakeRegex {
        destination: Register,
        pattern: Register,
        flags: String,
    },
    /// Writes its operands to standard output, separated by spaces.
    ///
    /// Rendering goes through `to_string`, so a class's own definition is
    /// honoured rather than bypassed. `print` is a bare call no binding
    /// claims, which is what distinguishes it from a local of that name.
    Print {
        destination: Register,
        first: Register,
        count: u16,
    },
    /// Guards an ANNOTATED binding or parameter, per `IRIS-V1-TYPES-C004`.
    ///
    /// `let s: String = 1` is a type failure the reference raises when the
    /// binding runs, so the annotation is a runtime guarantee rather than
    /// discarded metadata. It shares `CheckReturn`'s admission rule: an
    /// annotation the backend cannot decide admits every value.
    CheckAnnotation {
        value: Register,
        annotation: iris_syntax::TypeExpression,
    },
    /// Guards the RETURN boundary, per `IRIS-V1-TYPES-C004`.
    ///
    /// A declared return Type is checked before the value is published to the
    /// caller, whether the body fell off the end or returned explicitly.
    /// Without this a method annotated `-> Nil` could answer a Symbol, so the
    /// backend ran a program the reference refuses.
    CheckReturn {
        value: Register,
        annotation: iris_syntax::TypeExpression,
    },
    /// Applies a class REOPEN at the position it was written.
    ///
    /// A reopen takes effect where it appears in the source, not at load: a
    /// call made BEFORE `open class P { override fun m() }` still answers the
    /// original body. Publishing every reopen up front made the earlier call
    /// answer from the replacement, so the transaction is driven from here.
    ApplyReopen {
        class: usize,
        reopen: usize,
    },
    /// Crosses the native ABI, per `IRIS-V1-FFI-C018` and `C027`.
    ///
    /// `raise` converts the ABI's status plus context handle into the ordinary
    /// Iris exception a `catch` observes - `C017` makes a status alone
    /// insufficient, so the raised value is read back THROUGH the handle
    /// rather than recomputed. `resource` registers a payload whose release
    /// counter stays behind the ABI, which is what makes `C030`'s idempotence
    /// observable rather than asserted.
    NativeFixture {
        destination: Register,
        selector: String,
        first: Register,
        count: u16,
        /// Registers a NAME claims where this call appears.
        ///
        /// A frame's register file lives on the Rust stack, where a collector
        /// cannot walk it, and a temporary the source never bound is already
        /// unreachable - so the bound registers travel with the instruction
        /// rather than the whole file being treated as live.
        roots: Vec<Register>,
    },
    /// Answers the contexts a `finally` transfer DISCARDED.
    ///
    /// `IRIS-V1-ASYNC-C028` forbids a discarded propagation from disappearing
    /// silently, so a `finally` that returns out of a raising body records the
    /// context it dropped rather than losing it.
    DiscardedContexts {
        destination: Register,
    },
    /// Validates a package claim, per `IRIS-V1-LIBRARY-C028`.
    ///
    /// A separately versioned package MUST NOT claim core ABI or replace core
    /// literal semantics, so the claim is rejected at VALIDATION time and core
    /// behaviour is left untouched.
    PackageValidate {
        destination: Register,
        claims_core: bool,
    },
    /// Answers the pinned Unicode data version, per `C042`.
    UnicodeVersion {
        destination: Register,
    },
    /// Creates a fresh Gate, per `IRIS-V1-ASYNC-C014`.
    GateNew {
        destination: Register,
    },
    /// Posts a Gate's completion, readying every frame awaiting it.
    ///
    /// `C014` readies them in the order they SUSPENDED, so a Gate carries a
    /// queue rather than a set: two tasks awaiting one Gate must observe their
    /// effects in suspension order.
    GateComplete {
        destination: Register,
        gate: Register,
        value: Register,
    },
    /// Negates an already-tested truth value, for `!`.
    NegateTruth {
        destination: Register,
        value: Register,
    },
    /// Wraps a value as a NAMED argument.
    ///
    /// A keyword argument stays in the ordinary argument list rather than
    /// forming a second channel, which is what keeps the left-to-right
    /// evaluation order `IRIS-V1-CONTROL-C026` requires across both.
    MakeKeywordArgument {
        destination: Register,
        name: String,
        value: Register,
    },
    RaiseUnsupported {
        destination: Register,
    },
    /// Binds the frame's parameters, per `IRIS-V1-CONTROL-C023`.
    ///
    /// Positionals fill in order, `*rest` takes the remaining positionals as a
    /// fresh Array, a `key` parameter binds by NAME rather than position, and
    /// `**kwargs` collects the keywords no declared parameter matched. The
    /// binding happens in the CALLEE because a dynamic send does not know the
    /// signature until dispatch, and `C025` raises ArgumentError when a
    /// required parameter is left unbound or a keyword is supplied twice.
    BindParameters {
        destination: Register,
        kinds: Vec<(ParameterKind, String)>,
        /// Whether the frame's leading register holds `self`.
        receiver: bool,
        first: Register,
        count: u16,
    },
    /// Writes a parameter's DEFAULT when the caller supplied no argument.
    ///
    /// A frame is entered with unfilled parameters holding nil, and a call
    /// site that resolves the callee can substitute defaults itself - but a
    /// dynamic SEND cannot, because it does not know the signature until
    /// dispatch. Filling them in the callee makes every path agree.
    DefaultParameter {
        destination: Register,
        source: Register,
        index: usize,
    },
    /// Refuses a `break` or `continue` that has no enclosing loop.
    RaiseLoopTransfer {
        destination: Register,
    },
    /// Refuses a program the PARSER rejected, the way the reference does.
    ///
    /// A source the parser refuses is a program error the reference raises
    /// when the program RUNS, as `ParseDiagnostic`. Declining at compile time
    /// refused the same program while describing it differently, which holds
    /// the row instead of agreeing.
    RaiseParseDiagnostic {
        destination: Register,
    },
    /// Refuses an unbound NAME the way the reference does, at run time.
    ///
    /// A name with no binding is a `NameError` the reference raises when the
    /// read runs. Declining instead made both backends refuse the same
    /// program while describing it differently, which holds the row.
    RaiseNameError {
        destination: Register,
    },
    /// Binds one element of a DESTRUCTURING loop binding.
    ///
    /// `IRIS-V1-CONTROL-C045` raises `PatternMatchError` when the item is not
    /// an Array of exactly the pattern's arity, so the arity is carried here
    /// rather than the element being read with a plain index - an index would
    /// answer nil for a missing position instead of failing.
    DestructureElement {
        destination: Register,
        item: Register,
        position: usize,
        arity: usize,
    },
    /// Fails a call whose argument COUNT the signature cannot bind.
    ///
    /// `IRIS-V1-CONTROL-C023` binds each parameter from the arguments, so a
    /// count with no binding is an ArgumentError raised when the call runs -
    /// the reference refuses it there rather than statically.
    RaiseArgumentError {
        destination: Register,
    },
    /// Fails a generic MATERIALIZATION, per `IRIS-V1-TYPES-C067`.
    ///
    /// A `where T: SomeContract` bound is checked when the class is
    /// CONSTRUCTED, not where it is declared: `class Box<T> where T:
    /// Comparable<T> {}` declares fine and `Box<String>.new()` is the
    /// TypeContractError, since String declares no such contract.
    RaiseTypeContract {
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
    ReRaise {
        value: Register,
        context: Register,
        offset: usize,
    },
    RaiseNoActiveException,
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
        /// The CLASS whose body wrote this send, when one did.
        ///
        /// `IRIS-V1-RUNTIME-C077` refuses a private method from every path but
        /// the declaring class, so dispatch needs the caller's lexical owner.
        /// A send written outside any class has none and is external.
        caller: Option<usize>,
        /// The MODULE whose body wrote this send, when one did.
        ///
        /// A module composed with `private` access reaches the composing
        /// class's private methods, so the module is the authority there -
        /// a class owner cannot express it.
        caller_module: Option<String>,
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
    /// Registers a NAME claims at this instruction, for the collector.
    pub(crate) fn roots(&self) -> &[Register] {
        match self {
            Self::NativeFixture { roots, .. } => roots,
            _ => &[],
        }
    }

    /// The register this instruction writes, when it writes one.
    pub(crate) const fn destination(&self) -> Option<Register> {
        match self {
            Self::LoadInteger { destination, .. }
            | Self::LoadFloat64 { destination, .. }
            | Self::LoadFloat32 { destination, .. }
            | Self::LoadText { destination, .. }
            | Self::LoadBytes { destination, .. }
            | Self::LoadByteArray { destination, .. }
            | Self::MakeMutableString { destination, .. }
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
            Self::TestTruth { destination, .. } | Self::NegateTruth { destination, .. } => {
                Some(*destination)
            }
            Self::MakeKeywordArgument { destination, .. } => Some(*destination),
            Self::LoadRegex { destination, .. } => Some(*destination),
            Self::GateNew { destination } | Self::UnicodeVersion { destination } => {
                Some(*destination)
            }
            Self::MakeRegex { destination, .. } | Self::EscapeRegex { destination, .. } => {
                Some(*destination)
            }
            Self::ApplyReopen { .. } | Self::CheckReturn { .. } | Self::CheckAnnotation { .. } => {
                None
            }
            Self::NativeFixture { destination, .. } | Self::Print { destination, .. } => {
                Some(*destination)
            }
            Self::DiscardedContexts { destination } | Self::PackageValidate { destination, .. } => {
                Some(*destination)
            }
            Self::IrisValueEncode { destination, .. }
            | Self::IrisValueDecode { destination, .. }
            | Self::FfiOpen { destination, .. }
            | Self::EncodingDecode { destination, .. }
            | Self::RaiseEncodingSelection { destination, .. } => Some(*destination),
            Self::GateComplete { destination, .. } => Some(*destination),
            Self::DefaultParameter { destination, .. }
            | Self::BindParameters { destination, .. } => Some(*destination),
            Self::RaiseParseDiagnostic { destination }
            | Self::RaiseLoopTransfer { destination } => Some(*destination),
            Self::RaiseTypeContract { destination } | Self::RaiseArgumentError { destination } => {
                Some(*destination)
            }
            Self::DestructureElement { destination, .. } => Some(*destination),
            Self::RaiseUnsupported { destination } | Self::RaiseNameError { destination } => {
                Some(*destination)
            }
            Self::CatchMatch { destination, .. } => Some(*destination),
            // A branch or a return produces no value.
            Self::JumpUnless { .. }
            | Self::Jump { .. }
            | Self::EnterTry { .. }
            | Self::LeaveTry
            | Self::Raise { .. }
            | Self::ReRaise { .. }
            | Self::RaiseNoActiveException
            | Self::Propagate { .. }
            | Self::Return { .. } => None,
            Self::IteratorClose { .. } => None,
            Self::DeclareDeferred { register, .. } => Some(*register),
            Self::MarkAssigned { assigned } => Some(*assigned),
            Self::ReadDeferred { destination, .. } => Some(*destination),
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
    /// The exact argument count this function ACCEPTS, when it fixes one.
    ///
    /// `IRIS-V1-CONTROL-C023` lets a default, a `*rest`, a keyword or a block
    /// parameter accept a range of counts, so only a purely positional
    /// signature with no defaults fixes one. A call that supplies a different
    /// count then has no binding for a parameter, which is an ArgumentError
    /// rather than a nil quietly filled in.
    pub(crate) fixed_arity: Option<usize>,
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
    /// Declared modules and the modules they MIX IN, in declaration order.
    ///
    /// A module is otherwise discovered from the names of its functions, which
    /// misses one that only composes others: `module B mixin A { }` declares
    /// no method of its own, so nothing would register it and `C mixin B`
    /// would not reach `A`'s methods.
    pub(crate) modules: Vec<ModuleDeclaration>,
    pub(crate) builtin_reopens: Vec<BuiltinReopen>,
}

/// A reopen of a BUILT-IN Class, which has no user declaration to attach to.
///
/// `open class Integer { .. }` adds methods reachable on every Integer value.
/// The target is named rather than indexed, because the built-in classes are
/// created by the kernel and have no entry in `classes`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuiltinReopen {
    pub(crate) target: String,
    pub(crate) methods: Vec<(String, usize)>,
    /// Contracts the reopen declares the built-in class satisfies.
    ///
    /// `open class Integer for N { .. }` makes `1 as N` a legitimate view, so
    /// the conformance is recorded even though the class itself is the
    /// kernel's and has no entry in `classes`.
    pub(crate) contracts: Vec<usize>,
}

/// One parameter's binding CATEGORY, per `IRIS-V1-CONTROL-C023`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterKind {
    Positional,
    Rest,
    Keyword,
    KeywordRest,
    Block,
}

/// A declared module and the modules composed into it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModuleDeclaration {
    pub(crate) name: String,
    pub(crate) mixins: Vec<String>,
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
    /// The written Type of each parameter, by position.
    ///
    /// `D-173` puts the contract-visible SIGNATURE in the static spine, so a
    /// member whose parameter Type differs from the requirement is an
    /// incompatible replacement even when the arities agree. An unannotated
    /// position states nothing and is left alone.
    pub(crate) parameter_types: Vec<Option<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Class {
    pub(crate) name: String,
    /// Selectors the class declared `private`, per `IRIS-V1-RUNTIME-C077`.
    ///
    /// A private method is refused from every path but the declaring class -
    /// or a module composed with `private` access - so the marker has to reach
    /// the registry rather than being dropped at the declaration.
    pub(crate) private_methods: Vec<String>,
    pub(crate) generic: bool,
    pub(crate) superclass: Option<usize>,
    pub(crate) methods: Vec<(String, usize)>,
    pub(crate) class_methods: Vec<(String, usize)>,
    pub(crate) reopens: Vec<ClassReopen>,
    pub(crate) contracts: Vec<usize>,
    /// Each mixin's module NAME and whether it was written `private`.
    ///
    /// A module is registered before any class, so the name resolves to a
    /// module identity at load time rather than at compile time. A
    /// `mixin M private` grants `M` reach into the class's private methods,
    /// which is a composition-edge fact rather than an annotation, so the
    /// marker travels with the edge.
    pub(crate) mixins: Vec<(String, bool)>,
    /// Contract bounds on the class's type PARAMETERS, by position.
    ///
    /// `IRIS-V1-TYPES-C067` checks these at MATERIALIZATION rather than where
    /// the class is declared, so they are carried until a construction names
    /// concrete arguments.
    pub(crate) contract_bounds: Vec<(usize, usize)>,
    /// Methods written as `impl fun C::m()`, keyed by contract and selector.
    ///
    /// A QUALIFIED implementation is visible only through that contract's
    /// view: `(a as C)..m()` answers it while `a.m()` answers the class's
    /// ordinary method, so it cannot be published onto the class itself.
    pub(crate) qualified_impls: Vec<(usize, String, usize)>,
    pub(crate) property_methods: Vec<String>,
    pub(crate) class_variables: Vec<ClassVariable>,
    pub(crate) stored_properties: Vec<StoredProperty>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredProperty {
    pub(crate) name: String,
    pub(crate) initializer: LiteralValue,
    /// A function computing the initializer, when it is not a literal.
    ///
    /// An initializer is an ordinary EXPRESSION evaluated at construction with
    /// `self` bound, so `property tag: Symbol = arm()` calls the object's own
    /// `arm`. Only a literal can be stored directly; anything else needs a
    /// frame, and refusing those declined the form outright.
    pub(crate) initializer_function: Option<usize>,
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
    /// A reopen may also REPLACE a class method, which is published onto the
    /// singleton rather than the instance side - the two are separate tables,
    /// so an override of `class fun +` must not shadow an instance `+`.
    pub(crate) class_methods: Vec<(String, usize)>,
}
