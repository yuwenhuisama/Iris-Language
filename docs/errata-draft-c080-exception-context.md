# Errata draft: IRIS-V1-CONTROL-C080 — naming `ExceptionContext` for getter replacement

Status: awaiting owner approval. Nothing published.

## The gap

Two published clauses state that the getter may be replaced.

`IRIS-V1-CONTROL-C065`:

> Public getters may be dynamically replaced for ordinary property reads, but
> runtime unwinding, diagnostics, native bridges, and uncaught formatting MUST
> use the protected internal records.

`D-143`:

> The public `ExceptionContext.suppressed` getter may be dynamically replaced or
> removed, affecting ordinary Iris property reads only. The actual runtime chain
> remains an unforgeable internal channel.

`IRIS-V1-CONTROL-V286` observes exactly that replacement: replace the getter
with one returning `[]`, then confirm `context.suppressed == []` while the
uncaught diagnostic payload still carries the protected suppressed context with
`value == :close`.

But `ExceptionContext` is not reachable as a Class. The bare name raises
`NameError`, `open class ExceptionContext` is unsupported, a context value
answers no `class`, and `Reflection::Class.method(ExceptionContext, :suppressed)`
cannot resolve it. The replacement two clauses authorize has no entry point.

## What is NOT missing

The protected half is already implemented and covered. The runtime chain is a
separate internal channel, `suppressed` is a read-only view, and
`IRIS-V1-CONTROL-V285` observes it rejecting mutation. Only the REPLACEMENT half
is unreachable, so this errata supplies a name rather than a new semantic.

## Proposed clause

`IRIS-V1-CONTROL-C080`: The v1.30 errata fixes the surface through which the
replacement `IRIS-V1-CONTROL-C065` and `D-143` already authorize is performed.
`ExceptionContext` is a nameable built-in Class, resolvable as an ordinary name
and as a reflection target, so its public getters may be replaced through the
`IRIS-V1-META-C022` transaction model exactly as any other Class member is.
Replacing or removing a public getter affects ORDINARY property reads only.
Runtime unwinding, diagnostics, native bridges and uncaught formatting MUST
continue to read the protected internal records, which no getter replacement can
reach or forge, as `C065` and `D-143` already require. The collections those
records expose remain read-only under `D-142`, so a replacement changes which
value an ordinary read returns and never what the runtime itself observes.

This clause names an existing Class and reverses no published outcome. It grants
no ability to alter the runtime chain, and `IRIS-V1-CONTROL-C066` continues to
forbid fabricated propagation metadata.

## What this makes reachable

`CONTROL-V286` becomes transcribable: the ordinary read returns `[]` after the
replacement, while the protected record still carries the `:close` suppressed
context, which is precisely the separation `D-143` exists to state.

## Risk to probe before publication

Two things, both of which I will run and report before transcribing.

1. Whether naming `ExceptionContext` collides with any existing binding or
   changes any currently passing row, since `class_name` currently reports
   `"ExceptionContext"` for a context value.
2. Whether the replacement can reach the protected records through any path. If
   it can, the clause is not safe to publish as written, since `D-143`'s whole
   point is that it cannot.

## Asks

1. Approve publishing `IRIS-V1-CONTROL-C080` (EN + zh-cn, v1.29 → v1.30).
2. Confirm that naming the Class is the intended remedy, rather than a dedicated
   reflection entry point that does not make `ExceptionContext` generally
   nameable.
