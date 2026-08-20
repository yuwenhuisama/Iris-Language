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
- the verifier is a single linear pass, because no operand-stack depth has to
  be simulated per control path;
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

Eleven instructions. `dst` is the destination register.

### 3.1 Constant loads

| Instruction | Effect |
| --- | --- |
| `LoadInteger { dst, digits }` | Arbitrary-precision Integer from canonical decimal text. |
| `LoadFloat64 { dst, bits }` | IEEE-754 binary64, carried as BITS. |
| `LoadFloat32 { dst, bits }` | IEEE-754 binary32, carried as BITS. |
| `LoadText { dst, text }` | String. |
| `LoadBool { dst, value }` | Bool. |
| `LoadNil { dst }` | nil. |

Floats are carried as **bits, never as a decimal rendering**. A decimal round
trip can perturb the low bit, and `IRIS-V1-RUNTIME-V066` compares exact IEEE-754
results across backends, so a rendering would hide precisely the divergence that
row exists to detect.

Integers are carried as **canonical decimal text** rather than a machine
integer, because `IRIS-V1-RUNTIME-V052` requires arbitrary precision with no
representation type split.

### 3.2 Data movement

| Instruction | Effect |
| --- | --- |
| `Move { dst, source }` | `dst = source`. |

### 3.3 Sends

| Instruction | Effect |
| --- | --- |
| `Binary { dst, selector, left, right }` | `dst = left <selector> right` |
| `Unary { dst, selector, operand }` | `dst = operand.selector()` |

Both dispatch through `iris_runtime::Kernel` — **the same kernel the
tree-walking evaluator uses**. This is a hard rule, not a convenience: a second
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

Covered unary selectors: `negate`, `to_bits`, `hash`.

### 3.4 Aggregates

| Instruction | Effect |
| --- | --- |
| `BuildArray { dst, first, count }` | Array from the contiguous range `first .. first+count`. |

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

Only the false branch is conditional. One conditional form plus an unconditional
jump expresses every shape this subset needs, and each extra branch opcode is
another case the verifier must reason about.

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
| `Return { value }` | Returns `value` from the current frame. |

`function` is an **index**, not a name. Resolution happens before any
instruction is emitted, so nothing is looked up at run time. That resolution is
the responsibility the design review assigns to a HIR layer; it is done here in
one pass while the covered surface is small enough not to need a separate
representation, and it is the first thing that should move once it grows.

A zero-argument call still allocates a window start inside the file, so `first`
is always a valid register even when `count` is 0.

### 3.7 Float reinterpretation

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

Verification runs **before** execution and is a single linear pass.

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

Covered: integer, float, string, bool and nil literals; the binary and unary
selectors listed in §3.3; array literals; immutable `let` bindings; statement
sequences; `if`/`else` as a value; `return`; `Float32.from_bits`/
`Float64.from_bits`; `to_bits`; `hash`; and **plain module functions** with
positional parameters, including recursion and mutual calls.

This is the design review's first vertical-slice milestone - Integer, Bool/Nil,
local variables, arithmetic and comparison, function definition and call, `if`,
recursion, `return` - with agreement between the two backends asserted for each.

Declined, each by name: `declaration` (anything that is not a plain module),
`module` (open, mixin, generic or decorated), `module body` (a non-method
statement), `method` (async, override, `impl`, decorated, generic or
class-kind), `abstract method`, `parameter` (rest, keyword or block),
`statement` (which includes `mut`, `const`, global and deferred bindings, and
`while`), `closure`, `call` (any shape beyond §3.3/§3.6), `call arity`,
`name` (unbound), `member`, `index`, `symbol`, `hash`, `tuple`, `try`, `await`,
`yield`, `assignment`, plus the structural refusals `rejected source`,
`rejected literal`, `empty program`, `empty body`, `array too long`,
`call too wide`, `from_bits arity`, `branch patch` and `register exhaustion`.

This list is pinned by test, so widening coverage without updating this document
fails the build.

## 6. Not yet defined

Named so the gaps are not mistaken for decisions:

- **Loops.** `while` is declined. A loop needs a BACKWARD jump, which the
  verifier's single forward pass over written-before-read does not yet reason
  about: a register written inside a loop body is not written on the first
  iteration's entry edge.
- **Methods on Classes.** Only plain module functions are covered. An instance
  method needs a receiver, dispatch through the MRO, and revision awareness.
- **Closure capture.** Needs the HIR layer the design review places between AST
  and execution IR; capture analysis belongs there, not here.
- **Frames as GC roots.** The IR now has frames, but the COLLECTOR does not yet
  walk them: collection still runs only where the evaluator owns the whole root
  set. Connecting the two is what would let a collection run mid-call.
- **Serialisation.** There is no on-disk format. Programs are compiled and
  executed in memory. The review records a P0 against the old `.irc` reader for
  trusting file contents — no magic or version check, no field-count or string
  length limits, no remaining-size validation — so if a format is ever added,
  those checks are requirements from the start, not additions.
- **Disassembler.** The review lists readable disassembly as a benefit of the
  register form; the `Debug` rendering stands in for now.

## 7. Divergences from the design review

Recorded rather than silently taken.

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
