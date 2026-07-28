# Errors and Resources

This chapter covers failure and cleanup. Iris can raise any object, not just a special error base class. The runtime keeps propagation metadata in an `ExceptionContext`. `try`, `catch`, and `finally` are value-producing control forms, `using` is an ordinary helper built around `Closeable`, and async code uses single-runtime `Task<T>` values with `await`.

```iris
try {
  raise :failed
} catch value: Symbol, context {
  context.value          // :failed
  context.suppressed     // runtime-owned ExceptionContext list
  raise value from context
}
```

This snippet is reused from `IRIS-V1-CONTROL-EX010`. The raised object is the `Symbol` `:failed`. The stack, cause, re-raise sites, and suppressed cleanup failures belong to the `ExceptionContext`, not to the symbol.

## Raising any object

`raise value` accepts any Iris object or value. Each raise creates a fresh `ExceptionContext`. Catching gives you the original raised object unchanged, and optionally the context.

```iris
try {
  raise :tag
} catch value: Symbol, context {
  value
}
```

A typed catch narrows before the catch body runs. Class catches accept subclasses. Contract catches require explicit nominal conformance. A catch-all must come last because catches are tested in source order.

Bare `raise` is different from `raise value`. Bare `raise` continues the active propagation during the synchronous dynamic extent of a catch. Calling it outside that extent raises `NoActiveExceptionError`.

## Try, catch, and finally produce values

A `try` expression takes its provisional value from the try body or from the selected catch. A normally completing `finally` runs after that and discards its own final value.

```iris
let result = try {
  "try value"
} finally {
  "ignored final value"
}

result  // "try value"
```

This is reused from `IRIS-V1-CONTROL-EX011`. If `finally` raises, returns, breaks, or continues, it overrides the pending result or exception. When cleanup fails while another exception is already primary, the cleanup context is appended to the primary context's runtime-owned suppressed list.

## Closeable and using

The standard resource Contract is `Closeable`, with an ordinary `close() -> Nil` Method. It isn't magic syntax. The helper `using(resource) { ... }` runs the block, then closes the resource through `try/finally` equivalent control.

```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

This example is reused from `IRIS-V1-ASYNC-EX003`. It illustrates shape only. Because Iris has no implementation or standard library release, treat `File` here as a specified example surface, not something to open today.

If the block completes and `close()` completes, the helper returns the block value. If the block raises and `close()` also raises, the block's context stays primary and the close context becomes suppressed. If the block completes but `close()` raises, the close failure becomes primary.

## Single-threaded async

Async in Iris is cooperative and single-runtime. Calling an `async fun` returns a `Task<T>`. Awaiting the Task gives the `T`, or repropagates the captured `ExceptionContext` if the Task failed.

```iris
async fun read_name(path: String) -> String {
  return await File.read_text(path)
}

async fun log_name(path: String) -> Nil {
  print(await read_name(path))
}

let task: Task<String> = read_name("user.ir")
let name: String = await task
```

This snippet is reused from `IRIS-V1-ASYNC-EX001`. An async no-result Method returns `Task<Nil>`, not a special empty-result form. Iris v1 has no cancellation semantics, no fire-and-forget signature, and no implicit blocking wait in Iris source.

```iris
async fun value() -> Integer { 7 }

let task = value()
let first = await task
let second = await task
first == second  // true
```

This is reused from `IRIS-V1-ASYNC-EX002`. A completed Task is immutable and can be awaited many times. A failed Task retains one captured failure context; awaiting it observes that same root context with async diagnostic links.

## Static promise, dynamic freedom

Dynamic freedom: any object can be raised, catch selection depends on runtime Class or Contract conformance, and async Tasks complete later through the scheduler.

Static promise: `ExceptionContext` owns propagation metadata, `finally` and `using` have fixed cleanup precedence, `Task<T>` carries an awaited result type, and async interleaving happens only at defined await or scheduler boundaries. Open transactions can't suspend.

## Read the spec

For exact rules, read [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) and [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-CONTROL-C055` through `IRIS-V1-CONTROL-C067` | `raise`, `catch`, `finally`, value rules, and `ExceptionContext`. |
| `IRIS-V1-ASYNC-C003` through `IRIS-V1-ASYNC-C010` | Async callable surface and `Task<T>` result typing. |
| `IRIS-V1-ASYNC-C011` through `IRIS-V1-ASYNC-C019` | Single scheduler, suspension, and no implicit blocking wait. |
| `IRIS-V1-ASYNC-C020` through `IRIS-V1-ASYNC-C029` | Task completion, failed Task propagation, and unobserved failure diagnostics. |
| `IRIS-V1-ASYNC-C030` through `IRIS-V1-ASYNC-C038` | `Closeable`, `using`, idempotent close, and cleanup across async suspension. |
