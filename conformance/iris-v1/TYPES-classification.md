# TYPES Vector Classification

This document classifies the committed `IRIS-V1-TYPES` vectors from the
normative tables in `spec/iris-v1/05-types-contracts-generics.md`.

**Coverage is partial.** The chapter contains 80 vector rows: 64 standard vector
rows and 16 entries in the Type-normalization law tables. 60 are committed here;
the remaining 20 are NOT yet transcribed and are therefore not covered by any
evidence in this repository.

`executable` means the frozen row supplies a concrete source and a concrete
observable that the runner can compare today. Every `executable` row was probed
before transcription and tamper-tested: reversing the asserted behaviour
correctly fails each.

| Vector ID | Category | Bucket | Reason |
| --- | --- | --- | --- |
| `V209` | diagnostic | executable | The frozen row NAMES `RAW_GENERIC_TYPE_FORBIDDEN`, and `IRIS-V1-TYPES-C061` makes a bare generic Class name definition metadata rather than an instance Type. |
| `V210` | diagnostic | executable | The frozen row NAMES `GENERIC_BRACKET_SYNTAX_FORBIDDEN`; angle brackets are the accepted spelling under `IRIS-V1-GRAMMAR-C020`. |
| `V232` | diagnostic | executable | The frozen row NAMES `GENERIC_ARGUMENT_ARITY`; `C061` requires ordinary construction to supply a closed `Box<Type>`. |
| `V211` | positive | executable | A reified union annotation still answers an `is` test, now that String is a runtime value with its own builtin Class. |
| `V255` | positive | executable | `IRIS-V1-TYPES-C030` makes `value as? T` yield the SAME value on success and `nil` on a failed check, never converting. |
| `V220` | negative | executable | `IRIS-V1-TYPES-C004` makes a written annotation a RUNTIME boundary guard, so `NonNil` rejects `nil` before the value is published. It was grouped under reflection by mistake: it never reads `.type`. |
| `V258` | diagnostic | executable | `IRIS-V1-TYPES-C042` forbids a Method body in a Contract. The v1.16 errata makes the body PARSE, so it is reported as a static diagnostic rather than a parse error. |
| `V204` | diagnostic | executable | `D-177` gives a Contract no revision to reopen, so `open` is consumed by the parser and rejected statically rather than as a parse error. |
| `V248` | diagnostic | executable | `IRIS-V1-TYPES-C046` rejects an unmarked Class-provided implementation where an explicit `impl` is required. |
| `V261` | diagnostic | executable | `D-279` makes Contract conformance a Class fact, so `module M for C` parses and is rejected statically. |
| `V235` | diagnostic | executable | `IRIS-V1-TYPES-C063` makes `open class Box<String>` an error in v1; a parameter entry naming an existing Type is a closed construction. |
| `V243` | diagnostic | executable | `D-216`: `where T: U, U: T` resolves neither parameter first, so no argument can satisfy the pair. `C058` keeps F-bounded constraints legal. |
| `V245` | diagnostic | executable | `D-218`: `Pair<String>` against `class Pair<T,U>` supplies no default `U`, in annotation and expression position alike. |
| `V237` | negative | executable | `IRIS-V1-TYPES-C064` gives a `shared class property` ONE slot on the unapplied definition, so it cannot name a parameter that differs per construction. The v1.18 errata (`IRIS-V1-GRAMMAR-C064`) added the syntax. |
| `V233` | positive | executable | `D-206` interns a closed Type by definition AND normalized arguments, so `Box<String>` and `Box<Integer>` are different Types of one definition. |
| `V259` | diagnostic | executable | `IRIS-V1-TYPES-C004` requires a PROVABLE violation to be diagnosed BEFORE execution; `D-458` makes `Never` uninhabited, so any normal return violates it. |
| `V001` | positive | executable | Union is commutative: members are sorted when the normal form is built. |
| `V003` | positive | executable | Union is idempotent: members are deduplicated. |
| `V004` | positive | executable | Intersection is idempotent. |
| `V007` | positive | executable | `IRIS-V1-TYPES-C023` makes `Never` the union identity. |
| `V008` | positive | executable | `C023` makes `Never` an intersection annihilator. |
| `V009` | positive | executable | `C009` keeps `Object` the top, so it is the intersection identity. |
| `V010` | positive | executable | `Object` absorbs a union. |
| `V011` | positive | executable | `C011` makes `T?` sugar for `T | Nil`. |
| `V012` | positive | executable | An optional `Object` collapses to `Object`. |
| `V013` | positive | executable | An optional `Never` collapses to `Nil`. |
| `V015` | positive | executable | `Nil & NonNil` is uninhabited. |
| `V005` | positive | executable | Union absorption keeps the WIDER member of a declared subtype pair. |
| `V006` | positive | executable | Intersection absorption keeps the NARROWER member. |
| `V014` | positive | executable | `NonNil` removes `Nil` from an intersection. |
| `V016` | positive | executable | An intersection over a union is deliberately NOT distributed; the compact form is kept. |
| `V212` | positive | executable | A nilable Type equals its union with `Nil`. |
| `V213` | positive | executable | A union with a supertype absorbs the subtype. |
| `V215` | positive | executable | `Nil & NonNil` reduces to `Never`. |
| `V216` | positive | executable | `Never` is the union identity. |
| `V217` | positive | executable | `Object` is the intersection identity. |
| `V218` | positive | executable | An optional `Object` collapses to `Object`. |
| `V221` | positive | executable | `NonNil` removes `Nil` from a union. |
| `V226` | negative | executable | Generic arguments are INVARIANT: `Box<String>` is not assignable to `Box<Object>`, so argument lists must match exactly rather than by subtyping. |
| `V236` | positive | executable | `IRIS-V1-TYPES-C064` gives generic class-level storage INDEPENDENT storage per closed construction; the fixture reads both slots back rather than observing the assignment results. |
| `V238` | diagnostic | executable | A `shared class property` belongs to the UNAPPLIED definition, so a closed construction cannot reach it. The vector asserts that error; the bare-Class read is covered by a unit test. |
| `V254` | negative | executable | `IRIS-V1-TYPES-C014` CHECKS the value on entry to `Dynamic<T>`; the boundary lifts static member validation only INSIDE it. The row names `TypeError`, which is recorded as a defect. |
| `V018` | positive | executable | `IRIS-V1-TYPES-C032` constructs a checked view and `C049` makes `..name()` select the Contract slot. |
| `V250` | positive | executable | `IRIS-V1-TYPES-C047` lets ONE unqualified `impl` member satisfy the requirement, so the view reaches it. |
| `V251` | negative | executable | `IRIS-V1-TYPES-C050` makes a Contract view identity-LESS, so `same?` raises rather than comparing the receiver. |
| `V252` | positive | executable | `IRIS-V1-TYPES-C050` compares views by receiver equality plus Contract identity; a view is never equal to a non-view. |
| `V244` | diagnostic | executable | `IRIS-V1-TYPES-C058` needs EXPLICIT nominal conformance for an F-bounded bound; `C067` validates it at materialization and raises. |
| `V246` | positive | executable | `D-219` reifies a CLOSED Module Type as a mixin target. The fixture writes the header `class_mixin` the grammar admits; the frozen row's body-level `mixin` is not a `declaration_body` member. |
| `V247` | diagnostic | executable | `D-220` requires a generic Module mixin to state its arguments EXPLICITLY; host inference is not used. |
| `V219` | positive | executable | `IRIS-V1-TYPES-C056` gives an unconstrained parameter the implicit bound `Object`, so `Nil` is a valid argument. The fixture READS the construction back, since `let` yields nothing. |
| `V256` | diagnostic | executable | `IRIS-V1-TYPES-C058` rejects a fixed point: an alias naming ITSELF has no finite expansion. The alias-identity half of the row needs `Array` and is covered by a unit test. |
| `V224` | diagnostic | executable | `IRIS-V1-TYPES-C047` forbids choosing between INCOMPATIBLE same-name requirements or forming an overload set. |
| `V249` | positive | executable | `IRIS-V1-TYPES-C047` merges COMPATIBLE same-name requirements, so one `impl` member is reached through either Contract's view. |
| `V225` | positive | executable | `IRIS-V1-TYPES-C037` makes a parameter CONTRAVARIANT and a return COVARIANT. The helper lives in a Module body per `IRIS-V1-CONTROL-C012`; the row's bare spellings are recorded as a defect. |
| `V227` | positive | executable | `method_decl`'s `generic_params?` was never parsed. The inferred argument is observed through a `let` with a written Type, which is the boundary `C004` guards at RUNTIME. |
| `V231` | diagnostic | executable | `D-204`: a CONSTRUCTION may infer its argument from `Box<_>`, but a written annotation persists beyond that inference and must name a closed Type. |
| `V228` | diagnostic | executable | `IRIS-V1-TYPES-C069` unions multiple lower-bound candidates for ONE type parameter, so `pair("x", 1)` infers `String \| Integer` and a `String` target rejects it. |
| `V230` | diagnostic | executable | `IRIS-V1-TYPES-C060` gives every application FIXED FULL ARITY; the v1.20 errata (`IRIS-V1-GRAMMAR-C066`) added the call-site type-argument syntax `C071` presupposes. |
| `V229` | diagnostic | executable | `IRIS-V1-TYPES-C070` forbids a STANDALONE unconstrained call from defaulting its type parameter; an expected result or an argument-bound parameter is exempt. |
| `V223` | diagnostic | executable | `D-196` makes the accepted-arity INTERSECTION across a union's members the set a call may use; an empty one rejects every call. |

