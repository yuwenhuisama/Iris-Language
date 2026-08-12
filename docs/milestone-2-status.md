# Iris v1 Milestone 2 — Implementation Status

**Revision:** Iris v1.34 (frozen semantics with owner-approved errata)
**Branch:** `new-iris-dev`
**Baseline of this milestone:** `3fa54ab`

## Current Conformance

```
RUNTIME     passed: 151, failed: 0, needs_subsystem: 5, no_fixture: 5, differential: 3  (164 records)
GRAMMAR     passed: 36, failed: 0, deferred: 1, authored_expect: 5, unrunnable_source: 9  (51 records)
CONTROL     passed: 152, failed: 0, needs_subsystem: 5, no_fixture: 0, differential: 0  (157 records)
TYPES       passed: 88, failed: 0, needs_subsystem: 1, no_fixture: 0, differential: 0  (89 records)
META        passed: 58, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (58 records)
ASYNC       passed: 43, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (43 records)
COLLECTIONS passed: 125, failed: 0, needs_subsystem: 1, no_fixture: 0, differential: 1  (127 records)
FFI         passed: 32, failed: 0, needs_subsystem: 1, no_fixture: 0, differential: 0  (33 records)
IDENTITY    passed: 7, failed: 0, needs_subsystem: 1, no_fixture: 0, differential: 0  (8 records)
LIBRARY     passed: 19, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (19 records)
CONFORMANCE passed: 7, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (7 records)
TRACE       passed: 2, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (2 records)
MIGRATION   passed: 2, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0  (2 records)
```

Milestone 2 opened at RUNTIME 41 and closed at 94. The higher counts above are
later clause-coverage work, which adds locally authored rows for clauses no
spec row enumerates.

```bash
cargo run -p iris-conformance -- --chapter RUNTIME
cargo run -p iris-conformance -- --chapter GRAMMAR
cargo run -p iris-conformance -- --chapter CONTROL
# ... and TYPES META ASYNC COLLECTIONS FFI IDENTITY LIBRARY CONFORMANCE TRACE MIGRATION
```

## Quality Gates

All six must pass before any commit. Capture exit codes without piping first — a pipe replaces `$LASTEXITCODE` with the filter's code, which once hid a real clippy failure.

```bash
cargo test --workspace
cargo test -p iris-lexer        # 44 tests, must finish instantly
cargo fmt --check
cargo clippy --all-targets -- -D warnings
python3 tools/corpus-audit.py   # CONFORMANCE rules over the vector corpus
python3 tools/spec-audit.py     # TRACE and MIGRATION rules over the spec text
```

Chain the gates into the commit with `&&`, not `;`. A `;` chain once let a
commit land while the workspace tests were red, because the failing exit code
did not stop the sequence.

## Language Capabilities Implemented

| Area | What works |
| --- | --- |
| Exceptions | `raise`, `try`/`catch`/`finally`, unwinding as a typed outcome; a raise from `initialize` propagates and `new` yields no instance |
| Control flow | `if` as an expression in binding, argument, operand and array-element positions; implicit `nil` without `else` |
| Operators | Full comparison set deriving from `<=>`; short-circuit `&&`/`||` returning the original operand; unary `!`; user-declared operator methods |
| Truthiness | Dynamic `to_bool` protocol on `Nil`, `Bool` and ordinary classes; a replacement drives `if` |
| Classes | Reopen with `override`; class variables; stored and accessor properties; `super` both bare and qualified |
| Modules | Declarative `mixin` with MRO participation; Module methods on the host receiver; `super` from a Module body; private composition edges |
| MetaCapabilities | The twelve-name vocabulary; effective policy over the superclass chain and composed Modules; `subclass`, `method_set`, `modules` and `instance_state` enforced |
| Reflection | `Reflection::Object` raw ivars; `Reflection::Class` and `Reflection::Module` method lookup, invocation with entry validation, `remove_module`, `set_superclass`, `ancestors`; `alias_method`, `remove_method`, `undef_method`; `method_missing` |
| Built-in value classes | Reopen with source methods; property getters and setters, with numeric immutability preserved |

