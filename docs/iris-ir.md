# Iris Execution IR

**Status: AUTHORED DESIGN, NOT A SPECIFICATION CLAUSE.**

Nothing in this document is mandated by the frozen v1 specification.
`IRIS-V1-TRACE-C011` and the chapter 01 scope table place the Rust runtime,
VM, GC and JIT explicitly OUT OF SCOPE for that wave, and
`IRIS-V1-CONFORMANCE-C068` forbids a differential checker from comparing
"private bytecode layout" at all. The specification constrains what a backend
must observably DO; it never constrains how a backend executes.

This document is therefore the single source of truth for the IR itself, in the
same category as `authored-stdlib.md`. It may change freely, provided every
change keeps the backends in agreement — which the differential harness checks.

Having one is a deliberate response to a recorded defect. The design review
notes that the previous C++ implementation had **no single source of truth for
opcodes**, with the information spread across `IrisVirtualCodeNumber.h`,
`IrisVirtualCodeStructures.h`, `IrisInstructorMaker.cpp`, `IrisInterpreter.cpp`
and the `.irc` serializer. That implementation is not a reference for this one:
none of its design is carried over.

Implementation: `crates/iris-vm/`.

## 1. Shape

The IR is **register-based and three-address**, per the design review's section
5.7 and its section 9 decision table, which specify a register-based
bytecode/MIR rather than the old implicit stack/register hybrid.

Every instruction names its operands and its destination explicitly. An
instruction's meaning therefore never depends on execution history, which is
what gives the properties the review asks for:

- lowering to Cranelift IR is direct;
- there is no stack effect to track;
- the verifier is a CFG dataflow fixpoint over definite assignment, with no
  operand-stack depth to simulate per control path;
- a disassembly reads as data flow;
- an operand stack cannot underflow, because there is none;
- it suits later SSA lowering.

`1 + 2` lowers to three instructions and no push or pop:

```text
r0 = LoadInteger "1"
r1 = LoadInteger "2"
r2 = Binary "+" r0 r1        ; result = r2
```

## 2. Registers

A register is a `u16` index into a flat, per-program register file
(`Register` in `compile.rs`). Registers are **virtual and unbounded** at this
stage: allocation to a fixed machine bank belongs to a later pass, and doing it
here would bake a machine constraint into the IR before any backend needs one.

A `Program` carries:

| Field | Meaning |
| --- | --- |
| `instructions` | The top-level frame's instructions, executed in order. |
| `registers` | The size of the top-level register file. |
| `result` | The register holding the program's answer. |
| `functions` | Callable bodies, addressed by index from `Call`. |

A `Function` carries:

| Field | Meaning |
| --- | --- |
| `name` | The declared name, for diagnostics only. |
| `parameters` | How many LEADING registers hold parameters. |
| `is_async` | Whether invocation eagerly records the body's outcome in a fresh Task. |
| `registers` | The size of this frame's register file. |
| `instructions` | The body. |

### 2.1 SSA is a compiler property, not a verifier requirement

The compiler assigns a **fresh register for every value it produces**, so as
emitted, each register is written exactly once. This is verified by test rather
than assumed.

The verifier does **not** enforce that. It requires only that a register is
written before it is read. A hand-built program that writes a register twice
verifies successfully.

The distinction is deliberate and worth stating plainly, because conflating the
two would be a trap for a future optimisation pass: **an optimiser MAY reuse a
register**, and the verifier will accept it. Anything that relies on
single-assignment must establish it independently rather than assuming the
verifier guarantees it.

### 2.2 Binding registers

A local binding is pinned to the register holding its value; there is no
separate slot table. A rebinding **shadows into a fresh register** rather than
overwriting the old one, because an earlier instruction may still name the old
register and overwriting would corrupt that read:

```text
; let a = 1; let a = 2; a
r0 = LoadInteger "1"
r1 = Move r0                 ; a -> r1
r2 = LoadInteger "2"
r3 = Move r2                 ; a -> r3, shadowing r1
                             ; result = r3
```

Name resolution is lexical and innermost-wins, searched in reverse declaration
order.

### 2.3 Frames

A call gets a **fresh register file**. Nothing is shared with the caller: a
callee cannot read a caller's registers, and recursion needs no save or restore
of individual registers, because `M.fib(n-1)` and `M.fib(n-2)` each run in their
own file. Parameters arrive **pre-bound** in registers `0 .. parameters`, so a
body addresses them as ordinary registers with no prologue.

Arguments are passed through a **contiguous register window**, named by
`first`/`count` exactly as `BuildArray` names its elements. The caller copies
each argument into the window; the callee sees them as its leading registers and
never learns where they came from.

Every path out of a frame goes through a single `Return`. A body's last
expression is its value, so the compiler appends a `Return` even when the source
wrote none — which keeps the exit path uniform rather than special-casing
fall-through.

This is also what makes a GC root set enumerable: **the live frames ARE the
roots**. The tree-walking evaluator threads locals through a
`&HashMap<String, Value>` parameter instead, so its caller frames sit on the
Rust stack and cannot be walked — which is exactly why a collection there
refuses inside a method body (§7).

### 2.4 Statement sequencing

Statements execute in order. A non-final statement's value is simply never read
again — its register is abandoned, and nothing has to be discarded. This is one
of the concrete simplifications the register form buys: a stack IR needs an
explicit pop, which is one more instruction and one more thing for a verifier to
balance.

## 3. Instruction set

The register IR uses explicit destinations. `dst` is the destination register.

### 3.1 Constant loads

| Instruction | Effect |
| --- | --- |
| `LoadInteger { dst, digits }` | Arbitrary-precision Integer from canonical decimal text. |
| `LoadFloat64 { dst, bits }` | IEEE-754 binary64, carried as BITS. |
| `LoadFloat32 { dst, bits }` | IEEE-754 binary32, carried as BITS. |
| `LoadText { dst, text }` | String. |
| `LoadBytes { dst, bytes }` | Immutable Bytes, preserving raw `\xNN` octets. |
| `LoadByteArray { dst, bytes }` | A fresh identity-bearing ByteArray. |
| `MakeMutableString { dst, source }` | A fresh MutableString copied from the String in `source`. |
| `LoadBool { dst, value }` | Bool. |
| `LoadNil { dst }` | nil. |
| `LoadIterationDone { dst }` | Loads the unique identity-bearing `Iteration.done` singleton. |
| `LoadClass { dst, class }` | The runtime Class object registered at `class`. |
| `LoadType { dst, class }` | The interned nominal Type object for the runtime Class registered at `class`. |
| `LoadContract { dst, contract }` | The immutable Contract object registered at `contract`. |
| `LoadBuiltinClass { dst, name }` | Resolves a built-in nominal name such as `Integer` to its Class object. |
| `LoadBuiltinType { dst, name }` | Resolves a built-in nominal name to its interned Type object. |
| `BuildType { dst, expression }` | Resolves and normalizes a union or intersection Type expression. |
| `LoadGlobal { dst, name }` | Reads the current package-global cell. |
| `StoreGlobal { dst, name, value }` | Stores `value` in the package-global cell and writes the assigned value to `dst`. |
| `PublishBinding { dst, name, source }` | Publishes a top-level lexical value or cell under `name`, then copies it to `dst`. |
| `LoadBinding { dst, name, shared }` | Reads a published top-level lexical binding, dereferencing mutable storage when `shared` is true. |
| `StoreBinding { dst, name, source }` | Replaces the value in a published mutable top-level cell and copies the assigned value to `dst`. |

Floats are carried as **bits, never as a decimal rendering**. A decimal round
trip can perturb the low bit, and `IRIS-V1-RUNTIME-V066` compares exact IEEE-754
results across backends, so a rendering would hide precisely the divergence that
row exists to detect.

Integers are carried as **canonical decimal text** rather than a machine
integer, because `IRIS-V1-RUNTIME-V052` requires arbitrary precision with no
representation type split.

Generic Class and Method parameters are erased by this backend. A closed
construction such as `Box<Integer>` loads the same Class identity as `Box`, so
construction, class methods, identity checks, instance dispatch, and `is`
continue through the ordinary Class instructions. The closed spelling is only
retained long enough to forbid `open`; an unknown construction such as
`Array<Integer>` still raises `NameError` rather than making built-in Classes
generic.

### 3.2 Data movement

| Instruction | Effect |
| --- | --- |
| `Move { dst, source }` | `dst = source`. |
| `MakeCell { dst, source }` | Allocates hidden shared lexical storage initialized from `source`. |
| `LoadCell { dst, cell }` | Reads the current value in shared lexical storage. |
| `StoreCell { dst, cell, source }` | Replaces shared lexical storage from `source` and writes the assigned value to `dst`. |

Declared methods use the program binding table because their fresh register
frames cannot address top-level registers. Publishing a `mut` preserves the
same cell allocated by `MakeCell`, so method reads observe later writes and
method writes remain visible at top level. A published `let` carries its value
without adding an assignment path. Local bindings are resolved first, preserving
lexical shadowing.

### 3.3 Iteration results

