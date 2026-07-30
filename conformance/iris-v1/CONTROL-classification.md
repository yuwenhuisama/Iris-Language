# CONTROL Vector Classification

This document classifies the committed `IRIS-V1-CONTROL` vectors from the
normative table in `spec/iris-v1/04-bindings-callables-control-flow.md`.

**Coverage is partial.** The normative table contains 134 vector rows. 106 are
committed here; the remaining 28 are NOT yet transcribed and are therefore not
covered by any evidence in this repository. An assessment of those rows found
they are blocked on capabilities that do not exist yet, chiefly static type
reflection, top-level `fun` declarations, class variables and globals, subclass
`catch` matching, and named Iterator fixtures. Four legacy-rejection rows are
additionally blocked because the frozen text names no stable diagnostic code.

`executable` means the frozen row supplies a concrete source and a concrete
observable that the runner can compare today. `needs-subsystem` means the row is
concrete but its observable depends on a capability that does not exist yet.

Every `executable` row was probed before transcription, and three representative
rows were tamper-tested: reversing the asserted behaviour correctly fails each.

| Vector ID | Category | Bucket | Reason |
| --- | --- | --- | --- |
| `V001` | positive | executable | Binding initializer value. |
| `V002` | diagnostic | executable | The static pass reports `BINDING_LET_REQUIRES_INITIALIZER`, a code the spec NAMES in `IRIS-V1-CONTROL-C004`. |
| `V003` | diagnostic | executable | The static pass reports `BINDING_MISSING_TYPE_FOR_DEFERRED_INIT`, a code the spec NAMES in `IRIS-V1-CONTROL-C004`. |
| `V004` | negative | needs-subsystem | Deferred `mut` now parses and is retained, but definite-assignment tracking does not exist. |
| `V005` | negative | executable | `IRIS-V1-CONTROL-C009` accepts a runtime `NameError`, which is a spec-named code, so no invented static code is needed. |
| `V006` | positive | executable | Closure captures a mutable binding by reference, per `IRIS-V1-CONTROL-C028`. |
| `V007` | diagnostic | needs-subsystem | The diagnostic channel now exists, but `IRIS-V1-CONTROL-C017` conditions the code on there being no unique expected callable type, and no expected-type inference exists. |
| `V008` | positive | executable | All six `IRIS-V1-CONTROL-C023` parameter categories bind as declared, including `**kwargs` into a `Hash<Symbol,V>`. |
| `V009` | negative | executable | A duplicate keyword argument raises `ArgumentError`, per `D-357`. |
| `V010` | negative | executable | Arity mismatch raises `ArgumentError` per `IRIS-V1-CONTROL-C025`. |
| `V011` | positive | executable | Property setter marker propagation. |
| `V012` | positive | executable | Receiver, index and RHS are each evaluated exactly once, per `IRIS-V1-CONTROL-C036` and `D-347`. |
| `V013` | positive | executable | `&&=` skips the right side on the no-write path, per `IRIS-V1-CONTROL-C037`. |
| `V014` | positive | executable | `||=` writes on the falsy path. |
| `V015` | negative | executable | A non-Bool `to_bool` raises `TypeContractError`. |
| `V016` | positive | executable | `if` without `else` yields `nil`. |
| `V017` | positive | executable | `while` natural completion yields `nil`, per `IRIS-V1-CONTROL-C043`. |
| `V018` | positive | executable | `break 7` carries the loop result. |
| `V019` | negative | executable | The v1.14 errata names `CONTROL_TRANSFER_WITHOUT_TARGET` in `IRIS-V1-CONTROL-C077`. |
| `V020` | positive | executable | `for` runs the body for a yielded `nil` and closes the Iterator on exit, per `C044` and `C046`. |
| `V021` | negative | executable | A `for` destructuring mismatch raises `PatternMatchError`, per `IRIS-V1-CONTROL-C045`. |
| `V022` | positive | executable | Each iteration binds in a fresh scope, so escaped Closures return distinct values. |
| `V023` | positive | executable | A labelled `break` reaches the named outer loop while a bare one targets the nearest, per `IRIS-V1-CONTROL-C048`. |
| `V024` | diagnostic | executable | `CONTROL_TARGET_CROSSES_CLOSURE` was ALREADY named by `IRIS-V1-CONTROL-V339A` under the same `D-421`; no errata was needed. |
| `V025` | positive | executable | `match` tests arms in source order with no fallthrough, per `IRIS-V1-CONTROL-C050`. |
| `V026` | diagnostic | needs-subsystem | Requires match exhaustiveness over an open-ended type, which needs type information the pass does not have. |
| `V027` | positive | executable | A catch binds the `ExceptionContext` and `context.value` is the raised object. |
| `V028` | positive | executable | `raise value from nil` suppresses chaining, so the cause is `nil`. |
| `V029` | negative | executable | A non-`ExceptionContext` cause is a `TypeError`, per `IRIS-V1-CONTROL-C057`. |
| `V030` | positive | needs-subsystem | Requires re-raise site recording on the continued context. |
| `V031` | negative | executable | A bare `raise` outside a catch extent raises `NoActiveExceptionError`. |
| `V032` | positive | executable | Normal catch completion. |
| `V033` | positive | executable | A `finally` value is discarded and the provisional result is preserved. |
| `V034` | positive | executable | A `finally` raise becomes primary and the pending context becomes its cause, per `IRIS-V1-CONTROL-C064`. |
| `V035` | positive | executable | A cleanup failure is appended to the primary context's `suppressed`, per `IRIS-V1-CONTROL-C047`. |
| `V036` | negative | executable | A cause cycle raises `ExceptionChainError` and leaves the graph unchanged, per `D-161`. |
| `V037` | positive | executable | `return` runs active traversal cleanup before leaving the callable. |
| `V038` | diagnostic | executable | The v1.14 errata names `CONTROL_RETURN_OUTSIDE_CALLABLE` in `IRIS-V1-CONTROL-C077`. |
| `V039` | positive | executable | Mutable binding assignment. |
| `V040` | diagnostic | executable | The v1.14 errata names `BINDING_ASSIGN_TO_IMMUTABLE` in `IRIS-V1-CONTROL-C077`. |
| `V041` | diagnostic | needs-subsystem | Requires fixed inferred local types, which needs type inference the pass does not have. |