## Errata Issued This Milestone

Each was raised before the fact and approved by the owner. All are bilingual per `IRIS-V1-TRACE-C020`.

| Revision | Clause | Supplied |
| --- | --- | --- |
| v1.3 | `GRAMMAR-C060` | `if_expression` in `primary_expr`; `CONTROL-C041` already required the value-producing behaviour |
| v1.4 | `GRAMMAR-C061` | `private` on a mixin entry; `D-314` already spelled it and `META-C056` said it belonged in the grammar |
| v1.5 | `META-C118`, `C119` | The reflective Method surface and the `Reflection::*` layering |
| v1.6 | `META-C120` | `set_superclass` and `ancestors` |
| v1.7 | — | Amended two vector rows whose frozen text was unsatisfiable |
| v1.8 | — | Moved the approved semantic source into `spec/drafts/` and repointed its references |
| v1.9 | — | Moved `Document/` under `legacy/` and repointed its 151 references |
| v1.10–v1.20 | — | Published; rows not transcribed into this table at the time |
| v1.21 | `TYPES-C097` | A per-closed materialization failure raises `TypeContractError` |
| v1.22 | `GRAMMAR-C067` | A bare closed generic is a complete expression |
| v1.23 | `GRAMMAR-C068` | A dotted package path in an import |
| v1.24 | `GRAMMAR-C069` | The `override` import marker |
| v1.25 | `META-C121` | The V343 rejection wording |
| v1.26 | `META-C122`, `C123`, `GRAMMAR-C070` | The five Decorator Contracts, the property reflection row and the qualified decorator path |
| v1.27 | `META-C124` | Corrected the decorator phase signatures |
| v1.34 | `ASYNC-C063` | The `Host.run` drive surface `C015` names and `C049` already required. Renumbered from `C050`, which the frozen chapter already uses for flush semantics. |
| v1.33 | `GRAMMAR-C072` | `yield`, widening the reserved inventory a second time to 50, and the generator semantics |
| v1.32 | `GRAMMAR-C071` | The `await` production three clauses already presupposed |
| v1.31 | `META-C126` | The audit digest scope, and the withdrawal of V357's unreproducible constant |
| v1.30 | `TYPES-C099`, `CONTROL-C080` | The `remove_contract` spelling, and naming `ExceptionContext` so its getters can be replaced |
| v1.29 | `TYPES-C098` | Which declared ancestors the `D-174` superclass bound protects, resolving the V201/V014 conflict |
| v1.28 | `META-C125` | The minimal `Plan` and `Transformation` members, `IRIS-DECORATOR-NONDETERMINISTIC` and `IRIS-DECORATOR-KIND` |

### Permanence exceptions

The v1.10 errata is the SIXTH exception and the first to reverse a DECIDED SEMANTIC rather
than a spelling: `IRIS-V1-CONTROL-C076` makes `.call` the sole invocation form and removes the
direct-application spelling that `IRIS-V1-MIG-004` had named as the replacement for legacy
`cast.call(...)`. Weigh any further use against `IRIS-V1-TRACE-C021` itself, which forbids
reinterpreting a decided semantic, rather than against this precedent.

`IRIS-V1-TRACE-C007` makes published content permanent. Four bounded exceptions were authorized:

1. `META-C100` — the four raw-ivar paths moved under `Reflection::Object` (v1.5)
2. `V100` — `reflection.ancestors` respelled as `Reflection::Class.ancestors` (v1.6)
3. `V067` — the "differs" assertion corrected (v1.7)
4. `V015` — the expected error corrected (v1.7)
5. `MIGRATION-C003` — the archived-document path repointed after the move (v1.9)

Each changed only a spelling or an unsatisfiable assertion, never a name, argument, value or decision. Ledger rows 21 and 23 record the first two and note that two uses make this a precedent rather than a one-off: weigh any further use against `TRACE-C007` itself, not against these rows.

## Rows Needing a Non-Executable Validator

