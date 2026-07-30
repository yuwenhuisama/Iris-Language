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
| `V002` | diagnostic | needs-subsystem | Requires a compiler diagnostic channel; the runner observes runtime values and errors only. |
| `V003` | diagnostic | needs-subsystem | Same diagnostic channel. |
| `V004` | negative | needs-subsystem | Requires deferred `mut` bindings with definite-assignment tracking. |
| `V005` | negative | needs-subsystem | Requires static unresolved-binding detection. |
| `V006` | positive | executable | Closure captures a mutable binding by reference, per `IRIS-V1-CONTROL-C028`. |
| `V007` | diagnostic | needs-subsystem | Same diagnostic channel. |
| `V008` | positive | needs-subsystem | Requires optional, rest, keyword and keyword-rest parameter categories. |
| `V009` | negative | needs-subsystem | Requires keyword arguments. |
| `V010` | negative | executable | Arity mismatch raises `ArgumentError` per `IRIS-V1-CONTROL-C025`. |
| `V011` | positive | executable | Property setter marker propagation. |
| `V012` | positive | needs-subsystem | Requires index assignment targets. |
| `V013` | positive | executable | `&&=` skips the right side on the no-write path, per `IRIS-V1-CONTROL-C037`. |
| `V014` | positive | executable | `||=` writes on the falsy path. |
| `V015` | negative | executable | A non-Bool `to_bool` raises `TypeContractError`. |
| `V016` | positive | executable | `if` without `else` yields `nil`. |
| `V017` | positive | executable | `while` natural completion yields `nil`, per `IRIS-V1-CONTROL-C043`. |
| `V018` | positive | executable | `break 7` carries the loop result. |
| `V019` | negative | needs-subsystem | Requires loop control-target validation. |
| `V020` | positive | executable | `for` runs the body for a yielded `nil` and closes the Iterator on exit, per `C044` and `C046`. |
| `V021` | negative | executable | A `for` destructuring mismatch raises `PatternMatchError`, per `IRIS-V1-CONTROL-C045`. |
| `V022` | positive | executable | Each iteration binds in a fresh scope, so escaped Closures return distinct values. |
| `V023` | positive | executable | A labelled `break` reaches the named outer loop while a bare one targets the nearest, per `IRIS-V1-CONTROL-C048`. |
| `V024` | diagnostic | needs-subsystem | Same diagnostic channel. |
| `V025` | positive | executable | `match` tests arms in source order with no fallthrough, per `IRIS-V1-CONTROL-C050`. |
| `V026` | diagnostic | needs-subsystem | Same diagnostic channel. |
| `V027`-`V031` | mixed | needs-subsystem | Require `ExceptionContext` objects, cause chaining, or catch dynamic-extent tracking. |
| `V032` | positive | executable | Normal catch completion. |
| `V033` | positive | executable | A `finally` value is discarded and the provisional result is preserved. |
| `V034`-`V037` | mixed | needs-subsystem | Require exception chaining, suppressed-exception recording, cycle detection, or cleanup traversal. |
| `V038` | diagnostic | needs-subsystem | Same diagnostic channel. |
| `V039` | positive | executable | Mutable binding assignment. |
| `V040`-`V041` | diagnostic | needs-subsystem | Same diagnostic channel. |

## Note on the diagnostic rows

Eight rows are `category: diagnostic` and name a stable code such as
`BINDING_LET_REQUIRES_INITIALIZER`. The RUNTIME runner gained a `diagnostics`
observation channel in milestone 2, but it reports LEXICAL and PARSE
diagnostics. These eight are COMPILER diagnostics for semantic conditions that
parse successfully, so they need a separate static-analysis pass rather than the
existing channel.