| Instruction | Effect |
| --- | --- |
| `BuildIterationYield { dst, value }` | Builds an immutable, identity-less `Iteration.yield(value)` result; `value` may be nil. |

### 3.4 Sends

| Instruction | Effect |
| --- | --- |
| `Binary { dst, selector, left, right }` | `dst = left <selector> right` |
| `Unary { dst, selector, operand }` | `dst = operand.selector()` |
| `BindMember { dst, receiver, selector }` | Resolves an object member and allocates a BoundMethod without invoking it. |
| `TypeTest { dst, value, target }` | Tests `value` against the current nominal ancestry of the Class in `target`. |
| `Send { dst, receiver, selector, first, count }` | Sends an ordinary selector with arguments in the contiguous register window. |
| `SendSuper { dst, receiver, owner, selector, first, count }` | Resolves `selector` after lexical Class `owner` in the receiver's current MRO, then invokes that exact successor. |
| `SendClass { dst, class, selector, first, count }` | Sends a declared Class method to the registered Class object. |
| `Reflection { dst, namespace, selector, first, count }` | Executes a covered Reflection entry point with arguments in the contiguous register window. |
| `Revision { dst, namespace, selector, first, count }` | Executes a covered Revision or RevisionHistory entry point with arguments in the contiguous register window. |
| `OpenClass { dst, class, callback }` | Invokes an open callback, records the resulting commit, and enqueues its after-commit event. |
| `DefineMethod { dst, receiver, name, function }` | Stages `function` as the public instance Method named by the Symbol in `name` on the Class in `receiver`, and writes nil. |
| `Json { dst, selector, first, count }` | Executes covered `JSON.decode` or `JSON.encode` with arguments in the contiguous register window. |
| `ContractCast { dst, receiver, contract }` | Checks nominal conformance and builds an immutable Contract view. |
| `SendContract { dst, receiver, selector, first, count }` | Sends through the requirement namespace of a Contract view. |

Numeric sends dispatch through `iris_runtime::Kernel` — **the same kernel the
tree-walking evaluator uses**. String `+` is handled by the authored String
surface because its right operand is dynamically converted through `to_string`,
which the numeric kernel does not own. This split is a hard rule: a second
bigint, IEEE-754 or hash implementation is exactly the silent divergence the
differential rows exist to detect, and two copies of one bug would agree with
each other. The IR carries the selector name and the runtime owns its meaning.

`selector` is a `&'static str` naming a NATIVE selector. `NativeSelector::from_source`
must accept it; an unrecognised name is a machine error, not a program error.
Note the runtime spells unary minus `negate` — `-@` is not a native selector,
and emitting it produced a machine defect where the reference answered a
`RangeError`.

Covered binary selectors:

```text
+  -  *  /  **  <<  >>  &  ^  |  ==  !=  <  <=  >  >=  <=>
```

Covered unary selectors: `negate`, `~`, `to_bits`, `hash`. Unary `+` is an
identity and emits no send. Logical `&&` and `||` lower to branches so the right
operand remains lazy and the expression returns an operand value, not a Bool.

Built-in names `Object`, `Nil`, `Bool`, `Integer`, `Float32`, `Float64`, and
`String` are Class values. Their `.type` member yields a nominal Type, whose
covered query surface is `kind()`, `subtype?(other)`, and
`assignable?(source)`. `TypeTest` consults the runtime Class MRO rather than
lowering `is` to equality, so inherited user Classes and built-in scalar
Classes use one nominal rule. Contract targets are declined until the VM can
model the reference's nominal conformance test without approximating it.

`BuildType` preserves the parsed Type-expression tree until runtime Class and
Contract identities are available. It then applies the reference evaluator's
normal form: nested like-kind forms flatten; members sort and deduplicate;
nominal subtype absorption keeps the wider union member or narrower
intersection member; `Never` is the union identity and intersection absorber;
`Object` follows from nominal top-type absorption; and intersecting `NonNil`
removes `Nil`, including from a compact nested union. A union nested inside an
intersection remains one atom rather than distributing. One nominal atom
collapses to the ordinary nominal Type value, so normalization makes
`((String | Nil) & NonNil).type same? String.type` true. `same?` compares
nominal and composed Type values by their canonical structure.

An interpolated String is lowered as an ordered chain of String `+` operations.
Each `${expr}` is parsed as one expression, evaluated once from left to right,
and converted dynamically by the String operation. The final String is only
published after every segment succeeds; a non-String `to_string` result raises
`TypeContractError`, and later segments do not run after any failure. A bare `$`
has no special lowering.

`Send` also carries authored ordinary selectors which are not native kernel
operations. The machine routes Array, Hash, String, Integer, Bool, nil, and
Symbol receivers through the same documented convenience surface as the source runtime; selector absence,
wrong arity, and wrong argument shape remain program errors rather than machine
dispatch defects. In particular, Array `size` remains absent and `length` is
the supported spelling.

`Reflection` is a dedicated boundary because these calls are namespace entry
points rather than ordinary receiver dispatch. The covered surface is
`Reflection::Class.method`/`properties`/`revision`, `Reflection::Module.method`,
`Reflection::Contract.requirement`, and
`Reflection::Object.get_ivar`/`set_ivar`. Class method lookup returns the
runtime Method identity or nil when the slot is absent. Module method lookup
returns the Module-owned Method for the requested declared selector. Contract
requirement lookup returns an indexable Hash whose `:return_type` value is the
declared nominal Type, or nil when that requirement is absent. A reflected
Class property query returns an ordinary Array of stored-property Symbols. A
revision query returns an ordinary Hash with `:number` and `:commit_id`; its
number follows the active revision and therefore advances after a successful
`open()`. These collection values retain normal indexing and iteration.
A reflected
Method answers `selector`, `owner`, `visibility`,
`parameters`, `return_type`, and `source`; `bind(receiver)` produces a retained
BoundMethod whose `call` revalidates and invokes that exact Method. The VM
declines `Method.signature`, `Method.package`, and an unbound `Method.call`
before emitting bytecode because the reference runtime does not currently
implement those sends. `Reflection::Class.invoke` and
`Reflection::Module.invoke` are declined as those exact constructs because the
reference evaluator itself reports them unsupported. Raw ivar names accept the Symbol spelling used by the runtime selector
table, absent reads answer nil, and writes return the stored value. A machine
with Host grants checks `reflection.inspect` and `reflection.mutate` separately
against the target Class; a denied or out-of-scope operation raises the
catchable `ReflectionAccessError`.

`Json` is likewise a subsystem boundary rather than an ordinary receiver send.
`decode` accepts one String and produces nil, Bool, Integer, String, Array, or
Hash values recursively; object names remain String keys, so the resulting Hash
can be indexed and iterated through the ordinary collection surface. The parser
matches the reference runtime's current value set: decimal floats are rejected
as the catchable `JSONSyntaxError`. `encode` accepts that same value set and
emits compact JSON without spaces; unsupported values, including floats and
non-String Hash keys, raise the catchable `SerializationError`. Malformed input
never publishes a partial value.

`Revision` separates after-commit delivery from ordinary sends. The covered
surface is `Revision.subscribe`, `Revision.flush`, `Revision.event_errors`,
`RevisionHistory.events(from_commit, to_commit)`, and
`RevisionHistory.prune(commit)`. `OpenClass` invokes the callback before it
publishes a monotonically numbered audit record, then queues one immutable
`RevisionEvent` per subscriber. `flush` invokes subscribed Closures and answers
`(:delivered, delivered_events, 0, event_errors)`; a callback failure is
recorded without undoing the commit or preventing later delivery. History range
reads fail atomically with `AuditHistoryUnavailableError` when any requested
commit was never retained or has been pruned.

### 3.4 Aggregates

| Instruction | Effect |
| --- | --- |
| `BuildArray { dst, first, count }` | Array from the contiguous range `first .. first+count`. |
| `BuildTuple { dst, first, count }` | Tuple from the contiguous range `first .. first+count`. |
| `BuildRange { dst, start, end, inclusive_end }` | Integer Range preserving its written end boundary and inferred direction. |
| `BuildHash { dst, first, count }` | Hash from contiguous key/value register pairs. |
| `Index { dst, receiver, index }` | Reads an Array or Tuple element, or a Hash entry. |
| `SetIndex { dst, receiver, index, value }` | Mutates the shared Array or Hash body and writes the assigned value to `dst`. |

Elements are lowered into a **contiguous run** of registers so the instruction
names a range instead of carrying an operand list. The compiler emits a `Move`
per element to establish contiguity:

```text
; [1, 2]
r0 = LoadInteger "1"
r1 = LoadInteger "2"
r2 = Move r0
r3 = Move r1
r4 = BuildArray first=r2 count=2
```

### 3.5 Control flow