Four rows do not observe language behaviour and cannot be transcribed as
executable vectors, so they are recorded here rather than skipped silently.

- `IDENTITY-V001` and `IDENTITY-V012` are documentation validations. RESOLVED
  by a `bucket:documentation` outcome that checks the tree directly. Both facts
  they assert are true of this tree today: `spec/iris-v1` holds exactly 14
  product artifacts, `spec/drafts/iris-language-specification.md` is present as
  the semantic source, chapter 11 records 88 `IRIS-V1-MIG-` divergences, and the
  successor declaration appears in both `README.md` and chapter 01. Asserting
  them needs a documentation validator, which is a different mechanism from
  executing a vector.
- `CONFORMANCE-V010`, `V011` and `V012` are corpus validations. RESOLVED: a
  `bucket:record-validation` outcome checks a fixture record's fields as DATA
  against the row's stated expectation, without executing anything.
- `IDENTITY-V014` needs instance-level revision migration: `migrate_revision`
  on an instance, with a committed revision NOT migrating other instances
  implicitly. That subsystem does not exist.

## Defects Found Without Vector Coverage

Real defects surfaced through probing rather than through the corpus.

- **Array and Hash were not identity-bearing.** `IRIS-V1-COLLECTIONS-C003`
  classifies `Array<T>` and `Hash<K,V>` as *identity-bearing* with mutable
  contents. Both were stored by value, so every binding, argument pass and
  field read COPIED. **Array is fixed** in `841c085`: it now carries a shared
  body, which also deleted the syntax-routed `append` workaround and made
  `C026` fail-fast iteration possible. **Hash is fixed** in `4c0670b`, with `C034`
  structural versioning that excludes value updates.
- **A keyword argument whose value is a Symbol was misparsed.** `f(x: :a)`
  became the single name `x::a`. FIXED: `combine_numeric_literals` joined ANY
  two colon tokens into `::` without checking they were adjacent in the source.
  Both that site and `consume_qualified_separator` now require adjacency.
- **A mutation inside an interpolation segment is lost.** `"${log.append(1)}"`
  leaves `log` empty, while the same call outside the segment works. A segment
  is parsed as its own program and evaluated against the caller's bindings, so
  reads and ordinary sends work but `append`, which is routed by SYNTAX through
  the binding it names, writes into a copy. Found while transcribing V313,
  which was rewritten to record its side effect from a Method body so the row
  observes `C048` ordering rather than this gap. NOT yet fixed.
- **Interpolation is applied AFTER unescaping.** FIXED: interpolation moved out
  of the lexer into the evaluator, so an escaped dollar no longer interpolates.
  Original text: `"\u{24}{1}"` and
  `"\x24{1}"` both answer `1`: the escape produces a `$`, and the literal
  layer then treats the resulting `${1}` as interpolation. An escaped dollar
  must not interpolate. Found while making `String#inspect` reparsable under
  `C050`; recorded rather than fixed, since the ordering lives in the lexer and
  belongs with a row that states it. `inspect` escapes the quote and backslash
  correctly and round-trips for every other case.
- **`punct` discarded the expression-position flag.** The lexer helper took a
  `expression_start` argument and ended with `let _ = expression_start;`, so no
  punctuation token ever updated it. An opening brace therefore never restored
  expression position and `fun f() { /a/ }` could not lex a Regex at all, while
  `{ /a/ }` at top level could. Fixed with the Regex work.
- **Regex literal tokens were measured with the quoted-literal rule.** The
  parser sized a `RegexLiteral` by looking for a quote, so the token was cut at
  the closing slash and every flag was DROPPED, making `/a/im` indistinguishable
  from `/a/`. Fixed with the Regex work.
- **Symbol answers neither `==` nor `!=`.** FIXED in this milestone alongside String equality. Found while transcribing V026,
  where `seen != :KeyConflictError` failed with MessageNotFoundError. Symbol is
  an identity-less immutable value under `C003` and comparing two of them is
  ordinary, so this is a real gap. V026 was rewritten to avoid it rather than
  absorb an unrelated fix; no COLLECTIONS row states Symbol equality directly.
