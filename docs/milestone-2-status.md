# Iris v1 Milestone 2 — Implementation Status

**Revision:** Iris v1.34 (frozen semantics with owner-approved errata)
**Branch:** `new-iris-dev`
**Baseline of this milestone:** `3fa54ab`

## Current Conformance

```
COLLECTIONS passed: 97, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0 (97 records, buckets sum 97)
FFI         passed: 3, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0   (3 records, buckets sum 3)
ASYNC    passed: 39, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0      (39 records, buckets sum 39)
RUNTIME  passed: 96, failed: 0, needs_subsystem: 5, no_fixture: 5, differential: 3   (109 records, buckets sum 109)
CONTROL  passed: 129, failed: 0, needs_subsystem: 5                                  (134 records, buckets sum 134)
TYPES    passed: 79, failed: 0, needs_subsystem: 0, no_fixture: 0, differential: 0     (79 records, buckets sum 79)
META     passed: 51, failed: 0, needs_subsystem: 0                                  (51 records, buckets sum 51)
GRAMMAR  passed: 32, failed: 0, deferred: 1, authored_expect: 5, unrunnable_source: 9  (47 records)
```

Milestone 2 opened at RUNTIME 41 and closed at 94.

```bash
cargo run -p iris-conformance -- --chapter RUNTIME
cargo run -p iris-conformance -- --chapter GRAMMAR
cargo run -p iris-conformance -- --chapter CONTROL
```

## Quality Gates

All four must pass before any commit. Capture exit codes without piping first — a pipe replaces `$LASTEXITCODE` with the filter's code, which once hid a real clippy failure.

```bash
cargo test --workspace
cargo test -p iris-lexer        # 44 tests, must finish instantly
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

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

## Defects Found Without Vector Coverage

Real defects surfaced through probing rather than through the corpus.

- **Array and Hash were not identity-bearing.** `IRIS-V1-COLLECTIONS-C003`
  classifies `Array<T>` and `Hash<K,V>` as *identity-bearing* with mutable
  contents. Both were stored by value, so every binding, argument pass and
  field read COPIED. **Array is fixed** in `841c085`: it now carries a shared
  body, which also deleted the syntax-routed `append` workaround and made
  `C026` fail-fast iteration possible. **Hash is fixed** in `4c0670b`, with `C034`
  structural versioning that excludes value updates.
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

Six chapters, 435 of 462 vector rows, every chapter reporting `failed: 0`.

| Chapter | Transcribed | Rows |
| --- | --- | --- |
| RUNTIME | 109 | 109 |
| GRAMMAR | 47 | 48 |
| CONTROL | 134 | 134 |
| TYPES | 79 | 80 |
| META | 51 | 51 |
| ASYNC | 39 | 40 |
| COLLECTIONS | 97 | 122 |
| FFI | 3 | 32 |
| LIBRARY | 0 | 14 |
| IDENTITY | 0 | 5 |
| CONFORMANCE | 0 | 3 |
| **Total** | **554** | **638** |

Three chapters are complete. The 27 open rows are NOT spread thin: they group
into subsystems that each need an external boundary this milestone deliberately
did not invent.

### Blocked on an external completion source

RESOLVED. `Gate` supplies the external completion post `C014` names, so `V006`, `V081` and `V084` are transcribed. What still needs a driver-level surface is `V010` unobserved-failure diagnostics and `V012` cooperative cancellation. Original text: these rows need a genuinely INCOMPLETE Awaitable.
Nothing in this milestone can produce one: `IRIS-V1-ASYNC-C012` completes every
async body synchronously when no incomplete await is reached, so `C014`'s FIFO
continuation queue has no input and suspension across `using` or `for` cannot be
triggered. Building the queue anyway would have produced code no vector could
falsify.

### Blocked on their own subsystems

`V019`-`V025` and `V028` need structured diagnostic reporting, revision event
delivery, `GapEvent`, and audit history recovery. `V073`-`V084` continue the
same surfaces. Each is comparable in size to the `using` work this milestone
closed with.

### Remaining single rows

- `GRAMMAR-V008`: no defining row in chapter 02, recorded since milestone 1.
- `TYPES-V208`: blocked on static member-existence checking, recorded.

### Errata published this milestone

Nine clauses, each closing a gap where a published requirement had no spelling:
`TYPES-C098`, `TYPES-C099`, `CONTROL-C080`, `META-C125`, `META-C126`,
`GRAMMAR-C071`, `GRAMMAR-C072`, `ASYNC-C063`, and the v1.21-v1.27 set. The
reserved keyword inventory was widened once more, to 50, for `yield`.

### What the engine does and does not do

Generators and async bodies are stackless: a generator is re-entered on each
`next()` rather than resumed on a captured native stack. Ordinary synchronous
evaluation is UNCHANGED, which `IRIS-V1-ASYNC-C011` requires by forbidding
preemption between suspension boundaries. Synchronous recursion still uses the
native stack and its existing depth budget; making that stackless was considered
and rejected as outside what v1 specifies.

## Corpus Accounting

The five chapters this milestone covers hold 422 vector rows, not the 415 an
earlier count in this document used. That figure omitted seven GRAMMAR rows
(`V001`, `V002`, `V004`-`V008`) which are stated in chapter 02 as prose rather
than in the chapter's vector TABLE, so a table-driven count missed them.

| Chapter | Transcribed | Rows |
| --- | --- | --- |
| RUNTIME | 109 | 109 |
| GRAMMAR | 47 | 48 |
| CONTROL | 134 | 134 |
| TYPES | 79 | 80 |
| META | 51 | 51 |
| **Total** | **420** | **422** |

Four chapters are complete. The eight open rows are:

- `GRAMMAR-V005`, `V006`, `V007` are transcribed. `V007` needed
  `PARSE_BAD_PARAMETER_ORDER`, which the grammar's `parameter_sequence`
  requires and nothing enforced.
- `GRAMMAR-V004` is transcribed. It needed two parser gaps closed: a Regex
  literal had no primary production, and the lexer recognised `?=` but not `!=`
  as a setter selector, both of which `IRIS-V1-GRAMMAR-C019` and `C023` require.
- `GRAMMAR-V002` is transcribed. It needed `=~` and `!~`, two of C016's fixed
  spellings that the lexer never produced.
- `GRAMMAR-V001` is transcribed against the v1.33 count of 50. Its stated
  "exactly 48
  reserved keywords" while `IRIS-V1-GRAMMAR-C013` fixes the count at 49 after
  the v1.1 `typeof` errata, so the row and the clause it cites disagree and the
  row needs an owner ruling before it can be transcribed against either
  number.
- `GRAMMAR-V008`: recorded in `docs/spec-defects-v1.md` since milestone 1 as
  having no defining row in chapter 02.
- `TYPES-V208`: blocked on static member-existence checking, recorded.

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
