# CONTROL Vector Classification

This document classifies the 41 committed `IRIS-V1-CONTROL` vectors from the
normative table in `spec/iris-v1/04-bindings-callables-control-flow.md`.

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
| `V019` | negative | needs-subsystem | The static pass detects the condition, but `IRIS-V1-CONTROL-D-438` names no stable code, so the expectation would be authored rather than spec-derived. |
| `V020` | positive | executable | `for` runs the body for a yielded `nil` and closes the Iterator on exit, per `C044` and `C046`. |
| `V021` | negative | executable | A `for` destructuring mismatch raises `PatternMatchError`, per `IRIS-V1-CONTROL-C045`. |
| `V022` | positive | executable | Each iteration binds in a fresh scope, so escaped Closures return distinct values. |
| `V023` | positive | executable | A labelled `break` reaches the named outer loop while a bare one targets the nearest, per `IRIS-V1-CONTROL-C048`. |
| `V024` | diagnostic | needs-subsystem | The static pass detects a transfer crossing a Closure boundary, but `D-421` names no stable code for it. |
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
| `V037` | positive | needs-subsystem | Requires `return` with cleanup traversal. |
| `V038` | diagnostic | needs-subsystem | The static pass detects `return` outside any callable, but `D-421` names no stable code for it. |
| `V039` | positive | executable | Mutable binding assignment. |
| `V040` | diagnostic | needs-subsystem | The static pass detects the write to an immutable binding, but `D-426` names no stable code for it. |
| `V041` | diagnostic | needs-subsystem | Requires fixed inferred local types, which needs type inference the pass does not have. |

## Note on the diagnostic rows

Eight rows are `category: diagnostic` and name a stable code such as
`BINDING_LET_REQUIRES_INITIALIZER`. The RUNTIME runner gained a `diagnostics`
observation channel in milestone 2, but it reports LEXICAL and PARSE
diagnostics. These eight are COMPILER diagnostics for semantic conditions that
parse successfully, so they need a separate static-analysis pass rather than the
existing channel.