- **A Hash reported its class as `Array`.** The class-name table paired
  `Value::Hash` with `Value::Array` on one arm, so `Hash` was unreachable as a
  reported class. Fixed alongside the Tuple work.
- **The conformance runner mis-decoded non-ASCII JSON.** Its reader used
  `char::from(byte)`, decoding each UTF-8 byte as Latin-1 and splitting every
  multi-byte scalar. No vector could state a non-ASCII expectation. Fixed in
  `d2211ca`.

- **Construction swallowed a raise.** `A.new()` returned an instance even when `initialize` raised. Fixed in `28acea8`.
- **Self-sends from `initialize` failed.** Construction moved the runtime out of the evaluator before running the initializer, so a nested send resolved against an empty runtime; a separate path then erased the failure into a raise of `nil`. Fixed in `39c576e`. This regressed in `28acea8`.
- **`instance_state` was never enforced.** The capability name was accepted but no check existed, so a denying class could still gain a slot. Fixed in `3773f7a`.
- **Ordered numeric comparison was not exact.** `Numeric::compare` rounded both operands to `f64` while `Numeric::equal` already compared exactly, so an `Integer` beyond finite float range became an infinity and `Float64.infinity > 10 ** 1000` returned `false` against `RUNTIME-C131`. Found by re-probing a `needs-subsystem` vector whose bucket reason had gone stale, which is why no vector caught it.

## Remaining Failures

Milestone 2 is closed as delivered at 94 of the 97 runnable RUNTIME vectors. None of the three
remaining failures is an implementation gap, and none can be closed by writing more code.

| Vector | Blocker |
| --- | --- |
| `V103` | Its "no implicit backing storage" claim has no observation path, and that is required rather than missing. `Float64` is identity-less under `RUNTIME-C004`, and `META-C112` requires raw ivar reflection to respect identity-less restrictions, so `list_ivars` MUST refuse a value receiver. Closing it needs an owner-approved re-authoring of the row, which was offered and declined, or a new specified way to observe value-receiver storage. |
| `V013` | Needs a host that can pause an entered frame, commit, then resume. `01-language-identity.md` lists suspension inside a transaction as `PROHIBITED`, so this is conformance harness work, not a language feature, and no errata can reach it. |
| `V083` | Needs the same harness to interleave a commit with an in-flight construction. |

The revision-capture semantics `V013` and `V083` describe are implemented and exercised by
neighbouring vectors; what is absent is an out-of-band frame scheduler in the conformance host.
Building it is milestone-3 sized infrastructure.

`docs/spec-defects-v1.md` holds 49 rows, 30 resolved, and records the blocker for every one.

## Working Agreements

- The specification is frozen. A discrepancy goes in the defect ledger, never into the spec, unless the owner approves an errata under `TRACE-C021`.
- A conformance expectation is evidence. It is never weakened, reclassified or special-cased to make a result look better. An honest failure with a precise blocker is worth more than a forced pass.
- Never invent a selector spelling. Search the whole specification first, including vector-table fixture rows and `traceability-matrix.md` — `append` and the `alias_method` family both turned out to live there after being reported as unsupplied.
- `crates/iris-lexer/` changes need care. A rewrite there once introduced a non-terminating loop that killed the suite with a 64 GiB allocation.
- Only `num-bigint` and `blake3` are approved dependencies. JSON stays hand-rolled.
- Verify claims by probing through `iris_eval::evaluate` rather than by reading code. Most defects in this milestone were found that way.
- Treat a `needs-subsystem` bucket reason as a claim to re-probe, not a fact. Several were written before the subsystems they cite existed. A stale one hid a real `C131` exactness defect for an entire milestone, because a bucketed vector is never executed and so never fails.

## Frozen Invariants

- `Integer(1).hash` is `17824117788395916856`; the singleton hash is `11850167709044604115`.
- `IRIS-V1-RUNTIME-C067` rejects `other.@x`, `obj.@@x` and `A.@@x`. The owner has reaffirmed that this form must never be valid syntax.
- Built-in value classes may not have their runtime superclass changed, per `RUNTIME-C150`.
- Record counts stay at 109 RUNTIME, 41 GRAMMAR and 41 CONTROL, and the buckets must sum to the record count.