## Note on the diagnostic rows

Eight rows are `category: diagnostic` and name a stable code such as
`BINDING_LET_REQUIRES_INITIALIZER`. The RUNTIME runner gained a `diagnostics`
observation channel in milestone 2, but it reports LEXICAL and PARSE
diagnostics. These eight are COMPILER diagnostics for semantic conditions that
parse successfully, so they need a separate static-analysis pass rather than the
existing channel.
| `V287` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V293` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V297` | negative | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V299` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V308` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V309` | negative | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V314` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V326` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V328` | negative | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V356` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V357` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V360` | positive | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V361` | diagnostic | executable | Transcribed after probing; exception, truthiness and loop-target semantics verified against the frozen row. |
| `V338` | positive | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V340` | positive | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V341` | negative | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V343` | positive | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V346` | positive | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V353` | positive | executable | Transcribed after probing; Closure capture, block channel and callable-Type behaviour verified against the frozen row. |
| `V339` | positive | executable | Transcribed after probing; `return` boundary semantics verified against the frozen row. |
| `V307` | diagnostic | executable | Transcribed after probing; `try` in expression position verified against the frozen row. |
| `V310` | positive | executable | Transcribed after probing; `try` in expression position verified against the frozen row. |
| `V311` | positive | executable | Transcribed after probing; `try` in expression position verified against the frozen row. |
| `V312` | positive | executable | Transcribed after probing; `try` in expression position verified against the frozen row. |
| `V313` | positive | executable | Transcribed after probing; `try` in expression position verified against the frozen row. |
| `V291` | positive | executable | Transcribed after probing; catch selection and ExceptionContext identity verified against the frozen row. |
| `V296` | positive | executable | Transcribed after probing; catch selection and ExceptionContext identity verified against the frozen row. |
| `V302A` | negative | executable | Transcribed after probing; catch selection and ExceptionContext identity verified against the frozen row. |
| `V306` | positive | executable | Transcribed after probing; catch selection and ExceptionContext identity verified against the frozen row. |
| `V318` | positive | executable | Transcribed after probing; selector suffixes, arity, binding mutability and loop value verified against the frozen row. |
| `V329` | negative | executable | Transcribed after probing; selector suffixes, arity, binding mutability and loop value verified against the frozen row. |
| `V344` | positive | executable | Transcribed after probing; selector suffixes, arity, binding mutability and loop value verified against the frozen row. |
| `V355` | positive | executable | Transcribed after probing; selector suffixes, arity, binding mutability and loop value verified against the frozen row. |
| `V295` | positive | executable | Transcribed after probing; branch scope, match selection and raised-value delivery verified against the frozen row. |
| `V354` | positive | executable | Transcribed after probing; branch scope, match selection and raised-value delivery verified against the frozen row. |
| `V354A` | negative | executable | Transcribed after probing; branch scope, match selection and raised-value delivery verified against the frozen row. |
| `V357` | positive | executable | Transcribed after probing; branch scope, match selection and raised-value delivery verified against the frozen row. |
| `V289` | positive | executable | Transcribed after probing; catch/finally ordering and cause-cycle rejection verified against the frozen row. |
| `V304` | negative | executable | Transcribed after probing; catch/finally ordering and cause-cycle rejection verified against the frozen row. |
| `V324` | diagnostic | executable | The frozen row NAMES its diagnostic code, so the expectation is spec-derived. |
| `V359` | diagnostic | executable | The frozen row NAMES its diagnostic code, so the expectation is spec-derived. |
| `V339A` | diagnostic | executable | The frozen row NAMES `CONTROL_TARGET_CROSSES_CLOSURE`, so the expectation is spec-derived. |
| `V319` | positive | executable | Transcribed after probing; property getter/setter dispatch and assignment result verified against the frozen row. |
| `V320` | positive | executable | Transcribed after probing; property getter/setter dispatch and assignment result verified against the frozen row. |
| `V321` | positive | executable | Transcribed after probing; property getter/setter dispatch and assignment result verified against the frozen row. |
| `V322` | positive | executable | Transcribed after probing; property getter/setter dispatch and assignment result verified against the frozen row. |
| `V323` | positive | executable | Transcribed after probing; evaluation order, per-call defaults and bare-callable behaviour verified against the frozen row. |
| `V337` | positive | executable | Transcribed after probing; evaluation order, per-call defaults and bare-callable behaviour verified against the frozen row. |
| `V342` | positive | executable | Transcribed after probing; evaluation order, per-call defaults and bare-callable behaviour verified against the frozen row. |
| `V282` | positive | executable | Transcribed after probing; iterator cleanup and suppressed-context behaviour verified against the frozen row. |
| `V283` | negative | executable | Transcribed after probing; iterator cleanup and suppressed-context behaviour verified against the frozen row. |
| `V303` | positive | executable | Transcribed after probing; iterator cleanup and suppressed-context behaviour verified against the frozen row. |
| `V355A` | positive | executable | Transcribed after probing; iterator cleanup and suppressed-context behaviour verified against the frozen row. |
| `V288` | positive | executable | Transcribed after probing; automatic, explicit and suppressed causes verified against the frozen row. |
| `V327` | positive | executable | Transcribed after probing; `to_bool` call count and short-circuit verified against the frozen row. |
| `V290` | positive | executable | Transcribed after probing; subclass catch matching and six-category parameter binding verified against the frozen row. |
| `V336` | positive | executable | Transcribed after probing; subclass catch matching and six-category parameter binding verified against the frozen row. |
| `V347` | positive | executable | Transcribed after probing; raw current-receiver ivar creation verified against the frozen row. |
| `V362` | positive | needs-subsystem | `method_missing` IS reached for an unknown selector, but the truthiness path never consults it: with `to_bool` absent, `if` falls back to DEFAULT truthiness instead of dispatching the missing message. Probing this row produced a false positive, since the expected `:then` arrives either way; a call counter shows `method_missing` runs zero times. |
| `V349` | positive | executable | Transcribed after probing; instance and Class Methods read the declaring lexical Class cell. |
| `V348` | negative | needs-subsystem | The redeclaration IS rejected, but the runner renders every `ClassError` as a generic `RuntimeError`, so the expectation could not distinguish a class-variable redeclaration from any other Class failure. |
| `V315` | diagnostic | executable | Transcribed after probing; catch-binding immutability and logical-assignment short-circuit verified against the frozen row. |
| `V325` | positive | executable | Transcribed after probing; catch-binding immutability and logical-assignment short-circuit verified against the frozen row. |
| `V301` | diagnostic | executable | The frozen row NAMES `DISCARD_BINDING_READ`, so the expectation is spec-derived. |
| `V344A` | diagnostic | executable | Three independent fixtures, each asserting a spec-named binding diagnostic. |
| `V347A` | diagnostic | needs-subsystem | The local-binding fixture is diagnosed, but `$missing = 1` is a parse error rather than a missing-storage diagnostic and `@@missing = 1` produces none, so two of the three fixtures cannot yet be asserted. |
| `V305` | positive | executable | Transcribed after probing; two contexts from separate raises are distinct Hash keys. |
| `V294` | diagnostic | executable | Transcribed after probing; a write to a catch binding reports the spec-named immutable-binding code. |
