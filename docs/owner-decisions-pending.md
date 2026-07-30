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

## 2. Should a callable's KIND be statically distinguishable? (open spec question)

**Status:** open for discussion. Nothing implemented; no spec text changed.

The owner observes that under `(P1, ...) -> R` a signature cannot tell whether a
value is a Closure or a BoundMethod, and argues this sits badly with the
`IRIS-V1-IDENTITY-C008` static-promise principle. The owner asked to reopen the
specification on this point rather than settle it in implementation.

### What the frozen material already decides

`IRIS-V1-CONTROL-C018` states that BoundMethod and Closure values SHARE ordinary
callable function types after binding. `D-425` is the owning decision and is
deliberate rather than an oversight: it assigns a DISTINCT reified
`Method<Self, (P...) -> R>`-style type to UNBOUND Methods in the same sentence.
The authors therefore knew how to distinguish callable kinds by type and chose to
do so only for unbound Methods. `D-415` gives the reason: after binding, a
Closure and a BoundMethod present the same invocation contract.

`IRIS-V1-TYPES-C002` additionally forbids raw generic instance types, and
generic arguments are invariant under `C028`/`C031`, so a `Closure<T>` spelling
would not be substitutable with a differently-parameterised one.

### The question the frozen material does NOT answer

`IRIS-V1-IDENTITY-C008` enumerates the static facts that must be preserved:
declared superclass, declared Contracts, visible member names and SIGNATURES,
typed properties, generic constraints, native layout, package identity, API major
identity. It does not name callable KIND. Two readings follow and the
specification does not choose between them:

- Kind is not a promised fact. `(P) -> R` already promises the signature, so
  `C008` is satisfied and `C018` is consistent with the identity principle.
- `C021` makes the three callable kinds a normative table, so kind IS a static
  fact and ought to be statically distinguishable.

### Practical note that may bear on the decision

At a call site the difference is unobservable: both kinds accept arguments and
return a result. Distinguishing kind therefore buys nothing for INVOCATION. It
only matters for a declaration that wants to REJECT one kind, for example a
parameter accepting a Closure but not a method reference. That is a restriction,
not part of a signature, and might be better expressed as a Contract or a
constraint than as a change to callable typing.

### Cost if the owner decides to change it

Reversing `C018` would be the SEVENTH `TRACE-C007` exception this milestone and
the SECOND to reverse a decided semantic. It would also open a gap the current
design does not have: a parameter annotated `Closure<...>` could no longer accept
a BoundMethod, even though `C021` lists BoundMethod as an ordinary callable kind.
Any such errata should state what happens at that boundary.