| Instruction | Effect |
| --- | --- |
| `JumpUnless { condition, target }` | Jumps to `target` when `condition` is FALSEY. |
| `Jump { target }` | Jumps unconditionally. |
| `ArrayNext { dst, array, index, exhausted }` | Loads `array[index]` into `dst`, or jumps to `exhausted` when the Array is exhausted. |
| `RangeNext { dst, range, index, exhausted }` | Loads the indexed Range value into `dst`, or jumps to `exhausted` after the endpoint. |
| `IteratorOpen { dst, iterable }` | Sends `iterator()` once and writes the fresh cursor to `dst`. |
| `IteratorNext { dst, iterator, exhausted }` | Sends `next()`; writes a yield payload to `dst`, or jumps to `exhausted` for `Iteration.done`. |
| `IteratorClose { iterator }` | Sends the idempotent `close()` cleanup message and discards its nil result. |

Only the false branch is conditional. One conditional form plus an unconditional
jump expresses every shape this subset needs, and each extra branch opcode is
another case the verifier must reason about.

A `while` loop is a BACKWARD jump: the condition is evaluated at the top, a
`JumpUnless` leaves the loop, and the body ends in a `Jump` back to the top. The
loop's own value is nil, since `IRIS-V1-CONTROL-C023` gives a normal loop
completion no value and only a `break` with an operand carries one - which this
subset declines.

A `for` loop evaluates its source once, obtains the cursor with `IteratorOpen`,
and drives it through `IteratorNext`. Arrays, Hashes, and Ranges use the same
protocol instructions as user-defined iterables; their machine-owned cursors
preserve fail-fast version checks. `continue` targets the next advance. Natural
exhaustion and `break` join at `IteratorClose`; `return` closes every active
iterator before leaving the frame; and the loop body is protected by an
exception handler whose propagation path closes the iterator before re-raising.
These cleanup paths are explicit CFG predecessors in the verifier. Labels,
value-carrying `break`, and destructuring bindings remain declined rather than
approximated.

A `mut` binding keeps ONE register that assignment updates in place. That is
what carries a value across the back edge: allocating a fresh register per
assignment would leave the loop reading its pre-loop value forever.

A typed deferred `mut` reserves that same binding register PLUS an `assigned`
flag register that `DeclareDeferred` clears, every assignment sets via
`MarkAssigned`, and every read consults via `ReadDeferred`, which fails with
`DefiniteAssignment` when the flag is false. The check is emitted rather than
decided at compile time because it is not a compile-time fact: in
`mut x: Integer; if c { x = 1 }; x` the same code answers `1` when `c` holds
and fails when it does not. Deciding it in the compiler either declined a
program the reference accepts or, when any branch assignment discharged the
deferral, answered a silently wrong `nil` on the path that skipped it.

Truth is decided by the **runtime**, not re-derived here:
`IRIS-V1-CONTROL-C022` makes exactly `false` and `nil` falsey, and a second copy
of that rule would be one more place for the backends to diverge.

An `if` yields a value, so **both arms write the same destination register**,
and a missing `else` writes nil. The destination is therefore written on every
path, which is what keeps the verifier's written-before-read rule satisfied
however the branch goes.

### 3.6 Calls

| Instruction | Effect |
| --- | --- |
| `Call { dst, function, first, count }` | Calls `function` with the window `first .. first+count`. |
| `BareCall { dst, callee, name, first, count }` | Invokes a bound local BoundMethod; the absent numeric names retained by this subset raise the reference `Symbol` MessageNotFound result. |
| `Using { dst, resource, block }` | Invokes the Closure block, closes the resource on normal and raised exits, and merges both outcomes under the resource-cleanup rule. |
| `Return { value }` | Returns `value` from the current frame. |
| `MakeClosure { dst, function, first, count }` | Allocates a Closure whose capture handles and immutable values are copied from the register window. |
| `Await { dst, task }` | Observes an already-completed Task, answering its value or propagating its retained failure. |
| `HostRun { dst, task }` | Observes a completed Task outside async and Closure bodies. |
| `UnobservedFailures { dst }` | Answers the Tasks whose failed outcomes have not been observed. |

`function` is an **index**, not a name. Resolution happens before any
instruction is emitted, so nothing is looked up at run time. That resolution is
the responsibility the design review assigns to a HIR layer; it is done here in
one pass while the covered surface is small enough not to need a separate
representation, and it is the first thing that should move once it grows.

A zero-argument call still allocates a window start inside the file, so `first`
is always a valid register even when `count` is 0.

`super()` retains the lexical Class and selector in the instruction instead of
re-dispatching from the receiver Class. This is required for two-level override
chains: the same most-derived receiver must advance after each currently
executing owner. No successor raises `NoSuperMethod` and does not enter
`method_missing`.

`using(resource) { body }` answers the block value only after `close()` succeeds.
A body failure remains primary if cleanup also fails; a cleanup failure after a
normal body becomes primary. The instruction therefore has the same mandatory
cleanup discipline as iterator close rather than being compiled as two ordinary
calls where the second could be skipped.

For an omitted trailing positional parameter, the caller evaluates its default
expression and copies the result into the same contiguous argument window. Rest,
keyword, keyword-rest, and block channels are declined by their specific
parameter construct until the VM has channel-aware frames.

A Closure body is another function whether its source uses a parameter header
or the header-less block spelling. Its captures occupy the leading registers,
followed by invocation arguments. Immutable captures are value snapshots;
mutable lexical bindings are hidden shared cells, so copying their handles into
an escaped or nested Closure preserves one binding after the defining frame
ends. Reads and assignments use `LoadCell` and `StoreCell`; a nested `let` still
allocates a fresh binding and therefore shadows without replacing the captured
cell. D-421's `return` exits only that Closure frame. Nested Closure bodies are
allocated recursively before their enclosing body is appended, so every
`MakeClosure` carries its final function-table index.

`define_method` lowers its block body directly to a function with `self` in its
leading register. It intentionally does not create a Closure value or copy the
surrounding capture window, because Iris turns this block into an ordinary
Method body rather than a closure over the open callback's locals.

### 3.7 Exceptions

| Instruction | Effect |
| --- | --- |
| `EnterTry { handler, cleanup, exception, context }` | Pushes a handler and names the registers receiving the raised value and its `ExceptionContext`. |
| `CatchMatch { destination, exception, class }` | Writes whether the raised value matches the named Class filter. |
| `LeaveTry` | Removes the handler after normal completion. |
| `Raise { value, cause, offset }` | Creates a fresh propagation context, then transfers to the innermost handler or propagates from the frame. |
| `ReRaise { value, context, offset }` | Appends a `RaiseSite` to `context` and continues that existing propagation. |
| `RaiseNoActiveException` | Raises the catchable `NoActiveExceptionError` when no propagation can be continued. |
| `Propagate { value, context }` | Transfers an existing propagation context through another handler. |

Lowering emits cleanup on the normal and exceptional routes as distinct CFG
blocks, so exactly one route reaches it. A raise from a called frame is returned
as an explicit machine outcome and routed through the caller's handler stack;
Rust unwinding is not involved.

Every explicit `raise` allocates a distinct context identity. Its `value`,
explicit or automatic `cause`, empty `suppressed` and `re_raise_sites`
collections, empty `original_stack`, and initial `raise_location` are observable.
`from nil` suppresses automatic chaining. Context `same?` compares identity, so
two explicit raises of the same value remain distinct. The catch edge marks both
the exception and context registers written from protected-region entry state;
writes later in the protected body do not leak into the handler.

### 3.8 Float reinterpretation

| Instruction | Effect |
| --- | --- |
| `FromBits { dst, width, bits }` | Reinterprets an Integer register's bits at `Bits32` or `Bits64`. |

The width is carried **explicitly** rather than inferred from the value, because
`IRIS-V1-RUNTIME-C113` fixes the accepted range per width and requires a
`RangeError` outside it. Bits are reinterpreted, never converted numerically, so
`C114`'s round trip holds for every interchange pattern **including signaling
NaN**, which must not be quieted in transit.

An out-of-range value raises the runtime's own `Numeric(Range)`. Inventing a
separate error here would make the backends disagree for a reason that is not
semantic.

## 4. Verifier

Verification runs **before** execution.

Structural checks - registers in range, jump targets inside the body, call
indices defined - are a single linear pass. **Definite assignment is a dataflow
fixpoint** over the control-flow graph, keeping the INTERSECTION of what is
written along every path reaching a point.

`EnterTry` contributes an exceptional predecessor to its handler using the
state at protected-region entry, with only the exception register added. A
write later in the protected body therefore cannot become definitely assigned
at catch entry merely because it appears earlier in instruction order.

A linear scan is unsound the moment control flow exists, and this was a real
defect rather than a hypothetical one. A forward jump that SKIPS a write left
the scan believing the register was written, because the scan walked past an
instruction execution never runs; the resulting program read an unwritten
register and answered nil. A backward jump breaks it the other way, since a loop
body is entered before its own writes have happened.

The fixpoint terminates because each entry set only ever shrinks. An
UNREACHABLE instruction is not checked at all: it never executes, so it cannot
read anything.

This closes a P0 recorded against the previous C++ VM, whose opcode loop read
operands through `vector::operator[]` with no verifier or bounds check, so
corrupt bytecode was undefined behaviour. Here a malformed program is a
reportable error.

