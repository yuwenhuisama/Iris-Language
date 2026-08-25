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

A typed deferred `mut` reserves that same binding register without marking it
written. An assignment writes it normally. A read while the compiler still
knows the cell is empty is declined as `deferred read before assignment`; no nil
sentinel is emitted, because the reference reports `DefiniteAssignmentError`.

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

Measured against the 736 RUNNABLE source vectors, this compiles 419 of them, up
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

The DENOMINATOR was measured too, and it was wrong at first. 47 corpus vectors
are declared `malformed` and are SUPPOSED to be rejected, so counting them as
gaps measured the backend against programs it is right to refuse. Excluding them
took `rejected source` from 71 to 25 and the total from 783 to 736.

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
`call arity`, `name unbound`, `name assignment unbound`,
`member`, `index receiver`, `hash key name`, `yield`, and the
class forms `class decorator`, `class reopen target`, `class reopen header`,
`class reopen class method`,
`class mixin`, `class constraints`, `class meta deny`,
`class superclass` and `class body`, plus the structural refusals
`rejected source`,
`rejected literal`, `empty body`, `array too long`,
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
  read-only context collections. Bare `raise`, cleanup suppression, getter
  replacement, and writes to get-only context properties remain outside the VM
  surface rather than being approximated.
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
