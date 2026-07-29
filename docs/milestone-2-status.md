# Iris v1 Milestone 2 — Implementation Status

**Revision:** Iris v1.9 (frozen semantics with owner-approved errata)
**Branch:** `new-iris-dev`
**Baseline of this milestone:** `3fa54ab`

## Current Conformance

```
RUNTIME  passed: 57, failed: 3, needs_subsystem: 28, no_fixture: 18, differential: 3   (109 records, buckets sum 109)
GRAMMAR  passed: 26, failed: 0, deferred: 1, authored_expect: 5, unrunnable_source: 9  (41 records)
```

Milestone 2 opened at RUNTIME 41 and closed at 57.

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

## Remaining Failures

None is an implementation gap.

| Vector | Blocker |
| --- | --- |
| `V103` | The expectation is prose the runner cannot compare. Its "no implicit backing storage" claim also lacks an observation path, since `list_ivars` does not accept a value receiver. |
| `V013` | Needs a host that can pause an entered frame, commit, then resume. `01-language-identity.md` lists suspension inside a transaction as `PROHIBITED`, so this is conformance harness work, not a language feature. |
| `V083` | Needs the same harness to interleave a commit with a construction. |

`docs/spec-defects-v1.md` holds 23 rows, 8 resolved, and records the blocker for every one.

## Working Agreements

- The specification is frozen. A discrepancy goes in the defect ledger, never into the spec, unless the owner approves an errata under `TRACE-C021`.
- A conformance expectation is evidence. It is never weakened, reclassified or special-cased to make a result look better. An honest failure with a precise blocker is worth more than a forced pass.
- Never invent a selector spelling. Search the whole specification first, including vector-table fixture rows and `traceability-matrix.md` — `append` and the `alias_method` family both turned out to live there after being reported as unsupplied.
- `crates/iris-lexer/` changes need care. A rewrite there once introduced a non-terminating loop that killed the suite with a 64 GiB allocation.
- Only `num-bigint` and `blake3` are approved dependencies. JSON stays hand-rolled.
- Verify claims by probing through `iris_eval::evaluate` rather than by reading code. Most defects in this milestone were found that way.

## Frozen Invariants

- `Integer(1).hash` is `17824117788395916856`; the singleton hash is `11850167709044604115`.
- `IRIS-V1-RUNTIME-C067` rejects `other.@x`, `obj.@@x` and `A.@@x`. The owner has reaffirmed that this form must never be valid syntax.
- Built-in value classes may not have their runtime superclass changed, per `RUNTIME-C150`.
- Record counts stay at 109 RUNTIME and 41 GRAMMAR, and the buckets must sum to the record count.