### 4.1 Invariants

1. Every register an instruction READS is `< registers`.
2. Every register an instruction READS has been WRITTEN by an earlier
   instruction. A PARAMETER counts as written before the first instruction,
   since it arrives pre-bound.
3. Every register an instruction WRITES is `< registers`.
4. A `BuildArray` or `Call` range `first .. first+count` lies wholly within the
   file, with the addition checked for overflow.
5. Every jump target lies within the body.
6. Every `Call` names a function the program defines.
7. `result` is in range and has been written.

Violations are `RegisterOutOfRange`, `ReadBeforeWrite`, `ArrayRangeOutOfRange`,
`JumpOutOfRange` and `UnknownFunction`.

**Every function body is verified**, not only the top level.

The jump check answers a second recorded defect. The design review notes a
`SPR` opcode in the old VM with no `break` after its case, which fell through
into `LOAD_CAST` and overwrote its own result. A checked target makes that class
of defect a verification failure rather than silent corruption.

### 4.2 What verification buys execution

Because verification has already proven every access in range, the machine
indexes its register file directly and repeats no operand check. The safety
argument lives in one pass rather than being scattered across every opcode
handler — which is how the old implementation lost it.

### 4.3 What the verifier does NOT check

Stated explicitly so nothing is assumed of it:

- It does not check TYPES. `Binary "+"` on a String and an Integer verifies
  fine and raises at run time through the kernel, which is correct for a
  dynamically typed language.
- It does not check single assignment (§2.1).
- It does not check reachability or termination.
- It does not check ARITY at the instruction level. A `Call` window may be any
  width the register file allows; arity is enforced when the call is lowered,
  where the declared parameter list is in scope.

## 5. Coverage boundary

The backend is **deliberately partial**, and declines rather than approximates.
A backend that guessed would make a differential row agree for the wrong
reason, which is worse than leaving the row held. `compile` therefore answers a
`CompileError` naming the construct it lacks, and the harness reports fewer
than two RUNNING backends as insufficient rather than as agreement.

Covered: integer, float, string, bool, nil and Symbol literals; built-in
`to_string` on Integer, String, Bool, nil, and Symbol (Float has no such
reference conversion); the binary and
unary selectors listed in §3.3; identity (`same?`, in both its infix and method
spellings); array literals; Hash literals with explicit keys; indexing an Array
or Hash; `let` and `mut` bindings; assignment to a bound name; statement
sequences; `if`/`else` as a value; `while` loops; `return`; ordered catch clauses
with named Class filters, `try`/`catch`/`finally`, explicit `raise`, and
`catch value, context` bindings exposing `value`, `cause`, `suppressed`,
`re_raise_sites`, `original_stack`, and `raise_location`; Closure capture and `.call`;
`Float32.from_bits`/`Float64.from_bits`; `to_bits`; `hash`; native selectors on
arbitrary receivers; authored Array, Hash, and String sends on literal,
aggregate, bound-name, grouped, and call-result receivers; **plain module functions** with positional parameters,
including recursion and mutual calls; and **user-defined classes**: declaration,
construction through `new` with an initializer, raw ivar reads and writes,
`self`, instance dispatch through the receiver's class, `class fun` declarations
and their calls, a named superclass with inherited methods and initializers,
inherited `override` declarations, and declarative instance-method reopens. A
reopen is registered as an ordinary transaction after the origin transaction,
so it advances the active revision rather than being folded into revision one.
The covered class surface also includes literal-initialized instance stored
properties, literal-initialized `shared let`/`shared mut` class variables,
property getter methods on bare member reads, Contract declarations with plain
instance requirements, immutable `for` conformance lists, unqualified `impl`
methods, checked `as Contract` views, qualified `view..member()` dispatch, and
`Class.contracts()` metadata.

Also covered: `for` over an Array, a Hash or a Range with `break` and
`continue`, where a Hash element is a `(key, value)` Tuple and a collection
changed mid-loop raises `ConcurrentModification` on the next advance; declared
Class and Module names as VALUES, so one can be returned, passed and then sent
`new` or a class method; built-in class names as values with `.type`, `.name`,
`.kind()`, `.subtype?` and `.assignable?`, and `value is Class` tests; global
bindings and `$name` reads; index assignment on Arrays and Hashes, which
mutates the shared handle so aliases observe it and raises `IndexError` for a
position outside the Array; member assignment as a `p=` setter SEND; bare
member reads answering a callable `BoundMethod`; string interpolation; literal
`match`; tuples; ranges; default positional arguments; annotated local
bindings; and `catch e, context` binding the propagation context with
`.value`, `.cause`, `.suppressed`, `.re_raise_sites` and `.raise_location`.
The covered subsystem surface includes `JSON.decode` and `JSON.encode` for the
reference runtime's nil, Bool, Integer, String, Array, and String-keyed Hash
value set, including catchable syntax and serialization failures; the full
Iteration protocol, so a `for` obtains an iterator through `iterator()`, calls
`next()` until `Iteration.done` and CLOSES on every exit path, and an iterator
written entirely in Iris with `Iteration.yield`/`Iteration.done` drives a loop;
`Reflection::Class.method`, `properties` and `revision`,
`Reflection::Object.get_ivar`/`set_ivar`, `Reflection::Module.method` and
`Reflection::Contract.requirement`, with an ungranted call raising a catchable
`ReflectionAccessError`; and `Revision.subscribe`/`flush`/`event_errors` with
`RevisionHistory.events`/`prune`, where `Class.open()` publishes a real
revision that `active_revision` reports.
Also covered: reified union and intersection Type values over known nominal
Classes and Contracts, including `.type`, `.kind()`, structural `same?`,
nilability, nominal absorption, `Never`, `NonNil`, compact nested unions, and
use of those forms as binding annotations.

A NAMED runtime failure is an ordinary catchable Iris error: `IndexError`,
`MessageNotFound`, `ConcurrentModificationError`, `IteratorStateError`,
`JSONSyntaxError` and `ReflectionAccessError` all reach a handler as the
specification's name. Uncaught, the same failure surfaces as ITSELF rather than
as a raised Symbol, so a program that never wrote a handler sees what the
reference reports.

Also covered: erased generics, so `Box<Integer>` and `Box<String>` are the SAME
Class and a type argument list carries no runtime identity; `super()` through
several levels of inheritance; `using(resource) { body }`, which closes on every
exit including a raising body; header-less block closures, which is the form
`define_method(:a) { 1 }` passes; top-level bindings that a method body can
read, write and SEND to; Array `append`, `insert`, `delete` and `clear`, which
answer nil rather than the receiver unlike `push`; and **async methods**, where
C012 makes creating the Task and starting its initial run ONE call operation -
the body executes at CALL time, `Host.run` and `await` observe an
already-computed outcome, and a failing task stays in the Diagnostics channel
until observed.

Measured against the 736 RUNNABLE source vectors, this compiles all 736 of them, up
from 51 when the measurement started. The number is reported rather than
estimated because the first estimate of what blocked the backend was WRONG: the
assumed blockers were loops and calls, while the measurement showed a single
dominant one, `class`, at 325 programs. Each subsequent round was aimed the same
way - splitting the decline reasons showed `try` at 38 against `for` at 8, later
that the largest remaining bucket was Contracts rather than anything the coarse
`name` bucket suggested, and later still that `name unbound` was mostly a tail of
FFI fixture names rather than one gap.

Splitting a bucket by the actual NAME in it has repeatedly found something the
category hid. `closure` at 19 was not about closures: every declining one
carried `has_header: false`, so the gap was the bare block form. `call unbound
receiver` at 70 looked like missing subsystems, and 19 of them were an ordinary
accumulator named `log` calling an Array method the backend did not have.

Twice a bucket turned out not to be a coverage gap at all, but a REFUSAL the
backend was describing differently from the reference. An unbound name, and an
unbound ordinary receiver, are a `NameError` the reference raises when the read
or the call RUNS; declining them refused the same program in a way that could
not agree, and held the row. Emitting `RaiseNameError` instead moved 45
programs. The boundary matters: a name the reference resolves to a standard
SERVICE - `Unicode`, `Gate`, `FFI`, and the rest of `SERVICE_RECEIVERS` - is
unimplemented rather than absent, and `Unicode.version()` answers `"17.0.0"`
there, so those still decline rather than answering a confident wrong failure.

The DENOMINATOR was measured too, and it was wrong at first. 47 corpus vectors
are declared `malformed` and are SUPPOSED to be rejected, so counting them as
gaps measured the backend against programs it is right to refuse. Excluding them
took `rejected source` from 71 to 25 and the total from 783 to 736.

That denominator is now REPRODUCIBLE. The probe reads a TSV the corpus is
projected into, and that file used to be written by hand into `/tmp`, so once
it aged out neither the count nor the rules behind it could be recovered -
a reported gap could not be re-measured. `tools/vm-corpus.py` writes it from
the frozen corpus instead, and states the three exclusions: `input.malformed`,
`bucket:documentation`, and `bucket:record-validation`, whose `source_text` is
a conformance record's own JSON rather than Iris. Reconstructing the file
without that last rule inflated `rejected source` from 25 to 44 and the total
to 755, counting 19 JSON records as programs the backend had failed to compile.

