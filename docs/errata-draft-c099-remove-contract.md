# Errata draft: IRIS-V1-TYPES-C099 — the spelling that removes a declared Contract

Status: awaiting owner approval. Nothing published.

## The gap

`IRIS-V1-TYPES-V200` states its fixture and outcome plainly:

> Metadata fixture: `Class A for C`; candidate removes `C`.
> `TypeContractError` before commit; `A` still conforms to `C`.

`D-173` and `IRIS-V1-TYPES-C045` both presuppose that a candidate CAN attempt
this, since they exist to say the attempt must be refused:

> Metaprogramming cannot add, remove, rename, weaken, or incompatibly replace
> declared Contract facts in place.

But the specification supplies no source or reflective spelling for removing a
declared Contract at all. `IRIS-V1-META-C097` lists `contracts` on the Class
reflection view as a READ, and the programmatic surface has `define_method`,
`define_property`, `remove_method`, `remove_module`, `add_module` and
`set_superclass` — nothing that reaches declared conformance.

So the clause forbids an operation the language cannot express, and the row that
observes the refusal cannot be authored. That silence is the defect.

## Why a spelling must exist rather than the row being dropped

`C045` names FOUR entry points that must validate the static spine before
publication: candidate revisions, opens, package upgrades, native metadata, and
reflection construction. Three of those already exist and are checked. If no
spelling can reach declared conformance, the `remove` term in both `D-173` and
`C045` is unreachable prose, and `V200` tests nothing.

The alternative — deleting the term — is not available: `IRIS-V1-TRACE-C007`
makes published content permanent, and `TRACE-C021` forbids reinterpreting a
decided semantic.

## Proposed clause

`IRIS-V1-TYPES-C099`: The v1.30 errata fixes the spelling that attempts to remove
a declared Contract from a Class. A transaction candidate MAY attempt it as
`remove_contract(contract)`, the reflective form being
`Reflection::Class.remove_contract(target, contract)`, which
`IRIS-V1-META-C119` makes one implementation with the Class-level entry point
exactly as it does for `set_superclass`. The attempt is a candidate mutation
under `IRIS-V1-META-C023`, so it targets the CURRENT transaction candidate and
requires the `modules` capability no more than `define_method` does; capability
gating for it is `method_set`'s sibling `shape` under `IRIS-V1-META-C073`.

`IRIS-V1-TYPES-C045` makes declared Contract conformance IMMUTABLE for a
revision's static spine, so the attempt MUST be rejected with
`TypeContractError` BEFORE publication, and the target MUST retain its declared
Contract set, its active revision, and its conformance. This clause supplies
only the SPELLING the refusal needs in order to be observable; it grants no
ability to remove a declared Contract and reverses no published outcome. A
different declared Contract set requires a distinct Class revision or
declaration through the explicit class-evolution system, exactly as `D-173`
already states.

## What this makes reachable

`TYPES-V200` becomes transcribable end to end: the candidate attempts the
removal, the refusal is `TypeContractError` before commit, and `A.contracts`
still reports `C` afterwards.

## Risk to probe before publication

Whether any currently passing row already sends a selector named
`remove_contract`, which the new spelling would capture. I will run the full
corpus and report before transcribing.

## Asks

1. Approve publishing `IRIS-V1-TYPES-C099` (EN + zh-cn, v1.29 → v1.30).
2. Confirm the spelling `remove_contract` and its refusal-only semantics, or
   name a different spelling.
