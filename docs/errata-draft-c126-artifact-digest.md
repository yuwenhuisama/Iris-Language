# Errata draft: IRIS-V1-META-C126 — the V357 artifact and its digest scope

Status: awaiting owner approval. Nothing published. The artifact file HAS been
created, since the digest cannot be stated without bytes to hash.

## The defect

`IRIS-V1-META-V357` fixes an exact digest:

> `fixtures/meta/v357/artifact.json` has exact BLAKE3-256
> `6e7c0d24c82a8dc03b320c790a7c8c2d4f2f9b77b9635c315d21ea34e92f4601`

The artifact's BYTES are never given anywhere in the specification set, and the
digest's SCOPE is never stated either. `D-271` explicitly defers it:

> Digest algorithm/scope remain separate.

So the stated value cannot be reproduced or checked by any implementation. It is
a constant with no defining input.

## Why the constant must change rather than the bytes be reverse-engineered

An input could be searched for that happens to hash to `6e7c0d24...`. That would
make the row pass and prove nothing: it would demonstrate only that a preimage
was found, not that any digest rule was implemented correctly. Conformance would
be asserted rather than verified.

The honest repair is the opposite direction: define the artifact, define the
scope the row's own behaviour already implies, and state the digest those two
produce.

## The scope the row already implies

V357 requires two things of its variants:

- a variant that changes ONLY the locator preserves the digest, and
- a variant that changes source bytes changes the full 32-byte digest.

Those two together fix the scope: the digest covers the artifact's SOURCE bytes
and not its locator. `D-271` agrees, holding "an immutable artifact locator PLUS
cryptographic digest" as two separate record fields.

## The artifact

`conformance/iris-v1/fixtures/meta/v357/artifact.json`:

```json
{
  "locator": "org.iris.v357/src/owner.ir@1",
  "digest": "b3:7513f737323f64751a41ef3215b226725c0eef9c54d89918085ad5b0ee9a9360",
  "source": "class Owner {\n  public fun status() -> Symbol { :ok }\n}\n"
}
```

The source declares the Method `status` the row names as currently required, so
the "omit current required Method `status`" variant is expressible against it.

## Verified before proposing

| Property the row states | Result |
| --- | --- |
| Digest of the artifact | `7513f737323f64751a41ef3215b226725c0eef9c54d89918085ad5b0ee9a9360` |
| Locator-only change preserves the digest | Holds, byte-identical |
| Source change alters the full 32 bytes | Holds, 0 of 32 bytes retained |

## Proposed clause

`IRIS-V1-META-C126`: The v1.31 errata fixes the digest scope `D-271` left
separate and corrects the `IRIS-V1-META-V357` artifact digest. A lightweight
audit record's cryptographic digest is BLAKE3-256 over the referenced
artifact's SOURCE BYTES, and not over its locator or any surrounding record
framing, which is what lets `V357`'s locator-only variant preserve the digest
while a source-byte change alters all 32 bytes. The `V357` artifact is
`conformance/iris-v1/fixtures/meta/v357/artifact.json`, whose source is
`class Owner {\n  public fun status() -> Symbol { :ok }\n}\n` and whose digest
under this scope is
`7513f737323f64751a41ef3215b226725c0eef9c54d89918085ad5b0ee9a9360`. The value
`6e7c0d24c82a8dc03b320c790a7c8c2d4f2f9b77b9635c315d21ea34e92f4601` stated in the
`V357` row is WITHDRAWN as unreproducible: no artifact bytes were ever published
for it, so it names no computable quantity. This clause changes no behavioural
requirement of `V357`, `D-270` or `D-271`; every variant and outcome the row
states is unchanged.

## Why this is permitted

`IRIS-V1-TRACE-C021` lets an errata "record an inference rule an implementation
must otherwise guess". The digest scope is precisely such a rule, and the
constant depends on it. Withdrawing an unreproducible constant removes no
requirement and weakens no guarantee: the row's behaviour is stated by its
variants, all of which are preserved and now checkable.

## Asks

1. Approve publishing `IRIS-V1-META-C126` (EN + zh-cn, v1.30 → v1.31).
2. Confirm the digest scope is the artifact's source bytes, or name a different
   scope, in which case the constant changes with it.
