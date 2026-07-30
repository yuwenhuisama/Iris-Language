# Parked Owner Decisions

Questions raised during autonomous work that need an owner ruling before they can
be acted on. Nothing here has been implemented. Each entry states what was found,
why it cannot be settled without a decision, and what the options are.

Resolved entries are struck from this file and recorded in
`docs/spec-defects-v1.md` instead, which remains the permanent ledger.

## Standing constraints these questions are weighed against

- The v1 specification is frozen. A discrepancy is recorded, never silently patched.
- `IRIS-V1-TRACE-C007`: clause, example and vector IDs are permanent once published.
- A conformance expectation is evidence. It is never weakened to make a result pass.
- An errata supplies a missing spelling or production for behaviour another clause
  already requires; it never reinterprets a decided semantic.

---

_No open questions at the time of writing._

The registry unification recorded here was ruled on by the owner, who chose to
merge. It is implemented and its outcome is in `docs/spec-defects-v1.md`.

_No open questions at the time of writing._

Two questions were resolved rather than left pending. The callable-kind question
was answered by the owner's `Block<S>` union and published as the v1.11 errata,
so `Closure<S>` and `BoundMethod<S>` now reify the kind while the block channel
still accepts either. The `D-266` reserved-keyword conflict was resolved by the
v1.13 errata under option A: `IRIS-V1-RUNTIME-C164` restates the signature as
`migrate_revision(source, target)`, changing only the two parameter names.
Both outcomes are recorded in `docs/spec-defects-v1.md`.