Three more buckets were mislabelled refusals of the same kind. An empty method
body answers nil, a deferred `let` answers nil and fails only when READ, and a
selector no declaration mentions is a `MessageNotFound` - that last one was
reported as `UnknownSelector`, a MACHINE DEFECT rather than a program error,
which is the worst version of the mistake because it accuses the compiler.

Two were rules the language does not have. A Contract conformance is NOT
enforced at declaration: the reference runs `class X for C { }` with `C`'s
requirement unimplemented, and a plain method satisfies a requirement without
an `impl` marker, so demanding one refused the ordinary form. A reopen was
required to FOLLOW its target because declarations were collected in source
order, though `open class A { }` ahead of `class A { }` is an ordinary program;
origins are collected first now.

The `to_bool` protocol was a WRONG ANSWER rather than a gap. Truth was decided
structurally, so `if p` took the then-branch for a `p` whose `to_bool` answers
false, and the same defect reached `while`, `&&`, `||` and `!` alike. Truth is
a send now, and the value's own shape only decides when no authored method
answers.

The reverse mistake is just as easy. Lowering a keyword argument made
`M.f(a: 1)` answer `a: 1` by binding the wrapper into a positional slot, where
the reference raises ArgumentError - so a keyword argument to a RESOLVED
function is declined again while the wrapper itself lowers. A module mixing in
another module is declined for the same reason: only a class declaration passes
its modules to the registry, so accepting it would drop the edge silently.

The decline-vs-raise mistake had one more, and it was the second-largest
bucket left. A source the PARSER refuses is a program error the reference
reports when the program RUNS, as `ParseDiagnostic`; declining it refused the
same 25 programs while describing it differently. The same reading closed
`break outside loop` and `name assignment unbound`: `C069` gives every
transfer a target and `C009` makes a bare `name = expr` never create a
binding, so both fail when they run.

A DECLARATION ANNOTATION is not a gap either. A decorator, a `where`
constraint and a `meta deny` list annotate a declaration without changing what
it declares, and the reference runs the program: `class A<T> where T: Object
{ } 1` answers `1`. They are not dropped semantics - each governs a surface the
backend has no support for either, so a program that DEPENDS on one fails on
that surface rather than at the declaration. `open`, type parameters and
`extends` on a Contract are NOT annotations, because each changes the
requirement set, and accepting them would answer a wrong one.

Two silent wrong answers surfaced only from USING the backend on a combined
program, not from any single-feature test. A parameter's DEFAULT was
substituted at the call site, which works when the callee is resolved and
cannot work for a dynamic send - which does not know the signature until
dispatch - so `A.new().f(1)` answered nil for the unfilled parameter; defaults
are filled in the callee now. And a CLASS-level property was stored as one
class variable even on a generic class, where `IRIS-V1-TYPES-C064` puts a
plain one on each closed CONSTRUCTION and only a `shared` one on the unapplied
definition: the bare `C.n` answered a value the language does not have there.
That one is declined again, which is why the count moved down by three when it
was fixed.

Only a `mut` binding may be WRITTEN. `C009` makes `let`, a parameter and a loop
variable immutable, so a write to one names a place the program cannot change -
the reference raises when the write runs rather than refusing the program
statically. A DEFERRED binding is the exception: it is declared without a value
and its first write is what supplies one.

A REDEFINITION needs `override`. `C024` forbids an overload set: one complete
selector maps to at most one method per revision, so a second declaration
replaces the first and must say so. A reopen is a meta operation on a class
that already holds the selector and is checked, while an origin declaration is
a static fact and passes - which is why a duplicate inside ONE body is checked
though the first declaration is not. The refusal is raised at load rather than
at compile time, because the class identity it names exists only once the class
is defined.

A BUILT-IN class constructs like any other: it has no declaration entry, so it
declares no stored property and there is nothing to initialize - requiring an
entry reported `Object.new()` as a class the program never named.

A bare MODULE NAME is a value, whatever the module declares:
`A.remove_module(Mo)` names the module itself. A module composed into a class
has its methods lowered with a receiver, so recognising one only by a
receiverless signature missed exactly the modules a mixin names - and a module
with no methods contributes no signature at all, which is why the declaration
table is consulted rather than the signatures alone. Those tables travel
together as `Declarations`, since deciding what a bare name means needs all
four.

A CLASS answers its INSTANCE methods, with the Class itself as the receiver.
The singleton table holds only `class fun` declarations, so a plain method
reported absent for a selector the class plainly declares. A signature taking a
receiver is also not a module function: the module path copies arguments into
the callee's leading registers with no receiver among them, so `A.m()` for an
instance `m` arrived one argument short.

Bytes that do not DECODE name an encoding failure: a byte string is the right
kind of receiver for `to_string` and its content simply does not decode, so
reporting a type error described the receiver rather than the bytes.

A HASH ITERATOR removes the entry it just YIELDED, once. `C026` advances the
expected version with the removal, so the iterator keeps walking what remains
rather than reporting concurrent modification against itself - before the first
`next` there is no current entry, and a second removal names none either. A
BYTE string renders as text when its bytes are valid UTF-8, which the kernel
installs on String alone.

A BOUNDED subscriber queue coalesces what it DROPPED into one GapEvent. `C051`
bounds the queue at a capacity the subscriber names, so one that falls behind
loses its oldest events rather than growing without limit, and the dropped
range is delivered ahead of what it still holds. `C052` makes that range
inclusive and forbids pretending no change occurred, so a successive drop
extends the existing gap rather than reporting its own.

A CLASS method's `super()` walks the SINGLETON side. The receiver is the Class
itself, and a class method lives in a table separate from the instance one, so
the ancestor is found by walking the declared superclass chain - instance
dispatch refused a Class receiver outright.

`A.method(:f)` and the reflective call are ONE surface, per `C119`: the direct
form answers the same Method value rather than reporting the selector absent.

A MUTABLE string appends IN PLACE with `<<`, so every reference to it sees the
write, while `+` answers a new text and leaves the receiver untouched - both
reached the kernel, which installs neither on this family. An IDENTITY-bearing
value compares by which value it is, so two iterators over one array are
distinct even though they would yield the same elements.

A CLEANUP that raises chains the exception it interrupted. `C067` makes a
`finally` raising while another exception propagates report the interrupted one
as its `cause`, so the propagating context travels with the cleanup body -
without it the new exception reported no cause at all.

A class has a LAST SAY through `method_missing`, reached only once ordinary
dispatch found nothing - so a declared method still wins and the handler is a
fallback rather than an interception. `C099` passes the trailing block as the
separate `block` parameter rather than inside the positional snapshot, which is
what lets a handler tell one from the other.

A BOUND method is a fresh value per binding, so `obj.method` twice names two of
them, each with its own runtime identity - while a METHOD is the definition
itself, interned once per declaration, so two reads of one selector name the
same value.

A `mixin` may name a CLASS rather than a module, and the class then contributes
its methods the same way - a module is registered for it on first use, so the
MRO carries one identity per named class. The composing class still wins for a
selector it declares itself.

A HASH groups keys by the CURRENT `==`. `C028` dispatches each key's own `==`
to find its slot, which only the machine can do - leaving the slot unresolved
kept two keys that compare equal as separate entries. `C031`'s rehash rebuilds
against each key's current hash and aborts on a conflict rather than publishing
a partial table. A RANGE index answers a slice that is its own array rather
than a view, so writing through either leaves the other unchanged.

A reflective IVAR names its own failures: a name is a Symbol spelling `@x`, so
a Text is not a name at all, and a value with no instance state cannot hold
one. Reporting both as a generic type failure lost the distinction the language
draws between them.

A HASH KEY goes through the value's OWN `hash`. `C087` fixes a
specification-stable hash per family, but an object supplies its own - a class
defining `hash` is a legitimate key even though no family hash covers it, and
consulting only the stable table refused those keys while spelling the refusal
`StableHash(..)` where the language says `InvalidKeyError`. An identity-bearing
value hashes by WHICH value it is, which stays stable as it advances, and a NaN
names its own reason rather than the generic key failure.

An ANNOTATED binding or parameter is a guarded boundary. `C004` makes
`let s: String = 1` a type failure raised when the binding runs, so the
annotation is a runtime guarantee rather than discarded metadata - treating it
as static metadata answered the value instead of the failure the language
states. `Dynamic<T>` admits what T admits, since the wrapper defers the check
rather than removing it.

A call must supply a count the signature can BIND. `C023` binds each parameter
from the arguments, so a count with no binding is an ArgumentError rather than
a nil quietly filled in - `a.m(1, 2)` for `fun m(x)` answered `1`. Only a
purely positional signature with no defaults fixes a count, since a default, a
`*rest` or a block parameter accepts a range. A COMPOSED module method is
lowered with a receiver it did not write, so the count is compared against what
the source declared rather than against the frame's parameters.

