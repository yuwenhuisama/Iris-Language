# Errata draft: IRIS-V1-ASYNC-C050 — the Host drive surface tests use

Status: awaiting owner approval. Nothing published.

## The gap

`IRIS-V1-ASYNC-C003` makes an async Method return `Task<T>`, `C008` types
`await expr` through `Awaitable<T>`, and `C003`/`C004` give `await` meaning only
inside an async Method or async Closure. `C015` then closes the loop:

> Iris code has no implicit blocking wait, hidden synchronous Task join,
> sleep-until-completion primitive, or property read that blocks until a Task
> completes. Host APIs MAY drive or run the event loop according to the native
> chapter, but that Host control surface is not an Iris source-level blocking
> wait.

Together these mean pure Iris source can NEVER observe a Task's value:
unwrapping needs `await`, `await` needs an async body, and an async body itself
returns a `Task`. Every escape is another Task.

That is correct language design and this errata does not change it. But it
leaves rows like `IRIS-V1-ASYNC-V001` unauthorable, whose stated observation is
"invocation returns `Task<Integer>` and **await returns 1**". There is no
conforming way to reach the `1`.

## What already exists

`C015` names the Host control surface but gives it no spelling. `C049` goes
further and makes one MANDATORY:

> The runtime MUST provide a semantic flush or wait surface **for tests and
> controlled shutdown**.

`C049` scopes its own wording to revision events. The same surface is what a
conformance row needs to observe a Task, and requiring one surface for revision
events while leaving Task observation unreachable is the gap.

## Proposed clause

`IRIS-V1-ASYNC-C050`: The v1.34 errata fixes the spelling of the Host drive
surface `IRIS-V1-ASYNC-C015` names and `C049` already requires. The surface is
`Host.run(task)`, which drives the IrisRuntime scheduler until `task` completes
and answers its awaited result, or re-raises its captured `ExceptionContext`.
It is a HOST control surface and NOT an Iris source-level blocking wait: it is
unavailable inside an async body, inside a Closure, and inside an open or
revision transaction, so it cannot be used to build the implicit blocking wait,
hidden Task join, or sleep-until-completion primitive `C015` forbids. Driving a
already-complete Task answers immediately without enqueueing a continuation, as
`C013` requires of any await. This clause supplies a spelling only: Task
identity, completion, exception capture, and continuation ordering are unchanged
and remain owned by `C006`, `C012`, `C013`, `C014` and `C016`.

## What this makes reachable

`V001`, `V002`, `V005`, `V008` and `V009`, each of which states an observation
about an awaited RESULT. Rows that observe placement, signature shape or
suspension refusal are already transcribed and do not need it.

## Risk to probe before publication

Whether `Host.run` inside an async body, a Closure, or a transaction is
genuinely refused rather than merely undefined; a surface that leaks into those
positions WOULD be the blocking wait `C015` forbids. I will probe all three and
report before transcribing.

## Asks

1. Approve publishing `IRIS-V1-ASYNC-C050` (EN + zh-cn, v1.33 → v1.34).
2. Confirm the spelling `Host.run(task)` and its restriction to non-async,
   non-transaction positions, or name a different surface.

## Recorded regardless

Four rows were REMOVED from the corpus while drafting this. `V001`, `V002`,
`V005` and `V009` had been transcribed with a top-level `await`, which
`ASYNC-V007`'s own placement rule rejects. They passed only because the runtime
observation path does not run static analysis, so they were asserting behaviour
the compiler must refuse. A vector that cannot fail is worse than a missing one,
so they are withdrawn until this errata makes them expressible.