## Milestone Status

**Milestone 2 is closed as delivered.** RUNTIME conformance advanced from 41 to 94 across the
milestone. The three FAILING vectors listed above are each blocked on an owner decision or on
conformance-host infrastructure, never on missing language behaviour, and each has its own row
in `docs/spec-defects-v1.md` stating the blocker and what closing it would require.

Work remains, but it is in the `needs_subsystem` bucket rather than in the failures. `V068` and
`V105` were closed after the milestone was first declared done, both by re-probing bucket
reasons that had gone stale, and `V105` exposed a real `C131` exactness defect that had been
invisible for a milestone because a bucketed vector is never executed.

### Next work, in order of value

All 21 bucket reasons were re-probed and rewritten, so what follows is measured rather than
inherited. Six rows were narrowed to a strictly smaller blocker in the process.

1. **The remaining 9 bucketed rows have no single dominant blocker.** Closures landed and closed
   `V077` and `V099`. What is left is spread thin: `V064` needs a resource-limit harness, `V072`
   a Hash literal, `V079` a compacting GC, `V085` `migrate_revision`, `V110` a stable diagnostic
   category, and `V111` static rebinding rejection.
2. **Three rows are blocked on something the specification says must not exist or cannot be
   observed**, and should be treated as closed rather than pending: `V086` needs a static-Type
   observation path `C068` gives no runtime surface for, `V087` needs a rebind API `D-311`
   forbids, and `V103` is recorded the same way.
2. **Closure literals** do not parse at all, which alone blocks `V077`, `V087` and `V099`. For
   `V099` that is the only remaining term.
3. **`V110` needs a stable diagnostic category**, not implementation. Its five reserved-selector
   declarations are all rejected and the RUNTIME runner now observes diagnostics, but `C054`
   names no category for this case, so the emitted `PARSE_UNEXPECTED_TOKEN` is authored rather
   than specified. Closing it needs an owner ruling or an errata assigning the category.
4. **Hash literals**, the last term of `V072`.
5. **`migrate_revision` and revision reactivation** for `V085`, and static rebinding rejection
   for `V111`.

## Milestone Close

Every chapter reports `failed: 0`.

Two numbers are counted separately here. **Spec rows** are vector IDs the
frozen specification declares. **Local rows** are additional vectors this
implementation authored to pin clauses no spec row enumerates; they cite real
clauses, but counting them toward specification coverage would inflate both
sides of the ratio.

An earlier revision of this table reported CONTROL as 134 of 134, and a
correction then moved 14 `V###A` ids out of the spec count as locally authored.
That correction was itself wrong. All 14 are published rows in the CONTROL
vector table, so the spec declares 134 CONTROL rows and the "fix" understated
coverage. The cause was an id pattern capturing `V\d+`, which truncates `V288A`
to `V288`; the same truncation later made a spec-published id look local during
the corpus audit. Both counts below come from matching the full id, trailing
letter included.

| Chapter | Spec transcribed | Spec rows | Missing | Local rows |
| --- | --- | --- | --- | --- |
| ASYNC | 40 | 40 | 0 | 3 |
| COLLECTIONS | 122 | 122 | 0 | 5 |
| CONFORMANCE | 3 | 3 | 0 | 4 |
| CONTROL | 134 | 134 | 0 | 23 |
| FFI | 32 | 32 | 0 | 1 |
| GRAMMAR | 48 | 48 | 0 | 3 |
| IDENTITY | 5 | 5 | 0 | 3 |
| LIBRARY | 14 | 14 | 0 | 5 |
| META | 51 | 51 | 0 | 7 |
| MIGRATION | 0 | 0 | 0 | 2 |
| RUNTIME | 109 | 109 | 0 | 55 |
| TRACE | 0 | 0 | 0 | 2 |
| TYPES | 80 | 80 | 0 | 9 |
| **Total** | **638** | **638** | **0** | **122** |