`same?` asks whether two references name ONE value. `C029` accepts only
identity-BEARING operands, so a Text, a Symbol, a Tuple or a numeric raises
rather than being compared by content - the question has no answer for a value
with no identity of its own, and answering by content made `:t same? :t` true
where the language refuses the question entirely.

A BUILT-IN class answers its own CONSTANTS: `Float64.nan` and
`Float32.infinity` are read as bare members rather than called, so they are
consulted before a class's declared variables. `Integer(x)` and `Float64(x)`
are numeric CONVERSIONS rather than constructors - an Integer stays itself and
widens to Float64, while a Text spelling is not a conversion the language
defines and stays a MessageNotFound the caller can catch.

Every value family answers the UNIVERSAL selectors. `C087` fixes a
specification-stable hash per family and `C091` gives each its own equality,
but the kernel installs those selectors only on its own classes - so a Symbol,
a Range, a Tuple, a byte string and an iteration signal all answered
MessageNotFound for messages the language plainly defines. A family with no
stable hash raises a KEY failure rather than reporting the method absent, since
the selector exists on every value. `C092` gives every value a `<=>`: a pair
with no order answers nil rather than refusing, and `<` derives from `<=>`
rather than being a method of its own. An authored operator reaches its class
at all, because an operator arrives through the binary path rather than
through `Send`.

COMPILING a program and AGREEING with the reference are different
measurements, and the second is the load-bearing one. All 736 runnable corpus
vectors compile; `measure_corpus_agreement` in
`crates/iris-eval/src/whole_program_tests.rs` runs each one on both backends
and reports how many answer alike. At the time of writing that is 600 agreed,
124 disagreed, 12 held - so a quarter of the vectors the machine ACCEPTS still
answer something the language does not say. Coverage was never a
correctness claim, and quoting it as one overstated the machine.

A run that never terminates FAILS rather than hanging: each instruction charges
a step against a budget the reference also uses, so a program that exhausts it
answers alike on both. Without that bound a caller cannot tell a slow run from
a stuck one.

The machine has a user ENTRY POINT: `iris --vm <file>` runs a script on it,
and a construct it does not cover is reported as refused rather than rerouted
to the reference - falling back silently would report success for a program the
machine never ran. `print` is a built-in bare call rendering through
`to_string`, so a class's own definition is honoured.

Correctness is measured by TWO suites. The conformance corpus is a microscope:
its vectors have a median length near a hundred characters, each isolating one
rule, which leaves defects that appear only when features INTERACT invisible -
two such reached a fully covered backend, a property and its `@name` ivar
addressing different slots and a property having no setter at all. The
whole-program tests in `crates/iris-eval/src/whole_program_tests.rs` therefore
compare programs written the way a user writes them, value-for-value against
the reference.

A stored PROPERTY and its `@name` ivar are ONE slot. The property declares its
slot under the bare name while the source writes `@n` for it, so the sigil is
stripped - otherwise `@n` reads a slot the property never filled and answers
nil where the value is. The property is written through its setter selector
too: `a.n = 5` is a send of `n=` to the object, landing in that same slot. An
ivar the class never declared keeps its written spelling, which is what leaves
`@z = 5` working in a class with no such property.

A PRIVATE method answers only its declaring class. A send carries the class
whose body wrote it, which is the authority that decides the call - without one
the send is external and the method is refused. A module composed with
`private` access is that authority too, which a class owner cannot express, so
the module travels with the send as well. `initialize` is the exception:
construction calls it on the object's behalf rather than from a caller's frame,
so a class declaring it without `public` stays constructible. A qualified
`impl fun C::m()` governs nothing here either, since the contract's view
consults it directly.

A collection frees what is UNREACHABLE. The live frames are the root set, and a
frame's register file lives on the Rust stack where a collector cannot walk it,
so each frame publishes the registers a NAME claims before a call that may
collect - a temporary the source never bound is already unreachable, and
rooting the whole file would free nothing. An object hashes by IDENTITY, which
survives the relocation a compaction performs: that is what keeps a retained
object's hash stable across a collection while two distinct objects differ.

A module method mixed into a class binds `self` to the COMPOSING object, so it
reaches that object's own methods and an argument lands in its own parameter
rather than being displaced by the receiver dispatch prepends. A module never
composed keeps the receiverless form, where `M.f()` passes only its arguments.
A `private` mixin marker travels into the composition EDGE, which is what the
runtime consults to grant the module reach into the class's private methods.
Method VISIBILITY itself stays unmodelled: a private method's refusal depends
on the caller's lexical owner, which this machine does not track per frame, so
publishing the marker without that context refused programs that run.

A published spine is not REWOUND: `Reflection::Class.reactivate` names a
revision that is no longer active, so it is refused rather than performed, and
the target is resolved first so the refusal cannot be mistaken for an unknown
class. `C050` names shutdown as the condition under which a flush reports
INCOMPLETE with the accepted-but-undelivered count. `C055` makes a configured
audit SINK a separate persistence layer that survives a prune - `recover`
answers what it holds independently of retained history, and with no sink there
is no zero-loss guarantee to offer.

`C064` puts a plain `class property` on each closed CONSTRUCTION rather than on
the unapplied definition, so `Cache<String>.value` and `Cache<Integer>.value`
hold different values while the bare `Cache.value` reaches no slot at all. The
runtime keys class state by `(ClassId, Selector)` and a generic class has one
ClassId, so the construction is folded into the SELECTOR instead - one slot per
construction written in the source, without a class per construction. A
`shared` class property stays on the definition, which is what the bare name
reaches.

`Reflection::Class.define_method(K, :m) { .. }` names its TARGET as the first
argument and publishes exactly as `self.define_method(:m) { .. }` does, so the
reflective form is rewritten to the direct one rather than growing a second
path. A dynamic method's parameters come from the block it was defined with, so
a call supplying a different count has no binding for them and answers
ArgumentError - which is catchable, like any other Iris error.

A class body's ordinary STATEMENTS run with `self` bound to the class, at the
declaration's own source position - that is what lets
`class A { if true { self.define_method(:x) { .. } } }` publish a method, and
why a name bound only after the declaration is unbound inside it. A contract
may extend a GENERIC parent, which names the same contract as a bare one since
the backend interns one per definition; `Iterable` and `Iterator` are the
KERNEL's rather than program declarations, so a child extending one inherits
nothing this backend records and the declaration still runs.

A QUALIFIED `impl fun C::m()` belongs to that contract's VIEW rather than to
the class: `(a as C)..m()` answers it while `a.m()` answers the class's own
method, so it is recorded per contract instead of published. It supplies the
member itself, which is why the view answers it even when the contract declares
no matching requirement. Each is matched to its own body by BODY POSITION -
matching by selector alone made `impl fun C::m()` and `impl fun D::m()` both
resolve to the first.

A subclass INHERITS its superclass's conformances: an `impl` marker on
`class A extends B` names a requirement the ancestry declares even when `A`'s
own header does not, and `A` is viewable through `B`'s contract, so the
ancestry is walked to its root rather than only the class's own list. A
CONTRACT is itself a value - `C.hash()` sends to the contract rather than
resolving a class method - and it is interned once per definition, which is why
two reads hash alike.

A subclass INHERITS its superclass's conformances: an `impl` marker on
`class A extends B` names a requirement the ancestry declares even when `A`'s
own header does not, and `A` is viewable through `B`'s contract, so the
ancestry is walked to its root rather than only the class's own list. A
CONTRACT is itself a value - `C.hash()` sends to the contract rather than
resolving a class method - and it is interned once per definition, which is why
two reads hash alike.

A class reopen may also DECLARE a conformance, which is observable through
`A.contracts` and so joins the class's own list. A BUILT-IN class is the
kernel's and has no entry to join, so its conformance is recorded on the reopen
itself - that is what makes `1 as N` a legitimate view. A contract view is the
value seen THROUGH a contract rather than a different value, so an operator
applies to the value it wraps.

A class reopen may COMPOSE a module: its mixin edge joins the declaration's own
list, so the runtime composes it exactly as a declared one, while a superclass
or a conformance would change what the class IS and stays declined. A member
reached that way may not CONTRADICT a declared contract requirement - `D-173`
puts the contract-visible signature in the static spine, so a parameter Type
differing from the requirement is an incompatible replacement rather than a
satisfying one. An unannotated position states nothing and is left alone. The
check runs once every declaration is collected, because the module supplying
the member may be declared after the class that mixes it in.

A module REOPEN adds to the module it names rather than declaring a new one,
and the LAST definition wins - searching forwards answered from the body the
reopen replaced. A reopen whose target is not declared adds to nothing and the
program still runs, so its methods are dropped rather than the program being
refused. A GENERIC module mixin names the same module whatever its argument:
`mixin Helpers<_>` composes `Helpers` exactly as `mixin Helpers<String>` does,
since the backend specialises a module per argument no more than the reference
publishes one. A `where Self: T` constraint annotates the module the same way,
while a PRIVATE-access class mixin grants reach into the class's private
methods - a change of meaning, so it stays declined.

