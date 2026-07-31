# TYPES Vector Classification

This document classifies the committed `IRIS-V1-TYPES` vectors from the
normative tables in `spec/iris-v1/05-types-contracts-generics.md`.

**Coverage is partial.** The chapter contains 80 vector rows: 64 standard vector
rows and 16 entries in the Type-normalization law tables. 7 are committed here;
the remaining 73 are NOT yet transcribed and are therefore not covered by any
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

## Remaining rows and their blockers

Every row below was PROBED against the implementation rather than assumed
blocked, and re-probed after String became a runtime value rather than carried
forward. All 47 rows carrying Iris source were evaluated; three produced a value
and are now transcribed, and the groups below record why the rest did not.

| Blocker | Rows | What the probe showed |
| --- | --- | --- |
| Type-normalization law tables | `V001`-`V016` | These are not vector rows with a source and an observable. They are algebraic laws stated as a table, such as `String \| Never` normalizing to `String`. Asserting them needs Types to be constructible and comparable as VALUES, which needs the Type-expression surface below. |
| Generic Types in expression position | `V219`, `V226`, `V229`, `V233`, `V235`, `V237`, `V238`, `V243`-`V247`, `V254`, `V256`, `V257` | `IRIS-V1-GRAMMAR-C020` recognizes generic angle brackets only in declaration and Type grammar contexts, and `primary_expr` has no generic form, so `Box<String>.new()` is a parse error. These rows need a closed generic Type usable as an expression. |
| Metadata fixtures | `V200`-`V203`, `V206`-`V208`, `V214`, `V234`, `V239`-`V242`, `V260`, `V262` | The row supplies a PROSE metadata schedule, not source: opening a candidate, staging members, then forcing validation failure. This is the same out-of-band scheduling the RUNTIME chapter records as `no-fixture`. |
| String plus another missing capability | `V018`, `V205`, `V222`, `V224`, `V225`, `V227`, `V228`, `V230`, `V231`, `V236`, `V249`, `V250` | String IS now a runtime value, which closed `V211` and `V255`. Each row here needs a SECOND capability as well: a Contract declaration, a generic Method or Type in expression position, or an open-Class redeclaration. Re-probed individually rather than assumed. |
| Contract declarations | `V204`, `V248`, `V252`, `V261` | The v1.16 errata (`IRIS-V1-GRAMMAR-C062`) made the bodyless requirement form parse, which closed `V258`. These four still need `open contract` rejection, `impl` conformance checking, Contract views, or `module for C` rejection. |
| Type values and reflection | `V212`, `V213`, `V215`-`V218`, `V221`, `V223`, `V251`, `V253`, `V259` | These read `.type` on a Type expression and compare identities, as in `(String?).type same? (String \| Nil).type`. A Type expression is not an expression form today, and `.type` answers only on a Class value. |
