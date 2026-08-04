# Errata draft: V343's rejection wording

Status: DRAFT, awaiting owner approval under `IRIS-V1-TRACE-C019`.
Proposed revision: v1.24 to v1.25.
Blocked row: `IRIS-V1-META-V343`.

This document is a proposal. Nothing has been published to `spec/iris-v1/`, and
no implementation change has been made.

## The gap

`IRIS-V1-META-V343` states its observable as:

> The definition is rejected if it captures `local`; no Box revision is
> committed.

Its applicability row requires the compiler, so "rejected" reads as a
compile-phase rejection. Both halves diverge from the clauses that govern them.

### Half 1: "the definition is rejected"

`IRIS-V1-META-C026` states the semantic rule:

> A Method declaration inside an executable Class or Module body does not close
> over lexical locals created by that body execution.

It says the Method **does not capture**. It does not say the declaration is
rejected.

`IRIS-V1-CONTROL-C011` then governs what happens when the body reads a name
nothing binds:

> A non-call unresolved bare `name` MUST **raise or diagnose** `NameError`

That is an explicit either/or. Raising at the call is fully conforming.
Measured:

| Program | Result |
| --- | --- |
| the declaration alone | accepted |
| `Box.new().value()` | `NameError` |

So the implementation already satisfies `C026` and `C011`. `V343`'s wording
requires the *diagnose* branch specifically, which no clause mandates.

### Half 2: "no Box revision is committed"

Measured: the `value` slot IS published, because the declaration is accepted.
This half follows from half 1 — it only holds if the definition is rejected at
declaration time.

`IRIS-V1-META-C022` publishes nothing from a *failed* candidate, but this
candidate does not fail: capturing nothing is `C026`'s specified outcome, not an
error.

## Why this is not simply an implementation gap

Satisfying `V343` as written means adding general unresolved-name checking to
the analyser. I investigated that and recorded it as not viable without a
prerequisite:

- `StaticType` models only eight built-in members; it gained nominal Class
  modelling and superclass edges this session, but no member table.
- A name check would have to model reflective opens, `define_method`, and
  mixin-contributed Methods, all of which this codebase creates at runtime.
- `StaticType`'s own contract says an unmodelled Type stays `None` at the call
  site "since a wrong rejection is far worse than a missed one".

So the *diagnose* branch is reachable only behind a substantial analyser pass
that would produce wrong rejections until it models every runtime member source.

## Proposed clause

Next available META clause number, published in both `spec/iris-v1/` and
`spec/iris-v1/zh-cn/` in the same revision per `IRIS-V1-TRACE-C020`:

> IRIS-V1-META-C121: The v1.25 errata fixes how a Method declaration that reads
> a body local is reported. IRIS-V1-META-C026 makes such a Method not close over
> that local, and IRIS-V1-CONTROL-C011 makes an unresolved bare name raise OR
> diagnose `NameError`; either branch conforms. A conforming implementation
> therefore MAY accept the declaration and raise `NameError` when the Method
> runs, and MUST NOT be required to reject the declaration statically. The
> containing Class publishes normally, since IRIS-V1-META-C022 withholds
> publication only from a candidate that FAILED, and not capturing is C026's
> specified outcome rather than a failure. This supersedes in place the
> `IRIS-V1-META-V343` expectation that the definition is rejected and that no
> revision is committed.

## What the row becomes

With `C121` published, `V343` is transcribable against what `C026` and `C011`
actually require:

- the declaration is accepted;
- calling the Method raises `NameError`, so the local was NOT captured;
- a Method reading nothing raises nothing, which distinguishes the two.

I would transcribe it that way and tamper-test both directions.

## Alternative

If you would rather keep `V343`'s wording, the row stays blocked on the
analyser pass described above, and I would record it in
`docs/spec-defects-v1.md` rather than publish this clause. That is the more
conservative option: it leaves the frozen expectation intact and defers the
work.

## What I need from you

1. Approve publishing `IRIS-V1-META-C121`, EN and zh-cn together, bumping v1.24
   to v1.25.
2. Confirm the supersede is the right call rather than keeping the row blocked.

I have NOT taken either path without your decision.