`from S import K` binds the module's CONSTANT under the imported name, at the
import's own source position. Only a constant is bound - a module's methods are
reached as `S.f()` rather than by name - and an imported name does not disturb
the importing module's own lexical scope. A spec naming nothing the module
declares binds no name and the program still runs, so it is a no-op rather than
a refusal. A left side that names no assignable place raises when the
assignment RUNS, which is where the reference refuses it.

A module's `shared class property` is READ as a member: `M.first` answers it,
unlike a `const`, which is visible only lexically inside the module's own
methods. It is module state with no receiver, so it is synthesized into a
receiverless reader and resolved by index rather than dispatched - there is no
module receiver value to send to.

A declaration naming a target that does not EXIST raises when the program
runs. A reopen of an undeclared class and a contract inheriting an undeclared
parent are program errors the reference raises at RUN time, exactly as a parse
rejection is - declining made both backends refuse the same program while
describing it differently, which holds the row rather than agreeing.

A loop binding may DESTRUCTURE each item. `C045` binds `for [a, b] in source`
from each yielded Array and raises `PatternMatchError` when the item is not an
Array of exactly that arity - the arity travels into the binding rather than
the element being read with a plain index, since an index would answer nil for
a missing position instead of failing. The raise goes through the handler
dispatch so an enclosing `try` catches it. A NESTED sub-pattern decides more
than an arity check can express, so it stays declined.

A contract BOUND is decided at the construction it governs. `C067` checks a
`where T: SomeContract` bound at MATERIALIZATION rather than where the class is
declared, so the declaration alone runs and `Box<String>.new()` is the failure -
String declares no such contract. An argument the backend cannot resolve
decides nothing and passes, which keeps the check about catching a definite
violation.

A declared RETURN Type is GUARDED before the value reaches the caller. `C004`
guards that boundary whether the body fell off its end or returned explicitly,
so a method annotated `-> Nil` cannot answer a Symbol - without the check the
backend RAN a program the reference refuses, which is a wrong answer rather
than a missing feature. The guard raises, so a caller catches it like any other
error. An annotation the backend cannot decide admits every value, which keeps
the check about catching a definite mismatch rather than narrowing the accepted
surface.

A contract INHERITS its parents' requirements. `open` is an annotation - it
governs whether the contract may be reopened, a separate surface, and the
requirement set is the same either way - but `extends` is not: a child carries
every named parent's requirements alongside its own, so a class implementing
the child must satisfy what the parent required. A parent must already exist
for that, which is why an unbound one is declined rather than contributing
nothing silently.

A DECLARED contract cannot be dropped. `C119` makes the direct send and the
reflective call ONE implementation, so `A.remove_contract(C)` and
`Reflection::Class.remove_contract(A, C)` refuse identically, and the refusal
leaves both `contracts` and the active revision untouched. Removing a contract
the class never declared changes no static-spine fact, so that is a no-op
rather than a refusal. `C060` makes a correctly synchronized observer SEE a
concurrent write: the writer thread is joined before the read, so the observer
reports the replacement rather than the text the value was built with.

A class REOPEN takes effect where it was WRITTEN, not at load. A call made
before `open class P { override fun m() }` still answers the original body, so
the reopen's transaction is driven from an `ApplyReopen` at that source
position - publishing every reopen up front made the earlier call answer from
the replacement. It follows that which body runs is the REGISTRY's answer at
that moment rather than a static "last definition wins": a class method is
reached the same way, including as an operator, so `P + P` finds the `class fun
+` that `P.+(P)` already found.

A module's own function is reachable BARE from its siblings - inside `module
M`, `natural()` names `M.natural`. It has no receiver, so it resolves by index
rather than as a send to self, and a local binding of that name still wins.

The NATIVE boundary is the one subsystem that leaves the process. `C018` lets
native code raise only through an ABI operation that creates an
ExceptionContext, so the backend calls the real C ABI and converts its status
plus handle into the ordinary Iris exception a `catch` observes - `C017` makes
a status alone insufficient, so the raised value is read back THROUGH the
handle rather than recomputed, and a boundary that returned no usable context
cannot still produce a correct-looking exception. `C020` is what makes that a
conversion rather than a long jump across the native frame. `C027` validates a
payload descriptor before the runtime owns storage, and `C030` makes a second
close a no-op: the release counter lives behind the ABI, which is what makes
the idempotence observable rather than asserted.

A module's type parameters annotate it the way a class's do, and a CLOSED
generic mixin names the same module - `mixin Helpers<String>` composes
`Helpers`, since neither backend specialises a module per argument. A `super()`
with no owning class has no ancestor to reach, but the body carrying it may
never be invoked, so it is raised when the call runs rather than refused.

Telling an ANNOTATION from a change of meaning kept recurring. Type parameters
and a `where` constraint on a REOPEN restate the declaration's own header, and
a generic contract declares no more requirements than a plain one, so both are
accepted. A superclass, a conformance or a mixin on a reopen would change what
the class IS, and `open` or `extends` on a contract would change its
requirement set, so those stay declined. The sharpest case is a CONTRACT bound:
the reference checks `class Box<T> where T: Comparable<T> {}` when the class is
constructed, so accepting `Box<String>.new()` would answer an object where the
language answers a TypeContractError - it is declined until that check exists,
because a wrong answer is worse than a hold.

A class body's `let` or `mut` declares no instance variable at all: the
reference answers nil for `@done` after `mut done = false`, so the binding is
accepted and ignored rather than refused.

Reflection `invoke` calls a Method the program already SELECTED, so the
dispatch that found it is not repeated, and `set_superclass` is refused by the
target's own meta policy - a built-in class protects its superclass outright.
An Object compares by IDENTITY unless its class defines `==`, and `!=` is that
negated; neither reaches the kernel, because an Object dispatches through its
own class. `fetch` differs from indexing exactly in refusing an absent key,
which is what makes it an assertion that the key is present.

The regex refusals were the decline-vs-raise mistake once more: an unsupported
construct NAMES itself - a backreference is distinguishable from a lookbehind -
and the reference reports that code when the program runs. An INTERPOLATING
pattern splices a value the compiler cannot know, so it is assembled and
validated at run time, with each spliced value ESCAPED: `/${x}/` where `x`
holds `a+b` matches those three characters rather than reinterpreting them as
syntax.

`as?` is a CHECKED cast - the type test with a selection on top, answering nil
where `as` would fail - and a bare-NAME hash key is an ordinary expression
rather than a shorthand for a symbol, so `%{ a: 1 }` keys the hash by what `a`
HOLDS. That is what lets an exception context be used as a key, and an unbound
name there is the ordinary NameError.

The parameter CHANNELS came next, and they had to be bound in the callee: a
dynamic send does not know the signature until dispatch, so the categories
cannot be resolved at the call site. `IRIS-V1-CONTROL-C023` fills positionals
in order, gives `*rest` the remaining positionals as a fresh Array, binds a
`key` parameter by NAME, and collects the unmatched keywords into `**kwargs`;
`D-357` makes a duplicate keyword an ArgumentError rather than last-one-wins.

Using it found two wrong answers no single-feature test had. The argument
window was sized by the SIGNATURE, so a wider frame's unset registers were read
as arguments and `*rest` collected `[nil, nil]` where it should have collected
nothing - the window spans what the caller actually passed now. And a default
was skipped whenever a keyword or block argument padded the argument count past
its positional slot, because the check counted ARGUMENTS rather than asking
whether the slot was filled; defaults are written after binding now, to a slot
still holding nil.

A loop answers the operand its `break` carried, in statement or expression
position - `C023` gives a normal completion no value of its own, so exhausting
the condition answers nil and only a `break` carries something out. A LABELLED
break unwinds to the loop that name belongs to, which is the only way an inner
loop can stop an outer one. An indexed compound assignment evaluates its
receiver and index ONCE and reuses them for both the read and the write, since
lowering `a[i] += v` as `a[i] = a[i] + v` would call a receiver expression
twice. A match GUARD is tested after its pattern, and a false one falls through
to the next arm rather than failing the match.

Two composition subsystems came after those. A module was discovered only from
the names of its FUNCTIONS, which misses one that declares no method of its
own: `module B mixin A { }` exists solely to compose. Modules are declared
explicitly now and defined in DEPENDENCY order, so a composing module names an
identity that already exists - and a forward reference is declined rather than
reordered into working, because the reference refuses one and accepting it
would answer a value the language does not have.

A reopen of a BUILT-IN class has no entry in the compiled class table to attach
to, since the kernel creates those classes. It is recorded by name and
published onto the kernel's own class at load, and the ordinary send path
consults that class before calling a selector absent - otherwise the native
surface answers first and the added method is never reachable. `<` and `>` are
DERIVED from `<=>` rather than being separate methods, so a redefined `<=>` has
to reach them; without that they kept answering from the native comparison the
reopen had replaced, which is a wrong answer rather than a gap.

Three SERVICE subsystems followed, each a surface the backend had no support
for at all rather than a construct it lowered badly.

`IrisValue` validates a stream's HEADER before its payload: `C016` checks magic
and format version before decoding anything that depends on them, and `C017`
forbids allocating from a DECLARED length before that length is validated, so a
limit breach is refused before the payload is read. `C020` routes a nominal
value through the class's own factory, but only for a class that DECLARES
`Serializable` - a class carrying a `deserialize` factory without the
conformance is refused, which is what stops a crafted stream instantiating an
arbitrary class.

The FFI boundary refuses before it crosses. `C045` forbids invoking an unbound
symbol and `C046` denies any signature-less escape hatch, so a `call` to an
unbound name never reaches native code; `C047` lists what a signature must
declare, with pointer nullability, ownership, text encoding and buffer length
required only WHERE NEEDED - a scalar parameter owes none of them; and `C049`
supports the stable C ABI only. `C043` makes each open identity-bearing, so two
opens of one path are two objects and comparing them is false.

Decoding is STRICT by default and an encoding must be named explicitly. `C022`
makes a lossy result appear only because the caller asked for it by name, and
`C025` refuses an implicit selection: a host default, an OS locale or a code
page names no Encoding at all, and so does omitting the argument.

Building them found one more defect, again only from USE. `authored_send`
answers the BUILT-IN surface, so an object's own `serialize` was not reachable
through it and `IrisValue.encode` answered SerializationError for a class that
plainly conformed. Authored instance and class methods are dispatched directly
now.

Four SUBSYSTEMS were then built rather than buckets closed. A stored-property
initializer is an ordinary expression evaluated at construction with `self`
bound, so it is lowered as a frame; only a literal could be stored before. A
superclass initializes first, which is what lets a subclass read a fully built
base. A module body's ordinary statements run at the declaration's SOURCE
POSITION rather than in a separate phase, so the top level walks `entries`
instead of `statements` - lowering them ahead of every statement answered
NameError for the ordinary case. A Regex literal canonicalizes its flags into
`imsx` order at compile time and is validated by building it, so an
unsupported construct is named rather than guessed; the value carries the
canonical text pair rather than an engine. And an `await` on a pending Gate
SUSPENDS its frame, carrying the register file and instruction pointer out on
a control signal that no `catch` may see; completing the Gate readies the
parked frames, and observing the Task runs them, in the order they suspended.

The async work found the deepest defect of the round, and only through USE. A
module constant was bound before the function's parameters, to make a
same-named parameter shadow it - but a call copies arguments into the LEADING
registers, so allocating anything first displaced every parameter and the
verifier proved the argument registers unwritten. That is a machine defect,
reachable by any module function taking both a constant and a parameter, and
no single-feature test had both. Parameters are allocated first now, and a
constant whose name a parameter already claims is skipped, which preserves the
shadowing in both directions.

`assignment` was a coverage gap rather than a mislabelled refusal, and closing
it moved 6 programs. A compound assignment reads its target once and sends the
ordinary operator (`C036`), while `&&=` and `||=` truth-test the target and
evaluate the right side only on the WRITING path (`C037`) - so the logical
forms cannot be desugared to `x = x || v`, which runs the right side either
way. The target is also a shared CELL rather than a plain register, since every
`mut` binding is one, so both the read and the write go through the cell.

What the number does NOT measure is how much a compiled program can do. Adding
the authored stdlib moved it by zero while making ordinary Iris - `[1, 2,
3].map({ |x|; x * 2 }).join("-")` - run for the first time.

Declined, each by name: `declaration import`,
`declaration export`, and `declaration type alias`,
`module` (open, mixin, generic or decorated), `module body` (a non-method
statement), `qualified contract implementation`, `method
 decorator`, `method module`, `abstract
