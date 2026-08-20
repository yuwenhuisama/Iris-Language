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

Four questions were raised during autonomous work and all four were ruled on
rather than left pending:

- **Registry unification.** The owner chose to merge, which made `Object` the
  implicit root and closed `V075`, `V076` and `V100`.
- **Callable kind in the Type system.** The owner's `Block<S>` union answered the
  objection that a bare `Closure<T>` could not accept a BoundMethod. Published as
  the v1.11 errata, so `Closure<S>` and `BoundMethod<S>` now reify the kind while
  the block channel still accepts either.
- **`D-266` naming a reserved keyword.** Resolved by the v1.13 errata under
  option A: `IRIS-V1-RUNTIME-C164` restates the signature as
  `migrate_revision(source, target)`, changing only the two parameter names
  rather than widening the grammar to admit reserved words in parameter
  position.

Each outcome is recorded in `docs/spec-defects-v1.md`.

- **Moving vs non-moving GC.** The design review recommends a non-moving
  mark-sweep collector for the first version and lists moving GC among the
  things the first JIT explicitly does not do. The implemented collector
  RELOCATES surviving objects, which is a divergence rather than an oversight.
  The owner accepted it and directed that it not be reverted. It is safe today
  because the identity hash is a stored field assigned at allocation and never
  derived from an address, so `D-111`'s requirement that a hash survive movement
  by GC holds by construction, which `IRIS-V1-RUNTIME-V079` observes directly.
  The obligation it creates is on future work: a JIT that caches an object
  address must cooperate with relocation, so the points at which compaction may
  run have to stay explicit. Recorded in `docs/iris-ir.md` section 7.

- **The legacy VM as a reference.** The owner directed that the previous C++
  implementation is NOT to be adopted or carried over in any part. The new
  execution IR is designed from the design review's recommendations rather than
  ported, and the legacy implementation appears in `docs/iris-ir.md` only as a
  record of defects to avoid repeating - the missing opcode source of truth, the
  unverified operand reads, and the trusting `.irc` reader.

