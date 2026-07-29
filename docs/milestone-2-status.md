# Iris v1 Milestone 2 — Implementation Status

**Revision:** Iris v1.9 (frozen semantics with owner-approved errata)
**Branch:** `new-iris-dev`
**Baseline of this milestone:** `3fa54ab`

## Current Conformance

```
RUNTIME  passed: 59, failed: 3, needs_subsystem: 26, no_fixture: 18, differential: 3   (109 records, buckets sum 109)
GRAMMAR  passed: 26, failed: 0, deferred: 1, authored_expect: 5, unrunnable_source: 9  (41 records)
```

Milestone 2 opened at RUNTIME 41 and closed at 59.

```bash
cargo run -p iris-conformance -- --chapter RUNTIME
cargo run -p iris-conformance -- --chapter GRAMMAR
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

### Permanence exceptions

`IRIS-V1-TRACE-C007` makes published content permanent. Four bounded exceptions were authorized:

1. `META-C100` — the four raw-ivar paths moved under `Reflection::Object` (v1.5)
2. `V100` — `reflection.ancestors` respelled as `Reflection::Class.ancestors` (v1.6)
3. `V067` — the "differs" assertion corrected (v1.7)
4. `V015` — the expected error corrected (v1.7)
5. `MIGRATION-C003` — the archived-document path repointed after the move (v1.9)

Each changed only a spelling or an unsatisfiable assertion, never a name, argument, value or decision. Ledger rows 21 and 23 record the first two and note that two uses make this a precedent rather than a one-off: weigh any further use against `TRACE-C007` itself, not against these rows.

## Defects Found Without Vector Coverage

Three real defects surfaced through probing rather than through the corpus.

- **Construction swallowed a raise.** `A.new()` returned an instance even when `initialize` raised. Fixed in `28acea8`.
- **Self-sends from `initialize` failed.** Construction moved the runtime out of the evaluator before running the initializer, so a nested send resolved against an empty runtime; a separate path then erased the failure into a raise of `nil`. Fixed in `39c576e`. This regressed in `28acea8`.
- **`instance_state` was never enforced.** The capability name was accepted but no check existed, so a denying class could still gain a slot. Fixed in `3773f7a`.
- **Ordered numeric comparison was not exact.** `Numeric::compare` rounded both operands to `f64` while `Numeric::equal` already compared exactly, so an `Integer` beyond finite float range became an infinity and `Float64.infinity > 10 ** 1000` returned `false` against `RUNTIME-C131`. Found by re-probing a `needs-subsystem` vector whose bucket reason had gone stale, which is why no vector caught it.

## Remaining Failures

Milestone 2 is closed as delivered at 59 of the 62 runnable RUNTIME vectors. None of the three
remaining failures is an implementation gap, and none can be closed by writing more code.

| Vector | Blocker |
| --- | --- |
| `V103` | Its "no implicit backing storage" claim has no observation path, and that is required rather than missing. `Float64` is identity-less under `RUNTIME-C004`, and `META-C112` requires raw ivar reflection to respect identity-less restrictions, so `list_ivars` MUST refuse a value receiver. Closing it needs an owner-approved re-authoring of the row, which was offered and declined, or a new specified way to observe value-receiver storage. |
| `V013` | Needs a host that can pause an entered frame, commit, then resume. `01-language-identity.md` lists suspension inside a transaction as `PROHIBITED`, so this is conformance harness work, not a language feature, and no errata can reach it. |
| `V083` | Needs the same harness to interleave a commit with an in-flight construction. |

The revision-capture semantics `V013` and `V083` describe are implemented and exercised by
neighbouring vectors; what is absent is an out-of-band frame scheduler in the conformance host.
Building it is milestone-3 sized infrastructure.

`docs/spec-defects-v1.md` holds 29 rows, 11 resolved, and records the blocker for every one.

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
- Record counts stay at 109 RUNTIME and 41 GRAMMAR, and the buckets must sum to the record count.

## Milestone Status

**Milestone 2 is closed as delivered.** RUNTIME conformance advanced from 41 to 59 across the
milestone. The three FAILING vectors listed above are each blocked on an owner decision or on
conformance-host infrastructure, never on missing language behaviour, and each has its own row
in `docs/spec-defects-v1.md` stating the blocker and what closing it would require.

Work remains, but it is in the `needs_subsystem` bucket rather than in the failures. `V068` and
`V105` were closed after the milestone was first declared done, both by re-probing bucket
reasons that had gone stale, and `V105` exposed a real `C131` exactness defect that had been
invisible for a milestone because a bucketed vector is never executed.

### Next work, in order of value

1. **Unify the two `ClassRegistry` instances.** `SourceEvaluator` holds a `Runtime` and a
   `Kernel` that each own one, so their `ClassId` spaces are disjoint. This single blocker holds
   `V075`, `V076` and `V100`. `Object` is already registered as a sixth `BuiltinClass` with its
   protocol selectors installed; it is simply unreachable from source. Needs an owner ruling on
   how much surgery to authorize, recorded in `docs/owner-decisions-pending.md`.
2. **`@@x` hierarchy binding cells** shared across a subclass, with redeclaration rejected, for
   `V091`.
3. **Ordered stored-property initialization** across the MRO during construction, for `V092`.
4. **Built-in Class openness**: Module composition into `Nil` and Method addition to `Bool` and
   `Integer`, for `V101`.
5. **Hash literals**, for `V072`.

Deliberately deferred to a later milestone, unchanged from the milestone plan:

- `ReflectionPolicy` permission enforcement
- programmatic `Module` include and removal
- `Contract` obligations
- generic `Module` arguments
- any scheduling surface, which is also what `V013` and `V083` wait on
