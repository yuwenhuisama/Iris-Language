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

## 1. Unifying the two ClassRegistry instances (architecture, not semantics)

**Status:** needs a ruling on approach, not on language meaning.

`SourceEvaluator` holds a `Runtime` and a `Kernel`, and each owns a separate
`ClassRegistry`. Declared Classes live in the `Runtime` registry; built-in
Classes live in the `Kernel` registry. Their `ClassId` spaces are disjoint, so an
id from one is meaningless in the other.

This blocks `Object` as the implicit root Class, which in turn blocks
`IRIS-V1-RUNTIME-V076` (one term away from passing) and `IRIS-V1-RUNTIME-V100`
(cannot pass at all). Naming the kernel `Object` as a declared Class's
superclass currently resolves to an unrelated id and surfaces as
`UnknownClassId(ClassId(0))`.

The language semantics need no ruling. `IRIS-V1-RUNTIME-C005` makes `Object` the
sole root, chapter 03 line `505` writes `class C extends Object mixin A, B {}`,
`class_extends?` is optional in the chapter 02 grammar, and the frozen `V100`
row expects `[A, NewBase, Object]` for a Class that never wrote `Object`
explicitly. What needs a decision is how much surgery to authorize:

- **A. Unify into one registry.** Kernel and Runtime share a single
  `ClassRegistry`. Cleanest end state and the only one where built-ins and
  declared Classes can genuinely share an MRO. Touches the `Kernel` and
  `Runtime` constructors and every call site that assumes its own registry.
- **B. Bridge the id spaces.** Keep both registries and translate ids at the
  boundary. Smaller diff, but it leaves two sources of truth for Class identity
  and would need care wherever an MRO mixes built-in and declared entries.
- **C. Leave it.** `V076` and `V100` stay blocked, and the ancestry of every
  declared Class keeps terminating at itself rather than at `Object`, which
  contradicts `C005` in observable behaviour.

The risk in A and B is the same: `V011` through `V019` already lock MRO
invariants, and a hasty change there would break passing vectors. That is why
this was recorded rather than attempted.

The kernel-side half is already committed and harmless: `Object` is registered
as a sixth `BuiltinClass` with its protocol selectors installed. It is simply
not reachable from source yet.