method`, `parameter` (rest, keyword or block),
`statement <form>`, which names the form that stopped it - unsupported `for`,
`match`, `binding`, `global`, `shared`, `deferred`, `stored property`,
`break`, `continue`, `method` - and likewise `call <shape>` for a call:
`bare name`, `closure`, `callee`, and `unbound receiver`. Naming the form rather
than the category is what makes the measurement in §6 actionable: `statement`
alone said where the backend stopped, not what stopped it, and the split showed
`try` at 38 against `for` at 8. Also `assignment target` (anything but a bound
name), `nested closure`, non-name `try catch filter`,
`call arity`, `name assignment unbound`, `break outside loop`,
`member`, `yield`, and the
class forms `class reopen target`, `class reopen header`,
`class reopen class method`,
`class mixin`, `class superclass` and `class body`, plus the structural
refusals
`rejected literal`, `array too long`,
`call too wide`, `from_bits arity`, `branch patch` and `register exhaustion`.
Reified `typeof`, callable Types, composed generic members, and unresolved Type
names retain the exact `expression reified type` decline.

A program of only declarations, or only bindings, is NOT declined. It answers
no value, and the reference raises `UnsupportedConstruct` when it runs, so the
backend raises the same thing at run time. Refusing it at compile time made
both backends refuse the same program while describing the refusal
differently, which holds a differential row rather than agreeing on it.

This list is pinned by test, so widening coverage without updating this document
fails the build.

## 6. Not yet defined

Named so the gaps are not mistaken for decisions:

- **Method forms.** Qualified Contract implementations, property
  setters, decorators, and non-positional/default parameters retain
  runtime or type semantics the bytecode backend does not yet model.
- **General iteration.** Array iteration with name bindings, unlabelled
  `break`, and unlabelled `continue` is covered. Hash, Range, String, and custom
  Iterator protocols require protocol-close behavior; labelled loops,
  value-carrying breaks, and destructuring patterns retain their precise
  statement or pattern declines.
- **Nested Closures.** A Closure may capture from its immediate defining frame;
  recursively compiling Closure bodies needs a stable function-index allocator
  before nested Closure literals can be admitted without misaddressing code.
- **Exception context mutation and cleanup metadata.** Catch bindings preserve
  explicit propagation identity, causes, initial source locations, and the
  read-only context collections. Bare `raise` appends ordered continuation
  sites. Cleanup suppression, getter replacement, and writes to get-only
  context properties remain outside the VM surface rather than being
  approximated.
- **Collection.** The backend now owns a `Runtime`, so it allocates real
  objects, but the collector still runs only in the tree-walking evaluator,
  which registers its own frames (§7). The frames here are the right root-set
  shape for when that changes; nothing in this backend is walked yet.
- **Runtime subsystems.** Reflection method lookup and object raw-ivar access,
  the Iteration protocol, and the default JSON value surface are covered. The
  remaining `Reflection`, `Host`, `FFI`, and extended JSON option surfaces are
  SUBSYSTEMS rather than lowering gaps. Stubbing
  them would answer a differential row with a fabricated value, which is the
  one outcome worse than a held row.
- **Async suspension.** Async bodies run eagerly and completed Task outcomes,
  `await`, `Host.run`, and unobserved failures are covered. A body that reaches
  an incomplete Awaitable still requires continuation and scheduler state and
  remains outside the machine rather than being approximated.

**Bare-name Hash keys.** `%{ a: 1 }` is parsed as a Hash whose key is a NAME
expression, and the reference evaluates it as an ordinary variable: with `a = 9`
bound the key is `9`, and unbound it is a `NameError`. Reading it as the Symbol
`:a` would be a silent disagreement, so it declines as `hash key name` while
explicit Symbol keys stay covered.
- **Serialisation.** There is no on-disk format. Programs are compiled and
  executed in memory. The review records a P0 against the old `.irc` reader for
  trusting file contents — no magic or version check, no field-count or string
  length limits, no remaining-size validation — so if a format is ever added,
  those checks are requirements from the start, not additions.
- **Disassembler.** The review lists readable disassembly as a benefit of the
  register form; the `Debug` rendering stands in for now.

## 7. Divergences from the design review

Recorded rather than silently taken.

**GC roots and frames.** The tree-walking evaluator threads locals through a
`&HashMap<String, Value>` parameter, so a caller's bindings would live only on
the Rust stack. A collection could not see them and would have freed objects a
caller still held, which is why one refused to run inside a Method body at all.

Each active block now registers its locals, so the live frames are the root set
and a collection may run mid-call: a collection triggered in a CALLEE collects
that callee's garbage while leaving the caller's objects alive. The registered
frame is refreshed as the body binds, because registering only the entry
snapshot let a collection free an object a local had just bound.

**Moving GC.** The review recommends a non-moving mark-sweep collector for the
first version and lists moving GC among the things the first JIT explicitly
does not do. The implemented collector **relocates** surviving objects.

This is an accepted, owner-confirmed divergence. It is safe here because the
identity hash is a stored field assigned at allocation, never derived from an
address, so `D-111`'s requirement that a hash survive movement by GC holds by
construction — and `IRIS-V1-RUNTIME-V079` observes exactly that, pinning both
the freed count and the surviving hash. The constraint it does impose is on
future work: a JIT that caches an object address must cooperate with
relocation, so the point at which compaction may run has to stay explicit.