## Remaining rows and their blockers

Every row below was PROBED against the implementation rather than assumed
blocked, and re-probed after String became a runtime value rather than carried
forward. All 47 rows carrying Iris source were evaluated; three produced a value
and are now transcribed, and the groups below record why the rest did not.

| Blocker | Rows | What the probe showed |
| --- | --- | --- |
| Type-normalization law tables | `V002` | The v1.19 errata (`IRIS-V1-GRAMMAR-C065`) reified Type expressions, which closed eleven of these laws. The one left needs a CONTRACT usable as a Type, which no Contract declaration yet reifies. Asserting them needs Types to be constructible and comparable as VALUES, which needs the Type-expression surface below. |
| Generic Types in expression position | `V257` | The v1.17 errata (`IRIS-V1-GRAMMAR-C063`) added `closed_generic_name` to `primary_expr`, so `Box<String>.new()` now parses and `V245` closed. These rows each need a FURTHER capability: generic inference, Dynamic entry, Module generic arguments, or recursive alias detection. |
| Metadata fixtures | `V200`-`V203`, `V206`-`V208`, `V214`, `V234`, `V239`-`V242`, `V260`, `V262` | The row supplies a PROSE metadata schedule, not source: opening a candidate, staging members, then forcing validation failure. This is the same out-of-band scheduling the RUNTIME chapter records as `no-fixture`. |
| Open-class transaction ordering and MutableString | `V205`, `V222` | `V205` needs `D-178` declaration collection: an `open class A` may PRECEDE the `class A` it reopens, which requires the origin to be resolved before the open transaction runs. A reopen no longer collides in the qualified namespace, but deferring the transaction to a second pass broke the ordinary `class` then `open class` order, so the ordering itself is unimplemented. `V222` needs `MutableString`. |
| Type values and reflection | `V253` | These read `.type` on a Type expression and compare identities, as in `(String?).type same? (String \| Nil).type`. A Type expression is not an expression form today, and `.type` answers only on a Class value. |