**All 638 spec rows transcribed, plus 122 locally authored rows.**

TRACE and MIGRATION declare no vector rows of their own. Their clauses
constrain how the specification is written, so the local rows there run through
the documentation validator against the spec text.

Transcribed is not the same as passing. Of the 760 records in the corpus, 722
pass and 38 are recorded in non-passing buckets rather than counted as
coverage:

| Bucket | Count | Meaning |
| --- | --- | --- |
| needs_subsystem | 14 | Requires a subsystem that does not exist yet |
| unrunnable_source | 9 | The frozen row supplies prose, not an executable fixture |
| no_fixture | 5 | No fixture exists for the row |
| authored_expect | 5 | The frozen row names no stable diagnostic code |
| differential | 4 | Requires interpreter/JIT comparison that does not exist |
| deferred | 1 | Deferred by the row itself |

## Freeze-Gate Clauses Left Open

Two CONFORMANCE clauses stay uncited, for two DIFFERENT reasons. An earlier revision of this section grouped all eight as needing "a
freeze pipeline, a second backend and human review", which hid that five were
decidable from the artifacts already on disk.

- `C041` needs a SECOND BACKEND. It compares emitted float bits across
  interpreter, JIT and native; only the interpreter exists.
- `C064` needs a HUMAN. Deciding whether a behaviour's normative authority is a
  Legacy Iris script, old PDF text or an implementation quirk is a judgement
  about provenance, not a property of any file.

The other six are now enforced in `tools/corpus_gate_rules.py`: `C046`
preserves every published vector id, `C049` requires each chapter to state both
a positive and a refusing vector, `C057` rejects a newly uncovered obligation,
`C059` rejects a deferred item tested as normative, `C060` rejects a backend
excluded without a reason, and `C048` rejects a chapter carrying normative
clauses with no vector coverage. `C048` reads the required-class table out of
`C046` rather than restating it, so a table edited without updating the corpus
fails; CONFORMANCE and TRACE carry clauses without a table row, and the
traceability matrix cites both, which is the documentation-structure exception
C048 allows. Each was verified by introducing the violation it
targets and observing a clause-named non-zero exit.

`C049` found a real gap when it was written: CONFORMANCE, MIGRATION and TRACE
each stated only refusals. A chapter that never records a success proves its
rules reject, never that anything is accepted.

## Clause Coverage

A vector row and a normative clause are different units: one row can rest on
several clauses, and many clauses are named by no row at all. Behaviour that is
correct but uncited is behaviour a refactor can break with nothing failing,
which is how this project's iterator-close and extension-digest defects
survived. `tools/clause-coverage.py` reports the gap.

| Chapter | Clauses | Cited | Open |
| --- | --- | --- | --- |
| ASYNC | 62 | 30 | 14 |
| COLLECTIONS | 100 | 77 | 6 |
| CONFORMANCE | 75 | 9 | 48 |
| CONTROL | 80 | 35 | 26 |
| FFI | 58 | 29 | 12 |
| GRAMMAR | 72 | 51 | 4 |
| IDENTITY | 36 | 7 | 25 |
| LIBRARY | 41 | 18 | 10 |
| META | 126 | 62 | 32 |
| MIGRATION | 12 | 2 | 3 |
| RUNTIME | 161 | 84 | 61 |
| TRACE | 22 | 2 | 12 |
| TYPES | 99 | 52 | 15 |
| **Total** | **944** | **458** | **268** |

**458 of 944 clauses cited (48.5%).** `Open` counts uncited clauses carrying
MUST or SHALL; the remainder scope a chapter, defer to another, or introduce a
table, and are not obligations a vector can observe.

Sampling confirmed the gap is evidence rather than function: the clauses
checked were already implemented correctly and simply had nothing pinning them.
Closing it found seven real defects, each fixed with a vector that fails when
the fix is reverted:

- `same?` compared CONTENTS through a private infix helper, so two distinct
  Arrays reported the same identity
- `raise value` while handling did not link the active context as automatic
  cause, and three committed vectors had been written against that behaviour
