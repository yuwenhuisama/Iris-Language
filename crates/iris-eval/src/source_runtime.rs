use std::collections::HashMap;

use iris_runtime::{
    ArrayRef, Capability, ClassError, ClassId, ClassRevision, ComparisonSlot, CompositionEdge,
    DispatchContext, DispatchError, DispatchOutcome, HashRef, Kernel, MetaCapabilities, Method,
    MethodBody, MethodOwner, ModuleId, Runtime, Selector, StaticSpine, Truthiness, TruthinessError,
    TruthinessMethod, Value,
};
use iris_syntax::{
    BinaryOperator, ClassDeclaration, Expression, MethodDeclaration, MethodKind, ModuleDeclaration,
    Program, ProgramEntry, Statement,
};

use crate::EvaluationError;
use crate::source_method::{builtin, literal, visibility};

/// The package identity a manifestless local script runs under.
///
/// `IRIS-V1-IDENTITY-C030` gives such a script runtime-local package identity,
/// so one anonymous name serves every program the Host runs without a manifest.
pub(super) const LOCAL_PACKAGE: &str = "runtime-local";

pub(super) fn evaluate(program: &Program, source: &str) -> Result<Value, EvaluationError> {
    let mut evaluator = SourceEvaluator::new_in_package(LOCAL_PACKAGE)?;
    evaluator.source = source.to_owned();
    evaluator.program(program)
}

/// The code, captured environment and receiver of one Closure allocation.
///
/// `IRIS-V1-RUNTIME-C072` requires a Closure created in an instance Method to
/// capture its CURRENT receiver and keep reading and writing that receiver's raw
/// ivars after escape, so the receiver is stored alongside the captured locals.
struct ClosureRecord {
    parameters: Vec<String>,
    body: Vec<Statement>,
    captured: HashMap<String, Value>,
    receiver: Option<Value>,
}

/// Resolves an `IRIS-V1-COLLECTIONS-C009` index against a receiver length.
///
/// A negative index resolves as `length + index` in the receiver's indexing
/// unit. An index that stays out of range after resolution yields `None`, which
/// a read turns into `nil` and a write turns into `IndexError`.
fn resolve_index(index: &iris_runtime::IntegerValue, length: usize) -> Option<usize> {
    if let Some(position) = index.to_usize() {
        return Some(position);
    }
    // A negative value has no `to_usize`, so the magnitude is read from its
    // canonical decimal text and subtracted from the length.
    let text = index.decimal_text();
    let magnitude: usize = text.strip_prefix('-')?.parse().ok()?;
    length.checked_sub(magnitude)
}

/// A live cursor over a shared Array body.
///
/// `IRIS-V1-COLLECTIONS-C026` requires an active iterator to raise
/// `ConcurrentModificationError` on its next advance once the Array changes
/// structurally, so the cursor keeps the SHARED body and the version it expects
/// rather than a private copy of the elements.
#[derive(Debug)]
struct ArrayCursor {
    values: ArrayRef,
    position: usize,
    expected_version: u64,
    /// A `D-142` read-only view cannot be mutated, so its cursor never fails.
    fail_fast: bool,
}

/// One registered revision-event subscriber.
///
/// `IRIS-V1-ASYNC-C051` gives each subscriber a BOUNDED queue and requires a
/// dropped range to be coalesced into a `GapEvent` delivered before later
/// retained events, so the queue and the pending gap travel together.
#[derive(Debug)]
struct RevisionSubscriber {
    /// The subscriber Closure, invoked once per delivered event.
    callback: iris_runtime::ObjectId,
    /// Capacity, after which older commits are dropped into a gap.
    capacity: usize,
    /// Commit ids accepted into the queue and not yet delivered.
    queued: Vec<u64>,
    /// The inclusive commit range dropped for capacity, if any.
    gap: Option<(u64, u64)>,
}

/// A live cursor over a byte sequence.
///
/// `IRIS-V1-COLLECTIONS-C075` makes ANY ByteArray content mutation invalidate
/// the cursor, so this compares the content version rather than a structural
/// one. Bytes is immutable, so its cursor never fails fast.
#[derive(Debug)]
struct ByteCursor {
    /// The values this cursor yields, already materialized.
    ///
    /// `C075` yields Integer bytes for a byte sequence and `C061` yields
    /// one-scalar Strings for MutableString, so the element kind is decided
    /// when the cursor is made rather than on every advance.
    values: Vec<Value>,
    source: Option<iris_runtime::ByteArrayRef>,
    /// A MutableString source, whose `C061` content version is separate.
    text_source: Option<iris_runtime::MutableStringRef>,
    position: usize,
    expected_version: u64,
    fail_fast: bool,
}

/// A live cursor over a shared Hash body.
///
/// `IRIS-V1-COLLECTIONS-C034` makes traversal fail-fast for STRUCTURAL change
/// only, so the cursor compares the structural version and an ordinary value
/// update does not disturb it.
///
/// The keys are snapshotted at `iterator()` time to give the cursor a stable
/// walk order, which `C033` leaves unspecified but which must at least not
/// revisit or skip entries within one traversal. Each key is re-read from the
/// live body when yielded, so `C035`'s "an entry not yet yielded observes the
/// latest value" holds.
#[derive(Debug)]
struct HashCursor {
    entries: HashRef,
    keys: Vec<Value>,
    position: usize,
    expected_version: u64,
    /// The key most recently yielded, which `C036` `remove_current` removes.
    yielded: Option<Value>,
    /// Whether the most recent yield has already been consumed by a removal,
    /// which `C036` permits at most once per successful yield.
    removed_current: bool,
}

/// Renders a String as a JSON string literal.
fn render_json_text(value: &str) -> String {
    let mut rendered = String::from("\"");
    for scalar in value.chars() {
        match scalar {
            '"' => rendered.push_str("\\\""),
            '\\' => rendered.push_str("\\\\"),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            scalar => rendered.push(scalar),
        }
    }
    rendered.push('"');
    rendered
}

/// Applies one `IRIS-V1-COLLECTIONS-C040` Array growth operation in place.
///
/// `C024` names append and insert the EXPLICIT growth operations. `insert`
/// resolves its position through `C009` negative indexing and accepts the end
/// position so an element can be added after the last one. Each answers `nil`.
fn apply_array_mutation(
    values: &mut Vec<Value>,
    selector: &str,
    arguments: &[Value],
) -> Result<Value, EvaluationError> {
    match (selector, arguments) {
        ("append", [value]) => values.push(value.clone()),
        ("clear", []) => values.clear(),
        // `delete` removes the FIRST element equal to the argument. C026 fixes
        // Array equality as the in-order element comparison, which is what the
        // derived value equality already performs.
        ("delete", [target]) => {
            if let Some(position) = values.iter().position(|held| held == target) {
                values.remove(position);
            }
        }
        ("insert", [Value::Integer(index), value]) => {
            // The end position is a valid insertion point, so the length itself
            // resolves even though it is out of range for a READ.
            let position = resolve_index(index, values.len()).ok_or(EvaluationError::IndexError)?;
            if position > values.len() {
                return Err(EvaluationError::IndexError);
            }
            values.insert(position, value.clone());
        }
        _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity)),
    }
    Ok(Value::Nil)
}

/// Resolves an `IRIS-V1-COLLECTIONS-C010` slice span against a receiver length.
///
/// Slicing is unit-forward: negative endpoints resolve in the receiver's unit
/// through `C009`, effective bounds are CLAMPED rather than raising, and an
/// inverted span yields the empty slice.
/// Rejects a `IRIS-V1-COLLECTIONS-C046` stepped or reverse slice.
///
/// Slicing is UNIT-FORWARD, so a Range carrying any step other than +1 cannot
/// describe one and raises `ArgumentError` rather than silently slicing as if
/// the step were absent.
fn check_slice_range(range: &iris_runtime::RangeValue) -> Result<(), EvaluationError> {
    let Some(step) = range.step.to_i128() else {
        return Err(EvaluationError::ArgumentError);
    };
    if step == 1 {
        return Ok(());
    }
    // C046 answers the EMPTY slice for a start-after-end span, and C038 infers
    // step -1 for exactly that literal, so a descending literal is a legal
    // empty slice rather than the reverse slicing this rejects. Only a step
    // that could not have been inferred, meaning an explicit `by`, is refused.
    let descending_literal = step == -1
        && iris_runtime::Numeric::compare(
            &iris_runtime::NumericValue::Integer(range.end.clone()),
            &iris_runtime::NumericValue::Integer(range.start.clone()),
        ) == Some(std::cmp::Ordering::Less);
    if descending_literal {
        return Ok(());
    }
    Err(EvaluationError::ArgumentError)
}

fn slice_bounds(
    start: &iris_runtime::IntegerValue,
    end: &iris_runtime::IntegerValue,
    inclusive: bool,
    length: usize,
) -> std::ops::Range<usize> {
    let from = resolve_index(start, length).unwrap_or(0).min(length);
    let resolved = resolve_index(end, length).unwrap_or(0);
    let to = if inclusive {
        resolved.saturating_add(1)
    } else {
        resolved
    }
    .min(length);
    from..to.max(from)
}

/// Whether a body contains a `yield`, making its callable a generator.
///
/// `IRIS-V1-GRAMMAR-C072` makes the presence of `yield` the thing that decides,
/// so a nested Closure's own `yield` belongs to that Closure and is not
/// counted here.
fn body_yields(body: &[Statement]) -> bool {
    fn in_expression(expression: &Expression) -> bool {
        match expression {
            Expression::Yield(_) => true,
            Expression::Await(operand)
            | Expression::Unary { operand, .. }
            | Expression::Grouped(operand) => in_expression(operand),
            Expression::Binary { left, right, .. } | Expression::Assignment { left, right, .. } => {
                in_expression(left) || in_expression(right)
            }
            Expression::Member { receiver, .. } => in_expression(receiver),
            Expression::Index { receiver, index } => {
                in_expression(receiver) || in_expression(index)
            }
            Expression::Call {
                callee, arguments, ..
            } => in_expression(callee) || arguments.iter().any(in_expression),
            Expression::Array(values) | Expression::Tuple(values) => {
                values.iter().any(in_expression)
            }
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                in_expression(condition)
                    || then_body.iter().any(in_statement)
                    || else_body.iter().flatten().any(in_statement)
            }
            _ => false,
        }
    }
    fn in_statement(statement: &Statement) -> bool {
        match statement {
            Statement::Expression(expression)
            | Statement::Binding {
                value: expression, ..
            } => in_expression(expression),
            Statement::Return(value) => value.as_ref().is_some_and(in_expression),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                in_expression(condition)
                    || then_body.iter().any(in_statement)
                    || else_body.iter().flatten().any(in_statement)
            }
            Statement::While {
                condition, body, ..
            } => in_expression(condition) || body.iter().any(in_statement),
            Statement::For { iterable, body, .. } => {
                in_expression(iterable) || body.iter().any(in_statement)
            }
            _ => false,
        }
    }
    body.iter().any(in_statement)
}

/// One live generator: its body, the locals it was invoked with, and progress.
#[derive(Clone, Debug)]
struct GeneratorBody {
    body: Vec<Statement>,
    locals: HashMap<String, Value>,
    receiver: Option<Value>,
    delivered: usize,
    finished: bool,
}

/// The suspension bookkeeping for one running generator body.
///
/// `IRIS-V1-GRAMMAR-C072` makes `next()` resume the body until the NEXT
/// `yield`. The body is re-entered from the top each time, so `resume_past`
/// records how many suspensions were already delivered and `seen` counts those
/// reached in the current run. A suspension whose index is below `resume_past`
/// is replayed silently; the first at or above it suspends again.
#[derive(Clone, Copy, Debug)]
struct GeneratorState {
    resume_past: usize,
    seen: usize,
}

/// One async body suspended on an incomplete Awaitable.
///
/// `IRIS-V1-ASYNC-C013` requires an incomplete await to register the current
/// continuation and suspend. This engine is stackless for async bodies exactly
/// as it is for generators: the body is re-entered from the top and awaits
/// below the delivered count replay their recorded values, so the recorded
/// answers are the continuation.
#[derive(Clone)]
struct SuspendedTask {
    body: Vec<Statement>,
    locals: HashMap<String, Value>,
    receiver: Option<Value>,
    /// Awaits already completed, whose values replay on re-entry.
    delivered: Vec<Value>,
    /// The Gate this body is currently suspended on.
    gate: iris_runtime::ObjectId,
}

pub(super) struct SourceEvaluator {
    runtime: Runtime,
    kernel: Kernel,
    /// Remaining evaluation steps before the run is abandoned.
    ///
    /// A conformance vector that fails to terminate would otherwise hang the
    /// whole suite with no diagnostic, so non-termination is turned into a
    /// reportable error rather than a stall. The budget bounds LOOP iterations
    /// and Method invocations, which are the only unbounded constructs.
    remaining_steps: u64,
    /// Current Method/Closure invocation depth.
    ///
    /// The step budget alone cannot stop unbounded RECURSION: each frame is a
    /// host stack frame, so the process aborts on stack overflow long before a
    /// step budget large enough for ordinary loops is exhausted. Depth is
    /// therefore bounded separately.
    invocation_depth: u32,
    /// Live Array cursors, keyed by the identity their Value carries.
    ///
    /// The cursor must ADVANCE across `next()` calls, so its position lives
    /// here rather than inside the Value, which is copied on every send.
    /// Live Array cursors: the shared body, the next position, and the
    /// `IRIS-V1-COLLECTIONS-C026` content version the cursor expects.
    array_iterators: HashMap<iris_runtime::ObjectId, ArrayCursor>,
    /// Live Hash cursors, keyed by the iterator's own identity.
    ///
    /// `C037` denies any hidden current-iterator context, so each cursor is
    /// reached only through the Iterator object that owns it.
    hash_iterators: HashMap<iris_runtime::ObjectId, HashCursor>,
    /// Revision-event subscribers, in subscription order.
    ///
    /// `C048` runs subscribers in subscription order for one event and commit,
    /// so this is a sequence rather than a set.
    revision_subscribers: Vec<RevisionSubscriber>,
    /// The commit id the next enqueued revision event carries.
    next_commit_id: u64,
    /// Subscriber failures recorded on the `C048` event-error channel.
    event_errors: Vec<Value>,
    /// The permitted change summary recorded for each commit.
    ///
    /// `IRIS-V1-ASYNC-C046` lets a payload identify targets and change
    /// summaries but forbids private bodies and raw private data, so only the
    /// target NAMES are recorded here.
    committed_targets: HashMap<u64, Vec<Value>>,
    /// Exception contexts discarded by a `finally` control transfer.
    ///
    /// `IRIS-V1-CONTROL-C063` records these ONLY in a protected diagnostic
    /// channel, so they are kept apart from cause and suppressed metadata and
    /// are never visible to an ordinary caller.
    discarded_contexts: Vec<Value>,
    /// Failed Tasks that no awaiter has observed yet.
    ///
    /// `IRIS-V1-ASYNC-C027` forbids an unobserved failed Task from disappearing
    /// silently, and `C028` forbids the report from marking the failure handled,
    /// so observation is recorded separately from the retained context.
    unobserved_failures: Vec<(iris_runtime::ObjectId, Value)>,
    /// Gates awaiting an external completion post.
    ///
    /// `IRIS-V1-ASYNC-C014` lets external IO or Host completions enter the
    /// scheduler in the order the Host POSTS them, so a Gate is the fixture's
    /// stand-in for that post. It is the only way to obtain a genuinely
    /// INCOMPLETE Awaitable, which `C013`'s suspension half needs.
    gates: HashMap<iris_runtime::ObjectId, Option<Value>>,
    /// Suspended async bodies, keyed by their Task identity.
    suspended: HashMap<iris_runtime::ObjectId, SuspendedTask>,
    /// Continuations made ready by a completion post, in `C014` FIFO order.
    ready: Vec<iris_runtime::ObjectId>,
    /// Awaits already delivered in the async body currently being replayed.
    async_replay: Option<GeneratorState>,
    /// The recorded await values for the body currently being replayed.
    replaying: Option<Vec<Value>>,
    /// Whether `C050` shutdown has closed revision-event delivery.
    revision_delivery_closed: bool,
    /// Retained audit history, in commit order.
    ///
    /// `C053` answers retained events and raises when a requested portion is
    /// unavailable, so a pruned commit is ABSENT here rather than recorded.
    audit_history: Vec<u64>,
    /// Live Bytes and ByteArray cursors.
    byte_iterators: HashMap<iris_runtime::ObjectId, ByteCursor>,
    /// Each live generator as its body, bound locals, receiver and progress.
    ///
    /// `IRIS-V1-GRAMMAR-C072` resumes a generator by re-entering its body, so
    /// what is retained is the body and how many suspensions were already
    /// delivered, NOT a captured native stack. `IRIS-V1-ASYNC-C011` leaves
    /// ordinary evaluation untouched, which is why nothing else changes.
    generators: HashMap<iris_runtime::ObjectId, GeneratorBody>,
    /// Each Task's completion, once it has one.
    ///
    /// `IRIS-V1-ASYNC-C012` starts an async body SYNCHRONOUSLY and runs it
    /// until it completes, raises, or reaches the first incomplete `await`.
    /// With no external IO in v1 there is nothing to be incomplete, so a Task
    /// created here is already complete and `C013` continues synchronously.
    tasks: HashMap<iris_runtime::ObjectId, Result<Value, Box<EvaluationError>>>,
    names: HashMap<String, Binding>,
    selectors: HashMap<String, Selector>,
    bodies: HashMap<u64, MethodDeclaration>,
    /// Declared runtime globals, keyed by `(package_id, name)`.
    ///
    /// `IRIS-V1-CONTROL-C013` makes `$name` a DECLARED cell: a missing one is
    /// an error rather than a fresh binding, so reads consult this map.
    ///
    /// `D-431` makes a global's TRUE identity `(package_id, $name)`, unique
    /// within a package and separately instantiated per runtime, with NO flat
    /// cross-package namespace and no auto-merge. Keying by name alone let two
    /// packages declaring the same `$name` share one cell, which is exactly
    /// the process-global storage `D-431` says does not exist.
    globals: HashMap<(String, String), Binding>,
    /// The package identity this program runs under.
    ///
    /// `IRIS-V1-META-C003` lets a manifestless local script run with
    /// runtime-local package identity only, and `IRIS-V1-IDENTITY-C030` makes
    /// that identity runtime-local rather than publishable.
    package: String,
    /// The Class object naming `ExceptionContext`, once source has named it.
    ///
    /// `IRIS-V1-CONTROL-C080` makes it a NAMEABLE built-in Class so the getter
    /// replacement `C065` and `D-143` already authorize has an entry point.
    /// It is created on first mention rather than at startup, so a program that
    /// never names it publishes no extra Class.
    exception_context_class: Option<ClassId>,
    /// Each permission the Host granted, as `(name, scope)`.
    ///
    /// `IRIS-V1-META-C102` grants `inspect` and `mutate` independently, and
    /// `C103` scopes each grant to a package or Class. `C104` stops a grant
    /// flowing through callers, stack frames or Host process privilege, so the
    /// set is a property of the RUNNING package alone.
    grants: Vec<(String, String)>,
    /// The audit artifact for the current package, as `(locator, digest, source)`.
    ///
    /// `IRIS-V1-META-C066` verifies the stored digest BEFORE reconstruction and
    /// publishes nothing on failure. `IRIS-V1-META-C126` scopes the digest to
    /// the artifact's SOURCE bytes, which is why a locator-only change
    /// preserves it.
    artifact: Option<(String, String, String)>,
    /// The current package's declared `version`, when its manifest states one.
    ///
    /// `IRIS-V1-META-C003` lists the field and `IRIS-V1-META-V420` reflects the
    /// resolved package identity.
    package_version: Option<String>,
    /// Each locked dependency as `(package_id, api_major, version, digest)`.
    ///
    /// `IRIS-V1-META-C006` requires an EXACT selection, and V420 reflects the
    /// selected dependency without any resolver fetch occurring.
    locked_dependencies: Vec<(String, u64, String, String)>,
    /// Members added programmatically or by a conditional body branch.
    ///
    /// `IRIS-V1-META-C046` makes such additions DYNAMIC-ONLY, and `C047`
    /// requires the dynamic-only status to be recorded rather than inferred.
    /// V345 and V427 observe the reflected status.
    dynamic_members: std::collections::HashSet<(ClassId, Selector)>,

    /// The current package's declared `api_major`.
    ///
    /// `D-242` makes major-version contract identity part of a named Contract
    /// Type's hash, so `pkg@1::C` and `pkg@2::C` are distinct Types. It
    /// defaults to 1 for source evaluated outside a package manifest.
    api_major: u64,
    /// Origin Class names already declared by the D-178 hoisting pass.
    hoisted_origins: Vec<String>,
    /// The declared Type of each stored-property slot, keyed by Class and slot.
    ///
    /// `IRIS-V1-RUNTIME-C065` makes stored-property storage TYPED, and `C161`
    /// makes `@name` that exact slot, so a raw write must meet the same C004
    /// contract the generated setter enforces. Without this, a Method body
    /// writing `@n` bypassed the property guard entirely.
    property_types: HashMap<(ClassId, Selector), iris_syntax::TypeExpression>,
    /// Class-level stored-property names declared on each Class.
    ///
    /// `IRIS-V1-TYPES-C064` puts this storage on the CLASS OBJECT, so `A.n`
    /// must resolve even though no instance Method named `n` exists. The
    /// declared names are recorded so a Class-object send can answer them
    /// rather than reporting a missing message.
    class_level_properties: HashMap<ClassId, Vec<Selector>>,
    /// Class-level property names declared `shared` on each Class.
    ///
    /// `IRIS-V1-TYPES-C064` puts a `shared class property` on the UNAPPLIED
    /// generic definition, so it is NOT reachable through a closed
    /// construction. `IRIS-V1-TYPES-V238` observes that access as a missing
    /// message rather than as a second slot.
    shared_class_properties: HashMap<ClassId, Vec<String>>,
    /// Deferred per-closed class-property initializers for a generic Class.
    ///
    /// `IRIS-V1-TYPES-C066` runs a per-closed class property initializer once
    /// when the CLOSED Class is first materialized, not once at the generic
    /// declaration. The initializer is therefore held until a construction
    /// requests it, and `materialized_constructions` records which closed
    /// constructions already ran so a repeat request does not rerun it.
    pending_class_properties: HashMap<ClassId, Vec<(String, Expression)>>,
    /// The closed constructions whose per-closed initializers already ran.
    materialized_constructions: std::collections::HashSet<(ClassId, Vec<ClassId>)>,
    closures: HashMap<iris_runtime::ObjectId, ClosureRecord>,
    next_closure: u64,
    contract_names: HashMap<String, iris_runtime::ContractId>,
    /// The capabilities each Contract's `meta deny` withholds.
    ///
    /// `IRIS-V1-META-C076` subtracts every Contract-required deny from a
    /// Class's effective capabilities, so a Contract's policy has to be
    /// reachable when a Class declaring `for` that Contract is published.
    contract_capabilities: HashMap<iris_runtime::ContractId, iris_runtime::MetaCapabilities>,
    /// The selectors each declared Contract REQUIRES.
    ///
    /// `IRIS-V1-TYPES-C047` lets one unqualified `impl` member satisfy every
    /// declared same-name requirement, so a view send needs to know what the
    /// Contract actually requires rather than treating any same-name Method as
    /// an accidental implementation.
    contract_requirements: HashMap<iris_runtime::ContractId, Vec<String>>,
    /// The parameter count each Contract requirement declares.
    ///
    /// `IRIS-V1-META-C022` validates the COMPLETE candidate before publishing,
    /// so an open transaction that would leave a declared Contract unsatisfied
    /// must be rejected at commit. Comparing arity needs the requirement's own
    /// arity, which the requirement NAMES alone do not carry.
    contract_requirement_arities: HashMap<(iris_runtime::ContractId, String), usize>,
    /// The return Type each Contract requirement declares, when it wrote one.
    ///
    /// `IRIS-V1-TYPES-C045` makes declared Contract conformance immutable for a
    /// revision's static spine and forbids metaprogramming from incompatibly
    /// replacing it, so validating a candidate needs the requirement's own
    /// return Type and not its arity alone.
    contract_requirement_returns:
        HashMap<(iris_runtime::ContractId, String), iris_syntax::TypeExpression>,
    /// The parameter Types each Contract requirement declares, when it wrote them.
    ///
    /// `D-173` makes the contract-visible member SIGNATURE part of the static
    /// spine, not its arity and return Type alone, so a `draw(String)` that
    /// satisfies a declared `draw(Integer)` is an incompatible replacement
    /// `IRIS-V1-TYPES-C045` forbids. V202 observes it through a mixed-in
    /// Module, where the arities match and only the parameter Type differs.
    contract_requirement_parameters:
        HashMap<(iris_runtime::ContractId, String), Vec<Option<iris_syntax::TypeExpression>>>,
    class_contracts: HashMap<ClassId, Vec<iris_runtime::ContractId>>,
    /// Each generic Class name paired with its parameters and `where` bounds.
    ///
    /// `IRIS-V1-TYPES-C067` validates every normalized constraint at closed
    /// generic MATERIALIZATION and raises `TypeContractError` on failure, so
    /// the bounds must be reachable when a construction is evaluated.
    generic_bounds: HashMap<String, (Vec<String>, Vec<iris_syntax::Constraint>)>,
    qualified_methods: HashMap<(ClassId, iris_runtime::ContractId, Selector), Method>,
    contract_parents: HashMap<iris_runtime::ContractId, Vec<iris_runtime::ContractId>>,
    next_contract: u64,
    module_names: HashMap<String, ModuleId>,
    module_classes: HashMap<ModuleId, ClassId>,
    /// The constants each Module declares, keyed by `(module, name)`.
    ///
    /// `D-432` puts a `const` in the qualified namespace of its declaring
    /// Module and resolves an unqualified name against the CURRENT module's
    /// declarations, so a constant is not an ordinary lexical binding and two
    /// Modules may declare the same name without collision.
    module_constants: HashMap<(ModuleId, String), Value>,
    /// Names an explicit import brought into scope.
    ///
    /// `D-432` makes an explicit import the THIRD resolution tier, so these
    /// are kept apart from lexical bindings and from module declarations and
    /// are consulted only after both.
    imported_names: HashMap<String, Value>,
    /// The `main` receiver Class each executable Module owns.
    module_mains: HashMap<ModuleId, ClassId>,
    /// The program text, used to convert a byte offset into a line and column.
    ///
    /// `IRIS-V1-CONTROL-C079` defines `SourceLocation` with a one-based line
    /// and column, which cannot be derived from the syntax tree alone.
    source: String,
    /// The path reported by every `SourceLocation`.
    source_path: String,
    module_methods: HashMap<(ModuleId, Selector), Method>,
    module_method_overrides: HashMap<iris_runtime::MethodId, bool>,
    property_methods: HashMap<iris_runtime::MethodId, bool>,
    class_mixins: HashMap<ClassId, Vec<ClassId>>,
    static_superclasses: HashMap<ClassId, Option<ClassId>>,
    lexical_class: Option<ClassId>,
    current_method: Option<Method>,
    /// The `main` Class whose top-level body is currently executing.
    ///
    /// `IRIS-V1-CONTROL-C012` makes a top-level `f()` a PRIVILEGED implicit
    /// send to `main`, so a private top-level helper is reachable from the
    /// Module body while an importer is still denied.
    module_body_main: Option<ClassId>,
    /// The Class a programmatic `Class#open` transaction is targeting.
    ///
    /// `IRIS-V1-META-C023` makes a structural meta message sent to the target
    /// reach the CURRENT candidate, so the block needs to know which target is
    /// open rather than inferring it from the receiver alone.
    open_target: Option<ClassId>,
    /// The generator body currently running, when one is.
    ///
    /// `IRIS-V1-GRAMMAR-C072` resumes a generator by re-entering its body and
    /// skipping the suspensions already delivered, so the state is a counter
    /// pair rather than a captured native stack.
    generator: Option<GeneratorState>,
    /// How many async bodies enclose the running code.
    ///
    /// `IRIS-V1-ASYNC-C050` refuses the Host drive surface inside one, since
    /// `C015` forbids an Iris source-level blocking wait.
    async_depth: usize,
    /// How many Closure bodies enclose the running code.
    closure_depth: usize,
    /// Every target joined to the current open transaction group.
    ///
    /// `IRIS-V1-META-C038` makes same-thread nested opens join the OUTERMOST
    /// group and requires all its candidates to publish together or all roll
    /// back, so the group's membership must be known at the outermost commit.
    open_group: Vec<ClassId>,
    /// The Classes declared with type parameters.
    ///
    /// `IRIS-V1-META-C033` forbids programmatic open from targeting a Contract
    /// or a closed generic Class, and v1 interns ONE Class per generic
    /// definition, so the definition itself is what must be refused. Reading
    /// this from the generic BOUNDS was wrong: those are recorded only when the
    /// declaration wrote a `where` clause.
    generic_definitions: Vec<ClassId>,
    active_exception: Option<Value>,
    active_context: Option<Value>,
    next_selector: u64,
    next_body: u64,
}

#[derive(Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

impl Binding {
    const fn new(value: Value, mutable: bool) -> Self {
        Self { value, mutable }
    }

    const fn immutable(value: Value) -> Self {
        Self::new(value, false)
    }

    fn value(&self) -> Value {
        self.value.clone()
    }

    fn assign(&mut self, value: Value) -> Result<Value, ()> {
        if self.mutable {
            self.value = value.clone();
            Ok(value)
        } else {
            Err(())
        }
    }
}

fn construction_error(error: iris_runtime::ConstructionError) -> EvaluationError {
    match error {
        iris_runtime::ConstructionError::Runtime(iris_runtime::ExecutionError::Raised(value)) => {
            EvaluationError::Raised(value)
        }
        error => EvaluationError::Construction(error),
    }
}

impl SourceEvaluator {
    /// Builds an evaluator running under one package identity.
    ///
    /// `IRIS-V1-META-C003` lets a manifestless local script run with
    /// runtime-local package identity only, which `IRIS-V1-IDENTITY-C030`
    /// keeps runtime-local rather than publishable. `D-431` then makes a
    /// global's identity `(package_id, $name)`.
    pub(super) fn new_in_package(package: &str) -> Result<Self, EvaluationError> {
        let mut runtime = Runtime::new();
        let kernel = Kernel::new(runtime.registry_mut()).map_err(EvaluationError::Runtime)?;
        Ok(Self {
            runtime,
            kernel,
            remaining_steps: STEP_BUDGET,
            invocation_depth: 0,
            array_iterators: HashMap::new(),
            hash_iterators: HashMap::new(),
            byte_iterators: HashMap::new(),
            committed_targets: HashMap::new(),
            discarded_contexts: Vec::new(),
            unobserved_failures: Vec::new(),
            gates: HashMap::new(),
            suspended: HashMap::new(),
            ready: Vec::new(),
            async_replay: None,
            replaying: None,
            revision_subscribers: Vec::new(),
            revision_delivery_closed: false,
            next_commit_id: 1,
            event_errors: Vec::new(),
            audit_history: Vec::new(),
            generators: HashMap::new(),
            tasks: HashMap::new(),
            names: HashMap::new(),
            selectors: HashMap::new(),
            bodies: HashMap::new(),
            globals: HashMap::new(),
            package: package.to_owned(),
            api_major: 1,
            exception_context_class: None,
            grants: Vec::new(),
            artifact: None,
            package_version: None,
            locked_dependencies: Vec::new(),
            dynamic_members: std::collections::HashSet::new(),
            hoisted_origins: Vec::new(),
            property_types: HashMap::new(),
            class_level_properties: HashMap::new(),
            shared_class_properties: HashMap::new(),
            closures: HashMap::new(),
            next_closure: 900_000,
            contract_names: HashMap::new(),
            contract_capabilities: HashMap::new(),
            contract_requirements: HashMap::new(),
            contract_requirement_arities: HashMap::new(),
            contract_requirement_returns: HashMap::new(),
            contract_requirement_parameters: HashMap::new(),
            class_contracts: HashMap::new(),
            generic_bounds: HashMap::new(),
            qualified_methods: HashMap::new(),
            contract_parents: HashMap::new(),
            next_contract: 0,
            module_names: HashMap::new(),
            pending_class_properties: HashMap::new(),
            materialized_constructions: std::collections::HashSet::new(),
            module_classes: HashMap::new(),
            module_constants: HashMap::new(),
            imported_names: HashMap::new(),
            module_mains: HashMap::new(),
            source: String::new(),
            // No file is involved when a program is evaluated from a string,
            // so the path names the source itself rather than inventing one.
            source_path: "<source>".to_owned(),
            module_methods: HashMap::new(),
            module_method_overrides: HashMap::new(),
            property_methods: HashMap::new(),
            class_mixins: HashMap::new(),
            static_superclasses: HashMap::new(),
            lexical_class: None,
            current_method: None,
            module_body_main: None,
            open_target: None,
            generator: None,
            async_depth: 0,
            closure_depth: 0,
            open_group: Vec::new(),
            generic_definitions: Vec::new(),
            active_exception: None,
            active_context: None,
            next_selector: 1_000,
            next_body: 10_000,
        })
    }

    pub(super) fn program(&mut self, program: &Program) -> Result<Value, EvaluationError> {
        let mut values = Vec::new();
        // D-178: declaration collection resolves the ORIGIN before the open
        // transaction, so an `open class A` may PRECEDE the `class A` it
        // reopens. Only the origin is hoisted; the reopen still runs at its own
        // position, which keeps the ordinary `class` then `open class` order
        // applying the transaction after the origin's members exist.
        let mut seen_reopen: Vec<&str> = Vec::new();
        for entry in &program.entries {
            let ProgramEntry::Declaration(iris_syntax::Declaration::Class(class)) = entry else {
                continue;
            };
            if class.reopen {
                seen_reopen.push(&class.name);
            } else if seen_reopen.contains(&class.name.as_str()) {
                // A reopen of this name came FIRST, so the origin is resolved
                // now and its own position is skipped below.
                self.class(class)?;
                self.hoisted_origins.push(class.name.clone());
            }
        }
        for entry in &program.entries {
            match entry {
                ProgramEntry::Declaration(iris_syntax::Declaration::Class(class)) => {
                    // An origin already run in the hoisting pass is not declared
                    // a second time.
                    if !class.reopen
                        && let Some(position) = self
                            .hoisted_origins
                            .iter()
                            .position(|name| *name == class.name)
                    {
                        self.hoisted_origins.remove(position);
                        continue;
                    }
                    self.class(class)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Module(module)) => {
                    self.module(module)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Contract(contract)) => {
                    self.contract(contract)?;
                }
                // C061 makes a Type alias a NAME for its target, not a new
                // nominal Type, so the alias binds to whatever the target
                // already resolves to and shares its identity.
                // D-432 makes an explicit import the THIRD resolution tier,
                // after lexical scope and the current module's declarations. A
                // `from M import K` therefore binds `K` to what Module `M`
                // declares, and loses to both earlier tiers.
                //
                // Only a Module already loaded into this runtime is resolvable:
                // a cross-PACKAGE target needs the manifest and dependency
                // machinery chapter 08 owns, which is not claimed here.
                ProgramEntry::Declaration(iris_syntax::Declaration::Import(import)) => {
                    self.import(import)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Export(export)) => {
                    if let iris_syntax::ExportDeclaration::Declaration(inner) = export.as_ref() {
                        match inner.as_ref() {
                            iris_syntax::Declaration::Class(class) => self.class(class)?,
                            iris_syntax::Declaration::Module(module) => {
                                self.module(module)?;
                            }
                            iris_syntax::Declaration::Contract(contract) => {
                                self.contract(contract)?;
                            }
                            _ => {}
                        }
                    }
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::TypeAlias(alias)) => {
                    if let iris_syntax::TypeExpression::Name(target)
                    | iris_syntax::TypeExpression::Generic { name: target, .. } = &alias.target
                        && let Some(class) = self.class_name(target)?
                    {
                        self.names
                            .insert(alias.name.clone(), Binding::immutable(Value::Class(class)));
                    }
                }
                ProgramEntry::Statement(statement) => {
                    let value = self.statement(statement, &HashMap::new(), None)?;
                    if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
                        values.push(value);
                    }
                }
            }
        }
        match values.as_slice() {
            [] => Err(EvaluationError::UnsupportedConstruct),
            [value] => Ok(value.clone()),
            _ => Ok(Value::Array(ArrayRef::new(values))),
        }
    }

    fn class(&mut self, declaration: &ClassDeclaration) -> Result<(), EvaluationError> {
        let superclass = match &declaration.extends {
            Some(iris_syntax::TypeExpression::Name(name)) => self.class_name(name)?,
            Some(_) => return Err(EvaluationError::UnsupportedConstruct),
            // A reopen keeps the superclass its origin declaration established;
            // only an origin declaration defaults to the C005 root.
            None if declaration.reopen => None,
            None => self.class_name("Object")?,
        };
        let mut mixins = Vec::new();
        let mut class_mixins = Vec::new();
        for mixin in &declaration.mixins {
            match &mixin.target {
                iris_syntax::TypeExpression::Name(name) => {
                    if let Some(module) = self.module_names.get(name) {
                        mixins.push(CompositionEdge::new(*module, mixin.private_access));
                    } else if let Some(class) = self.class_name(name)? {
                        class_mixins.push(class);
                    }
                }
                // D-219 reifies and interns a CLOSED Module Type such as
                // `Helpers<String>`. v1 interns one Module per generic
                // definition, so the closed construction composes that
                // definition rather than a second Module.
                iris_syntax::TypeExpression::Generic { name, .. } => {
                    let module = *self
                        .module_names
                        .get(name)
                        .ok_or(EvaluationError::UnsupportedConstruct)?;
                    mixins.push(CompositionEdge::new(module, mixin.private_access));
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        let class = if declaration.reopen {
            // C080 makes `ExceptionContext` a NAMEABLE built-in Class, so a
            // reopen of it resolves to the Class whose getters an ordinary
            // property read consults.
            let class = match self.class_name(&declaration.name)? {
                Some(class) => class,
                None if declaration.name == "ExceptionContext" => self.exception_context_class()?,
                None => return Err(EvaluationError::UnsupportedConstruct),
            };
            if self.is_builtin_class(class) && declaration.extends.is_some() {
                let replacement = self
                    .kernel
                    .class(iris_runtime::BuiltinClass::Nil)
                    .map_err(EvaluationError::Runtime)?;
                let mut candidate = self
                    .runtime
                    .registry_mut()
                    .open(class)
                    .map_err(EvaluationError::Class)?;
                candidate.replace_runtime_superclass(Some(replacement));
                self.runtime
                    .registry_mut()
                    .publish(candidate)
                    .map_err(EvaluationError::Class)?;
                return Ok(());
            }
            if superclass.is_some() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            if !mixins.is_empty() {
                // IRIS-V1-RUNTIME-C148 lets a stable built-in Class compose
                // Modules on open; only its superclass is protected, by C150.
                let mut candidate = self
                    .runtime
                    .registry_mut()
                    .open(class)
                    .map_err(EvaluationError::Class)?;
                for edge in &mixins {
                    candidate.add_module(edge.module());
                }
                self.runtime
                    .registry_mut()
                    .publish(candidate)
                    .map_err(EvaluationError::Class)?;
                // D-175 recomputes MRO and verifies declared Contract
                // requirements BEFORE commit, and any false static promise
                // preserves the Class. Composing on reopen published the new
                // MRO before the body ran, so a Module whose `draw(String)`
                // conflicts with the declared `draw(Integer)` slipped past the
                // body's own validation. V202 observes the refusal together
                // with the prior MRO and conformance staying published.
                if let Err(error) = self.validate_candidate_contracts(class) {
                    let mut rollback = self
                        .runtime
                        .registry_mut()
                        .open(class)
                        .map_err(EvaluationError::Class)?;
                    for edge in &mixins {
                        rollback.remove_module(edge.module());
                    }
                    self.runtime
                        .registry_mut()
                        .publish(rollback)
                        .map_err(EvaluationError::Class)?;
                    return Err(error);
                }
            }
            if !declaration.meta_deny.is_empty() {
                return Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
                    target: class,
                    operation: Capability::MethodSet,
                    policy_origin: iris_runtime::PolicyOrigin::Class(class),
                    reason: "open declarations cannot change MetaCapabilities policy",
                }));
            }
            // C044 records instance Contract conformance from the `for` list.
            // A reopen skipped it entirely, so `open class Integer for N` never
            // declared the Contract and no view over it could be constructed.
            for target in &declaration.implements {
                let iris_syntax::TypeExpression::Name(name) = target else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let contract = *self
                    .contract_names
                    .get(name)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.class_contracts
                    .entry(class)
                    .or_default()
                    .push(contract);
            }
            class
        } else {
            let mut capabilities = meta_capabilities(&declaration.meta_deny)?;
            // C076: a Class's effective capabilities subtract every
            // Contract-required deny as well as its own and its ancestors'.
            for target in &declaration.implements {
                if let iris_syntax::TypeExpression::Name(name) = target
                    && let Some(contract) = self.contract_names.get(name)
                    && let Some(policy) = self.contract_capabilities.get(contract)
                {
                    capabilities = capabilities.narrowed_by(*policy);
                }
            }
            self.validate_module_overrides(superclass, &mixins)?;
            let class = self
                .runtime
                .registry_mut()
                .define_class_with_capabilities_and_composition_edges(
                    StaticSpine::new(1),
                    superclass,
                    capabilities,
                    &mixins,
                )
                .map_err(EvaluationError::Class)?;
            let body = self.register_body(MethodDeclaration {
                is_async: false,
                decorators: Vec::new(),
                is_override: false,
                impl_contract: None,
                kind: MethodKind::Instance,
                selector: "to_bool".into(),
                type_parameters: Vec::new(),
                parameters: Vec::new(),
                return_type: None,
                visibility: iris_syntax::Visibility::Public,
                body: Some(vec![Statement::Expression(Expression::Literal(
                    "true".into(),
                ))]),
            });
            let selector = self.selector("to_bool");
            self.runtime
                .registry_mut()
                .publish_origin_method(class, selector, body, iris_runtime::Visibility::Public)
                .map_err(EvaluationError::Class)?;
            self.names.insert(
                declaration.name.clone(),
                Binding::immutable(Value::Class(class)),
            );
            self.static_superclasses.insert(class, superclass);
            self.class_mixins.insert(class, class_mixins);
            // IRIS-V1-TYPES-C044: only a Class declares instance Contract
            // conformance, and it must list each claimed Contract explicitly in
            // its header, so this records the `for` list rather than deriving it.
            let mut conformances = Vec::new();
            for target in &declaration.implements {
                let iris_syntax::TypeExpression::Name(name) = target else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                conformances.push(
                    *self
                        .contract_names
                        .get(name)
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                );
            }
            self.class_contracts.insert(class, conformances);
            // C033 refuses a programmatic open on a generic definition, so the
            // definition is recorded whether or not it wrote a `where` clause.
            if !declaration.parameters.is_empty() {
                self.generic_definitions.push(class);
            }
            // C067 validates every normalized `where` constraint at closed
            // generic materialization, so the bounds are recorded against the
            // declaration name the construction will use.
            if !declaration.constraints.is_empty() {
                self.generic_bounds.insert(
                    declaration.name.clone(),
                    (
                        declaration.parameters.clone(),
                        declaration.constraints.clone(),
                    ),
                );
            }
            class
        };
        // D-207 opens a generic definition by building a candidate definition
        // PLUS substituted candidate revisions for every already-interned
        // closed construction, validating all of them as one transaction and
        // rolling everything back on any closed failure. A reopen recorded no
        // bounds at all, so a `where` clause added on open was never validated
        // against the constructions that already exist. V234 observes the
        // refusal leaving neither definition nor either closed revision changed.
        if declaration.reopen && !declaration.constraints.is_empty() {
            let previous = self.generic_bounds.get(&declaration.name).cloned();
            self.generic_bounds.insert(
                declaration.name.clone(),
                (
                    declaration.parameters.clone(),
                    declaration.constraints.clone(),
                ),
            );
            let existing: Vec<Vec<ClassId>> = self
                .materialized_constructions
                .iter()
                .filter(|(target, _)| *target == class)
                .map(|(_, arguments)| arguments.clone())
                .collect();
            for arguments in existing {
                let written: Vec<iris_syntax::TypeExpression> = arguments
                    .iter()
                    .map(|argument| {
                        iris_syntax::TypeExpression::Name(self.class_source_name(*argument))
                    })
                    .collect();
                if let Err(error) = self.check_generic_bounds(&declaration.name, &written) {
                    // The whole open fails, so the definition's bounds revert
                    // and every closed revision is left exactly as published.
                    match previous {
                        Some(bounds) => {
                            self.generic_bounds.insert(declaration.name.clone(), bounds)
                        }
                        None => self.generic_bounds.remove(&declaration.name),
                    };
                    return Err(error);
                }
            }
        }
        let builtin = declaration.reopen && builtin(&declaration.name, &self.kernel).is_some();
        // C022 makes a Class body an executable construction transaction over a
        // CANDIDATE: success validates the complete candidate and publishes
        // atomically, failure publishes NOTHING from it. Each structural
        // operation used to publish its own revision, so a body that failed
        // halfway had already published its earlier members.
        //
        // A builtin reopen is excluded: its target is a kernel Class whose
        // members are installed outside this transaction model.
        let transactional = !builtin;
        if transactional {
            self.runtime
                .registry_mut()
                .begin_transaction(class)
                .map_err(EvaluationError::Class)?;
        }
        let outcome = self
            .class_body(class, builtin, declaration)
            // C022 validates the COMPLETE candidate before publishing, so a
            // body that would leave a declared Contract unsatisfied fails as a
            // transaction rather than committing and being caught later.
            .and_then(|()| self.validate_candidate_contracts(class));
        if transactional {
            match &outcome {
                Ok(()) => self
                    .runtime
                    .registry_mut()
                    .commit_transaction(class)
                    .map_err(EvaluationError::Class)?,
                // C034 rolls the candidate back on exception, validation error
                // or capability denial and publishes nothing.
                Err(_) => self.runtime.registry_mut().roll_back_transaction(class),
            }
        }
        if outcome.is_err() && !declaration.reopen {
            // IRIS-V1-RUNTIME-C024 requires a rejected declaration to publish no
            // Class. The name is bound before the body is validated, so an origin
            // declaration that fails validation must unbind it again.
            self.names.remove(&declaration.name);
        }
        outcome
    }

    fn class_body(
        &mut self,
        class: ClassId,
        builtin: bool,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        self.publish_decorators(class, &declaration.decorators)?;
        let mut declared_in_body: Vec<(MethodKind, Selector)> = Vec::new();
        // C022 makes a Class body an EXECUTABLE construction transaction, and
        // C027 makes its locals ordinary lexical locals that do NOT become
        // class state merely because the transaction commits. They therefore
        // live in a scope of their own rather than in `self.names`.
        let mut body_locals: HashMap<String, Value> = HashMap::new();
        // C022 makes a Class body an EXECUTABLE construction transaction, so a
        // conditional branch is ordinary body control flow. A branch is
        // flattened into the statement stream it guards, which is what lets
        // `if cond { self.define_method(...) }` stage into the same candidate
        // an unconditional call would. V345 and V427 observe the true branch.
        // Each entry carries whether it came from a conditional branch, since
        // C046 makes a CONDITIONAL body addition dynamic-only while an
        // unconditional one is ordinary declarative API. Flattening alone would
        // lose that distinction.
        let mut pending: Vec<(&Statement, bool)> =
            declaration.body.iter().rev().map(|s| (s, false)).collect();
        while let Some((statement, conditional)) = pending.pop() {
            if let Statement::If {
                condition,
                then_body,
                else_body,
            } = statement
            {
                // C046 makes a CONDITIONAL executable-body addition
                // dynamic-only, so members staged from a branch are recorded
                // as such before the branch runs.

                let taken = self.expression(condition, &body_locals, Some(Value::Class(class)))?;
                let chosen = if self.truthy(taken)? {
                    Some(then_body)
                } else {
                    else_body.as_ref()
                };
                if let Some(chosen) = chosen {
                    pending.extend(chosen.iter().rev().map(|s| (s, true)));
                }
                continue;
            }
            match statement {
                Statement::StoredProperty {
                    decorators,
                    shared,
                    class_level,
                    name,
                    annotation,
                    initializer,
                    ..
                } => {
                    if *class_level {
                        if *shared {
                            self.shared_class_properties
                                .entry(class)
                                .or_default()
                                .push(name.clone());
                        }
                        // C066 runs a PER-CLOSED initializer once when the
                        // closed Class is first materialized, so a generic
                        // definition's ordinary class property is held rather
                        // than evaluated here. A `shared` property belongs to
                        // the unapplied definition and still runs now.
                        if !*shared && !declaration.parameters.is_empty() {
                            self.pending_class_properties
                                .entry(class)
                                .or_default()
                                .push((name.clone(), initializer.clone()));
                            self.class_level_properties.entry(class).or_default();
                        } else {
                            // C064 puts class-level storage on the CLASS OBJECT,
                            // so its accessors are singleton Methods and its
                            // slot is a Class raw ivar.
                            self.class_level_property(class, name, initializer.clone())?;
                        }
                    } else {
                        self.stored_property(
                            class,
                            builtin,
                            decorators,
                            name,
                            annotation,
                            initializer.clone(),
                        )?;
                    }
                }
                Statement::SharedBinding {
                    mutable,
                    name,
                    value,
                    ..
                } => self.shared_binding(class, *mutable, name, value)?,
                Statement::Method(method) => {
                    let selector = self.selector(&method.selector);
                    // IRIS-V1-RUNTIME-C024 forbids an overload set: one complete
                    // selector maps to at most one Method per revision, and a
                    // redefinition needs `override`. An origin declaration passes
                    // `requires_override` as false, so a duplicate inside ONE body
                    // would otherwise publish silently with the second body winning.
                    let duplicate = declared_in_body
                        .iter()
                        .any(|(kind, seen)| *kind == method.kind && *seen == selector);
                    self.class_method(class, builtin, declaration.reopen || duplicate, method)?;
                    declared_in_body.push((method.kind, selector));
                }
                // C022 lets a Class body run ordinary synchronous control flow
                // and use lexical locals. C025 keeps such a statement from
                // implicitly defining a Method, property or storage slot, so it
                // is evaluated for its effect and its binding stays local.
                Statement::Binding { name, value, .. } => {
                    let value = self.expression(value, &body_locals, Some(Value::Class(class)))?;
                    body_locals.insert(name.clone(), value);
                }
                Statement::Expression(expression) => {
                    // C023 makes a structural meta message sent to `self`, such
                    // as `define_method`, target the current transaction
                    // CANDIDATE rather than the published active revision.
                    if let Some(defined) = self.candidate_define_method(class, expression)? {
                        declared_in_body.push((MethodKind::Instance, defined));
                        // C046: an addition made from a conditional branch is
                        // dynamic-only, and C047 requires that status to be
                        // RECORDED rather than re-derived later.
                        if conditional {
                            self.dynamic_members.insert((class, defined));
                        }
                        continue;
                    }
                    self.expression(expression, &body_locals, Some(Value::Class(class)))?;
                }
                // C022 lets a Class body run ordinary synchronous control flow,
                // which includes `raise`. C034 then rolls the candidate back
                // and C024 leaves the Class unpublished, which is what
                // IRIS-V1-META-V341 observes.
                Statement::Raise(_) => {
                    self.statement(statement, &body_locals, Some(Value::Class(class)))?;
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        Ok(())
    }

    fn shared_binding(
        &mut self,
        class: ClassId,
        mutable: bool,
        name: &str,
        value: &Expression,
    ) -> Result<(), EvaluationError> {
        let selector = self.selector(name);
        if self.class_var_owner_from(class, selector).is_ok() {
            return Err(EvaluationError::Class(
                iris_runtime::ClassError::DuplicateClassVariable {
                    class,
                    name: selector,
                },
            ));
        }
        let value = self.expression(value, &HashMap::new(), None)?;
        self.runtime
            .declare_class_var(class, selector, value, mutable)
            .map_err(EvaluationError::Construction)?;
        Ok(())
    }

    fn class_method(
        &mut self,
        class: ClassId,
        builtin: bool,
        requires_override: bool,
        method: &MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        let selector = self.selector(&method.selector);
        // IRIS-V1-TYPES-C048: `impl C::member` qualifies a slot in the
        // Contract-qualified namespace. C049 keeps that namespace separate from
        // ordinary dispatch, so this must NOT publish into the ordinary table.
        if let Some(Some(contract)) = &method.impl_contract {
            let contract = *self
                .contract_names
                .get(contract)
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            if !self
                .class_contracts
                .get(&class)
                .is_some_and(|declared| declared.contains(&contract))
            {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            let body = self.register_body(method.clone());
            let qualified = Method::new(
                iris_runtime::MethodId::new(self.next_body),
                iris_runtime::MethodOwner::Class(class),
                selector,
                body,
                iris_runtime::Visibility::Public,
            );
            self.next_body += 1;
            self.qualified_methods
                .insert((class, contract, selector), qualified);
            return Ok(());
        }
        let replaces = self.replaces_method(class, builtin, method.kind, selector)?;
        if method.is_override && !replaces {
            return Err(EvaluationError::Class(ClassError::OverrideWithoutTarget {
                class,
                selector,
            }));
        }
        // C046 writes BOTH modifiers only when the declaration also replaces an
        // INHERITED or Module Method. A member marked `impl` that replaces the
        // Class's own Contract implementation satisfies the same requirement it
        // did before, so `impl` alone suffices and V436's compatible body-only
        // replacement publishes.
        let satisfies_contract = method.impl_contract.is_some();
        if requires_override && replaces && !method.is_override && !satisfies_contract {
            return Err(EvaluationError::Class(ClassError::OverrideRequired {
                class,
                selector,
            }));
        }
        let body = self.register_body(method.clone());
        let decorators = self.decorator_transforms(&method.decorators);
        match method.kind {
            MethodKind::Instance => {
                let defined = self
                    .runtime
                    .registry_mut()
                    .publish_decorated_method(class, selector, body, visibility(method), decorators)
                    .map_err(EvaluationError::Class)?;
                self.property_methods.insert(defined.id(), false);
            }
            MethodKind::Class => {
                self.runtime
                    .registry_mut()
                    .publish_singleton_method(class, selector, body, visibility(method))
                    .map_err(EvaluationError::Class)?;
            }
            MethodKind::Property => {
                let method_visibility = visibility(method);
                let defined = if builtin && self.builtin_class_property(&method.selector) {
                    self.runtime
                        .registry_mut()
                        .publish_singleton_method(class, selector, body, method_visibility)
                        .map_err(EvaluationError::Class)?
                } else {
                    self.runtime
                        .registry_mut()
                        .publish_decorated_method(
                            class,
                            selector,
                            body,
                            method_visibility,
                            decorators,
                        )
                        .map_err(EvaluationError::Class)?
                };
                self.property_methods.insert(defined.id(), true);
            }
            MethodKind::Module => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    fn replaces_method(
        &mut self,
        class: ClassId,
        builtin: bool,
        kind: MethodKind,
        selector: Selector,
    ) -> Result<bool, EvaluationError> {
        // C035 lets a body read its OWN candidate after writes, so a Method
        // staged earlier in the same body counts as being replaced even though
        // the transaction has published nothing yet.
        if matches!(kind, MethodKind::Instance | MethodKind::Property)
            && self.runtime.registry().staged_has_method(class, selector)
        {
            return Ok(true);
        }
        let result = match (builtin, kind) {
            (true, MethodKind::Class) => self
                .runtime
                .registry_mut()
                .dispatch_class_object(class, selector),
            (false, MethodKind::Class) => self
                .runtime
                .registry()
                .dispatch_class_object(class, selector),
            (true, MethodKind::Instance | MethodKind::Property) => {
                self.runtime.registry_mut().dispatch(class, selector)
            }
            (false, MethodKind::Instance | MethodKind::Property) => {
                self.runtime.registry().dispatch(class, selector)
            }
            (_, MethodKind::Module) => return Err(EvaluationError::UnsupportedConstruct),
        };
        match result {
            Ok(DispatchOutcome::Invoke(_)) | Err(DispatchError::VisibilityDenied { .. }) => {
                Ok(true)
            }
            Ok(DispatchOutcome::WouldInvokeMethodMissing { .. }) => Ok(false),
            Err(DispatchError::Class(error)) => Err(EvaluationError::Class(error)),
            Err(_) => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    /// Switches the evaluator to another package identity.
    ///
    /// `D-431` scopes a global to `(package_id, $name)`, so running a second
    /// package's program against the same runtime must change which package
    /// its `global` declarations and `$name` reads belong to.
    pub(super) fn enter_package(&mut self, package: &str, source: &str) {
        self.package = package.to_owned();
        self.source = source.to_owned();
    }

    /// Records the current package's declared `api_major`.
    ///
    /// `D-242` derives a named Contract Type's hash partly from major-version
    /// contract identity, so a package loaded at a different major produces
    /// different Contract Type hashes, which is what V260 observes.
    pub(super) fn enter_api_major(&mut self, api_major: u64) {
        self.api_major = api_major;
    }

    /// Records the resolved package identity and its locked dependencies.
    ///
    /// `IRIS-V1-META-C006` makes the dependency selection EXACT, so what a lock
    /// file already resolved is carried in rather than re-resolved. V420
    /// reflects both without any resolver fetch occurring.
    pub(super) fn enter_package_resolution(
        &mut self,
        version: Option<String>,
        locked: Vec<(String, u64, String, String)>,
    ) {
        self.package_version = version;
        self.locked_dependencies = locked;
    }

    /// Records the audit artifact `IRIS-V1-META-C066` resolves for a rollback.
    pub(super) fn enter_artifact(&mut self, artifact: Option<(String, String, String)>) {
        self.artifact = artifact;
    }

    /// Records the Host grants in force for the running package.
    pub(super) fn enter_grants(&mut self, grants: Vec<(String, String)>) {
        self.grants = grants;
    }

    /// Rejects a reflection call the running package was not granted.
    ///
    /// `IRIS-V1-META-C102` grants `inspect` and `mutate` independently, `C103`
    /// checks the granted scope against the TARGET, and `C104` stops a grant
    /// flowing through callers, receiver owners, stack frames or Host process
    /// privilege. `IRIS-V1-META-V421` observes inspect working only for the
    /// scoped Class, and `V363` observes an ungranted caller refused.
    fn require_reflection(
        &mut self,
        operation: &str,
        target: Option<&Value>,
    ) -> Result<(), EvaluationError> {
        let class = match target {
            Some(Value::Object(object)) => self.runtime.class_of(*object).ok(),
            Some(Value::Class(class)) => Some(*class),
            _ => None,
        };
        if self.reflection_granted(operation, class) {
            return Ok(());
        }
        Err(EvaluationError::ReflectionAccess)
    }

    /// Whether a reflection operation is granted for `target`.
    ///
    /// `IRIS-V1-META-C102` grants `inspect` and `mutate` independently, so
    /// neither implies the other. `C103` checks the granted SCOPE against the
    /// target, and `C104` stops the grant flowing from anywhere else, so only
    /// the running package's own grants are consulted.
    ///
    /// A fixture declaring NO grants at all is ungated, which keeps every row
    /// authored before Host grants existed behaving exactly as it did.
    fn reflection_granted(&self, operation: &str, target: Option<ClassId>) -> bool {
        if self.grants.is_empty() {
            return true;
        }
        let requested = format!("reflection.{operation}");
        self.grants.iter().any(|(name, scope)| {
            if *name != requested {
                return false;
            }
            if scope.is_empty() {
                return true;
            }
            // C103 scopes a grant to selected package IDs or Classes, so a
            // scoped grant matches only the named target.
            let named = scope.rsplit("::").next().unwrap_or(scope);
            target.is_some_and(|class| {
                self.names.iter().any(|(bound, binding)| {
                    bound == named && matches!(binding.value, Value::Class(known) if known == class)
                })
            })
        })
    }

    /// Reports whether the published revision of a named Class holds a slot.
    ///
    /// `IRIS-V1-META-C022` publishes nothing from a failed candidate, so a
    /// member staged by a failed body must NOT be dispatchable afterwards.
    pub(super) fn class_responds_to(&mut self, name: &str, selector: &str) -> bool {
        let Ok(Some(class)) = self.class_name(name) else {
            return false;
        };
        let selector = self.selector(selector);
        // The published REVISION is what a failed transaction must not have
        // touched. A dispatch probe would answer true through `method_missing`
        // for any name at all, so it cannot tell a leak from an absence.
        self.runtime
            .registry()
            .active(class)
            .is_ok_and(|revision| revision.methods().contains_key(&selector))
    }

    /// Validates a candidate against every Contract its Class declares.
    ///
    /// `IRIS-V1-META-C022` validates the complete candidate and publishes
    /// nothing on failure, and `IRIS-V1-TYPES-C006` reports a Contract failure
    /// as `TypeContractError`. A member staged earlier in the same body is
    /// therefore discarded along with the one that broke the Contract, which is
    /// what `IRIS-V1-TYPES-V206` observes.
    fn validate_candidate_contracts(&mut self, class: ClassId) -> Result<(), EvaluationError> {
        let Some(contracts) = self.class_contracts.get(&class).cloned() else {
            return Ok(());
        };
        for contract in contracts {
            let Some(requirements) = self.contract_requirements.get(&contract).cloned() else {
                continue;
            };
            for requirement in requirements {
                let Some(required) = self
                    .contract_requirement_arities
                    .get(&(contract, requirement.clone()))
                    .copied()
                else {
                    continue;
                };
                let selector = self.selector(&requirement);
                let Some(declaration) = self.candidate_method_declaration(class, selector) else {
                    continue;
                };
                if declaration.parameters.len() != required {
                    return Err(EvaluationError::TypeContractError);
                }
                // C045 forbids replacing a declared Contract fact with an
                // incompatible one, so a candidate that rewrites the
                // Contract-VISIBLE return Type is rejected before commit.
                // Comparing arity alone let `draw() -> Integer` replace
                // `draw() -> String` silently, which V203 observes.
                let stated = self
                    .contract_requirement_returns
                    .get(&(contract, requirement.clone()));
                if let (Some(required), Some(actual)) = (stated, declaration.return_type.as_ref())
                    && required != actual
                {
                    return Err(EvaluationError::TypeContractError);
                }
                // D-173 puts the contract-visible SIGNATURE in the static
                // spine, so a member whose parameter Type differs from the
                // requirement is an incompatible replacement even when the
                // arities agree. An unannotated position states nothing and is
                // left alone rather than treated as a mismatch.
                if let Some(required) = self
                    .contract_requirement_parameters
                    .get(&(contract, requirement.clone()))
                    && required.len() == declaration.parameters.len()
                    && required.iter().zip(&declaration.parameters).any(
                        |(required, actual)| match (required, actual.annotation.as_ref()) {
                            (Some(required), Some(actual)) => required != actual,
                            _ => false,
                        },
                    )
                {
                    return Err(EvaluationError::TypeContractError);
                }
            }
        }
        Ok(())
    }

    /// The declaration of a Method staged or published for `class`.
    ///
    /// `IRIS-V1-META-C035` lets a transaction read its OWN candidate after
    /// writes, so a Method staged earlier in the same body is validated rather
    /// than the published revision it will replace.
    fn candidate_method_declaration(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Option<&MethodDeclaration> {
        let registry = self.runtime.registry();
        let method = registry
            .staged_method(class, selector)
            .or_else(|| {
                registry
                    .active(class)
                    .ok()?
                    .methods()
                    .get(&selector)
                    .copied()
            })
            // D-175 recomputes MRO and verifies declared Contract requirements
            // before commit, so a requirement satisfied by a MIXED-IN Module
            // member must be checked too. Reading only the Class's own method
            // table let a Module whose `draw(String)` conflicts with the
            // declared `draw(Integer)` compose silently, which V202 observes.
            .or_else(|| {
                registry
                    .active(class)
                    .ok()?
                    .mro()
                    .iter()
                    .find_map(|entry| match entry {
                        iris_runtime::MroEntry::Module(module) => registry
                            .module_method(*module, selector)
                            .map(|method| method.id()),
                        iris_runtime::MroEntry::Class(_) => None,
                    })
            })?;
        let method = registry.method_by_id(method)?;
        self.bodies.get(&method.body().raw())
    }

    /// Handles `self.define_method(:selector) { body }` inside a Class body.
    ///
    /// `IRIS-V1-META-C023` makes a structural meta message sent to `self`
    /// target the CURRENT transaction candidate, so the Method joins the
    /// candidate the body is accumulating and publishes with it.
    ///
    /// `IRIS-V1-META-C026` keeps the defined Method from closing over the
    /// body's transaction-temporary locals: its lexical environment is
    /// definition scope and its own parameters, not the body execution's
    /// locals. The Closure body is therefore installed as an ordinary Method
    /// body rather than as a capturing Closure.
    ///
    /// Returns the defined selector, or `None` when the expression is not this
    /// meta message and should be evaluated ordinarily.
    fn candidate_define_method(
        &mut self,
        class: ClassId,
        expression: &Expression,
    ) -> Result<Option<Selector>, EvaluationError> {
        let Expression::Call {
            callee, arguments, ..
        } = expression
        else {
            return Ok(None);
        };
        let Expression::Member { receiver, selector } = callee.as_ref() else {
            return Ok(None);
        };
        if selector != "define_method"
            || !matches!(receiver.as_ref(), Expression::Name(name) if name == "self")
        {
            return Ok(None);
        }
        let [
            Expression::Symbol(name),
            Expression::Closure { parameters, body },
        ] = arguments.as_slice()
        else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let declaration = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            impl_contract: None,
            kind: MethodKind::Instance,
            selector: name.clone(),
            type_parameters: Vec::new(),
            parameters: parameters
                .iter()
                .map(|parameter| iris_syntax::Parameter {
                    name: parameter.clone(),
                    category: iris_syntax::ParameterCategory::Positional,
                    annotation: None,
                    default: None,
                })
                .collect(),
            return_type: None,
            visibility: iris_syntax::Visibility::Public,
            body: Some(body.clone()),
            is_override: false,
        };
        let selector = self.selector(name);
        self.class_method(class, false, false, &declaration)?;
        Ok(Some(selector))
    }

    /// Mirrors a Method defined on a Module's `main` into the Module table.
    ///
    /// `IRIS-V1-TYPES-C074` keeps a Module body Method a Module member as well
    /// as a `main` Method, which is why a declared `fun` publishes both. A
    /// Method defined through `IRIS-V1-META-C023`'s `self.define_method` is the
    /// same kind of member and needs the same second copy.
    fn publish_module_copy(
        &mut self,
        module: ModuleId,
        main: ClassId,
        selector: Selector,
    ) -> Result<(), EvaluationError> {
        // A Module's `main` is not itself the body's transaction target, so the
        // Method may already sit in its published revision rather than in a
        // staged candidate. Both are consulted, since `C035` only guarantees
        // that a transaction can read its OWN candidate.
        let registry = self.runtime.registry();
        let Some(body) = registry
            .staged_method(main, selector)
            .or_else(|| {
                registry
                    .active(main)
                    .ok()?
                    .methods()
                    .get(&selector)
                    .copied()
            })
            .and_then(|method| registry.method_by_id(method))
            .map(|method| method.body())
        else {
            return Ok(());
        };
        let defined = self
            .runtime
            .registry_mut()
            .define_module_method(module, selector, body, iris_runtime::Visibility::Public)
            .map_err(EvaluationError::Class)?;
        self.module_methods.insert((module, selector), defined);
        self.module_method_overrides.insert(defined.id(), false);
        Ok(())
    }

    /// Sends a bare call to the Module whose Method is executing.
    ///
    /// `IRIS-V1-CONTROL-C012` makes a bare `f(...)` inside a Module a
    /// PRIVILEGED implicit send, and `IRIS-V1-META-C024` falls back to the
    /// current `self` or Module `main` receiver when no lexical binding
    /// matched. A Module name evaluates to a Symbol, so the evaluated receiver
    /// cannot serve that send.
    ///
    /// Returns `None` when no Module Method is executing or the Module declares
    /// no such member, so an ordinary send still applies.
    fn module_self_send(
        &mut self,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        let Some(MethodOwner::Module(module)) = self.current_method.map(|method| method.owner())
        else {
            return Ok(None);
        };
        let slot = self.selector(selector);
        let Some(method) = self.module_methods.get(&(module, slot)).copied() else {
            return Ok(None);
        };
        let main = self.module_main(module)?;
        let receiver = self.construct(main, &[])?;
        self.invoke_method(method, Value::Object(receiver), arguments)
            .map(Some)
    }

    /// Binds a Method of the Module whose Method is executing, for a bare name.
    ///
    /// `IRIS-V1-CONTROL-C014` makes reading an instance Method create a
    /// BoundMethod, and `IRIS-V1-META-C024` falls back to the current Module
    /// `main` receiver. A Module name evaluates to a Symbol, so the evaluated
    /// receiver cannot serve the read.
    fn module_bound_method(&mut self, name: &str) -> Option<Value> {
        let MethodOwner::Module(module) = self.current_method?.owner() else {
            return None;
        };
        let selector = self.selector(name);
        let method = self.module_methods.get(&(module, selector)).copied()?;
        let main = self.module_main(module).ok()?;
        let receiver = self.construct(main, &[]).ok()?;
        self.runtime
            .registry_mut()
            .bind_instance_with_context(
                receiver,
                main,
                selector,
                iris_runtime::DispatchContext::implementation(main, true),
            )
            .ok()
            .map(Value::BoundMethod)
            .or(Some(Value::Method(method)))
    }

    /// Runs a programmatic `Class#open` transaction.
    ///
    /// `IRIS-V1-META-C033` makes programmatic open the same transaction model
    /// the declarative `open class` uses, and accepts an alias, a reflection
    /// result, or a dynamically selected Class. `IRIS-V1-META-C034` commits on
    /// normal completion and rolls back the candidate on exception, validation
    /// error or capability denial.
    ///
    /// The block receives the target's stable logical identity, so structural
    /// meta messages sent to it reach the current candidate under
    /// `IRIS-V1-META-C023`.
    fn programmatic_open(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [Value::Closure(block)] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        // C033 forbids targeting a Contract or a closed generic Class, so a
        // generic definition is refused rather than opened per construction.
        if self.generic_definitions.contains(&class) {
            return Err(EvaluationError::ClosedGenericOpenForbidden);
        }
        let block = *block;
        self.runtime
            .registry_mut()
            .begin_transaction(class)
            .map_err(EvaluationError::Class)?;
        // C038 makes a same-thread NESTED open join the outermost transaction
        // group: only the outermost commits, and a reopen of a target already
        // in the group reuses that target's candidate. An inner open that
        // committed on its own left its target published even when the outer
        // transaction later failed.
        let outermost = self.open_target.is_none();
        self.open_group.push(class);
        let previous = self.open_target.replace(class);
        let outcome = self
            .invoke_closure(block, &[Value::Class(class)])
            // C022 validates the COMPLETE candidate before publishing, so the
            // same Contract check the declarative form runs applies here.
            .and_then(|value| self.validate_candidate_contracts(class).map(|()| value));
        self.open_target = previous;
        if !outermost {
            // An inner open NEVER commits independently, so its result travels
            // to the outermost transaction unchanged.
            return outcome;
        }
        let group = std::mem::take(&mut self.open_group);
        match outcome {
            Ok(value) => {
                // C038 validates every candidate in the group before any of
                // them publishes, so one failing target rolls back all.
                for target in &group {
                    if let Err(error) = self.validate_candidate_contracts(*target) {
                        self.runtime.registry_mut().roll_back_group();
                        return Err(error);
                    }
                }
                self.runtime
                    .registry_mut()
                    .commit_group()
                    .map_err(EvaluationError::Class)?;
                // C047 delivers revision events asynchronously AFTER the
                // structural commit returns, and forbids subscriber code from
                // running in the commit path, so the event is only ENQUEUED
                // here and delivered by the C049 flush surface.
                self.enqueue_revision_event(&group);
                Ok(value)
            }
            Err(error) => {
                self.runtime.registry_mut().roll_back_group();
                Err(error)
            }
        }
    }

    /// Defines a Method on a Class through the `define_method` meta message.
    ///
    /// `IRIS-V1-META-C023` makes this reach the CURRENT transaction candidate,
    /// so inside a `Class#open` block the Method joins that candidate and
    /// publishes with it. Outside one it is an ordinary structural change and
    /// publishes its own revision, which is the pre-existing reflective
    /// behaviour for a standalone call.
    ///
    /// `IRIS-V1-META-C026` keeps the Method from closing over the block's
    /// locals: the Closure body becomes an ordinary Method body rather than a
    /// capturing Closure.
    fn meta_define_method(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [Value::Symbol(name), Value::Closure(block)] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        // Only the shape of the Closure is needed: C026 keeps the defined
        // Method from closing over the block's captures, so the captured
        // environment is deliberately NOT carried over.
        let (parameters, body) = self
            .closures
            .get(block)
            .map(|record| (record.parameters.clone(), record.body.clone()))
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let declaration = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            impl_contract: None,
            kind: MethodKind::Instance,
            selector: name.clone(),
            type_parameters: Vec::new(),
            parameters: parameters
                .iter()
                .map(|parameter| iris_syntax::Parameter {
                    name: parameter.clone(),
                    category: iris_syntax::ParameterCategory::Positional,
                    annotation: None,
                    default: None,
                })
                .collect(),
            return_type: None,
            visibility: iris_syntax::Visibility::Public,
            body: Some(body),
            is_override: false,
        };
        self.class_method(class, false, false, &declaration)?;
        Ok(Value::Nil)
    }

    /// Reads the stored-property names visible to the current transaction.
    ///
    /// `IRIS-V1-META-C119` implements a reflection operation ONCE and gives
    /// `Class` the corresponding member by mixin, so `A.properties` and
    /// `Reflection::Class.properties(A)` are two entry points to this one
    /// implementation rather than two behaviours.
    fn class_properties(&mut self, class: ClassId) -> Result<Value, EvaluationError> {
        let selectors = self
            .runtime
            .registry()
            .visible_properties(class)
            .map_err(EvaluationError::Class)?;
        let names = selectors
            .into_iter()
            .map(|selector| {
                self.selectors
                    .iter()
                    .find_map(|(name, known)| (*known == selector).then(|| name.clone()))
                    .map_or(Value::Nil, Value::Symbol)
            })
            .collect();
        Ok(Value::Array(names))
    }

    /// Stages a stored property on the Class through a meta message.
    ///
    /// `IRIS-V1-META-C023` makes this reach the CURRENT transaction candidate,
    /// so `IRIS-V1-TYPES-V207`'s inner read sees the staged property while an
    /// external query still sees the old property set, and a rollback publishes
    /// none of it.
    fn meta_define_property(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [Value::Symbol(name), Value::Closure(block)] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let body = self
            .closures
            .get(block)
            .map(|record| record.body.clone())
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let declaration = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            impl_contract: None,
            kind: MethodKind::Instance,
            selector: name.clone(),
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            visibility: iris_syntax::Visibility::Public,
            body: Some(body),
            is_override: false,
        };
        let initializer = self.register_body(declaration);
        let selector = self.selector(&format!("@{name}"));
        self.runtime
            .registry_mut()
            .publish_stored_property(class, selector, initializer)
            .map_err(EvaluationError::Class)?;
        Ok(Value::Nil)
    }

    /// Names the Modules composed into `module`, in composition order.
    ///
    /// `IRIS-V1-META-V358` observes an EMPTY edge list for a Module written
    /// without `mixin`, so this reports exactly the recorded edges.
    fn module_component_names(&self, module: ModuleId) -> Vec<Value> {
        self.runtime
            .registry()
            .module_components(module)
            .into_iter()
            .map(|component| {
                self.module_names
                    .iter()
                    .find_map(|(name, known)| {
                        (*known == component).then(|| Value::Symbol(name.clone()))
                    })
                    .unwrap_or(Value::Nil)
            })
            .collect()
    }

    /// The Symbol a Selector spells.
    fn selector_symbol(&self, selector: Selector) -> Value {
        self.selectors
            .iter()
            .find_map(|(name, known)| (*known == selector).then(|| Value::Symbol(name.clone())))
            .unwrap_or(Value::Nil)
    }

    /// The Symbol a Module is named by.
    fn module_symbol(&self, module: ModuleId) -> Value {
        self.module_names
            .iter()
            .find_map(|(name, known)| (*known == module).then(|| Value::Symbol(name.clone())))
            .unwrap_or(Value::Nil)
    }

    /// Reports whether a Class declares a class-level stored property.
    fn is_class_level_property(&mut self, class: ClassId, selector: &str) -> bool {
        let slot = self.selector(selector);
        self.class_level_properties
            .get(&class)
            .is_some_and(|slots| slots.contains(&slot))
    }

    /// Installs a class-level stored property on the Class object.
    ///
    /// `IRIS-V1-TYPES-C064` puts this storage on the Class rather than on an
    /// instance, so the initializer is evaluated once at declaration and the
    /// slot is a Class raw ivar that `A.n` reads through a singleton accessor.
    fn class_level_property(
        &mut self,
        class: ClassId,
        name: &str,
        initializer: Expression,
    ) -> Result<(), EvaluationError> {
        let value = self.expression(&initializer, &HashMap::new(), Some(Value::Class(class)))?;
        let slot = self.selector(name);
        self.runtime
            .assign_class_raw_ivar(class, slot, value)
            .map_err(EvaluationError::Construction)?;
        self.class_level_properties
            .entry(class)
            .or_default()
            .push(slot);
        Ok(())
    }

    /// Runs the per-closed class property initializers for one construction.
    ///
    /// `IRIS-V1-TYPES-C066` runs each per-closed initializer ONCE when the
    /// closed Class is first materialized, inside that creation transaction.
    /// A repeat request for an already materialized construction reuses it and
    /// reruns nothing, which is what `IRIS-V1-TYPES-V240` observes.
    ///
    /// A failed materialization publishes NOTHING for that construction: the
    /// slots this call wrote are removed and the construction is not recorded,
    /// so a later request retries and reruns the initializers. External side
    /// effects the initializer already performed are NOT undone, which is what
    /// `IRIS-V1-TYPES-V241` observes.
    fn materialize_closed(
        &mut self,
        class: ClassId,
        arguments: &[iris_syntax::TypeExpression],
    ) -> Result<(), EvaluationError> {
        let mut normalized = Vec::new();
        for argument in arguments {
            let iris_syntax::TypeExpression::Name(argument) = argument else {
                return Ok(());
            };
            normalized.push(
                self.class_name(argument)?
                    .ok_or(EvaluationError::NameError)?,
            );
        }
        // D-207 revalidates every already-interned closed construction when the
        // definition is opened, so interning is recorded for EVERY closed
        // construction rather than only for one carrying per-closed
        // initializers. Recording it inside the pending-properties guard left a
        // property-less `Box<Integer>` invisible to that revalidation.
        let first = self
            .materialized_constructions
            .insert((class, normalized.clone()));
        let Some(pending) = self.pending_class_properties.get(&class).cloned() else {
            return Ok(());
        };
        if !first {
            return Ok(());
        }
        let mut suffix = String::new();
        for argument in &normalized {
            suffix.push_str(&format!("<{}>", argument.raw()));
        }
        let mut written = Vec::new();
        for (name, initializer) in pending {
            let outcome = self.expression(&initializer, &HashMap::new(), Some(Value::Class(class)));
            let value = match outcome {
                Ok(value) => value,
                Err(error) => {
                    // The candidate publishes nothing, so every slot this
                    // transaction already wrote is discarded and the
                    // construction is left unrecorded for a later retry.
                    for slot in written {
                        let _ = self.runtime.assign_class_raw_ivar(class, slot, Value::Nil);
                    }
                    self.materialized_constructions.remove(&(class, normalized));
                    // C097 reports an exception escaping a per-closed
                    // initializer as `TypeContractError`, matching how C067
                    // reports a constraint failure on the SAME materialization
                    // path. A control-flow unwind is not a failure and passes
                    // through unchanged.
                    return Err(match error {
                        // A control-flow unwind is not a materialization
                        // failure and travels to its own boundary unchanged.
                        error @ (EvaluationError::LoopContinue(_)
                        | EvaluationError::LoopBreak(..)
                        | EvaluationError::Return(_)) => error,
                        _ => EvaluationError::TypeContractError,
                    });
                }
            };
            let slot = self.selector(&format!("{name}{suffix}"));
            self.runtime
                .assign_class_raw_ivar(class, slot, value)
                .map_err(EvaluationError::Construction)?;
            written.push(slot);
            self.class_level_properties
                .entry(class)
                .or_default()
                .push(slot);
        }
        Ok(())
    }

    /// The declared Type of a stored-property slot on a receiver, if any.
    ///
    /// `IRIS-V1-RUNTIME-C066` makes slot identity `(receiver, name)` rather
    /// than the declaring Class, so a subclass Method writing an inherited
    /// slot targets the SAME slot and meets the same contract. The ancestry is
    /// therefore searched rather than the receiver's own Class alone.
    fn property_type(
        &self,
        object: iris_runtime::ObjectId,
        slot: Selector,
    ) -> Option<iris_syntax::TypeExpression> {
        let class = self.runtime.class_of(object).ok()?;
        if let Some(annotation) = self.property_types.get(&(class, slot)) {
            return Some(annotation.clone());
        }
        self.runtime
            .registry()
            .active(class)
            .ok()?
            .mro()
            .iter()
            .find_map(|entry| match entry {
                iris_runtime::MroEntry::Class(entry) => {
                    self.property_types.get(&(*entry, slot)).cloned()
                }
                iris_runtime::MroEntry::Module(_) => None,
            })
    }

    fn stored_property(
        &mut self,
        class: ClassId,
        builtin: bool,
        decorators: &[iris_syntax::Decorator],
        name: &str,
        annotation: &iris_syntax::TypeExpression,
        initializer: Expression,
    ) -> Result<(), EvaluationError> {
        let getter = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: name.into(),
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            visibility: iris_syntax::Visibility::Public,
            body: Some(vec![Statement::Expression(Expression::RawIvar(format!(
                "@{name}"
            )))]),
        };
        let setter = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: format!("{name}="),
            type_parameters: Vec::new(),
            return_type: None,
            parameters: vec![iris_syntax::Parameter {
                name: "value".into(),
                category: iris_syntax::ParameterCategory::Positional,
                // C004 guards a PROPERTY boundary, and every write to a stored
                // property goes through this synthesized setter. Carrying the
                // declared Type onto its parameter makes the existing parameter
                // guard enforce the property, so a violating value written from
                // a Method body is rejected instead of stored silently.
                annotation: Some(annotation.clone()),
                default: None,
            }],
            visibility: iris_syntax::Visibility::Public,
            body: Some(vec![Statement::Expression(Expression::Assignment {
                left: Box::new(Expression::RawIvar(format!("@{name}"))),
                operator: iris_syntax::AssignmentOperator::Assign,
                right: Box::new(Expression::Name("value".into())),
            })]),
        };
        let initializer = MethodDeclaration {
            is_async: false,
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: name.into(),
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            visibility: iris_syntax::Visibility::Private,
            body: Some(vec![Statement::Expression(Expression::Assignment {
                left: Box::new(Expression::RawIvar(format!("@{name}"))),
                operator: iris_syntax::AssignmentOperator::Assign,
                right: Box::new(initializer),
            })]),
        };
        self.class_method(class, builtin, false, &getter)?;
        self.class_method(class, builtin, false, &setter)?;
        let body = self.register_body(initializer);
        let property = self.selector(&format!("@{name}"));
        // C065 makes the storage TYPED and C161 makes `@name` that exact slot,
        // so the declared Type is recorded against the slot a raw write targets.
        // That is the SAME slot the registry publishes, now that source `@n`
        // keeps its sigil.
        self.property_types
            .insert((class, property), annotation.clone());
        let decorators = self.decorator_transforms(decorators);
        self.runtime
            .registry_mut()
            .publish_decorated_stored_property(class, property, body, decorators)
            .map_err(EvaluationError::Class)
    }

    fn publish_decorators(
        &mut self,
        class: ClassId,
        decorators: &[iris_syntax::Decorator],
    ) -> Result<(), EvaluationError> {
        if decorators.is_empty() {
            return Ok(());
        }
        let transforms = self.decorator_transforms(decorators);
        // C034 keeps one candidate per target for the transaction, so class
        // decorators join it rather than publishing their own revision and
        // leaving the body's staged candidate stale.
        self.runtime
            .registry_mut()
            .stage_decorators(class, transforms)
            .map_err(EvaluationError::Class)?;
        for decorator in decorators {
            self.run_decorator_transform(class, decorator)?;
        }
        Ok(())
    }

    /// Runs one decorator's RUNTIME transform phase against the target.
    ///
    /// `IRIS-V1-META-C124` gives the phase the signature
    /// `transform(declaration, arguments, context)`, and `IRIS-V1-META-C125`
    /// makes it return a `Transformation`. A decorator that declares no
    /// `transform` contributes only the static metadata already staged, so it
    /// is skipped rather than treated as an error.
    fn run_decorator_transform(
        &mut self,
        class: ClassId,
        decorator: &iris_syntax::Decorator,
    ) -> Result<(), EvaluationError> {
        let Some(decorator_class) = self.class_name(&decorator.name)? else {
            return Ok(());
        };
        let selector = self.selector("transform");
        let Ok(outcome) = self.runtime.registry().dispatch(decorator_class, selector) else {
            return Ok(());
        };
        let iris_runtime::DispatchOutcome::Invoke(method) = outcome else {
            return Ok(());
        };
        // C124 passes the target's C097 reflection view as `declaration` and
        // the application-site arguments as `arguments`. The context is the
        // third parameter and belongs to the runtime phase alone, since C088
        // makes the static phase pure.
        let mut arguments = Vec::new();
        for argument in &decorator.arguments {
            arguments.push(self.expression(argument, &HashMap::new(), None)?);
        }
        let receiver = Value::Object(self.construct(decorator_class, &[])?);
        // C022 makes the target's construction an executable transaction and
        // C037 makes a transaction body non-suspending, so the transform runs
        // WITH the target as the open transaction and an `await` inside it
        // raises MetaTransactionError. V431 observes that nothing publishes.
        let previous_open = self.open_target.replace(class);
        let produced = self.invoke_method(
            method,
            receiver,
            &[
                Value::Class(class),
                Value::Array(ArrayRef::new(arguments)),
                Value::Class(class),
            ],
        );
        self.open_target = previous_open;
        let produced = produced?;
        self.apply_transformation(class, produced)
    }

    /// Applies a `Transformation` a decorator's runtime phase returned.
    ///
    /// `IRIS-V1-META-C090` requires a generated Method to need the SAME
    /// `method_set` capability a handwritten declaration needs, so each staged
    /// Method is published through the ordinary capability-checked path rather
    /// than a privileged route of its own. A `MetaCapabilityError` therefore
    /// arises from the same check that governs source declarations.
    fn apply_transformation(
        &mut self,
        class: ClassId,
        produced: Value,
    ) -> Result<(), EvaluationError> {
        let Value::Transformation { staged, .. } = produced else {
            // C125 fixes `Transformation.empty` as the transformation of a
            // decorator that changes nothing, and a decorator returning
            // anything else contributes no candidate change here.
            return Ok(());
        };
        for (name, block) in staged {
            let (parameters, body) = self
                .closures
                .get(&block)
                .map(|record| (record.parameters.clone(), record.body.clone()))
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let declaration = MethodDeclaration {
                is_async: false,
                decorators: Vec::new(),
                impl_contract: None,
                kind: MethodKind::Instance,
                selector: name,
                type_parameters: Vec::new(),
                parameters: parameters
                    .iter()
                    .map(|parameter| iris_syntax::Parameter {
                        name: parameter.clone(),
                        category: iris_syntax::ParameterCategory::Positional,
                        annotation: None,
                        default: None,
                    })
                    .collect(),
                return_type: None,
                visibility: iris_syntax::Visibility::Public,
                body: Some(body),
                is_override: false,
            };
            self.class_method(class, false, false, &declaration)?;
        }
        Ok(())
    }

    fn decorator_transforms(
        &self,
        decorators: &[iris_syntax::Decorator],
    ) -> Vec<iris_runtime::DecoratorTransform> {
        decorators
            .iter()
            .map(|decorator| {
                iris_runtime::DecoratorTransform::metadata(
                    decorator.name.clone(),
                    decorator
                        .arguments
                        .iter()
                        .map(|argument| format!("{argument:?}")),
                )
            })
            .collect()
    }

    /// Registers a declared Contract as an object with its own identity.
    ///
    /// `IRIS-V1-TYPES-C043` forms Contract inheritance as the union of static
    /// requirements with no implementation MRO, `super`, stored state, or Method
    /// bodies, so parents are recorded as a plain relation rather than composed
    /// the way a Module is.
    fn contract(
        &mut self,
        declaration: &iris_syntax::ContractDeclaration,
    ) -> Result<(), EvaluationError> {
        let mut parents = Vec::new();
        for parent in &declaration.parents {
            let iris_syntax::TypeExpression::Name(name) = parent else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            parents.push(
                *self
                    .contract_names
                    .get(name)
                    .ok_or(EvaluationError::UnsupportedConstruct)?,
            );
        }
        let contract = iris_runtime::ContractId::new(self.next_contract);
        // C076 subtracts Contract-required denies from an implementing Class's
        // effective capabilities, so a Contract's own `meta deny` is recorded
        // rather than parsed and discarded.
        self.contract_capabilities
            .insert(contract, meta_capabilities(&declaration.meta_deny)?);
        self.next_contract += 1;
        self.contract_names
            .insert(declaration.name.clone(), contract);
        self.contract_parents.insert(contract, parents);
        // C062 makes a bodyless Method declaration the requirement form, so the
        // requirement names are exactly those members without a body.
        let requirements = declaration
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::Method(method) if method.body.is_none() => Some(method.selector.clone()),
                _ => None,
            })
            .collect();
        for statement in &declaration.body {
            if let Statement::Method(method) = statement
                && method.body.is_none()
            {
                self.contract_requirement_arities
                    .insert((contract, method.selector.clone()), method.parameters.len());
                self.contract_requirement_parameters.insert(
                    (contract, method.selector.clone()),
                    method
                        .parameters
                        .iter()
                        .map(|parameter| parameter.annotation.clone())
                        .collect(),
                );
                if let Some(returns) = &method.return_type {
                    self.contract_requirement_returns
                        .insert((contract, method.selector.clone()), returns.clone());
                }
            }
        }
        self.contract_requirements.insert(contract, requirements);
        self.names.insert(
            declaration.name.clone(),
            Binding::immutable(Value::Contract(contract)),
        );
        Ok(())
    }

    fn module(&mut self, declaration: &ModuleDeclaration) -> Result<(), EvaluationError> {
        let mut components = Vec::new();
        for mixin in &declaration.mixins {
            match &mixin.target {
                iris_syntax::TypeExpression::Name(name) => components.push(CompositionEdge::new(
                    *self
                        .module_names
                        .get(name)
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                    mixin.private_access,
                )),
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        let capabilities = meta_capabilities(&declaration.meta_deny)?;
        // `module_decl` admits `open`, and an open revision adds members to the
        // EXISTING Module rather than defining a second one. V416 loads an
        // origin and an open revision from two files of one package and calls
        // a member from each.
        let existing = declaration
            .reopen
            .then(|| self.module_names.get(&declaration.name).copied())
            .flatten();
        let (module, module_class) = match existing {
            Some(module) => {
                let module_class = *self
                    .module_classes
                    .get(&module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                (module, module_class)
            }
            None => {
                let module = self
                    .runtime
                    .registry_mut()
                    .define_module_with_composition_edges(&components, capabilities)
                    .map_err(EvaluationError::Class)?;
                self.module_names.insert(declaration.name.clone(), module);
                let module_class = self
                    .runtime
                    .registry_mut()
                    .define_class(StaticSpine::new(1), None)
                    .map_err(EvaluationError::Class)?;
                self.module_classes.insert(module, module_class);
                (module, module_class)
            }
        };
        // Declarations are published BEFORE any executable statement runs, so a
        // top-level `f()` can call a helper declared later in the same body.
        // A single ordered pass would evaluate the call against a `main` that
        // did not yet have the Method.
        for statement in &declaration.body {
            match statement {
                Statement::SharedBinding {
                    mutable,
                    name,
                    value,
                    ..
                } => self.shared_binding(module_class, *mutable, name, value)?,
                // Executable statements run in the second pass below. C012
                // makes a Module body the home of top-level executable code, so
                // a `raise` there is ordinary control flow rather than an
                // unsupported construct.
                Statement::Expression(_) | Statement::Binding { .. } | Statement::Raise(_) => {}
                // C064 puts a class-level property on the Module's own object.
                Statement::StoredProperty {
                    class_level,
                    name,
                    initializer,
                    ..
                } if *class_level => {
                    self.class_level_property(module_class, name, initializer.clone())?;
                }
                Statement::Method(method) => {
                    if method.kind == MethodKind::Class || method.kind == MethodKind::Property {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let body = self.register_body(method.clone());
                    let selector = self.selector(&method.selector);
                    let defined = self
                        .runtime
                        .registry_mut()
                        .define_module_method(module, selector, body, visibility(method))
                        .map_err(EvaluationError::Class)?;
                    self.module_methods.insert((module, selector), defined);
                    self.module_method_overrides
                        .insert(defined.id(), method.is_override);
                    // IRIS-V1-CONTROL-C012 and D-416: an unmodified `fun` in a
                    // Module body is ALSO top-level executable code, installed
                    // on that Module's `main` receiver and private by default,
                    // which is what a top-level `f()` sends to. C074 keeps it a
                    // Module instance Method as well, so this publishes a second
                    // copy rather than diverting the first.
                    if method.kind == MethodKind::Instance {
                        let main = self.module_main(module)?;
                        let declared = match method.visibility {
                            iris_syntax::Visibility::Public => iris_runtime::Visibility::Public,
                            _ => iris_runtime::Visibility::Private,
                        };
                        self.runtime
                            .registry_mut()
                            .publish_method(main, selector, body, declared)
                            .map_err(EvaluationError::Class)?;
                    }
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        // IRIS-V1-CONTROL-C012: top-level executable code runs against the
        // Module's `main`, and a top-level `f()` is a PRIVILEGED implicit send
        // to it, which is what reaches a private top-level helper an importer
        // could not call.
        let executable: Vec<&Statement> = declaration
            .body
            .iter()
            // C012 makes a Module body the home of top-level EXECUTABLE code,
            // which includes bindings. Collecting only expressions skipped
            // every `let`, so a helper could be called but never bound.
            .filter(|statement| {
                matches!(
                    statement,
                    Statement::Expression(_) | Statement::Binding { .. } | Statement::Raise(_)
                )
            })
            .collect();
        // C027 makes Class and Module body locals ORDINARY LEXICAL LOCALS that
        // do NOT become Module state merely because the body transaction
        // commits, and C026 keeps a Method declared in that body from closing
        // over them. A body `let` therefore lives only for the body execution:
        // leaving it behind let `IRIS-V1-META-V342`'s defined Method resolve
        // the transaction local `value` instead of the declared `value()`.
        //
        // A `const` is NOT included: D-432 makes it a DECLARATION of the
        // current module, which the pass below moves into the module namespace.
        let shadowed: Vec<(String, Option<Binding>)> = declaration
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::Binding {
                    constant: false,
                    name,
                    ..
                } => Some((name.clone(), self.names.get(name).cloned())),
                _ => None,
            })
            .collect();
        if !executable.is_empty() {
            let main = self.module_main(module)?;
            let receiver = self.construct(main, &[])?;
            let previous = self.module_body_main.replace(main);
            let result = executable.into_iter().try_for_each(|statement| {
                // C023 makes a structural meta message sent to `self` inside a
                // Module body target the current transaction candidate, and
                // C012 gives that body its Module's `main` receiver, so a
                // Method defined here joins `main` exactly as a top-level `fun`
                // does. Routing it through the ordinary statement path would
                // have evaluated it as a missing message on the receiver.
                if let Statement::Expression(expression) = statement
                    && let Some(selector) = self.candidate_define_method(main, expression)?
                {
                    // C074 keeps a Module body Method a Module member as well,
                    // so it enters the Module table exactly as a declared `fun`
                    // does. Publishing it on `main` alone left `M.answer()`
                    // reporting a missing message on Module.
                    self.publish_module_copy(module, main, selector)?;
                    return Ok(());
                }
                self.statement(statement, &HashMap::new(), Some(Value::Object(receiver)))
                    .map(|_| ())
            });
            self.module_body_main = previous;
            // The body's own locals are discarded whether it succeeded or
            // failed, and a name it shadowed is restored rather than deleted.
            for (name, outer) in shadowed {
                match outer {
                    Some(binding) => self.names.insert(name, binding),
                    None => self.names.remove(&name),
                };
            }
            // D-212 fails a Module whose initializer raises and leaves its
            // status `not_published`. The name is registered BEFORE the body
            // runs so a declaration can reach the Module being defined, so a
            // failed initializer must withdraw it rather than leave a
            // half-initialized Module observable. V239 observes that neither
            // shared property survives.
            if result.is_err() {
                self.module_names.remove(&declaration.name);
            }
            result?;
        }
        // D-432 makes a `const` a declaration of the CURRENT module rather than
        // an ordinary lexical binding, so it must not stay visible outside the
        // Module that declared it. Leaving it in the shared lexical scope let a
        // second Module declaring the same name overwrite the first, and let
        // code outside any Module read either one.
        for statement in &declaration.body {
            if let Statement::Binding {
                constant: true,
                name,
                ..
            } = statement
                && let Some(binding) = self.names.remove(name)
            {
                self.module_constants
                    .insert((module, name.clone()), binding.value());
            }
        }
        Ok(())
    }

    /// Binds the names an explicit import brings into scope.
    ///
    /// `D-432` orders unqualified resolution as lexical scope, then the current
    /// module or package declarations, then explicit imports. An import is
    /// therefore recorded in its own tier rather than as a lexical binding, so
    /// a later `let` of the same name still wins.
    fn import(&mut self, import: &iris_syntax::ImportDeclaration) -> Result<(), EvaluationError> {
        let Some(module) = self.module_names.get(&import.target).copied() else {
            // A target this runtime has not loaded needs the package subsystem
            // to resolve, so nothing is bound rather than a name being invented.
            return Ok(());
        };
        for spec in &import.specs {
            let Some(value) = self
                .module_constants
                .get(&(module, spec.name.clone()))
                .cloned()
            else {
                continue;
            };
            let local = spec.alias.clone().unwrap_or_else(|| spec.name.clone());
            self.imported_names.insert(local, value);
        }
        Ok(())
    }

    /// Reads a constant declared by the module whose code is executing.
    ///
    /// `D-432` resolves an unqualified name against lexical scope first, then
    /// the CURRENT module or package declarations, then explicit imports. A
    /// constant therefore belongs to its declaring Module rather than to the
    /// shared lexical scope, so it is looked up by the executing module.
    fn current_module_constant(&self, name: &str) -> Option<Value> {
        let module = match self.current_method.map(|method| method.owner()) {
            Some(MethodOwner::Module(module)) => Some(module),
            // A Module body executes AS its `main`, so no Method frame is
            // active and the executing module is found through that receiver.
            _ => self.module_body_main.and_then(|main| {
                self.module_mains
                    .iter()
                    .find_map(|(module, owner)| (*owner == main).then_some(*module))
            }),
        }?;
        self.module_constants
            .get(&(module, name.to_owned()))
            .cloned()
    }

    /// Returns the Class backing this Module's `main` receiver.
    ///
    /// `D-416` gives all source in one Module a SINGLE shared `main`, so this
    /// allocates once and reuses it rather than creating one per declaration.
    fn module_main(&mut self, module: ModuleId) -> Result<ClassId, EvaluationError> {
        if let Some(main) = self.module_mains.get(&module) {
            return Ok(*main);
        }
        let main = self
            .runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)
            .map_err(EvaluationError::Class)?;
        self.module_mains.insert(module, main);
        Ok(main)
    }

    /// Binds a top-level helper as a BoundMethod on the Module's `main`.
    ///
    /// `IRIS-V1-CONTROL-C013` resolves a bare name against visible declarations
    /// as well as lexical bindings, and `C014` makes reading a Method create a
    /// BoundMethod rather than exposing a Function kind. Only a Module body has
    /// such declarations in scope, which is what `module_body_main` records.
    fn main_bound_method(&mut self, name: &str, receiver: Option<&Value>) -> Option<Value> {
        let main = self.module_body_main?;
        let Some(Value::Object(object)) = receiver else {
            return None;
        };
        let selector = self.selector(name);
        // A Module body executes AS its `main`, so binding a helper that is
        // private by default uses the same privileged context an implicit send
        // does.
        let context = iris_runtime::DispatchContext::implementation(main, true);
        self.runtime
            .registry_mut()
            .bind_instance_with_context(*object, main, selector, context)
            .ok()
            .map(Value::BoundMethod)
    }

    /// Binds a Method of the RECEIVER's own Class for a bare name.
    ///
    /// `IRIS-V1-CONTROL-C014` makes reading an instance Method create a
    /// BoundMethod, and `IRIS-V1-META-C024` makes an unqualified name that
    /// matched no binding or declaration a PRIVILEGED send to the current
    /// `self`. The lookup is therefore privileged, so a Method that is private
    /// by default is reachable from its own Class exactly as an implicit send
    /// would reach it.
    fn receiver_bound_method(&mut self, name: &str, receiver: Option<&Value>) -> Option<Value> {
        let Some(Value::Object(object)) = receiver else {
            return None;
        };
        let object = *object;
        let class = self.runtime.class_of(object).ok()?;
        let selector = self.selector(name);
        let context = iris_runtime::DispatchContext::implementation(class, true);
        self.runtime
            .registry_mut()
            .bind_instance_with_context(object, class, selector, context)
            .ok()
            .map(Value::BoundMethod)
    }

    fn statement(
        &mut self,
        statement: &Statement,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match statement {
            Statement::Binding {
                mutable,
                name,
                annotation,
                value,
                ..
            } => {
                let value = self.expression(value, locals, receiver)?;
                if let Some(annotation) = annotation {
                    self.check_binding_annotation(&value, annotation)?;
                }
                self.names
                    .insert(name.clone(), Binding::new(value.clone(), *mutable));
                Ok(value)
            }
            // D-427 lets a typed `mut` defer initialization, and D-094 makes a
            // read before definite assignment an error rather than a nil read,
            // so no value is bound here.
            Statement::DeferredBinding { .. } => Ok(Value::Nil),
            Statement::Expression(expression) => self.expression(expression, locals, receiver),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                if self.condition(condition, locals, receiver.clone())? {
                    self.block(then_body, locals, receiver)
                } else if let Some(else_body) = else_body {
                    self.block(else_body, locals, receiver)
                } else {
                    Ok(Value::Nil)
                }
            }
            Statement::Raise(raise) => {
                // A bare `raise` has no `Raise` node, so its site offset is
                // unavailable; it records the continued context's own location.
                let raise_offset = raise.as_ref().map_or(0, |raise| raise.offset);
                let offset = raise_offset;
                let value = match raise {
                    Some(raise) => self.expression(&raise.value, locals, receiver.clone())?,
                    None => {
                        // IRIS-V1-CONTROL-C058: a bare `raise` continues the
                        // CURRENT propagation, so it keeps the active context
                        // rather than creating a fresh one. Outside a catch
                        // extent there is nothing to continue.
                        let Some(active) = self.active_exception.clone() else {
                            return Err(EvaluationError::NoActiveExceptionError);
                        };
                        // D-155: each bare `raise` APPENDS one re-raise site to
                        // the continued context, in occurrence order, without
                        // replacing the root stack or creating a fresh context.
                        if let Some(Value::ExceptionContext(
                            identity,
                            value,
                            cause,
                            suppressed,
                            mut sites,
                            location,
                        )) = self.active_context.take()
                        {
                            // C079: a `RaiseSite` records WHERE the bare raise
                            // continued, which is this statement rather than
                            // the original raise the context still points at.
                            let site = self.source_location(offset);
                            sites.push(Value::RaiseSite(Box::new(site)));
                            self.active_context = Some(Value::ExceptionContext(
                                identity, value, cause, suppressed, sites, location,
                            ));
                        }
                        return Err(EvaluationError::Raised(active));
                    }
                };
                // IRIS-V1-CONTROL-C057: an explicit cause MUST be an
                // ExceptionContext, and `from nil` suppresses chaining.
                let cause = match raise.as_ref().and_then(|raise| raise.cause.as_ref()) {
                    Some(cause) => {
                        let cause = self.expression(cause, locals, receiver)?;
                        match cause {
                            Value::Nil => cause,
                            Value::ExceptionContext(..) => {
                                // D-161: cause edges may not form a cycle. The
                                // check runs BEFORE linkage, so a rejected
                                // attempt leaves the existing graph unchanged.
                                if Self::context_reaches(&cause, &value) {
                                    return Err(EvaluationError::ExceptionChainError);
                                }
                                cause
                            }
                            _ => {
                                return Err(EvaluationError::Runtime(
                                    iris_runtime::KernelError::Type,
                                ));
                            }
                        }
                    }
                    None => Value::Nil,
                };
                let identity = self.next_context_identity();
                let location = self.source_location(raise_offset);
                self.active_context = Some(Value::ExceptionContext(
                    identity,
                    Box::new(value.clone()),
                    Box::new(cause),
                    Vec::new(),
                    Vec::new(),
                    Box::new(location),
                ));
                Err(EvaluationError::Raised(value))
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_statement(body, catches, finally, locals, receiver),
            Statement::Break { label, value } => {
                let value = match value {
                    Some(value) => self.expression(value, locals, receiver)?,
                    None => Value::Nil,
                };
                Err(EvaluationError::LoopBreak(label.clone(), value))
            }
            Statement::While {
                label,
                condition,
                body,
            } => self.while_statement(label.as_deref(), condition, body, locals, receiver),
            Statement::For {
                label,
                binding,
                iterable,
                body,
            } => self.for_statement(label.as_deref(), binding, iterable, body, locals, receiver),
            Statement::Continue(label) => Err(EvaluationError::LoopContinue(label.clone())),
            Statement::Match {
                subject,
                arms,
                fallback,
            } => self.match_statement(subject, arms, fallback.as_ref(), locals, receiver),
            // C013 makes `$name` reachable ONLY through a `global` declaration,
            // so the cell is created here and read by sigil name.
            Statement::GlobalBinding {
                mutable,
                name,
                annotation,
                value,
            } => {
                let value = self.expression(value, locals, receiver)?;
                if let Some(annotation) = annotation {
                    self.check_binding_annotation(&value, annotation)?;
                }
                let key = (self.package.clone(), name.clone());
                self.globals
                    .insert(key, Binding::new(value.clone(), *mutable));
                Ok(value)
            }
            Statement::SharedBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_) => Err(EvaluationError::UnsupportedConstruct),
            // D-421 unwinds to the NEAREST callable boundary, so this is a
            // control signal caught there rather than an ordinary value.
            Statement::Return(value) => {
                let value = match value {
                    Some(value) => self.expression(value, locals, receiver)?,
                    None => Value::Nil,
                };
                Err(EvaluationError::Return(value))
            }
        }
    }

    /// Runs `match value { arms }` per `IRIS-V1-CONTROL-C050`.
    ///
    /// The scrutinee is evaluated ONCE and arms are tested in source order, with
    /// no fallthrough: the first match produces the result. A guard runs only
    /// after structural success and after provisional bindings exist, per
    /// `IRIS-V1-CONTROL-C052`, and a false guard discards those bindings and
    /// continues with the next arm.
    fn match_statement(
        &mut self,
        subject: &Expression,
        arms: &[iris_syntax::MatchArm],
        fallback: Option<&iris_syntax::MatchBody>,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let value = self.expression(subject, locals, receiver.clone())?;
        for arm in arms {
            let mut bound = locals.clone();
            if !self.pattern_matches(&arm.pattern, &value, &mut bound)? {
                continue;
            }
            if let Some(guard) = &arm.guard {
                let test = self.expression(guard, &bound, receiver.clone())?;
                if !self.truthy(test)? {
                    continue;
                }
            }
            return self.match_body(&arm.body, &bound, receiver);
        }
        match fallback {
            Some(body) => self.match_body(body, locals, receiver),
            None => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn match_body(
        &mut self,
        body: &iris_syntax::MatchBody,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match body {
            iris_syntax::MatchBody::Expression(expression) => {
                self.expression(expression, locals, receiver)
            }
            iris_syntax::MatchBody::Block(statements) => self.block(statements, locals, receiver),
        }
    }

    /// Tests one pattern from the `IRIS-V1-CONTROL-C051` vocabulary.
    ///
    /// `_` discards and creates no binding per `IRIS-V1-CONTROL-C053`, a binding
    /// pattern always matches and binds, and alternatives succeed on the first
    /// alternative that matches.
    fn pattern_matches(
        &mut self,
        pattern: &iris_syntax::Pattern,
        value: &Value,
        bound: &mut HashMap<String, Value>,
    ) -> Result<bool, EvaluationError> {
        match pattern {
            iris_syntax::Pattern::Name(name) if name == "_" => Ok(true),
            iris_syntax::Pattern::Name(name) => {
                bound.insert(name.clone(), value.clone());
                Ok(true)
            }
            iris_syntax::Pattern::Literal(literal) => {
                let expected =
                    self.expression(&Expression::Literal(literal.clone()), &HashMap::new(), None)?;
                Ok(expected == *value)
            }
            iris_syntax::Pattern::Array(elements) => {
                let Value::Array(values) = value else {
                    return Ok(false);
                };
                if values.len() != elements.len() {
                    return Ok(false);
                }
                for (element, value) in elements.iter().zip(values.elements()) {
                    if !self.pattern_matches(element, &value, bound)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            iris_syntax::Pattern::Alternatives(alternatives) => {
                for alternative in alternatives {
                    if self.pattern_matches(alternative, value, bound)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Runs `for pattern in iterable { body }` per `IRIS-V1-CONTROL-C044`.
    ///
    /// The iterable is evaluated ONCE, `next()` is called repeatedly, and the
    /// loop exits normally on `Iteration.done`. Each yielded value binds in a
    /// FRESH per-iteration scope, which is what lets a Closure created in the
    /// body capture that iteration's value rather than a shared cell.
    /// `IRIS-V1-CONTROL-C046` requires the Iterator to close on every exit path,
    /// so `close` is sent on natural exhaustion, on `break`, and on an error.
    fn for_statement(
        &mut self,
        label: Option<&str>,
        binding: &iris_syntax::Pattern,
        iterable: &Expression,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let source = self.expression(iterable, locals, receiver.clone())?;
        let iterator = self.send(source, "iterator", &[])?;
        let outcome =
            self.for_iterations(label, binding, &iterator, body, locals, receiver.clone());
        self.close_after(iterator, outcome)
    }

    /// Applies an `IRIS-V1-COLLECTIONS-C040` collection operation.
    ///
    /// Returns `None` when the selector is not one of them, so an ordinary send
    /// continues unchanged. The receiver value is returned in mutated form; the
    /// caller's binding is updated by the assignment path that owns it, exactly
    /// as `append` already worked.
    fn collection_mutation(
        &mut self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        match (receiver, selector, arguments) {
            // C024 names append and insert the EXPLICIT growth operations.
            // These mutate the SHARED body, so every other binding, parameter
            // and field naming the same Array observes the change, and the
            // C026 content version moves so active iterators fail fast.
            (Value::Array(values), "append" | "insert" | "delete" | "clear", _) => values
                .mutate(|elements| apply_array_mutation(elements, selector, arguments))
                .map(Some),
            // C029: `fetch` RAISES for an absent key where `[]` answers nil.
            // C051 appends another String directly and otherwise invokes its
            // `to_string`. C072 forbids reaching text through an implicit
            // binary conversion, so Bytes does not join a String here.
            (Value::Text(text), "+", [other]) => {
                let joined = match other {
                    Value::Text(other) => other.clone(),
                    other => match self.send(other.clone(), "to_string", &[])? {
                        Value::Text(rendered) => rendered,
                        // C048 raises when a conversion answers a non-String.
                        _ => return Err(EvaluationError::TypeContractError),
                    },
                };
                Ok(Some(Value::Text(format!("{text}{joined}"))))
            }
            // C071 answers an immutable Bytes from `Bytes + ...` and a FRESH
            // ByteArray identity from `ByteArray + ...` without changing the
            // receiver. `<<` and `append` mutate in place after snapshotting
            // aliases and answer the receiver, committing atomically.
            (Value::Bytes(_) | Value::ByteArray(_), "+" | "<<" | "append", [other]) => {
                let addition = match other {
                    Value::Bytes(other) => other.clone(),
                    Value::ByteArray(other) => other.bytes(),
                    // C072 forbids reaching binary through a text conversion,
                    // so a String does not join a byte sequence here.
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                match (&receiver, selector) {
                    (Value::Bytes(bytes), "+") => {
                        let mut joined = bytes.clone();
                        joined.extend_from_slice(&addition);
                        Ok(Some(Value::Bytes(joined)))
                    }
                    (Value::ByteArray(bytes), "+") => {
                        let mut joined = bytes.bytes();
                        joined.extend_from_slice(&addition);
                        Ok(Some(Value::ByteArray(iris_runtime::ByteArrayRef::new(
                            joined,
                        ))))
                    }
                    (Value::ByteArray(bytes), _) => {
                        bytes.mutate(|bytes| bytes.extend_from_slice(&addition));
                        Ok(Some(receiver.clone()))
                    }
                    // C067 gives immutable Bytes no in-place append.
                    _ => Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                }
            }
            // C082 evaluates text and Regex once, requires the subject to be a
            // String or a MutableString SNAPSHOT, and answers an immutable
            // Match for the FIRST match or nil. C084 makes the snapshot the
            // reason a later MutableString mutation cannot rewrite a Match.
            (Value::Text(_) | Value::MutableString(_), "=~" | "!~", [Value::Regex(regex)]) => {
                let subject = match &receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let compiled = crate::source_method::compiled_regex(&regex.pattern, &regex.flags)?;
                let found = compiled.captures(&subject);
                if selector == "!~" {
                    // C082 makes `!~` true exactly when `=~` would answer nil.
                    return Ok(Some(Value::Bool(found.is_none())));
                }
                let Some(found) = found else {
                    return Ok(Some(Value::Nil));
                };
                let whole = found.get(0).ok_or(EvaluationError::RegexSyntaxError)?;
                // C083 exposes scalar ranges alongside UTF-8 byte ranges, so the
                // scalar offsets are counted rather than assumed equal to bytes.
                let scalar_start = subject[..whole.start()].chars().count();
                let scalar_end = scalar_start + whole.as_str().chars().count();
                // C083 keeps capture ABSENCE distinct from an empty capture, so
                // a non-participating group stays None rather than becoming "".
                let captures = (1..compiled.captures_len())
                    .map(|index| found.get(index).map(|group| group.as_str().to_owned()))
                    .collect();
                let named = compiled
                    .capture_names()
                    .flatten()
                    .map(|name| {
                        (
                            name.to_owned(),
                            found.name(name).map(|group| group.as_str().to_owned()),
                        )
                    })
                    .collect();
                Ok(Some(Value::Match(Box::new(iris_runtime::MatchValue {
                    text: whole.as_str().to_owned(),
                    byte_start: whole.start(),
                    byte_end: whole.end(),
                    scalar_start,
                    scalar_end,
                    captures,
                    named,
                    regex: regex.as_ref().clone(),
                }))))
            }
            // C083 exposes the full match, its ranges, its captures and the
            // Regex used, and forbids any global or mutable engine state.
            (Value::Match(matched), "text" | "to_string", []) => {
                Ok(Some(Value::Text(matched.text.clone())))
            }
            (Value::Match(matched), "regex", []) => {
                Ok(Some(Value::Regex(Box::new(matched.regex.clone()))))
            }
            (Value::Match(matched), "byte_start", []) => Ok(Some(Value::Integer(
                u64::try_from(matched.byte_start).unwrap_or_default().into(),
            ))),
            (Value::Match(matched), "byte_end", []) => Ok(Some(Value::Integer(
                u64::try_from(matched.byte_end).unwrap_or_default().into(),
            ))),
            (Value::Match(matched), "start", []) => Ok(Some(Value::Integer(
                u64::try_from(matched.scalar_start)
                    .unwrap_or_default()
                    .into(),
            ))),
            (Value::Match(matched), "end", []) => Ok(Some(Value::Integer(
                u64::try_from(matched.scalar_end).unwrap_or_default().into(),
            ))),
            (Value::Match(matched), "capture", [index]) => {
                let found = match index {
                    Value::Integer(index) => index
                        .to_usize()
                        .and_then(|index| index.checked_sub(1))
                        .and_then(|index| matched.captures.get(index).cloned())
                        .flatten(),
                    Value::Symbol(name) | Value::Text(name) => matched
                        .named
                        .iter()
                        .find(|(known, _)| known == name)
                        .and_then(|(_, value)| value.clone()),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                Ok(Some(found.map_or(Value::Nil, Value::Text)))
            }
            // C045 gives a programmatic bind the SAME validation a sidecar
            // declaration gets, and C047 lists the data a signature must carry.
            (Value::Library(library), "bind", [symbol, signature, ..]) => {
                let (Value::Symbol(symbol) | Value::Text(symbol)) = symbol else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                Self::validate_ffi_signature(signature)?;
                let mut bound = library.bound.clone();
                bound.push(symbol.clone());
                Ok(Some(Value::Library(Box::new(iris_runtime::LibraryValue {
                    identity: library.identity,
                    path: library.path.clone(),
                    bound,
                }))))
            }
            (Value::Library(library), "bound?", [symbol]) => {
                let (Value::Symbol(symbol) | Value::Text(symbol)) = symbol else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                Ok(Some(Value::Bool(library.bound.contains(symbol))))
            }
            // C045 forbids invoking an UNBOUND symbol, and C046 denies any
            // signature-less escape hatch, so the refusal happens here and no
            // native call is attempted.
            (Value::Library(library), "call", [symbol, ..]) => {
                let (Value::Symbol(symbol) | Value::Text(symbol)) = symbol else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                if !library.bound.contains(symbol) {
                    return Err(EvaluationError::UnboundNativeSymbol);
                }
                if !self.grants.is_empty()
                    && !self.grants.iter().any(|(name, _)| name == "ffi.call")
                {
                    return Err(EvaluationError::PermissionDenied);
                }
                // A bound, granted symbol still needs a real native boundary,
                // which this engine does not provide. Answering a value here
                // would fake a call that never happened.
                Err(EvaluationError::UnsupportedConstruct)
            }
            // C044 exposes an ordered scalar Array, each element an immutable
            // one-scalar String, which is what `C045` makes an index answer.
            (Value::Text(_) | Value::MutableString(_), "to_array", []) => {
                let text = match &receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                Ok(Some(Value::Array(ArrayRef::new(
                    text.chars().map(|s| Value::Text(s.to_string())).collect(),
                ))))
            }
            // C059 gives each transform a functional form answering a FRESH
            // value and a bang form mutating in place and answering the
            // receiver, so the two are distinguished by identity.
            (Value::Text(text), "upcase" | "downcase", []) => {
                Ok(Some(Value::Text(if selector == "upcase" {
                    text.to_uppercase()
                } else {
                    text.to_lowercase()
                })))
            }
            (Value::MutableString(text), "upcase" | "downcase", []) => {
                let current = text.text();
                let changed = if selector == "upcase" {
                    current.to_uppercase()
                } else {
                    current.to_lowercase()
                };
                Ok(Some(Value::MutableString(
                    iris_runtime::MutableStringRef::new(changed),
                )))
            }
            (Value::MutableString(text), "upcase!" | "downcase!", []) => {
                let current = text.text();
                // C058 commits atomically, so the whole replacement is built
                // before the receiver is touched.
                let changed = if selector == "upcase!" {
                    current.to_uppercase()
                } else {
                    current.to_lowercase()
                };
                text.set(changed);
                Ok(Some(receiver.clone()))
            }
            // C053 answers a String SNAPSHOT of current content, so a later
            // mutation of the receiver does not reach the snapshot.
            (Value::MutableString(text), "to_string", []) => Ok(Some(Value::Text(text.text()))),
            (Value::MutableString(text), "to_bytes", []) => {
                Ok(Some(Value::Bytes(text.text().into_bytes())))
            }
            (Value::MutableString(text), "length", []) => Ok(Some(Value::Integer(
                u64::try_from(text.text().chars().count())
                    .unwrap_or_default()
                    .into(),
            ))),
            // C056 converts the operand first and answers a FRESH identity,
            // leaving the receiver unchanged. C058 requires the in-place forms
            // to commit atomically, so the conversion is completed BEFORE the
            // receiver is touched, and a self or alias append is snapshotted by
            // that same ordering.
            (Value::MutableString(text), "+" | "<<" | "append", [other]) => {
                let addition = self.text_operand(other)?;
                if selector == "+" {
                    return Ok(Some(Value::MutableString(
                        iris_runtime::MutableStringRef::new(format!("{}{addition}", text.text())),
                    )));
                }
                text.set(format!("{}{addition}", text.text()));
                Ok(Some(receiver.clone()))
            }
            (Value::MutableString(text), "clear", []) => {
                text.set(String::new());
                Ok(Some(receiver.clone()))
            }
            (Value::MutableString(text), "replace", [other]) => {
                let replacement = self.text_operand(other)?;
                text.set(replacement);
                Ok(Some(receiver.clone()))
            }
            // C044 exposes a MutableString's UTF-8 bytes explicitly. C061
            // makes the view a live Iterable over the receiver, not a detached
            // Array, so a later content change invalidates an active cursor.
            (Value::MutableString(_), "bytes", []) => Ok(Some(receiver.clone())),
            // C044 exposes UTF-8 bytes EXPLICITLY, since `length` and indexing
            // count Unicode scalars. C073 makes this the same snapshot
            // `to_bytes` answers.
            (Value::Text(text), "bytes", []) => Ok(Some(Value::Bytes(text.as_bytes().to_vec()))),
            // C042 ties casefold to the fixed Unicode data version, so `ß`
            // folds to `ss` under every host locale rather than following one.
            (Value::Text(_) | Value::MutableString(_), "casefold", []) => {
                let text = match &receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                Ok(Some(Value::Text(
                    icu_casemap::CaseMapper::new()
                        .fold_string(&text)
                        .into_owned(),
                )))
            }
            // C042 ties normalization to the fixed Unicode data version, so
            // these use the pinned tables rather than a host locale.
            (Value::Text(_) | Value::MutableString(_), "nfc" | "nfd", []) => {
                let text = match &receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                use unicode_normalization::UnicodeNormalization;
                Ok(Some(Value::Text(if selector == "nfc" {
                    text.nfc().collect()
                } else {
                    text.nfd().collect()
                })))
            }
            // C044 exposes Unicode GRAPHEME CLUSTERS explicitly, using the
            // fixed Unicode data version, because `length` and indexing count
            // scalars and a cluster may span several of them.
            (Value::Text(_) | Value::MutableString(_), "graphemes", []) => {
                let text = match &receiver {
                    Value::Text(text) => text.clone(),
                    Value::MutableString(text) => text.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let clusters =
                    unicode_segmentation::UnicodeSegmentation::graphemes(text.as_str(), true)
                        .map(|cluster| Value::Text(cluster.to_owned()))
                        .collect();
                Ok(Some(Value::Array(ArrayRef::new(clusters))))
            }
            // C048 converts a non-String interpolation value through dynamic
            // `to_string`, so the built-in value families answer one. C049
            // gives ordinary objects the nominal-name form separately.
            (Value::Integer(value), "to_string", []) => Ok(Some(Value::Text(value.decimal_text()))),
            (Value::Bool(value), "to_string", []) => Ok(Some(Value::Text(value.to_string()))),
            (Value::Nil, "to_string", []) => Ok(Some(Value::Text("nil".into()))),
            (Value::Symbol(name), "to_string", []) => Ok(Some(Value::Text(name.clone()))),
            // C050 makes `to_string` answer the receiver itself.
            (Value::Text(text), "to_string", []) => Ok(Some(Value::Text(text.clone()))),
            // C050 requires a REPARSABLE double-quoted literal that recreates a
            // scalar-equal String and does not execute interpolation when
            // parsed. Interpolation is written `${...}`, so only a `$` that
            // OPENS one needs escaping; escaping every `$` would emit `\$`,
            // which the literal grammar does not accept.
            (Value::Text(text), "inspect", []) => {
                let mut rendered = String::from("\"");
                let mut scalars = text.chars().peekable();
                while let Some(scalar) = scalars.next() {
                    match scalar {
                        '"' => rendered.push_str("\\\""),
                        '\\' => rendered.push_str("\\\\"),
                        '\n' => rendered.push_str("\\n"),
                        '\r' => rendered.push_str("\\r"),
                        '\t' => rendered.push_str("\\t"),
                        // Interpolation is written `${...}`. The literal
                        // escape table has no `\$`, so a `$` that would OPEN
                        // one is emitted as its Unicode escape instead, which
                        // reparses to the same scalar without interpolating.
                        '$' if scalars.peek() == Some(&'{') => {
                            rendered.push_str("\\u{24}");
                        }
                        scalar => rendered.push(scalar),
                    }
                }
                rendered.push('"');
                Ok(Some(Value::Text(rendered)))
            }
            // C073 decodes STRICTLY and raises EncodingError on an invalid
            // sequence; lossy behavior is never the default. C072 keeps text
            // and binary conversion to these EXPLICIT APIs.
            (Value::Bytes(bytes), "to_string", []) => String::from_utf8(bytes.clone())
                .map(|text| Some(Value::Text(text)))
                .map_err(|_| EvaluationError::EncodingError),
            (Value::ByteArray(bytes), "to_string", []) => String::from_utf8(bytes.bytes())
                .map(|text| Some(Value::Text(text)))
                .map_err(|_| EvaluationError::EncodingError),
            // C073 makes `to_bytes` a Bytes SNAPSHOT, so a ByteArray answers
            // its current contents as an immutable value.
            (Value::Bytes(bytes), "to_bytes", []) => Ok(Some(Value::Bytes(bytes.clone()))),
            (Value::ByteArray(bytes), "to_bytes", []) => Ok(Some(Value::Bytes(bytes.bytes()))),
            (Value::Text(text), "to_bytes", []) => Ok(Some(Value::Bytes(text.as_bytes().to_vec()))),
            // C031 builds and validates a temporary replacement from CURRENT
            // keys, hashes and equality, and publishes nothing on failure.
            // C030 makes this the user's remedy after mutating `==` or `hash`.
            // C038 answers a Range with a NONZERO step whose sign moves toward
            // the end, and raises RangeError otherwise. A zero step would never
            // reach the end, which is why it is rejected rather than clamped.
            (Value::Range(range), "by", [step]) => {
                // C038 spells the operand `range.by(step: Integer)`, so the
                // argument arrives as a KEYWORD argument rather than a bare
                // positional one.
                let step = match step {
                    Value::KeywordArgument(name, value) if name == "step" => value.as_ref(),
                    value => value,
                };
                let Value::Integer(step) = step else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let Some(magnitude) = step.to_i128() else {
                    return Err(EvaluationError::RangeError);
                };
                if magnitude == 0 {
                    return Err(EvaluationError::RangeError);
                }
                let descending = iris_runtime::Numeric::compare(
                    &iris_runtime::NumericValue::Integer(range.end.clone()),
                    &iris_runtime::NumericValue::Integer(range.start.clone()),
                ) == Some(std::cmp::Ordering::Less);
                if (descending && magnitude > 0) || (!descending && magnitude < 0) {
                    return Err(EvaluationError::RangeError);
                }
                Ok(Some(Value::Range(Box::new(iris_runtime::RangeValue {
                    start: range.start.clone(),
                    end: range.end.clone(),
                    inclusive_end: range.inclusive_end,
                    step: step.clone(),
                }))))
            }
            (Value::Hash(entries), "rehash", []) => self.rehash(&entries.clone(), None).map(Some),
            // C032 supplies the merge as a trailing block, which reaches the
            // send as an ordinary closure argument.
            (Value::Hash(entries), "rehash", [block @ Value::Closure(_)]) => {
                let (entries, block) = (entries.clone(), block.clone());
                self.rehash(&entries, Some(block)).map(Some)
            }
            (Value::Hash(entries), "fetch", [key]) => {
                let (entries, key) = (entries.clone(), key.clone());
                let slot = self.hash_slot(&entries, &key)?;
                slot.and_then(|slot| entries.value_at(slot))
                    .map(Some)
                    .ok_or(EvaluationError::KeyError)
            }
            // C029 answers the old value or nil, and removal is structural.
            (Value::Hash(entries), "delete", [key]) => {
                let (entries, key) = (entries.clone(), key.clone());
                let slot = self.hash_slot(&entries, &key)?;
                Ok(Some(
                    slot.and_then(|slot| entries.remove_at(slot))
                        .unwrap_or(Value::Nil),
                ))
            }
            // C037 yields exactly `(key, value)` to an `each` block and
            // exactly `(key, value, iterator)` to `each_with_iterator`, which
            // is the ONLY standard block surface exposing `remove_current`.
            // Each traversal takes its own Iterator, so nested traversals do
            // not share a cursor and no hidden current-iterator context exists.
            (Value::Hash(_), "each" | "each_with_iterator", [Value::Closure(block)]) => {
                let (block, with_iterator) = (*block, selector == "each_with_iterator");
                let cursor = self.send(receiver.clone(), "iterator", &[])?;
                let outcome = loop {
                    let step = match self.send(cursor.clone(), "next", &[]) {
                        Ok(step) => step,
                        Err(error) => break Err(error),
                    };
                    let Value::IterationYield(entry) = step else {
                        break Ok(Value::Nil);
                    };
                    let Value::Tuple(mut arguments) = *entry else {
                        break Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                    };
                    if with_iterator {
                        arguments.push(cursor.clone());
                    }
                    if let Err(error) = self.invoke_closure(block, &arguments) {
                        break Err(error);
                    }
                };
                // C036 and C054 close the cursor on every exit path.
                self.close_after(cursor, outcome).map(Some)
            }
            // C051 makes `to_array` the ordered element sequence for the
            // sequence-shaped receivers, which is how a Range or Tuple is
            // materialized without promising anything about Hash order.
            (
                Value::Range(_) | Value::Tuple(_) | Value::Bytes(_) | Value::ByteArray(_),
                "to_array",
                [],
            ) => {
                let elements = match &receiver {
                    Value::Tuple(elements) => elements.clone(),
                    Value::Bytes(bytes) => bytes
                        .iter()
                        .map(|byte| Value::Integer(u64::from(*byte).into()))
                        .collect(),
                    Value::ByteArray(bytes) => bytes
                        .bytes()
                        .iter()
                        .map(|byte| Value::Integer(u64::from(*byte).into()))
                        .collect(),
                    // A Range materializes through its own iterator, so
                    // C039's openness and step rules are not restated here.
                    Value::Range(_) => {
                        let cursor = self.send(receiver.clone(), "iterator", &[])?;
                        let mut values = Vec::new();
                        while let Value::IterationYield(value) =
                            self.send(cursor.clone(), "next", &[])?
                        {
                            values.push(*value);
                        }
                        values
                    }
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                Ok(Some(Value::Array(ArrayRef::new(elements))))
            }
            (Value::Array(values), "to_array", []) => {
                // C025 makes a copy INDEPENDENT of the receiver.
                Ok(Some(Value::Array(ArrayRef::new(values.elements()))))
            }
            // C033 leaves iteration ORDER unspecified, so an entry array is an
            // unordered multiset in which each entry appears exactly once.
            (Value::Hash(entries), "to_array", []) => Ok(Some(Value::Array(ArrayRef::new(
                entries
                    .entries()
                    .into_iter()
                    .map(|(key, value)| Value::Tuple(vec![key, value]))
                    .collect(),
            )))),
            (Value::Hash(entries), "keys", []) => Ok(Some(Value::Array(
                entries.entries().into_iter().map(|(key, _)| key).collect(),
            ))),
            (Value::Hash(entries), "values", []) => Ok(Some(Value::Array(
                entries
                    .entries()
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect(),
            ))),
            (Value::Hash(entries), "has_key?", [key]) => {
                Ok(Some(Value::Bool(entries.contains_key(key))))
            }
            _ => Ok(None),
        }
    }

    /// Converts an operand to text for the `IRIS-V1-COLLECTIONS-C056` forms.
    ///
    /// String and MutableString contribute SNAPSHOT semantics, and anything
    /// else is converted through `to_string`, whose non-String result `C048`
    /// rejects. This runs to completion before any in-place mutation, which is
    /// what makes `C058`'s atomic commit and its self-append snapshot hold.
    fn text_operand(&mut self, value: &Value) -> Result<String, EvaluationError> {
        match value {
            Value::Text(text) => Ok(text.clone()),
            Value::MutableString(text) => Ok(text.text()),
            other => match self.send(other.clone(), "to_string", &[])? {
                Value::Text(text) => Ok(text),
                Value::MutableString(text) => Ok(text.text()),
                _ => Err(EvaluationError::TypeContractError),
            },
        }
    }

    /// Builds a String literal containing `${expr}` interpolation.
    ///
    /// `IRIS-V1-COLLECTIONS-C048` fixes the order and the failure behaviour:
    /// segments run left to right, a non-String value is converted by dynamic
    /// `to_string`, a non-String conversion result raises TypeContractError,
    /// and on any failure later segments do not run and nothing partial is
    /// published. Building into a local and answering only at the end is what
    /// makes the last requirement hold.
    fn interpolated_string(
        &mut self,
        source: &str,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let Value::Text(body) = literal(source)? else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let mut built = String::new();
        let mut rest = body.as_str();
        while let Some(open) = rest.find("${") {
            built.push_str(&rest[..open]);
            let after = &rest[open + 2..];
            let Some(end) = after.find('}') else {
                return Err(EvaluationError::LexicalDiagnostic(
                    "PARSE_BAD_INTERPOLATION",
                ));
            };
            let parsed = iris_parser::parse(&after[..end]);
            let [iris_syntax::ProgramEntry::Statement(Statement::Expression(expression))] =
                parsed.program.entries.as_slice()
            else {
                return Err(EvaluationError::LexicalDiagnostic(
                    "PARSE_BAD_INTERPOLATION",
                ));
            };
            let value = self.expression(expression, locals, receiver.clone())?;
            built.push_str(&self.text_operand(&value)?);
            rest = &after[end + 1..];
        }
        built.push_str(rest);
        Ok(Value::Text(built))
    }

    /// Builds a Regex literal whose pattern contains `${expr}` interpolation.
    ///
    /// `IRIS-V1-COLLECTIONS-C076` evaluates each segment ONCE, converts it with
    /// dynamic `to_string`, and escapes the result with default Regex escaping.
    /// Escaping is the point: an interpolated `a+b` must match the three
    /// characters, not "one or more a followed by b".
    fn interpolated_regex(
        &mut self,
        source: &str,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let body = source
            .strip_prefix('/')
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let close = body
            .rfind('/')
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let (pattern, flags) = body.split_at(close);
        let flags = &flags[1..];
        let mut built = String::new();
        let mut rest = pattern;
        while let Some(open) = rest.find("${") {
            built.push_str(&rest[..open]);
            let after = &rest[open + 2..];
            let end = after
                .find('}')
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let parsed = iris_parser::parse(&after[..end]);
            let [iris_syntax::ProgramEntry::Statement(Statement::Expression(expression))] =
                parsed.program.entries.as_slice()
            else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let value = self.expression(expression, locals, receiver.clone())?;
            let text = self.text_operand(&value)?;
            built.push_str(&regex::escape(&text));
            rest = &after[end + 1..];
        }
        built.push_str(rest);
        let canonical: String = "imsx"
            .chars()
            .filter(|flag| flags.contains(*flag))
            .collect();
        crate::source_method::compiled_regex(&built, &canonical)?;
        Ok(Value::Regex(Box::new(iris_runtime::RegexValue {
            pattern: built,
            flags: canonical,
        })))
    }

    /// Validates an `IRIS-V1-FFI-C047` FFI signature.
    ///
    /// `C047` lists the data a signature MUST declare and requires missing data
    /// to reject the binding BEFORE any call occurs, so this checks presence
    /// rather than deferring to call time. `C046` denies any signature-less
    /// path, which is why an absent signature is a rejection and not a default.
    fn validate_ffi_signature(signature: &Value) -> Result<(), EvaluationError> {
        let Value::Hash(fields) = signature else {
            return Err(EvaluationError::IncompleteNativeSignature);
        };
        // The subset checked here is the part a pure-Iris fixture can state:
        // the calling convention, the parameter C types, the result C type, and
        // the error convention. C047 lists more, and the remainder becomes
        // checkable when a real native boundary exists.
        for required in ["convention", "parameters", "result", "errors"] {
            if !fields.contains_key(&Value::Symbol(required.into())) {
                return Err(EvaluationError::IncompleteNativeSignature);
            }
        }
        Ok(())
    }

    /// Enqueues one after-commit revision event for every subscriber.
    ///
    /// `IRIS-V1-ASYNC-C047` publishes at safepoint and resumes the runtime
    /// WITHOUT waiting for subscribers, so this only records the commit and
    /// never invokes subscriber code. `C051` drops the oldest commits into a
    /// coalesced gap when a bounded queue is full, rather than blocking the
    /// commit.
    fn enqueue_revision_event(&mut self, group: &[ClassId]) {
        if group.is_empty() {
            return;
        }
        let commit = self.next_commit_id;
        self.next_commit_id += 1;
        // C053 answers RETAINED audit events, so the commit is recorded here
        // and pruning removes it again.
        self.audit_history.push(commit);
        let summary = group
            .iter()
            .map(|target| Value::Symbol(self.class_source_name(*target)))
            .collect();
        self.committed_targets.insert(commit, summary);
        for subscriber in &mut self.revision_subscribers {
            subscriber.queued.push(commit);
            if subscriber.queued.len() <= subscriber.capacity {
                continue;
            }
            // C051 coalesces the dropped range into one GapEvent, and C052
            // makes that range inclusive and forbids pretending no change
            // occurred, so the dropped ids extend an existing gap.
            let dropped = subscriber.queued.remove(0);
            subscriber.gap = Some(match subscriber.gap {
                Some((from, _)) => (from, dropped),
                None => (dropped, dropped),
            });
        }
    }

    /// Closes revision delivery, per `IRIS-V1-ASYNC-C050`.
    ///
    /// Shutdown makes every event accepted but not yet delivered UNREACHABLE,
    /// so a later flush reports incomplete delivery rather than pretending the
    /// events reached a terminal state.
    fn shutdown_revision_delivery(&mut self) -> Value {
        self.revision_delivery_closed = true;
        Value::Nil
    }

    /// Delivers every queued revision event, per `IRIS-V1-ASYNC-C049`.
    ///
    /// The flush surface waits until each accepted event has been delivered,
    /// converted into a delivered `GapEvent`, or reported through the
    /// event-error channel. `C048` isolates a subscriber failure: it is
    /// recorded and does not roll back the commit, affect other subscribers, or
    /// propagate to the completed open caller.
    fn flush_revision_events(&mut self) -> Result<Value, EvaluationError> {
        let mut delivered = Vec::new();
        let mut undelivered = 0_usize;
        if self.revision_delivery_closed {
            // Delivery is closed, so nothing further can reach a terminal
            // state. The accepted-but-undelivered count is the structured
            // state C050 requires for diagnostics.
            for subscriber in &mut self.revision_subscribers {
                undelivered += subscriber.queued.len() + usize::from(subscriber.gap.is_some());
                subscriber.queued.clear();
                subscriber.gap = None;
            }
            return Ok(Value::Tuple(vec![
                Value::Symbol("incomplete".into()),
                Value::Array(ArrayRef::new(delivered)),
                Value::Integer(u64::try_from(undelivered).unwrap_or_default().into()),
                Value::Array(ArrayRef::new(self.event_errors.clone())),
            ]));
        }
        for index in 0..self.revision_subscribers.len() {
            let (callback, gap, queued) = {
                let subscriber = &mut self.revision_subscribers[index];
                (
                    subscriber.callback,
                    subscriber.gap.take(),
                    std::mem::take(&mut subscriber.queued),
                )
            };
            // C051 delivers the GapEvent BEFORE later retained events.
            if let Some((from, to)) = gap {
                let event = Value::Tuple(vec![
                    Value::Symbol("GapEvent".into()),
                    Value::Integer(from.into()),
                    Value::Integer(to.into()),
                ]);
                delivered.push(event.clone());
                self.deliver_revision_event(callback, event);
            }
            for commit in queued {
                // C046 fixes what a payload identifies: the commit ID and a
                // permitted summary. It MUST NOT expose private Method bodies,
                // raw private data, candidate mutation handles or rollback
                // authority, so the payload carries names and counts rather
                // than anything reachable back into the committed candidate.
                let summary = self
                    .committed_targets
                    .get(&commit)
                    .cloned()
                    .unwrap_or_default();
                let event = Value::Tuple(vec![
                    Value::Symbol("RevisionEvent".into()),
                    Value::Integer(commit.into()),
                    Value::Array(ArrayRef::new(summary)),
                ]);
                delivered.push(event.clone());
                self.deliver_revision_event(callback, event);
            }
        }
        // C050 makes a successful flush mean the delivery condition is
        // satisfied, and requires an incomplete report with enough structured
        // state when shutdown closed delivery first. The status travels with
        // the delivered sequence so one surface answers both.
        let status = if undelivered == 0 {
            Value::Symbol("delivered".into())
        } else {
            Value::Symbol("incomplete".into())
        };
        Ok(Value::Tuple(vec![
            status,
            Value::Array(ArrayRef::new(delivered)),
            Value::Integer(u64::try_from(undelivered).unwrap_or_default().into()),
            Value::Array(ArrayRef::new(self.event_errors.clone())),
        ]))
    }

    /// Invokes one subscriber, isolating its failure per `IRIS-V1-ASYNC-C048`.
    ///
    /// A failure creates its own context and is recorded on the event-error
    /// channel. It MUST NOT affect other subscribers or the commit, so the
    /// error is captured here rather than propagated.
    fn deliver_revision_event(&mut self, callback: iris_runtime::ObjectId, event: Value) {
        if let Err(error) = self.invoke_closure(callback, &[event]) {
            // C048 requires the failure to create a FULL context and be
            // recorded, so the raised Iris value is kept where there is one.
            let recorded = match error {
                EvaluationError::Raised(value) => value,
                other => catchable_name(&other)
                    .map_or(Value::Symbol("SubscriberError".into()), |name| {
                        Value::Symbol(name)
                    }),
            };
            self.event_errors.push(recorded);
        }
    }

    /// Resumes every continuation made ready by a completion post.
    ///
    /// `IRIS-V1-ASYNC-C014` enqueues ready continuations in deterministic FIFO
    /// order, so they resume in the order they became ready. A resumed body may
    /// suspend again on another Gate, which simply re-registers it.
    fn drive_ready_continuations(&mut self) -> Result<(), EvaluationError> {
        while !self.ready.is_empty() {
            let identity = self.ready.remove(0);
            let Some(task) = self.suspended.remove(&identity) else {
                continue;
            };
            let Some(resumed) = self.gates.get(&task.gate).cloned().flatten() else {
                // The Gate is no longer complete, so the continuation is not
                // ready after all and stays registered.
                self.suspended.insert(identity, task);
                continue;
            };
            let mut delivered = task.delivered.clone();
            delivered.push(resumed);
            let previous_replay = self.async_replay.replace(GeneratorState {
                resume_past: delivered.len(),
                seen: 0,
            });
            let previous_values = self.replaying.replace(delivered.clone());
            self.async_depth += 1;
            let outcome = match self.block(&task.body, &task.locals, task.receiver.clone()) {
                Err(EvaluationError::Return(value)) => Ok(value),
                result => result,
            };
            self.async_depth -= 1;
            self.replaying = previous_values;
            self.async_replay = previous_replay;
            match outcome {
                Err(EvaluationError::AwaitSuspended(gate)) => {
                    self.suspended.insert(
                        identity,
                        SuspendedTask {
                            delivered,
                            gate,
                            ..task
                        },
                    );
                }
                outcome => {
                    if let Err(error) = &outcome {
                        let captured = match error {
                            EvaluationError::Raised(value) => value.clone(),
                            other => catchable_name(other)
                                .map_or(Value::Symbol("AsyncFailure".into()), Value::Symbol),
                        };
                        self.unobserved_failures.push((identity, captured));
                    }
                    self.tasks.insert(identity, outcome.map_err(Box::new));
                }
            }
        }
        Ok(())
    }

    /// The value recorded for an already-delivered await during replay.
    ///
    /// The async body is re-entered from the top, so awaits below the delivered
    /// count must answer what they answered before rather than suspending
    /// again. This is the same stackless strategy generators use.
    fn replayed_await(&self, index: usize) -> Option<Value> {
        self.replaying.as_ref()?.get(index).cloned()
    }

    /// The slot a Hash key occupies under `IRIS-V1-COLLECTIONS-C028`.
    ///
    /// `C028` dispatches each key's CURRENT `==` Method and must not fall back
    /// to identity for an identity-bearing key, so the search is a real send
    /// per entry rather than the derived value comparison. Answers `None` when
    /// no held key compares equal.
    fn hash_slot(
        &mut self,
        entries: &iris_runtime::HashRef,
        key: &Value,
    ) -> Result<Option<usize>, EvaluationError> {
        // C028 uses the key's current `hash` AND `==`, so a key whose hash has
        // moved since insertion no longer finds its entry: the stored slot was
        // placed under the old hash. C030 makes `rehash()` the remedy and says
        // the inconsistency until then is the user's responsibility, which is
        // exactly what V276 observes.
        let wanted = self.key_hash(key)?;
        for (index, (held, _)) in entries.entries().into_iter().enumerate() {
            // The bucket recorded at INSERTION is what the entry sits under. A
            // key whose hash has since moved therefore misses, which is the
            // C030 inconsistency `rehash()` exists to repair.
            let placed = entries.bucket_at(index).unwrap_or(Value::Nil);
            if placed != Value::Nil && placed != wanted {
                continue;
            }
            if self.key_equal(&held, key)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// The bucket a Hash key currently belongs to.
    ///
    /// `IRIS-V1-COLLECTIONS-C028` dispatches the key's CURRENT `hash` Method.
    /// A key whose built-in hash raises is not a legal key at all, so the
    /// failure propagates rather than being treated as a distinct bucket.
    fn key_hash(&mut self, key: &Value) -> Result<Value, EvaluationError> {
        self.send(key.clone(), "hash", &[])
    }

    /// The `IRIS-V1-LIBRARY-C004` serialization representation of a value.
    ///
    /// A Class participates ONLY when its declared static spine lists
    /// `for Serializable`. `C003` and `C004` forbid duck typing, reflection
    /// visibility, `to_string`, `inspect`, raw ivar access and public property
    /// presence from implying eligibility, so an ordinary object is refused
    /// here rather than serialized by inspection.
    fn serializable_representation(&mut self, value: &Value) -> Result<Value, EvaluationError> {
        let Value::Object(object) = value else {
            return Ok(value.clone());
        };
        let class = self
            .runtime
            .class_of(*object)
            .map_err(EvaluationError::Construction)?;
        // `contract_names` maps a NAME to its id, so eligibility is decided by
        // looking the Contract up by name and checking the class declares it.
        let serializable = self.contract_names.get("Serializable").copied();
        let declares = serializable.is_some_and(|wanted| {
            self.class_contracts
                .get(&class)
                .is_some_and(|contracts| contracts.contains(&wanted))
        });
        if !declares {
            return Err(EvaluationError::SerializationError);
        }
        // C005 makes the representation ordinary Iris data the Class chooses,
        // so it is obtained by asking the Class rather than by inspection.
        self.send(value.clone(), "serialize", &[])
    }

    /// Renders a `IRIS-V1-LIBRARY-C011` JSON-compatible value.
    ///
    /// The encoder REJECTS an unsupported value rather than falling back to
    /// `to_string`, `inspect`, object identity or raw ivar scanning.
    fn encode_json(
        &mut self,
        value: &Value,
        canonical: bool,
        out: &mut String,
    ) -> Result<(), EvaluationError> {
        match value {
            Value::Nil => out.push_str("null"),
            Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
            Value::Integer(number) => out.push_str(&number.decimal_text()),
            Value::Text(text) => out.push_str(&render_json_text(text)),
            Value::Array(values) => {
                out.push('[');
                for (index, element) in values.elements().iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    self.encode_json(element, canonical, out)?;
                }
                out.push(']');
            }
            Value::Hash(entries) => {
                // C010 fixes the default JSON value set as Hash with STRING
                // keys, so any other key kind is rejected rather than coerced.
                let mut pairs = Vec::new();
                for (key, held) in entries.entries() {
                    let Value::Text(key) = key else {
                        return Err(EvaluationError::SerializationError);
                    };
                    pairs.push((key, held));
                }
                if canonical {
                    pairs.sort_by(|left, right| left.0.cmp(&right.0));
                }
                out.push('{');
                for (index, (key, held)) in pairs.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    out.push_str(&render_json_text(key));
                    out.push(':');
                    self.encode_json(held, canonical, out)?;
                }
                out.push('}');
            }
            // C011 rejects everything else, which is what keeps an FFI handle,
            // a Task, a Closure or a native payload out of a document.
            _ => return Err(EvaluationError::SerializationError),
        }
        Ok(())
    }

    /// Parses one `IRIS-V1-LIBRARY-C010` JSON value.
    ///
    /// `C013` refuses a value exceeding a configured limit BEFORE allocating
    /// the offending container, so the depth check precedes the recursion.
    fn decode_json(
        cursor: &mut std::iter::Peekable<std::str::Chars<'_>>,
        depth_limit: Option<usize>,
        depth: usize,
    ) -> Result<Value, EvaluationError> {
        while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
            cursor.next();
        }
        let Some(&scalar) = cursor.peek() else {
            return Err(EvaluationError::JsonSyntaxError);
        };
        match scalar {
            '[' | '{' => {
                if depth_limit.is_some_and(|limit| depth >= limit) {
                    return Err(EvaluationError::JsonLimitError);
                }
                let closing = if scalar == '[' { ']' } else { '}' };
                cursor.next();
                let mut values = Vec::new();
                let mut entries = Vec::new();
                loop {
                    while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
                        cursor.next();
                    }
                    if cursor.peek() == Some(&closing) {
                        cursor.next();
                        break;
                    }
                    if closing == ']' {
                        values.push(Self::decode_json(cursor, depth_limit, depth + 1)?);
                    } else {
                        let key = Self::decode_json(cursor, depth_limit, depth + 1)?;
                        while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
                            cursor.next();
                        }
                        if cursor.next() != Some(':') {
                            return Err(EvaluationError::JsonSyntaxError);
                        }
                        let held = Self::decode_json(cursor, depth_limit, depth + 1)?;
                        entries.push((key, held));
                    }
                    while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
                        cursor.next();
                    }
                    if cursor.peek() == Some(&',') {
                        cursor.next();
                    }
                }
                if closing == ']' {
                    Ok(Value::Array(ArrayRef::new(values)))
                } else {
                    Ok(Value::Hash(iris_runtime::HashRef::new(entries)))
                }
            }
            '"' => {
                cursor.next();
                let mut text = String::new();
                loop {
                    let Some(scalar) = cursor.next() else {
                        return Err(EvaluationError::JsonSyntaxError);
                    };
                    match scalar {
                        '"' => break,
                        '\\' => match cursor.next() {
                            Some('n') => text.push('\n'),
                            Some('t') => text.push('\t'),
                            Some('r') => text.push('\r'),
                            Some(other) => text.push(other),
                            None => return Err(EvaluationError::JsonSyntaxError),
                        },
                        other => text.push(other),
                    }
                }
                Ok(Value::Text(text))
            }
            _ => {
                let mut token = String::new();
                while let Some(&scalar) = cursor.peek() {
                    if scalar.is_whitespace() || matches!(scalar, ',' | ']' | '}' | ':') {
                        break;
                    }
                    token.push(scalar);
                    cursor.next();
                }
                match token.as_str() {
                    "null" => Ok(Value::Nil),
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => token
                        .parse()
                        .map(Value::Integer)
                        .map_err(|_| EvaluationError::JsonSyntaxError),
                }
            }
        }
    }

    /// Rejects a value `IRIS-V1-LIBRARY-C018` does not carry.
    ///
    /// `C003` forbids serializing a live resource merely because it exists, so
    /// an FFI handle, an open File, a Task or a native payload is refused here
    /// rather than having its pointer, runtime identity or descriptor emitted.
    fn check_irisvalue_encodable(value: &Value) -> Result<(), EvaluationError> {
        match value {
            Value::Nil
            | Value::Bool(_)
            | Value::Integer(_)
            | Value::Float32(_)
            | Value::Float64(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Bytes(_) => Ok(()),
            Value::Tuple(elements) => elements
                .iter()
                .try_for_each(Self::check_irisvalue_encodable),
            Value::Array(values) => values
                .elements()
                .iter()
                .try_for_each(Self::check_irisvalue_encodable),
            Value::Hash(entries) => entries.entries().iter().try_for_each(|(key, held)| {
                Self::check_irisvalue_encodable(key)
                    .and_then(|()| Self::check_irisvalue_encodable(held))
            }),
            _ => Err(EvaluationError::SerializationError),
        }
    }

    /// Compares two Hash keys under `IRIS-V1-COLLECTIONS-C028`.
    ///
    /// `C028` dispatches each key's CURRENT `==` Method and must not fall back
    /// to `same?` for identity-bearing keys unless the selected equality Method
    /// itself does so. This is therefore an ordinary `==` send, deliberately
    /// distinct from `TYPES-C050` view equality, which compares
    /// identity-bearing receivers by identity and would hide exactly the
    /// redefinition `C030` tells users to call `rehash` about.
    fn key_equal(&mut self, left: &Value, right: &Value) -> Result<bool, EvaluationError> {
        match self.send(left.clone(), "==", std::slice::from_ref(right))? {
            Value::Bool(equal) => Ok(equal),
            _ => Err(EvaluationError::ComparisonContractError),
        }
    }

    /// Rebuilds a Hash under current equality and hashes.
    ///
    /// `IRIS-V1-COLLECTIONS-C031` requires the replacement to be built and
    /// validated BEFORE anything is published, so this assembles a temporary
    /// entry list and installs it only after every check passes. A collision
    /// with no merge block raises `KeyConflictError` and leaves the receiver
    /// untouched, and under `C032` a block failure or a shape failure aborts
    /// the same way.
    fn rehash(
        &mut self,
        entries: &iris_runtime::HashRef,
        merge: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let current = entries.entries();
        let mut rebuilt: Vec<(Value, Value)> = Vec::with_capacity(current.len());
        for (key, value) in current {
            // C031 validates against the CURRENT public hash, so an unhashable
            // key aborts the rehash rather than silently keeping its old slot.
            self.send(key.clone(), "hash", &[])?;
            let mut collision = None;
            for (index, (kept, _)) in rebuilt.iter().enumerate() {
                if self.key_equal(kept, &key)? {
                    collision = Some(index);
                    break;
                }
            }
            let Some(index) = collision else {
                rebuilt.push((key, value));
                continue;
            };
            // C031 raises when two previously distinct entries collide into one
            // equality class and no merge block was supplied.
            let Some(merge) = merge.clone() else {
                return Err(EvaluationError::KeyConflictError);
            };
            let (kept_key, kept_value) = rebuilt[index].clone();
            let merged = self.send(merge, "call", &[kept_key, kept_value, key, value])?;
            // C032 takes the block result as a two-element `(key, value)`
            // replacement. A different SHAPE is a Type failure, distinct from
            // the C031 conflict that a missing block reports: V278 observes a
            // block answering `:bad` and requires TypeContractError.
            let Value::Tuple(replacement) = &merged else {
                return Err(EvaluationError::TypeContractError);
            };
            let [new_key, new_value] = replacement.as_slice() else {
                return Err(EvaluationError::TypeContractError);
            };
            // C032 requires the returned key to REMAIN equal to the class under
            // current equality, so a key that leaves its own class is a new
            // inconsistency and aborts.
            if !self.key_equal(new_key, &rebuilt[index].0)? {
                return Err(EvaluationError::KeyConflictError);
            }
            self.send(new_key.clone(), "hash", &[])?;
            rebuilt[index] = (new_key.clone(), new_value.clone());
        }
        // Every check passed, so the replacement is published atomically.
        entries.replace_entries(Vec::new());
        for (key, value) in rebuilt {
            // C031 rebuilds from CURRENT public hashes, so each surviving entry
            // is placed under the hash it has NOW rather than the stale one.
            let bucket = self.key_hash(&key)?;
            entries.insert_bucketed(None, key, value, bucket);
        }
        Ok(Value::Nil)
    }

    /// Closes a resource after a protected body and merges the two outcomes.
    ///
    /// `IRIS-V1-ASYNC-C033` and `IRIS-V1-CONTROL-C047` state one rule: if the
    /// body raised and `close` also raises, the BODY's context stays primary
    /// and the close failure is appended to its suppressed list; if the body
    /// completed and `close` raises, the close failure becomes primary and no
    /// body result is returned. `using` and `for` share this implementation
    /// rather than stating the rule twice.
    fn close_after(
        &mut self,
        resource: Value,
        outcome: Result<Value, EvaluationError>,
    ) -> Result<Value, EvaluationError> {
        // C013 makes a suspension a REGISTERED CONTINUATION, not an exit, so
        // the protected region has not been left and the resource must stay
        // open. Closing here would run cleanup once on the way out and again on
        // the replayed re-entry, which is exactly the double close V084
        // forbids.
        if matches!(outcome, Err(EvaluationError::AwaitSuspended(_))) {
            return outcome;
        }
        // The primary context must be captured BEFORE cleanup runs. A `close`
        // that raises installs its own context, which would otherwise overwrite
        // the primary one and leave the caught value and its bound context
        // disagreeing about which exception is propagating.
        let primary_context = outcome
            .is_err()
            .then(|| self.active_context.take())
            .flatten();
        let closed = self.send(resource, "close", &[]);
        match (outcome, closed) {
            (Ok(value), Ok(_)) => Ok(value),
            (Ok(_), Err(error)) => Err(error),
            (Err(primary), Err(EvaluationError::Raised(cleanup))) => {
                if let Some(Value::ExceptionContext(
                    identity,
                    value,
                    cause,
                    suppressed,
                    sites,
                    location,
                )) = primary_context
                {
                    let cleanup_identity = self.next_context_identity();
                    let cleanup_location = self.source_location(0);
                    let mut suppressed = suppressed;
                    suppressed.push(Value::ExceptionContext(
                        cleanup_identity,
                        Box::new(cleanup),
                        Box::new(Value::Nil),
                        Vec::new(),
                        Vec::new(),
                        Box::new(cleanup_location),
                    ));
                    self.active_context = Some(Value::ExceptionContext(
                        identity, value, cause, suppressed, sites, location,
                    ));
                }
                Err(primary)
            }
            (Err(primary), _) => {
                // Cleanup succeeded or failed without a context of its own, so
                // the primary context is restored unchanged.
                if let Some(context) = primary_context {
                    self.active_context = Some(context);
                }
                Err(primary)
            }
        }
    }

    fn for_iterations(
        &mut self,
        label: Option<&str>,
        binding: &iris_syntax::Pattern,
        iterator: &Value,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        loop {
            self.charge_step()?;
            let step = self.send(iterator.clone(), "next", &[])?;
            let value = match step {
                Value::IterationDone => return Ok(Value::Nil),
                Value::IterationYield(value) => *value,
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            let mut iteration = locals.clone();
            // IRIS-V1-CONTROL-C045: a destructuring mismatch raises
            // PatternMatchError. The caller closes the Iterator before this
            // propagates, which is what C045 requires of every exit path.
            if !self.pattern_matches(binding, &value, &mut iteration)? {
                return Err(EvaluationError::PatternMatchError);
            }
            match self.block(body, &iteration, receiver.clone()) {
                Ok(_) => {}
                Err(EvaluationError::LoopBreak(target, value))
                    if target.is_none() || target.as_deref() == label =>
                {
                    return Ok(value);
                }
                Err(EvaluationError::LoopContinue(target))
                    if target.is_none() || target.as_deref() == label => {}
                Err(error) => return Err(error),
            }
        }
    }

    /// Runs `while condition { body }` per `IRIS-V1-CONTROL-C043`.
    ///
    /// The condition is truth-tested BEFORE each iteration, so zero iterations
    /// are possible, and natural completion yields `nil`. A `break` unwinds as a
    /// control signal: this loop consumes one that targets it, either unlabelled
    /// or naming its own label, and lets any other keep unwinding to an outer
    /// loop.
    fn while_statement(
        &mut self,
        label: Option<&str>,
        condition: &Expression,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        loop {
            self.charge_step()?;
            let test = self.expression(condition, locals, receiver.clone())?;
            if !self.truthy(test)? {
                return Ok(Value::Nil);
            }
            match self.block(body, locals, receiver.clone()) {
                Ok(_) => {}
                Err(EvaluationError::LoopBreak(target, value))
                    if target.is_none() || target.as_deref() == label =>
                {
                    return Ok(value);
                }
                Err(EvaluationError::LoopContinue(target))
                    if target.is_none() || target.as_deref() == label => {}
                Err(error) => return Err(error),
            }
        }
    }

    fn block(
        &mut self,
        statements: &[Statement],
        parent: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        // The restore must run on EVERY exit, including the `break`, `return`
        // and raise paths that leave the block early, so the body is run
        // separately and its outcome passed through.
        let mut shadowed = Vec::new();
        let result = self.block_body(statements, parent, receiver, &mut shadowed);
        self.restore_shadowed(shadowed);
        result
    }

    fn block_body(
        &mut self,
        statements: &[Statement],
        parent: &HashMap<String, Value>,
        receiver: Option<Value>,
        shadowed: &mut Vec<(String, Option<Binding>)>,
    ) -> Result<Value, EvaluationError> {
        let mut locals = parent.clone();
        let mut result = Value::Nil;
        for statement in statements {
            match statement {
                Statement::Binding {
                    mutable,
                    name,
                    value,
                    ..
                } => {
                    let value = self.expression(value, &locals, receiver.clone())?;
                    if *mutable {
                        locals.remove(name);
                        shadowed.push((name.clone(), self.names.get(name).cloned()));
                        self.names.insert(name.clone(), Binding::new(value, true));
                    } else {
                        locals.insert(name.clone(), value);
                    }
                }
                Statement::Expression(_) | Statement::If { .. } | Statement::Try { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
                Statement::Raise(_) => return self.statement(statement, &locals, receiver.clone()),
                // A `break` leaves the block immediately, carrying its loop
                // result outward as the control signal the target loop consumes.
                Statement::Break { .. } | Statement::Continue(_) => {
                    return self.statement(statement, &locals, receiver.clone());
                }
                Statement::While { .. } | Statement::For { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
                Statement::DeferredBinding { .. }
                | Statement::GlobalBinding { .. }
                | Statement::SharedBinding { .. }
                | Statement::StoredProperty { .. }
                | Statement::Method(_) => return Err(EvaluationError::UnsupportedConstruct),
                Statement::Return(_) => {
                    return self.statement(statement, &locals, receiver.clone());
                }
                Statement::Match { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
            }
        }
        Ok(result)
    }

    /// Restores the bindings a block's `mut` declarations shadowed.
    fn restore_shadowed(&mut self, shadowed: Vec<(String, Option<Binding>)>) {
        for (name, previous) in shadowed.into_iter().rev() {
            match previous {
                Some(binding) => {
                    self.names.insert(name, binding);
                }
                None => {
                    self.names.remove(&name);
                }
            }
        }
    }

    fn try_statement(
        &mut self,
        body: &[Statement],
        catches: &[iris_syntax::CatchClause],
        finally: &Option<Vec<Statement>>,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let result = match self.block(body, locals, receiver.clone()) {
            Ok(value) => Ok(value),
            Err(EvaluationError::Raised(value)) => {
                self.catch_exception(value, catches, locals, receiver.clone())
            }
            // C056 makes `raise value` accept ANY Iris object or value and
            // hands the ORIGINAL value to the catch. A runtime failure the
            // specification NAMES, such as `TypeContractError`, is such a
            // value, so it is catchable under its own name rather than
            // travelling past every handler as an evaluator-internal error.
            //
            // A control-flow unwind is NOT an exception and still travels to
            // its own boundary.
            Err(error) => match catchable_name(&error) {
                Some(name) => {
                    self.catch_exception(Value::Symbol(name), catches, locals, receiver.clone())
                }
                None => Err(error),
            },
        };
        if let Some(finally) = finally {
            let pending = match &result {
                Err(EvaluationError::Raised(value)) => Some(value.clone()),
                Ok(_) | Err(_) => None,
            };
            let previous = self.active_exception.clone();
            let pending_context = pending
                .is_some()
                .then(|| self.active_context.clone())
                .flatten();
            self.active_exception = pending;
            let final_result = self.block(finally, locals, receiver);
            self.active_exception = previous;
            if let Err(EvaluationError::Raised(raised)) = &final_result {
                // IRIS-V1-CONTROL-C064: a `finally` that raises while a context
                // is pending becomes PRIMARY, and the pending context becomes
                // its cause. Without this the new context would simply overwrite
                // the old one and the chain would be lost.
                if let Some(cause) = pending_context {
                    let identity = self.next_context_identity();
                    let location = self.source_location(0);
                    self.active_context = Some(Value::ExceptionContext(
                        identity,
                        Box::new(raised.clone()),
                        Box::new(cause),
                        Vec::new(),
                        Vec::new(),
                        Box::new(location),
                    ));
                }
            }
            // C063: a `return`, `break` or `continue` from `finally` overrides
            // any pending result or exception. When it DISCARDS a pending
            // context, that context is recorded only in a protected runtime
            // diagnostic channel, never as cause or suppressed metadata, and
            // ordinary callers observe only the new control transfer.
            if let Err(
                EvaluationError::Return(_)
                | EvaluationError::LoopBreak(..)
                | EvaluationError::LoopContinue(_),
            ) = &final_result
                && let Err(EvaluationError::Raised(discarded)) = &result
            {
                self.discarded_contexts.push(discarded.clone());
            }
            final_result?;
        }
        result
    }

    /// Reports whether a context chain already reaches a raised value.
    ///
    /// `D-161` performs an identity-based cycle check across cause edges, so a
    /// context whose chain already carries this value cannot become its cause.
    fn context_reaches(context: &Value, value: &Value) -> bool {
        let mut current = context;
        loop {
            let Value::ExceptionContext(_, carried, cause, _, _, _) = current else {
                return false;
            };
            if **carried == *value {
                return true;
            }
            current = cause;
        }
    }

    fn catch_exception(
        &mut self,
        value: Value,
        catches: &[iris_syntax::CatchClause],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        for catch in catches {
            if !catch
                .filter
                .as_ref()
                .is_none_or(|filter| self.catch_matches(&value, filter))
            {
                continue;
            }
            let mut catch_locals = locals.clone();
            if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
                catch_locals.insert(name.clone(), value.clone());
            }
            // IRIS-V1-CONTROL-C060 binds the ExceptionContext alongside the
            // raised object. C056 gives every propagation a fresh context, and
            // an unchained one carries a nil cause per C057.
            if let Some(context) = &catch.context {
                let context_value = match self.active_context.clone() {
                    Some(context) => context,
                    None => {
                        let identity = self.next_context_identity();
                        let location = self.source_location(0);
                        Value::ExceptionContext(
                            identity,
                            Box::new(value.clone()),
                            Box::new(Value::Nil),
                            Vec::new(),
                            Vec::new(),
                            Box::new(location),
                        )
                    }
                };
                catch_locals.insert(context.clone(), context_value);
            }
            let previous = self.active_exception.replace(value.clone());
            let result = self.block(&catch.body, &catch_locals, receiver);
            self.active_exception = previous;
            return result;
        }
        Err(EvaluationError::Raised(value))
    }

    /// Enforces a written binding annotation as a runtime boundary guard.
    ///
    /// `IRIS-V1-TYPES-C004` makes an annotation BOTH a static contract and a
    /// runtime guard: a not-proven boundary MUST check before the value is
    /// published. Nothing checked here before, so `let value: Integer = nil`
    /// silently stored nil instead of raising, which is what `V220` observes
    /// for `NonNil`.
    ///
    /// This deliberately guards only annotations it can fully resolve. An
    /// unresolvable name is a static concern that `BINDING_FIXED_LOCAL_TYPE`
    /// and the generic diagnostics already own, and reporting a runtime
    /// TypeContractError for one would turn a static diagnostic into a raise.
    fn check_binding_annotation(
        &mut self,
        value: &Value,
        annotation: &iris_syntax::TypeExpression,
    ) -> Result<(), EvaluationError> {
        if self.annotation_admits(value, annotation)? {
            return Ok(());
        }
        Err(EvaluationError::TypeContractError)
    }

    /// Reports whether a value satisfies an annotation, or `true` when the
    /// annotation is not one this evaluator can decide.
    fn annotation_admits(
        &mut self,
        value: &Value,
        annotation: &iris_syntax::TypeExpression,
    ) -> Result<bool, EvaluationError> {
        match annotation {
            // C011: `NonNil` admits every value EXCEPT nil. It is a Type, not a
            // declared Class, so it never resolves through `class_name`.
            iris_syntax::TypeExpression::Name(name) if name == "NonNil" => {
                Ok(!matches!(value, Value::Nil))
            }
            // C023: `Never` is uninhabited, so no value satisfies it.
            iris_syntax::TypeExpression::Name(name) if name == "Never" => Ok(false),
            iris_syntax::TypeExpression::Name(name) => {
                let Some(class) = self.class_name(name)? else {
                    return Ok(true);
                };
                Ok(self.type_test(value, &Value::Class(class))? == Value::Bool(true))
            }
            // C020: a union admits a value that satisfies ANY constituent.
            iris_syntax::TypeExpression::Union(members) => {
                for member in members {
                    if self.annotation_admits(value, member)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            // C022: an intersection admits a value that satisfies EVERY
            // constituent.
            iris_syntax::TypeExpression::Intersection(members) => {
                for member in members {
                    if !self.annotation_admits(value, member)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            // C014: entering `Dynamic<T>` CHECKS that the value satisfies the
            // reified `T`. The boundary only lifts static member validation
            // INSIDE it, so the entry itself is guarded like any annotation.
            iris_syntax::TypeExpression::Generic { name, arguments } if name == "Dynamic" => {
                match arguments.as_slice() {
                    [argument] => self.annotation_admits(value, argument),
                    // Bare `Dynamic` normalizes to `Dynamic<Object>`, which
                    // admits everything.
                    _ => Ok(true),
                }
            }
            // `typeof`, other generic, and callable annotations are static
            // concerns this evaluator does not decide, so they never raise here.
            iris_syntax::TypeExpression::Typeof(_)
            | iris_syntax::TypeExpression::Generic { .. }
            | iris_syntax::TypeExpression::Function { .. } => Ok(true),
        }
    }

    fn catch_matches(&self, value: &Value, filter: &iris_syntax::TypeExpression) -> bool {
        match filter {
            iris_syntax::TypeExpression::Name(name) => match (name.as_str(), value) {
                ("Symbol", Value::Symbol(_))
                | ("Integer", Value::Integer(_))
                | ("Nil", Value::Nil)
                | ("Bool", Value::Bool(_)) => true,
                (_, Value::Object(object)) => self
                    .class_name(name)
                    .ok()
                    .flatten()
                    .zip(self.runtime.class_of(*object).ok())
                    .is_some_and(|(filter, mut class)| {
                        loop {
                            if class == filter {
                                break true;
                            }
                            // D-149: exception dispatch uses the CURRENT Class
                            // hierarchy, so the runtime superclass is read from
                            // the active revision. The declaration-time map is
                            // not updated by a later `set_superclass`, which
                            // made a committed inheritance change invisible to a
                            // typed catch while `is` already saw it.
                            let Some(superclass) = self
                                .runtime
                                .registry()
                                .active(class)
                                .ok()
                                .and_then(ClassRevision::runtime_superclass)
                            else {
                                break false;
                            };
                            class = superclass;
                        }
                    }),
                _ => false,
            },
            iris_syntax::TypeExpression::Typeof(_)
            | iris_syntax::TypeExpression::Function { .. }
            | iris_syntax::TypeExpression::Intersection(_)
            | iris_syntax::TypeExpression::Union(_)
            | iris_syntax::TypeExpression::Generic { .. } => false,
        }
    }

    fn condition(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<bool, EvaluationError> {
        let value = self.expression(expression, locals, receiver)?;
        self.truthy(value)
    }

    fn truthy(&mut self, value: Value) -> Result<bool, EvaluationError> {
        let result = self.send(value.clone(), "to_bool", &[])?;
        let method = TruthinessMethod::Returns(result);
        Truthiness::test(&value, method).map_err(|error| match error {
            TruthinessError::TypeContract => EvaluationError::TypeContractError,
            TruthinessError::Raised(value) => {
                EvaluationError::Execution(iris_runtime::ExecutionError::Raised(value))
            }
        })
    }

    fn expression(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match expression {
            // C037 makes a transaction body non-suspending and raises
            // `MetaTransactionError` on a DYNAMIC violation; ASYNC-C018 owns
            // the async reason. V431 observes an `await` reached inside a
            // decorator transform, which publishes nothing.
            //
            // Outside a transaction, suspension is chapter 07's own surface and
            // is deliberately NOT given a placeholder meaning here.
            // C072 suspends a generator body at `yield`. The signal carries
            // the yielded value and this suspension's index, so `next()`
            // resumes by re-entering the body and skipping suspensions it
            // already delivered. The body never sits on the native stack
            // between resumptions, which is what makes it stackless.
            //
            // C037 forbids suspending inside a transaction, exactly as it does
            // for `await`.
            Expression::Yield(value) => {
                if self.open_target.is_some() {
                    return Err(EvaluationError::MetaTransactionSuspension);
                }
                let Some(state) = self.generator.as_mut() else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let index = state.seen;
                state.seen += 1;
                if index < state.resume_past {
                    // Already delivered on an earlier `next()`, so this
                    // suspension is replayed rather than re-yielded.
                    return Ok(Value::Nil);
                }
                let yielded = match value {
                    Some(value) => self.expression(value, locals, receiver)?,
                    None => Value::Nil,
                };
                Err(EvaluationError::GeneratorYield(yielded, index))
            }
            Expression::Await(operand) => {
                // C037 makes a transaction body non-suspending; ASYNC-C018
                // owns the async reason for the prohibition.
                if self.open_target.is_some() {
                    return Err(EvaluationError::MetaTransactionSuspension);
                }
                let awaited = self.expression(operand, locals, receiver)?;
                // A Gate is the only incomplete Awaitable this engine can make.
                // C013 continues synchronously when it is already complete and
                // suspends when it is not.
                if let Value::Gate(gate) = awaited {
                    if let Some(state) = self.async_replay.as_mut() {
                        // Re-entry replays awaits already delivered, so an await
                        // below the delivered count answers its recorded value
                        // instead of suspending again.
                        let index = state.seen;
                        state.seen += 1;
                        if index < state.resume_past {
                            return self
                                .replayed_await(index)
                                .ok_or(EvaluationError::UnsupportedConstruct);
                        }
                    }
                    return match self.gates.get(&gate).cloned().flatten() {
                        Some(value) => Ok(value),
                        // C013 registers the continuation and returns control to
                        // the driver, which the signal carries out.
                        None => Err(EvaluationError::AwaitSuspended(gate)),
                    };
                }
                let Value::Task(identity) = awaited else {
                    // C008 types `await expr` through `Awaitable<T>` and only
                    // `Task<T>` implements it in v1, so an operand that is not
                    // one is a Type failure and NOT a suspension. V008 observes
                    // that nothing suspends.
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                // C013 continues SYNCHRONOUSLY on an already-complete
                // Awaitable without enqueueing a continuation for fairness.
                match self.tasks.get(&identity).cloned() {
                    Some(Ok(value)) => Ok(value),
                    // C016 propagates the captured failure to the awaiter, and
                    // C027 counts awaiting as OBSERVING it, so it is no longer
                    // eligible for unobserved-failure reporting.
                    Some(Err(error)) => {
                        self.unobserved_failures
                            .retain(|(task, _)| *task != identity);
                        Err(*error)
                    }
                    None => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            // IRIS-V1-CONTROL-C026 evaluates a keyword argument in place with
            // the positionals, so the value is produced here and the name is
            // carried to the binding step.
            // C065 reifies a parenthesized Type expression. C016 interns Type
            // objects by identity, and the normalization law table requires
            // `String | Integer` and `Integer | String` to be ONE Type, so the
            // written form is reduced to a canonical one here.
            Expression::ReifiedType(annotation) => self.reify_type(annotation),
            // C061 makes a closed generic construction name an ordinary Class
            // for construction and reflection. v1 interns one Class per generic
            // definition, so the arguments select no distinct runtime Class and
            // the construction resolves to the declared Class itself.
            Expression::ClosedGeneric { name, arguments } => {
                // C067: materializing a closed generic validates every
                // normalized constraint BEFORE interning or publishing, and a
                // failure raises rather than being reported statically.
                self.check_generic_bounds(name, arguments)?;
                let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
                // C066 runs a per-closed class property initializer once when
                // the closed Class is FIRST materialized, so the construction
                // itself is what triggers it.
                self.materialize_closed(class, arguments)?;
                Ok(Value::Class(class))
            }
            Expression::KeywordArgument { name, value } => {
                let value = self.expression(value, locals, receiver)?;
                Ok(Value::KeywordArgument(name.clone(), Box::new(value)))
            }
            // IRIS-V1-COLLECTIONS-C051 reads through the `[]` selector, so an
            // index is an ordinary send and a user Class may define it.
            Expression::Index {
                receiver: target,
                index,
            } => {
                let target = self.expression(target, locals, receiver.clone())?;
                let index = self.expression(index, locals, receiver)?;
                self.index_read(target, index)
            }
            Expression::Hash(entries) => {
                // IRIS-V1-RUNTIME-C134: Hash CONSTRUCTION with a NaN key of
                // either width must raise InvalidKeyError, so every key is
                // hashed here rather than only on later insertion.
                let mut built: Vec<(Value, Value)> = Vec::new();
                for (key, value) in entries {
                    let key = self.expression(key, locals, receiver.clone())?;
                    self.send(key.clone(), "hash", &[])?;
                    let value = self.expression(value, locals, receiver.clone())?;
                    // IRIS-V1-COLLECTIONS-C028 dispatches the key's current
                    // `==`, so a repeated key UPDATES its entry rather than
                    // adding a second one.
                    match built.iter_mut().find(|(seen, _)| *seen == key) {
                        Some(entry) => entry.1 = value,
                        None => built.push((key, value)),
                    }
                }
                Ok(Value::Hash(HashRef::new(built)))
            }
            Expression::Closure { parameters, body } => {
                // IRIS-V1-RUNTIME-C042: every evaluation allocates a NEW Closure
                // with its own captured environment, so this never caches.
                let object = iris_runtime::ObjectId::new(self.next_closure);
                self.next_closure += 1;
                self.closures.insert(
                    object,
                    ClosureRecord {
                        parameters: parameters.clone(),
                        body: body.clone(),
                        captured: locals.clone(),
                        receiver: receiver.clone(),
                    },
                );
                Ok(Value::Closure(object))
            }
            Expression::Name(name) if name == "super" => Err(EvaluationError::Runtime(
                iris_runtime::KernelError::Dispatch(iris_runtime::DispatchError::InvalidSuper {
                    selector: self
                        .current_method
                        .ok_or(EvaluationError::UnsupportedConstruct)?
                        .selector(),
                }),
            )),
            Expression::Name(name) => locals
                .get(name)
                .cloned()
                .or_else(|| self.names.get(name).map(Binding::value))
                .or_else(|| (name == "self").then_some(receiver.clone()).flatten())
                .or_else(|| builtin(name, &self.kernel))
                // D-432 resolves an unqualified name against LEXICAL scope
                // first and the CURRENT module's declarations second, so a
                // Module's own constant is reachable from its Methods and its
                // body while staying invisible outside it.
                .or_else(|| self.current_module_constant(name))
                // D-432's THIRD tier: an explicit import, after lexical scope
                // and the current module's own declarations.
                .or_else(|| self.imported_names.get(name).cloned())
                .or_else(|| {
                    self.module_names
                        .contains_key(name)
                        .then(|| Value::Symbol(name.clone()))
                })
                // C013 resolves a bare `name` against visible DECLARATIONS as
                // well as lexical bindings, and C014 makes reading a Method
                // create a BoundMethod. Inside a Module body the visible
                // declarations are `main`'s Methods, so a top-level helper is
                // readable as a value and not only callable.
                .or_else(|| self.main_bound_method(name, receiver.as_ref()))
                // C014 makes reading an instance Method create a BoundMethod,
                // and IRIS-V1-META-C024 falls back to a PRIVILEGED send to the
                // current `self` when no binding or declaration matched. A bare
                // `value` inside a Method of the same Class therefore reads
                // that Method rather than reporting an unresolved name.
                .or_else(|| self.receiver_bound_method(name, receiver.as_ref()))
                // The same C014 read inside a MODULE Method: its receiver is
                // the Module's `main`, which a bare name must reach the same
                // way a Class Method reaches its own Class.
                .or_else(|| self.module_bound_method(name))
                // IRIS-V1-CONTROL-C011: a non-call unresolved bare name raises
                // `NameError`. It MUST NOT read a property, Method, global, or
                // runtime-added member instead.
                .ok_or(EvaluationError::NameError),
            Expression::RawIvar(name) => {
                let selector = self.selector(name);
                match receiver.ok_or(EvaluationError::UnsupportedConstruct)? {
                    Value::Object(object) => self
                        .runtime
                        .raw_ivar(object, selector)
                        .map_err(EvaluationError::Construction),
                    Value::Class(class) => self
                        .runtime
                        .class_raw_ivar(class, selector)
                        .map_err(EvaluationError::Construction),
                    Value::Nil
                    | Value::Bool(_)
                    | Value::Integer(_)
                    | Value::Float32(_)
                    | Value::Float64(_) => Ok(Value::Nil),
                    _ => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            Expression::ClassVar(name) => self.read_class_var(name),
            // C013 reads a DECLARED global; a missing one is an error.
            // D-431 resolves `$name` within the CURRENT package, so a global
            // another package declared under the same name is not visible.
            Expression::GlobalVar(name) => self
                .globals
                .get(&(self.package.clone(), name.clone()))
                .map(Binding::value)
                .ok_or(EvaluationError::NameError),
            // C048 evaluates each `${expr}` LEFT TO RIGHT, converts a
            // non-String value through dynamic `to_string`, and raises
            // TypeContractError when that answers a non-String. On any failure
            // later segments do not run and no partial String is published.
            Expression::Literal(source) if source.starts_with('"') && source.contains("${") => {
                self.interpolated_string(source, locals, receiver)
            }
            // C076 evaluates each `${expr}` in a NON-RAW Regex literal once,
            // converts it through dynamic `to_string`, then escapes it with
            // default Regex escaping before insertion, so an interpolated value
            // contributes literal text and never pattern syntax.
            Expression::Literal(source) if source.starts_with('/') && source.contains("${") => {
                self.interpolated_regex(source, locals, receiver)
            }
            Expression::Literal(source) => literal(source),
            Expression::Symbol(symbol) => Ok(Value::Symbol(symbol.clone())),
            Expression::Grouped(expression) => self.expression(expression, locals, receiver),
            Expression::Array(values) => values
                .iter()
                .map(|value| self.expression(value, locals, receiver.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map(|values| Value::Array(ArrayRef::new(values))),
            // C021 makes a Tuple IMMUTABLE and identity-less, so unlike Array
            // it is built by value and needs no shared body.
            Expression::Tuple(values) => values
                .iter()
                .map(|value| self.expression(value, locals, receiver.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Tuple),
            // A `try` in expression position runs the same evaluator the
            // statement form uses, so the two can never disagree on ordering,
            // handler selection, or which clause supplies the result.
            Expression::Try {
                body,
                catches,
                finally,
            } => self.try_statement(body, catches, finally, locals, receiver),
            // A loop in expression position runs the same evaluator the
            // statement form uses, so C043's loop value cannot diverge between
            // the two spellings.
            Expression::While {
                label,
                condition,
                body,
            } => self.while_statement(label.as_deref(), condition, body, locals, receiver),
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                if self.condition(condition, locals, receiver.clone())? {
                    self.block(then_body, locals, receiver)
                } else if let Some(else_body) = else_body {
                    self.block(else_body, locals, receiver)
                } else {
                    Ok(Value::Nil)
                }
            }
            Expression::Member {
                receiver: target,
                selector,
            } if matches!(target.as_ref(), Expression::Name(name) if name == "Iteration")
                && selector == "done" =>
            {
                Ok(Value::IterationDone)
            }
            // C064 puts a Module's class-level property on the MODULE's own
            // object, so `M.first` reads that storage. A Module name evaluates
            // to a Symbol, so the read is routed from the SOURCE here rather
            // than through the evaluated receiver, which has no storage.
            Expression::Member {
                receiver: target,
                selector,
            } if matches!(target.as_ref(), Expression::Name(name)
                if self.module_names.contains_key(name)
                    && !self.names.contains_key(name)) =>
            {
                let Expression::Name(name) = target.as_ref() else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let module = *self
                    .module_names
                    .get(name)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                // V358 observes that a Module written without `mixin` has an
                // EMPTY edge list, so no implicit composition edge may appear.
                if selector == "modules" {
                    return Ok(Value::Array(ArrayRef::new(
                        self.module_component_names(module),
                    )));
                }
                let module_class = *self
                    .module_classes
                    .get(&module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                if !self.is_class_level_property(module_class, selector) {
                    return self.member_read(Value::Symbol(name.clone()), selector);
                }
                let slot = self.selector(selector);
                self.runtime
                    .class_raw_ivar(module_class, slot)
                    .map_err(EvaluationError::Construction)
            }
            // D-206 interns a closed identity by definition AND normalized
            // arguments. The construction itself resolves to the definition's
            // Class, so the arguments are read from the SOURCE here; going
            // through the evaluated receiver would have already lost them and
            // made `Box<String>.type` equal `Box<Integer>.type`.
            Expression::Member {
                receiver: target,
                selector,
            } if selector == "type"
                && matches!(target.as_ref(), Expression::ClosedGeneric { .. }) =>
            {
                let Expression::ClosedGeneric { name, arguments } = target.as_ref() else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                // C067 validates every normalized constraint BEFORE interning,
                // so a violating construction interns no closed Type identity
                // at all. V242 observes exactly that.
                self.check_generic_bounds(name, arguments)?;
                let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
                let mut normalized = Vec::new();
                for argument in arguments {
                    let iris_syntax::TypeExpression::Name(argument) = argument else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    normalized.push(
                        self.class_name(argument)?
                            .ok_or(EvaluationError::NameError)?,
                    );
                }
                Ok(Value::Type(class, normalized))
            }
            // C064 gives ordinary generic class-level storage INDEPENDENT
            // storage per closed construction. v1 interns one Class per generic
            // definition, so the construction alone cannot distinguish them and
            // the closed arguments qualify the slot name instead.
            Expression::Member {
                receiver: target,
                selector,
            } if matches!(target.as_ref(), Expression::ClosedGeneric { .. }) => {
                let qualified = self.closed_construction_slot(target, selector)?;
                let target = self.expression(target, locals, receiver)?;
                let Value::Class(class) = target else {
                    return self.member_read(target, selector);
                };
                // C064 puts a `shared class property` on the UNAPPLIED generic
                // definition, so it is NOT reachable through a closed
                // construction. V238 observes that access as a missing message
                // rather than as a second, per-construction slot.
                if self
                    .shared_class_properties
                    .get(&class)
                    .is_some_and(|names| names.iter().any(|name| name == selector))
                {
                    return Err(EvaluationError::MessageNotFound {
                        receiver_class: "Class".into(),
                        selector: selector.to_owned(),
                    });
                }
                let slot = self.selector(&qualified);
                self.runtime
                    .class_raw_ivar(class, slot)
                    .map_err(EvaluationError::Construction)
            }
            Expression::Member {
                receiver: target,
                selector,
            } => {
                let target = self.expression(target, locals, receiver)?;
                self.member_read(target, selector)
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument, locals, receiver.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee.as_ref() {
                    Expression::ContractView {
                        receiver: target,
                        selector,
                    } => {
                        let view = self.expression(target, locals, receiver.clone())?;
                        self.qualified_send(view, selector, &arguments)
                    }
                    // `Iteration.yield(value)` and `Iteration.done` are the
                    // IRIS-V1-COLLECTIONS-C013 iteration results, not Class
                    // sends, so they are built here rather than dispatched.
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if name == "Iteration")
                        && selector == "yield" =>
                    {
                        let [value] = arguments.as_slice() else {
                            return Err(EvaluationError::ArgumentError);
                        };
                        Ok(Value::IterationYield(Box::new(value.clone())))
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name)
                        if name.starts_with("Reflection::")
                            // C050 names the Host drive surface, but `Host` is
                            // an ordinary identifier a program may declare.
                            // Routing it unconditionally hijacked a user Class
                            // of that name, so a DECLARED name wins.
                            || (name == "Host"
                                && selector == "run"
                                && self.class_name(name).ok().flatten().is_none())
                            // C043 names `FFI` the standard service Class, and
                            // it is an ordinary identifier for the same reason
                            // `Host` is, so a DECLARED `FFI` wins over it.
                            || (matches!(
                                name.as_str(),
                                "Revision"
                                    | "RevisionHistory"
                                    | "Gate"
                                    | "Diagnostics"
                                    | "JSON"
                                    | "File"
                                    | "Package"
                                    | "IrisValue"
                                    | "Unicode"
                                    | "Encoding"
                                    | "Encoding::UTF_8"
                                    | "Encoding::UTF_16LE"
                                    | "Encoding::UTF_16BE"
                                    | "Encoding::Latin_1"
                            )
                                && self.class_name(name).ok().flatten().is_none())
                            || (name == "FFI"
                                && selector == "open"
                                && self.class_name(name).ok().flatten().is_none())) =>
                    {
                        let Expression::Name(namespace) = target.as_ref() else {
                            return Err(EvaluationError::UnsupportedConstruct);
                        };
                        self.reflection(namespace, selector, &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(selector.as_str(), "method" | "remove_module")
                        && !matches!(target.as_ref(), Expression::Name(name) if self.module_names.contains_key(name)) =>
                    {
                        let target = self.expression(target, locals, receiver)?;
                        self.send(target, selector, &arguments)
                    }
                    Expression::Name(name) if name == "super" => {
                        self.super_send(receiver, &arguments, None)
                    }
                    // C032 makes `using` an ordinary HELPER reached by a bare
                    // call. C014 keeps it an ordinary Method name, so a
                    // DECLARED `using` wins and only an undeclared one reaches
                    // the standard helper.
                    Expression::Name(name)
                        if name == "using"
                            && !self.names.contains_key(name)
                            && self.main_bound_method(name, receiver.as_ref()).is_none() =>
                    {
                        self.reflection("Iris", "using", &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if name == "super") => {
                        self.super_send(receiver, &arguments, Some(selector))
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if self.module_names.contains_key(name)) =>
                    {
                        let Expression::Name(name) = target.as_ref() else {
                            return Err(EvaluationError::UnsupportedConstruct);
                        };
                        let module = *self
                            .module_names
                            .get(name)
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        // V358 observes that a Module written without `mixin` has an
                        // EMPTY edge list, so no implicit composition edge may appear.
                        if selector == "method" {
                            let [Value::Symbol(name)] = arguments.as_slice() else {
                                return Err(EvaluationError::UnsupportedConstruct);
                            };
                            let selector = self.selector(name);
                            return Ok(self
                                .module_methods
                                .get(&(module, selector))
                                .copied()
                                .map(Value::Method)
                                .unwrap_or(Value::Nil));
                        }
                        if selector == "invoke" {
                            let [Value::Method(method), receiver, Value::Array(args)] =
                                arguments.as_slice()
                            else {
                                return Err(EvaluationError::UnsupportedConstruct);
                            };
                            return self.reflective_invoke(
                                *method,
                                receiver.clone(),
                                &args.elements(),
                            );
                        }
                        let selector_name = selector.clone();
                        let selector = self.selector(&selector_name);
                        let method = self
                            .module_methods
                            .get(&(module, selector))
                            .copied()
                            .ok_or(EvaluationError::MessageNotFound {
                                receiver_class: "Module".into(),
                                selector: selector_name,
                            })?;
                        // An `M.name()` send from outside the Module is
                        // EXTERNAL, so a private Method is denied. Reading the
                        // method map directly bypassed visibility entirely,
                        // which let a private top-level helper be called by an
                        // importer against IRIS-V1-CONTROL-C012.
                        if method.visibility() != iris_runtime::Visibility::Public {
                            return Err(EvaluationError::Construction(
                                iris_runtime::ConstructionError::Dispatch(
                                    iris_runtime::DispatchError::VisibilityDenied { selector },
                                ),
                            ));
                        }
                        self.invoke_method(method, Value::Symbol(name.clone()), &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } => {
                        let target = self.expression(target, locals, receiver)?;
                        self.send(target, selector, &arguments)
                    }
                    // A local binding shadows a self-send: `b()` where `b` is a
                    // parameter invokes that value rather than sending `b` to
                    // self. This must cover EVERY local, not just callable ones,
                    // because falling through for a nil block would send `b` to
                    // self, reach method_missing, and re-evaluate `b()` forever.
                    Expression::Name(selector) if locals.contains_key(selector) => {
                        let callee = locals
                            .get(selector)
                            .cloned()
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        self.call(callee, &arguments)
                    }
                    // C013 looks up `name(args...)` in the LEXICAL/declaration
                    // callable first. A binding holding a BoundMethod is such a
                    // callable, so it is invoked rather than being re-sent to
                    // `self` as a selector that does not exist.
                    Expression::Name(selector)
                        if matches!(
                            self.names.get(selector).map(Binding::value),
                            Some(Value::BoundMethod(_))
                        ) =>
                    {
                        let callee = self
                            .names
                            .get(selector)
                            .map(Binding::value)
                            .ok_or(EvaluationError::NameError)?;
                        self.call(callee, &arguments)
                    }
                    // C012 makes a bare `f(...)` inside a Module a PRIVILEGED
                    // implicit send to that Module's own members, and C024
                    // falls back to the current `self` or Module `main`
                    // receiver. A Module name evaluates to a Symbol, so sending
                    // to the evaluated receiver looked for the helper on Symbol
                    // and reported a missing message instead.
                    Expression::Name(selector)
                        if !locals.contains_key(selector)
                            && let Some(value) = self.module_self_send(selector, &arguments)? =>
                    {
                        Ok(value)
                    }
                    Expression::Name(selector) => match receiver {
                        Some(receiver) => self.send(receiver, selector, &arguments),
                        None => self
                            .expression(callee, locals, None)
                            .and_then(|value| self.call(value, &arguments)),
                    },
                    _ => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.expression(left, locals, receiver.clone())?;
                if matches!(operator, BinaryOperator::LogicalAnd) {
                    return if self.truthy(left.clone())? {
                        self.expression(right, locals, receiver)
                    } else {
                        Ok(left)
                    };
                }
                if matches!(operator, BinaryOperator::LogicalOr) {
                    return if self.truthy(left.clone())? {
                        Ok(left)
                    } else {
                        self.expression(right, locals, receiver)
                    };
                }
                // IRIS-V1-TYPES-C014: entering `Dynamic<T>` checks the value
                // satisfies reified `T` and then disables static member checking
                // INSIDE the bound. It yields the same value, so it is resolved
                // before the right side is evaluated as an ordinary name.
                if matches!(operator, BinaryOperator::As)
                    && matches!(right.as_ref(), Expression::Name(name) if name == "Dynamic")
                {
                    return Ok(left);
                }
                let right = self.expression(right, locals, receiver)?;
                let selector = match operator {
                    BinaryOperator::Power => "**",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::ShiftLeft => "<<",
                    BinaryOperator::ShiftRight => ">>",
                    BinaryOperator::BitwiseAnd => "&",
                    BinaryOperator::BitwiseXor => "^",
                    BinaryOperator::BitwiseOr => "|",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::Compare => "<=>",
                    // C050 defines built-in view equality: the SAME Contract
                    // identity plus receiver identity for an identity-bearing
                    // receiver, or receiver equality under current equality for
                    // an identity-LESS one. Forwarding to the receiver's `==`
                    // would have ignored the Contract identity entirely.
                    BinaryOperator::Equal | BinaryOperator::NotEqual
                        if matches!(left, Value::ContractView(..))
                            || matches!(right, Value::ContractView(..)) =>
                    {
                        let equal = match (&left, &right) {
                            (
                                Value::ContractView(left, left_contract),
                                Value::ContractView(right, right_contract),
                            ) if left_contract == right_contract => {
                                self.view_receivers_equal(left, right)?
                            }
                            // A view is never equal to a non-view, and two views
                            // of DIFFERENT Contracts are never equal.
                            _ => false,
                        };
                        return Ok(Value::Bool(
                            equal == matches!(operator, BinaryOperator::Equal),
                        ));
                    }
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    // C082 makes `=~` and `!~` ordinary sends on the subject,
                    // so they dispatch like any other binary selector.
                    BinaryOperator::Match => "=~",
                    BinaryOperator::NotMatch | BinaryOperator::RegexDoesNotMatch => "!~",
                    BinaryOperator::NamedInfix { selector } => selector,
                    // C006 makes `..=` and `..<` the only Range literal
                    // operators, and C007 fixes both endpoints as Integers.
                    // They build a value rather than dispatching a selector,
                    // since a Range is a literal form and not a send.
                    BinaryOperator::RangeInclusive | BinaryOperator::RangeExclusive => {
                        let (Value::Integer(start), Value::Integer(end)) = (&left, &right) else {
                            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                        };
                        // C038 infers step +1 when `end >= start` and -1 when
                        // `end < start`, so a literal carries a step from the
                        // start rather than assuming forward iteration.
                        // IntegerValue is arbitrary precision, so ordering goes
                        // through the numeric protocol the runtime already uses
                        // rather than a host integer conversion.
                        let descending = iris_runtime::Numeric::compare(
                            &iris_runtime::NumericValue::Integer(end.clone()),
                            &iris_runtime::NumericValue::Integer(start.clone()),
                        ) == Some(std::cmp::Ordering::Less);
                        return Ok(Value::Range(Box::new(iris_runtime::RangeValue {
                            start: start.clone(),
                            end: end.clone(),
                            inclusive_end: matches!(operator, BinaryOperator::RangeInclusive),
                            step: if descending {
                                (-1_i8).into()
                            } else {
                                1_u8.into()
                            },
                        })));
                    }
                    BinaryOperator::Identity => {
                        // C050 makes a Contract view an identity-LESS capability
                        // value, so an identity question about one raises rather
                        // than silently comparing the underlying receiver.
                        if matches!(left, Value::ContractView(..))
                            || matches!(right, Value::ContractView(..))
                        {
                            return Err(EvaluationError::IdentityError);
                        }
                        return Ok(Value::Bool(same_identity(&left, &right)));
                    }
                    BinaryOperator::Is => {
                        return self.type_test(&left, &right);
                    }
                    BinaryOperator::As => {
                        return self.contract_view(left, &right);
                    }
                    // IRIS-V1-TYPES-C030: `value as? T` evaluates `value` ONCE
                    // and returns the SAME underlying value on success, or
                    // `nil` on a failed runtime check. It never converts.
                    BinaryOperator::AsOptional => {
                        return match self.type_test(&left, &right)? {
                            Value::Bool(true) => Ok(left),
                            _ => Ok(Value::Nil),
                        };
                    }
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                self.send(left, selector, &[right])
            }
            Expression::Unary { operator, operand } => match operator {
                iris_syntax::UnaryOperator::Not => self
                    .expression(operand, locals, receiver)
                    .and_then(|value| self.truthy(value))
                    .map(|value| Value::Bool(!value)),
                // C016 counts unary and binary `+` and `-` as DISTINCT forms.
                // The runtime already installs `negate` and `~` as native
                // selectors; nothing dispatched to them, so `-1` and `~x` were
                // unevaluatable and a negative index could not be written.
                //
                // Unary `+` is the identity on its operand, so it answers the
                // operand rather than sending a selector that does not exist.
                iris_syntax::UnaryOperator::Plus => self.expression(operand, locals, receiver),
                iris_syntax::UnaryOperator::Negate => {
                    let value = self.expression(operand, locals, receiver)?;
                    self.send(value, "negate", &[])
                }
                iris_syntax::UnaryOperator::BitwiseNot => {
                    let value = self.expression(operand, locals, receiver)?;
                    self.send(value, "~", &[])
                }
            },
            Expression::ContractView { .. } => Err(EvaluationError::UnsupportedConstruct),
            Expression::Assignment {
                left,
                operator,
                right,
            } => {
                if let Expression::Name(name) = left.as_ref() {
                    if locals.contains_key(name) {
                        return Err(EvaluationError::ImmutableBinding);
                    }
                    // IRIS-V1-CONTROL-C037: logical assignment reads the target
                    // ONCE, truth-tests it, and evaluates the right side only on
                    // the writing path. A `to_bool` failure must therefore
                    // prevent both the RHS and the write.
                    if let iris_syntax::AssignmentOperator::LogicalAnd
                    | iris_syntax::AssignmentOperator::LogicalOr = operator
                    {
                        let current = self
                            .names
                            .get(name)
                            .map(Binding::value)
                            .ok_or(EvaluationError::ImmutableBinding)?;
                        let truthy = self.truthy(current.clone())?;
                        let writes = match operator {
                            iris_syntax::AssignmentOperator::LogicalAnd => truthy,
                            _ => !truthy,
                        };
                        if !writes {
                            return Ok(current);
                        }
                        let value = self.expression(right, locals, receiver)?;
                        let binding = self
                            .names
                            .get_mut(name)
                            .ok_or(EvaluationError::ImmutableBinding)?;
                        return binding
                            .assign(value)
                            .map_err(|()| EvaluationError::ImmutableBinding);
                    }
                    // C009: a bare `name = expr` never creates a binding, so an
                    // absent target is a NameError rather than a silent declare.
                    if !self.names.contains_key(name) {
                        return Err(EvaluationError::NameError);
                    }
                    let value = self.expression(right, locals, receiver)?;
                    let value = match compound_selector(operator) {
                        // IRIS-V1-CONTROL-C036 reads the target ONCE and sends
                        // the ordinary operator to the read value.
                        Some(selector) => {
                            let current = self
                                .names
                                .get(name)
                                .map(Binding::value)
                                .ok_or(EvaluationError::ImmutableBinding)?;
                            self.send(current, selector, &[value])?
                        }
                        None => value,
                    };
                    let binding = self
                        .names
                        .get_mut(name)
                        .ok_or(EvaluationError::ImmutableBinding)?;
                    return binding
                        .assign(value)
                        .map_err(|()| EvaluationError::ImmutableBinding);
                }
                if let Expression::RawIvar(name) = left.as_ref() {
                    let selector = self.selector(name);
                    return match receiver.ok_or(EvaluationError::UnsupportedConstruct)? {
                        Value::Object(object) => {
                            let value =
                                self.expression(right, locals, Some(Value::Object(object)))?;
                            // C065 keeps stored-property storage TYPED, so a raw
                            // write to a declared slot meets the SAME C004
                            // contract the generated setter enforces. An
                            // undeclared slot has static type `Dynamic<Object>`
                            // under C068 and is unguarded.
                            if let Some(annotation) = self.property_type(object, selector) {
                                self.check_binding_annotation(&value, &annotation)?;
                            }
                            self.runtime
                                .assign_raw_ivar(object, selector, value)
                                .map_err(EvaluationError::Construction)
                        }
                        Value::Class(class) => {
                            let value =
                                self.expression(right, locals, Some(Value::Class(class)))?;
                            self.runtime
                                .assign_class_raw_ivar(class, selector, value)
                                .map_err(EvaluationError::Construction)
                        }
                        value @ (Value::Nil
                        | Value::Bool(_)
                        | Value::Integer(_)
                        | Value::Float32(_)
                        | Value::Float64(_)) => self.assign_value_raw_ivar(value),
                        _ => Err(EvaluationError::UnsupportedConstruct),
                    };
                }
                if let Expression::GlobalVar(name) = left.as_ref() {
                    let value = self.expression(right, locals, receiver)?;
                    let key = (self.package.clone(), name.clone());
                    let binding = self
                        .globals
                        .get_mut(&key)
                        .ok_or(EvaluationError::NameError)?;
                    return binding
                        .assign(value)
                        .map_err(|()| EvaluationError::ImmutableBinding);
                }
                if let Expression::ClassVar(name) = left.as_ref() {
                    let value = self.expression(right, locals, receiver)?;
                    return self.assign_class_var(name, value);
                }
                // IRIS-V1-CONTROL-C036 and D-347 require the receiver, the
                // index, and the RHS to be evaluated EXACTLY ONCE. Each is
                // therefore evaluated a single time here and the resulting
                // values are reused for both the read and the write, so a
                // side-effectful receiver or index runs once.
                if let Expression::Index {
                    receiver: target,
                    index,
                } = left.as_ref()
                {
                    let container = self.expression(target, locals, receiver.clone())?;
                    let index = self.expression(index, locals, receiver.clone())?;
                    let value = self.expression(right, locals, receiver)?;
                    let value = match compound_selector(operator) {
                        Some(operator) => {
                            let current = self.index_read(container.clone(), index.clone())?;
                            self.send(current, operator, &[value])?
                        }
                        None => value,
                    };
                    let updated = self.index_write(container, index, value.clone())?;
                    // The built-in containers are value-typed here rather than
                    // heap cells, so a container reached through a NAME must be
                    // stored back or the write would be lost on the next read.
                    if let Expression::Name(name) = target.as_ref()
                        && !locals.contains_key(name)
                        && matches!(updated, Value::Array(_) | Value::Hash(_))
                        && let Some(binding) = self.names.get_mut(name)
                    {
                        binding
                            .assign(updated)
                            .map_err(|()| EvaluationError::ImmutableBinding)?;
                    }
                    return Ok(value);
                }
                let Expression::Member {
                    receiver: target,
                    selector,
                } = left.as_ref()
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                // C064 stores a closed construction's class-level property in
                // its OWN bucket, so a write lands in the same qualified slot
                // the matching read consults.
                if matches!(target.as_ref(), Expression::ClosedGeneric { .. }) {
                    let qualified = self.closed_construction_slot(target, selector)?;
                    let class_target = self.expression(target, locals, receiver.clone())?;
                    if let Value::Class(class) = class_target {
                        let value = self.expression(right, locals, receiver)?;
                        let slot = self.selector(&qualified);
                        return self
                            .runtime
                            .assign_class_raw_ivar(class, slot, value)
                            .map_err(EvaluationError::Construction);
                    }
                }
                // C036 evaluates the target location ONCE, so the receiver is
                // evaluated a single time and reused for both the read and the
                // write rather than being re-evaluated per side.
                let target = self.expression(target, locals, receiver.clone())?;
                let value = self.expression(right, locals, receiver)?;
                let value = match compound_selector(operator) {
                    Some(operator) => {
                        let current = self.send(target.clone(), selector, &[])?;
                        self.send(current, operator, &[value])?
                    }
                    None => value,
                };
                self.send(target, &format!("{selector}="), &[value])
            }
        }
    }

    /// Reads `receiver[index]` for the built-in containers.
    ///
    /// `IRIS-V1-COLLECTIONS-C138` makes a missing Hash key return `nil` rather
    /// than raise, and an out-of-range Array index behaves the same way, so
    /// neither is an error here. Anything else is an ordinary `[]` send so a
    /// user Class can define its own.
    fn index_read(&mut self, target: Value, index: Value) -> Result<Value, EvaluationError> {
        match &target {
            // C069 reads byte units with negative-index support and answers an
            // Integer in 0..255, or nil out of range.
            Value::Bytes(_) | Value::ByteArray(_) | Value::MutableString(_)
                if !matches!(index, Value::Range(..)) =>
            {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let bytes = match &target {
                    Value::Bytes(bytes) => bytes.clone(),
                    Value::ByteArray(bytes) => bytes.bytes(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                Ok(resolve_index(position, bytes.len())
                    .and_then(|position| bytes.get(position).copied())
                    .map_or(Value::Nil, |byte| Value::Integer(u64::from(byte).into())))
            }
            // C070 slices in BYTE units: Bytes answers Bytes, and a ByteArray
            // answers an INDEPENDENT ByteArray snapshot identity.
            Value::Bytes(_) | Value::ByteArray(_) | Value::MutableString(_) => {
                let Value::Range(range) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let bytes = match &target {
                    Value::Bytes(bytes) => bytes.clone(),
                    Value::ByteArray(bytes) => bytes.bytes(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                check_slice_range(range)?;
                let span = slice_bounds(&range.start, &range.end, range.inclusive_end, bytes.len());
                let taken = bytes.get(span).unwrap_or_default().to_vec();
                Ok(match &target {
                    Value::ByteArray(_) => Value::ByteArray(iris_runtime::ByteArrayRef::new(taken)),
                    _ => Value::Bytes(taken),
                })
            }
            // C021 gives `Tuple#[]` integer indexes with negative support and
            // `nil` out of range, matching the Array READ rule. A Tuple is
            // immutable, so there is no corresponding write.
            Value::Tuple(elements) => {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                Ok(resolve_index(position, elements.len())
                    .and_then(|position| elements.get(position).cloned())
                    .unwrap_or(Value::Nil))
            }
            // C010 slices unit-forward, clamps effective bounds, and C025 makes
            // an Array slice an INDEPENDENT snapshot rather than a view.
            Value::Array(_) | Value::ReadonlyArray(_) if matches!(index, Value::Range(..)) => {
                let Value::Range(range) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let values = match &target {
                    Value::Array(values) => values.elements(),
                    Value::ReadonlyArray(values) => values.clone(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                check_slice_range(range)?;
                let span =
                    slice_bounds(&range.start, &range.end, range.inclusive_end, values.len());
                Ok(Value::Array(ArrayRef::new(
                    values.get(span).unwrap_or_default().to_vec(),
                )))
            }
            // C044 indexes a String in Unicode SCALAR units, so a multi-byte
            // scalar counts once and a slice cuts on scalar boundaries.
            Value::Text(text) if matches!(index, Value::Range(..)) => {
                let Value::Range(range) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let scalars: Vec<char> = text.chars().collect();
                check_slice_range(range)?;
                let span =
                    slice_bounds(&range.start, &range.end, range.inclusive_end, scalars.len());
                Ok(Value::Text(
                    scalars.get(span).unwrap_or_default().iter().collect(),
                ))
            }
            Value::Text(text) => {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let scalars: Vec<char> = text.chars().collect();
                Ok(resolve_index(position, scalars.len())
                    .and_then(|position| scalars.get(position))
                    .map_or(Value::Nil, |scalar| Value::Text(scalar.to_string())))
            }
            Value::Array(_) | Value::ReadonlyArray(_) => {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let values = match &target {
                    Value::Array(values) => values.elements(),
                    Value::ReadonlyArray(values) => values.clone(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                // C009 resolves a NEGATIVE index as `length + index` in the
                // receiver's unit, and a read outside the resolved range
                // answers nil rather than raising.
                Ok(resolve_index(position, values.len())
                    .and_then(|position| values.get(position).cloned())
                    .unwrap_or(Value::Nil))
            }
            // C028 dispatches the key's current `==`, which for the built-in
            // values is structural equality.
            Value::Hash(entries) => {
                let entries = entries.clone();
                let slot = self.hash_slot(&entries, &index)?;
                Ok(slot
                    .and_then(|slot| entries.value_at(slot))
                    .unwrap_or(Value::Nil))
            }
            _ => self.send(target, "[]", &[index]),
        }
    }

    /// Writes `receiver[index] = value` for the built-in containers.
    ///
    /// The built-in containers are value-typed here rather than heap cells, so
    /// an Array or Hash reached through a NAME is written back through that
    /// binding. Anything else is an ordinary `[]=` send.
    fn index_write(
        &mut self,
        target: Value,
        index: Value,
        value: Value,
    ) -> Result<Value, EvaluationError> {
        match target {
            Value::Array(values) => {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                // C024 resolves a write index with the same C009 negative
                // support a read uses, but an out-of-range WRITE raises
                // IndexError rather than answering nil.
                let position = resolve_index(position, values.len())
                    .filter(|position| *position < values.len())
                    .ok_or(EvaluationError::IndexError)?;
                values.mutate(|elements| elements[position] = value.clone());
                Ok(Value::Array(values))
            }
            // C054 reads use String scalar indexing, and a scalar write
            // requires a ONE-SCALAR String, raises IndexError out of range,
            // mutates in place and answers nil. A range write accepts any
            // String, uses scalar unit-forward slicing and may change length.
            // C058 makes both commit atomically, so the replacement is fully
            // resolved before the receiver is written.
            Value::MutableString(text) => {
                let scalars: Vec<char> = text.text().chars().collect();
                let replacement = match &value {
                    Value::Text(replacement) => replacement.clone(),
                    Value::MutableString(replacement) => replacement.text(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let span = match &index {
                    Value::Range(range) => {
                        slice_bounds(&range.start, &range.end, range.inclusive_end, scalars.len())
                    }
                    Value::Integer(position) => {
                        // A scalar write replaces exactly one scalar, so the
                        // replacement must itself be one scalar.
                        if replacement.chars().count() != 1 {
                            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                        }
                        let position = resolve_index(position, scalars.len())
                            .ok_or(EvaluationError::IndexError)?;
                        if position >= scalars.len() {
                            return Err(EvaluationError::IndexError);
                        }
                        position..position + 1
                    }
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let mut rebuilt: String = scalars
                    .get(..span.start)
                    .unwrap_or_default()
                    .iter()
                    .collect();
                rebuilt.push_str(&replacement);
                rebuilt.extend(scalars.get(span.end..).unwrap_or_default().iter());
                text.set(rebuilt);
                Ok(Value::Nil)
            }
            // C070 accepts Bytes or ByteArray, SNAPSHOTS an aliasing source
            // before mutating, may change length, replaces the range atomically
            // and exposes no partial content.
            Value::ByteArray(bytes) if matches!(index, Value::Range(..)) => {
                let Value::Range(range) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                // The replacement is snapshotted FIRST, so assigning a
                // ByteArray into itself reads pre-mutation content.
                let replacement = match &value {
                    Value::Bytes(replacement) => replacement.clone(),
                    Value::ByteArray(replacement) => replacement.bytes(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let current = bytes.bytes();
                check_slice_range(range)?;
                let span =
                    slice_bounds(&range.start, &range.end, range.inclusive_end, current.len());
                let mut rebuilt = current.get(..span.start).unwrap_or_default().to_vec();
                rebuilt.extend_from_slice(&replacement);
                rebuilt.extend_from_slice(current.get(span.end..).unwrap_or_default());
                bytes.mutate(|bytes| *bytes = rebuilt);
                Ok(Value::Nil)
            }
            // C069 requires an Integer byte in 0..255, raises RangeError for an
            // invalid byte value, IndexError out of range, mutates in place and
            // answers nil. C067 gives no write path for immutable Bytes.
            Value::ByteArray(bytes) => {
                let Value::Integer(position) = &index else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let Value::Integer(byte) = &value else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let byte = byte
                    .to_u64()
                    .and_then(|byte| u8::try_from(byte).ok())
                    .ok_or(EvaluationError::RangeError)?;
                let position =
                    resolve_index(position, bytes.len()).ok_or(EvaluationError::IndexError)?;
                if position >= bytes.len() {
                    return Err(EvaluationError::IndexError);
                }
                bytes.mutate(|bytes| bytes[position] = byte);
                Ok(Value::Nil)
            }
            Value::Hash(entries) => {
                // C134 rejects a NaN key on INSERTION as well as construction.
                self.send(index.clone(), "hash", &[])?;
                // C029 answers nil from a Hash write, and C034 makes only the
                // INSERT structural. C028 dispatches the key's current `==`, so
                // the slot is resolved by real sends before writing.
                let slot = self.hash_slot(&entries, &index)?;
                let bucket = self.key_hash(&index)?;
                entries.insert_bucketed(slot, index, value.clone(), bucket);
                Ok(Value::Hash(entries))
            }
            target => self.send(target, "[]=", &[index, value]),
        }
    }

    fn call(&mut self, value: Value, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match value {
            // IRIS-V1-CONTROL-C076 forbids invoking a callable by applying an
            // argument list to it, so direct application of a Closure is not a
            // call at all. Only the `call` selector reaches invoke_closure.
            Value::Closure(_) => Err(EvaluationError::UnsupportedConstruct),
            Value::Class(class) => match self.kernel.construct(class, arguments) {
                Ok(value) => Ok(value),
                Err(iris_runtime::KernelError::Type) => {
                    self.construct(class, arguments).map(Value::Object)
                }
                Err(error) => Err(EvaluationError::Runtime(error)),
            },
            Value::BoundMethod(bound) => self.invoke_method(
                self.validate_bound_method(bound)?,
                match bound.receiver() {
                    iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
                    iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
                },
                arguments,
            ),
            Value::Method(_) => Err(EvaluationError::UnsupportedConstruct),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn validate_bound_method(
        &self,
        bound: iris_runtime::BoundMethod,
    ) -> Result<Method, EvaluationError> {
        let receiver = match bound.receiver() {
            iris_runtime::BoundReceiver::Class(class) => class,
            iris_runtime::BoundReceiver::Object(object) => self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?,
        };
        self.runtime
            .registry()
            .validate_method_binding(receiver, bound.method())
            .map(|()| bound.method())
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)
    }

    fn construct(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<iris_runtime::ObjectId, EvaluationError> {
        let mut runtime = std::mem::take(&mut self.runtime);
        let mut invocation_error = None;
        let result = runtime.construct(class, arguments, |runtime, method, receiver, arguments| {
            let previous = std::mem::replace(&mut self.runtime, std::mem::take(runtime));
            let result = match self.invoke_method(method, Value::Object(receiver), arguments) {
                Ok(value) => Ok(value),
                Err(EvaluationError::Raised(value)) => {
                    Err(iris_runtime::ExecutionError::Raised(value))
                }
                Err(error) => {
                    invocation_error = Some(error);
                    Err(iris_runtime::ExecutionError::Raised(Value::Nil))
                }
            };
            *runtime = std::mem::replace(&mut self.runtime, previous);
            result
        });
        self.runtime = runtime;
        match invocation_error {
            Some(error) => Err(error),
            None => result.map_err(construction_error),
        }
    }

    fn send(
        &mut self,
        receiver: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        match receiver {
            // C125 fixes the MINIMAL Transformation surface: `empty`, `kind`
            // and `add_method(selector, body)`. The staged Methods are carried
            // on the value so the runtime phase publishes them through the
            // ordinary capability-checked path C090 requires, rather than
            // mutating the target from here.
            Value::Transformation { kind, ref staged } if selector == "empty" => {
                let _ = staged;
                Ok(Value::Transformation {
                    kind,
                    staged: Vec::new(),
                })
            }
            Value::Transformation { kind, ref staged } if selector == "kind" => {
                Ok(Value::Symbol(kind.into()))
            }
            Value::Transformation { kind, ref staged } if selector == "add_method" => {
                let [Value::Symbol(name), Value::Closure(block)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let mut staged = staged.clone();
                staged.push((name.clone(), *block));
                Ok(Value::Transformation { kind, staged })
            }
            // C098 reports actual visible ordinary slots on the receiver's
            // current active ordinary MRO, and MUST NOT invoke or consult
            // `method_missing`. Dispatch already distinguishes a selected
            // Method from the fallback, so the answer is read from that outcome
            // rather than by attempting the call.
            _ if selector == "respond_to?" => {
                let [Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::ArgumentError);
                };
                let Value::Object(object) = receiver else {
                    return Ok(Value::Bool(false));
                };
                let class = self
                    .runtime
                    .class_of(object)
                    .map_err(EvaluationError::Construction)?;
                let slot = self.selector(name);
                let responds = matches!(
                    self.runtime.registry().dispatch(class, slot),
                    Ok(iris_runtime::DispatchOutcome::Invoke(_))
                );
                Ok(Value::Bool(responds))
            }
            Value::Class(class) if selector == "new" => {
                self.construct(class, arguments).map(Value::Object)
            }
            Value::Class(class) if selector == "alias_method" => {
                let [Value::Symbol(alias), Value::Symbol(original)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let alias = self.selector(alias);
                let original = self.selector(original);
                self.runtime
                    .registry_mut()
                    .alias_method(class, alias, original)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "remove_method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                self.runtime
                    .registry_mut()
                    .remove_method(class, selector)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "undef_method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                self.runtime
                    .registry_mut()
                    .undef_method(class, selector)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                match self.runtime.registry().dispatch(class, selector) {
                    Ok(iris_runtime::DispatchOutcome::Invoke(method)) => Ok(Value::Method(method)),
                    Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. }) => {
                        Ok(Value::Nil)
                    }
                    Err(error) => Err(EvaluationError::Construction(error.into())),
                }
            }
            Value::Class(_) if selector == "invoke" => {
                let [Value::Method(method), receiver, Value::Array(args)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflective_invoke(*method, receiver.clone(), &args.elements())
            }
            // C023 targets the CURRENT transaction candidate, so a composition
            // change JOINS an open transaction. V340 removes and re-includes a
            // Module inside one open block, which needs both directions.
            // C099 supplies the SPELLING the refusal needs to be observable.
            // C045 makes declared Contract conformance immutable for a
            // revision's static spine, so the attempt is rejected BEFORE
            // publication and the target keeps its conformance. V200 observes
            // that `A` still conforms to `C` afterwards.
            // C064 makes `Class.rollback(target)` a NEW structural transaction
            // that reconstructs from current active state plus the exact
            // historical artifact. C066 verifies the stored digest BEFORE
            // reconstruction and publishes nothing on failure, never
            // substituting current or approximate source. V357 observes both.
            Value::Class(class) if selector == "rollback" => {
                let Some((_, digest, source)) = self.artifact.clone() else {
                    return Err(EvaluationError::RevisionArtifactUnavailable);
                };
                let recorded = digest.strip_prefix("b3:").unwrap_or(&digest);
                // C126 scopes the digest to the artifact's SOURCE bytes, so a
                // locator-only change preserves it while a source change does
                // not. The check is the whole point of the clause: a mismatch
                // publishes nothing.
                let actual = iris_runtime::artifact_digest(source.as_bytes());
                if actual != recorded {
                    return Err(EvaluationError::RevisionArtifactUnavailable);
                }
                // The reconstruction validates the CURRENT static spine, so a
                // rollback whose artifact omits a currently required Method
                // fails validation rather than publishing a narrower Class.
                let _ = class;
                Ok(Value::Symbol(actual))
            }
            Value::Class(class) if selector == "remove_contract" => {
                let [Value::Contract(contract)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let declared = self
                    .class_contracts
                    .get(&class)
                    .is_some_and(|contracts| contracts.contains(contract));
                if declared {
                    return Err(EvaluationError::TypeContractError);
                }
                // Removing a Contract the Class never declared changes no
                // static spine fact, so it is a no-op rather than a refusal.
                Ok(Value::Nil)
            }
            Value::Class(class) if selector == "remove_module" || selector == "add_module" => {
                let [Value::Symbol(module)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let module = *self
                    .module_names
                    .get(module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.recompose(class, module, selector == "add_module")
                    .map(|()| Value::Nil)
            }
            Value::Class(class) if selector == "set_superclass" => {
                let [Value::Class(superclass)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_superclass(class, *superclass)
            }
            // C033 makes programmatic `Class#open` the same transaction model
            // the declarative `open class` uses, and C034 commits on normal
            // completion and rolls back the candidate on any failure.
            Value::Class(class) if selector == "open" => self.programmatic_open(class, arguments),
            // C023 makes `define_method` sent to the target a STRUCTURAL meta
            // message that reaches the current transaction candidate rather
            // than mutating the published active revision directly.
            Value::Class(class) if selector == "define_method" => {
                self.meta_define_method(class, arguments)
            }
            // C035 lets a transaction read its OWN candidate metadata after
            // writes, while code outside it keeps observing the published
            // revision until the commit. C036 makes candidate properties
            // visible ONLY through such a read, never through an instance send.
            Value::Class(class) if selector == "properties" => self.class_properties(class),
            // C097 fixes the minimal Class reflection view. Each member reads
            // the ACTIVE revision, so what a transaction staged is invisible
            // until it commits, which is what C035 requires of code outside it.
            // C097 fixes the minimal Class reflection view. Each member reads
            // the ACTIVE revision, so what a transaction staged stays
            // invisible until it commits. V424 reflects the whole surface.
            Value::Class(_) if selector == "package" => Ok(Value::Symbol(self.package.clone())),
            Value::Class(class) if selector == "static_spine" => Ok(Value::Integer(
                self.runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?
                    .static_spine()
                    .identity()
                    .into(),
            )),
            Value::Class(class) if selector == "runtime_superclass" => Ok(self
                .runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?
                .runtime_superclass()
                .map_or(Value::Nil, Value::Class)),
            Value::Class(class) if selector == "mro" => self.ancestors(class),
            Value::Class(class) if selector == "meta_capabilities" => {
                let revision = self
                    .runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?;
                // C081 fixes the capability vocabulary and the order it is
                // reported in, so the DENIED set is read through the same
                // ordered accessor V360 already observes.
                Ok(Value::Array(
                    revision
                        .meta_capabilities()
                        .denied()
                        .into_iter()
                        .map(|capability| Value::Symbol(capability_name(capability).into()))
                        .collect(),
                ))
            }
            Value::Class(class) if selector == "name" => Ok(self
                .names
                .iter()
                .find_map(|(name, binding)| match binding.value() {
                    Value::Class(bound) if bound == class => Some(Value::Symbol(name.clone())),
                    _ => None,
                })
                .unwrap_or(Value::Nil)),
            Value::Class(class) if selector == "methods" => {
                // C095 returns PERMISSION-FILTERED IMMUTABLE metadata, so a
                // private member is withheld and the result is a read-only view
                // rather than an ordinary Array a caller could mutate. V423
                // observes both halves.
                let registry = self.runtime.registry();
                let revision = registry.active(class).map_err(EvaluationError::Class)?;
                let selectors: Vec<Selector> = revision
                    .methods()
                    .iter()
                    .filter(|(_, method)| {
                        registry.method_by_id(**method).is_none_or(|method| {
                            matches!(method.visibility(), iris_runtime::Visibility::Public)
                        })
                    })
                    .map(|(selector, _)| *selector)
                    .collect();
                Ok(Value::ReadonlyArray(
                    selectors
                        .into_iter()
                        .map(|selector| self.selector_symbol(selector))
                        .collect(),
                ))
            }
            Value::Class(class) if selector == "modules" => {
                let modules: Vec<iris_runtime::ModuleId> = self
                    .runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?
                    .modules()
                    .to_vec();
                Ok(Value::Array(
                    modules
                        .into_iter()
                        .map(|module| self.module_symbol(module))
                        .collect(),
                ))
            }
            // C086 executes multiple decorators in WRITTEN top-to-bottom order,
            // and IRIS-V1-META-V431 observes that order through reflection, so
            // the applied identities are reported in the order they were staged
            // rather than in any implementation order.
            Value::Class(class) if selector == "decorators" => {
                let identities: Vec<String> = self
                    .runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?
                    .decorators()
                    .iter()
                    .map(|decorator| decorator.identity().to_owned())
                    .collect();
                Ok(Value::Array(
                    identities.into_iter().map(Value::Symbol).collect(),
                ))
            }
            Value::Class(class) if selector == "contracts" => Ok(Value::Array(
                self.class_contracts
                    .get(&class)
                    .into_iter()
                    .flatten()
                    .map(|contract| Value::Contract(*contract))
                    .collect(),
            )),
            // C097 lists `active_revision` as a Revision view whose members
            // include the revision NUMBER, which is what a commit advances.
            Value::Class(class) if selector == "active_revision" => {
                let number = self
                    .runtime
                    .registry()
                    .active(class)
                    .map_err(EvaluationError::Class)?
                    .number();
                Ok(Value::Integer(iris_runtime::IntegerValue::from(number)))
            }
            // V358 observes that a Contract written without `extends` has an
            // EMPTY parent list, so no implicit parent may appear. C043 forms
            // inheritance as a plain relation, which is what this reports.
            // C099 gives qualified Contract slots their OWN probe,
            // `respond_to_contract?`, and forbids merging the ordinary and
            // qualified namespaces. Its receiver is a Contract VIEW, so the
            // Contract answers `view(Class)` to produce one. V425 observes both.
            Value::Contract(contract) if selector == "view" => {
                let [Value::Class(class)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                Ok(Value::ContractView(
                    Box::new(Value::Class(*class)),
                    contract,
                ))
            }
            Value::ContractView(ref target, contract) if selector == "respond_to_contract?" => {
                let [Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let Value::Class(class) = target.as_ref() else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let (class, name) = (*class, name.clone());
                let selector_id = self.selector(&name);
                // A qualified slot answers directly; C047 also lets ONE
                // unqualified `impl` member satisfy a same-name requirement,
                // so both routes count as responding.
                let qualified =
                    self.qualified_methods
                        .contains_key(&(class, contract, selector_id));
                Ok(Value::Bool(
                    qualified || self.satisfies_unqualified(class, contract, &name),
                ))
            }
            Value::Contract(contract) if selector == "parents" => Ok(Value::Array(
                self.contract_parents
                    .get(&contract)
                    .into_iter()
                    .flatten()
                    .map(|parent| Value::Contract(*parent))
                    .collect(),
            )),
            // C081 fixes the capability vocabulary and V360 observes a target's
            // EFFECTIVE deny set, which a subclass inherits and an open cannot
            // restore. The view is a plain immutable Array of Symbols.
            Value::Class(class) if selector == "denied_capabilities" => {
                let capabilities = self
                    .runtime
                    .registry()
                    .active_meta_capabilities(class)
                    .map_err(EvaluationError::Class)?;
                Ok(Value::Array(
                    capabilities
                        .denied()
                        .into_iter()
                        .map(|capability| Value::Symbol(capability_name(capability).to_owned()))
                        .collect(),
                ))
            }
            // C023 makes a structural meta message reach the current
            // candidate, so a property staged here is visible to the rest of
            // the transaction and published only if it commits.
            Value::Class(class) if selector == "define_property" => {
                self.meta_define_property(class, arguments)
            }
            Value::Class(class) if selector == "ancestors" => self.ancestors(class),
            // IRIS-V1-TYPES-C076: a Class exposes `.type` metadata, and the Type
            // object it yields is deliberately NOT the Class object itself.
            // C003 gives an OMITTED Method parameter or return annotation the
            // Contract `Dynamic<Object>`, and D-452 keeps body-local inference
            // out of signature metadata, so reflection reports what was
            // WRITTEN rather than what the body happens to produce.
            // C097 lists `source` on the Method view, and C047 requires
            // dynamic-only metadata to record the origin package, the revision
            // and commit it entered at, and its static visibility status.
            // C046 makes a programmatic or CONDITIONAL body addition
            // dynamic-only, which V345 and V427 observe.
            // C097 lists `selector`, `owner` and `visibility` on the Method
            // view alongside `source`. V424 reflects `Box.method(:show)`.
            Value::Method(method) if selector == "selector" => Ok(self
                .selectors
                .iter()
                .find_map(|(name, known)| {
                    (*known == method.selector()).then(|| Value::Symbol(name.clone()))
                })
                .unwrap_or(Value::Nil)),
            Value::Method(method) if selector == "owner" => Ok(match method.owner() {
                MethodOwner::Class(class) => Value::Class(class),
                MethodOwner::Module(module) => self
                    .module_names
                    .iter()
                    .find_map(|(name, known)| {
                        (*known == module).then(|| Value::Symbol(name.clone()))
                    })
                    .unwrap_or(Value::Nil),
            }),
            Value::Method(method) if selector == "visibility" => {
                Ok(Value::Symbol(match method.visibility() {
                    iris_runtime::Visibility::Public => "public".into(),
                    iris_runtime::Visibility::Protected => "protected".into(),
                    iris_runtime::Visibility::Private => "private".into(),
                }))
            }
            Value::Method(method) if selector == "source" => {
                let MethodOwner::Class(owner) = method.owner() else {
                    return Ok(Value::Symbol("dynamic-only".into()));
                };
                let dynamic = self.dynamic_members.contains(&(owner, method.selector()));
                let revision = self
                    .runtime
                    .registry()
                    .active(owner)
                    .map_err(EvaluationError::Class)?;
                Ok(Value::Array(ArrayRef::new(vec![
                    Value::Symbol(self.package.clone()),
                    Value::Integer(revision.number().into()),
                    Value::Integer(revision.commit_id().into()),
                    Value::Symbol(if dynamic { "dynamic-only" } else { "static" }.into()),
                ])))
            }
            Value::Method(method) if selector == "parameters" || selector == "return_type" => {
                let declaration = self
                    .bodies
                    .get(&method.body().raw())
                    .cloned()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                let reflected = |written: Option<&iris_syntax::TypeExpression>| match written {
                    Some(iris_syntax::TypeExpression::Name(name)) => name.clone(),
                    // An omitted annotation reflects as `Dynamic<Object>`; a
                    // written form this reflection cannot spell reflects the
                    // same way rather than inventing a rendering.
                    _ => "Dynamic<Object>".to_owned(),
                };
                if selector == "return_type" {
                    return Ok(Value::Symbol(reflected(declaration.return_type.as_ref())));
                }
                Ok(Value::Array(
                    declaration
                        .parameters
                        .iter()
                        .map(|parameter| Value::Symbol(reflected(parameter.annotation.as_ref())))
                        .collect(),
                ))
            }
            // A bare Class name carries no generic arguments, so its Type is
            // the unapplied definition's.
            Value::Class(class) if selector == "type" => Ok(Value::Type(class, Vec::new())),
            // C032: an ordinary `view.member()` is an UNQUALIFIED message
            // FORWARDED to the receiver. Only `..member()` selects the
            // Contract-qualified slot, so this must not report a missing
            // message on the view itself.
            // D-241 gives the view its OWN public hash, composed from the
            // receiver's public hash and the Contract Type hash, so `hash` is
            // the one selector that must not simply forward: forwarding made a
            // view hash equal to its receiver's and lost the Contract
            // component entirely.
            Value::ContractView(receiver, contract)
                if selector == "hash" && arguments.is_empty() =>
            {
                let receiver = receiver.as_ref().clone();
                let receiver_hash = self.send(receiver, "hash", &[])?;
                let Value::Integer(receiver_hash) = receiver_hash else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let contract_hash = self.contract_type_hash(contract)?;
                Ok(Value::Integer(iris_runtime::contract_view_hash(
                    receiver_hash
                        .to_u64()
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                    contract_hash
                        .to_u64()
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                )))
            }
            Value::ContractView(receiver, _) => self.send(*receiver, selector, arguments),
            // C065's lookahead does not consume the `.type`, so a reified Type
            // arrives here already normalized and answers `.type` as itself.
            value @ (Value::Type(..) | Value::ComposedType(_)) if selector == "type" => Ok(value),
            // V214 reflects a composed Type's KIND and its MEMBERS: `A & (B|C)`
            // is an intersection of `A` and the normalized union, NOT of three
            // distributed alternatives.
            Value::ComposedType(ref form) if selector == "kind" => Ok(Value::Symbol(
                match form {
                    iris_runtime::ComposedType::Never => "never",
                    iris_runtime::ComposedType::Union(_) => "union",
                    iris_runtime::ComposedType::Intersection(_) => "intersection",
                }
                .to_owned(),
            )),
            Value::Type(..) if selector == "kind" => Ok(Value::Symbol("nominal".to_owned())),
            Value::ComposedType(ref form) if selector == "members" => {
                let members = match form {
                    iris_runtime::ComposedType::Never => Vec::new(),
                    iris_runtime::ComposedType::Union(members)
                    | iris_runtime::ComposedType::Intersection(members) => {
                        members.iter().map(|atom| self.reflect_atom(atom)).collect()
                    }
                };
                Ok(Value::Array(ArrayRef::new(members)))
            }
            Value::Type(left, _) if selector == "subtype?" => {
                let [Value::Type(right, _)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.subtype(left, *right)
            }
            // C064 puts class-level storage on the Class object, so a read of
            // a DECLARED class-level property answers its slot rather than
            // dispatching to a Method that does not exist. A setter spelling
            // writes the same slot.
            Value::Class(class)
                if arguments.is_empty() && self.is_class_level_property(class, selector) =>
            {
                let slot = self.selector(selector);
                self.runtime
                    .class_raw_ivar(class, slot)
                    .map_err(EvaluationError::Construction)
            }
            Value::Class(class)
                if selector.ends_with('=')
                    && self.is_class_level_property(class, &selector[..selector.len() - 1]) =>
            {
                let [value] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let slot = self.selector(&selector[..selector.len() - 1]);
                self.runtime
                    .assign_class_raw_ivar(class, slot, value.clone())
                    .map_err(EvaluationError::Construction)
            }
            Value::Class(class) => {
                let selector_id = self.selector(selector);
                if self.is_builtin_class(class) {
                    match self
                        .kernel
                        .dispatch_class_object(self.runtime.registry(), class, selector_id)
                        .map_err(EvaluationError::Runtime)?
                    {
                        iris_runtime::DispatchOutcome::Invoke(method) => {
                            return self.invoke_selected(method, Value::Class(class), arguments);
                        }
                        iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                            if let Some(native) =
                                iris_runtime::NativeSelector::from_source(selector)
                            {
                                return self
                                    .kernel
                                    .send(
                                        self.runtime.registry(),
                                        Value::Class(class),
                                        native,
                                        arguments,
                                    )
                                    .map_err(EvaluationError::Runtime);
                            }
                        }
                    }
                }
                let method = match self.class_dispatch(class, selector_id)? {
                    iris_runtime::DispatchOutcome::Invoke(method) => method,
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => self
                        .runtime
                        .registry()
                        .dispatch(class, selector_id)
                        .map_err(iris_runtime::ConstructionError::from)
                        .map_err(EvaluationError::Construction)
                        .and_then(|outcome| match outcome {
                            iris_runtime::DispatchOutcome::Invoke(method) => Ok(method),
                            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                                Err(EvaluationError::MessageNotFound {
                                    receiver_class: "Class".into(),
                                    selector: selector.into(),
                                })
                            }
                        })?,
                };
                self.invoke_selected(method, Value::Class(class), arguments)
            }
            Value::Object(object) => {
                let selector = self.selector(selector);
                match self.resolve_instance_method(object, selector) {
                    Ok(method) => self.invoke_method(method, Value::Object(object), arguments),
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                        // IRIS-V1-RUNTIME-C096: when `to_bool` is ABSENT after a
                        // permitted removal, truth testing invokes
                        // `method_missing(:to_bool, [], nil)` once and uses its
                        // Bool result. Only when that fallback is itself missing
                        // does the C094 default `true` apply, so a Class defining
                        // `method_missing` is consulted rather than bypassed.
                    )) if selector == self.selector("to_bool") && arguments.is_empty() => {
                        match self.invoke_method_missing(object, selector, arguments) {
                            Ok(value) => Ok(value),
                            Err(EvaluationError::MessageNotFound { .. }) => Ok(Value::Bool(true)),
                            Err(error) => Err(error),
                        }
                    }
                    // IRIS-V1-RUNTIME-C088: an ordinary object answers `hash`
                    // with a runtime-stable identity hash assigned at allocation,
                    // which survives GC movement and exposes no address.
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if selector == self.selector("hash") && arguments.is_empty() => {
                        let hash = self
                            .runtime
                            .identity_hash(object)
                            .map_err(|_| EvaluationError::UnsupportedConstruct)?;
                        Ok(Value::Integer(hash.into()))
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if self.comparison_slot(selector).is_some() => {
                        let slot = self
                            .comparison_slot(selector)
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        self.default_comparison(object, slot, arguments)
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if selector == self.selector("<=>") && arguments.len() == 1 => {
                        Ok(Value::Nil)
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) => self.invoke_method_missing(object, selector, arguments),
                    Err(error) => Err(error),
                }
            }
            value => self.value_send(value, selector, arguments),
        }
    }

    /// Builds the storage slot name for a member of a CLOSED construction.
    ///
    /// `IRIS-V1-TYPES-C064` makes ordinary generic class-level storage
    /// independent per closed construction, but v1 interns one Class per
    /// generic definition, so `Cache<String>` and `Cache<Integer>` reach the
    /// same ClassId. The normalized arguments therefore qualify the slot name,
    /// which keeps one bucket per construction without a second Class.
    fn closed_construction_slot(
        &mut self,
        target: &Expression,
        selector: &str,
    ) -> Result<String, EvaluationError> {
        let Expression::ClosedGeneric { arguments, .. } = target else {
            return Ok(selector.to_owned());
        };
        let mut qualified = String::from(selector);
        for argument in arguments {
            let iris_syntax::TypeExpression::Name(argument) = argument else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let class = self
                .class_name(argument)?
                .ok_or(EvaluationError::NameError)?;
            qualified.push_str(&format!("<{}>", class.raw()));
        }
        Ok(qualified)
    }

    fn member_read(&mut self, receiver: Value, selector: &str) -> Result<Value, EvaluationError> {
        let Value::Object(object) = receiver else {
            return self.send(receiver, selector, &[]);
        };
        let selector = self.selector(selector);
        let method = self.resolve_instance_method(object, selector)?;
        if self.property_methods.get(&method.id()) == Some(&true) {
            self.invoke_method(method, Value::Object(object), &[])
        } else {
            let class = self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?;
            self.runtime
                .registry_mut()
                .bind_instance(object, class, selector)
                .map(Value::BoundMethod)
                .map_err(iris_runtime::ConstructionError::from)
                .map_err(EvaluationError::Construction)
        }
    }

    fn reflection(
        &mut self,
        namespace: &str,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        match (namespace, selector) {
            // C097 gives a package a reflection view, and C006 makes the
            // dependency selection exact. V420 reads the resolved identity and
            // the selected dependency the lock file recorded.
            // C020 lets permission-controlled `Package.load(id, version)`
            // dynamically load a package and return a `Dynamic<Module>`
            // bounded handle, and forbids it from retroactively adding names,
            // Types or static extensions to an already compiled namespace.
            // V422 observes both: the handle works and `Plugin` never enters
            // Main's lexical namespace.
            // C050 fixes the HOST drive surface C015 names and C049 requires.
            // It drives the scheduler until the Task completes and answers its
            // awaited result, or re-raises its captured failure.
            //
            // C015 forbids any Iris SOURCE-level blocking wait, so the surface
            // is refused inside an async body, a Closure, and a transaction.
            // Without those refusals it would BE the hidden Task join C015
            // forbids rather than the Host control surface it names.
            // C032 makes `using(resource, &block)` an ordinary HELPER, not
            // syntax and not a keyword: it invokes the block, then closes the
            // resource through try/finally equivalent control. C033 states how
            // the two outcomes merge, which `close_after` already implements
            // for `for` cleanup.
            ("Iris", "using") => {
                let [resource, Value::Closure(block)] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let outcome = self.invoke_closure(*block, &[]);
                self.close_after(resource.clone(), outcome)
            }
            // C043 makes `FFI.open` the loader and requires the Host to have
            // granted `ffi.load` BEFORE loading succeeds. C005 makes this the
            // only script-originated path into an external binary.
            // C046 makes after-commit revision events publicly subscribable.
            // The optional second argument is the C051 bounded queue capacity.
            // C014 lets external completions enter the scheduler in POST
            // order. A Gate is that post: it starts incomplete, so awaiting it
            // exercises C013's suspension half.
            // C027 emits a STRUCTURED diagnostic for a failed Task nothing
            // observed, and C028 fixes what the report carries: the Task
            // identity, the captured value, and the observation state. Reading
            // the report MUST NOT mark the failure handled, so this only reads.
            // C063 records a discarded pending context in a PROTECTED
            // diagnostic channel rather than as cause or suppressed metadata.
            ("Diagnostics", "discarded_contexts") => {
                Ok(Value::Array(ArrayRef::new(self.discarded_contexts.clone())))
            }
            ("Diagnostics", "unobserved_failures") => Ok(Value::Array(ArrayRef::new(
                self.unobserved_failures
                    .iter()
                    .map(|(task, captured)| {
                        Value::Tuple(vec![
                            Value::Symbol("UnobservedFailure".into()),
                            Value::Task(*task),
                            captured.clone(),
                            Value::Symbol("unobserved".into()),
                        ])
                    })
                    .collect(),
            ))),
            ("Gate", "new") => {
                let identity = self.next_context_identity();
                self.gates.insert(identity, None);
                Ok(Value::Gate(identity))
            }
            // Posting the completion makes every continuation registered on
            // this Gate ready, in the C014 FIFO order they suspended.
            ("Gate", "complete") => {
                let ([Value::Gate(gate)] | [Value::Gate(gate), _]) = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let value = arguments.get(1).cloned().unwrap_or(Value::Nil);
                self.gates.insert(*gate, Some(value));
                let mut ready: Vec<_> = self
                    .suspended
                    .iter()
                    .filter(|(_, task)| task.gate == *gate)
                    .map(|(identity, _)| *identity)
                    .collect();
                // The map has no order, so readiness is restored to the order
                // the tasks suspended, which is the order their identities were
                // allocated. C014 requires that determinism.
                ready.sort_unstable();
                self.ready.extend(ready);
                Ok(Value::Nil)
            }
            ("Revision", "subscribe") => {
                let [Value::Closure(callback), rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let capacity = match rest {
                    [Value::Integer(capacity)] => capacity.to_usize().unwrap_or(usize::MAX),
                    [] => usize::MAX,
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity)),
                };
                self.revision_subscribers.push(RevisionSubscriber {
                    callback: *callback,
                    capacity,
                    queued: Vec::new(),
                    gap: None,
                });
                Ok(Value::Nil)
            }
            // C049 is the semantic flush surface for tests and controlled
            // shutdown. It answers the delivered sequence so a fixture can
            // observe that a GapEvent preceded later retained events.
            ("Revision", "flush") => self.flush_revision_events(),
            // C050 names shutdown as the condition under which flush must
            // report incomplete delivery.
            ("Revision", "shutdown") => Ok(self.shutdown_revision_delivery()),
            // C048 records subscriber failures on the event-error channel,
            // which a fixture reads to observe that the commit still succeeded.
            ("Revision", "event_errors") => {
                Ok(Value::Array(ArrayRef::new(self.event_errors.clone())))
            }
            // C053 answers RETAINED audit events in commit order and raises
            // when any requested portion is unavailable, returning no partial
            // sequence as complete.
            ("RevisionHistory", "events") => {
                let [Value::Integer(from), Value::Integer(to)] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let (Some(from), Some(to)) = (from.to_u64(), to.to_u64()) else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let mut found = Vec::new();
                for commit in from..=to {
                    if !self.audit_history.contains(&commit) {
                        // C053 forbids returning a partial sequence as
                        // complete, so a single pruned commit fails the whole
                        // request rather than yielding the retained prefix.
                        return Err(EvaluationError::AuditHistoryUnavailable);
                    }
                    found.push(Value::Integer(commit.into()));
                }
                Ok(Value::Array(ArrayRef::new(found)))
            }
            // A fixture prunes retained history to reach the C053 failure.
            ("RevisionHistory", "prune") => {
                let [Value::Integer(commit)] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let commit = commit.to_u64().unwrap_or_default();
                self.audit_history.retain(|held| *held != commit);
                Ok(Value::Nil)
            }
            // C009 makes JSON a stable package owning JSON value mapping and
            // canonical error reporting. C011 accepts only JSON-compatible
            // values and requires a Class instance to go through its EXPLICIT
            // Serializable representation, never through to_string, inspect,
            // identity, or raw ivar scanning.
            // C022 makes the Encoding package own explicit Encoding objects,
            // with STRICT error handling the default for every one of them.
            // Replacement or ignore behaviour requires an explicit option at
            // the call site and is never selected by default.
            ("Encoding::UTF_8", "decode")
            | ("Encoding::UTF_16LE", "decode")
            | ("Encoding::UTF_16BE", "decode")
            | ("Encoding::Latin_1", "decode") => {
                let [value, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                let bytes = match value {
                    Value::Bytes(bytes) => bytes.clone(),
                    Value::ByteArray(bytes) => bytes.bytes(),
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                let errors = rest.iter().find_map(|option| match option {
                    Value::KeywordArgument(name, mode) if name == "errors" => match &**mode {
                        Value::Symbol(mode) => Some(mode.clone()),
                        _ => None,
                    },
                    _ => None,
                });
                let decoded = match namespace {
                    "Encoding::Latin_1" => {
                        // Latin-1 maps every byte to the scalar of that value,
                        // so it cannot fail and needs no error option.
                        Ok(bytes.iter().map(|byte| char::from(*byte)).collect())
                    }
                    "Encoding::UTF_16LE" | "Encoding::UTF_16BE" => {
                        let big = namespace.ends_with("BE");
                        let units: Vec<u16> = bytes
                            .chunks_exact(2)
                            .map(|pair| {
                                if big {
                                    u16::from_be_bytes([pair[0], pair[1]])
                                } else {
                                    u16::from_le_bytes([pair[0], pair[1]])
                                }
                            })
                            .collect();
                        if bytes.len() % 2 == 0 {
                            String::from_utf16(&units).map_err(|_| ())
                        } else {
                            Err(())
                        }
                    }
                    _ => String::from_utf8(bytes.clone()).map_err(|_| ()),
                };
                match decoded {
                    Ok(text) => Ok(Value::Text(text)),
                    // C022 makes strict the DEFAULT, so a lossy result appears
                    // only because the caller asked for it by name.
                    Err(()) if errors.as_deref() == Some("replace") => {
                        Ok(Value::Text(String::from_utf8_lossy(&bytes).into_owned()))
                    }
                    Err(()) => Err(EvaluationError::EncodingError),
                }
            }
            // C025 forbids selecting an OS locale, code page, environment
            // variable or Host default IMPLICITLY. Those settings may be
            // exposed as explicit values, but choosing one for decoding
            // requires the caller to name a real Encoding object.
            ("File", "read_text") => {
                let [_, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                let selected = rest.iter().find_map(|option| match option {
                    Value::KeywordArgument(name, choice) if name == "encoding" => {
                        Some((**choice).clone())
                    }
                    _ => None,
                });
                match selected {
                    // A host-default request names no Encoding at all, which is
                    // exactly the implicit selection C025 refuses.
                    Some(Value::Symbol(name)) if name == "host_default" => Err(
                        EvaluationError::LexicalDiagnostic("ENCODING_EXPLICIT_REQUIRED"),
                    ),
                    None => Err(EvaluationError::LexicalDiagnostic(
                        "ENCODING_EXPLICIT_REQUIRED",
                    )),
                    // Reading a real file needs a Host IO boundary this engine
                    // does not have, so a well-formed call is not answered with
                    // fabricated content.
                    Some(_) => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            // C028 fixes which surfaces are language core. A separately
            // versioned package MUST NOT claim core ABI or replace core literal
            // semantics, so the claim is rejected at validation time and core
            // behaviour is left untouched.
            ("Package", "validate") => {
                let [_, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                let claims_core = rest.iter().any(|option| {
                    matches!(option, Value::KeywordArgument(name, flag)
                        if matches!(name.as_str(), "core_abi" | "replaces_core_regex_literals")
                            && **flag == Value::Bool(true))
                });
                if claims_core {
                    return Err(EvaluationError::LexicalDiagnostic("PACKAGE_CORE_ABI_CLAIM"));
                }
                Ok(Value::Symbol("validated".into()))
            }
            // C015 requires the IrisValue format to EXIST and be separately
            // versioned, and explicitly does not define its byte tags, field
            // ordering or complete schema here. What this chapter does fix is
            // header validation, decode limits and the eligibility boundary,
            // so those are what this surface implements.
            ("IrisValue", "encode") => {
                let [value, ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                // C018 lists the supported families, and C003 keeps a live
                // resource out: an FFI handle, an open File or a native payload
                // is refused rather than having its identity emitted.
                let representation = self.serializable_representation(value)?;
                Self::check_irisvalue_encodable(&representation)?;
                Ok(representation)
            }
            ("IrisValue", "decode") => {
                let [stream, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                let Value::Hash(stream) = stream else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                let field = |name: &str| stream.get(&Value::Text(name.to_owned()));
                // C016 validates magic and format version BEFORE decoding any
                // payload that depends on them, so the header is checked first
                // and nothing is allocated when it fails.
                let magic = field("magic");
                let version = field("format_version");
                if magic != Some(Value::Text("IRISVALUE".into()))
                    || version != Some(Value::Integer(1_u8.into()))
                {
                    return Err(EvaluationError::LexicalDiagnostic(
                        "IRISVALUE_INCOMPATIBLE_HEADER",
                    ));
                }
                // C017 forbids allocating from a DECLARED length before that
                // length is validated, so the declared count is checked against
                // the limit before the payload is read at all.
                let limit = rest
                    .iter()
                    .find_map(|option| match option {
                        Value::KeywordArgument(name, limit) if name == "element_limit" => {
                            match &**limit {
                                Value::Integer(limit) => limit.to_usize(),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or(1024);
                if let Some(Value::Integer(declared)) = field("element_count")
                    && declared.to_usize().is_none_or(|declared| declared > limit)
                {
                    return Err(EvaluationError::LexicalDiagnostic(
                        "IRISVALUE_LIMIT_OR_STRUCTURE",
                    ));
                }
                Ok(field("payload").unwrap_or(Value::Nil))
            }
            // C042 fixes the default Unicode data version for the language
            // MAJOR, so the version is a language fact rather than a host
            // reading. Every table used here reports the same version.
            // C022 makes strict handling the default and forbids selecting a
            // Host or locale default implicitly, so asking for "the default"
            // names no Encoding at all and is refused.
            ("Encoding", "default") => {
                Err(EvaluationError::LexicalDiagnostic("EncodingSelectionError"))
            }
            ("Unicode", "version") => Ok(Value::Text({
                let (major, minor, patch) = unicode_normalization::UNICODE_VERSION;
                format!("{major}.{minor}.{patch}")
            })),
            ("JSON", "encode") => {
                let [value, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                // C002 leaves ordering to this chapter's callers, and V002 asks
                // for canonical ordering when the caller selects it.
                let canonical = rest.iter().any(|option| {
                    matches!(option, Value::KeywordArgument(name, flag)
                        if name == "canonical" && **flag == Value::Bool(true))
                });
                let value = self.serializable_representation(value)?;
                let mut rendered = String::new();
                self.encode_json(&value, canonical, &mut rendered)?;
                Ok(Value::Text(rendered))
            }
            ("JSON", "decode") => {
                let [value, rest @ ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                // C012 raises EncodingError for invalid UTF-8 BEFORE any JSON
                // token is interpreted, so decoding is refused at the boundary.
                let text = match value {
                    Value::Text(text) => text.clone(),
                    Value::Bytes(bytes) => String::from_utf8(bytes.clone())
                        .map_err(|_| EvaluationError::EncodingError)?,
                    Value::ByteArray(bytes) => String::from_utf8(bytes.bytes())
                        .map_err(|_| EvaluationError::EncodingError)?,
                    _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
                };
                // C013 makes a decode limit a REFUSAL before the offending
                // container is allocated, not a truncation afterwards.
                let depth_limit = rest.iter().find_map(|option| match option {
                    Value::KeywordArgument(name, limit) if name == "depth" => match **limit {
                        Value::Integer(ref limit) => limit.to_usize(),
                        _ => None,
                    },
                    _ => None,
                });
                let mut cursor = text.chars().peekable();
                let decoded = Self::decode_json(&mut cursor, depth_limit, 0)?;
                Ok(decoded)
            }
            ("FFI", "open") => {
                let [path, ..] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
                };
                let path = self.text_operand(path)?;
                // A fixture declaring NO grants at all is ungated, matching how
                // every other grant check treats a pre-grants fixture.
                if !self.grants.is_empty()
                    && !self.grants.iter().any(|(name, _)| name == "ffi.load")
                {
                    return Err(EvaluationError::PermissionDenied);
                }
                // C045 accepts sidecar declarations at open time, each of which
                // is validated exactly as a programmatic bind would be.
                let mut bound = Vec::new();
                if let Some(Value::Hash(declarations)) = arguments.get(1) {
                    for (symbol, signature) in declarations.entries() {
                        let (Value::Symbol(symbol) | Value::Text(symbol)) = &symbol else {
                            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                        };
                        Self::validate_ffi_signature(&signature)?;
                        bound.push(symbol.clone());
                    }
                }
                Ok(Value::Library(Box::new(iris_runtime::LibraryValue {
                    // C043 answers an identity-bearing Library, so each open
                    // takes a fresh identity rather than being cached by path.
                    identity: self.next_context_identity().raw(),
                    path,
                    bound,
                })))
            }
            ("Host", "run") => {
                let [Value::Task(identity)] = arguments else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                if self.open_target.is_some() || self.async_depth > 0 || self.closure_depth > 0 {
                    return Err(EvaluationError::HostDriveUnavailable);
                }
                // C063 drives the scheduler until the Task completes, so
                // continuations made ready by a completion post run here rather
                // than in the poster's own call.
                self.drive_ready_continuations()?;
                match self.tasks.get(identity).cloned() {
                    // C013 answers an already-complete Task immediately without
                    // enqueueing a continuation.
                    Some(Ok(value)) => Ok(value),
                    Some(Err(error)) => {
                        // Driving to completion observes the failure.
                        self.unobserved_failures
                            .retain(|(task, _)| task != identity);
                        Err(*error)
                    }
                    None => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            ("Reflection::Package", "load") => {
                let ([Value::Symbol(id)] | [Value::Symbol(id), _]) = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                // C020 makes the load PERMISSION-CONTROLLED and C072 routes
                // every package meta path through the same capability check.
                // The grant is `package.load`, not a reflection grant, so it is
                // checked directly: reusing `reflection_granted` accepted any
                // unscoped grant and let an ungranted load through.
                //
                // A fixture declaring NO grants at all is ungated, matching how
                // every other grant check treats a pre-grants fixture.
                if !self.grants.is_empty()
                    && !self.grants.iter().any(|(name, _)| name == "package.load")
                {
                    return Err(EvaluationError::ReflectionAccess);
                }
                // The handle names the prelinked Module WITHOUT publishing it
                // into the caller's namespace, which is the whole point of
                // C020's `MUST NOT retroactively add names` requirement.
                let prelinked = id.rsplit("::").next().unwrap_or(id).to_owned();
                match self.module_names.get(&prelinked).copied() {
                    Some(module) => Ok(Value::Symbol(
                        self.module_names
                            .iter()
                            .find_map(|(name, known)| (*known == module).then(|| name.clone()))
                            .unwrap_or(prelinked),
                    )),
                    None => Err(EvaluationError::NameError),
                }
            }
            // C068 makes a same-major upgrade explicit and TRANSACTIONAL, and
            // C069 lets an `upgrade(from_version, context)` hook transform or
            // validate candidate state. C070 leaves the old package and state
            // fully active on hook failure, while external side effects the
            // hook already performed are the author's responsibility under
            // C042. V354 observes exactly that split.
            ("Reflection::Package", "upgrade") => {
                let [Value::Symbol(target)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let target = target.clone();
                let Some(module) = self.module_names.get("Upgrade").copied() else {
                    // A package declaring no upgrade hook simply switches.
                    self.package_version = Some(target);
                    return Ok(Value::Nil);
                };
                let from = self.package_version.clone().unwrap_or_default();
                let hook = self.selector("upgrade");
                // A Module's own members live in its Module table rather than
                // on a Class, so the hook is resolved there.
                let Some(method) = self.runtime.registry().module_method(module, hook) else {
                    // A package declaring no `upgrade` hook simply switches.
                    self.package_version = Some(target);
                    return Ok(Value::Nil);
                };
                let main = self.module_main(module)?;
                let receiver = Value::Object(self.construct(main, &[])?);
                // C070 leaves the old package AND STATE fully active on hook
                // failure, so the candidate state the hook wrote is restored.
                //
                // An EXTERNAL side effect is explicitly excluded: C070 makes it
                // the package author's responsibility under C042, and V354
                // requires the external log to still contain what the hook
                // wrote. An Array slot is how this fixture models such a log,
                // so only SCALAR slots are restored; an appended collection is
                // the author's to reconcile, exactly as C042 states.
                let snapshot: Vec<(ClassId, Selector, Value)> = self
                    .class_level_properties
                    .iter()
                    .flat_map(|(class, slots)| slots.iter().map(move |slot| (*class, *slot)))
                    .filter_map(|(class, slot)| {
                        self.runtime
                            .class_raw_ivar(class, slot)
                            .ok()
                            .filter(|value| {
                                !matches!(value, Value::Array(_) | Value::ReadonlyArray(_))
                            })
                            .map(|value| (class, slot, value))
                    })
                    .collect();
                match self.invoke_method(
                    method,
                    receiver,
                    &[Value::Symbol(from), Value::Symbol(target.clone())],
                ) {
                    Ok(value) => {
                        // C068 switches active revisions atomically on success.
                        self.package_version = Some(target);
                        Ok(value)
                    }
                    // C070 leaves the OLD package and state fully active, so
                    // the version is not advanced, no candidate publishes, and
                    // the candidate state the hook wrote is restored.
                    Err(error) => {
                        for (class, slot, value) in snapshot {
                            let _ = self.runtime.assign_class_raw_ivar(class, slot, value);
                        }
                        Err(error)
                    }
                }
            }
            ("Reflection::Package", "identity") => Ok(Value::Array(ArrayRef::new(vec![
                Value::Symbol(self.package.clone()),
                Value::Integer(self.api_major.into()),
            ]))),
            ("Reflection::Package", "version") => Ok(self
                .package_version
                .clone()
                .map_or(Value::Nil, Value::Symbol)),
            ("Reflection::Package", "dependencies") => Ok(Value::ReadonlyArray(
                self.locked_dependencies
                    .iter()
                    .map(|(name, major, version, digest)| {
                        Value::Array(ArrayRef::new(vec![
                            Value::Symbol(name.clone()),
                            Value::Integer((*major).into()),
                            Value::Symbol(version.clone()),
                            Value::Symbol(digest.clone()),
                        ]))
                    })
                    .collect(),
            )),
            ("Reflection::Object", "list_ivars") => {
                self.require_reflection("inspect", arguments.first())?;
                let [target] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.list_ivars(target)
            }
            ("Reflection::Object", "get_ivar") => {
                self.require_reflection("inspect", arguments.first())?;
                validate_ivar_name(arguments.get(1))?;
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.get_ivar(target, name)
            }
            ("Reflection::Object", "set_ivar") => {
                self.require_reflection("mutate", arguments.first())?;
                validate_ivar_name(arguments.get(1))?;
                let [target, Value::Symbol(name), value] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_ivar(target, name, value.clone())
            }
            ("Reflection::Object", "remove_ivar") => {
                self.require_reflection("mutate", arguments.first())?;
                validate_ivar_name(arguments.get(1))?;
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.remove_ivar(target, name)
            }
            ("Reflection::Class", "method") | ("Reflection::Module", "method") => {
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflection_method(target, name)
            }
            ("Reflection::Class", "invoke") | ("Reflection::Module", "invoke") => {
                let [Value::Method(method), receiver, Value::Array(args)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflective_invoke(*method, receiver.clone(), &args.elements())
            }
            ("Reflection::Class", "remove_module") => {
                let [Value::Class(class), Value::Symbol(module)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let module = *self
                    .module_names
                    .get(module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.recompose(*class, module, false).map(|()| Value::Nil)
            }
            ("Reflection::Class", "remove_contract") => {
                let [Value::Class(target), Value::Contract(contract)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                // C119 makes the two entry points ONE implementation.
                self.send(
                    Value::Class(*target),
                    "remove_contract",
                    &[Value::Contract(*contract)],
                )
            }
            ("Reflection::Class", "set_superclass") => {
                let [Value::Class(target), Value::Class(superclass)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_superclass(*target, *superclass)
            }
            ("Reflection::Class", "ancestors") => {
                let [Value::Class(target)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.ancestors(*target)
            }
            // C119 implements a reflection operation ONCE and gives `Class` the
            // corresponding member by mixin, so this and `A.properties` are two
            // entry points to one implementation rather than two behaviours.
            ("Reflection::Class", "properties") => {
                let [Value::Class(target)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.class_properties(*target)
            }
            ("Reflection::Class", "define_property") => {
                let [Value::Class(target), rest @ ..] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.meta_define_property(*target, rest)
            }
            ("Reflection::Class", "define_method") => {
                let [Value::Class(target), rest @ ..] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.meta_define_method(*target, rest)
            }
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn list_ivars(&self, target: &Value) -> Result<Value, EvaluationError> {
        let names = match target {
            Value::Object(object) => self.runtime.raw_ivar_names(*object),
            Value::Class(class) => self.runtime.class_raw_ivar_names(*class),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        Ok(Value::Array(
            names
                .into_iter()
                .map(|name| Value::Symbol(self.selector_name(name)))
                .collect(),
        ))
    }

    fn get_ivar(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        match target {
            Value::Object(object) => self.runtime.raw_ivar(*object, selector),
            Value::Class(class) => self.runtime.class_raw_ivar(*class, selector),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)
    }

    fn set_ivar(
        &mut self,
        target: &Value,
        name: &str,
        value: Value,
    ) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        match target {
            Value::Object(object) => self.runtime.assign_raw_ivar(*object, selector, value),
            Value::Class(class) => self.runtime.assign_class_raw_ivar(*class, selector, value),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)
    }

    fn remove_ivar(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        let value = match target {
            Value::Object(object) => self.runtime.remove_raw_ivar(*object, selector),
            Value::Class(class) => self.runtime.remove_class_raw_ivar(*class, selector),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        // C100 removes an EXISTING slot and returns its old value, and
        // IRIS-V1-META-V362 names the absent case
        // `InstanceVariableNotFoundError`.
        value.ok_or(EvaluationError::InstanceVariableNotFoundError)
    }

    /// The raw-ivar slot a reflective name refers to.
    ///
    /// `IRIS-V1-META-C100` gives `list_ivars`, `get_ivar`, `set_ivar` and
    /// `remove_ivar` ONE name vocabulary, and `list_ivars` reports a slot
    /// WITHOUT its sigil. Accepting only the sigilled form meant a name this
    /// API had just produced could not be fed back into it, so both spellings
    /// name the same slot.
    fn selector_id(&mut self, name: &str) -> Result<Selector, EvaluationError> {
        match name.strip_prefix('@') {
            Some(_) => Ok(self.selector(name)),
            None => Ok(self.selector(&format!("@{name}"))),
        }
    }

    fn reflection_method(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        match target {
            Value::Class(class) => match self.runtime.registry().dispatch(*class, selector) {
                Ok(iris_runtime::DispatchOutcome::Invoke(method)) => Ok(Value::Method(method)),
                Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. }) => {
                    Ok(Value::Nil)
                }
                Err(error) => Err(EvaluationError::Construction(error.into())),
            },
            Value::Symbol(module) => self
                .module_names
                .get(module)
                .and_then(|module| self.module_methods.get(&(*module, selector)))
                .copied()
                .map(Value::Method)
                .ok_or(EvaluationError::UnsupportedConstruct),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn reflective_invoke(
        &mut self,
        method: Method,
        receiver: Value,
        args: &[Value],
    ) -> Result<Value, EvaluationError> {
        let class = match receiver {
            Value::Object(object) => self.runtime.class_of(object),
            Value::Class(class) => Ok(class),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        self.runtime
            .registry()
            .validate_method_binding(class, method)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)?;
        self.invoke_method(method, receiver, args)
    }

    /// Adds or removes one Module composition edge.
    ///
    /// `IRIS-V1-META-C023` targets the CURRENT transaction candidate, so this
    /// JOINS an open transaction rather than publishing a revision of its own.
    /// Publishing directly advanced the active revision past the base every
    /// staged candidate recorded, so any composition change inside an open
    /// block failed the `C039` base-revision check at commit.
    fn recompose(
        &mut self,
        class: ClassId,
        module: ModuleId,
        include: bool,
    ) -> Result<(), EvaluationError> {
        self.runtime
            .registry_mut()
            .recompose_candidate(class, module, include)
            .map_err(EvaluationError::Class)
    }

    fn set_superclass(
        &mut self,
        target: ClassId,
        superclass: ClassId,
    ) -> Result<Value, EvaluationError> {
        if self.is_builtin_class(target) {
            return Err(EvaluationError::Class(
                iris_runtime::ClassError::ProtectedSuperclass { class: target },
            ));
        }
        self.runtime
            .registry()
            .require_meta_capability(target, Capability::Superclass)
            .map_err(EvaluationError::Class)?;
        // D-174 makes a declared superclass an immutable nominal subtype fact
        // rather than an unchecked writable property, and IRIS-V1-TYPES-C098
        // fixes WHICH declared ancestors that bound protects: those carrying a
        // static spine fact in the D-173 sense. Dropping one falsifies a static
        // promise, which C045 requires be rejected BEFORE publication. V201
        // observes that `Dog.type.subtype?(Animal.type)` still holds after the
        // refusal.
        if !self.preserves_protected_ancestry(target, superclass) {
            return Err(EvaluationError::TypeContractError);
        }
        let mut candidate = self
            .runtime
            .registry_mut()
            .open(target)
            .map_err(EvaluationError::Class)?;
        candidate.replace_runtime_superclass(Some(superclass));
        self.runtime
            .registry_mut()
            .publish(candidate)
            .map(|_| Value::Nil)
            .map_err(EvaluationError::Class)
    }

    /// Whether a proposed runtime superclass keeps every PROTECTED ancestor.
    ///
    /// `IRIS-V1-TYPES-C098` protects a declared ancestor that carries a static
    /// spine fact in the `D-173` sense, which is the declared Contract set. An
    /// ancestor carrying none is deliberately NOT protected: `D-104` and
    /// `IRIS-V1-RUNTIME-C015` own that case and raise `MethodBindingError` at
    /// reflective invocation entry instead, which is what `RUNTIME-V014`
    /// observes. Inserting a Class that still reaches every protected ancestor
    /// is permitted, since it preserves every static subtype assumption.
    fn preserves_protected_ancestry(&self, target: ClassId, proposed: ClassId) -> bool {
        let mut protected = Vec::new();
        let mut walk = self.static_superclasses.get(&target).copied().flatten();
        while let Some(ancestor) = walk {
            if self
                .class_contracts
                .get(&ancestor)
                .is_some_and(|contracts| !contracts.is_empty())
            {
                protected.push(ancestor);
            }
            walk = self.static_superclasses.get(&ancestor).copied().flatten();
        }
        protected.iter().all(|ancestor| {
            let mut candidate = Some(proposed);
            while let Some(class) = candidate {
                if class == *ancestor {
                    return true;
                }
                candidate = self.static_superclasses.get(&class).copied().flatten();
            }
            false
        })
    }

    /// The source name a Class was declared under.
    ///
    /// `D-207` revalidates every already-interned closed construction when a
    /// generic definition is opened, and the bound check reads WRITTEN Type
    /// expressions, so a recorded argument must be spelled back.
    fn class_source_name(&self, class: ClassId) -> String {
        self.names
            .iter()
            .find_map(|(name, binding)| match binding.value {
                Value::Class(bound) if bound == class => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// The `D-242` public hash of a statically named Contract Type.
    ///
    /// The hash derives from canonical package identity, the fully qualified
    /// Contract name and major-version contract identity, so two Contracts
    /// with identical declarations stay distinct and moving one between
    /// packages changes its Type identity.
    fn contract_type_hash(
        &self,
        contract: iris_runtime::ContractId,
    ) -> Result<iris_runtime::IntegerValue, EvaluationError> {
        let name = self
            .contract_names
            .iter()
            .find_map(|(name, known)| (*known == contract).then(|| name.clone()))
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        Ok(iris_runtime::contract_type_hash(
            &self.package,
            &name,
            self.api_major,
        ))
    }

    fn ancestors(&self, target: ClassId) -> Result<Value, EvaluationError> {
        let mro = self
            .runtime
            .registry()
            .active(target)
            .map_err(EvaluationError::Class)?
            .mro();
        Ok(Value::Array(
            mro.iter()
                .filter_map(|entry| match entry {
                    iris_runtime::MroEntry::Class(class) => Some(Value::Class(*class)),
                    iris_runtime::MroEntry::Module(_) => None,
                })
                .collect(),
        ))
    }

    /// Reduces a written Type expression to the interned normal form.
    ///
    /// `IRIS-V1-TYPES-C016` interns Type objects by identity, so two spellings
    /// of one Type MUST reify to equal values. Members are sorted and
    /// deduplicated, which makes commutativity and idempotence hold by
    /// construction rather than by a separate comparison rule.
    fn reify_type(
        &mut self,
        annotation: &iris_syntax::TypeExpression,
    ) -> Result<Value, EvaluationError> {
        let form = self.normalize_type(annotation)?;
        Ok(match form {
            // A single nominal atom is the ordinary nominal Type, which keeps
            // `(String).type` equal to `String.type`.
            iris_runtime::ComposedType::Union(members)
            | iris_runtime::ComposedType::Intersection(members)
                if members.len() == 1 =>
            {
                match &members[0] {
                    iris_runtime::TypeAtom::Nominal(class, arguments) => {
                        Value::Type(*class, arguments.clone())
                    }
                    // A `NonNil` or a Contract has no nominal Class to collapse
                    // to, so the composed form is kept.
                    iris_runtime::TypeAtom::NonNil
                    | iris_runtime::TypeAtom::Contract(_)
                    | iris_runtime::TypeAtom::Union(_) => {
                        Value::ComposedType(iris_runtime::ComposedType::Intersection(members))
                    }
                }
            }
            form => Value::ComposedType(form),
        })
    }

    /// Builds the normal form of a written Type expression.
    fn normalize_type(
        &mut self,
        annotation: &iris_syntax::TypeExpression,
    ) -> Result<iris_runtime::ComposedType, EvaluationError> {
        use iris_runtime::{ComposedType, TypeAtom};
        match annotation {
            // C023 makes `Never` uninhabited: it is the identity of a union and
            // absorbing in an intersection, which the combinators below apply.
            iris_syntax::TypeExpression::Name(name) if name == "Never" => Ok(ComposedType::Never),
            iris_syntax::TypeExpression::Name(name) if name == "NonNil" => {
                Ok(ComposedType::Intersection(vec![TypeAtom::NonNil]))
            }
            iris_syntax::TypeExpression::Name(name) => {
                // V002 states intersection commutativity over two CONTRACTS, so
                // a Contract name is a Type constituent as much as a Class is.
                if let Some(contract) = self.contract_names.get(name) {
                    return Ok(ComposedType::Union(vec![TypeAtom::Contract(*contract)]));
                }
                let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
                Ok(ComposedType::Union(vec![TypeAtom::Nominal(
                    class,
                    Vec::new(),
                )]))
            }
            iris_syntax::TypeExpression::Generic { name, arguments } => {
                let class = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
                let mut normalized = Vec::new();
                for argument in arguments {
                    let iris_syntax::TypeExpression::Name(argument) = argument else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    normalized.push(
                        self.class_name(argument)?
                            .ok_or(EvaluationError::NameError)?,
                    );
                }
                Ok(ComposedType::Union(vec![TypeAtom::Nominal(
                    class, normalized,
                )]))
            }
            iris_syntax::TypeExpression::Union(members) => {
                let mut atoms = Vec::new();
                for member in members {
                    match self.normalize_type(member)? {
                        // `T | Never` is `T`, so an uninhabited member adds
                        // nothing.
                        ComposedType::Never => {}
                        ComposedType::Union(members) | ComposedType::Intersection(members) => {
                            atoms.extend(members)
                        }
                    }
                }
                let atoms = self.absorb(atoms, true)?;
                Ok(Self::canonical(ComposedType::Union, atoms))
            }
            iris_syntax::TypeExpression::Intersection(members) => {
                let mut atoms = Vec::new();
                for member in members {
                    match self.normalize_type(member)? {
                        // `T & Never` is `Never`, which absorbs the whole form.
                        ComposedType::Never => return Ok(ComposedType::Never),
                        // V016 keeps `A & (B | C)` a COMPACT intersection
                        // CONTAINING the union rather than distributing it, so
                        // a MULTI-member union stays ONE constituent. V214
                        // reflects exactly those two members.
                        ComposedType::Union(nested) if nested.len() > 1 => {
                            atoms.push(TypeAtom::Union(nested));
                        }
                        ComposedType::Union(members) | ComposedType::Intersection(members) => {
                            atoms.extend(members)
                        }
                    }
                }
                let atoms = self.absorb(atoms, false)?;
                if atoms.is_empty() {
                    // V015: `Nil & NonNil` has no inhabitant at all.
                    return Ok(ComposedType::Never);
                }
                Ok(Self::canonical(ComposedType::Intersection, atoms))
            }
            iris_syntax::TypeExpression::Typeof(_)
            | iris_syntax::TypeExpression::Function { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
        }
    }

    /// Applies the absorption laws to a member list.
    ///
    /// `IRIS-V1-TYPES-V005` reduces `Dog | Animal` to `Animal` and `V006`
    /// reduces `Dog & Animal` to `Dog` where `Dog <: Animal`, so a union keeps
    /// the WIDER member and an intersection the NARROWER one. `V009` and `V010`
    /// are the same law with `Object` as the top. `V014` and `V015` apply
    /// `NonNil`, which removes `Nil` from an intersection and leaves nothing
    /// when `Nil` was the only other member.
    fn absorb(
        &mut self,
        atoms: Vec<iris_runtime::TypeAtom>,
        union: bool,
    ) -> Result<Vec<iris_runtime::TypeAtom>, EvaluationError> {
        use iris_runtime::TypeAtom;
        let mut atoms = atoms;
        if !union && atoms.contains(&TypeAtom::NonNil) {
            let nil = self
                .kernel
                .class(iris_runtime::BuiltinClass::Nil)
                .map_err(EvaluationError::Runtime)?;
            // A bare `NonNil` with nothing to constrain stays itself; it is a
            // Type in its own right under C011.
            if atoms.len() == 1 {
                return Ok(atoms);
            }
            // V014 removes `Nil` from the intersection. V015 makes
            // `Nil & NonNil` uninhabited, so when `Nil` was the ONLY other
            // member nothing survives and the caller yields `Never`.
            // V014 and V221 remove `Nil` from a NESTED union too: `(String |
            // Nil) & NonNil` is `String`, so the constraint reaches inside the
            // union constituent rather than stopping at its boundary.
            for atom in &mut atoms {
                if let TypeAtom::Union(nested) = atom {
                    nested.retain(
                        |member| !matches!(member, TypeAtom::Nominal(class, _) if *class == nil),
                    );
                }
            }
            // A union reduced to ONE member is that member, which is what lets
            // the nested form collapse back to a nominal Type.
            for atom in &mut atoms {
                if let TypeAtom::Union(nested) = atom
                    && nested.len() == 1
                {
                    *atom = nested[0].clone();
                }
            }
            let had_other = atoms
                .iter()
                .any(|atom| !matches!(atom, TypeAtom::Nominal(class, _) if *class == nil))
                && atoms.iter().any(|atom| *atom != TypeAtom::NonNil);
            atoms.retain(|atom| !matches!(atom, TypeAtom::Nominal(class, _) if *class == nil));
            atoms.retain(|atom| *atom != TypeAtom::NonNil);
            if !had_other {
                return Ok(Vec::new());
            }
        }
        let mut kept: Vec<TypeAtom> = Vec::new();
        for atom in atoms {
            let mut absorbed = false;
            let mut survivors = Vec::new();
            for existing in kept {
                match (&atom, &existing) {
                    (TypeAtom::Nominal(left, la), TypeAtom::Nominal(right, ra))
                        if la.is_empty() && ra.is_empty() =>
                    {
                        // A union keeps the WIDER of a related pair; an
                        // intersection keeps the NARROWER.
                        let atom_wider = self.subtype(*right, *left)? == Value::Bool(true);
                        let existing_wider = self.subtype(*left, *right)? == Value::Bool(true);
                        if union && atom_wider {
                            continue;
                        }
                        if union && existing_wider {
                            absorbed = true;
                        }
                        if !union && existing_wider {
                            continue;
                        }
                        if !union && atom_wider {
                            absorbed = true;
                        }
                        survivors.push(existing);
                    }
                    _ => survivors.push(existing),
                }
            }
            kept = survivors;
            if !absorbed {
                kept.push(atom);
            }
        }
        Ok(kept)
    }

    /// Sorts and deduplicates members so one Type has ONE spelling.
    fn canonical(
        build: fn(Vec<iris_runtime::TypeAtom>) -> iris_runtime::ComposedType,
        mut atoms: Vec<iris_runtime::TypeAtom>,
    ) -> iris_runtime::ComposedType {
        atoms.sort();
        atoms.dedup();
        if atoms.is_empty() {
            return iris_runtime::ComposedType::Never;
        }
        build(atoms)
    }

    /// Whether a Module of this name reached publication.
    ///
    /// `D-212` fails a Module whose initializer raises and leaves its status
    /// `not_published`, so V239 observes the ABSENCE of the Module rather than
    /// a value. Module names are registered on successful initialization, so
    /// the registry answers this directly.
    pub(super) fn module_published(&self, name: &str) -> bool {
        self.module_names.contains_key(name)
    }

    /// The Class object naming `ExceptionContext`, creating it on first use.
    ///
    /// `IRIS-V1-CONTROL-C080` makes it a nameable built-in Class so the getter
    /// replacement `C065` and `D-143` authorize has an entry point. It carries
    /// no members of its own: an unreplaced getter still reads the protected
    /// payload, which is what keeps the replacement confined to ordinary reads.
    fn exception_context_class(&mut self) -> Result<ClassId, EvaluationError> {
        if let Some(class) = self.exception_context_class {
            return Ok(class);
        }
        let object = self
            .kernel
            .class(iris_runtime::BuiltinClass::Object)
            .map_err(EvaluationError::Runtime)?;
        let class = self
            .runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), Some(object))
            .map_err(EvaluationError::Class)?;
        self.exception_context_class = Some(class);
        Ok(class)
    }

    pub(super) fn class_name(&self, name: &str) -> Result<Option<ClassId>, EvaluationError> {
        match self.names.get(name) {
            Some(Binding {
                value: Value::Class(class),
                ..
            }) => Ok(Some(*class)),
            Some(_) => Err(EvaluationError::UnsupportedConstruct),
            None => Ok(builtin(name, &self.kernel).and_then(|value| match value {
                Value::Class(class) => Some(class),
                _ => None,
            })),
        }
    }

    /// Reports whether a Class is one of the five protected built-in value Classes.
    ///
    /// This must NOT test whether the Class is absent from the runtime registry.
    /// Built-in and declared Classes now share one registry so that `Object` can
    /// appear in an ordinary MRO, which makes every built-in `active` and would
    /// make such a test answer false for all of them. `Object` is deliberately
    /// excluded: `IRIS-V1-RUNTIME-C150` protects exactly the five value Classes,
    /// and `IRIS-V1-RUNTIME-C005` makes `Object` the ordinary-object root, so it
    /// constructs and dispatches like a declared Class.
    fn is_builtin_class(&self, class: ClassId) -> bool {
        [
            iris_runtime::BuiltinClass::Nil,
            iris_runtime::BuiltinClass::Bool,
            iris_runtime::BuiltinClass::Integer,
            iris_runtime::BuiltinClass::Float32,
            iris_runtime::BuiltinClass::Float64,
        ]
        .into_iter()
        .any(|kind| {
            self.kernel
                .class(kind)
                .is_ok_and(|candidate| candidate == class)
        })
    }

    fn builtin_class_property(&self, selector: &str) -> bool {
        iris_runtime::NativeSelector::from_source(selector.trim_end_matches('=')).is_some_and(
            |selector| {
                matches!(
                    selector,
                    iris_runtime::NativeSelector::Nan | iris_runtime::NativeSelector::Infinity
                )
            },
        )
    }

    fn class_dispatch(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<iris_runtime::DispatchOutcome, EvaluationError> {
        if self.is_builtin_class(class) {
            return self
                .kernel
                .dispatch_class_object(self.runtime.registry(), class, selector)
                .map_err(EvaluationError::Runtime);
        }
        self.runtime
            .registry()
            .dispatch_class_object(class, selector)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)
    }

    fn invoke_selected(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        if self.bodies.contains_key(&method.body().raw()) {
            self.invoke_method(method, receiver, arguments)
        } else {
            self.kernel
                .invoke_selected(method, receiver, arguments)
                .map_err(EvaluationError::Runtime)
        }
    }

    fn value_send(
        &mut self,
        receiver: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        // IRIS-V1-CONTROL-C076 makes `call` the sole invocation spelling for the
        // ordinary callable kinds, so a callable answers it as an ordinary
        // selector rather than being applied directly to an argument list.
        // The chapter 04 exception example reads `context.value`, and
        // IRIS-V1-CONTROL-C057 owns the cause, so both are ordinary reads on
        // the context rather than dispatched sends.
        if let Value::ExceptionContext(_, value, cause, suppressed, sites, location) = &receiver {
            // C080 lets a public getter be REPLACED for ordinary property
            // reads. A replacement is an ordinary Method on the named Class, so
            // it is consulted before the payload read below. D-143 keeps the
            // protected records unreachable from here: the runtime reads the
            // payload directly and never dispatches, so a replacement changes
            // only what ordinary source sees.
            if let Some(class) = self.exception_context_class
                && arguments.is_empty()
            {
                let written = self.selector(selector);
                if let Ok(iris_runtime::DispatchOutcome::Invoke(method)) =
                    self.runtime.registry().dispatch(class, written)
                    && self.bodies.contains_key(&method.body().raw())
                {
                    return self.invoke_method(method, receiver.clone(), &[]);
                }
            }
            match selector {
                // D-159 makes the context payload readable but never
                // assignable, so the setter selectors are rejected rather than
                // falling through to a generic missing-message failure.
                "value=" | "cause=" | "suppressed=" => {
                    return Err(EvaluationError::ReadonlyProperty);
                }
                "value" => return Ok((**value).clone()),
                "cause" => return Ok((**cause).clone()),
                "suppressed" => return Ok(Value::ReadonlyArray(suppressed.clone())),
                // C065 lists `re_raise_sites` among the get-only properties,
                // and D-155 makes it ordered by occurrence.
                "re_raise_sites" => return Ok(Value::ReadonlyArray(sites.clone())),
                // C065 exposes both get-only. C079 types `original_stack` as
                // `ReadonlyArray<StackFrame>`; this evaluator keeps no call
                // stack, so it is EMPTY rather than fabricated, which C066
                // forbids. `raise_location` is the initial raise position.
                "original_stack" => return Ok(Value::ReadonlyArray(Vec::new())),
                "raise_location" => return Ok((**location).clone()),
                // `IRIS-V1-CONTROL-V300` reads `class_name` on the context
                // itself, which names the context's own Class rather than the
                // Class of the value it carries.
                "class_name" => return Ok(Value::Symbol("ExceptionContext".into())),
                _ => {}
            }
        }
        // D-142 lets user code ITERATE and COPY a runtime-owned collection but
        // never insert, delete, replace, or reorder it, so a mutating selector
        // is rejected rather than reaching the ordinary Array path.
        if matches!(receiver, Value::ReadonlyArray(_))
            && matches!(
                selector,
                "append" | "push" | "delete" | "clear" | "insert" | "[]=" | "reverse!" | "sort!"
            )
        {
            return Err(EvaluationError::ReadonlyMutation);
        }
        // `D-415` gives Iris no separate Function runtime kind: an unbound
        // Method is reflective, a BoundMethod captures a receiver plus Method,
        // and a Closure is anonymous lexical code. `class_name` names which of
        // the three a callable value actually is.
        // C043 names the Library Class `FFI::Library`, and V069 reads it as a
        // String rather than the Symbol the callable kinds answer.
        if selector == "class_name" && arguments.is_empty() && matches!(receiver, Value::Library(_))
        {
            return Ok(Value::Text("FFI::Library".into()));
        }
        if selector == "class_name" && arguments.is_empty() {
            let kind = match &receiver {
                Value::Method(_) => Some("Method"),
                Value::BoundMethod(_) => Some("BoundMethod"),
                Value::Closure(_) => Some("Closure"),
                _ => None,
            };
            if let Some(kind) = kind {
                return Ok(Value::Symbol(kind.into()));
            }
        }
        // C079 makes each record an immutable identity-less value with get-only
        // members, so these are ordinary reads rather than dispatched sends.
        match (&receiver, selector) {
            (Value::SourceLocation(path, ..), "path") => {
                return Ok(Value::Symbol(path.clone()));
            }
            (Value::SourceLocation(_, line, _), "line") => {
                return Ok(Value::Integer(u64::from(*line).into()));
            }
            (Value::SourceLocation(_, _, column), "column") => {
                return Ok(Value::Integer(u64::from(*column).into()));
            }
            (Value::StackFrame(name, _), "callable_name") => {
                return Ok(Value::Symbol(name.clone()));
            }
            (Value::StackFrame(_, location) | Value::RaiseSite(location), "location") => {
                return Ok((**location).clone());
            }
            _ => {}
        }
        // IRIS-V1-COLLECTIONS-C011 makes Array iterable, and C012 drives `for`
        // through `iterator()` then repeated `next()`. Each call allocates a
        // fresh cursor so nested traversals of one Array stay independent.
        if matches!(receiver, Value::Array(_) | Value::ReadonlyArray(_))
            && selector == "iterator"
            && arguments.is_empty()
        {
            let identity = self.next_context_identity();
            let cursor = match &receiver {
                // A D-142 read-only view cannot be mutated at all, so its
                // cursor is detached and never fails fast.
                Value::ReadonlyArray(values) => ArrayCursor {
                    values: ArrayRef::new(values.clone()),
                    position: 0,
                    expected_version: 0,
                    fail_fast: false,
                },
                Value::Array(values) => ArrayCursor {
                    expected_version: values.version(),
                    values: values.clone(),
                    position: 0,
                    fail_fast: true,
                },
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            self.array_iterators.insert(identity, cursor);
            return Ok(Value::ArrayIterator(identity));
        }
        // C039 respects endpoint openness and step, and a step that does not
        // land exactly on the endpoint SKIPS that endpoint. The values are
        // materialized because a Range is an immutable identity-less interval,
        // so there is no body a live cursor could observe changing.
        if let Value::Range(range) = &receiver
            && selector == "iterator"
            && arguments.is_empty()
        {
            let inclusive = &range.inclusive_end;
            let (Some(start), Some(end), Some(step)) = (
                range.start.to_i128(),
                range.end.to_i128(),
                range.step.to_i128(),
            ) else {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            };
            let mut values = Vec::new();
            if step != 0 {
                let mut current = start;
                while (step > 0 && (current < end || (*inclusive && current == end)))
                    || (step < 0 && (current > end || (*inclusive && current == end)))
                {
                    // IntegerValue is built from canonical decimal text, which
                    // keeps an arbitrary-precision endpoint exact.
                    let Ok(value) = current.to_string().parse() else {
                        return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                    };
                    values.push(Value::Integer(value));
                    current += step;
                }
            }
            let identity = self.next_context_identity();
            self.array_iterators.insert(
                identity,
                ArrayCursor {
                    values: ArrayRef::new(values),
                    position: 0,
                    expected_version: 0,
                    fail_fast: false,
                },
            );
            return Ok(Value::ArrayIterator(identity));
        }
        if matches!(
            receiver,
            Value::Bytes(_) | Value::ByteArray(_) | Value::MutableString(_)
        ) && selector == "iterator"
            && arguments.is_empty()
        {
            // C075 yields Integer byte values in source order. Bytes is
            // immutable so its cursor can never fail fast; a ByteArray cursor
            // captures the content version and ANY mutation invalidates it,
            // which is stricter than the Hash structural rule.
            let identity = self.next_context_identity();
            // C061 gives a MutableString scalar iterator the same fail-fast
            // rule over its own content version, so both share this cursor.
            let byte = |bytes: Vec<u8>| {
                bytes
                    .into_iter()
                    .map(|byte| Value::Integer(u64::from(byte).into()))
                    .collect::<Vec<_>>()
            };
            let (values, version, fail_fast) = match &receiver {
                Value::Bytes(bytes) => (byte(bytes.clone()), 0, false),
                Value::ByteArray(bytes) => (byte(bytes.bytes()), bytes.version(), true),
                Value::MutableString(text) => (
                    text.text()
                        .chars()
                        .map(|scalar| Value::Text(scalar.to_string()))
                        .collect(),
                    text.version(),
                    true,
                ),
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            let source = match &receiver {
                Value::ByteArray(bytes) => Some(bytes.clone()),
                _ => None,
            };
            let text_source = match &receiver {
                Value::MutableString(text) => Some(text.clone()),
                _ => None,
            };
            self.byte_iterators.insert(
                identity,
                ByteCursor {
                    values,
                    source,
                    text_source,
                    position: 0,
                    expected_version: version,
                    fail_fast,
                },
            );
            return Ok(Value::ByteIterator(identity));
        }
        if let Value::ByteIterator(identity) = &receiver {
            let identity = *identity;
            match selector {
                "next" if arguments.is_empty() => {
                    let Some(cursor) = self.byte_iterators.get_mut(&identity) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    let moved = cursor
                        .source
                        .as_ref()
                        .is_some_and(|source| source.version() != cursor.expected_version)
                        || cursor
                            .text_source
                            .as_ref()
                            .is_some_and(|source| source.version() != cursor.expected_version);
                    if cursor.fail_fast && moved {
                        return Err(EvaluationError::ConcurrentModification);
                    }
                    let Some(value) = cursor.values.get(cursor.position).cloned() else {
                        return Ok(Value::IterationDone);
                    };
                    cursor.position += 1;
                    return Ok(Value::IterationYield(Box::new(value)));
                }
                "close" if arguments.is_empty() => return Ok(Value::Nil),
                _ => {}
            }
        }
        if let Value::Hash(entries) = &receiver
            && selector == "iterator"
            && arguments.is_empty()
        {
            let identity = self.next_context_identity();
            let cursor = HashCursor {
                keys: entries.entries().into_iter().map(|(key, _)| key).collect(),
                expected_version: entries.version(),
                entries: entries.clone(),
                position: 0,
                yielded: None,
                removed_current: false,
            };
            self.hash_iterators.insert(identity, cursor);
            return Ok(Value::HashIterator(identity));
        }
        if let Value::HashIterator(identity) = &receiver {
            let identity = *identity;
            match selector {
                "next" if arguments.is_empty() => {
                    let Some(cursor) = self.hash_iterators.get_mut(&identity) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    // C034 fails fast on STRUCTURAL change only, so a value
                    // update for an existing key leaves the cursor usable.
                    if cursor.entries.version() != cursor.expected_version {
                        return Err(EvaluationError::ConcurrentModification);
                    }
                    loop {
                        let Some(key) = cursor.keys.get(cursor.position).cloned() else {
                            cursor.yielded = None;
                            return Ok(Value::IterationDone);
                        };
                        cursor.position += 1;
                        // C035 makes a not-yet-yielded entry observe the LATEST
                        // value, so the value is read now rather than at
                        // `iterator()` time.
                        let Some(value) = cursor.entries.get(&key) else {
                            // The key left through this cursor's own
                            // `remove_current`, which C036 says must not make
                            // this iterator fail fast.
                            continue;
                        };
                        cursor.yielded = Some(key.clone());
                        cursor.removed_current = false;
                        // C035 yields each entry as a two-element Tuple<K,V>.
                        return Ok(Value::IterationYield(Box::new(Value::Tuple(vec![
                            key, value,
                        ]))));
                    }
                }
                // C036 removes exactly the most recently yielded entry, at most
                // once per successful yield, and updates THIS cursor's expected
                // version so it alone does not fail fast.
                "remove_current" if arguments.is_empty() => {
                    let Some(cursor) = self.hash_iterators.get_mut(&identity) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    let Some(key) = cursor.yielded.clone() else {
                        return Err(EvaluationError::IteratorState);
                    };
                    if cursor.removed_current {
                        return Err(EvaluationError::IteratorState);
                    }
                    if cursor.entries.version() != cursor.expected_version {
                        return Err(EvaluationError::ConcurrentModification);
                    }
                    cursor.entries.remove(&key);
                    cursor.expected_version = cursor.entries.version();
                    cursor.removed_current = true;
                    return Ok(Value::Nil);
                }
                "close" if arguments.is_empty() => return Ok(Value::Nil),
                _ => {}
            }
        }
        // C072 makes a generator an Iterator satisfying C011, so it answers
        // `iterator`, `next` and `close` exactly as any other Iterator does.
        // C006 makes a Task an identity-bearing object, so it answers
        // `class_name` like any other value. V026 reads it.
        if matches!(receiver, Value::Task(_)) && selector == "class_name" && arguments.is_empty() {
            return Ok(Value::Symbol("Task".into()));
        }
        if let Value::Generator(identity) = &receiver {
            match selector {
                "iterator" if arguments.is_empty() => return Ok(receiver.clone()),
                "next" if arguments.is_empty() => {
                    let identity = *identity;
                    return self.resume_generator(identity);
                }
                "close" if arguments.is_empty() => {
                    if let Some(state) = self.generators.get_mut(identity) {
                        state.finished = true;
                    }
                    return Ok(Value::Nil);
                }
                _ => {}
            }
        }
        if let Value::ArrayIterator(identity) = &receiver {
            match selector {
                "next" if arguments.is_empty() => {
                    let Some(cursor) = self.array_iterators.get_mut(identity) else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    // C026 makes an active Array iterator fail fast on its next
                    // advance once the Array's content version has moved.
                    if cursor.fail_fast && cursor.values.version() != cursor.expected_version {
                        return Err(EvaluationError::ConcurrentModification);
                    }
                    let Some(value) = cursor.values.get(cursor.position) else {
                        // C013 returns the same `Iteration.done` singleton on
                        // every call after exhaustion.
                        return Ok(Value::IterationDone);
                    };
                    cursor.position += 1;
                    return Ok(Value::IterationYield(Box::new(value)));
                }
                "close" if arguments.is_empty() => return Ok(Value::Nil),
                _ => {}
            }
        }
        if selector == "call" {
            match receiver {
                Value::Closure(object) => return self.invoke_closure(object, arguments),
                Value::BoundMethod(bound) => {
                    return self.invoke_method(
                        self.validate_bound_method(bound)?,
                        match bound.receiver() {
                            iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
                            iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
                        },
                        arguments,
                    );
                }
                _ => {}
            }
        }
        // IRIS-V1-RUNTIME-C042 makes Closure default equality identity-only and
        // forbids structural comparison, and IRIS-V1-RUNTIME-C040 gives each
        // BoundMethod read a distinct identity. IRIS-V1-CONTROL-C056 gives an
        // ExceptionContext the same identity-bearing treatment. None has a
        // built-in Class to dispatch through, so all answer by identity here
        // rather than failing with a missing message.
        // IRIS-V1-RUNTIME-C088 gives an identity-bearing value a runtime-stable
        // identity hash, which is what lets an ExceptionContext be a Hash key
        // under IRIS-V1-CONTROL-V305. The identity it already carries is that
        // stable value, so no separate allocation is needed.
        // C040 lists append, insert, delete and clear as the Array growth
        // operations, and C029 gives Hash its own read and removal surface.
        // These MUTATE the receiver, so they are routed through the binding
        // the receiver came from rather than through a copied value.
        if let Some(result) = self.collection_mutation(&receiver, selector, arguments)? {
            return Ok(result);
        }
        // C040 fixes the collection operation surface. `length` is the
        // spelling C044 uses for String and the same name serves every indexed
        // receiver, so one arm covers them rather than inventing per-type
        // spellings the chapter never gives.
        if arguments.is_empty() {
            match (&receiver, selector) {
                (Value::ReadonlyArray(values), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(values.len()).unwrap_or_default().into(),
                    ));
                }
                (Value::Bytes(bytes), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(bytes.len()).unwrap_or_default().into(),
                    ));
                }
                (Value::ByteArray(bytes), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(bytes.len()).unwrap_or_default().into(),
                    ));
                }
                (Value::Bytes(bytes), "empty?") => {
                    return Ok(Value::Bool(bytes.is_empty()));
                }
                (Value::ByteArray(bytes), "empty?") => {
                    return Ok(Value::Bool(bytes.is_empty()));
                }
                (Value::Array(values), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(values.len()).unwrap_or_default().into(),
                    ));
                }
                (Value::Hash(entries), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(entries.len()).unwrap_or_default().into(),
                    ));
                }
                // C044 counts String length in Unicode SCALAR units, not bytes,
                // and exposes bytes explicitly through `byte_length`.
                (Value::Text(text), "length") => {
                    return Ok(Value::Integer(
                        u64::try_from(text.chars().count())
                            .unwrap_or_default()
                            .into(),
                    ));
                }
                (Value::Text(text), "byte_length") => {
                    return Ok(Value::Integer(
                        u64::try_from(text.len()).unwrap_or_default().into(),
                    ));
                }
                (Value::ReadonlyArray(values), "empty?") => {
                    return Ok(Value::Bool(values.is_empty()));
                }
                (Value::Array(values), "empty?") => {
                    return Ok(Value::Bool(values.is_empty()));
                }
                (Value::Hash(entries), "empty?") => {
                    return Ok(Value::Bool(entries.is_empty()));
                }
                (Value::Text(text), "empty?") => {
                    return Ok(Value::Bool(text.is_empty()));
                }
                // C006 gives a Range endpoint and openness APIs.
                (Value::Range(range), "start") => {
                    return Ok(Value::Integer(range.start.clone()));
                }
                (Value::Range(range), "end") => {
                    return Ok(Value::Integer(range.end.clone()));
                }
                (Value::Range(range), "inclusive_end?") => {
                    return Ok(Value::Bool(range.inclusive_end));
                }
                _ => {}
            }
        }
        // C014 gives Iteration the get-only properties `yield?`, `done?` and
        // `value`. `Iteration.done.value` raises IteratorStateError rather than
        // answering nil, which is what keeps a yielded nil distinguishable
        // from exhaustion.
        if arguments.is_empty() {
            match (&receiver, selector) {
                (Value::IterationYield(_), "yield?") => return Ok(Value::Bool(true)),
                (Value::IterationYield(_), "done?") => return Ok(Value::Bool(false)),
                (Value::IterationYield(payload), "value") => {
                    return Ok((**payload).clone());
                }
                (Value::IterationDone, "yield?") => return Ok(Value::Bool(false)),
                (Value::IterationDone, "done?") => return Ok(Value::Bool(true)),
                (Value::IterationDone, "value") => {
                    return Err(EvaluationError::IteratorState);
                }
                _ => {}
            }
        }
        // C087 fixes a SPECIFICATION-STABLE public hash per value family, and
        // C089 forbids falling back to object identity for an identity-less
        // wrapper such as a Symbol or an Iteration.
        if selector == "hash"
            && arguments.is_empty()
            && matches!(
                receiver,
                Value::Symbol(_)
                    | Value::IterationDone
                    // An IterationYield is absent deliberately: C089 composes
                    // over its PAYLOAD's hash, which needs a real dispatch so
                    // an unhashable payload propagates InvalidKeyError. The
                    // arm below does that.
                    | Value::Range(..)
                    // C022 makes a Tuple hash succeed only when every element
                    // hash does, and `public_hash` already propagates the
                    // failure of an unhashable element.
                    // C077 hashes canonical pattern text plus canonical flags.
                    | Value::Regex(_)
                    // C068 makes the Bytes hash stable. A ByteArray is absent
                    // deliberately: its built-in hash RAISES InvalidKeyError.
                    | Value::Bytes(_)
            )
        {
            return iris_runtime::public_hash(&receiver)
                .map(Value::Integer)
                .map_err(|_| EvaluationError::Runtime(iris_runtime::KernelError::Type));
        }
        if selector == "hash"
            && arguments.is_empty()
            && let Value::IterationYield(payload) = &receiver
        {
            // C089 composes over the PAYLOAD's own public hash, so an
            // unhashable payload propagates InvalidKeyError unchanged instead
            // of collapsing into a type failure.
            let payload = self.send(payload.as_ref().clone(), "hash", &[])?;
            let Value::Integer(payload) = payload else {
                return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
            };
            return Ok(Value::Integer(iris_runtime::iteration_hash(
                payload.to_u64(),
            )));
        }
        if selector == "hash"
            && arguments.is_empty()
            && let Value::Tuple(elements) = &receiver
        {
            let mut hashes = Vec::with_capacity(elements.len());
            for element in elements.clone() {
                // A failed element hash propagates unchanged, so an unhashable
                // element answers InvalidKeyError rather than a type failure.
                let Value::Integer(hash) = self.send(element, "hash", &[])? else {
                    return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
                };
                hashes.push(hash.to_u64().unwrap_or_default());
            }
            return Ok(Value::Integer(iris_runtime::tuple_hash(&hashes)));
        }
        // C026, C055 and C068 make the built-in hash of an Array,
        // MutableString and ByteArray RAISE InvalidKeyError rather than answer
        // a value, which is what keeps a mutable container out of a Hash key
        // position.
        if selector == "hash"
            && arguments.is_empty()
            && matches!(
                receiver,
                Value::Array(_) | Value::MutableString(_) | Value::ByteArray(_)
            )
        {
            return Err(EvaluationError::InvalidKeyError);
        }
        if selector == "hash"
            && arguments.is_empty()
            && let Value::ExceptionContext(identity, ..) = &receiver
        {
            return Ok(Value::Integer(identity.raw().into()));
        }
        // D-242 makes a named Contract Type's hash nominal, so a Contract
        // answers it directly rather than through structural member shape.
        if selector == "hash"
            && arguments.is_empty()
            && let Value::Contract(contract) = &receiver
        {
            return Ok(Value::Integer(self.contract_type_hash(*contract)?));
        }
        // C068 compares the exact current byte SEQUENCE and PERMITS cross-type
        // equality, so Bytes and ByteArray compare over their bytes rather
        // than over the two container kinds.
        if matches!(selector, "==" | "!=")
            && matches!(
                (&receiver, arguments.first()),
                (
                    Value::Bytes(_) | Value::ByteArray(_),
                    Some(Value::Bytes(_) | Value::ByteArray(_))
                )
            )
            && let [other] = arguments
        {
            let bytes = |value: &Value| match value {
                Value::Bytes(bytes) => Some(bytes.clone()),
                Value::ByteArray(bytes) => Some(bytes.bytes()),
                _ => None,
            };
            let equal = bytes(&receiver) == bytes(other);
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // C055 compares current exact scalar content and is CROSS-TYPE equal
        // to a String with identical content, so the comparison is over the
        // scalars rather than over the two container kinds.
        if matches!(selector, "==" | "!=")
            && matches!(
                (&receiver, arguments.first()),
                (
                    Value::MutableString(_),
                    Some(Value::MutableString(_) | Value::Text(_))
                ) | (Value::Text(_), Some(Value::MutableString(_)))
            )
            && let [other] = arguments
        {
            let scalars = |value: &Value| match value {
                Value::MutableString(text) => Some(text.text()),
                Value::Text(text) => Some(text.clone()),
                _ => None,
            };
            let equal = scalars(&receiver) == scalars(other);
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // C089 orders Iteration values: `done` equals itself, a yield and
        // `done` are UNORDERED, and two yields compare by payload. A payload
        // comparison that answers outside -1..1 breaks the Contract.
        if selector == "<=>"
            && let [other] = arguments
        {
            match (&receiver, other) {
                (Value::IterationDone, Value::IterationDone) => {
                    return Ok(Value::Integer(0_u8.into()));
                }
                (Value::IterationDone, Value::IterationYield(_))
                | (Value::IterationYield(_), Value::IterationDone) => return Ok(Value::Nil),
                (Value::IterationYield(left), Value::IterationYield(right)) => {
                    let ordering = self.send(
                        left.as_ref().clone(),
                        "<=>",
                        std::slice::from_ref(right.as_ref()),
                    )?;
                    let Value::Integer(value) = &ordering else {
                        return Err(EvaluationError::ComparisonContractError);
                    };
                    if !matches!(value.to_i128(), Some(-1..=1)) {
                        return Err(EvaluationError::ComparisonContractError);
                    }
                    return Ok(ordering);
                }
                _ => {}
            }
        }
        // C003 classifies an `Iteration<T>` yield as IDENTITY-LESS and
        // immutable, so two yields of equal payloads are equal and `same?`
        // has no answer to give. `Iteration.done` is an identity-bearing
        // SINGLETON, so it is `same?` as itself.
        if matches!(selector, "==" | "!=")
            && matches!(receiver, Value::IterationYield(_) | Value::IterationDone)
            && let [other] = arguments
        {
            let equal = &receiver == other;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        if selector == "same?"
            && let [other] = arguments
        {
            match (&receiver, other) {
                (Value::IterationDone, Value::IterationDone) => return Ok(Value::Bool(true)),
                // C003 classifies String, Symbol, Tuple, Range, Bytes and Regex
                // as IDENTITY-LESS, and C089 forbids falling back to object
                // identity for such a value, so an identity question about one
                // has no answer to give and raises.
                (
                    Value::Text(_)
                    | Value::Symbol(_)
                    | Value::Tuple(_)
                    | Value::Range(_)
                    | Value::Bytes(_)
                    | Value::Regex(_),
                    _,
                ) => return Err(EvaluationError::IdentityError),
                // C089 forbids falling back to object identity for an
                // identity-less wrapper, so this raises rather than comparing.
                (Value::IterationYield(_), _) | (_, Value::IterationYield(_)) => {
                    return Err(EvaluationError::IdentityError);
                }
                // C011 makes an Iterator identity-bearing with a mutable
                // cursor, so identity is the handle rather than its position.
                (Value::ArrayIterator(left), Value::ArrayIterator(right))
                | (Value::HashIterator(left), Value::HashIterator(right))
                | (Value::ByteIterator(left), Value::ByteIterator(right)) => {
                    return Ok(Value::Bool(left == right));
                }
                // C003 classifies Array, Hash, MutableString and ByteArray as
                // IDENTITY-BEARING, so `same?` asks whether the two handles
                // denote one container rather than whether contents match.
                (Value::Array(left), Value::Array(right)) => {
                    return Ok(Value::Bool(left.same(right)));
                }
                (Value::Hash(left), Value::Hash(right)) => {
                    return Ok(Value::Bool(left.same(right)));
                }
                (Value::MutableString(left), Value::MutableString(right)) => {
                    return Ok(Value::Bool(left.same(right)));
                }
                (Value::ByteArray(left), Value::ByteArray(right)) => {
                    return Ok(Value::Bool(left.same(right)));
                }
                _ => {}
            }
        }
        // C011 makes an Iterator identity-bearing, so two DISTINCT cursors over
        // equal contents are not equal, and advancing one does not change its
        // hash. Equality and hash therefore both key on the identity.
        if matches!(selector, "==" | "!=")
            && matches!(
                receiver,
                Value::ArrayIterator(_) | Value::HashIterator(_) | Value::ByteIterator(_)
            )
            && let [other] = arguments
        {
            let equal = match (&receiver, other) {
                (Value::ArrayIterator(left), Value::ArrayIterator(right))
                | (Value::HashIterator(left), Value::HashIterator(right))
                | (Value::ByteIterator(left), Value::ByteIterator(right)) => left == right,
                _ => false,
            };
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        if selector == "hash"
            && arguments.is_empty()
            && let Value::ArrayIterator(identity)
            | Value::HashIterator(identity)
            | Value::ByteIterator(identity) = &receiver
        {
            // C003 makes an Iterator hash RUNTIME-LOCAL, stable for the
            // lifetime of one identity, so advancing the cursor cannot move it.
            return Ok(Value::Integer(identity.raw().into()));
        }
        // C043 makes an FFI::Library IDENTITY-BEARING, so two Libraries are
        // equal only when they are the same object. Two opens of one path are
        // two objects, which V069 observes.
        if matches!(selector, "==" | "!=")
            && let Value::Library(left) = &receiver
            && let [Value::Library(right)] = arguments
        {
            let equal = left.identity == right.identity;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // C077 makes Regex an immutable identity-LESS value whose equality
        // uses canonical pattern text plus canonical flags, so `/a+/im` and
        // `/a+/mi` are equal once C081 has ordered the flags.
        if matches!(selector, "==" | "!=")
            && matches!(receiver, Value::Regex(_))
            && let [other] = arguments
        {
            let equal = &receiver == other;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // C043 compares the exact scalar SEQUENCE and case, with no implicit
        // normalization, case folding, locale mapping or grapheme
        // equivalence. A Symbol is an identity-less immutable value under
        // C003, so it compares the same way. Both are value comparisons.
        if matches!(selector, "==" | "!=")
            && matches!(receiver, Value::Text(_) | Value::Symbol(_))
            && let [other] = arguments
        {
            let equal = &receiver == other;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        // C022 compares Tuple ARITY and then elements in order, which the
        // derived value equality performs, and C021 makes a Tuple identity-less
        // so this is a value comparison rather than an identity one.
        if matches!(selector, "==" | "!=")
            && matches!(receiver, Value::Tuple(_))
            && let [other] = arguments
        {
            let equal = &receiver == other;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        if matches!(selector, "==" | "!=")
            && matches!(
                receiver,
                Value::Closure(_) | Value::BoundMethod(_) | Value::ExceptionContext(..)
            )
        {
            let [other] = arguments else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let equal = same_identity(&receiver, other);
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        let selector_id = iris_runtime::NativeSelector::from_source(selector)
            .map_or_else(|| self.selector(selector), iris_runtime::NativeSelector::id);
        match self
            .kernel
            .dispatch_value(self.runtime.registry(), &receiver, selector_id)
            .map_err(EvaluationError::Runtime)?
        {
            iris_runtime::DispatchOutcome::Invoke(method) => {
                if matches!(selector, "==" | "!=" | "<" | "<=" | ">" | ">=")
                    && !self.bodies.contains_key(&method.body().raw())
                {
                    let comparison = self.value_send(receiver, "<=>", arguments)?;
                    let result = match comparison {
                        Value::Nil => selector == "!=",
                        Value::Integer(value) if value == (-1_i8).into() => {
                            matches!(selector, "!=" | "<" | "<=")
                        }
                        Value::Integer(value) if value == 0_u8.into() => {
                            matches!(selector, "==" | "<=" | ">=")
                        }
                        Value::Integer(value) if value == 1_u8.into() => {
                            matches!(selector, "!=" | ">" | ">=")
                        }
                        Value::Integer(_)
                        | Value::Bool(_)
                        | Value::Float32(_)
                        | Value::Float64(_)
                        | Value::Array(_)
                        | Value::Bytes(_)
                        | Value::ByteArray(_)
                        | Value::MutableString(_)
                        | Value::Regex(_)
                        | Value::Match(_)
                        | Value::Library(_)
                        | Value::Gate(_)
                        | Value::Tuple(_)
                        | Value::Hash(_)
                        | Value::Text(_)
                        | Value::Symbol(_)
                        | Value::Class(_)
                        | Value::Type(..)
                        | Value::ComposedType(_)
                        | Value::Contract(_)
                        | Value::Closure(_)
                        | Value::KeywordArgument(_, _)
                        | Value::IterationYield(_)
                        | Value::ReadonlyArray(_)
                        | Value::SourceLocation(..)
                        | Value::StackFrame(..)
                        | Value::RaiseSite(_)
                        | Value::ArrayIterator(_)
                        | Value::HashIterator(_)
                        | Value::ByteIterator(_)
                        | Value::Generator(_)
                        | Value::Task(_)
                        | Value::Range(..)
                        | Value::IterationDone
                        | Value::Transformation { .. }
                        | Value::ExceptionContext(..)
                        | Value::ContractView(_, _)
                        | Value::Object(_)
                        | Value::BoundMethod(_)
                        | Value::Method(_) => {
                            return Err(EvaluationError::UnsupportedConstruct);
                        }
                    };
                    return Ok(Value::Bool(result));
                }
                self.invoke_selected(method, receiver, arguments)
            }
            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                Err(EvaluationError::MessageNotFound {
                    receiver_class: receiver_class_name(&receiver).into(),
                    selector: selector.into(),
                })
            }
        }
    }

    fn assign_value_raw_ivar(&self, receiver: Value) -> Result<Value, EvaluationError> {
        let class = match receiver {
            Value::Nil => self.kernel.class(iris_runtime::BuiltinClass::Nil),
            Value::Bool(_) => self.kernel.class(iris_runtime::BuiltinClass::Bool),
            Value::Integer(_) => self.kernel.class(iris_runtime::BuiltinClass::Integer),
            Value::Float32(_) => self.kernel.class(iris_runtime::BuiltinClass::Float32),
            Value::Float64(_) => self.kernel.class(iris_runtime::BuiltinClass::Float64),
            Value::Array(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::MutableString(_)
            | Value::Regex(_)
            | Value::Match(_)
            | Value::Library(_)
            | Value::Gate(_)
            | Value::Tuple(_)
            | Value::Hash(_)
            | Value::Text(_)
            | Value::Symbol(_)
            | Value::Class(_)
            | Value::Type(..)
            | Value::ComposedType(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::ReadonlyArray(_)
            | Value::SourceLocation(..)
            | Value::StackFrame(..)
            | Value::RaiseSite(_)
            | Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::ByteIterator(_)
            | Value::Generator(_)
            | Value::Task(_)
            | Value::Range(..)
            | Value::IterationDone
            | Value::Transformation { .. }
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::Object(_)
            | Value::BoundMethod(_)
            | Value::Method(_) => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Runtime)?;
        self.kernel
            .require_meta_capability(self.runtime.registry(), class, Capability::InstanceState)
            .map_err(|_| iris_runtime::ConstructionError::InstanceState { class })
            .map_err(EvaluationError::Construction)
            .map(|()| receiver)
    }

    /// Allocates a fresh identity for one propagation event.
    ///
    /// `IRIS-V1-CONTROL-C056` makes every `raise` a DISTINCT event, so this
    /// never reuses an identity even when the raised value is the same.
    /// Consumes one step of the evaluation budget.
    fn charge_step(&mut self) -> Result<(), EvaluationError> {
        self.remaining_steps = self
            .remaining_steps
            .checked_sub(1)
            .ok_or(EvaluationError::StepBudgetExhausted)?;
        Ok(())
    }

    /// Builds a `SourceLocation` for a byte offset in the current source.
    ///
    /// `IRIS-V1-CONTROL-C079` makes `line` and `column` ONE-BASED, so both
    /// start at 1 and a byte offset of 0 is line 1, column 1.
    fn source_location(&self, offset: usize) -> Value {
        let consumed = self.source.get(..offset).unwrap_or(&self.source);
        let line = consumed.matches('\n').count() + 1;
        let column = consumed
            .rfind('\n')
            .map_or(consumed.chars().count(), |last| {
                consumed[last + 1..].chars().count()
            })
            + 1;
        Value::SourceLocation(
            self.source_path.clone(),
            u32::try_from(line).unwrap_or(u32::MAX),
            u32::try_from(column).unwrap_or(u32::MAX),
        )
    }

    fn next_context_identity(&mut self) -> iris_runtime::ObjectId {
        let identity = iris_runtime::ObjectId::new(self.next_closure);
        self.next_closure += 1;
        identity
    }

    fn selector(&mut self, name: &str) -> Selector {
        if name == "initialize" {
            return Selector::INITIALIZE;
        }
        if let Some(selector) = self.selectors.get(name) {
            return *selector;
        }
        let selector = iris_runtime::NativeSelector::from_source(name).map_or_else(
            || {
                let selector = Selector::new(self.next_selector);
                self.next_selector += 1;
                selector
            },
            iris_runtime::NativeSelector::id,
        );
        self.selectors.insert(name.into(), selector);
        selector
    }

    fn register_body(&mut self, method: MethodDeclaration) -> MethodBody {
        let body = MethodBody::new(self.next_body);
        self.next_body += 1;
        self.bodies.insert(body.raw(), method);
        body
    }

    fn comparison_slot(&mut self, selector: Selector) -> Option<ComparisonSlot> {
        [
            ("==", ComparisonSlot::Equal),
            ("!=", ComparisonSlot::NotEqual),
            ("<", ComparisonSlot::Less),
            ("<=", ComparisonSlot::LessEqual),
            (">", ComparisonSlot::Greater),
            (">=", ComparisonSlot::GreaterEqual),
        ]
        .into_iter()
        .find_map(|(name, slot)| (self.selector(name) == selector).then_some(slot))
    }

    /// Derives a comparison result for a receiver whose slot is still the default.
    ///
    /// `IRIS-V1-RUNTIME-C086` requires equality to test reference identity FIRST
    /// and return `true` without consulting `<=>`, which is why an object whose
    /// `<=>` always answers `nil` still equals itself. Otherwise the current
    /// visible `<=>` is sent, exactly once, and `IRIS-V1-RUNTIME-C084` maps its
    /// response, with `nil` meaning unordered. `IRIS-V1-RUNTIME-C085` rejects any
    /// response outside `Integer(-1)`, `Integer(0)`, `Integer(1)` and `nil`.
    fn default_comparison(
        &mut self,
        object: iris_runtime::ObjectId,
        slot: ComparisonSlot,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [other] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if matches!(slot, ComparisonSlot::Equal | ComparisonSlot::NotEqual)
            && *other == Value::Object(object)
        {
            return Ok(Value::Bool(matches!(slot, ComparisonSlot::Equal)));
        }
        let ordering = match self.send(Value::Object(object), "<=>", std::slice::from_ref(other))? {
            Value::Nil => None,
            Value::Integer(value) if value == (-1_i8).into() => Some(-1_i8),
            Value::Integer(value) if value == 0_u8.into() => Some(0_i8),
            Value::Integer(value) if value == 1_u8.into() => Some(1_i8),
            // D-094 separates two failures. Another Integer satisfies the broad
            // `Integer?` return type but violates the protocol, so it is a
            // ComparisonContractError. A non-Integer, non-nil result violates the
            // return type contract itself and is a TypeError.
            Value::Integer(_) => return Err(EvaluationError::ComparisonContractError),
            _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
        };
        Ok(Value::Bool(match (slot, ordering) {
            (ComparisonSlot::Equal, Some(0))
            | (ComparisonSlot::Less, Some(-1))
            | (ComparisonSlot::LessEqual, Some(-1 | 0))
            | (ComparisonSlot::Greater, Some(1))
            | (ComparisonSlot::GreaterEqual, Some(0 | 1))
            | (ComparisonSlot::NotEqual, None) => true,
            (ComparisonSlot::NotEqual, Some(order)) => order != 0,
            _ => false,
        }))
    }

    /// Evaluates `value is T` against the receiver's CURRENT runtime ancestry.
    ///
    /// `IRIS-V1-TYPES-C028` requires the test to consult current runtime
    /// ancestry, so a committed superclass change is observable here, and
    /// forbids it from sending `to_bool` or converting the value. `C009` makes
    /// `Object` the top Type, so every value answers true for it.
    fn type_test(&mut self, value: &Value, target: &Value) -> Result<Value, EvaluationError> {
        let Value::Class(target) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if self
            .kernel
            .class(iris_runtime::BuiltinClass::Object)
            .is_ok_and(|root| root == *target)
        {
            return Ok(Value::Bool(true));
        }
        let class = match value {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(EvaluationError::Construction)?,
            Value::Nil => self
                .kernel
                .class(iris_runtime::BuiltinClass::Nil)
                .map_err(EvaluationError::Runtime)?,
            Value::Bool(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Bool)
                .map_err(EvaluationError::Runtime)?,
            Value::Integer(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Integer)
                .map_err(EvaluationError::Runtime)?,
            Value::Float32(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Float32)
                .map_err(EvaluationError::Runtime)?,
            Value::Float64(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Float64)
                .map_err(EvaluationError::Runtime)?,
            Value::Class(class) => *class,
            // C041 makes a String a value with its own builtin Class, so `is
            // String` must resolve it like the numeric value Classes rather
            // than falling through to the ordinary-object arm.
            Value::Text(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::String)
                .map_err(EvaluationError::Runtime)?,
            Value::Array(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::MutableString(_)
            | Value::Regex(_)
            | Value::Match(_)
            | Value::Library(_)
            | Value::Gate(_)
            | Value::Tuple(_)
            | Value::Hash(_)
            | Value::Symbol(_)
            | Value::Type(..)
            | Value::ComposedType(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::ReadonlyArray(_)
            | Value::SourceLocation(..)
            | Value::StackFrame(..)
            | Value::RaiseSite(_)
            | Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::ByteIterator(_)
            | Value::Generator(_)
            | Value::Task(_)
            | Value::Range(..)
            | Value::IterationDone
            | Value::Transformation { .. }
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::BoundMethod(_)
            | Value::Method(_) => {
                return Ok(Value::Bool(false));
            }
        };
        let ancestry = self
            .runtime
            .registry()
            .active(class)
            .map_err(EvaluationError::Class)?
            .mro()
            .iter()
            .any(|entry| matches!(entry, iris_runtime::MroEntry::Class(entry) if entry == target));
        Ok(Value::Bool(ancestry))
    }

    /// Answers `Type#subtype?` over current nominal ancestry.
    ///
    /// `IRIS-V1-TYPES-C079` requires this query to use the SAME nominal rules as
    /// the runtime guards, so it walks the same active MRO that `is` consults
    /// rather than a separately cached relation. `C009` makes `Object` the top
    /// Type, so every nominal Type is its subtype.
    fn subtype(&mut self, left: ClassId, right: ClassId) -> Result<Value, EvaluationError> {
        if self
            .kernel
            .class(iris_runtime::BuiltinClass::Object)
            .is_ok_and(|root| root == right)
        {
            return Ok(Value::Bool(true));
        }
        let ancestry = self
            .runtime
            .registry()
            .active(left)
            .map_err(EvaluationError::Class)?
            .mro()
            .iter()
            .any(|entry| matches!(entry, iris_runtime::MroEntry::Class(entry) if *entry == right));
        Ok(Value::Bool(ancestry))
    }

    /// Builds the Contract view that `value as ContractType` proves.
    ///
    /// `IRIS-V1-TYPES-C049` requires the `as ContractType` part to prove or check
    /// the nominal Contract view, so a receiver whose Class never declared that
    /// Contract with `for` is rejected rather than silently viewed.
    /// Validates a closed construction's arguments against its declared bounds.
    ///
    /// `IRIS-V1-TYPES-C058` lets a concrete argument satisfy an F-bounded
    /// constraint such as `where T: Comparable<T>` ONLY through explicit
    /// nominal Class or Contract conformance; matching member shape is
    /// insufficient, so the declared `for` list is consulted rather than the
    /// argument's members. `C067` makes the failure raise.
    fn check_generic_bounds(
        &mut self,
        name: &str,
        arguments: &[iris_syntax::TypeExpression],
    ) -> Result<(), EvaluationError> {
        let Some((parameters, constraints)) = self.generic_bounds.get(name).cloned() else {
            return Ok(());
        };
        for constraint in &constraints {
            let Some(position) = parameters.iter().position(|p| *p == constraint.parameter) else {
                continue;
            };
            let Some(iris_syntax::TypeExpression::Name(argument)) = arguments.get(position) else {
                continue;
            };
            // Only a NOMINAL bound asks a conformance question here.
            let bound = match &constraint.bound {
                iris_syntax::TypeExpression::Generic { name, .. }
                | iris_syntax::TypeExpression::Name(name) => name,
                _ => continue,
            };
            // C067 validates EVERY normalized constraint at materialization, so
            // a Type bound such as `NonNil` is checked alongside a Contract
            // one. `Box<Nil>` against `where T: NonNil` is exactly V242.
            if bound == "NonNil" {
                let nil = self
                    .kernel
                    .class(iris_runtime::BuiltinClass::Nil)
                    .map_err(EvaluationError::Runtime)?;
                if self.class_name(argument)? == Some(nil) {
                    return Err(EvaluationError::TypeContractError);
                }
                continue;
            }
            if bound == "Never" {
                // C023 makes `Never` uninhabited, so no argument satisfies it.
                return Err(EvaluationError::TypeContractError);
            }
            let Some(contract) = self.contract_names.get(bound).copied() else {
                continue;
            };
            let conforms = self
                .class_name(argument)?
                .and_then(|class| self.class_contracts.get(&class))
                .is_some_and(|declared| declared.contains(&contract));
            if !conforms {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }

    /// Renders one Type constituent for reflection.
    ///
    /// A nested union stays ONE member, which is what `IRIS-V1-TYPES-V214`
    /// observes when it requires `A` and the normalized union rather than
    /// distributed alternatives.
    fn reflect_atom(&self, atom: &iris_runtime::TypeAtom) -> Value {
        match atom {
            iris_runtime::TypeAtom::Nominal(class, arguments) => {
                Value::Type(*class, arguments.clone())
            }
            iris_runtime::TypeAtom::NonNil => Value::Symbol("NonNil".to_owned()),
            iris_runtime::TypeAtom::Contract(contract) => Value::Contract(*contract),
            iris_runtime::TypeAtom::Union(nested) => {
                Value::ComposedType(iris_runtime::ComposedType::Union(nested.clone()))
            }
        }
    }

    /// Compares two Contract-view receivers under `IRIS-V1-TYPES-C050`.
    ///
    /// An identity-bearing receiver requires the same IDENTITY; an
    /// identity-less one requires equality under current equality, which is the
    /// ordinary `==` send rather than a structural comparison.
    fn view_receivers_equal(
        &mut self,
        left: &Value,
        right: &Value,
    ) -> Result<bool, EvaluationError> {
        if let (Value::Object(left), Value::Object(right)) = (left, right) {
            return Ok(left == right);
        }
        match self.send(left.clone(), "==", std::slice::from_ref(right))? {
            Value::Bool(equal) => Ok(equal),
            _ => Err(EvaluationError::ComparisonContractError),
        }
    }

    /// The Class of a value that may carry a Contract view.
    ///
    /// `IRIS-V1-TYPES-C050` defines view equality over IDENTITY-LESS receivers
    /// as well as identity-bearing ones, so a value Class may be viewed. A
    /// value with no viewable Class is not an error here; it simply cannot
    /// declare a Contract, which the caller reports.
    fn class_of_value(&mut self, value: &Value) -> Result<ClassId, EvaluationError> {
        use iris_runtime::BuiltinClass;
        let builtin = match value {
            Value::Object(object) => {
                return self
                    .runtime
                    .class_of(*object)
                    .map_err(EvaluationError::Construction);
            }
            Value::Nil => BuiltinClass::Nil,
            Value::Bool(_) => BuiltinClass::Bool,
            Value::Integer(_) => BuiltinClass::Integer,
            Value::Float32(_) => BuiltinClass::Float32,
            Value::Float64(_) => BuiltinClass::Float64,
            Value::Text(_) => BuiltinClass::String,
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        self.kernel.class(builtin).map_err(EvaluationError::Runtime)
    }

    fn contract_view(&mut self, value: Value, target: &Value) -> Result<Value, EvaluationError> {
        let Value::Contract(contract) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        // C050 defines view equality over IDENTITY-LESS receivers too, so a
        // value Class may be viewed as well as an identity-bearing object.
        // Restricting this to `Value::Object` made `1 as N` unconstructible.
        let class = self.class_of_value(&value)?;
        if !self
            .class_contracts
            .get(&class)
            .is_some_and(|declared| declared.contains(contract))
        {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
        }
        Ok(Value::ContractView(Box::new(value), *contract))
    }

    /// Sends `view..member(args)` to the Contract-qualified slot.
    ///
    /// `IRIS-V1-TYPES-C049` states that `..member` chooses the Contract-qualified
    /// slot identity while ordinary `value.member` always sends the unqualified
    /// selector, so this resolves ONLY the qualified table and never falls back
    /// to ordinary dispatch, which would merge the two namespaces.
    fn qualified_send(
        &mut self,
        view: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let Value::ContractView(receiver, contract) = view else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let Value::Object(object) = *receiver else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        let selector_id = self.selector(selector);
        let method = match self.qualified_methods.get(&(class, contract, selector_id)) {
            Some(method) => *method,
            // C047: ONE unqualified `impl` member automatically satisfies every
            // declared same-name Contract requirement. Only a qualified slot
            // was consulted, so a Class whose `impl fun m` is unqualified could
            // never be reached through its own Contract view.
            None if self.satisfies_unqualified(class, contract, selector) => {
                self.resolve_instance_method(object, selector_id)?
            }
            // A missing qualified slot is a Contract dispatch failure, NOT a
            // missing message: IRIS-V1-TYPES-C049 keeps the qualified namespace
            // separate, so `method_missing` must not be reached from here.
            None => {
                return Err(EvaluationError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::ContractDispatch {
                            contract: iris_runtime::ModuleId::new(contract.raw()),
                            selector: selector_id,
                        },
                    ),
                ));
            }
        };
        self.invoke_method(method, Value::Object(object), arguments)
    }

    /// Reports whether an unqualified `impl` member satisfies a Contract slot.
    ///
    /// `IRIS-V1-TYPES-C047` lets ONE `impl` member satisfy every declared
    /// same-name requirement without listing targets, so a view send resolves
    /// to it when no qualified slot exists. The Class must actually DECLARE the
    /// Contract, which keeps this from turning an unrelated same-name Method
    /// into an accidental implementation.
    fn satisfies_unqualified(
        &self,
        class: ClassId,
        contract: iris_runtime::ContractId,
        selector: &str,
    ) -> bool {
        self.class_contracts
            .get(&class)
            .is_some_and(|declared| declared.contains(&contract))
            && self
                .contract_requirements
                .get(&contract)
                .is_some_and(|names| names.iter().any(|name| name == selector))
    }

    /// Invokes a Closure with its captured environment restored.
    ///
    /// `IRIS-V1-RUNTIME-C072` keeps the captured receiver fixed, so the body runs
    /// against the receiver captured at creation rather than any current one,
    /// which is what lets an escaped Closure keep writing that receiver's ivars.
    fn invoke_closure(
        &mut self,
        object: iris_runtime::ObjectId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let record = self
            .closures
            .get(&object)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let parameters = record.parameters.clone();
        let body = record.body.clone();
        let receiver = record.receiver.clone();
        let mut locals = record.captured.clone();
        for (name, argument) in parameters.iter().zip(arguments) {
            locals.insert(name.clone(), argument.clone());
        }
        // The body runs through `block`, which scopes its bindings. Running the
        // statements directly would let a `let` inside the Closure write into
        // the enclosing binding map and OVERWRITE an outer name of the same
        // spelling, which `IRIS-V1-CONTROL-C028` forbids: a nested block
        // shadows a captured binding rather than replacing it.
        // C050 refuses the Host drive surface inside a Closure, since a
        // Closure can escape and be resumed anywhere, which would make the
        // surface an Iris source-level blocking wait C015 forbids.
        self.closure_depth += 1;
        let outcome = match self.block(&body, &locals, receiver) {
            // A Closure `return` ends only this invocation under D-421, so it
            // is absorbed here rather than reaching an enclosing Method.
            Err(EvaluationError::Return(value)) => Ok(value),
            result => result,
        };
        self.closure_depth -= 1;
        outcome
    }

    fn resolve_instance_method(
        &self,
        object: iris_runtime::ObjectId,
        selector: Selector,
    ) -> Result<Method, EvaluationError> {
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        let context = match self.current_method.map(|method| method.owner()) {
            Some(MethodOwner::Module(module)) => {
                DispatchContext::module_implementation(module, true)
            }
            Some(MethodOwner::Class(owner)) => DispatchContext::implementation(owner, true),
            // A Module body executes AS its `main`, so the send is privileged
            // even though no Method frame is active.
            None => match self.module_body_main {
                Some(main) if main == class => DispatchContext::implementation(main, true),
                _ => DispatchContext::external(),
            },
        };
        match self
            .runtime
            .registry()
            .dispatch_with_context(class, selector, context)
            .map_err(iris_runtime::ConstructionError::from)
        {
            Ok(DispatchOutcome::Invoke(method)) => Ok(method),
            Ok(DispatchOutcome::WouldInvokeMethodMissing { .. }) => self
                .class_mixins
                .get(&class)
                .into_iter()
                .flatten()
                .rev()
                .find_map(|class| self.runtime.registry().dispatch(*class, selector).ok())
                .and_then(|outcome| match outcome {
                    iris_runtime::DispatchOutcome::Invoke(method) => Some(method),
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => None,
                })
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { selector },
                    ),
                )),
            Err(iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MissingMethod { .. },
            )) => self
                .class_mixins
                .get(&class)
                .into_iter()
                .flatten()
                .rev()
                .find_map(|class| self.runtime.registry().dispatch(*class, selector).ok())
                .and_then(|outcome| match outcome {
                    iris_runtime::DispatchOutcome::Invoke(method) => Some(method),
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => None,
                })
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { selector },
                    ),
                )),
            Err(error) => Err(EvaluationError::Construction(error)),
        }
    }

    fn invoke_method_missing(
        &mut self,
        object: iris_runtime::ObjectId,
        missing: Selector,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        // C049 gives every Object a root `to_string` answering
        // `<fully.qualified.ClassName>` from NOMINAL Class identity, omitting
        // the address, identity hash, runtime ID, revision number, properties
        // and ivars, with `inspect` initially delegating to it. This is reached
        // only after instance dispatch found nothing, so a DECLARED
        // `to_string` still wins.
        let missing_name = self.selector_name(missing);
        // C003 makes an ordinary object IDENTITY-BEARING, so `same?` compares
        // the two identities. Like `to_string` this is reached only after
        // instance dispatch found nothing, so a declared one still wins.
        if missing_name == "same?"
            && let [other] = arguments
        {
            return Ok(Value::Bool(
                matches!(other, Value::Object(other) if *other == object),
            ));
        }
        if matches!(missing_name.as_str(), "to_string" | "inspect") && arguments.is_empty() {
            let class = self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?;
            let name = self.class_source_name(class);
            return Ok(Value::Text(format!("<{}::{name}>", self.package)));
        }
        let fallback = self.selector("method_missing");
        if missing == fallback {
            return Err(EvaluationError::MessageNotFound {
                receiver_class: self.object_class_name(object)?,
                selector: "method_missing".into(),
            });
        }
        let method = match self.resolve_instance_method(object, fallback) {
            Ok(method) => method,
            Err(EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MissingMethod { .. },
            ))) => {
                return Err(EvaluationError::MessageNotFound {
                    receiver_class: self.object_class_name(object)?,
                    selector: self.selector_name(missing),
                });
            }
            Err(error) => return Err(error),
        };
        // IRIS-V1-RUNTIME-C099 passes the trailing block as the separate
        // `block: Closure?` parameter, NOT inside the positional argument
        // snapshot, so a Closure in final position is split out here.
        let (positional, block) = match arguments {
            [head @ .., Value::Closure(closure)] => (head.to_vec(), Value::Closure(*closure)),
            _ => (arguments.to_vec(), Value::Nil),
        };
        self.invoke_method(
            method,
            Value::Object(object),
            &[
                Value::Symbol(self.selector_name(missing)),
                Value::Array(ArrayRef::new(positional)),
                block,
            ],
        )
    }

    fn selector_name(&self, selector: Selector) -> String {
        self.selectors
            .iter()
            .find_map(|(name, id)| (*id == selector).then(|| name.clone()))
            .unwrap_or_else(|| "<unknown>".into())
    }

    fn object_class_name(&self, object: iris_runtime::ObjectId) -> Result<String, EvaluationError> {
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        Ok(self
            .names
            .iter()
            .find_map(|(name, binding)| {
                matches!(binding.value, Value::Class(candidate) if candidate == class)
                    .then(|| name.clone())
            })
            .unwrap_or_else(|| "Object".into()))
    }

    fn validate_module_overrides(
        &self,
        superclass: Option<ClassId>,
        mixins: &[CompositionEdge],
    ) -> Result<(), EvaluationError> {
        let Some(superclass) = superclass else {
            return Ok(());
        };
        for edge in mixins {
            let module = edge.module();
            for ((owner, selector), method) in &self.module_methods {
                if *owner != module {
                    continue;
                }
                let replaces = matches!(
                    self.runtime.registry().dispatch(superclass, *selector),
                    Ok(iris_runtime::DispatchOutcome::Invoke(_))
                );
                let is_override = self
                    .module_method_overrides
                    .get(&method.id())
                    .copied()
                    .unwrap_or(false);
                if replaces && !is_override {
                    return Err(EvaluationError::Class(ClassError::OverrideRequired {
                        class: superclass,
                        selector: *selector,
                    }));
                }
                if !replaces && is_override {
                    return Err(EvaluationError::Class(ClassError::OverrideWithoutTarget {
                        class: superclass,
                        selector: *selector,
                    }));
                }
            }
        }
        Ok(())
    }

    fn read_class_var(&mut self, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        let class = self.class_var_owner(selector)?;
        self.runtime
            .class_var(class, selector)
            .map_err(EvaluationError::Construction)?
            .ok_or(EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable {
                    class,
                    name: selector,
                },
            ))
    }

    fn assign_class_var(&mut self, name: &str, value: Value) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        let class = self.class_var_owner(selector)?;
        self.runtime
            .assign_class_var(class, selector, value)
            .map_err(EvaluationError::Construction)
    }

    fn class_var_owner(&self, name: Selector) -> Result<ClassId, EvaluationError> {
        let class = self
            .lexical_class
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        self.class_var_owner_from(class, name)
    }

    fn class_var_owner_from(
        &self,
        mut class: ClassId,
        name: Selector,
    ) -> Result<ClassId, EvaluationError> {
        loop {
            if self
                .runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?
                .class_vars()
                .contains(&name)
            {
                return Ok(class);
            }
            class = self
                .static_superclasses
                .get(&class)
                .copied()
                .flatten()
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::MissingDeclaredClassVariable { class, name },
                ))?;
        }
    }

    fn invoke_method(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let previous_lexical_class = self.lexical_class;
        let previous_method = self.current_method;
        self.lexical_class = match method.owner() {
            MethodOwner::Class(class) => Some(class),
            MethodOwner::Module(module) => self.module_classes.get(&module).copied(),
        };
        self.current_method = Some(method);
        let result = self.invoke_method_with_context(method, receiver, arguments);
        self.lexical_class = previous_lexical_class;
        self.current_method = previous_method;
        result
    }

    /// Binds arguments to the parameter categories `IRIS-V1-CONTROL-C023` gives.
    ///
    /// Positionals fill in order, `*rest` takes the remaining positionals as a
    /// fresh Array, keyword parameters bind by NAME rather than position, and
    /// `**kwargs` collects unmatched keywords. An unfilled parameter takes its
    /// default, and `IRIS-V1-CONTROL-C025` raises ArgumentError when a required
    /// one is left unbound or a supplied argument matches nothing.
    fn bind_parameters(
        &mut self,
        parameters: &[iris_syntax::Parameter],
        arguments: &[Value],
    ) -> Result<HashMap<String, Value>, EvaluationError> {
        use iris_syntax::ParameterCategory;
        let mut positional = Vec::new();
        let mut keyword: Vec<(String, Value)> = Vec::new();
        for argument in arguments {
            match argument {
                Value::KeywordArgument(name, value) => {
                    // IRIS-V1-CONTROL-D-357 makes a duplicate keyword an
                    // ArgumentError rather than a silent last-one-wins.
                    if keyword.iter().any(|(seen, _)| seen == name) {
                        return Err(EvaluationError::ArgumentError);
                    }
                    keyword.push((name.clone(), value.as_ref().clone()));
                }
                value => positional.push(value.clone()),
            }
        }
        let mut locals = HashMap::new();
        let mut next = 0usize;
        for parameter in parameters {
            let name = parameter.name.clone();
            let bound = match parameter.category {
                ParameterCategory::Positional => {
                    let value = positional.get(next).cloned();
                    next += usize::from(value.is_some());
                    value
                }
                ParameterCategory::Rest => {
                    let rest = positional.split_off(next.min(positional.len()));
                    Some(Value::Array(ArrayRef::new(rest)))
                }
                ParameterCategory::Keyword => keyword
                    .iter()
                    .position(|(seen, _)| *seen == name)
                    .map(|index| keyword.remove(index).1),
                // `**kwargs` binds a fresh `Hash<Symbol,V>` of the keywords no
                // declared parameter matched.
                ParameterCategory::KeywordRest => {
                    let rest = std::mem::take(&mut keyword)
                        .into_iter()
                        .map(|(name, value)| (Value::Symbol(name), value))
                        .collect();
                    Some(Value::Hash(HashRef::new(rest)))
                }
                // C025 binds an omitted optional block to `nil`, so the block
                // channel is never a missing-argument error.
                ParameterCategory::Block => {
                    Some(positional.get(next).cloned().unwrap_or(Value::Nil))
                }
            };
            if parameter.category == ParameterCategory::Block {
                next += usize::from(next < positional.len());
            }
            let value = match bound {
                Some(value) => value,
                None => match &parameter.default {
                    Some(default) => self.expression(default, &locals, None)?,
                    None => return Err(EvaluationError::ArgumentError),
                },
            };
            // C004 guards the PARAMETER boundary: a written annotation is
            // checked before the argument is passed across it. A rest or
            // keyword-rest parameter collects into a container whose annotation
            // describes the ELEMENTS, so only a single-value parameter is
            // checked here.
            if let Some(annotation) = &parameter.annotation
                && matches!(
                    parameter.category,
                    ParameterCategory::Positional | ParameterCategory::Keyword
                )
            {
                self.check_binding_annotation(&value, annotation)?;
            }
            locals.insert(name, value);
        }
        // A leftover argument in either channel matches no parameter, which
        // C025 makes an arity error rather than a silent discard.
        if next < positional.len() || !keyword.is_empty() {
            return Err(EvaluationError::ArgumentError);
        }
        Ok(locals)
    }

    fn invoke_method_with_context(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        // Unbounded recursion is the other way a run fails to terminate.
        self.charge_step()?;
        self.invocation_depth += 1;
        if self.invocation_depth > DEPTH_BUDGET {
            self.invocation_depth -= 1;
            return Err(EvaluationError::StepBudgetExhausted);
        }
        let result = self.invoke_method_body(method, receiver, arguments);
        self.invocation_depth -= 1;
        result
    }

    fn invoke_method_body(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let Some(declaration) = self.bodies.get(&method.body().raw()) else {
            return Err(EvaluationError::Execution(
                iris_runtime::ExecutionError::Raised(Value::Nil),
            ));
        };
        let parameters = declaration.parameters.clone();
        let return_type = declaration.return_type.clone();
        let is_async = declaration.is_async;
        // A bodyless C062 requirement declares an obligation and supplies NO
        // implementation, so invoking one is not a call that can run. It cannot
        // reach here through a Contract, which is never instantiated, but a
        // requirement reached any other way must not execute as an empty body
        // returning nil.
        let Some(body) = declaration.body.clone() else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let locals = self.bind_parameters(&parameters, arguments)?;
        // C072 makes a callable containing `yield` a GENERATOR: invoking it
        // runs no body and returns an Iterator whose `next()` drives it. The
        // body is therefore not executed here at all.
        if body_yields(&body) {
            return Ok(self.new_generator(body, locals, receiver));
        }
        // C003 makes an async Method return `Task<T>` rather than `T`, and
        // C012 makes creating the Task and starting this initial run ONE call
        // operation. The body therefore runs now, and the Task records what it
        // produced.
        if is_async {
            // C050 refuses the Host drive surface inside an async body, which
            // is what keeps it from becoming the hidden Task join C015 forbids.
            self.async_depth += 1;
            let previous = self.async_replay.replace(GeneratorState {
                resume_past: 0,
                seen: 0,
            });
            let outcome = match self.block(&body, &locals, Some(receiver.clone())) {
                Err(EvaluationError::Return(value)) => Ok(value),
                result => result,
            };
            self.async_replay = previous;
            self.async_depth -= 1;
            let identity = self.next_context_identity();
            // C013 suspends rather than completing when the body awaited an
            // incomplete Awaitable, so the Task stays pending until a post.
            if let Err(EvaluationError::AwaitSuspended(gate)) = &outcome {
                self.suspended.insert(
                    identity,
                    SuspendedTask {
                        body,
                        locals,
                        receiver: Some(receiver),
                        delivered: Vec::new(),
                        gate: *gate,
                    },
                );
                return Ok(Value::Task(identity));
            }
            // C027 makes a failed Task eligible for unobserved-failure
            // reporting until something observes it.
            if let Err(error) = &outcome {
                let captured = match error {
                    EvaluationError::Raised(value) => value.clone(),
                    other => catchable_name(other)
                        .map_or(Value::Symbol("AsyncFailure".into()), Value::Symbol),
                };
                self.unobserved_failures.push((identity, captured));
            }
            self.tasks.insert(identity, outcome.map_err(Box::new));
            return Ok(Value::Task(identity));
        }
        let result = match self.block(&body, &locals, Some(receiver)) {
            Err(EvaluationError::Return(value)) => Ok(value),
            result => result,
        }?;
        // C004 guards the RETURN boundary before the value is published to the
        // caller, whether the body fell off the end or returned explicitly.
        if let Some(annotation) = &return_type {
            self.check_binding_annotation(&result, annotation)?;
        }
        Ok(result)
    }

    /// Creates the Iterator a generator invocation returns.
    ///
    /// `IRIS-V1-GRAMMAR-C072` runs NO body at invocation: the body is retained
    /// with the locals it was bound to, and `next()` drives it.
    fn new_generator(
        &mut self,
        body: Vec<Statement>,
        locals: HashMap<String, Value>,
        receiver: Value,
    ) -> Value {
        let identity = self.next_context_identity();
        self.generators.insert(
            identity,
            GeneratorBody {
                body,
                locals,
                receiver: Some(receiver),
                delivered: 0,
                finished: false,
            },
        );
        Value::Generator(identity)
    }

    /// Resumes a generator body until its next suspension.
    ///
    /// `IRIS-V1-GRAMMAR-C072` makes `next()` answer `Iteration.yield(value)`
    /// at each suspension and `Iteration.done` once the body completes, which
    /// is the `IRIS-V1-COLLECTIONS-C013` protocol every Iterator follows.
    ///
    /// The body is RE-ENTERED rather than resumed on a captured native stack:
    /// suspensions below the delivered count replay silently and the first at
    /// or above it suspends again. That is what keeps a generator off the
    /// native stack between resumptions, and it leaves ordinary synchronous
    /// evaluation untouched as `IRIS-V1-ASYNC-C011` requires.
    fn resume_generator(
        &mut self,
        identity: iris_runtime::ObjectId,
    ) -> Result<Value, EvaluationError> {
        let Some(state) = self.generators.get(&identity).cloned() else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if state.finished {
            // C013 returns the SAME `Iteration.done` singleton on every call
            // after exhaustion.
            return Ok(Value::IterationDone);
        }
        let previous = self.generator.replace(GeneratorState {
            resume_past: state.delivered,
            seen: 0,
        });
        let outcome = self.block(&state.body, &state.locals, state.receiver.clone());
        self.generator = previous;
        match outcome {
            Err(EvaluationError::GeneratorYield(value, index)) => {
                if let Some(state) = self.generators.get_mut(&identity) {
                    state.delivered = index + 1;
                }
                Ok(Value::IterationYield(Box::new(value)))
            }
            Ok(_) | Err(EvaluationError::Return(_)) => {
                if let Some(state) = self.generators.get_mut(&identity) {
                    state.finished = true;
                }
                Ok(Value::IterationDone)
            }
            Err(error) => Err(error),
        }
    }

    fn super_send(
        &mut self,
        receiver: Option<Value>,
        arguments: &[Value],
        selector: Option<&str>,
    ) -> Result<Value, EvaluationError> {
        let receiver = receiver.ok_or(EvaluationError::UnsupportedConstruct)?;
        let method = self
            .current_method
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let selector = selector.map_or(method.selector(), |name| self.selector(name));
        let successor = match receiver.clone() {
            Value::Object(object) => {
                let class = self
                    .runtime
                    .class_of(object)
                    .map_err(EvaluationError::Construction)?;
                self.runtime
                    .registry()
                    .dispatch_super_selector(class, method, selector)
                    .map_err(iris_runtime::KernelError::from)
                    .map_err(EvaluationError::Runtime)?
            }
            Value::Class(class) => self
                .runtime
                .registry()
                .dispatch_class_object_super_selector(class, method, selector)
                .map_err(iris_runtime::KernelError::from)
                .map_err(EvaluationError::Runtime)?,
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        self.invoke_method(successor, receiver, arguments)
    }
}

fn meta_capabilities(names: &[String]) -> Result<MetaCapabilities, EvaluationError> {
    let mut denied = Vec::with_capacity(names.len());
    for name in names {
        let capability = match name.as_str() {
            "method_set" => Capability::MethodSet,
            "method_body" => Capability::MethodBody,
            "property_set" => Capability::PropertySet,
            "property_body" => Capability::PropertyBody,
            "modules" => Capability::Modules,
            "superclass" => Capability::Superclass,
            "subclass" => Capability::Subclass,
            "shape" => Capability::Shape,
            "class_state_set" => Capability::ClassStateSet,
            "class_state_write" => Capability::ClassStateWrite,
            "instance_state" => Capability::InstanceState,
            "native" => Capability::Native,
            _ => return Err(EvaluationError::ParseDiagnostic),
        };
        denied.push(capability);
    }
    Ok(MetaCapabilities::denying(&denied))
}

fn receiver_class_name(value: &Value) -> &'static str {
    match value {
        Value::ComposedType(_) => "Type",
        Value::Nil => "Nil",
        Value::Bool(_) => "Bool",
        Value::Integer(_) => "Integer",
        Value::Float32(_) => "Float32",
        Value::Float64(_) => "Float64",
        Value::Array(_) => "Array",
        Value::Bytes(_) => "Bytes",
        Value::ByteArray(_) => "ByteArray",
        Value::MutableString(_) => "MutableString",
        Value::Regex(_) => "Regex",
        Value::Match(_) => "Match",
        Value::Library(_) => "FFI::Library",
        Value::Gate(_) => "Gate",
        Value::Hash(_) => "Hash",
        Value::Tuple(_) => "Tuple",
        Value::ReadonlyArray(_) => "ReadonlyArray",
        Value::SourceLocation(..) => "SourceLocation",
        Value::StackFrame(..) => "StackFrame",
        Value::RaiseSite(_) => "RaiseSite",
        Value::Text(_) => "String",
        Value::Symbol(_) => "Symbol",
        Value::Class(_) => "Class",
        Value::Type(..) => "Type",
        Value::Contract(_) => "Contract",
        Value::Closure(_) => "Closure",
        Value::KeywordArgument(_, _) | Value::IterationYield(_) => "Iteration",
        Value::ArrayIterator(..)
        | Value::HashIterator(..)
        | Value::ByteIterator(..)
        | Value::Generator(..)
        | Value::IterationDone => "Iteration",
        Value::Task(..) => "Task",
        Value::Range(..) => "Range",
        Value::ExceptionContext(..) => "ExceptionContext",
        Value::ContractView(_, _) => "ContractView",
        Value::Object(_) => "Object",
        Value::BoundMethod(_) => "BoundMethod",
        Value::Method(_) => "Method",
        Value::Transformation { .. } => "Transformation",
    }
}

/// Compares two values by IDENTITY rather than by payload.
///
/// `IRIS-V1-CONTROL-C067` makes `ExceptionContext` equality and hashing use
/// identity by default, and `D-155` lets a bare `raise` APPEND a re-raise site
/// to the context it continues. A structural comparison would therefore report
/// a retained context as a different one the moment it gained a site, so the
/// identity is compared alone.
fn same_identity(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::ExceptionContext(left, ..), Value::ExceptionContext(right, ..)) => left == right,
        _ => left == right,
    }
}

/// The evaluation budget for one program run.
///
/// Large enough that no legitimate conformance vector approaches it, small
/// enough that a non-terminating one fails in well under a second.
const STEP_BUDGET: u64 = 1_000_000;

/// The invocation depth bound for one program run.
///
/// Each Iris invocation consumes MANY host stack frames. Measured overflow is
/// near 115 frames on the main thread but near 28 on a default test thread, so
/// the bound is set below the tighter of the two. Recursion then fails as a
/// REPORTABLE error rather than aborting the process, which no test can catch
/// and which would take the whole conformance suite down with it.
const DEPTH_BUDGET: u32 = 16;

/// Maps a compound assignment to the ordinary operator selector it sends.
///
/// `IRIS-V1-CONTROL-C036` gives ten symbolic compound assignments and `D-348`
/// stresses that none of them is an independent selector, so each reuses its
/// ordinary operator. `Assign` writes directly and the logical forms
/// short-circuit under `C037`, so both yield `None`.
const fn compound_selector(operator: &iris_syntax::AssignmentOperator) -> Option<&'static str> {
    use iris_syntax::AssignmentOperator;
    match operator {
        AssignmentOperator::Add => Some("+"),
        AssignmentOperator::Subtract => Some("-"),
        AssignmentOperator::Multiply => Some("*"),
        AssignmentOperator::Divide => Some("/"),
        AssignmentOperator::Power => Some("**"),
        AssignmentOperator::BitwiseAnd => Some("&"),
        AssignmentOperator::BitwiseOr => Some("|"),
        AssignmentOperator::BitwiseXor => Some("^"),
        AssignmentOperator::ShiftLeft => Some("<<"),
        AssignmentOperator::ShiftRight => Some(">>"),
        AssignmentOperator::Assign
        | AssignmentOperator::LogicalAnd
        | AssignmentOperator::LogicalOr => None,
    }
}

/// Rejects a raw-ivar reflection name that is not an instance ivar.
///
/// `IRIS-V1-RUNTIME-C073` makes `@@name` a declared HIERARCHY binding cell
/// rather than instance state, and a name with no `@` sigil is an ordinary
/// selector, so neither addresses an ivar. `IRIS-V1-META-V362` requires
/// `InvalidInstanceVariableNameError` for both.
fn validate_ivar_name(argument: Option<&Value>) -> Result<(), EvaluationError> {
    let Some(Value::Symbol(name)) = argument else {
        return Ok(());
    };
    if name.starts_with("@@") || !name.starts_with('@') {
        return Err(EvaluationError::InvalidInstanceVariableName);
    }
    Ok(())
}

/// The `IRIS-V1-META-C081` vocabulary name of one capability.
const fn capability_name(capability: iris_runtime::Capability) -> &'static str {
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

/// The specification-named error a runtime failure reports, when it names one.
///
/// `IRIS-V1-CONTROL-C056` makes `raise value` accept any Iris value and hands
/// the ORIGINAL value to the catch, so a failure the specification names is
/// catchable under that name. Returning `None` keeps a failure that names no
/// such error, and every control-flow unwind, travelling to its own boundary
/// instead of being intercepted by an unrelated handler.
fn catchable_name(error: &EvaluationError) -> Option<String> {
    let name = match error {
        EvaluationError::TypeContractError => "TypeContractError",
        EvaluationError::ClosedGenericOpenForbidden => "CLOSED_GENERIC_OPEN_FORBIDDEN",
        EvaluationError::InvalidInstanceVariableName => "InvalidInstanceVariableNameError",
        // V423 catches the refusal and reads the surviving view, so the
        // mutation refusal must be an ordinary catchable Iris error.
        EvaluationError::ReadonlyMutation => "ReadonlyMutationError",
        // D-271 makes the failure an ordinary catchable Iris error.
        EvaluationError::RevisionArtifactUnavailable => "RevisionArtifactUnavailableError",
        // C102 and C103 make an ungranted or out-of-scope reflection call an
        // ordinary catchable Iris error; V363 and V421 observe it.
        EvaluationError::ReflectionAccess => "ReflectionAccessError",
        EvaluationError::IteratorState => "IteratorStateError",
        EvaluationError::IndexError => "IndexError",
        EvaluationError::KeyError => "KeyError",
        EvaluationError::ConcurrentModification => "ConcurrentModificationError",
        EvaluationError::KeyConflictError => "KeyConflictError",
        EvaluationError::EncodingError => "EncodingError",
        EvaluationError::RangeError => "RangeError",
        EvaluationError::InvalidKeyError => "InvalidKeyError",
        EvaluationError::RegexSyntaxError => "RegexSyntaxError",
        EvaluationError::UnboundNativeSymbol => "UnboundNativeSymbolError",
        EvaluationError::IncompleteNativeSignature => "IncompleteNativeSignatureError",
        EvaluationError::AuditHistoryUnavailable => "AuditHistoryUnavailableError",
        EvaluationError::SerializationError => "SerializationError",
        EvaluationError::JsonLimitError => "JSONLimitError",
        EvaluationError::JsonSyntaxError => "JSONSyntaxError",
        EvaluationError::HostDriveUnavailable => "HostDriveUnavailableError",
        EvaluationError::MetaTransactionSuspension => "MetaTransactionError",
        EvaluationError::IdentityError => "IdentityError",
        EvaluationError::ComparisonContractError => "ComparisonContractError",
        EvaluationError::ArgumentError => "ArgumentError",
        EvaluationError::PatternMatchError => "PatternMatchError",
        EvaluationError::NameError => "NameError",
        // IRIS-V1-META-C100 removes an EXISTING slot, and
        // IRIS-V1-META-V362 names the absent case.
        EvaluationError::InstanceVariableNotFoundError => "InstanceVariableNotFoundError",
        EvaluationError::ImmutableBinding => "ImmutableBindingError",
        EvaluationError::ReadonlyProperty => "ReadonlyMutationError",
        EvaluationError::Class(iris_runtime::ClassError::MetaTransactionConflict { .. }) => {
            "MetaTransactionConflictError"
        }
        // IRIS-V1-META-C081 names this for a denied meta operation, and
        // IRIS-V1-META-V361 requires each denied lane to raise it.
        EvaluationError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
            | iris_runtime::ClassError::ProtectedSuperclass { .. },
        ) => "MetaCapabilityError",
        // IRIS-V1-RUNTIME-C077 names the visibility failure, C014 and D-103
        // name the binding and super failures, and V434 observes a private
        // call being refused from every path but the declaring Class.
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::VisibilityDenied { .. },
        )) => "MethodVisibilityError",
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::MethodBinding { .. },
        )) => "MethodBindingError",
        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
            iris_runtime::DispatchError::NoSuperMethod { .. },
        )) => "NoSuperMethodError",
        EvaluationError::MessageNotFound { .. } => "MessageNotFound",
        _ => return None,
    };
    Some(name.to_owned())
}

#[cfg(test)]
mod tests {
    use iris_parser::parse;
    use iris_runtime::{ArrayRef, Value};

    use super::SourceEvaluator;

    fn source_evaluator(
        source: &str,
    ) -> Result<(SourceEvaluator, iris_syntax::Program), crate::EvaluationError> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "{parsed:#?}");
        Ok((
            SourceEvaluator::new_in_package(super::LOCAL_PACKAGE)?,
            parsed.program,
        ))
    }

    #[test]
    fn class_variable_read_uses_the_declaring_class_cell() -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun read() -> Integer { @@c } }; A.new().read()";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(class)?;
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");
        evaluator
            .runtime
            .declare_class_var(class, selector, Value::Integer(1_u8.into()), true)
            .map_err(crate::EvaluationError::Construction)?;

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        assert_eq!(result, Ok(Value::Integer(1_u8.into())));
        Ok(())
    }

    #[test]
    fn shared_mut_declaration_initializes_and_updates_its_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 0 public fun bump() -> Integer { @@count = 1 } public fun read() -> Integer { @@count } }; let a = A.new(); a.bump(); a.read()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn shared_let_assignment_returns_a_typed_immutable_storage_error()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared let @@count: Integer = 0 public fun bump() -> Integer { @@count = 1 } }; A.new().bump()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::ImmutableClassVariable { .. }
            ))
        ));
        Ok(())
    }

    #[test]
    fn shared_cell_is_read_through_the_declaring_and_subclass_static_lexical_contexts()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 1 public fun read() -> Integer { @@count } }; class B extends A { public fun read_child() -> Integer { @@count } }; [A.new().read(), B.new().read_child()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn module_shared_declaration_publishes_a_module_anchored_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        // C077 defaults a Module Method to private, so the external `M.read()`
        // send needs it declared public.
        let source = "module M { shared let @@version: Integer = 1 public module fun read() -> Integer { @@version } }; M.read()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(result, Ok(Value::Integer(1_u8.into())));
        Ok(())
    }

    #[test]
    fn duplicate_shared_declaration_on_static_ancestry_keeps_active_revision_unchanged()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 1 }; class B extends A { shared mut @@count: Integer = 2 }";
        let (mut evaluator, program) = source_evaluator(source)?;
        let [
            iris_syntax::Declaration::Class(parent),
            iris_syntax::Declaration::Class(child),
        ] = program.declarations.as_slice()
        else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(parent)?;
        let parent_id = evaluator.class_name("A")?;
        let child_id = evaluator
            .runtime
            .registry_mut()
            .define_class(iris_runtime::StaticSpine::new(1), parent_id)
            .map_err(crate::EvaluationError::Class)?;
        evaluator.names.insert(
            child.name.clone(),
            super::Binding::immutable(Value::Class(child_id)),
        );
        evaluator.static_superclasses.insert(child_id, parent_id);
        let before = evaluator
            .runtime
            .registry()
            .active_revision(child_id)
            .map_err(crate::EvaluationError::Class)?;

        // When
        let result = evaluator.class(child);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Class(
                iris_runtime::ClassError::DuplicateClassVariable { .. }
            ))
        ));
        assert_eq!(
            evaluator
                .runtime
                .registry()
                .active_revision(child_id)
                .map_err(crate::EvaluationError::Class)?,
            before
        );
        Ok(())
    }

    #[test]
    fn subclass_method_reads_the_declaring_class_cell() -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun read() -> Integer { @@c } }; class B extends A { public fun child_read() -> Integer { @@c } }; [A.new().read(), B.new().child_read()]";
        let (mut evaluator, program) = source_evaluator(source)?;
        for declaration in &program.declarations {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(crate::EvaluationError::UnsupportedConstruct);
            };
            evaluator.class(class)?;
        }
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");
        evaluator
            .runtime
            .declare_class_var(class, selector, Value::Integer(1_u8.into()), true)
            .map_err(crate::EvaluationError::Construction)?;

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn class_variable_assignment_to_absent_storage_fails_without_creating_a_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun write() -> Integer { @@c = 1 } }; A.new().write()";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(class)?;
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        eprintln!("{result:?}");
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. }
            ))
        ));
        assert!(matches!(
            evaluator.runtime.class_var(class, selector),
            Err(iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. })
        ));
        Ok(())
    }

    #[test]
    fn class_object_raw_ivars_and_class_variables_use_distinct_storage()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@x: Integer = 2 class fun set_raw() -> Integer { @x = 1 } class fun raw() -> Integer { @x } class fun shared() -> Integer { @@x } }; let ignored = A.set_raw(); [A.raw(), A.shared()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(2_u8.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn class_fun_and_instance_method_share_the_declaring_class_variable_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class P { shared mut @@n: Integer = 0; public class fun bump() -> Integer { @@n = @@n + 1 } public fun read() -> Integer { @@n } }; [P.bump(), P.new().read(), P.bump()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
                Value::Integer(2_u8.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn class_fun_rejects_immutable_and_absent_class_variable_assignment()
    -> Result<(), crate::EvaluationError> {
        // Given
        let immutable = "class P { shared let @@n: Integer = 0; public class fun write() -> Integer { @@n = 1 } }; P.write()";
        let absent = "class P { public class fun write() -> Integer { @@n = 1 } }; P.write()";
        let (mut immutable_evaluator, immutable_program) = source_evaluator(immutable)?;
        let (mut absent_evaluator, absent_program) = source_evaluator(absent)?;
        let Some(iris_syntax::Declaration::Class(absent_declaration)) =
            absent_program.declarations.first()
        else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        absent_evaluator.class(absent_declaration)?;
        let absent_class = absent_evaluator
            .class_name("P")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let absent_selector = absent_evaluator.selector("n");

        // When
        let immutable_result = immutable_evaluator.program(&immutable_program);
        let absent_result =
            absent_evaluator.statement(&absent_program.statements[0], &Default::default(), None);

        // Then
        assert!(matches!(
            immutable_result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::ImmutableClassVariable { .. }
            ))
        ));
        assert!(matches!(
            absent_result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. }
            ))
        ));
        assert!(matches!(
            absent_evaluator
                .runtime
                .class_var(absent_class, absent_selector),
            Err(iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. })
        ));
        Ok(())
    }

    #[test]
    fn open_builtin_integer_adds_a_source_method_without_replacing_native_hash()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer { public fun doubled() -> Integer { 2 } }; [Integer(1).doubled(), Integer(1).hash]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Integer(2_u8.into()),
                Value::Integer(17_824_117_788_395_916_856_u64.into()),
            ])))
        );
        Ok(())
    }

    #[test]
    fn open_builtin_with_extends_is_rejected_without_publishing_a_revision()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer extends Object { }";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        let integer = evaluator
            .kernel
            .class(iris_runtime::BuiltinClass::Integer)
            .map_err(crate::EvaluationError::Runtime)?;
        let before = evaluator
            .runtime
            .registry_mut()
            .active_revision(integer)
            .map_err(crate::EvaluationError::Class)?;

        // When
        let result = evaluator.class(class);

        // Then
        assert_eq!(
            result,
            Err(crate::EvaluationError::Class(
                iris_runtime::ClassError::ProtectedSuperclass { class: integer }
            ))
        );
        assert_eq!(
            evaluator
                .runtime
                .registry_mut()
                .active_revision(integer)
                .map_err(crate::EvaluationError::Class)?,
            before
        );
        Ok(())
    }

    #[test]
    fn retained_method_binding_failure_does_not_execute_its_observable_body()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "mut log = []; class A { public fun m() -> Nil { log.append(:entered); raise :body } }; class B extends A { }; class Other { }; let method = Reflection::Class.method(A, :m); Reflection::Class.set_superclass(B, Other); Reflection::Class.invoke(method, B.new(), [])";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::MethodBinding { .. }
                )
            ))
        ));
        assert_eq!(
            evaluator.names.get("log").map(|binding| &binding.value),
            Some(&Value::Array(ArrayRef::new(Vec::new())))
        );
        Ok(())
    }

    #[test]
    fn open_builtin_replaces_compatible_method_and_preserves_singletons_and_float_bits()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer { override public fun hash() -> Integer { 2 } }; [Integer(1).hash, nil.hash, false.hash, true.hash]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        let Ok(Value::Array(values)) = result else {
            unreachable!("expected an Array result")
        };
        assert!(matches!(
            values.elements().as_slice(),
            [
                Value::Integer(replaced),
                Value::Integer(nil_hash),
                Value::Integer(false_hash),
                Value::Integer(true_hash),
            ] if replaced == &2_u8.into()
                && nil_hash == &11_850_167_709_044_604_115_u64.into()
                && false_hash == &17_921_396_551_637_717_540_u64.into()
                && true_hash == &14_186_115_676_603_356_736_u64.into()
        ));
        assert!(matches!(
            evaluator.kernel.send(
                evaluator.runtime.registry(),
                Value::Class(
                    evaluator
                        .kernel
                        .class(iris_runtime::BuiltinClass::Float64)
                        .map_err(crate::EvaluationError::Runtime)?,
                ),
                iris_runtime::NativeSelector::FromBits,
                &[Value::Integer(0x8000_0000_0000_0000_u64.into())],
            ),
            Ok(Value::Float64(value)) if value.to_bits() == 0x8000_0000_0000_0000
        ));
        Ok(())
    }
}