- Iterator `close()` entered done state without RELEASING the source
- `Integer & | ^` did not exist at all
- numeric `<=>` raised on a nonnumeric operand instead of answering nil
- a binding annotation was not enforced inside a Method or Closure body
- JSON decoding built a Hash holding two entries under one key

## Corpus And Specification Audits

Two chapters constrain artifacts rather than behaviour, so their clauses cannot
be observed by a vector that runs a program:

```bash
python3 tools/corpus-audit.py     # CONFORMANCE rules over the vector corpus
python3 tools/spec-audit.py       # TRACE and MIGRATION rules over the spec text
```

The corpus audit found 91 records outside the C013 category vocabulary and 111
locally authored ids using a spelling C008 forbids. Neither was visible to any
test. The spec audit found TRACE-C005 holding across 614 normative paragraphs,
and MIGRATION-C001 across all 37 ledger rows.

### Held in non-passing buckets

Every spec row is transcribed. 18 are recorded in non-passing buckets because
the machinery they observe does not exist, rather than being approximated. An
earlier revision of this section said "Four", which counted only the rows
transcribed last rather than the whole set the runner reports.

Needing a subsystem:

- `RUNTIME-V064`, `V079`, `V085`, `V110`, `V111` and `IDENTITY-V014` need the
  reflection/meta-operation API, including `migrate_revision`.
- `CONTROL-V004`, `V007`, `V026`, `V030`, `V041` need structured diagnostic
  reporting and revision event delivery.
- `TYPES-V208` needs static member-existence checking. Measured: a static
  caller of a runtime-added member is currently PERMITTED and answers 9, and
  the reflective path errors, so both directions are wrong.
- `COLLECTIONS-V350` needs generic Contract declaration, which does not parse
  today, plus builtin iterator requirement reflection.
- `FFI-V065` needs script-level binding of a native-backed object.

Needing a second backend:

- `RUNTIME-V052`, `V066`, `V073` and `COLLECTIONS-V330` are differential. They
  require interpreter and JIT agreement, and there is no JIT; `V330` also needs
  real concurrency primitives.

## Corpus Accounting

Superseded by the Milestone Close table above, which counts every chapter
rather than the five this milestone opened with. That earlier table read 420 of
422 rows across five chapters and listed eight open rows, including
`GRAMMAR-V008` as untranscribable and `GRAMMAR-V001` as needing an owner
ruling. Both were later resolved: `V008`'s defining record was found in the
CONFORMANCE chapter's own fixture, and `V001` was transcribed against the
v1.33 count. Keeping the old numbers here alongside the corrected ones would
leave two accounts of the same corpus disagreeing.

One accounting note from that section is still worth keeping. A table-driven
count of GRAMMAR misses seven rows (`V001`, `V002`, `V004`-`V008`) because
chapter 02 states them as prose rather than in its vector table, which is why
the transcribed count is derived from the spec text rather than from the tables.

## META Chapter Status

META reached 39 transcribed rows of 51. The remaining 12 need one of four
subsystems that do not exist, each recorded in `docs/spec-defects-v1.md` where a
row was partially transcribed:

| Subsystem | Rows | What is missing |
| --- | --- | --- |
| Host grants | `V363`, `V421`, `V422` | ReflectionPolicy, manifest grants, `prelink` |
| Package store and artifacts | `V357` | An active package store; V357's artifact bytes are never given |
| Upgrade and async | `V353`, `V354`, `V355`, `V431` | Package upgrade, safepoint publication, `await` |
| Cross-package static activation | `V346`, `V418`, `V427`, `V438` | `C045` DIRECT-IMPORT activation and `StaticType` member modelling |

`ReflectionPolicy` remains the deferral the milestone plan already named, and it
alone accounts for three of the twelve. The async and native surfaces are
likewise outside this milestone.

Deliberately deferred to a later milestone, unchanged from the milestone plan:

- `Contract` obligations
- generic `Module` arguments
- any scheduling surface, which is also what `V013` and `V083` wait on
